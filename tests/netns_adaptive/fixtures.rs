use std::time::Duration;

use network_sim::bond::{BondRow, CarrierMode, MappingMode};
use network_sim::profile::{Action, TimedEvent};
use network_sim::scenarios::{Profile, scenario_i};

pub fn mapping(ids: &[&str]) -> MappingMode {
    MappingMode::BindMap {
        rows: ids
            .iter()
            .enumerate()
            .map(|(iface_index, id)| BondRow {
                link_id: (*id).to_owned(),
                iface_index,
                priority: (*id == "modem-b").then_some(0.2),
            })
            .collect(),
    }
}

pub fn twins_profile() -> Profile {
    let mut profile = scenario_i();
    for link in &mut profile.timeline.links {
        link.carrier = CarrierMode::Nat;
        link.base.loss_percent = Some(0.0);
    }
    profile
        .timeline
        .events
        .retain(|e| matches!(e.action, Action::OfferedRate { .. }));
    for (at, link, action) in [
        (20, None, Action::OfferedRate { bps: 6_400_000 }),
        (20, Some(0), Action::DataBlackhole { on: true }),
        (28, Some(0), Action::DataBlackhole { on: false }),
        (28, None, Action::OfferedRate { bps: 12_800_000 }),
    ] {
        profile.timeline.events.push(TimedEvent {
            at: Duration::from_secs(at),
            link,
            action,
            horizon: Duration::ZERO,
            graded: true,
        });
    }
    profile
}

/// Preserve the impairment waveform, but separate recovery from aggregate-load stress.
pub fn feasible_recovery(profile: &mut Profile, full_rate_at: u64) {
    profile
        .timeline
        .events
        .retain(|e| e.at.is_zero() || !matches!(e.action, Action::OfferedRate { .. }));
    for (at, bps) in [(20, 6_400_000), (full_rate_at, profile.offered_bps)] {
        profile.timeline.events.push(TimedEvent::new(
            Duration::from_secs(at),
            None,
            Action::OfferedRate { bps },
        ));
    }
}
