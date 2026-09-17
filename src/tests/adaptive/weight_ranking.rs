use super::*;
use crate::bind_map::Priority;
use crate::mode::SchedulingMode;
use crate::telemetry_doc::conns_from_stats;

fn cfg() -> ConfigSnapshot {
    ConfigSnapshot {
        mode: SchedulingMode::Adaptive,
        ..config()
    }
}

#[tokio::test]
async fn published_rank_includes_quality_preference_ramp_and_soft_cap() {
    // Given a 307-base link over the floor-32 BDP cap and a 200-base neighbour.
    let _clock = TestClock::new(10_000);
    for (health, multiplier, winner, share) in [
        (HealthState::Healthy, 1.1 * 1.2 * 0.5, 1, 48),
        (HealthState::Rejoining, 1.1 * 0.05 * 0.5, 1, 4),
    ] {
        let mut conns = pool().await;
        conns[0].health = HealthMachine::new(health, 10_000);
        conns[0].in_flight_packets = 64;
        conns[0].priority_baseline = Some(Priority::try_from(0.2).unwrap());
        conns[1].window = 200;
        let stats = SharedStats::new();
        let mut state = AdaptiveState::new(stats.clone());
        // When one scheduler pass supplies both the rank and its published cache.
        state.update_stats(&mut conns, &cfg());
        let snapshot = stats.get();
        // Then independent numeric expectations pin every multiplier and normalization.
        assert_eq!(snapshot.links[0].base_score, 307);
        assert!((snapshot.links[0].effective_multiplier - multiplier).abs() < 1e-12);
        assert_eq!(conns_from_stats(&snapshot)[0].weight_percent, share);
        assert_eq!(pick(&mut conns, &mut state, 10_000), winner);
    }
}

#[tokio::test]
async fn stats_read_preserves_the_packet_selection_epoch_and_cached_quality() {
    // Given a live packet selection whose quality cache differs from a fresh calculation.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].quality_cache.last_calculated_ms = 10_000;
    conns[0].quality_cache.multiplier = 0.7;
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    pick(&mut conns, &mut state, 10_000);
    conns[0].window = 1;
    // When the read-only projection runs after packet state has moved on.
    stats.update(&conns, &cfg());
    // Then it uses the selector's actual cached pair, not an independently recomputed rank.
    let snapshot = stats.get();
    assert_eq!(snapshot.links[0].base_score, 20_000);
    assert_eq!(snapshot.links[0].quality_multiplier, 0.7);
    assert_eq!(snapshot.links[0].effective_multiplier, 0.7);
    assert_eq!(conns_from_stats(&snapshot)[0].weight_percent, 39);
}

#[tokio::test]
async fn deadline_readmission_is_published_on_the_same_tick_as_selection() {
    // Given a deadline hold followed by continuous below-threshold clearance.
    let clock = TestClock::new(10_000);
    let mut conns = pool().await;
    rtt(&mut conns[0], 600.0);
    conns[1].window = 1;
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    state.update_stats(&mut conns, &cfg());
    conns[0].rtt = Default::default();
    rtt(&mut conns[0], 100.0);
    // When idle ticks advance the actual hysteresis through its 1000ms clearance.
    for (now, held, winner) in [(10_100, true, 1), (11_099, true, 1), (11_100, false, 0)] {
        clock.set(now);
        state.update_stats(&mut conns, &cfg());
        // Then no packet is needed to release the telemetry weight, and selection agrees.
        assert_eq!(stats.get().links[0].effective_multiplier == 0.0, held);
        assert_eq!(pick(&mut conns, &mut state, now), winner);
    }
}

#[tokio::test]
async fn connected_fallback_publishes_only_its_base_ranked_carrier() {
    // Given two stalled links while the sole-carrier mechanism is ablated.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    for conn in &mut conns {
        conn.health = HealthMachine::new(HealthState::Stalled, 0);
    }
    conns[0].window = 1;
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    let features = AdaptiveFeatures::ALL - AdaptiveFeatures::SOLE;
    // When the existing connected-pool escape is used.
    state.update_stats(&mut conns, &ConfigSnapshot { features, ..cfg() });
    // Then fallback ordering stays base-only, without reviving every held neighbour.
    let snapshot = stats.get();
    assert_eq!(snapshot.links[0].effective_multiplier, 0.0);
    assert_eq!(snapshot.links[1].effective_multiplier, 1.0);
    assert_eq!(conns_from_stats(&snapshot)[1].weight_percent, 100);
    assert_eq!(
        pick_with_features(&mut conns, &mut state, 10_000, features),
        1
    );
}

#[tokio::test]
async fn recovery_resets_cached_weights_and_legacy_modes_bypass_them() {
    // Given a cached adaptive admission followed by socket recovery.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    state.update_stats(&mut conns, &cfg());
    conns[0].mark_for_recovery();
    // When a stats read occurs before the next selector refresh.
    stats.update(&conns, &cfg());
    // Then the old generation cannot retain weight, and legacy modes ignore the cache.
    assert_eq!(stats.get().links[0].effective_multiplier, 0.0);
    for mode in [
        SchedulingMode::Classic,
        SchedulingMode::Enhanced,
        SchedulingMode::RttThreshold,
        SchedulingMode::Edpf,
    ] {
        let legacy = ConfigSnapshot { mode, ..config() };
        state.update_stats(&mut conns, &legacy);
        let snapshot = stats.get();
        assert_eq!(
            snapshot.links[1].effective_multiplier,
            snapshot.links[1].quality_multiplier
        );
    }
}
