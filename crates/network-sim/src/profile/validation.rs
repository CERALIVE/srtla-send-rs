use std::time::Duration;

use anyhow::{Context, Result, ensure};

use super::{Action, LinkProfile, TimedEvent};

pub(super) fn validate_target(event: &TimedEvent, link_count: usize) -> Result<()> {
    ensure!(
        event.link.is_some() == event.action.link_scoped(),
        "action has incorrect link/global scope"
    );
    ensure!(
        event.link.is_none_or(|i| i < link_count),
        "event link is out of range"
    );
    match &event.action {
        Action::AddLink(link) => {
            ensure!(link_count < 253, "profile exceeds 253 links");
            super::qdisc::apply_commands("validation", &link.base)?;
        }
        Action::SighupReorder(order) => {
            let mut sorted = order.clone();
            sorted.sort_unstable();
            ensure!(
                sorted == (0..link_count).collect::<Vec<_>>(),
                "reorder must be a complete permutation"
            );
        }
        Action::SetImpairment(config) => {
            super::qdisc::apply_commands("validation", config)?;
        }
        Action::CrossTraffic { mbit, on } => {
            ensure!(!on || *mbit > 0, "cross-traffic requires positive mbit")
        }
        Action::Periodic { .. } => anyhow::bail!("expand periodic events before target validation"),
        Action::DataBlackhole { .. }
        | Action::LinkUp(_)
        | Action::DefaultRoute(_)
        | Action::Replug
        | Action::ReceiverRestart
        | Action::OfferedRate { .. } => {}
    }
    Ok(())
}

pub(crate) fn link_states(index: usize, profile: &LinkProfile, at: Duration) -> Vec<TimedEvent> {
    [
        Action::SetImpairment(profile.base.clone()),
        Action::DataBlackhole { on: false },
        Action::LinkUp(true),
        Action::DefaultRoute(true),
        Action::CrossTraffic { mbit: 0, on: false },
    ]
    .into_iter()
    .map(|action| TimedEvent::new(at, Some(index), action))
    .collect()
}

pub(crate) fn observe_creation(
    states: &mut Vec<TimedEvent>,
    link_count: &mut usize,
    event: &TimedEvent,
) -> Result<()> {
    match &event.action {
        Action::AddLink(link) => {
            states.extend(link_states(*link_count, link, event.at));
            let mut order = states
                .iter()
                .rev()
                .find_map(|state| match &state.action {
                    Action::SighupReorder(order) => Some(order.clone()),
                    Action::AddLink(_)
                    | Action::SetImpairment(_)
                    | Action::DataBlackhole { .. }
                    | Action::LinkUp(_)
                    | Action::DefaultRoute(_)
                    | Action::Replug
                    | Action::ReceiverRestart
                    | Action::CrossTraffic { .. }
                    | Action::OfferedRate { .. }
                    | Action::Periodic { .. } => None,
                })
                .context("missing initial publication order")?;
            order.push(*link_count);
            states.push(TimedEvent::new(
                event.at,
                None,
                Action::SighupReorder(order),
            ));
            *link_count += 1;
        }
        Action::SetImpairment(_)
        | Action::DataBlackhole { .. }
        | Action::LinkUp(_)
        | Action::DefaultRoute(_)
        | Action::Replug
        | Action::ReceiverRestart
        | Action::SighupReorder(_)
        | Action::CrossTraffic { .. }
        | Action::OfferedRate { .. }
        | Action::Periodic { .. } => {}
    }
    Ok(())
}
