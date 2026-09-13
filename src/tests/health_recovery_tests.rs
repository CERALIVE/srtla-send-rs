use HealthState::{Degraded, Healthy, Rejoining, Stalled};

use super::transition_tests::signals;
use super::{HealthConstants, HealthMachine, HealthSignals, HealthState};

#[test]
fn degraded_by_loss_recovers_on_probe_evidence_when_ewma_is_stale() {
    // Given: real loss demotion; no normal DATA cohorts after demotion.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(Healthy, 0);
    let s = HealthSignals {
        loss_ewma: Some(0.20),
        ..signals(0)
    };
    machine.step(&s, &k);
    assert_eq!(machine.state(), Degraded);
    let clear = HealthSignals {
        now_ms: 10_000,
        loss_cohort_ok: false,
        probe_loss: Some(0.05),
        ..s
    };
    // When: probe-supported clearance persists for tau, leaving EWMA unchanged.
    machine.step(&clear, &k);
    machine.step(
        &HealthSignals {
            now_ms: 10_999,
            ..clear
        },
        &k,
    );
    assert_eq!(machine.state(), Degraded);
    machine.step(
        &HealthSignals {
            now_ms: 11_000,
            ..clear
        },
        &k,
    );
    // Then: stale high normal loss cannot deadlock or immediately re-demote recovery.
    assert_eq!(machine.state(), Rejoining);
    machine.step(
        &HealthSignals {
            now_ms: 13_000,
            ..clear
        },
        &k,
    );
    assert_eq!(machine.state(), Healthy);
}

#[test]
fn loss_clearance_table_requires_the_right_evidence() {
    // Given: a loss-triggered demotion and fresh/stale/absent alternatives.
    let cases = [
        ("fresh low", Some(0.05), Some(10_000), None, true),
        (
            "loss clear gap",
            Some(0.051),
            Some(10_000),
            Some(0.0),
            false,
        ),
        (
            "fresh high beats probe",
            Some(0.20),
            Some(10_000),
            Some(0.0),
            false,
        ),
        ("just fresh", Some(0.20), Some(1_001), Some(0.0), false),
        ("stale at boundary", Some(0.20), Some(0), Some(0.05), true),
        ("stale low needs proof", Some(0.0), Some(0), None, false),
        ("probe clear gap", Some(0.20), Some(0), Some(0.051), false),
        ("absent EWMA probe", None, None, Some(0.0), true),
        ("absent everything", None, None, None, false),
        ("undated EWMA", Some(0.0), None, None, false),
    ];
    for (name, loss_ewma, last_cohort_ms, probe_loss, rejoins) in cases {
        let mut machine = HealthMachine::new(Healthy, 0);
        let k = HealthConstants::default();
        machine.step(
            &HealthSignals {
                loss_ewma: Some(0.2),
                ..signals(0)
            },
            &k,
        );
        let s = HealthSignals {
            loss_ewma,
            last_cohort_ms,
            probe_loss,
            loss_cohort_ok: false,
            ..signals(10_000)
        };
        // When: the candidate clear condition is observed for one full tau.
        machine.step(&s, &k);
        machine.step(
            &HealthSignals {
                now_ms: 11_000,
                ..s
            },
            &k,
        );
        // Then: only supported low-loss evidence permits recovery.
        assert_eq!(
            machine.state(),
            if rejoins { Rejoining } else { Degraded },
            "{name}"
        );
    }
}

#[test]
fn queue_clearance_requires_continuous_dwell_and_both_latched_causes() {
    // Given: queue demotion with no loss samples (and a second case with loss).
    for loss_ewma in [None, Some(0.2)] {
        let k = HealthConstants::default();
        let mut machine = HealthMachine::new(Healthy, 0);
        machine.step(
            &HealthSignals {
                queue_delay_ms: 10.0,
                loss_ewma,
                ..signals(0)
            },
            &k,
        );
        let clear = HealthSignals {
            queue_delay_ms: 5.0,
            loss_ewma,
            last_cohort_ms: Some(0),
            ..signals(1_000)
        };
        // When: a gap interrupts clearance, followed by a full clear dwell.
        machine.step(&clear, &k);
        machine.step(
            &HealthSignals {
                now_ms: 1_500,
                queue_delay_ms: 5.001,
                ..clear
            },
            &k,
        );
        machine.step(
            &HealthSignals {
                now_ms: 2_000,
                ..clear
            },
            &k,
        );
        machine.step(
            &HealthSignals {
                now_ms: 2_999,
                ..clear
            },
            &k,
        );
        assert_eq!(machine.state(), Degraded);
        machine.step(
            &HealthSignals {
                now_ms: 3_000,
                ..clear
            },
            &k,
        );
        // Then: the gap resets dwell; queue recovery cannot override latched loss.
        assert_eq!(
            machine.state(),
            if loss_ewma.is_none() {
                Rejoining
            } else {
                Degraded
            }
        );
    }
}

#[test]
fn three_held_links_rejoin_within_feasible_span() {
    // Given: three links sharing 10pps, with a 40ms RTT.
    let k = HealthConstants::default();
    let mut machine = HealthMachine::new(Stalled, 0);
    let s = HealthSignals {
        held_links: 3,
        probe_rounds_started_ms: Some(0),
        ..signals(3_000)
    };
    // When: the two trains finish at 3000 and 6000ms.
    machine.step(
        &HealthSignals {
            probe_rounds_ok: 1,
            ..s
        },
        &k,
    );
    assert_eq!(machine.state(), Stalled);
    machine.step(
        &HealthSignals {
            now_ms: 6_000,
            probe_rounds_ok: 2,
            ..s
        },
        &k,
    );
    // Then: the feasible 6040ms window admits the link.
    assert_eq!(k.rejoin_span(s.srtt_ms, s.held_links), 6040.0);
    assert_eq!(machine.state(), Rejoining);
}

#[test]
fn expired_probe_window_does_not_permanently_trap_a_stalled_link() {
    // Given: evidence exactly at and just beyond the feasible window boundary.
    for (now, expected) in [(6_040, Rejoining), (6_041, Stalled)] {
        let mut machine = HealthMachine::new(Stalled, 0);
        let s = HealthSignals {
            held_links: 3,
            probe_rounds_ok: 2,
            probe_rounds_started_ms: Some(0),
            ..signals(now)
        };
        // When: the evidence is evaluated, independent of time spent stalled.
        machine.step(&s, &HealthConstants::default());
        // Then: only the inclusive fresh boundary passes.
        assert_eq!(machine.state(), expected);
    }
    let mut machine = HealthMachine::new(Stalled, 0);
    let s = HealthSignals {
        held_links: 3,
        probe_rounds_ok: 2,
        probe_rounds_started_ms: Some(100_000),
        ..signals(106_000)
    };
    machine.step(&s, &HealthConstants::default());
    assert_eq!(machine.state(), Rejoining);
}
