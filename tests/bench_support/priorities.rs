use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use network_sim::profile::Action;

use super::Manifest;

pub(super) fn validate(manifest: &Manifest) -> Result<()> {
    let mut paired = BTreeMap::new();
    for cell in &manifest.cells {
        ensure!(
            paired
                .insert(&cell.cell_id, &cell.priority_sidecar)
                .is_none_or(|previous| previous == &cell.priority_sidecar),
            "paired candidates must agree on priority_sidecar; use distinct variants for \
             different policies"
        );
        if let Some(priorities) = &cell.priority_sidecar {
            let profile = manifest.scenario(&cell.scenario)?;
            let additions = profile
                .timeline
                .expanded_events()?
                .iter()
                .filter(|event| matches!(event.action, Action::AddLink(_)))
                .count();
            priorities.validate_links(profile.timeline.links.len() + additions)?;
        }
    }
    Ok(())
}
