use std::time::Duration;

use super::*;

fn fixture(events: Vec<TimedEvent>, duration: u64) -> Profile {
    Profile {
        links: vec![LinkProfile {
            base: ImpairmentConfig::default(),
            carrier: CarrierMode::Direct,
        }],
        events,
        duration: Duration::from_secs(duration),
    }
}

#[test]
fn impairment_episode_horizon_starts_at_explicit_restore() {
    // Given: delay restored at 15s, with the restoration edge itself ungraded/zero-tail.
    let on = TimedEvent::new(
        Duration::from_secs(5),
        Some(0),
        Action::SetImpairment(ImpairmentConfig {
            delay_ms: Some(100),
            ..Default::default()
        }),
    );
    let off = TimedEvent {
        at: Duration::from_secs(15),
        action: Action::SetImpairment(ImpairmentConfig::default()),
        horizon: Duration::ZERO,
        graded: false,
        ..on.clone()
    };
    // When/Then: the original event's 30-second horizon ends at 45, not at 35.
    assert!(fixture(vec![on, off], 44).validate().is_err());
}

#[test]
fn periodic_rejects_zero_interval_nested_and_overlapping_holds() {
    // Given: malformed periodic declarations.
    for (every, hold, nested) in [(0, 1, false), (1, 2, false), (1, 1, true)] {
        let action = Action::Periodic {
            every: Duration::from_secs(every),
            action: Box::new(Action::LinkUp(false)),
            hold: Duration::from_secs(hold),
            until: Duration::from_secs(3),
        };
        let action = if nested {
            Action::Periodic {
                every: Duration::from_secs(1),
                action: Box::new(action),
                hold: Duration::from_secs(1),
                until: Duration::from_secs(3),
            }
        } else {
            action
        };
        // When/Then: validation rejects them without looping or reaching the kernel.
        assert!(
            fixture(vec![TimedEvent::new(Duration::ZERO, Some(0), action)], 60)
                .validate()
                .is_err()
        );
    }
}

#[test]
fn periodic_restores_the_latest_state_not_the_initial_base() {
    // Given: a capacity change followed by a short periodic delay spike.
    let changed = ImpairmentConfig {
        delay_ms: Some(45),
        ..Default::default()
    };
    let initial = TimedEvent {
        horizon: Duration::ZERO,
        ..TimedEvent::new(
            Duration::ZERO,
            Some(0),
            Action::SetImpairment(changed.clone()),
        )
    };
    let spike = TimedEvent::new(
        Duration::from_secs(2),
        Some(0),
        Action::Periodic {
            every: Duration::from_secs(5),
            action: Box::new(Action::SetImpairment(ImpairmentConfig {
                delay_ms: Some(200),
                ..Default::default()
            })),
            hold: Duration::from_secs(1),
            until: Duration::from_secs(2),
        },
    );
    // When: expanding the episode.
    let events = fixture(vec![initial, spike], 10).expanded_events().unwrap();
    // Then: restoration preserves the 45ms state, not the original zero-delay link.
    assert_eq!(events[2].action, Action::SetImpairment(changed));
}

#[test]
fn failed_application_is_never_retried_or_logged_as_success() {
    // Given: a runtime action that can have partially changed kernel state.
    let p = fixture(
        vec![TimedEvent::new(
            Duration::ZERO,
            None,
            Action::OfferedRate { bps: 123 },
        )],
        0,
    );
    let mut scheduler = Scheduler::new(&p).unwrap();
    let mut calls = 0;
    let mut apply = |_: &TimedEvent| {
        calls += 1;
        anyhow::bail!("injected I/O failure")
    };
    // When: execution fails, then the caller attempts to run again.
    assert!(scheduler.run(&mut apply).is_err());
    assert!(scheduler.run(&mut apply).is_err());
    // Then: no duplicate side effect or fabricated successful log entry exists.
    assert_eq!(calls, 1);
    assert!(scheduler.log().entries.is_empty());
}

#[test]
fn random_walk_is_deterministic_and_clipped_to_duration() {
    // Given: a non-integral duration/step ratio in the legacy generator.
    let config = ScenarioConfig {
        seed: 9,
        duration: Duration::from_millis(2500),
        step: Duration::from_secs(1),
        links: vec![crate::LinkScenarioConfig {
            min_rate_kbit: 100,
            max_rate_kbit: 500,
            rate_step_kbit: 50,
            base_delay_ms: 20,
            delay_jitter_ms: 5,
            delay_step_ms: 2,
            max_loss_percent: 1.0,
            loss_step_percent: 0.1,
        }],
    };
    // When: generating the same profile twice.
    let a = Profile::from_random_walk(config.clone()).unwrap();
    let b = Profile::from_random_walk(config).unwrap();
    // Then: the overshooting 3s frame is omitted, with reproducible ungraded samples.
    assert_eq!(a.events, b.events);
    assert_eq!(a.events.len(), 3);
    assert!(
        a.events
            .iter()
            .all(|event| !event.graded && event.at <= a.duration)
    );
}
