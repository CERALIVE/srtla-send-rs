use anyhow::{Context, Result, ensure};

use super::{Action, Profile, TimedEvent};

impl Profile {
    /// Stable expansion, including restores to the state immediately before each onset.
    /// One-shot holds end at an explicit restore of the previous property value.
    pub fn expanded_events(&self) -> Result<Vec<TimedEvent>> {
        ensure!(!self.links.is_empty(), "profile requires links");
        for link in &self.links {
            super::qdisc::apply_commands("validation", &link.base)?;
        }
        let mut pending = Vec::new();
        let mut episode = 0;
        for event in &self.events {
            self.validate_target(event)?;
            match &event.action {
                Action::Periodic {
                    every,
                    action,
                    hold,
                    until,
                } => {
                    ensure!(
                        !every.is_zero() && !hold.is_zero() && hold <= every,
                        "periodic requires 0 < hold <= every"
                    );
                    ensure!(*until >= event.at, "periodic until precedes first onset");
                    ensure!(
                        !matches!(**action, Action::Periodic { .. }),
                        "nested periodic events are unsupported"
                    );
                    let mut at = event.at;
                    loop {
                        let end = at.checked_add(*hold).context("periodic hold overflow")?;
                        ensure!(
                            end.checked_add(event.horizon)
                                .is_some_and(|t| t <= self.duration),
                            "periodic hold + horizon exceeds duration at {at:?}"
                        );
                        let onset = TimedEvent {
                            at,
                            action: *action.clone(),
                            ..event.clone()
                        };
                        pending.push((onset.clone(), Some((episode, false))));
                        if action.reversible() {
                            pending.push((
                                TimedEvent {
                                    at: end,
                                    horizon: std::time::Duration::ZERO,
                                    ..onset
                                },
                                Some((episode, true)),
                            ));
                        }
                        episode += 1;
                        ensure!(
                            pending.len() <= 1_000_000,
                            "profile expansion exceeds one million events"
                        );
                        match at.checked_add(*every) {
                            Some(next) if next <= *until => at = next,
                            Some(_) | None => break,
                        }
                    }
                }
                Action::SetImpairment(_)
                | Action::DataBlackhole { .. }
                | Action::LinkUp(_)
                | Action::DefaultRoute(_)
                | Action::Replug
                | Action::ReceiverRestart
                | Action::SighupReorder(_)
                | Action::CrossTraffic { .. }
                | Action::OfferedRate { .. } => pending.push((event.clone(), None)),
            }
        }
        pending
            .sort_by_key(|(event, marker)| (event.at, marker.is_none_or(|(_, restore)| !restore)));
        let mut states = self.initial_states();
        let mut restores = vec![None; episode];
        let mut active: Vec<(Option<usize>, Action, usize)> = Vec::new();
        let mut events = Vec::with_capacity(pending.len());
        for (mut event, marker) in pending {
            match marker {
                Some((id, true)) => {
                    event.action = restores[id].take().context("missing periodic restore")?;
                    active.retain(|(_, _, active_id)| *active_id != id);
                }
                Some((id, false)) => {
                    ensure!(
                        !active.iter().any(|(link, action, _)| *link == event.link
                            && action.same_channel(&event.action)),
                        "overlapping periodic changes on the same property"
                    );
                    if event.action.reversible() {
                        restores[id] = states
                            .iter()
                            .rev()
                            .find(|e| same_channel(e, &event))
                            .map(|e| e.action.clone());
                        active.push((event.link, event.action.clone(), id));
                    }
                }
                None => ensure!(
                    !active.iter().any(|(link, action, _)| *link == event.link
                        && action.same_channel(&event.action)),
                    "one-shot change overlaps periodic hold"
                ),
            }
            states.push(event.clone());
            events.push(event);
        }
        let mut observed = self.initial_states();
        for (i, event) in events.iter().enumerate() {
            let prior = observed.iter().rev().find(|old| same_channel(old, event));
            let restored = prior
                .filter(|old| old.action != event.action)
                .and_then(|old| {
                    events[i + 1..]
                        .iter()
                        .find(|next| same_channel(event, next) && next.action == old.action)
                })
                .map_or(event.at, |next| next.at);
            ensure!(
                restored
                    .checked_add(event.horizon)
                    .is_some_and(|t| t <= self.duration),
                "event hold + horizon exceeds duration at {:?}",
                event.at
            );
            observed.push(event.clone());
        }
        Ok(events)
    }

    fn validate_target(&self, event: &TimedEvent) -> Result<()> {
        ensure!(
            event.link.is_some() == event.action.link_scoped(),
            "action has incorrect link/global scope"
        );
        ensure!(
            event.link.is_none_or(|i| i < self.links.len()),
            "event link is out of range"
        );
        let action = match &event.action {
            Action::Periodic { action, .. } => action.as_ref(),
            action @ (Action::SetImpairment(_)
            | Action::DataBlackhole { .. }
            | Action::LinkUp(_)
            | Action::DefaultRoute(_)
            | Action::Replug
            | Action::ReceiverRestart
            | Action::SighupReorder(_)
            | Action::CrossTraffic { .. }
            | Action::OfferedRate { .. }) => action,
        };
        match action {
            Action::SighupReorder(order) => {
                let mut sorted = order.clone();
                sorted.sort_unstable();
                ensure!(
                    sorted == (0..self.links.len()).collect::<Vec<_>>(),
                    "reorder must be a complete permutation"
                );
            }
            Action::SetImpairment(config) => {
                super::qdisc::apply_commands("validation", config)?;
            }
            Action::CrossTraffic { mbit, on } => {
                ensure!(!on || *mbit > 0, "cross-traffic requires positive mbit")
            }
            Action::DataBlackhole { .. }
            | Action::LinkUp(_)
            | Action::DefaultRoute(_)
            | Action::Replug
            | Action::ReceiverRestart
            | Action::OfferedRate { .. }
            | Action::Periodic { .. } => {}
        }
        Ok(())
    }

    pub(crate) fn initial_states(&self) -> Vec<TimedEvent> {
        let mut states = vec![
            TimedEvent::new(
                std::time::Duration::ZERO,
                None,
                Action::OfferedRate { bps: 0 },
            ),
            TimedEvent::new(
                std::time::Duration::ZERO,
                None,
                Action::SighupReorder((0..self.links.len()).collect()),
            ),
        ];
        for (link, profile) in self.links.iter().enumerate() {
            for action in [
                Action::SetImpairment(profile.base.clone()),
                Action::DataBlackhole { on: false },
                Action::LinkUp(true),
                Action::DefaultRoute(true),
                Action::CrossTraffic { mbit: 0, on: false },
            ] {
                states.push(TimedEvent::new(
                    std::time::Duration::ZERO,
                    Some(link),
                    action,
                ));
            }
        }
        states
    }
}

fn same_channel(a: &TimedEvent, b: &TimedEvent) -> bool {
    a.link == b.link && a.action.same_channel(&b.action)
}
