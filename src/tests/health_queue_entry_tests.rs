use HealthState::{Degraded, Down, Healthy, Rejoining, Stalled};

use super::transition_tests::signals;
use super::{HealthConstants, HealthMachine, HealthSignals, HealthState, RecoveryRounds};

#[test]
fn queue_entry_requires_continuous_latched_tau_from_healthy_and_rejoining() {
    // Given: either eligible state, 40ms RTT and queue exactly at entry.
    for initial in [Healthy, Rejoining] {
        let mut machine = HealthMachine::new(initial, 0);
        let k = HealthConstants::default();
        // When: the production entry point observes one uninterrupted episode.
        for (now, expected) in [(100, initial), (1_099, initial), (1_100, Degraded)] {
            machine.step_with_originals(
                &HealthSignals {
                    queue_delay_ms: 10.0,
                    ..signals(now)
                },
                &k,
                RecoveryRounds::default(),
            );
            // Then: only maturity demotes and only an actual rejoin relapse backs off.
            assert_eq!(machine.state(), expected, "{initial:?} at {now}");
            assert_eq!(
                machine.dwell_multiplier(),
                if initial == Rejoining && now == 1_100 {
                    2
                } else {
                    1
                }
            );
        }
    }
}

#[test]
fn queue_entry_gap_resets_pending() {
    // Given: a Healthy link whose first episode is interrupted just below entry.
    let mut machine = HealthMachine::new(Healthy, 0);
    let k = HealthConstants::default();
    // When: queue crosses, clears, and crosses again at the exact boundary.
    for (now, queue, expected) in [
        (100, 10.0, Healthy),
        (1_099, 9.999, Healthy),
        (1_100, 10.0, Healthy),
        (2_099, 10.0, Healthy),
        (2_100, 10.0, Degraded),
    ] {
        machine.step(
            &HealthSignals {
                queue_delay_ms: queue,
                ..signals(now)
            },
            &k,
        );
        // Then: a fresh complete tau is required after the gap.
        assert_eq!(machine.state(), expected, "at {now}");
    }
}

#[test]
fn brief_rejoin_queue_spike_does_not_backoff() {
    // Given: a real 40ms-RTT rejoin at t=0 (a 2000ms ramp).
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(Down, 0);
    machine.step(&signals(0), &k);
    // When: queue remains high for 999ms then clears on the would-be maturity tick.
    for (now, queue, expected) in [
        (100, 10.0, Rejoining),
        (1_099, 10.0, Rejoining),
        (1_100, 9.999, Rejoining),
        (2_000, 0.0, Healthy),
    ] {
        machine.step(
            &HealthSignals {
                queue_delay_ms: queue,
                ..signals(now)
            },
            &k,
        );
        // Then: the original ramp completes without ever doubling dwell.
        assert_eq!(machine.state(), expected, "at {now}");
        assert_eq!(machine.dwell_multiplier(), 1);
    }
}

#[test]
fn pending_queue_does_not_delay_hard_stall_or_loss() {
    // Given: an immature queue episode on a Healthy link.
    let k = HealthConstants::default();
    let mut pending = HealthMachine::new(Healthy, 0);
    let s = HealthSignals {
        queue_delay_ms: 10.0,
        ..signals(100)
    };
    pending.step(&s, &k);
    assert_eq!(pending.state(), Healthy);
    let next = HealthSignals { now_ms: 200, ..s };
    // When: each independent higher-priority failure arrives before queue maturity.
    for (s, expected) in [
        (
            HealthSignals {
                connected: false,
                ..next
            },
            Down,
        ),
        (
            HealthSignals {
                attempts_since_proof: 32,
                proof_age_ms: Some(1_000),
                ..next
            },
            Stalled,
        ),
        (
            HealthSignals {
                loss_ewma: Some(0.10),
                loss_cohort_ok: true,
                ..next
            },
            Degraded,
        ),
        (
            HealthSignals {
                route_health: crate::connection::route::RouteHealth::NoDefaultRoute,
                ..next
            },
            Degraded,
        ),
    ] {
        let mut machine = pending.clone();
        let transition = machine.step(&s, &k).unwrap();
        // Then: it takes effect immediately.
        assert_eq!((transition.to, transition.at_ms), (expected, 200));
        assert!(machine.queue_entry_pending.is_none());
    }
}

#[test]
fn sustained_queue_is_bounded_by_max_tau() {
    // Given: RTT at the 3000ms ceiling and queue exactly at its scaled threshold.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(Healthy, 0);
    // When: qualifying evidence persists for the maximum entry delay.
    for (now, expected) in [(100, Healthy), (3_099, Healthy), (3_100, Degraded)] {
        machine.step(
            &HealthSignals {
                srtt_ms: Some(1_000.0),
                slow_min_rtt_ms: 200.0,
                queue_delay_ms: 50.0,
                ..signals(now)
            },
            &k,
        );
        // Then: no ramp/backoff duration can extend this bound.
        assert_eq!(machine.state(), expected, "at {now}");
    }
}

#[test]
fn queue_entry_tau_is_latched_despite_rtt_changes() {
    // Given: episodes latched at the floor and ceiling, respectively.
    for (initial_rtt, later_rtt, maturity) in [(40.0, 1_000.0, 1_100), (1_000.0, 40.0, 3_100)] {
        let k = HealthConstants::default();
        let mut machine = HealthMachine::new(Healthy, 0);
        machine.step(
            &HealthSignals {
                srtt_ms: Some(initial_rtt),
                queue_delay_ms: 10.0,
                ..signals(100)
            },
            &k,
        );
        // When: RTT changes during the same continuous queue episode.
        for (now, expected) in [
            (1_099, Healthy),
            (maturity - 1, Healthy),
            (maturity, Degraded),
        ] {
            machine.step(
                &HealthSignals {
                    srtt_ms: Some(later_rtt),
                    queue_delay_ms: 10.0,
                    ..signals(now)
                },
                &k,
            );
            // Then: neither a rising nor falling RTT moves the original deadline.
            assert_eq!(
                machine.state(),
                expected,
                "initial RTT {initial_rtt}, at {now}"
            );
        }
    }
}

#[test]
fn queue_entry_survives_rejoining_to_healthy_without_second_grace() {
    // Given: a 2000ms ramp with queue entry beginning 500ms before completion.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(Down, 0);
    machine.step(&signals(0), &k);
    // When: queue stays high across ramp completion.
    for (now, expected) in [
        (1_500, Rejoining),
        (2_000, Healthy),
        (2_499, Healthy),
        (2_500, Degraded),
    ] {
        machine.step(
            &HealthSignals {
                queue_delay_ms: 10.0,
                ..signals(now)
            },
            &k,
        );
        // Then: Healthy inherits the evidence, not a second grace period.
        assert_eq!(machine.state(), expected, "at {now}");
        assert_eq!(machine.dwell_multiplier(), 1);
    }
}
