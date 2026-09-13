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
fn probe_targeting_requires_soft_health_deadline_hold_and_not_sole_carrier() {
    // Given every health state, a deadline-release and a sole-carrier veto.
    for health in [
        HealthState::Healthy,
        HealthState::Down,
        HealthState::Rejoining,
    ] {
        let mut target = held(1);
        target.health = health;
        // When scheduling, then these links are never probed.
        assert!(ProbeScheduler::default().next(&[target], 0).is_none());
    }
    for target in [
        ProbeTarget {
            deadline_held: false,
            ..held(1)
        },
        ProbeTarget {
            sole_carrier: true,
            ..held(1)
        },
    ] {
        assert!(ProbeScheduler::default().next(&[target], 0).is_none());
    }
    assert!(
        ProbeScheduler::default()
            .next(
                &[ProbeTarget {
                    health: HealthState::Degraded,
                    ..held(1)
                }],
                0
            )
            .is_some()
    );
}
