use crate::{manifest, manifest_json};

#[test]
fn m1_offered_override_is_diagnostic_and_preserves_catalog_warmup() {
    // Given an overloaded B1 diagnostic, not a mutated catalog entry.
    let mut json = manifest_json();
    json["cells"][0]["scenario"] = "B1".into();
    json["cells"][0]["variant"] = "offered-24".into();
    json["cells"][0]["covering"] = false.into();
    json["cells"][0]["offered_mbit_override"] = 24.into();
    // When resolving the per-cell profile, then only the measurement rate changes.
    let parsed = manifest::parse(&json.to_string()).unwrap();
    let profile = parsed.cell_profile(&parsed.cells[0]).unwrap();
    assert_eq!(profile.offered_bps, 24_000_000);
    assert_eq!(profile.warmup_offered_bps, 9_600_000);
    assert_eq!(manifest::measurement_rate(&profile).unwrap(), 24_000_000);
    assert_eq!(network_sim::scenarios::scenario_b1().offered_bps, 9_600_000);
}

#[test]
fn m1_freeze_diagnostic_has_one_ordered_lossy_link_and_low_rate_warmup() {
    // Given the low-pps freeze arm.
    let mut json = manifest_json();
    json["cells"][0]["scenario"] = "S-FREEZE-NORDR".into();
    json["cells"][0]["variant"] = "offered-2".into();
    json["cells"][0]["covering"] = false.into();
    json["cells"][0]["offered_mbit_override"] = 2.into();
    // When resolving, then no jitter, GE, extra link or overload is inherited.
    let parsed = manifest::parse(&json.to_string()).unwrap();
    let profile = parsed.cell_profile(&parsed.cells[0]).unwrap();
    assert_eq!(profile.timeline.links.len(), 1);
    let link = &profile.timeline.links[0].base;
    assert_eq!(link.delay_ms, Some(60));
    assert_eq!(link.rate_kbit, Some(8_000));
    assert_eq!(link.loss_percent, Some(1.0));
    assert_eq!(link.gemodel, None);
    assert_eq!(link.jitter_ms, None);
    assert_eq!(profile.warmup_offered_bps, 2_000_000);
    assert_eq!(profile.offered_bps, 2_000_000);
}

#[test]
fn manifest_sls_accepts_only_explicit_noncovering_conformance_ports() {
    // Given each bonded alias as an explicit conformance-only cell.
    for port in [4002, 4003] {
        let mut json = manifest_json();
        json["cells"] = serde_json::json!([{
            "candidate":"base", "scenario":"A", "receiver":"ceralive",
            "srt_profile":"strict", "runs":1, "sink":"sls", "port":port,
            "metrics":"none", "covering":false
        }]);
        // When parsing, then the sink/port remain part of the canonical identity.
        let parsed = manifest::parse(&json.to_string()).unwrap();
        assert!(parsed.cells[0].cell_id.contains(&format!("sls:{port}")));
        for (key, value) in [
            ("port", serde_json::json!(4001)),
            ("covering", serde_json::json!(true)),
            ("metrics", serde_json::json!("full")),
            ("fec", serde_json::json!(true)),
        ] {
            let mut invalid = json.clone();
            invalid["cells"][0][key] = value;
            assert!(manifest::parse(&invalid.to_string()).is_err(), "{invalid}");
        }
    }
}

#[test]
fn manifest_legacy_string_receiver_parses() {
    // Given a legacy string receiver.
    let mut json = manifest_json();
    json["receivers"] = serde_json::json!(["ceralive"]);
    // When parsing, then its label remains usable by legacy cells.
    let parsed = manifest::parse(&json.to_string()).unwrap();
    assert_eq!(parsed.receivers[0].name, "ceralive");
}

#[test]
fn manifest_lineage_object_receiver_parses() {
    // Given an explicit receiver lineage with independent binaries and options.
    let mut json = manifest_json();
    json["receivers"] = serde_json::json!([{
        "label":"ceralive", "lineage":"ours-new", "srtla_rec_kind":"ceralive",
        "srtla_rec_bin":"/bin/true", "srt_live_transmit_bin":"/bin/false",
        "listener_uri_extra":"&periodicnakgate=1&lossmaxttl=200"
    }]);
    // When parsing, then all per-cell overrides survive.
    let parsed = manifest::parse(&json.to_string()).unwrap();
    let receiver = &parsed.receivers[0];
    assert_eq!(receiver.lineage.as_deref(), Some("ours-new"));
    assert_eq!(
        receiver.srt_live_transmit_bin.as_deref(),
        Some(std::path::Path::new("/bin/false"))
    );
    assert_eq!(
        receiver.listener_uri_extra,
        "&periodicnakgate=1&lossmaxttl=200"
    );
}

#[test]
fn manifest_lock_reference_resolves_both_binaries() {
    // Given a real temporary lock document, without relying on installed binaries.
    let dir = tempfile::tempdir().unwrap();
    let lock = dir.path().join("receivers.lock.json");
    std::fs::write(&lock, r#"{"receivers":{"ours-new":{"srtla_rec_bin":"/bin/true","srt_live_transmit_bin":"/bin/false"}}}"#).unwrap();
    let mut json = manifest_json();
    json["receivers"] = serde_json::json!([{
        "label":"ceralive", "lineage":"ours-new", "srtla_rec_kind":"ceralive",
        "srtla_rec_bin":"lock:ours-new", "srt_live_transmit_bin":"lock:ours-new"
    }]);
    // When resolving the manifest, then each reference selects its corresponding binary.
    let parsed = manifest::parse_with_lock(&json.to_string(), &lock).unwrap();
    assert_eq!(
        parsed.receivers[0].bin.as_deref(),
        Some(std::path::Path::new("/bin/true"))
    );
    assert_eq!(
        parsed.receivers[0].srt_live_transmit_bin.as_deref(),
        Some(std::path::Path::new("/bin/false"))
    );
}

#[test]
fn manifest_missing_lock_entry_names_the_reference() {
    // Given a missing lock file and a named reference.
    let dir = tempfile::tempdir().unwrap();
    let mut json = manifest_json();
    json["receivers"][0]["bin"] = "lock:does-not-exist".into();
    // When parsing, then the error identifies the missing receiver entry.
    let error =
        manifest::parse_with_lock(&json.to_string(), &dir.path().join("missing.json")).unwrap_err();
    assert!(format!("{error:#}").contains("does-not-exist"));
}

#[test]
fn manifest_cell_identity_excludes_candidate_but_includes_receiver_options() {
    // Given paired candidates and an independently varied receiver option.
    let json = manifest_json();
    let original = manifest::parse(&json.to_string()).unwrap();
    let mut changed = json;
    changed["receivers"][0]["listener_uri_extra"] = "&lossmaxttl=200".into();
    // When normalizing cell identities, then candidates pair but options do not collide.
    let changed = manifest::parse(&changed.to_string()).unwrap();
    assert_eq!(original.cells[0].cell_id, original.cells[1].cell_id);
    assert_ne!(original.cells[0].cell_id, changed.cells[0].cell_id);
    assert!(original.cells[0].cell_id.contains("slt:4001--fec:off"));
}

#[test]
fn manifest_receiver_override_replaces_defaults_without_environment_mutation() {
    let json = manifest_json();
    let parsed = manifest::parse(&json.to_string()).unwrap();
    let defaults = network_sim::harness::ReceiverSpec {
        label: "belabox".into(),
        lineage: "belabox".into(),
        srtla_rec_kind: "belabox".into(),
        srtla_rec_bin: "/missing/default".into(),
        srt_live_transmit_bin: "/bin/false".into(),
        listener_uri_extra: String::new(),
    };
    let resolved = parsed.receivers[0].resolve(&defaults).unwrap();
    assert_eq!(
        resolved.srtla_rec_bin,
        std::fs::canonicalize("/bin/true").unwrap()
    );
    assert_eq!(
        resolved.srt_live_transmit_bin,
        std::fs::canonicalize("/bin/false").unwrap()
    );
    assert_eq!(resolved.srtla_rec_kind, "ceralive");
}

#[test]
fn manifest_listener_override_replaces_query_key_without_changing_presets() {
    let spec = network_sim::harness::ReceiverSpec {
        label: "ours-new".into(),
        lineage: "ours-new".into(),
        srtla_rec_kind: "ceralive".into(),
        srtla_rec_bin: "/bin/true".into(),
        srt_live_transmit_bin: "/bin/false".into(),
        listener_uri_extra: "&lossmaxttl=200&periodicnakgate=1".into(),
    };
    let args = spec
        .listener_argv(network_sim::harness::SrtProfile::PRODUCTION, 4002, None)
        .unwrap();
    assert_eq!(
        args[0],
        "srt://:4002?mode=listener&latency=2000&reorderfreeze=1&lossmaxttl=200&periodicnakgate=1"
    );
    assert_eq!(network_sim::harness::SrtProfile::PRODUCTION.lossmaxttl, 40);
}

#[test]
fn manifest_candidate_lock_records_campaign_hashes_and_preserves_receiver_metadata() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("receivers.lock.json");
    std::fs::write(
        &path,
        r#"{"receivers":{"ours-new":{"revision":"pinned"}},"parallel_lanes":2}"#,
    )
    .unwrap();
    let manifest = manifest::parse(&manifest_json().to_string()).unwrap();
    crate::bench_support::candidate_lock::record(&path, &manifest).unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["receivers"]["ours-new"]["revision"], "pinned");
    assert_eq!(saved["parallel_lanes"], 2);
    assert_eq!(
        saved["candidates"]["unit"]["base"]["srtla_send_sha256"],
        serde_json::to_value(
            crate::bench_support::record::binary_hash(std::path::Path::new("/bin/true")).unwrap()
        )
        .unwrap()
    );
}

#[test]
fn manifest_smoke_preserves_cell_options_and_covering() {
    let mut json = manifest_json();
    for cell in json["cells"].as_array_mut().unwrap() {
        cell["port"] = 4002.into();
        cell["covering"] = false.into();
        cell["variant"] = "parser-fixture".into();
    }
    let mut manifest = manifest::parse(&json.to_string()).unwrap();
    manifest.smoke();
    assert!(
        manifest
            .cells
            .iter()
            .all(|c| c.port == 4002 && !c.covering && c.variant == "parser-fixture")
    );
}

#[test]
fn manifest_receiver_binary_path_is_not_a_shell_expression_or_directory() {
    let mut spec = network_sim::harness::ReceiverSpec {
        label: "test".into(),
        lineage: "ours-new".into(),
        srtla_rec_kind: "ceralive".into(),
        srtla_rec_bin: ".".into(),
        srt_live_transmit_bin: "/bin/true".into(),
        listener_uri_extra: String::new(),
    };
    assert!(spec.clone().resolved().is_err());
    spec.srtla_rec_bin = "$(true)".into();
    assert!(spec.resolved().is_err());
}
