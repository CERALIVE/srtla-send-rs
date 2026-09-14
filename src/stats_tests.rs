use super::*;
use crate::mode::SchedulingMode;

#[tokio::test]
async fn scheduler_observations_follow_mode_and_effective_priority() {
    // Given a priority override distinct from the sidecar baseline.
    let mut conns = crate::test_helpers::create_test_connections(2).await;
    conns[0].priority_baseline = Some(crate::bind_map::Priority::try_from(-0.2).unwrap());
    conns[0].priority_override_conn = Some(crate::bind_map::Priority::try_from(0.2).unwrap());
    let stats = SharedStats::new();
    // When snapshots are published in each mode.
    for mode in [
        SchedulingMode::Adaptive,
        SchedulingMode::Classic,
        SchedulingMode::Enhanced,
        SchedulingMode::RttThreshold,
        SchedulingMode::Edpf,
    ] {
        stats.update(
            &conns,
            &ConfigSnapshot {
                mode,
                ..test_config()
            },
        );
        let links = stats.get().links;
        // Then health is adaptive-only, but configured priority is echoed even when inactive.
        assert_eq!(
            links[0].health,
            matches!(mode, SchedulingMode::Adaptive).then_some("down")
        );
        assert_eq!(links[0].priority, Some(0.2));
        assert_eq!(links[1].priority, None);
    }
}

#[test]
fn negotiated_latency_is_unknown_until_observed() {
    // Given a fresh container, When read directly, Then no latency is invented.
    assert_eq!(SharedStats::new().negotiated_latency_ms(), None);
    assert_eq!(SharedStats::default().negotiated_latency_ms(), None);
}

#[test]
fn negotiated_latency_is_shared_and_independent_of_snapshot_locks() {
    // Given all snapshot locks held, When a clone publishes, Then atomic read still works.
    let stats = SharedStats::new();
    let _snapshot = stats.inner.write().unwrap();
    let _mode = stats.bind_map.write().unwrap();
    let _bytes = stats.session_bytes.lock().unwrap();
    stats.clone().set_negotiated_latency_ms(2000);
    assert_eq!(stats.negotiated_latency_ms(), Some(2000));
}

#[test]
fn negotiated_latency_survives_housekeeping() {
    // Given a prior observation, When housekeeping rebuilds stats, Then it is retained.
    let stats = SharedStats::new();
    stats.set_negotiated_latency_ms(2000);
    stats.update(&[], &test_config());
    assert_eq!(stats.negotiated_latency_ms(), Some(2000));
}

#[test]
fn negotiated_zero_latency_means_unknown() {
    // Given an earlier observation, When a zero delay is observed, Then expose unknown.
    let stats = SharedStats::new();
    stats.set_negotiated_latency_ms(500);
    stats.set_negotiated_latency_ms(0);
    assert_eq!(stats.negotiated_latency_ms(), None);
}

#[test]
fn test_shared_stats_new() {
    let stats = SharedStats::new();
    let snapshot = stats.get();
    assert_eq!(snapshot.active_links, 0);
    assert_eq!(snapshot.total_links, 0);
}

#[test]
fn test_shared_stats_empty_update() {
    let stats = SharedStats::new();
    let config = ConfigSnapshot {
        mode: SchedulingMode::Enhanced,
        quality_enabled: true,
        exploration_enabled: false,
        rtt_delta_ms: 30,
        earned_ack_window: false,
        stall_deselect: false,
        stall_min_in_flight: 32,
        stall_ack_stale_ms: 3000,
        stall_reprobe_ms: 1000,
    };
    stats.update(&[], &config);
    let snapshot = stats.get();
    assert_eq!(snapshot.mode, "enhanced");
    assert!(snapshot.quality_enabled);
}

#[test]
fn test_to_json_contains_expected_fields() {
    let stats = SharedStats::new();
    let json = stats.to_json();
    assert!(json.contains("\"mode\""));
    assert!(json.contains("\"active_links\""));
    assert!(json.contains("\"total_window\""));
    assert!(json.contains("\"links\""));
    assert!(json.contains("\"session_bytes_sent\""));
}

// ---- ADR-002 session-bytes accumulator --------------------------------

#[test]
fn session_bytes_sums_live_links() {
    let mut acc = SessionBytes::default();
    assert_eq!(acc.observe_totals([(7, 1_000), (9, 500)]), 1_500);
}

#[test]
fn session_bytes_banks_only_the_delta_between_observations() {
    let mut acc = SessionBytes::default();
    acc.observe_totals([(7, 1_000)]);
    assert_eq!(
        acc.observe_totals([(7, 1_600)]),
        1_600,
        "a link's own counter is cumulative, so re-observing it must add 600, not 1600"
    );
}

#[test]
fn session_bytes_survives_a_link_teardown() {
    // A SIGHUP reload that drops an uplink must not take its bytes with it —
    // this is the regression a naive `links.map(total).sum()` would ship.
    let mut acc = SessionBytes::default();
    acc.observe_totals([(7, 1_000), (9, 500)]);

    assert_eq!(acc.observe_totals([(7, 1_000)]), 1_500);
}

#[test]
fn session_bytes_counts_a_readded_link_from_zero() {
    // A re-added IP comes back as a NEW connection (fresh conn_id, counter at
    // 0). Its bytes must accrue on top of the banked total, never replace it.
    let mut acc = SessionBytes::default();
    acc.observe_totals([(7, 1_000)]);
    acc.observe_totals([]);

    assert_eq!(acc.observe_totals([(11, 300)]), 1_300);
}

#[test]
fn session_bytes_never_regresses_on_a_backwards_link_counter() {
    // Per-link counters are monotonic by construction; if one ever went
    // backwards the bond total must still refuse to shrink.
    let mut acc = SessionBytes::default();
    acc.observe_totals([(7, 1_000)]);

    assert_eq!(acc.observe_totals([(7, 400)]), 1_000);
}

#[test]
fn session_bytes_forgets_departed_links() {
    // Bookkeeping for a link that is gone must not accumulate across a long
    // session of SIGHUP churn.
    let mut acc = SessionBytes::default();
    acc.observe_totals([(1, 10), (2, 10), (3, 10)]);
    acc.observe_totals([(3, 10)]);

    assert_eq!(acc.last_seen.len(), 1);
    assert_eq!(acc.total, 30);
}

#[test]
fn empty_update_reports_zero_session_bytes() {
    let stats = SharedStats::new();
    stats.update(&[], &test_config());
    assert_eq!(stats.get().session_bytes_sent, 0);
}

fn test_config() -> ConfigSnapshot {
    ConfigSnapshot {
        mode: SchedulingMode::Enhanced,
        quality_enabled: true,
        exploration_enabled: false,
        rtt_delta_ms: 30,
        earned_ack_window: false,
        stall_deselect: false,
        stall_min_in_flight: 32,
        stall_ack_stale_ms: 3000,
        stall_reprobe_ms: 1000,
    }
}
