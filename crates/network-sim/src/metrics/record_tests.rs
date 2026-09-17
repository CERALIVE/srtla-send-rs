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
#[test]
fn sls_record_preserves_captured_publisher_keys() {
    // Given the captured real SLS publisher and a legacy metric record.
    let evidence: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../docs/evidence/bpc/sls-stats-map.json"
    ))
    .unwrap();
    let captured = &evidence["exact_response"]["publishers"]["publish/live/baseline"];
    let mut json: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/run-record-example.json")).unwrap();
    json["sink"] = "sls".into();
    json["metrics"] = "none".into();
    json["sls_stats"] = captured.clone();
    // When a RunRecord round-trips, then every captured field is retained.
    let record: super::RunRecord = serde_json::from_value(json).unwrap();
    let output = serde_json::to_value(record).unwrap();
    for (key, value) in captured.as_object().unwrap() {
        assert_eq!(&output["sls_stats"][key], value, "{key}");
    }
    assert_eq!(output["sink"], "sls");
    assert_eq!(output["metrics"], "none");
}
#[test]
fn sls_record_keeps_pre_ring_unknown_sentinels() {
    let raw = r#"{"players":[],"ingestDiscontinuities":-1,"maxReaderBacklogBytes":-1,"ringOverruns":-1,"sendBackpressure":-1,"viewerPktSndDrop":-1}"#;
    let stats: super::sls::SlsPublisherStats = serde_json::from_str(raw).unwrap();
    assert_eq!(stats.ingest_discontinuities, Some(-1));
    assert_eq!(stats.max_reader_backlog_bytes, Some(-1));
    assert_eq!(stats.ring_overruns, Some(-1));
    assert_eq!(stats.send_backpressure, Some(-1));
    assert_eq!(stats.viewer_pkt_snd_drop, Some(-1));
}
#[test]
fn sls_identity_changes_checkpoint_fingerprint_without_changing_legacy_identity() {
    let mut record: super::RunRecord =
        serde_json::from_str(include_str!("../../fixtures/run-record-example.json")).unwrap();
    let original = record.compute_fingerprint().unwrap();
    record.sls_identity = Some(super::sls::SlsIdentity {
        binary_sha256: super::identity::Hash256::digest(b"server"),
        template_sha256: super::identity::Hash256::digest(b"template"),
        libsrt_sha256: super::identity::Hash256::digest(b"libsrt"),
    });
    let sls = record.compute_fingerprint().unwrap();
    assert_ne!(original, sls);
    record.sls_identity.as_mut().unwrap().libsrt_sha256 =
        super::identity::Hash256::digest(b"other libsrt");
    assert_ne!(record.compute_fingerprint().unwrap(), sls);
    record.sls_identity = None;
    assert_eq!(record.compute_fingerprint().unwrap(), original);
}
