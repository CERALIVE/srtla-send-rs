use std::time::Duration;

use super::{Profile, lte};
use crate::bond::CarrierMode;
use crate::profile::{Action, TimedEvent};
use crate::{ImpairmentConfig, LinkScenarioConfig, Scenario, ScenarioConfig};

pub(super) fn profile(seconds: u64, spike: bool) -> Profile {
    let mut starlink = lte(45, 20_000, 5);
    starlink.carrier = CarrierMode::Nat;
    starlink.base.loss_percent = Some(0.0);
    // Draft [3]: satellite uplink queue approximately 4000 packets.
    starlink.base.queue_limit = Some(4_000);
    let base = starlink.base.clone();
    let mut profile = Profile::new(vec![starlink, lte(65, 8_000, 0)], 22_400_000, seconds);
    let until = profile.timeline.duration - Duration::from_secs(6);
    let dip = ImpairmentConfig {
        rate_kbit: Some(10_000),
        ..base.clone()
    };
    // Whole-property holds cannot overlap. A restore at 15.215s sorts BEFORE
    // the next onset, composing exactly 215ms spike + 500ms total capacity dip.
    let phases = if spike {
        vec![
            (
                15_000,
                215,
                ImpairmentConfig {
                    delay_ms: Some(45 + 74),
                    ..dip.clone()
                },
            ),
            (15_215, 285, dip),
        ]
    } else {
        vec![(15_000, 500, dip)]
    };
    profile
        .timeline
        .events
        .extend(phases.into_iter().map(|(at, hold, config)| TimedEvent {
            at: Duration::from_millis(at),
            link: Some(0),
            action: Action::Periodic {
                every: Duration::from_secs(15),
                action: Box::new(Action::SetImpairment(config)),
                hold: Duration::from_millis(hold),
                until,
            },
            horizon: Duration::from_secs(5),
            graded: true,
        }));
    profile
}

pub(super) fn random_walk() -> Profile {
    let mut profile = profile(60, false);
    profile.offered_bps = 12_000_000;
    profile.warmup_offered_bps = 12_000_000;
    profile.timeline.events[0].action = Action::OfferedRate { bps: 12_000_000 };
    // Draft [6–9]: synthetic 2–20Mbit LTE range. Restrict this 8Mbit link to
    // 2–8Mbit, ±1Mbit/step, delay 65ms with 20ms jitter and 5ms steps, loss ≤1%.
    let config = ScenarioConfig {
        seed: 42,
        step: Duration::from_secs(2),
        duration: profile.timeline.duration,
        links: vec![LinkScenarioConfig {
            min_rate_kbit: 2_000,
            max_rate_kbit: 8_000,
            rate_step_kbit: 1_000,
            base_delay_ms: 65,
            delay_jitter_ms: 20,
            delay_step_ms: 5,
            max_loss_percent: 1.0,
            loss_step_percent: 0.1,
        }],
    };
    // Use the same seeded generator as Timeline::from_random_walk, without a
    // fallible constructor in this infallible library; validate() checks the result.
    for frame in Scenario::new(config).frames() {
        for sample in frame.configs {
            let base = &profile.timeline.links[1].base;
            let config = ImpairmentConfig {
                rate_kbit: sample.rate_kbit,
                delay_ms: sample.delay_ms,
                jitter_ms: sample.jitter_ms,
                loss_percent: sample.loss_percent,
                ..base.clone()
            };
            profile.timeline.events.push(TimedEvent {
                at: frame.t,
                link: Some(1),
                action: Action::SetImpairment(config),
                horizon: Duration::ZERO,
                graded: false,
            });
        }
    }
    profile
}
