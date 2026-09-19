use std::time::Duration;

use super::*;
use crate::profile::Profile as Timeline;
use crate::{LinkScenarioConfig, ScenarioConfig};

#[test]
fn satellite_waveform_preserves_both_periodic_durations() {
    // Given: both combined-waveform scenarios, including D's longer duration.
    for profile in [scenario_c(), scenario_d()] {
        // When: expand every cycle and inspect physical link-0 state transitions.
        let events = profile.timeline.expanded_events().unwrap();
        let changes: Vec<_> = events
            .iter()
            .filter_map(|event| match &event.action {
                Action::SetImpairment(config) if event.link == Some(0) => Some((event, config)),
                _ => None,
            })
            .collect();
        let cycles = if profile.timeline.duration.as_secs() == 60 {
            3
        } else {
            4
        };
        // Then: each cycle is +74ms/half-rate, base restore, half-rate, final base restore.
        assert_eq!(changes.len(), cycles * 4);
        for (cycle, chunk) in changes.chunks_exact(4).enumerate() {
            let start = 15_000 * (u64::try_from(cycle).unwrap() + 1);
            let values: Vec<_> = chunk
                .iter()
                .map(|(event, config)| {
                    (
                        u64::try_from(event.at.as_millis()).unwrap() - start,
                        config.delay_ms,
                        config.rate_kbit,
                        event.horizon.as_secs(),
                    )
                })
                .collect();
            assert_eq!(
                values,
                [
                    (0, Some(119), Some(10_000), 5),
                    (215, Some(45), Some(20_000), 0),
                    (215, Some(45), Some(10_000), 5),
                    (500, Some(45), Some(20_000), 0)
                ]
            );
        }
        for event in &profile.timeline.events {
            if let Action::Periodic { until, every, .. } = event.action {
                assert_eq!(until, profile.timeline.duration - Duration::from_secs(6));
                assert_eq!(every, Duration::from_secs(15));
            }
        }
    }
}

#[test]
fn base_profiles_preserve_pinned_constants() {
    // Given / When: the library's physical link configurations.
    let a = scenario_a();
    let b1 = scenario_b1();
    let b2 = scenario_b2();
    let e = scenario_e();
    let f = scenario_f();
    let g = scenario_g();
    // Then: pin units and GE positional values rather than prose descriptions.
    assert!(
        a.timeline
            .links
            .iter()
            .all(|link| link.base.delay_ms == Some(60)
                && link.base.jitter_ms == Some(15)
                && link.base.rate_kbit == Some(10_000)
                && link.base.loss_percent == Some(0.2)
                && link.base.queue_limit == Some(500)
                && link.base.delay_distribution == Some(DelayDistribution::Normal)
                && link.base.tbf_shaping)
    );
    assert_eq!(
        b1.timeline
            .links
            .iter()
            .map(|l| (l.base.delay_ms, l.base.rate_kbit))
            .collect::<Vec<_>>(),
        [
            (Some(20), Some(4_000)),
            (Some(60), Some(4_000)),
            (Some(120), Some(4_000))
        ]
    );
    assert_eq!(
        b2.timeline
            .links
            .iter()
            .map(|l| (l.base.delay_ms, l.base.jitter_ms, l.base.rate_kbit))
            .collect::<Vec<_>>(),
        [
            (Some(35), Some(5), Some(8_000)),
            (Some(90), Some(20), Some(8_000))
        ]
    );
    assert_eq!(
        (
            e.timeline.links[0].base.tbf_latency_ms,
            e.timeline.links[0].base.queue_limit
        ),
        (Some(2_000), Some(1_000))
    );
    assert_eq!(
        (
            e.timeline.events[1].at.as_secs(),
            &e.timeline.events[1].action
        ),
        (15, &Action::CrossTraffic { mbit: 9, on: true })
    );
    assert_eq!(e.timeline.events[2].at.as_secs(), 45);
    assert_eq!(
        f.timeline.links[1].base.gemodel,
        Some(GemodelConfig {
            p: 1.0,
            r: 0.2,
            one_h: 0.5,
            one_k: 0.01
        })
    );
    assert_eq!(
        g.timeline.links[0].base.gemodel,
        Some(GemodelConfig {
            p: 5.0,
            r: 0.1,
            one_h: 0.8,
            one_k: 0.02
        })
    );
    assert_eq!(
        g.timeline
            .links
            .iter()
            .map(|l| l.base.rate_kbit)
            .collect::<Vec<_>>(),
        [Some(1_000), Some(5_000), Some(5_000), Some(5_000)]
    );
}

#[test]
fn lifecycle_events_preserve_order_and_targets() {
    // Given / When: the complete H timeline.
    let profile = scenario_h();
    let events: Vec<_> = profile
        .timeline
        .events
        .iter()
        .skip(1)
        .map(|e| (e.at.as_secs(), e.link, e.action.clone()))
        .collect();
    // Then: each operation targets the intended physical link, not reordered conn_id.
    assert_eq!(
        events,
        [
            (15, Some(2), Action::LinkUp(false)),
            (25, Some(2), Action::LinkUp(true)),
            (35, Some(1), Action::Replug),
            (50, Some(0), Action::DefaultRoute(false)),
            (60, Some(0), Action::DefaultRoute(true)),
            (70, None, Action::SighupReorder(vec![2, 0, 1]))
        ]
    );
}

#[test]
fn k_matches_seeded_random_walk_without_disabling_tbf() {
    // Given: the pinned random-walk input and the existing public adapter.
    let expected = Timeline::from_random_walk(ScenarioConfig {
        seed: 42,
        step: Duration::from_secs(2),
        duration: Duration::from_secs(60),
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
    })
    .unwrap();
    // When: construct K's combined satellite/LTE timeline.
    let profile = scenario_k();
    let actual: Vec<_> = profile
        .timeline
        .events
        .iter()
        .filter(|e| e.link == Some(1))
        .collect();
    // Then: every sample is retained with LTE-only scope and real TBF enforcement.
    assert_eq!(actual.len(), 31);
    for (actual, expected) in actual.iter().zip(expected.events) {
        assert_eq!(actual.at, expected.at);
        let (Action::SetImpairment(actual), Action::SetImpairment(expected)) =
            (&actual.action, expected.action)
        else {
            panic!("walk sample must be an impairment");
        };
        assert_eq!(
            (
                actual.rate_kbit,
                actual.delay_ms,
                actual.jitter_ms,
                actual.loss_percent
            ),
            (
                expected.rate_kbit,
                expected.delay_ms,
                expected.jitter_ms,
                expected.loss_percent
            )
        );
        assert!(actual.tbf_shaping);
    }
    let periodic: Vec<_> = profile
        .timeline
        .events
        .iter()
        .filter(|e| matches!(e.action, Action::Periodic { .. }))
        .collect();
    assert_eq!(periodic.len(), 1);
    assert!(
        matches!(periodic[0].action, Action::Periodic { hold, until, .. }
        if hold == Duration::from_millis(500) && until == Duration::from_secs(54))
    );
}
