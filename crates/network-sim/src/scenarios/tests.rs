use std::time::Duration;

use super::*;
use crate::bond::CarrierMode;
use crate::harness::SrtProfile;
use crate::metrics::load_intervals;
use crate::metrics::sink::{SinkBucket, SinkSeries};
use crate::profile::{Action, EventLog, EventRecord};

#[test]
fn all_returns_catalog_entries() {
    // Given / When: enumerate the concrete library, including dynamic M1-M8 additions.
    let profiles = all();
    // Then: stable, unique IDs include both B variants and all M scenarios.
    assert_eq!(profiles.len(), 21);
    assert_eq!(
        profiles.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
        [
            "A", "B1", "B2", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M1", "M2", "M3",
            "M4", "M5", "M6", "M7", "M8"
        ]
    );
}

#[test]
fn every_scenario_has_offered_rate_and_window() {
    // Given / When: all source policies.
    for (id, profile) in all() {
        // Then: a feasible warm-up, explicit source load and production listener.
        assert!(profile.offered_bps > 0, "{id}");
        assert!(profile.warmup_offered_bps > 0, "{id}");
        assert!(
            profile.warmup_offered_bps <= profile.aggregate_capacity_bps(),
            "{id}"
        );
        assert!(profile.timeline.duration >= Duration::from_secs(45), "{id}");
        assert_eq!(profile.srt_profile, SrtProfile::PRODUCTION);
        assert_eq!(
            profile.timeline.events[0].action,
            Action::OfferedRate {
                bps: profile.offered_bps
            }
        );
        if id != "L" {
            assert_eq!(profile.warmup_offered_bps, profile.offered_bps, "{id}");
        }
    }
}

#[test]
fn all_thirteen_validate() {
    // Given / When / Then: validate the FULL periodic expansion of every profile.
    for (id, profile) in all() {
        profile
            .validate()
            .unwrap_or_else(|error| panic!("{id}: {error:#}"));
        profile.timeline.validate().unwrap();
    }
}

#[test]
fn d_has_data_blackhole_on_nat_link() {
    // Given / When: D's concrete events.
    let profile = scenario_d();
    let events = profile.timeline.expanded_events().unwrap();
    // Then: DATA-only fault on the NAT satellite, restored after exactly eight seconds.
    assert_eq!(profile.timeline.links[0].carrier, CarrierMode::Nat);
    let fault: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.action, Action::DataBlackhole { .. }))
        .collect();
    assert_eq!(fault.len(), 2);
    assert_eq!(
        (fault[0].at.as_secs(), fault[0].link, &fault[0].action),
        (20, Some(0), &Action::DataBlackhole { on: true })
    );
    assert_eq!(
        (fault[1].at.as_secs(), &fault[1].action),
        (28, &Action::DataBlackhole { on: false })
    );
    assert_eq!(fault[0].horizon, Duration::from_secs(30));
}

#[test]
fn d_horizon_inside_window() {
    // Given: D keeps the full obstruction/rejoin and periodic tails.
    let profile = scenario_d();
    // When: expand without clipping.
    let events = profile.timeline.expanded_events().unwrap();
    // Then: no horizon is outside the 75-second measurement.
    assert_eq!(profile.timeline.duration, Duration::from_secs(75));
    assert!(
        events
            .iter()
            .all(|event| event.at + event.horizon <= profile.timeline.duration)
    );
    assert!(Duration::from_secs(28 + 30) <= profile.timeline.duration);
}

#[test]
fn periodic_until_too_close_to_duration_is_rejected() {
    // Given: first onset fits, but the inclusive final onset does not.
    let mut profile = scenario_c();
    profile.timeline.events.push(crate::profile::TimedEvent {
        at: Duration::from_secs(14),
        link: Some(1),
        action: Action::Periodic {
            every: Duration::from_secs(15),
            action: Box::new(Action::DataBlackhole { on: true }),
            hold: Duration::from_millis(500),
            until: Duration::from_secs(59),
        },
        horizon: Duration::from_secs(5),
        graded: true,
    });
    // When / Then: the final 59 + 0.5 + 5 seconds is rejected, not clipped.
    let error = profile.timeline.validate().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("periodic hold + horizon exceeds duration at 59s")
    );
}

#[test]
fn l_warmup_rate_is_feasible_and_overload_starts_at_t0() {
    // Given / When: L's source policy and timeline.
    let profile = scenario_l();
    let capacity = profile.aggregate_capacity_bps();
    let events = &profile.timeline.events;
    // Then: warm-up is feasible; exactly three measurement boundaries retain one burst interval.
    assert_eq!(profile.warmup_offered_bps, capacity * 8 / 10);
    assert_eq!(profile.offered_bps, capacity * 5 / 4);
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].at, Duration::ZERO);
    assert_eq!(
        events[0].action,
        Action::OfferedRate {
            bps: capacity * 5 / 4
        }
    );
    assert!(events[0].graded);
    assert_eq!(events[1].at, Duration::from_secs(20));
    assert_eq!(events[1].action, Action::OfferedRate { bps: 0 });
    assert_eq!(events[2].at, Duration::from_secs(30));
    assert_eq!(events[2].action, Action::OfferedRate { bps: 10_000_000 });
    assert_eq!(events[2].horizon, Duration::from_secs(10));
    let ramp = profile.source_ramp.unwrap();
    assert_eq!(
        (ramp.at, ramp.duration, ramp.from_bps, ramp.to_bps),
        (
            Duration::from_secs(30),
            Duration::from_millis(400),
            0,
            10_000_000
        )
    );
}

#[test]
fn ungraded_intervals_declared_for_i_j_l() {
    // Given / When: intentional unavailability, not measured scheduler failure.
    let i = scenario_i();
    let j = scenario_j();
    let l = scenario_l();
    // Then: restart, total-loss holds/restores and idle are explicitly ungraded.
    assert!(
        i.timeline
            .events
            .iter()
            .any(|e| e.action == Action::ReceiverRestart && !e.graded)
    );
    let faults: Vec<_> = j
        .timeline
        .events
        .iter()
        .filter(|e| matches!(e.action, Action::SetImpairment(_)))
        .collect();
    assert_eq!(faults.len(), j.timeline.links.len() * 2);
    assert!(faults.iter().all(|e| !e.graded));
    assert!(!l.timeline.events[1].graded);
    assert!(l.timeline.events[2].graded);
}

fn grade_l(bytes_per_bucket: u64) -> Vec<load_intervals::LoadInterval> {
    let profile = scenario_l();
    let log = EventLog {
        entries: profile
            .timeline
            .expanded_events()
            .unwrap()
            .into_iter()
            .map(|event| EventRecord {
                t_actual_ms: u64::try_from(event.at.as_millis()).unwrap(),
                event,
            })
            .collect(),
    };
    let sink = SinkSeries::new(
        (1..=600)
            .map(|i| SinkBucket {
                t_ms: i * 100,
                duration_ms: 100,
                bytes: if (201..=300).contains(&i) {
                    0
                } else {
                    bytes_per_bucket
                },
                pkts: 0,
            })
            .collect(),
    )
    .unwrap();
    load_intervals::evaluate(
        &profile.timeline,
        &log,
        (&sink, profile.aggregate_capacity_bps()),
    )
    .unwrap()
}

#[test]
fn l_dead_sink_cannot_recover_from_idle_baseline() {
    // Given / When: a still-dead sink through L's actual boundaries and frozen evaluator.
    let intervals = grade_l(0);
    // Then: explicit feasible targets stay positive; idle creates no obligation.
    assert_eq!(intervals.len(), 3);
    assert_eq!(intervals[0].target_bps, 14_400_000.0);
    assert!(!intervals[1].graded);
    assert_eq!(intervals[2].target_bps, 9_000_000.0);
    assert_eq!(intervals[2].reached_ms, None);
    assert!(!intervals[2].recovered);
}

#[test]
fn l_feasible_sink_passes_overload_and_reaches_burst_target() {
    // Given / When: 16Mbit sink delivery, despite a 20Mbit offered overload.
    let intervals = grade_l(200_000);
    // Then: no impossible 18Mbit target; burst success requires a reached bucket.
    assert!(intervals[0].no_collapse);
    assert!(intervals[2].no_collapse);
    assert_eq!(intervals[2].reached_ms, Some(1_000));
    assert!(intervals[2].recovered);
}
