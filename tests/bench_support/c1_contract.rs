use std::collections::BTreeSet;

use crate::manifest;

fn c1() -> manifest::Manifest {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts/bench/manifests/c1-baselines.json");
    manifest::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn c1_keeps_all_seven_candidates_and_455_runs() {
    // Given the explicit C1 campaign, including the intentionally divergent upstream.
    let manifest = c1();
    let candidates = [
        "classic",
        "enhanced",
        "rtt-threshold",
        "edpf",
        "adaptive",
        "upstream-classic",
        "upstream-enhanced",
    ];
    let scenarios = [
        "A", "B1", "B2", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L",
    ];
    // When enumerating the requested matrix, then no arm or profile is missing.
    assert_eq!(manifest.seed, 20260913);
    assert_eq!(
        manifest
            .candidates
            .iter()
            .map(|c| c.label.as_str())
            .collect::<Vec<_>>(),
        candidates
    );
    let actual: BTreeSet<_> = manifest
        .cells
        .iter()
        .map(|c| {
            assert_eq!(c.receiver, "ceralive");
            assert_eq!(c.srt_profile, "production");
            assert_eq!(c.runs, 5);
            (c.candidate.as_str(), c.scenario.as_str())
        })
        .collect();
    let expected = candidates
        .into_iter()
        .flat_map(|c| scenarios.map(|s| (c, s)))
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(manifest.order().len(), 455);
}

#[test]
fn c1_upstream_uses_only_its_supported_instrumentation() {
    // Given df0b393, which supports a control socket but not --stats-file or adaptive metrics.
    let manifest = c1();
    // When selecting both external arms, then no fork-only flag/expectation is requested.
    for label in ["upstream-classic", "upstream-enhanced"] {
        let candidate = manifest
            .candidates
            .iter()
            .find(|c| c.label == label)
            .unwrap();
        assert!(
            !candidate.stats_file,
            "{label} must not receive --stats-file"
        );
        assert_eq!(candidate.effective_config, None);
        assert!(candidate.env.is_empty());
        assert_eq!(
            candidate.args,
            ["--mode", label.strip_prefix("upstream-").unwrap()]
        );
    }
}

#[test]
fn c1_fork_modes_declare_the_test_builds_observed_configuration() {
    // Given the test-internals artifact, whose metrics expose config in every mode.
    let manifest = c1();
    let expected = serde_json::from_value(serde_json::json!({
        "adaptive_features": ["stall", "loss", "queue", "deadline", "rejoin", "sole", "pref", "ratecap"],
        "adaptive_tuning": {"stall_attempts": 32, "loss_enter": 0.1,
                            "deadline_hold_fraction": 0.5, "ratecap_loss_backoff": 0.85}
    })).unwrap();
    // When selecting fork arms, then expectations match without altering legacy mode env.
    for label in ["classic", "enhanced", "rtt-threshold", "edpf", "adaptive"] {
        let candidate = manifest
            .candidates
            .iter()
            .find(|c| c.label == label)
            .unwrap();
        assert!(candidate.stats_file);
        assert_eq!(candidate.effective_config.as_ref(), Some(&expected));
        assert_eq!(candidate.args, ["--mode", label]);
        if label == "adaptive" {
            assert_eq!(candidate.env.len(), 1);
            assert_eq!(candidate.env["SRTLA_ADAPTIVE_FEATURES"], "all");
        } else {
            assert!(candidate.env.is_empty());
        }
        assert_eq!(candidate.bin, manifest.candidates[0].bin);
    }
}
