use HealthState::{Degraded, Down, Healthy, Rejoining, Stalled};

use super::{HealthConstants, HealthMachine, HealthSignals, HealthState, Transition};

pub(super) const STATES: [HealthState; 5] = [Healthy, Degraded, Stalled, Rejoining, Down];

pub(super) fn signals(now_ms: u64) -> HealthSignals {
    HealthSignals {
        connected: true,
        socket_valid: true,
        iface_present: true,
        route_health: crate::connection::route::RouteHealth::Unknown,
        attempts_since_proof: 0,
        proof_age_ms: Some(0),
        srtt_ms: Some(40.0),
        loss_ewma: Some(0.0),
        loss_cohort_ok: true,
        last_cohort_ms: Some(now_ms),
        queue_delay_ms: 0.0,
        slow_min_rtt_ms: 40.0,
        probe_rounds_ok: 0,
        probe_rounds_started_ms: None,
        probe_loss: None,
        held_links: 1,
        now_ms,
    }
}

#[test]
fn hard_failure_dominates_from_every_state() {
    // Given: each state and each independent hard-failure axis (15 cases).
    for state in STATES {
        for axis in 0..3 {
            let mut machine = HealthMachine::new(state, 0);
            let mut s = signals(10_000);
            s.attempts_since_proof = 32;
            s.proof_age_ms = Some(10_000);
            s.loss_ewma = Some(1.0);
            match axis {
                0 => s.connected = false,
                1 => s.socket_valid = false,
                2 => s.iface_present = false,
                _ => unreachable!(),
            }
            // When: a hard failure competes with soft failures.
            let transition = machine.step(&s, &HealthConstants::default());
            // Then: Down wins, and repeated Down does not emit a transition.
            assert_eq!(machine.state(), Down);
            assert_eq!(
                transition,
                (state != Down).then_some(Transition {
                    from: state,
                    to: Down,
                    at_ms: 10_000
                })
            );
        }
    }
}

#[test]
fn transition_and_hysteresis_table() {
    // Given: threshold equality, both gaps, absence, and competing predicates.
    let base = signals(1_000);
    let stall = HealthSignals {
        attempts_since_proof: 32,
        proof_age_ms: Some(1_000),
        ..base
    };
    let loss = HealthSignals {
        loss_ewma: Some(0.10),
        ..base
    };
    let queue = HealthSignals {
        queue_delay_ms: 10.0,
        ..base
    };
    let cases = [
        ("healthy idle", Healthy, base, Healthy),
        ("healthy stalls", Healthy, stall, Stalled),
        ("degraded stalls", Degraded, stall, Stalled),
        ("rejoining stalls", Rejoining, stall, Stalled),
        ("healthy loss", Healthy, loss, Degraded),
        ("rejoining loss", Rejoining, loss, Degraded),
        ("healthy queue", Healthy, queue, Degraded),
        ("rejoining queue", Rejoining, queue, Degraded),
        (
            "stall wins",
            Healthy,
            HealthSignals {
                loss_ewma: Some(0.5),
                ..stall
            },
            Stalled,
        ),
        (
            "31 attempts never stall",
            Healthy,
            HealthSignals {
                attempts_since_proof: 31,
                proof_age_ms: Some(10_000),
                ..base
            },
            Healthy,
        ),
        (
            "young proof",
            Healthy,
            HealthSignals {
                proof_age_ms: Some(999),
                ..stall
            },
            Healthy,
        ),
        (
            "unknown proof age",
            Healthy,
            HealthSignals {
                proof_age_ms: None,
                ..stall
            },
            Healthy,
        ),
        (
            "no RTT uses 3000",
            Healthy,
            HealthSignals {
                srtt_ms: None,
                ..stall
            },
            Healthy,
        ),
        (
            "no RTT threshold",
            Healthy,
            HealthSignals {
                srtt_ms: None,
                proof_age_ms: Some(3_000),
                ..stall
            },
            Stalled,
        ),
        (
            "loss enter gap",
            Healthy,
            HealthSignals {
                loss_ewma: Some(0.099),
                ..base
            },
            Healthy,
        ),
        (
            "subfloor loss",
            Healthy,
            HealthSignals {
                loss_cohort_ok: false,
                ..loss
            },
            Healthy,
        ),
        (
            "absent loss",
            Healthy,
            HealthSignals {
                loss_ewma: None,
                ..base
            },
            Healthy,
        ),
        (
            "queue enter gap",
            Healthy,
            HealthSignals {
                queue_delay_ms: 9.999,
                ..base
            },
            Healthy,
        ),
        (
            "slow queue scales",
            Healthy,
            HealthSignals {
                slow_min_rtt_ms: 200.0,
                ..queue
            },
            Healthy,
        ),
        (
            "one round cannot rejoin",
            Stalled,
            HealthSignals {
                probe_rounds_ok: 1,
                probe_rounds_started_ms: Some(0),
                ..base
            },
            Stalled,
        ),
        (
            "two rounds rejoin",
            Stalled,
            HealthSignals {
                probe_rounds_ok: 2,
                probe_rounds_started_ms: Some(0),
                ..base
            },
            Rejoining,
        ),
        (
            "undated rounds",
            Stalled,
            HealthSignals {
                probe_rounds_ok: 2,
                ..base
            },
            Stalled,
        ),
        ("restored link ramps", Down, base, Rejoining),
        ("ramp not done", Rejoining, base, Rejoining),
    ];
    for (name, from, s, to) in cases {
        let mut machine = HealthMachine::new(from, 0);
        // When: the observation is applied.
        let result = machine.step(&s, &HealthConstants::default());
        // Then: exactly the expected edge (or no edge) is reported.
        assert_eq!(machine.state(), to, "{name}");
        assert_eq!(
            result,
            (from != to).then_some(Transition {
                from,
                to,
                at_ms: s.now_ms
            }),
            "{name}"
        );
        assert_eq!(
            machine.entered_at_ms(),
            if from == to { 0 } else { s.now_ms }
        );
        assert_eq!(machine.last_transition_ms(), machine.entered_at_ms());
    }
}
