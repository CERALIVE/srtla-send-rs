use std::time::Duration;

use super::episodes::evaluate;
use super::sink::{SinkBucket, SinkSeries};
use crate::ImpairmentConfig;
use crate::bond::CarrierMode;
use crate::profile::{Action, EventLog, EventRecord, LinkProfile, Profile, TimedEvent};

fn profile(action: Action) -> Profile {
    Profile {
        links: vec![LinkProfile {
            base: ImpairmentConfig::default(),
            carrier: CarrierMode::Nat,
        }],
        events: vec![TimedEvent::new(Duration::from_secs(10), Some(0), action)],
        duration: Duration::from_secs(45),
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

fn series(end: i64, rate: impl Fn(i64) -> u64) -> SinkSeries {
    SinkSeries::new(
        (1..=end)
            .map(|s| SinkBucket {
                t_ms: s * 1000,
                duration_ms: 1000,
                bytes: rate(s) / 8,
                pkts: 1,
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn episode_not_recovered_before_impact() {
    // Given: two seconds of buffered delivery, then a dead viewer.
    let profile = profile(Action::DataBlackhole { on: true });
    let sink = series(40, |s| if s <= 12 { 8000 } else { 0 });
    // When: observe the entire episode, including the first sub-threshold bucket.
    let episodes = evaluate(&profile, &log(&profile), &sink).unwrap();
    // Then: pre-impact buffered delivery never qualifies as failover.
    assert!(episodes[0].impacted);
    assert_eq!(episodes[0].failover_ms, None);
    assert!(!episodes[0].recovered);
    let buffered_only = series(12, |_| 8000);
    assert_eq!(
        evaluate(&profile, &log(&profile), &buffered_only).unwrap()[0].failover_ms,
        None
    );
}

#[test]
fn episode_unimpacted_failover_zero() {
    // Given: a complete horizon with no viewer impact.
    let profile = profile(Action::DataBlackhole { on: true });
    let sink = series(40, |_| 8000);
    // When / Then: the unimpacted completed episode explicitly reports zero.
    let episode = &evaluate(&profile, &log(&profile), &sink).unwrap()[0];
    assert!(!episode.impacted);
    assert_eq!(episode.failover_ms, Some(0));
    assert!(episode.recovered);
}

#[test]
fn episode_recovery_waits_for_a_full_post_impact_second() {
    // Given: baseline, two buffered seconds, impact, then real recovery.
    let mut profile = profile(Action::DataBlackhole { on: true });
    let mut restore = TimedEvent::new(
        Duration::from_secs(15),
        Some(0),
        Action::DataBlackhole { on: false },
    );
    restore.horizon = Duration::ZERO;
    profile.events.push(restore);
    let sink = series(45, |s| if (13..=16).contains(&s) { 0 } else { 8000 });
    // When / Then: recovery is timestamped at the end of its confirming bucket.
    let episode = &evaluate(&profile, &log(&profile), &sink).unwrap()[0];
    assert_eq!(episode.restore_ms, Some(15000));
    assert_eq!(episode.failover_ms, Some(7000));
    assert_eq!(episode.recovery_ms, Some(2000));
    assert_eq!(episode.outage_ms, 4000);
}

#[test]
fn episode_horizon_comes_from_event() {
    // Given: full C/D-style periodic expansion, plus D's eight-second obstruction.
    for duration in [60, 75] {
        let mut profile = profile(Action::Periodic {
            every: Duration::from_secs(15),
            action: Box::new(Action::SetImpairment(ImpairmentConfig {
                delay_ms: Some(119),
                ..Default::default()
            })),
            hold: Duration::from_millis(215),
            until: Duration::from_secs(duration - 6),
        });
        profile.duration = Duration::from_secs(duration);
        profile.events[0].at = Duration::from_secs(15);
        let base = ImpairmentConfig {
            delay_ms: Some(45),
            jitter_ms: Some(5),
            delay_distribution: Some(crate::impairment::DelayDistribution::Normal),
            rate_kbit: Some(20000),
            tbf_shaping: true,
            ..Default::default()
        };
        profile.links[0].base = base.clone();
        profile.links.push(LinkProfile {
            base: ImpairmentConfig {
                delay_ms: Some(65),
                rate_kbit: Some(8000),
                loss_percent: Some(0.2),
                tbf_shaping: true,
                ..Default::default()
            },
            carrier: CarrierMode::Direct,
        });
        profile.events[0].action = Action::Periodic {
            every: Duration::from_secs(15),
            action: Box::new(Action::SetImpairment(ImpairmentConfig {
                delay_ms: Some(119),
                rate_kbit: Some(10000),
                ..base.clone()
            })),
            hold: Duration::from_millis(215),
            until: Duration::from_secs(duration - 6),
        };
        // The full 500ms capacity dip continues after the 215ms delay spike; held property writes cannot overlap.
        profile.events.push(TimedEvent::new(
            Duration::from_millis(15215),
            Some(0),
            Action::Periodic {
                every: Duration::from_secs(15),
                action: Box::new(Action::SetImpairment(ImpairmentConfig {
                    rate_kbit: Some(10000),
                    ..base
                })),
                hold: Duration::from_millis(285),
                until: Duration::from_secs(duration - 6),
            },
        ));
        if duration == 75 {
            profile.events.push(TimedEvent::new(
                Duration::from_secs(20),
                Some(0),
                Action::DataBlackhole { on: true },
            ));
            let mut restore = TimedEvent::new(
                Duration::from_secs(28),
                Some(0),
                Action::DataBlackhole { on: false },
            );
            restore.horizon = Duration::ZERO;
            profile.events.push(restore);
        }
        let events = log(&profile);
        let sink = series(i64::try_from(duration).unwrap(), |_| 8000);
        // When / Then: every expanded event retains its own horizon, never a default.
        let episodes = evaluate(&profile, &events, &sink).unwrap();
        assert_eq!(episodes.len(), events.entries.len());
        for (episode, event) in episodes.iter().zip(&events.entries) {
            assert_eq!(
                episode.horizon_ms,
                u64::try_from(event.event.horizon.as_millis()).unwrap()
            );
            assert!(episode.end_ms <= i64::try_from(duration * 1000).unwrap());
        }
        assert!(episodes.iter().any(|e| e.horizon_ms == 5000));
        if duration == 75 {
            assert!(
                episodes
                    .iter()
                    .any(|e| e.horizon_ms == 30000 && e.end_ms == 58000)
            );
        }
    }
}

#[test]
fn load_interval_dead_sink_never_recovers() {
    // Given: L's overload, planned idle, then graded burst; idle must not be a baseline.
    let mut profile = profile(Action::OfferedRate { bps: 12500 });
    profile.events[0].link = None;
    profile.events[0].at = Duration::ZERO;
    profile.events.extend([
        TimedEvent {
            at: Duration::from_secs(20),
            link: None,
            action: Action::OfferedRate { bps: 0 },
            horizon: Duration::ZERO,
            graded: false,
        },
        TimedEvent {
            at: Duration::from_secs(30),
            link: None,
            action: Action::OfferedRate { bps: 8000 },
            horizon: Duration::from_secs(10),
            graded: true,
        },
    ]);
    let dead = series(45, |_| 0);
    // When / Then: positive targets cannot be satisfied by zero sink throughput.
    let intervals =
        super::load_intervals::evaluate(&profile, &log(&profile), (&dead, 10000)).unwrap();
    assert_eq!(intervals.len(), 3);
    assert_eq!(intervals[0].target_bps, 9000.0);
    assert!(!intervals[1].graded);
    assert_eq!(intervals[1].reached_ms, None);
    assert!(!intervals[2].recovered);
    assert_eq!(intervals[2].reached_ms, None);
    let working = series(45, |s| if s > 30 { 8000 } else { 0 });
    let intervals =
        super::load_intervals::evaluate(&profile, &log(&profile), (&working, 10000)).unwrap();
    assert_eq!(intervals[2].reached_ms, Some(1000));
    assert!(intervals[2].no_collapse);
}
