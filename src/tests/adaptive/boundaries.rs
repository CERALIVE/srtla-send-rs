use super::*;
use crate::connection::adaptive::{DeadlineBudget, DeadlineGate};

#[tokio::test]
async fn queue_ablation_removes_queue_delay_from_deadline_prediction() {
    // Given rising raw RTT minima but a smoothed RTT still below the deadline.
    let clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].rtt.update_estimate(20);
    clock.set(11_001);
    conns[0].rtt.update_estimate(500);
    conns[0].rtt.kalman_rtt.reset();
    rtt(&mut conns[0], 100.0);
    conns[1].window = 100;
    let mut state = AdaptiveState::default();
    assert_eq!(pick(&mut conns, &mut state, 11_001), 1);
    let features = AdaptiveFeatures::ALL - AdaptiveFeatures::QUEUE;
    // When queue is ablated and the clear dwell elapses, then the link is readmitted.
    assert_eq!(
        pick_with_features(&mut conns, &mut state, 11_001, features),
        1
    );
    clock.set(12_001);
    assert_eq!(
        pick_with_features(&mut conns, &mut state, 12_001, features),
        0
    );
}

#[test]
fn deadline_readmission_requires_continuous_strict_clearance() {
    // Given a 500ms budget with tau=1000ms and observations spanning both boundaries.
    let mut gate = DeadlineGate::default();
    let trace = [
        (0, 250.0, false),
        (1, 251.0, true),
        (2, 199.0, true),
        (1001, 199.0, true),
        (1002, 200.0, true),
        (1003, 199.0, true),
        (2002, 199.0, true),
        (2003, 199.0, false),
    ];
    // When each observation is applied, then the equality gap resets clear dwell.
    for (now_ms, predicted, held) in trace {
        assert_eq!(
            gate.update(
                predicted,
                DeadlineBudget {
                    latency_ms: 500,
                    tau_ms: 1000.0,
                    now_ms
                }
            ),
            held
        );
    }
}

#[tokio::test]
async fn unknown_rtt_cannot_prove_a_twice_as_fast_challenger() {
    // Given two Stalled links with no RTT sample; their proof deadline is 3000ms.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    for conn in &mut conns {
        conn.health = HealthMachine::new(HealthState::Stalled, 0);
    }
    let mut state = AdaptiveState::default();
    assert_eq!(pick(&mut conns, &mut state, 10_000), 0);
    // When the minimum hold expires but proof time remains, then unknown is not faster.
    assert_eq!(pick(&mut conns, &mut state, 12_000), 0);
}

#[tokio::test]
async fn sole_hard_failure_overrides_minimum_hold() {
    // Given a just-elected carrier that loses transport eligibility.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    for conn in &mut conns {
        conn.health = HealthMachine::new(HealthState::Stalled, 0);
    }
    let mut state = AdaptiveState::default();
    assert_eq!(pick(&mut conns, &mut state, 10_000), 0);
    conns[0].connected = false;
    // When the next packet arrives, then hard failure bypasses the two-second hold.
    assert_eq!(pick(&mut conns, &mut state, 10_001), 1);
}

#[tokio::test]
async fn sole_identity_survives_pool_reorder_without_restarting_hold() {
    // Given an incumbent followed by a positional reorder of the same connections.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    for conn in &mut conns {
        conn.health = HealthMachine::new(HealthState::Stalled, 0);
    }
    rtt(&mut conns[0], 20.0);
    rtt(&mut conns[1], 100.0);
    let mut state = AdaptiveState::default();
    assert_eq!(pick(&mut conns, &mut state, 10_000), 0);
    conns.swap(0, 1);
    // When selected again, then the stored identity maps to its new index and old time.
    assert_eq!(pick(&mut conns, &mut state, 10_001), 1);
    assert_eq!(state.sole_carrier, Some((1, 10_000)));
}

#[tokio::test]
async fn adaptive_feature_ablations_change_their_own_decisions() {
    // Given a separate distinguishing pool for each ranking/admission mechanism.
    let _clock = TestClock::new(10_000);
    for feature in [
        AdaptiveFeatures::LOSS,
        AdaptiveFeatures::REJOIN,
        AdaptiveFeatures::RATECAP,
        AdaptiveFeatures::DEADLINE,
        AdaptiveFeatures::PREF,
        AdaptiveFeatures::SOLE,
    ] {
        let mut conns = pool().await;
        let (enabled, disabled) = if feature == AdaptiveFeatures::LOSS {
            conns[0].health = HealthMachine::new(HealthState::Degraded, 0);
            conns[1].window = 100;
            (1, 0)
        } else if feature == AdaptiveFeatures::REJOIN {
            conns[0].health = HealthMachine::new(HealthState::Rejoining, 10_000);
            conns[1].window = 2000;
            (1, 0)
        } else if feature == AdaptiveFeatures::RATECAP {
            conns[0].window = 40_000;
            conns[0].in_flight_packets = 64;
            conns[1].window = 400;
            (1, 0)
        } else if feature == AdaptiveFeatures::DEADLINE {
            rtt(&mut conns[0], 600.0);
            conns[1].window = 100;
            (1, 0)
        } else if feature == AdaptiveFeatures::PREF {
            conns[0].window = 19_999;
            conns[0].priority_baseline = Some(crate::bind_map::Priority::try_from(0.2).unwrap());
            (0, 1)
        } else {
            for conn in &mut conns {
                conn.health = HealthMachine::new(HealthState::Stalled, 0);
            }
            rtt(&mut conns[0], 100.0);
            rtt(&mut conns[1], 20.0);
            (1, 0)
        };
        let mut state = AdaptiveState::default();
        assert_eq!(pick(&mut conns, &mut state, 10_000), enabled);
        // When only this bit is removed, then its distinct control decision changes.
        let features = AdaptiveFeatures::ALL - feature;
        assert_eq!(
            pick_with_features(&mut conns, &mut state, 10_000, features),
            disabled,
            "feature={feature:?}"
        );
    }
}
