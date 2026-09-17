use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use network_sim::metrics::identity::Hash256;
use serde::{Deserialize, Serialize};

use crate::manifest::Manifest;

#[derive(Default, Deserialize, Serialize)]
struct Lock {
    #[serde(default)]
    candidates: BTreeMap<String, BTreeMap<String, CandidateArtifact>>,
    #[serde(flatten)]
    metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize, Serialize)]
struct CandidateArtifact {
    srtla_send_bin: PathBuf,
    srtla_send_sha256: Hash256,
}

pub fn record(path: &Path, manifest: &Manifest) -> Result<()> {
    let mut lock: Lock = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Lock::default(),
        Err(error) => return Err(error.into()),
    };
    let candidates = manifest
        .candidates
        .iter()
        .map(|candidate| {
            Ok((
                candidate.label.clone(),
                CandidateArtifact {
                    srtla_send_bin: candidate.bin.clone(),
                    srtla_send_sha256: super::record::binary_hash(&candidate.bin)?,
                },
            ))
        })
        .collect::<Result<_>>()?;
    lock.candidates
        .insert(manifest.campaign.clone(), candidates);
    crate::checkpoint::atomic(path, &lock)
}
