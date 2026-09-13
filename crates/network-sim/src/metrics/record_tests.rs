use super::RunRecord;

#[test]
fn run_record_roundtrip_json() {
    // Given: a committed record with both impairment and source-load outcomes.
    let fixture = include_str!("../../fixtures/run-record-example.json");
    // When: parse and serialize through the public typed result schema.
    let record: RunRecord = serde_json::from_str(fixture).unwrap();
    let json = serde_json::to_string(&record).unwrap();
    let restored: RunRecord = serde_json::from_str(&json).unwrap();
    // Then: both result families and all embedded measurements survive.
    assert_eq!(record, restored);
    assert_eq!(restored.episodes.len(), 1);
    assert_eq!(restored.load_intervals.len(), 1);
    assert_eq!(restored.raw.sink_series.buckets().len(), 40);
    assert_eq!(
        restored.fingerprint,
        restored.compute_fingerprint().unwrap()
    );
}

#[test]
fn fingerprint_includes_process_effective_config() {
    // Given: identical candidate/receiver/profile/scenario identities.
    let fixture = include_str!("../../fixtures/run-record-example.json");
    let mut record: RunRecord = serde_json::from_str(fixture).unwrap();
    let before = record.compute_fingerprint().unwrap();
    // When: only the process-reported tuning changes.
    record.sender.effective_config = Some(super::control::EffectiveConfig {
        adaptive_features: None,
        adaptive_tuning: Some(serde_json::json!({"test": 0.4})),
    });
    // Then: runs cannot be falsely resumed across effective configuration changes.
    assert_ne!(before, record.compute_fingerprint().unwrap());
}

#[test]
fn malformed_run_identity_is_rejected_at_json_boundary() {
    // Given / When / Then: schema version, UUID, digest and window are validated.
    let fixture = include_str!("../../fixtures/run-record-example.json");
    for invalid in [
        fixture.replacen("\"schema_version\": 1", "\"schema_version\": 2", 1),
        fixture.replace("f2870e52-f63c-4acf-9228-d78692b3bf11", "not-a-uuid"),
        fixture.replacen("\"end_ms\": 40000", "\"end_ms\": -1", 1),
    ] {
        assert!(serde_json::from_str::<RunRecord>(&invalid).is_err());
    }
}

#[test]
fn example_summaries_recompute_from_embedded_raw_measurements() {
    // Given: the fixture is a small synthetic run, not merely structurally valid JSON.
    let record: RunRecord =
        serde_json::from_str(include_str!("../../fixtures/run-record-example.json")).unwrap();
    let profile = crate::profile::Profile {
        links: vec![crate::profile::LinkProfile {
            base: crate::ImpairmentConfig::default(),
            carrier: crate::bond::CarrierMode::Nat,
        }],
        events: record.events.iter().map(|e| e.event.clone()).collect(),
        duration: std::time::Duration::from_secs(40),
    };
    let log = crate::profile::EventLog {
        entries: record.events.clone(),
    };
    // When / Then: input evidence independently reproduces every primary summary.
    assert_eq!(
        record.useful_goodput_bps,
        record
            .raw
            .sink_series
            .useful_goodput_bps(record.window)
            .unwrap()
    );
    assert_eq!(
        record.viewer_loss_ratio,
        record
            .raw
            .stats_csv
            .as_ref()
            .unwrap()
            .window(record.window, 38000)
            .unwrap()
            .viewer_loss_ratio
    );
    assert_eq!(
        record.episodes,
        super::episodes::evaluate(&profile, &log, &record.raw.sink_series).unwrap()
    );
    assert_eq!(
        record.load_intervals,
        super::load_intervals::evaluate(&profile, &log, (&record.raw.sink_series, 8000)).unwrap()
    );
    assert_eq!(
        record.per_link,
        super::link_counters::shares(&record.raw.link_counters, record.window).unwrap()
    );
}
