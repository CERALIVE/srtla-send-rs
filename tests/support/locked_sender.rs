use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use network_sim::metrics::identity::Hash256;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LockedSenders {
    candidates: BTreeMap<String, BTreeMap<String, Artifact>>,
}

#[derive(Deserialize)]
struct Artifact {
    srtla_send_bin: PathBuf,
    srtla_send_sha256: Hash256,
}

impl LockedSenders {
    pub fn read(path: &Path) -> Result<Self> {
        Ok(serde_json::from_slice(&std::fs::read(path)?)?)
    }

    pub fn verified(&self, campaign: &str, label: &str) -> Result<PathBuf> {
        let artifact = self
            .candidates
            .get(campaign)
            .and_then(|entries| entries.get(label))
            .with_context(|| format!("missing historical candidate lock: {campaign}/{label}"))?;
        let path = artifact.srtla_send_bin.canonicalize()?;
        ensure!(
            Hash256::digest(&std::fs::read(&path)?) == artifact.srtla_send_sha256,
            "historical candidate hash mismatch: {campaign}/{label}"
        );
        Ok(path)
    }
}
