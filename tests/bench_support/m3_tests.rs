use crate::manifest;

#[test]
#[ignore = "M3 idle control-wire supplement; requires M3_IDLE_DIR and privileges"]
fn m3_idle_conformance() -> anyhow::Result<()> {
    use std::path::PathBuf;
    use std::time::Duration;

    use crate::bench_support::record::{Attempt, Request, prepare};
    use crate::bench_support::stack::Stack;

    let _guard = crate::measurement::measurement_lock();
    let output = PathBuf::from(std::env::var("M3_IDLE_DIR")?);
    let mut manifest = manifest::parse(include_str!(
        "../../scripts/bench/manifests/m3-interop.json"
    ))?;
    manifest.receiver_defaults = Some(network_sim::harness::ReceiverSpec::from_env()?);
    for (index, cell) in manifest
        .cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.variant == "interop-conformance")
    {
        let (record, receiver_spec) = prepare(
            &manifest,
            manifest::Work {
                cell: index,
                run: 0,
            },
            0,
        )?;
        let artifacts = output.join(&cell.candidate);
        std::fs::create_dir_all(&artifacts)?;
        let mut request = Request {
            manifest: manifest.clone(),
            work: manifest::Work {
                cell: index,
                run: 0,
            },
            result: Attempt {
                record,
                attempt: 1,
                reason: None,
                detail: None,
            },
            artifacts,
            srt_binary: receiver_spec.srt_live_transmit_bin.clone(),
            receiver_spec,
        };
        request.result.record.raw.stats_csv_path = request.artifacts.join("receiver.csv");
        request.result.record.raw.sink_series_path = request.artifacts.join("sink.csv");
        let mut stack = Stack::start(&request, &manifest.cell_profile(cell)?)?;
        std::thread::sleep(Duration::from_secs(10));
        stack.logs(&request.artifacts)?;
        stack.finish_pcaps(true)?;
        crate::checkpoint::atomic(
            &request.artifacts.join("identity.json"),
            &request.result.record,
        )?;
    }
    Ok(())
}

#[test]
fn m3_matrix_contains_all_quadrants_and_foreign_conformance() {
    let manifest = manifest::parse(include_str!(
        "../../scripts/bench/manifests/m3-interop.json"
    ))
    .unwrap();
    assert_eq!(manifest.cells.len(), 43);
    assert_eq!(manifest.order().len(), 123);
    for candidate in &manifest.candidates {
        for receiver in &manifest.receivers {
            for scenario in ["B1", "G", "C", "M1"] {
                let cell = manifest
                    .cells
                    .iter()
                    .find(|cell| {
                        cell.candidate == candidate.label
                            && cell.receiver == receiver.name
                            && cell.scenario == scenario
                    })
                    .unwrap();
                assert_eq!(cell.runs, 3);
                let actual = manifest.cell_profile(cell).unwrap();
                let expected = network_sim::scenarios::all()
                    .into_iter()
                    .find(|(id, _)| *id == scenario)
                    .unwrap()
                    .1;
                assert_eq!(actual.timeline.duration, expected.timeline.duration);
                assert_eq!(actual.offered_bps, expected.offered_bps);
            }
        }
    }
    assert_eq!(
        manifest
            .cells
            .iter()
            .filter(|cell| cell.variant == "interop-conformance"
                && cell.scenario == "I"
                && !cell.covering)
            .count(),
        3
    );
}

#[test]
fn m3_retains_measured_failures_without_changing_ordinary_campaigns() {
    // Given a synthetic ordinary manifest.
    let mut manifest = manifest::parse(&crate::manifest_json().to_string()).unwrap();
    assert!(!manifest.retains_measured_timeouts());
    // When selecting the explicitly bounded M3 measurement campaign.
    manifest.campaign = "m3-interop".into();
    // Then a settling failure still supplies a full measurement, never a retry.
    assert!(manifest.retains_measured_timeouts());
}

#[test]
fn released_semver_candidate_label_is_accepted() {
    // Given the exact rollout-population identity.
    let mut json = crate::manifest_json();
    json["candidates"][0]["label"] = "ours-3.3.0".into();
    json["cells"][0]["candidate"] = "ours-3.3.0".into();
    // When parsing the manifest, then the release identity survives verbatim.
    assert_eq!(
        manifest::parse(&json.to_string()).unwrap().candidates[0].label,
        "ours-3.3.0"
    );
}
