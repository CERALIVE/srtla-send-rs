use anyhow::{Result, ensure};
use network_sim::profile::Action;
use network_sim::scenarios::Profile;

use super::{Cell, Manifest, ManifestError};

pub fn sls_profile() -> Result<Profile> {
    let mut profile = network_sim::scenarios::scenario_a();
    profile.timeline.links.truncate(2);
    for link in &mut profile.timeline.links {
        link.base = network_sim::ImpairmentConfig {
            delay_ms: Some(5),
            rate_kbit: Some(10_000),
            ..Default::default()
        };
    }
    profile.offered_bps = 1_000_000;
    profile.warmup_offered_bps = 1_000_000;
    profile.timeline.duration = Duration::from_secs(20);
    profile.timeline.events = vec![network_sim::profile::TimedEvent::new(
        Duration::ZERO,
        None,
        Action::OfferedRate { bps: 1_000_000 },
    )];
    profile.timeline.validate()?;
    Ok(profile)
}

pub fn freeze_profile() -> Profile {
    let mut profile = network_sim::scenarios::scenario_a();
    profile.timeline.links.truncate(1);
    profile.timeline.links[0].base = network_sim::ImpairmentConfig {
        delay_ms: Some(60),
        rate_kbit: Some(8_000),
        loss_percent: Some(1.0),
        tbf_shaping: true,
        queue_limit: Some(500),
        ..Default::default()
    };
    profile.offered_bps = 6_000_000;
    profile.warmup_offered_bps = 6_000_000;
    profile.timeline.events[0].action = Action::OfferedRate { bps: 6_000_000 };
    profile
}

impl Manifest {
    pub fn scenario(&self, id: &str) -> Result<Profile> {
        if id == "S-HSRSP" {
            ensure!(
                self.window_secs_override.is_none(),
                "HSRSP window is fixed at 20s"
            );
            ensure!(
                self.cells
                    .iter()
                    .filter(|c| c.scenario == id)
                    .all(|c| !c.covering),
                "HSRSP is noncovering"
            );
            return sls_profile();
        }
        if id == "S-FREEZE-NORDR" {
            ensure!(
                self.window_secs_override.is_none(),
                "freeze window is fixed at 45s"
            );
            let profile = freeze_profile();
            profile.validate()?;
            return Ok(profile);
        }
        if id == "SLS" {
            ensure!(
                self.cells
                    .iter()
                    .filter(|c| c.scenario == "SLS")
                    .all(|c| c.sink == "sls"),
                "SLS smoke profile is conformance-only"
            );
            ensure!(
                self.window_secs_override.is_none(),
                "SLS smoke window is fixed at 20s"
            );
            return sls_profile();
        }
        let mut profile = network_sim::scenarios::all()
            .into_iter()
            .find(|(name, _)| *name == id)
            .map(|(_, profile)| profile)
            .ok_or_else(|| ManifestError::UnknownScenario(id.into()))?;
        if let Some(seconds) = self.window_secs_override {
            profile.timeline.duration = Duration::from_secs(seconds);
        }
        profile.validate()?;
        Ok(profile)
    }

    pub fn cell_profile(&self, cell: &Cell) -> Result<Profile> {
        let mut profile = self.scenario(&cell.scenario)?;
        if let Some(mbit) = cell.offered_mbit_override {
            ensure!(
                mbit > 0 && mbit <= 1000 && !cell.covering && !cell.variant.is_empty(),
                "offered-rate overrides require a positive bounded rate and a noncovering variant"
            );
            let bps = u64::from(mbit) * 1_000_000;
            profile.offered_bps = bps;
            for event in &mut profile.timeline.events {
                if event.at.is_zero() && matches!(event.action, Action::OfferedRate { .. }) {
                    event.action = Action::OfferedRate { bps };
                }
            }
            if cell.scenario == "S-FREEZE-NORDR" {
                ensure!([2, 6].contains(&mbit), "freeze arms are 2 or 6 Mbit");
                profile.warmup_offered_bps = bps;
            }
            profile.validate()?;
        }
        Ok(profile)
    }
}
use std::time::Duration;
