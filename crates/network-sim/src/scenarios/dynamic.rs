//! Dynamic conditions, not hardware measurements. Draft citation numbers retain
//! docs/notes/bench-scenarios.md's key. Task 11 fixes the synthetic schedules;
//! unspecified choices below explicitly reserve load/recovery headroom.
//! All delays are netem values; LTE inherits lte()'s 0.2% loss, queue 500,
//! direct carrier and TBF defaults. Grading described here belongs to campaigns.
// allow: SIZE_OK — Task 11 fences all eight definitions and their regression tests
// into this single new file; extract scenario_catalog when that fence is lifted.

use std::time::Duration;

use anyhow::{Context, Result, ensure};

use super::{Profile, WARMUP_SETTLE_RATIO, WARMUP_SETTLE_SECONDS, WARMUP_TIMEOUT, edge, lte};
use crate::profile::{Action, TimedEvent};
use crate::{ImpairmentConfig, LinkScenarioConfig, Scenario, ScenarioConfig};

/// Manifest-only policy: copy this JSON into EVERY M6 cell's `priority_sidecar`.
/// Task 11 prescribes the extrema, not a measured radio property. Profile has no
/// sidecar field; merely selecting M6 does NOT install priorities.
pub const M6_PRIORITY_SIDECAR: &str = r#"[{"link":0,"priority":0.2},{"link":1,"priority":-0.2}]"#;

/// BENCH_SOAK marker: M8 is lookup-visible but excluded from normal N=5 manifests.
/// Todo 20 must select it only in a separate N=1 manifest under BENCH_SOAK=1.
/// No environment-dependent catalog or execution gate exists in the profile API.
pub const BENCH_SOAK: &str = "M8";

/// M1: two LTE 60±15ms/10Mbit links, 9.6Mbit offered, 90s.
/// Source: Draft [6] 10.1145/3618257.3624814, [7] Mahimahi, [13] tc-netem(8).
/// Task 11's synthetic link-1 cycles ramp to 300ms/1Mbit/3% in 10s,
/// hold 15s, recover in 10s. Starts 10/50s leave 10s baseline, 5s between/after.
/// One-second linear samples retain 15ms jitter; like K, samples have no episode
/// grade/tail (whole-window delivery remains graded by the offered-rate boundary).
pub(super) fn scenario_m1() -> Profile {
    let mut profile = Profile::new(vec![lte(60, 10_000, 15); 2], 9_600_000, 90);
    for start in [10, 50] {
        for (at, recovering) in [(start, false), (start + 25, true)] {
            for step in 0_u16..=10 {
                let severity = if recovering { 10 - step } else { step };
                let config = ImpairmentConfig {
                    delay_ms: Some(60 + 24 * u32::from(severity)),
                    rate_kbit: Some(10_000 - 900 * u64::from(severity)),
                    loss_percent: Some(0.2 + 0.28 * f32::from(severity)),
                    ..profile.timeline.links[1].base.clone()
                };
                profile.timeline.events.push(TimedEvent {
                    horizon: Duration::ZERO,
                    graded: false,
                    ..edge(at + u64::from(step), Some(1), Action::SetImpairment(config))
                });
            }
        }
    }
    profile
}

/// M2: Wi-Fi 10±3ms/30Mbit plus LTE 60ms/8Mbit; 8Mbit offered, 60s.
/// Source: Draft [6–9] LTE ranges, [13] tc-netem(8); Task 11 synthetic handoff.
/// Wi-Fi uses zero baseline loss and inherited queue 500; down 20s, up 40s.
/// The 20s outage is NOT an IEEE 802.11r roaming-latency claim. Grade first
/// Wi-Fi DATA after restoration by 45s; the 5s recovery tail preserves that window.
pub(super) fn scenario_m2() -> Profile {
    let mut wifi = lte(10, 30_000, 3);
    wifi.base.loss_percent = Some(0.0);
    let mut profile = Profile::new(vec![wifi, lte(60, 8_000, 0)], 8_000_000, 60);
    profile.timeline.events.extend([
        TimedEvent {
            horizon: Duration::from_secs(5),
            ..edge(20, Some(0), Action::LinkUp(false))
        },
        TimedEvent {
            horizon: Duration::ZERO,
            ..edge(40, Some(0), Action::LinkUp(true))
        },
    ]);
    profile
}

/// M3: Ethernet 5ms/50Mbit plus LTE 60ms/8Mbit, 8Mbit offered.
/// Source: Draft [6–9], [13] tc-netem(8), [24] tc-u32(8); Task 11 synthetic asymmetry.
/// Ethernet has no baseline loss/jitter, queue 500. DATA-only blackhole 30–35s;
/// grade zero receiver-side drops. Chosen 60s duration leaves 25s after restore;
/// a 5s episode tail matches the neighboring handoff recovery budget.
pub(super) fn scenario_m3() -> Profile {
    let mut ethernet = lte(5, 50_000, 0);
    ethernet.base.loss_percent = Some(0.0);
    let mut profile = Profile::new(vec![ethernet, lte(60, 8_000, 0)], 8_000_000, 60);
    profile.timeline.events.extend([
        TimedEvent {
            horizon: Duration::from_secs(5),
            ..edge(30, Some(0), Action::DataBlackhole { on: true })
        },
        TimedEvent {
            horizon: Duration::ZERO,
            ..edge(35, Some(0), Action::DataBlackhole { on: false })
        },
    ]);
    profile
}

/// M4: NAT Starlink 45±5ms/20Mbit/queue 4000/zero loss plus 2×LTE 60ms/5Mbit.
/// Source: Draft [1] nature s44459-026-00044-z, [2] OASIcs.NINeS.2026.7,
/// [3] sigcomm26-dissect-starlink; reuse satellite.rs's base profile verbatim.
/// Task 11 strengthens the 15s cadence into synthetic FULL 2s outages at 15/30/45s,
/// not a claim those papers measured this exact outage pattern. Duration 60s;
/// choose 8Mbit offered (80% of surviving LTE capacity), 5s periodic recovery tail.
pub(super) fn scenario_m4() -> Profile {
    let starlink = super::satellite::profile(60, false)
        .timeline
        .links
        .remove(0);
    let mut profile = Profile::new(
        vec![starlink, lte(60, 5_000, 0), lte(60, 5_000, 0)],
        8_000_000,
        60,
    );
    profile.timeline.events.push(edge(
        15,
        Some(0),
        Action::Periodic {
            every: Duration::from_secs(15),
            action: Box::new(Action::LinkUp(false)),
            hold: Duration::from_secs(2),
            until: Duration::from_secs(45),
        },
    ));
    profile
}

/// M5: one LTE 60ms/8Mbit, add an identical second link at 20s, 60s total.
/// Source: Draft [6–9] LTE ranges, [13] tc-netem(8); Task 11 synthetic link-add.
/// Choose 6.4Mbit before addition (80% capacity, feasible settle), 12.8Mbit after
/// addition (80% bonded capacity) so the source cannot cap the growth test.
/// Grade goodput after 25s ≥1.6× before 20s. The 5s AddLink tail marks acquisition;
/// it does not assume registration finishes in that time or waive a late result.
pub(super) fn scenario_m5() -> Profile {
    let mut profile = Profile::new(vec![lte(60, 8_000, 0)], 6_400_000, 60);
    profile.timeline.events.extend([
        TimedEvent {
            horizon: Duration::from_secs(5),
            ..edge(20, None, Action::AddLink(lte(60, 8_000, 0)))
        },
        edge(20, None, Action::OfferedRate { bps: 12_800_000 }),
    ]);
    profile
}

/// M6: two equal LTE 60ms/8Mbit links, 45s; choose 12.8Mbit (80% aggregate).
/// Source: Draft [6–9], [13] tc-netem(8); Task 11 synthetic priority control.
/// Requires M6_PRIORITY_SIDECAR in each manifest cell: +0.2 link 0, −0.2 link 1.
/// Grade link-0 DATA share in [0.55,0.75] for EVERY CLI candidate mode; shared
/// admission, not the selected ranking algorithm, is responsible for preference.
pub(super) fn scenario_m6() -> Profile {
    Profile::new(vec![lte(60, 8_000, 0); 2], 12_800_000, 45)
}

/// M7: three LTE 60ms/10Mbit, 12Mbit offered; 6Mbit cross-load EACH at 15–45s.
/// Source: Draft [9] Alfredsson WoWMoM13, [13] tc-netem(8); Task 11 synthetic congestion.
/// Choose 10Mbit per link (A baseline), no jitter, 60s duration: residual aggregate
/// 12Mbit makes ≥0.5× delivery physically feasible. Grade no settle-phase loss,
/// ≥0.5× offered during congestion, ≥0.9× by 50s; hence 5s recovery tails.
pub(super) fn scenario_m7() -> Profile {
    let mut profile = Profile::new(vec![lte(60, 10_000, 0); 3], 12_000_000, 60);
    for link in 0..3 {
        profile.timeline.events.extend([
            TimedEvent {
                horizon: Duration::from_secs(5),
                ..edge(15, Some(link), Action::CrossTraffic { mbit: 6, on: true })
            },
            edge(45, Some(link), Action::CrossTraffic { mbit: 0, on: false }),
        ]);
    }
    profile
}

/// M8: BENCH_SOAK, 600s, three LTE links with K's seeded random walk on ALL links.
/// Source: Draft [6–9] synthetic LTE bounds; scenario K / satellite.rs::random_walk.
/// Mirror seed 42, step 2s, rate 2–8Mbit ±1Mbit, delay 65ms/20ms jitter/5ms steps,
/// loss ≤1% ±0.1 percentage point. Preserve lte() queue 500 and TBF per sample.
/// Choose 4.8Mbit offered (80% of the 3×2Mbit capacity floor) for a feasible soak;
/// initial links 65±20ms/8Mbit, sampled changes ungraded/zero-tail exactly like K.
pub(super) fn scenario_m8() -> Profile {
    let mut profile = Profile::new(vec![lte(65, 8_000, 20); 3], 4_800_000, 600);
    let walk = ScenarioConfig {
        seed: 42,
        step: Duration::from_secs(2),
        duration: profile.timeline.duration,
        links: vec![
            LinkScenarioConfig {
                min_rate_kbit: 2_000,
                max_rate_kbit: 8_000,
                rate_step_kbit: 1_000,
                base_delay_ms: 65,
                delay_jitter_ms: 20,
                delay_step_ms: 5,
                max_loss_percent: 1.0,
                loss_step_percent: 0.1,
            };
            3
        ],
    };
    for frame in Scenario::new(walk).frames() {
        for (link, sample) in frame.configs.into_iter().enumerate() {
            let config = ImpairmentConfig {
                rate_kbit: sample.rate_kbit,
                delay_ms: sample.delay_ms,
                jitter_ms: sample.jitter_ms,
                loss_percent: sample.loss_percent,
                ..profile.timeline.links[link].base.clone()
            };
            profile.timeline.events.push(TimedEvent {
                at: frame.t,
                link: Some(link),
                action: Action::SetImpairment(config),
                horizon: Duration::ZERO,
                graded: false,
            });
        }
    }
    profile
}

/// Named catalog validation, including the post-start topology-growth contract.
/// Uses the shared settle constants; does not change generic Profile validation.
pub fn validate_dynamic(name: &str, profile: &Profile) -> Result<()> {
    profile
        .validate()
        .with_context(|| format!("{name}: invalid scenario"))?;
    ensure!(
        WARMUP_SETTLE_RATIO > 0.0
            && WARMUP_SETTLE_RATIO <= 1.0
            && WARMUP_SETTLE_SECONDS > 0
            && Duration::from_secs(u64::from(WARMUP_SETTLE_SECONDS)) < WARMUP_TIMEOUT,
        "{name}: degenerate settle criterion"
    );
    for event in profile.timeline.expanded_events()? {
        ensure!(
            !matches!(event.action, Action::AddLink(_)) || !event.at.is_zero(),
            "{name}: AddLink must be strictly after scenario start"
        );
    }
    Ok(())
}

#[cfg(test)]
mod scenario_catalog {
    use std::time::Duration;

    use super::*;
    use crate::profile::Action;

    #[test]
    fn m1_through_m8_are_registered_and_valid() {
        // Given: the task's complete dynamic family, including the soak definition.
        let expected = [
            ("M1", 90, 2),
            ("M2", 60, 2),
            ("M3", 60, 2),
            ("M4", 60, 3),
            ("M5", 60, 1),
            ("M6", 45, 2),
            ("M7", 60, 3),
            ("M8", 600, 3),
        ];
        // When: use the same catalog lookup as the campaign runner.
        let catalog = crate::scenarios::all();
        let dynamic: Vec<_> = catalog
            .iter()
            .filter(|(id, _)| id.starts_with('M'))
            .collect();
        // Then: exactly eight profiles pass named settle/event validation.
        assert_eq!(dynamic.len(), expected.len());
        for ((id, profile), (name, seconds, links)) in dynamic.into_iter().zip(expected) {
            assert_eq!(*id, name);
            assert_eq!(
                profile.timeline.duration,
                Duration::from_secs(seconds),
                "{id}"
            );
            assert_eq!(profile.timeline.links.len(), links, "{id}");
            validate_dynamic(id, profile).unwrap_or_else(|error| panic!("{error:#}"));
            println!("{id}: settle, events, duration and AddLink timing valid");
        }
    }

    #[test]
    fn event_past_duration_is_rejected_with_scenario_name() {
        // Given: an M2 boundary beyond its measurement window.
        let mut profile = scenario_m2();
        profile.timeline.events.last_mut().unwrap().at =
            profile.timeline.duration + Duration::from_secs(1);
        // When / Then: validation rejects it and identifies the scenario.
        let error = validate_dynamic("M2", &profile).unwrap_err();
        assert!(format!("{error:#}").contains("M2"));
    }

    #[test]
    fn add_link_at_start_is_rejected_with_scenario_name() {
        // Given: a topology creation moved before the measurement starts.
        let mut profile = scenario_m5();
        profile
            .timeline
            .events
            .iter_mut()
            .find(|e| matches!(e.action, Action::AddLink(_)))
            .unwrap()
            .at = Duration::ZERO;
        // When / Then: the dynamic catalog rejects the named creation boundary.
        let error = validate_dynamic("M5", &profile).unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("M5") && message.contains("AddLink"));
    }

    #[test]
    fn infeasible_settle_is_rejected_with_scenario_name() {
        // Given: M6's source cannot settle above its initial capacity.
        let mut profile = scenario_m6();
        profile.warmup_offered_bps = profile.aggregate_capacity_bps() + 1;
        // When / Then: fail with the scenario name rather than a silent timeout.
        assert!(format!("{:#}", validate_dynamic("M6", &profile).unwrap_err()).contains("M6"));
    }

    #[test]
    fn m1_has_two_complete_degrade_hold_recover_cycles() {
        // Given / When: inspect the actual generated impairment waveform.
        let profile = scenario_m1();
        let samples: Vec<_> = profile
            .timeline
            .events
            .iter()
            .filter_map(|event| {
                if let Action::SetImpairment(config) = &event.action {
                    Some((event, config))
                } else {
                    None
                }
            })
            .collect();
        // Then: both ramps reach the prescribed endpoints and return to baseline.
        assert_eq!(samples.len(), 44);
        for start in [10, 50] {
            for (at, delay, rate, loss) in [
                (start, 60, 10_000, 0.2),
                (start + 10, 300, 1_000, 3.0),
                (start + 25, 300, 1_000, 3.0),
                (start + 35, 60, 10_000, 0.2),
            ] {
                let (event, config) = samples.iter().find(|(e, _)| e.at.as_secs() == at).unwrap();
                assert_eq!(event.link, Some(1));
                assert_eq!(
                    (config.delay_ms, config.rate_kbit),
                    (Some(delay), Some(rate))
                );
                assert!((config.loss_percent.unwrap() - loss).abs() < f32::EPSILON);
            }
        }
    }

    #[test]
    fn fault_and_growth_edges_match_task_windows() {
        // Given: explicit expected event streams, independent of the constructors.
        let cases = [
            (
                scenario_m2(),
                vec![
                    (20, Some(0), Action::LinkUp(false)),
                    (40, Some(0), Action::LinkUp(true)),
                ],
            ),
            (
                scenario_m3(),
                vec![
                    (30, Some(0), Action::DataBlackhole { on: true }),
                    (35, Some(0), Action::DataBlackhole { on: false }),
                ],
            ),
            (
                scenario_m4(),
                [15, 30, 45]
                    .into_iter()
                    .flat_map(|t| {
                        [
                            (t, Some(0), Action::LinkUp(false)),
                            (t + 2, Some(0), Action::LinkUp(true)),
                        ]
                    })
                    .collect(),
            ),
            (
                scenario_m5(),
                vec![
                    (20, None, Action::OfferedRate { bps: 12_800_000 }),
                    (20, None, Action::AddLink(lte(60, 8_000, 0))),
                ],
            ),
            (
                scenario_m7(),
                [15, 45]
                    .into_iter()
                    .flat_map(|t| {
                        (0..3).map(move |link| {
                            (
                                t,
                                Some(link),
                                Action::CrossTraffic {
                                    mbit: if t == 15 { 6 } else { 0 },
                                    on: t == 15,
                                },
                            )
                        })
                    })
                    .collect(),
            ),
        ];
        for (profile, expected) in cases {
            // When / Then: expansion preserves every onset/restore and no extra faults.
            let events = profile.timeline.expanded_events().unwrap();
            let actual: Vec<_> = events
                .into_iter()
                .skip(1)
                .map(|e| (e.at.as_secs(), e.link, e.action))
                .collect();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn m6_sidecar_uses_the_manifest_priority_type() {
        // Given / When: parse the published recipe with the real manifest field type.
        let priorities: crate::bond::PrioritySidecar =
            serde_json::from_str(M6_PRIORITY_SIDECAR).unwrap();
        // Then: both existing topology indices carry the requested priorities.
        priorities
            .validate_links(scenario_m6().timeline.links.len())
            .unwrap();
        assert_eq!(
            (priorities.priority(0), priorities.priority(1)),
            (Some(0.2), Some(-0.2))
        );
    }

    #[test]
    fn m8_is_marked_soak_and_all_links_walk_reproducibly() {
        // Given / When: construct the full seeded soak twice without running a campaign.
        let profile = scenario_m8();
        let repeated = scenario_m8();
        // Then: every link has all 301 frames, kept inside K's rate/loss bounds.
        assert_eq!(BENCH_SOAK, "M8");
        assert_eq!(profile.timeline.events, repeated.timeline.events);
        for link in 0..3 {
            let samples: Vec<_> = profile
                .timeline
                .events
                .iter()
                .filter(|e| e.link == Some(link))
                .collect();
            assert_eq!(samples.len(), 301);
            for (step, event) in samples.iter().enumerate() {
                assert_eq!(
                    event.at,
                    Duration::from_secs(u64::try_from(step).unwrap() * 2)
                );
                let Action::SetImpairment(config) = &event.action else {
                    panic!("expected walk sample")
                };
                assert!((2_000..=8_000).contains(&config.rate_kbit.unwrap()));
                assert!((0.0..=1.0).contains(&config.loss_percent.unwrap()));
                assert_eq!((config.queue_limit, config.tbf_shaping), (Some(500), true));
            }
            assert!(
                samples
                    .windows(2)
                    .any(|pair| pair[0].action != pair[1].action)
            );
        }
    }
}
