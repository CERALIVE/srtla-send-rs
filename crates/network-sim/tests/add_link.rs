use std::time::Duration;

use network_sim::ImpairmentConfig;
use network_sim::bond::CarrierMode;
use network_sim::profile::{Action, LinkProfile, Profile, TimedEvent};

fn link() -> LinkProfile {
    LinkProfile {
        base: ImpairmentConfig::default(),
        carrier: CarrierMode::Nat,
    }
}

fn event(at: u64, target: Option<usize>, action: Action) -> TimedEvent {
    TimedEvent {
        horizon: Duration::ZERO,
        ..TimedEvent::new(Duration::from_secs(at), target, action)
    }
}

fn profile(events: Vec<TimedEvent>) -> Profile {
    Profile {
        links: vec![link()],
        events,
        duration: Duration::from_secs(60),
    }
}

#[test]
fn add_link_follows_existing_link_actions_at_the_same_tick() {
    // Given: declaration order puts the addition first.
    let p = profile(vec![
        event(5, None, Action::AddLink(link())),
        event(5, Some(0), Action::LinkUp(true)),
        event(
            5,
            Some(0),
            Action::SetImpairment(ImpairmentConfig::default()),
        ),
    ]);
    // When: expanding the schedule.
    let events = p.expanded_events().unwrap();
    // Then: existing-link changes precede topology growth, keeping their own order.
    assert!(matches!(events[0].action, Action::LinkUp(true)));
    assert!(matches!(events[1].action, Action::SetImpairment(_)));
    assert!(matches!(events[2].action, Action::AddLink(_)));
}

#[test]
fn added_link_accepts_later_actions_and_complete_reorder() {
    // Given: a new link followed by a flap and a two-link reorder.
    let p = profile(vec![
        event(5, None, Action::AddLink(link())),
        event(6, Some(1), Action::LinkUp(false)),
        event(7, None, Action::SighupReorder(vec![1, 0])),
    ]);
    // When: validating the changing pool.
    let events = p.expanded_events().unwrap();
    // Then: the added topology index is available to subsequent actions.
    assert_eq!(events.len(), 3);
}

#[test]
fn added_link_cannot_be_targeted_before_creation() {
    // Given: an action on a future link, including at its addition tick.
    for at in [4, 5] {
        let p = profile(vec![
            event(5, None, Action::AddLink(link())),
            event(at, Some(1), Action::LinkUp(true)),
        ]);
        // When/Then: expansion rejects the reference rather than panicking at runtime.
        assert!(p.expanded_events().is_err());
    }
}

#[test]
fn added_link_periodic_restore_uses_its_initial_impairment() {
    // Given: a new link whose own initial delay is not the default.
    let mut added = link();
    added.base.delay_ms = Some(30);
    let p = profile(vec![
        event(5, None, Action::AddLink(added.clone())),
        event(
            6,
            Some(1),
            Action::Periodic {
                every: Duration::from_secs(5),
                action: Box::new(Action::SetImpairment(ImpairmentConfig::default())),
                hold: Duration::from_secs(1),
                until: Duration::from_secs(6),
            },
        ),
    ]);
    // When: expanding the periodic hold after creation.
    let events = p.expanded_events().unwrap();
    // Then: restoration uses the newly created link's base, not a startup link's.
    assert_eq!(events[2].action, Action::SetImpairment(added.base));
}

#[test]
fn periodic_add_link_is_rejected_before_network_setup() {
    // Given: a repeating structural mutation has no stable target-index contract.
    let p = profile(vec![event(
        5,
        None,
        Action::Periodic {
            every: Duration::from_secs(5),
            action: Box::new(Action::AddLink(link())),
            hold: Duration::from_secs(1),
            until: Duration::from_secs(10),
        },
    )]);
    // When/Then: reject this unsupported combination explicitly.
    assert!(p.expanded_events().is_err());
}

#[test]
fn add_link_action_round_trips_carrier_and_impairment() {
    // Given: a NAT addition with a non-default impairment.
    let mut added = link();
    added.base.rate_kbit = Some(2000);
    let action = Action::AddLink(added);
    // When: recording then loading its wire representation.
    let decoded: Action = serde_json::from_str(&serde_json::to_string(&action).unwrap()).unwrap();
    // Then: both topology and shaping survive the event log.
    assert_eq!(decoded, action);
}

#[test]
fn added_link_hold_requires_its_full_post_restore_horizon() {
    // Given: a new link impaired at 10s, restored at 40s, with a 30s observation tail.
    let mut onset = event(10, Some(1), Action::LinkUp(false));
    onset.horizon = Duration::from_secs(30);
    let p = profile(vec![
        event(5, None, Action::AddLink(link())),
        onset,
        event(40, Some(1), Action::LinkUp(true)),
    ]);
    // When/Then: 60s is too short; the newly born link still requires 70s.
    assert!(p.expanded_events().is_err());
}

#[test]
fn later_additions_are_not_restorations_of_earlier_additions() {
    // Given: three additions, where the last payload repeats the first.
    let mut different = link();
    different.base.delay_ms = Some(50);
    let mut middle = event(10, None, Action::AddLink(different));
    middle.horizon = Duration::from_secs(30);
    let p = profile(vec![
        event(5, None, Action::AddLink(link())),
        middle,
        event(40, None, Action::AddLink(link())),
    ]);
    // When: checking the one-shot observation tails.
    let events = p.expanded_events().unwrap();
    // Then: a later creation never extends an earlier creation's horizon.
    assert_eq!(events.len(), 3);
}
