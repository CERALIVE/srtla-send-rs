//! Explicit-cell, paired campaign harness. Live tests require Linux netns privileges.
#![cfg(unix)]

mod bench_support;
#[path = "bench_support/c1_contract.rs"]
mod c1_contract;
#[path = "bench_support/checkpoint.rs"]
mod checkpoint;
#[path = "bench_support/lineage_tests.rs"]
mod lineage_tests;
#[path = "bench_support/m1_tests.rs"]
mod m1_tests;
#[path = "bench_support/m2_tests.rs"]
mod m2_tests;
#[path = "bench_support/manifest.rs"]
mod manifest;
#[path = "support/measurement_lock.rs"]
mod measurement;

#[test]
fn conformance_smoke() -> anyhow::Result<()> {
    bench_support::conformance::smoke()
}

#[test]
#[ignore = "full campaign requires netns privileges and BENCH_MANIFEST"]
fn campaign() -> anyhow::Result<()> {
    bench_support::runner::campaign(false)
}

#[test]
#[ignore = "A+D smoke requires netns privileges and BENCH_MANIFEST; full scenario windows"]
fn smoke() -> anyhow::Result<()> {
    bench_support::runner::campaign(true)
}

#[test]
#[ignore = "private bounded single-run worker; launched by campaign"]
fn worker() -> anyhow::Result<()> {
    bench_support::runner::worker()
}

use network_sim::metrics::identity::Hash256;
use network_sim::metrics::record::RunStatus;

fn manifest_json() -> serde_json::Value {
    serde_json::json!({
        "campaign": "unit", "seed": 42,
        "candidates": [{"label":"base", "bin":"/bin/true"},
                       {"label":"next", "bin":"/bin/true"}],
        "receivers": [{"name":"ceralive", "bin":"/bin/true"}],
        "cells": [
            {"candidate":"base", "scenario":"A", "receiver":"ceralive", "srt_profile":"production", "runs":2},
            {"candidate":"next", "scenario":"A", "receiver":"ceralive", "srt_profile":"production", "runs":2},
            {"candidate":"next", "scenario":"D", "receiver":"ceralive", "srt_profile":"production", "runs":1}
        ]
    })
}

#[test]
fn manifest_parsing_requires_explicit_cells() {
    // Given a legacy Cartesian manifest without the authoritative matrix.
    let mut json = manifest_json();
    json.as_object_mut().unwrap().remove("cells");
    // When it crosses the manifest boundary, then it is rejected.
    assert!(manifest::parse(&json.to_string()).is_err());
}

#[test]
fn cell_enumeration_preserves_sparse_matrix() {
    // Given an explicitly sparse campaign.
    let manifest = manifest::parse(&manifest_json().to_string()).unwrap();
    // When enumerating work, then only requested cells and counts appear.
    let order = manifest.order();
    assert_eq!(order.len(), 5);
    assert_eq!(order.iter().filter(|r| r.cell == 2).count(), 1);
    assert_eq!(manifest.cells.len(), 3);
}

#[test]
fn interleaved_order_is_deterministic_for_fixed_seed() {
    // Given two arms of one scenario and a fixed seed.
    let manifest = manifest::parse(&manifest_json().to_string()).unwrap();
    // When ordering, then both candidates occur once before the next run index.
    let order = manifest.order();
    assert_eq!(order, manifest.order());
    assert_eq!(order[..2].iter().map(|r| r.run).collect::<Vec<_>>(), [0, 0]);
    assert_ne!(order[0].cell, order[1].cell);
    assert_eq!(
        order[2..4].iter().map(|r| r.run).collect::<Vec<_>>(),
        [1, 1]
    );
}

#[test]
fn unknown_scenario_is_typed_before_network_setup() {
    // Given an unknown scenario and deliberately unusable binary paths.
    let mut json = manifest_json();
    json["cells"][0]["scenario"] = "unknown".into();
    json["candidates"][0]["bin"] = "/does/not/exist".into();
    // When parsing, then scenario validation fails before binary or network work.
    let error = manifest::parse(&json.to_string()).unwrap_err();
    assert!(matches!(error.downcast_ref::<manifest::ManifestError>(),
        Some(manifest::ManifestError::UnknownScenario(id)) if id == "unknown"));
}

#[test]
fn l_enters_measurement_at_warmup_rate_then_switches() {
    // Given L, whose overload cannot satisfy a settle target at its measurement rate.
    let profile = network_sim::scenarios::scenario_l();
    let mut settle = manifest::Settle::for_profile(&profile);
    assert!(!settle.observe(1, 12_000_000.0));
    assert!(!settle.observe(2, 12_000_000.0));
    // When the third warm-up second arrives, then measurement starts with t=0 overload.
    assert!(settle.observe(3, 12_000_000.0));
    assert_eq!(manifest::measurement_rate(&profile).unwrap(), 20_000_000);
}

#[test]
fn smoke_uses_a_and_d_full_windows_for_all_candidates() {
    // Given a sparse manifest with a shorter override.
    let mut manifest = manifest::parse(&manifest_json().to_string()).unwrap();
    manifest.window_secs_override = Some(45);
    // When smoke is selected, then both complete scenarios run twice for each arm.
    manifest.smoke();
    assert_eq!(manifest.cells.len(), 4);
    assert!(manifest.cells.iter().all(|c| c.runs == 2));
    assert_eq!(manifest.window_secs_override, None);
    assert_eq!(
        manifest.scenario("D").unwrap().timeline.duration.as_secs(),
        75
    );
}

fn receipt(fingerprint: Hash256, status: RunStatus) -> checkpoint::Receipt {
    checkpoint::Receipt {
        fingerprint,
        status,
    }
}

#[test]
fn resume_retries_failed_run_and_counts_only_ok() {
    // Given a persisted failed attempt from a previous invocation.
    let dir = tempfile::tempdir().unwrap();
    let hash = Hash256::digest(b"current");
    let store = checkpoint::Store::new(dir.path(), hash.clone(), 2);
    checkpoint::atomic(
        &dir.path().join("run-0.failed-1.json"),
        &receipt(hash.clone(), RunStatus::Failed),
    )
    .unwrap();
    // When resuming, execute attempt 2 and atomically publish success.
    assert_eq!(store.next(0).unwrap(), checkpoint::Next::Attempt(2));
    store.save(0, 2, &receipt(hash, RunStatus::Ok)).unwrap();
    // Then the failed attempt never contributes to N.
    assert_eq!(store.next(0).unwrap(), checkpoint::Next::Complete);
    assert_eq!(store.count_ok(2).unwrap(), 1);
    assert!(dir.path().join("run-0.failed-1.json").exists());
}

#[test]
fn resume_rejects_stale_fingerprint() {
    // Given an ok result produced by a different binary/configuration.
    let dir = tempfile::tempdir().unwrap();
    checkpoint::atomic(
        &dir.path().join("run-0.json"),
        &receipt(Hash256::digest(b"old"), RunStatus::Ok),
    )
    .unwrap();
    let store = checkpoint::Store::new(dir.path(), Hash256::digest(b"new"), 2);
    // When resuming, then the old result is archived, not counted or overwritten.
    assert_eq!(store.next(0).unwrap(), checkpoint::Next::Attempt(1));
    assert_eq!(store.count_ok(1).unwrap(), 0);
    assert_eq!(
        std::fs::read_dir(dir.path().join("stale")).unwrap().count(),
        1
    );
}

#[test]
fn exhausted_after_max_retries() {
    // Given two failed attempts of this fingerprint.
    let dir = tempfile::tempdir().unwrap();
    let hash = Hash256::digest(b"same");
    let store = checkpoint::Store::new(dir.path(), hash.clone(), 2);
    let failed = receipt(hash, RunStatus::Failed);
    store.save(0, 1, &failed).unwrap();
    store.save(0, 2, &failed).unwrap();
    // When resuming, then exhaustion is durable but contributes no success.
    assert_eq!(store.next(0).unwrap(), checkpoint::Next::Exhausted);
    assert_eq!(store.count_ok(1).unwrap(), 0);
    assert!(dir.path().join("run-0.exhausted.json").exists());
}
