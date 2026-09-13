use std::time::Duration;

use super::sink::{SinkBucket, SinkSeries};
use super::{Window, episodes, load_intervals};
use crate::ImpairmentConfig;
use crate::bond::CarrierMode;
use crate::profile::{Action, EventLog, EventRecord, LinkProfile, Profile, TimedEvent};

fn sink(end_ms: i64, bytes: impl Fn(i64) -> u64) -> SinkSeries {
    SinkSeries::new(
        (1..=end_ms / 100)
            .map(|i| SinkBucket {
                t_ms: i * 100,
                duration_ms: 100,
                bytes: bytes(i * 100),
                pkts: 0,
            })
            .collect(),
    )
    .unwrap()
}

fn profile(events: Vec<TimedEvent>) -> Profile {
    Profile {
        links: vec![LinkProfile {
            base: ImpairmentConfig::default(),
            carrier: CarrierMode::Nat,
        }],
        duration: Duration::from_secs(40),
        events,
    }
}

fn log(profile: &Profile) -> EventLog {
    EventLog {
        entries: profile
            .expanded_events()
            .unwrap()
            .into_iter()
            .map(|event| EventRecord {
                t_actual_ms: u64::try_from(event.at.as_millis()).unwrap(),
                event,
            })
            .collect(),
    }
}

#[test]
fn micro_impact_is_seen_at_sink_resolution_not_hidden_by_second_averaging() {
    // Given: 100ms impact inside an otherwise healthy second.
    let profile = profile(vec![TimedEvent::new(
        Duration::from_secs(10),
        Some(0),
        Action::DataBlackhole { on: true },
    )]);
    let sink = sink(40000, |t| if t == 12100 { 0 } else { 100 });
    // When / Then: detect at 100ms, confirm only with a full post-impact second.
    let episode = &episodes::evaluate(&profile, &log(&profile), &sink).unwrap()[0];
    assert!(episode.impacted);
    assert_eq!(episode.outage_ms, 100);
    assert_eq!(episode.failover_ms, Some(4000));
}

#[test]
fn forced_outage_is_excluded_from_load_no_collapse() {
    // Given: a graded load with a scenario-declared, ungraded ten-second total outage.
    let profile = profile(vec![
        TimedEvent::new(Duration::ZERO, None, Action::OfferedRate { bps: 8000 }),
        TimedEvent {
            at: Duration::from_secs(10),
            link: Some(0),
            action: Action::DataBlackhole { on: true },
            horizon: Duration::from_secs(20),
            graded: false,
        },
        TimedEvent {
            at: Duration::from_secs(20),
            link: Some(0),
            action: Action::DataBlackhole { on: false },
            horizon: Duration::ZERO,
            graded: false,
        },
    ]);
    let sink = sink(40000, |t| if t > 10000 && t <= 20000 { 0 } else { 100 });
    // When / Then: the forced interval is not falsely charged as scheduler collapse.
    let intervals = load_intervals::evaluate(&profile, &log(&profile), (&sink, 8000)).unwrap();
    assert!(intervals[0].no_collapse);
}

#[test]
fn source_idle_never_creates_an_impairment_episode() {
    // Given / When / Then: an offered-rate edge is a source boundary only.
    let profile = profile(vec![TimedEvent::new(
        Duration::ZERO,
        None,
        Action::OfferedRate { bps: 0 },
    )]);
    let sink = sink(40000, |_| 0);
    assert!(
        episodes::evaluate(&profile, &log(&profile), &sink)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn malformed_sink_gap_is_missing_measurement_not_zero_goodput() {
    // Given / When / Then: missing buckets cannot become a passing no-impact episode.
    assert!(SinkSeries::parse("t,bytes,pkts\n100,1,1\n300,1,1\n", 0).is_err());
    assert!(Window::new(10, 10).is_err());
}

#[test]
fn one_hz_sampler_uses_actual_time_and_emits_the_initial_sample() {
    // Given: a zero-length collection observes its start edge without sleeping.
    // When / Then: use the real collector wrapper, not a duplicate clock algorithm.
    let samples = super::sampling::sample_for(Duration::ZERO, |t| Ok(t)).unwrap();
    assert_eq!(samples.len(), 1);
    assert!(samples[0] >= 0);
}

#[test]
fn zero_tail_restore_does_not_exclude_the_healthy_interval_until_next_fault() {
    // Given: two ungraded periodic faults, with unrelated collapse between them.
    let profile = profile(vec![
        TimedEvent::new(Duration::ZERO, None, Action::OfferedRate { bps: 8000 }),
        TimedEvent {
            at: Duration::from_secs(10),
            link: Some(0),
            action: Action::Periodic {
                every: Duration::from_secs(10),
                action: Box::new(Action::DataBlackhole { on: true }),
                hold: Duration::from_secs(2),
                until: Duration::from_secs(20),
            },
            horizon: Duration::from_secs(5),
            graded: false,
        },
    ]);
    let sink = sink(40000, |t| {
        if (t > 10000 && t <= 19000) || (t > 20000 && t <= 22000) {
            0
        } else {
            100
        }
    });
    // When / Then: only fault holds are excluded; the intervening collapse still fails.
    assert!(
        !load_intervals::evaluate(&profile, &log(&profile), (&sink, 8000)).unwrap()[0].no_collapse
    );
}
