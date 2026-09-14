use HealthState::{Down, Healthy, Rejoining, Stalled};
use proptest::prelude::*;

use super::transition_tests::{STATES, signals};
use super::{HealthConstants, HealthMachine, HealthSignals, HealthState};

#[test]
fn constants_and_state_names_match_the_contract() {
    // Given / When: the default tuning and public state projections.
    let k = HealthConstants::default();
    // Then: fixed defaults and all derived boundary values match the specification.
    assert_eq!(
        (
            k.stall_attempts,
            k.loss_cohort_min_sends,
            k.loss_stale_after_ms
        ),
        (32, 100, 10_000)
    );
    assert_eq!((k.loss_enter, k.loss_clear), (0.10, 0.05));
    assert_eq!(
        (
            k.rejoin_rounds,
            k.probe_train_len,
            k.probe_max_pps,
            k.dwell_backoff_max
        ),
        (2, 10, 10, 16)
    );
    for (rtt, tau) in [
        (None, 3000.0),
        (Some(40.0), 1000.0),
        (Some(500.0), 2000.0),
        (Some(1000.0), 3000.0),
    ] {
        assert_eq!(k.stall_tau(rtt), tau);
    }
    assert_eq!((k.queue_enter(40.0), k.queue_clear(40.0)), (10.0, 5.0));
    assert_eq!((k.queue_enter(200.0), k.queue_clear(200.0)), (50.0, 25.0));
    assert_eq!(k.train_period_ms(3), 3000.0);
    assert_eq!(k.rejoin_span(Some(40.0), 3), 6040.0);
    assert_eq!(
        STATES.map(HealthState::as_str),
        ["healthy", "degraded", "stalled", "rejoining", "down"]
    );
}

#[test]
fn relapsed_ramp_uses_scaled_span() {
    // Given: a genuine rejoin followed by a queue relapse, with three held links.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(Down, 0);
    let s = HealthSignals {
        held_links: 3,
        ..signals(0)
    };
    machine.step(&s, &k);
    machine.step(
        &HealthSignals {
            now_ms: 100,
            queue_delay_ms: 10.0,
            ..s
        },
        &k,
    );
    machine.step(
        &HealthSignals {
            now_ms: 1_100,
            queue_delay_ms: 10.0,
            ..s
        },
        &k,
    );
    machine.step(&HealthSignals { now_ms: 1_200, ..s }, &k);
    machine.step(&HealthSignals { now_ms: 2_200, ..s }, &k);
    assert_eq!(machine.state(), Rejoining);
    assert_eq!(machine.dwell_multiplier(), 2);
    // When: the doubled 6000ms ramp advances.
    for (now, expected) in [(2_200, 0.05), (5_200, 0.525), (8_200, 1.0)] {
        assert!((machine.ramp_multiplier(now, 3) - expected).abs() < 1e-12);
    }
    machine.step(&HealthSignals { now_ms: 8_199, ..s }, &k);
    assert_eq!(machine.state(), Rejoining);
    machine.step(&HealthSignals { now_ms: 8_200, ..s }, &k);
    // Then: Healthy entry coincides exactly with ramp completion and resets dwell.
    assert_eq!(machine.state(), Healthy);
    assert_eq!(machine.dwell_multiplier(), 1);
}

#[test]
fn repeated_relapses_cap_backoff_and_down_does_not_double_it() {
    // Given: a restored connection alternating probe recovery and stalled relapse.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(Down, 0);
    machine.step(&signals(0), &k);
    // When: six consecutive relapses occur before any completed healthy ramp.
    for (round, expected) in [2, 4, 8, 16, 16, 16].into_iter().enumerate() {
        let now = u64::try_from(round).unwrap() * 2_000 + 1_000;
        machine.step(
            &HealthSignals {
                attempts_since_proof: 32,
                proof_age_ms: Some(1_000),
                ..signals(now)
            },
            &k,
        );
        // Then: doubling saturates at 16, including on stalled relapses.
        assert_eq!(machine.state(), Stalled);
        assert_eq!(machine.dwell_multiplier(), expected);
        machine.step(
            &HealthSignals {
                probe_rounds_ok: 2,
                probe_rounds_started_ms: Some(now),
                ..signals(now + 1)
            },
            &k,
        );
    }
    machine.step(
        &HealthSignals {
            connected: false,
            ..signals(20_000)
        },
        &k,
    );
    assert_eq!(machine.state(), Down);
    assert_eq!(machine.dwell_multiplier(), 16);
    assert_eq!(machine.ramp_multiplier(20_000, 3), 1.0);
}

#[test]
fn ramp_uses_entry_tuning_and_current_held_link_count() {
    // Given: nondefault probe cadence and RTT, captured when the ramp starts.
    let k = HealthConstants {
        probe_max_pps: 5,
        ..HealthConstants::default()
    };
    let mut machine = HealthMachine::new(Down, 100);
    machine.step(
        &HealthSignals {
            srtt_ms: Some(500.0),
            ..signals(100)
        },
        &k,
    );
    // When: RTT/tuning change mid-ramp and the held set grows to three links.
    machine.step(
        &HealthSignals {
            held_links: 3,
            ..signals(4_100)
        },
        &HealthConstants::default(),
    );
    // Then: both APIs retain the entry tuning and agree on the live 6000ms span.
    assert_eq!(machine.state(), Rejoining);
    assert_eq!(machine.ramp_multiplier(0, 3), 0.05);
    assert!((machine.ramp_multiplier(3_100, 3) - 0.525).abs() < 1e-12);
    machine.step(
        &HealthSignals {
            held_links: 3,
            ..signals(6_100)
        },
        &HealthConstants::default(),
    );
    assert_eq!(machine.state(), Healthy);
}

#[test]
fn hard_failure_preserves_backoff_below_the_cap() {
    // Given: one relapse, far below the cap so accidental doubling is observable.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(Down, 0);
    machine.step(&signals(0), &k);
    machine.step(
        &HealthSignals {
            attempts_since_proof: 32,
            proof_age_ms: Some(1_000),
            ..signals(1_000)
        },
        &k,
    );
    machine.step(
        &HealthSignals {
            probe_rounds_ok: 2,
            probe_rounds_started_ms: Some(1_000),
            ..signals(1_001)
        },
        &k,
    );
    assert_eq!(machine.dwell_multiplier(), 2);
    // When: a rejoining link loses its socket.
    machine.step(
        &HealthSignals {
            socket_valid: false,
            ..signals(1_002)
        },
        &k,
    );
    // Then: hard failure is not a soft relapse.
    assert_eq!(machine.state(), Down);
    assert_eq!(machine.dwell_multiplier(), 2);
}

proptest! {
    #[test]
    fn health_trace_preserves_state_and_ramp_invariants(
        initial in prop::sample::select(STATES.to_vec()),
        events in prop::collection::vec((0u64..10_000, any::<bool>(), 0u32..64,
            prop::option::of(0u64..20_000), 0u32..5, 0u32..101, 0u32..101, 0u32..5), 1..100)
    ) {
        // Given: a real machine and arbitrary time-ordered observations.
        let mut machine = HealthMachine::new(initial, 0);
        let mut now = 0;
        for (delta, connected, attempts, age, rounds, loss, queue, held) in events {
            now += delta;
            let s = HealthSignals { connected, attempts_since_proof: attempts, proof_age_ms: age,
                probe_rounds_ok: rounds, probe_rounds_started_ms: Some(now.saturating_sub(2_000)),
                loss_ewma: Some(f64::from(loss) / 100.0), queue_delay_ms: f64::from(queue),
                held_links: held, ..signals(now) };
            let before = machine.state();
            // When: each generated observation drives the production transition function.
            let transition = machine.step(&s, &HealthConstants::default());
            // Then: domain, edge, dwell, and ramp invariants always hold.
            prop_assert!(STATES.contains(&machine.state()));
            prop_assert!(!(before == Stalled && machine.state() == Healthy));
            prop_assert_eq!(transition.is_some(), before != machine.state());
            prop_assert!((1..=16).contains(&machine.dwell_multiplier()));
            prop_assert!((0.05..=1.0).contains(&machine.ramp_multiplier(now, held)));
        }
    }
}
