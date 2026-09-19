use super::health::HealthState;
use super::probe::{PROBE_MAX_PPS, ProbeScheduler, ProbeTarget};

fn held(conn_id: u64) -> ProbeTarget {
    ProbeTarget {
        conn_id,
        socket_generation: 0,
        health: HealthState::Stalled,
        deadline_held: true,
        sole_carrier: false,
        srtt_ms: 100,
    }
}

#[test]
fn probe_token_bucket() {
    // Given a bond-wide scheduler and two held links.
    let mut scheduler = ProbeScheduler::default();
    let targets = [held(1), held(2)];
    // When every millisecond offers DATA over five seconds.
    let count = (0..5000)
        .filter(|&now| scheduler.next(&targets, now).is_some())
        .count();
    // Then the global limit, not a per-link limit, applies.
    assert_eq!(count, usize::try_from(PROBE_MAX_PPS * 5).unwrap());
}

#[test]
fn probe_train_rotation_across_two_held_links() {
    // Given two held links.
    let mut scheduler = ProbeScheduler::default();
    let targets = [held(1), held(2)];
    // When three ten-packet trains are scheduled.
    let selected: Vec<_> = (0..30)
        .map(|i| scheduler.next(&targets, i * 100).unwrap().conn_id)
        .collect();
    // Then whole trains rotate, without electing a new DATA carrier.
    assert_eq!(selected, [vec![1; 10], vec![2; 10], vec![1; 10]].concat());
}

#[test]
fn probe_zero_rate_emits_nothing_over_five_seconds() {
    // Given the failure-QA rate override at the same bucket construction seam.
    let mut scheduler = ProbeScheduler::with_rate(0);
    // When DATA is offered throughout five seconds, then no probe is permitted.
    for now in 0..=5000 {
        assert!(scheduler.next(&[held(1)], now).is_none());
    }
}

#[test]
fn probe_targeting_accepts_health_or_deadline_hold_but_never_down_or_sole() {
    // Given every health state and an independent deadline hold.
    for health in [
        HealthState::Healthy,
        HealthState::Stalled,
        HealthState::Degraded,
        HealthState::Down,
        HealthState::Rejoining,
    ] {
        for deadline_held in [false, true] {
            for sole_carrier in [false, true] {
                let target = ProbeTarget {
                    health,
                    deadline_held,
                    sole_carrier,
                    ..held(1)
                };
                let expected = match health {
                    HealthState::Down => false,
                    HealthState::Stalled | HealthState::Degraded => !sole_carrier,
                    HealthState::Healthy | HealthState::Rejoining => deadline_held && !sole_carrier,
                };
                // When scheduled, then either hold qualifies, with hard/sole vetoes.
                assert_eq!(
                    ProbeScheduler::default().next(&[target], 0).is_some(),
                    expected
                );
            }
        }
    }
}
