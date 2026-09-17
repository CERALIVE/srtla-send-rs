use crate::{manifest, manifest_json};

#[test]
fn m1_manifest_declares_all_195_planned_indices() {
    let parsed =
        manifest::parse(include_str!("../../scripts/bench/manifests/m1-ttl.json")).unwrap();
    assert_eq!(parsed.cells.len(), 65);
    assert_eq!(parsed.order().len(), 195);
    assert_eq!(
        parsed
            .cells
            .iter()
            .filter(|cell| cell.scenario == "S-FREEZE-NORDR")
            .count(),
        12
    );
}

#[test]
fn foreign_candidate_omits_unsupported_control_arguments() {
    // Given BELABOX's positional-only CLI.
    let mut json = manifest_json();
    json["candidates"][0]["control_socket"] = false.into();
    let parsed = manifest::parse(&json.to_string()).unwrap();
    // When constructing launch options, then no fork-only flags reach it.
    let args = parsed.candidates[0].runtime_args("/tmp/control", "/tmp/stats");
    assert!(args.is_empty());
}

#[test]
fn ordinary_candidate_retains_control_and_optional_stats_arguments() {
    // Given an ordinary fork candidate with stats enabled.
    let mut json = manifest_json();
    json["candidates"][0]["stats_file"] = true.into();
    let parsed = manifest::parse(&json.to_string()).unwrap();
    // When constructing launch options, then both owned paths are supplied.
    assert_eq!(
        parsed.candidates[0].runtime_args("/tmp/control", "/tmp/stats"),
        [
            "--control-socket",
            "/tmp/control",
            "--stats-file",
            "/tmp/stats"
        ]
    );
}
