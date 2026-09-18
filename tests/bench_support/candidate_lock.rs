use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Result, ensure};
use network_sim::metrics::identity::Hash256;
use serde::{Deserialize, Serialize};

use crate::manifest::Manifest;

#[path = "../support/locked_sender.rs"]
mod locked_sender;

pub fn requires_history(manifest: &Manifest) -> bool {
    let retired = |mode: &str| matches!(mode, "classic" | "rtt-threshold" | "edpf" | "adaptive");
    manifest.historical
        || manifest.candidates.iter().any(|candidate| {
            retired(&candidate.label)
                || candidate
                    .args
                    .windows(2)
                    .any(|args| args[0] == "--mode" && retired(&args[1]))
                || candidate
                    .args
                    .iter()
                    .any(|arg| arg.strip_prefix("--mode=").is_some_and(retired))
        })
}

pub fn resolve(path: &Path, manifest: &mut Manifest, from_lock: bool) -> Result<bool> {
    if !requires_history(manifest) {
        return Ok(false);
    }
    ensure!(
        from_lock,
        "historical campaign requires --historical-from-lock; current binaries are forbidden"
    );
    let lock = locked_sender::LockedSenders::read(path)?;
    let paths = manifest
        .candidates
        .iter()
        .map(|candidate| lock.verified(&manifest.campaign, &candidate.label))
        .collect::<Result<Vec<_>>>()?;
    for (candidate, path) in manifest.candidates.iter_mut().zip(paths) {
        candidate.bin = path;
    }
    Ok(true)
}

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
    #[serde(flatten)]
    metadata: BTreeMap<String, serde_json::Value>,
}

pub fn record(path: &Path, manifest: &Manifest) -> Result<()> {
    ensure!(
        !requires_history(manifest),
        "historical candidate locks are read-only"
    );
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
                    metadata: BTreeMap::new(),
                },
            ))
        })
        .collect::<Result<_>>()?;
    lock.candidates
        .insert(manifest.campaign.clone(), candidates);
    crate::checkpoint::atomic(path, &lock)
}

#[test]
fn lock_roundtrip_keeps_release_and_historical_provenance() {
    let bytes = br#"{"candidates":{"reverse-interop":{"ours-3.3.0":{"srtla_send_bin":"/release","srtla_send_sha256":"ebe34ee2bb6c8d832801e20ec32adc7cf8aba9c544915de751efad14f8f1d77c","built_from_source":false}}}}"#;
    let original: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let parsed: Lock = serde_json::from_slice(bytes).unwrap();
    assert_eq!(serde_json::to_value(parsed).unwrap(), original);
}
