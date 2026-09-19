use std::path::Path;

use super::candidate_lock;
use crate::manifest;

fn historical() -> manifest::Manifest {
    let mut json = crate::manifest_json();
    json["historical"] = true.into();
    manifest::parse(&json.to_string()).unwrap()
}

#[test]
fn historical_execution_requires_explicit_opt_in_before_reading_binaries() {
    // Given a historical manifest and absent artifacts.
    let mut manifest = historical();
    // When execution has not opted into the lock.
    let error =
        candidate_lock::resolve(Path::new("missing-lock"), &mut manifest, false).unwrap_err();
    // Then rejection names the required switch rather than trying a current binary.
    assert!(error.to_string().contains("--historical-from-lock"));
}

#[test]
fn deleted_mode_requires_opt_in_even_without_historical_marker() {
    // Given the immutable owner C1-style manifest shape without a marker.
    let mut manifest = manifest::parse(&crate::manifest_json().to_string()).unwrap();
    manifest.candidates[0].args = vec!["--mode=classic".into()];
    // When running without historical authorization.
    let error =
        candidate_lock::resolve(Path::new("missing-lock"), &mut manifest, false).unwrap_err();
    // Then omission of the marker cannot bypass the gate.
    assert!(error.to_string().contains("--historical-from-lock"));
}

#[test]
fn historical_execution_uses_locked_path_and_hash_not_manifest_path() {
    // Given a valid locked binary whose path differs from the manifest.
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("sender");
    std::fs::write(&binary, b"locked historical bytes").unwrap();
    let hash = network_sim::metrics::identity::Hash256::digest(b"locked historical bytes");
    let lock = dir.path().join("lock.json");
    std::fs::write(
        &lock,
        serde_json::json!({"candidates":{"unit":{
            "base":{"srtla_send_bin":binary,"srtla_send_sha256":hash},
            "next":{"srtla_send_bin":binary,"srtla_send_sha256":hash}
        }}})
        .to_string(),
    )
    .unwrap();
    let mut manifest = historical();
    // When opting in to historical execution.
    assert!(candidate_lock::resolve(&lock, &mut manifest, true).unwrap());
    // Then both executable paths come from the verified lock.
    let locked = std::fs::canonicalize(&binary).unwrap();
    assert!(
        manifest
            .candidates
            .iter()
            .all(|candidate| candidate.bin == locked)
    );
}

#[test]
fn historical_execution_rejects_modified_locked_bytes() {
    // Given a binary that no longer matches its committed checksum.
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("sender");
    std::fs::write(&binary, b"changed").unwrap();
    let lock = dir.path().join("lock.json");
    std::fs::write(
        &lock,
        serde_json::json!({"candidates":{"unit":{
            "base":{"srtla_send_bin":binary,"srtla_send_sha256":
                network_sim::metrics::identity::Hash256::digest(b"original")}
        }}})
        .to_string(),
    )
    .unwrap();
    // When resolving it with explicit opt-in.
    let error = candidate_lock::resolve(&lock, &mut historical(), true).unwrap_err();
    // Then opt-in never waives artifact integrity.
    assert!(error.to_string().contains("hash mismatch"));
}

#[test]
fn historical_execution_refuses_missing_candidate_lock() {
    // Given an empty candidate lock and a historical manifest.
    let dir = tempfile::tempdir().unwrap();
    let lock = dir.path().join("lock.json");
    std::fs::write(&lock, br#"{"candidates":{}}"#).unwrap();
    // When resolving, then no fallback path is invented.
    assert!(
        candidate_lock::resolve(&lock, &mut historical(), true)
            .unwrap_err()
            .to_string()
            .contains("missing historical candidate lock")
    );
}
