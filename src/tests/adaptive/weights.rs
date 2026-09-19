//! Tick publication and packet selection must consume the same admission/ranking pass.
use super::*;
use crate::connection::delivery::DataSend;
use crate::connection::route::RouteHealth;
use crate::mode::SchedulingMode;
use crate::sender::housekeeping::HealthTicker;
use crate::telemetry_doc::conns_from_stats;

fn adaptive_config() -> ConfigSnapshot {
    ConfigSnapshot {
        mode: SchedulingMode::Enhanced,
        ..config()
    }
}

#[tokio::test]
async fn tick_weight_matches_selection_when_data_proof_stalls() {
    // Given a previously admitted link with enough unproved DATA to stall this tick.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    state.update_stats(&mut conns, &adaptive_config());
    assert!(stats.get().links[0].effective_multiplier > 0.0);
    for seq in 0..32 {
        conns[0].delivery.record_sent(
            seq,
            DataSend {
                sent_ms: 6000,
                len: 1316,
            },
        );
    }
    // When housekeeping changes health, publish before any further packet arrives.
    HealthTicker::default().tick(&mut conns, &state.targets, 10_000);
    state.update_stats(&mut conns, &adaptive_config());
    // Then the published zero and the real selector agree, even inside cooldown.
    assert_eq!(conns[0].health.state(), HealthState::Stalled);
    assert_eq!(stats.get().links[0].effective_multiplier, 0.0);
    assert_eq!(conns_from_stats(&stats.get())[0].weight_percent, 0);
    assert_eq!(
        adaptive::select(
            &mut conns,
            Some(0),
            10_000,
            10_000,
            &adaptive_config(),
            &mut state
        ),
        Some(1)
    );
}

#[tokio::test]
async fn tick_weight_matches_selection_when_route_degrades() {
    // Given an admitted link that loses its route without losing ACK liveness.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].route_health = RouteHealth::NoDefaultRoute;
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    // When the real health tick observes the route crossing.
    HealthTicker::default().tick(&mut conns, &state.targets, 10_000);
    state.update_stats(&mut conns, &adaptive_config());
    // Then degradation is held out in telemetry and in packet selection.
    assert_eq!(conns[0].health.state(), HealthState::Degraded);
    assert_eq!(stats.get().links[0].effective_multiplier, 0.0);
    assert_eq!(conns_from_stats(&stats.get())[0].weight_percent, 0);
    assert_eq!(pick(&mut conns, &mut state, 10_000), 1);
}

#[tokio::test]
async fn tick_weight_matches_selection_when_deadline_holds_a_healthy_link() {
    // Given a healthy link beyond the negotiated deadline, with positive capacity.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    rtt(&mut conns[0], 600.0);
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    // When publishing without a packet-selection call first.
    state.update_stats(&mut conns, &adaptive_config());
    // Then health alone cannot explain the zero: the actual deadline gate does.
    assert_eq!(conns[0].health.state(), HealthState::Healthy);
    assert!(state.targets[0].deadline_held);
    assert_eq!(stats.get().links[0].effective_multiplier, 0.0);
    assert_eq!(conns_from_stats(&stats.get())[0].weight_percent, 0);
    assert_eq!(pick(&mut conns, &mut state, 10_000), 1);
}

#[tokio::test]
async fn only_the_elected_sole_carrier_has_weight_and_rotation_publishes_without_packets() {
    // Given two stalled links with distinct RTTs and no proof.
    let clock = TestClock::new(10_000);
    let mut conns = pool().await;
    for conn in &mut conns {
        conn.health = HealthMachine::new(HealthState::Stalled, 0);
    }
    rtt(&mut conns[0], 20.0);
    rtt(&mut conns[1], 100.0);
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    state.update_stats(&mut conns, &adaptive_config());
    assert_eq!(
        conns_from_stats(&stats.get())
            .iter()
            .map(|c| c.weight_percent)
            .collect::<Vec<_>>(),
        [100, 0]
    );
    // When the proof deadline expires on an idle housekeeping tick.
    clock.set(12_000);
    state.update_stats(&mut conns, &adaptive_config());
    // Then the newly elected carrier alone has weight and is the next packet's choice.
    assert_eq!(
        conns_from_stats(&stats.get())
            .iter()
            .map(|c| c.weight_percent)
            .collect::<Vec<_>>(),
        [0, 100]
    );
    assert_eq!(pick(&mut conns, &mut state, 12_000), 1);
}

#[tokio::test]
async fn adaptive_zero_capacity_does_not_resurrect_held_links_via_equal_share() {
    // Given one admitted zero-score link and a stalled positive-score link.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].window = 0;
    conns[1].health = HealthMachine::new(HealthState::Stalled, 0);
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    // When the zero-total snapshot is normalized.
    state.update_stats(&mut conns, &adaptive_config());
    // Then there is no invented positive weight for the held link.
    assert_eq!(conns_from_stats(&stats.get())[1].weight_percent, 0);
    assert_eq!(pick(&mut conns, &mut state, 10_000), 0);
}

#[tokio::test]
async fn feature_ablation_changes_the_published_admission_too() {
    // Given a stalled high-capacity link with the stall mechanism ablated.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].health = HealthMachine::new(HealthState::Stalled, 0);
    conns[1].window = 1;
    let stats = SharedStats::new();
    let mut state = AdaptiveState::new(stats.clone());
    let features = AdaptiveFeatures::ALL - AdaptiveFeatures::STALL;
    // When the same configured pipeline publishes and selects.
    state.update_stats(
        &mut conns,
        &ConfigSnapshot {
            features,
            ..adaptive_config()
        },
    );
    // Then stats does not independently veto based on the raw health enum.
    assert!(stats.get().links[0].effective_multiplier > 0.0);
    assert_eq!(
        pick_with_features(&mut conns, &mut state, 10_000, features),
        0
    );
}
