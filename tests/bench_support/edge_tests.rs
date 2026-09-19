use network_sim::metrics::control::EffectiveConfig;
use network_sim::metrics::identity::Hash256;
use network_sim::metrics::record::RunStatus;

use super::record::{RunFailure, binary_hash, check_config};
use crate::checkpoint::{self, Next, Store};
use crate::{manifest, manifest_json, receipt};

#[test]
fn requested_configuration_must_be_observed_not_inferred() {
    // Given requested treatment knobs that differ from the sender's actual reply.
    let requested = EffectiveConfig {
        adaptive_features: Some(serde_json::json!({"probe":true})),
        adaptive_tuning: None,
    };
    let observed = EffectiveConfig {
        adaptive_features: Some(serde_json::json!({"probe":false})),
        adaptive_tuning: None,
    };
    // When comparing, then a typed config_mismatch rejects the run.
    let error = check_config(Some(&requested), Some(&observed)).unwrap_err();
    assert!(matches!(
        error.downcast_ref::<RunFailure>(),
        Some(RunFailure::ConfigMismatch)
    ));
}

#[test]
fn unsupported_metrics_cannot_satisfy_requested_features() {
    // Given a manifest requiring feature verification but an unsupported metrics endpoint.
    let requested = EffectiveConfig {
        adaptive_features: Some(serde_json::json!({"probe":true})),
        adaptive_tuning: None,
    };
    // When compared, then absence fails rather than becoming a zero/default configuration.
    assert!(check_config(Some(&requested), None).is_err());
}

#[test]
fn rebuilt_binary_changes_checkpoint_identity() {
    // Given an ok checkpoint fingerprinted from the original binary bytes.
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("candidate");
    std::fs::write(&binary, b"original").unwrap();
    let old = binary_hash(&binary).unwrap();
    checkpoint::atomic(&dir.path().join("run-0.json"), &receipt(old, RunStatus::Ok)).unwrap();
    // When the binary is rebuilt, then resume refuses the old success.
    std::fs::write(&binary, b"rebuilt").unwrap();
    let store = Store::new(dir.path(), binary_hash(&binary).unwrap(), 2);
    assert_eq!(store.next(0).unwrap(), Next::Attempt(1));
}

#[test]
fn stale_failures_do_not_exhaust_new_fingerprint() {
    // Given exhausted attempts belonging to a different configuration.
    let dir = tempfile::tempdir().unwrap();
    let old = Store::new(dir.path(), Hash256::digest(b"old"), 2);
    let failed = receipt(Hash256::digest(b"old"), RunStatus::Failed);
    old.save(0, 2, &failed).unwrap();
    assert_eq!(old.next(0).unwrap(), Next::Exhausted);
    // When a new configuration resumes, then its retry budget starts fresh.
    let new = Store::new(dir.path(), Hash256::digest(b"new"), 2);
    assert_eq!(new.next(0).unwrap(), Next::Attempt(1));
}

#[test]
fn invalid_campaign_exits_before_creating_output_or_network() {
    // Given the real campaign entry point and an unknown scenario.
    let dir = tempfile::tempdir().unwrap();
    let mut json = manifest_json();
    json["cells"][0]["scenario"] = "unknown".into();
    // When invoked as a subprocess, then typed validation fails before output setup.
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "campaign", "--ignored", "--nocapture"])
        .env("BENCH_MANIFEST", json.to_string())
        .env("BENCH_OUT_DIR", dir.path().join("out"))
        .env_remove("BENCH_SMOKE")
        .env_remove("BENCH_WORKER_REQUEST")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!dir.path().join("out").exists());
}

#[test]
fn duplicate_cells_are_rejected_instead_of_running_twice() {
    // Given two identical explicit cells.
    let mut json = manifest_json();
    let duplicate = json["cells"][0].clone();
    json["cells"].as_array_mut().unwrap().push(duplicate);
    // When parsed, then the ambiguous checkpoint destination is rejected.
    assert!(manifest::parse(&json.to_string()).is_err());
}

#[test]
fn missing_or_repeated_seconds_do_not_satisfy_settle() {
    // Given one high-rate second followed by a gap and a repeated sample.
    let mut settle = manifest::Settle::new(1_000_000);
    assert!(!settle.observe(1, 1_000_000.0));
    assert!(!settle.observe(3, 1_000_000.0));
    assert!(!settle.observe(3, 1_000_000.0));
    // When the next consecutive second arrives, then only two valid seconds exist.
    assert!(!settle.observe(4, 1_000_000.0));
}

#[test]
fn different_seeds_change_candidate_order() {
    // Given identical cells with different pairing seeds.
    let mut manifest = manifest::parse(&manifest_json().to_string()).unwrap();
    // When ordering each seed, then randomization actually changes the first pair.
    let orders: std::collections::BTreeSet<_> = (0..16)
        .map(|seed| {
            manifest.seed = seed;
            manifest.order()[..2]
                .iter()
                .map(|work| work.cell)
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(orders.len(), 2);
}

#[test]
fn full_run_record_survives_checkpoint_retry_roundtrip() {
    // Given a real RunRecord with embedded receiver/sink evidence and a failed attempt.
    let mut record: network_sim::metrics::RunRecord = serde_json::from_str(include_str!(
        "../../crates/network-sim/fixtures/run-record-example.json"
    ))
    .unwrap();
    record.fingerprint = record.compute_fingerprint().unwrap();
    record.status = RunStatus::Failed;
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path(), record.fingerprint.clone(), 2);
    store.save(0, 1, &record).unwrap();
    assert_eq!(store.next(0).unwrap(), Next::Attempt(2));
    // When the retry succeeds, then the canonical document is losslessly readable as RunRecord.
    record.status = RunStatus::Ok;
    let attempt = super::record::Attempt {
        record: record.clone(),
        attempt: 2,
        reason: None,
        detail: None,
    };
    store.save(0, 2, &attempt).unwrap();
    let actual: network_sim::metrics::RunRecord =
        serde_json::from_slice(&std::fs::read(directory.path().join("run-0.json")).unwrap())
            .unwrap();
    assert_eq!(actual, record);
    assert_eq!(store.count_ok(1).unwrap(), 1);
}
