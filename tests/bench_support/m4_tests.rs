use crate::manifest;

#[test]
fn m4_manifest_retains_546_outcomes_and_separate_soaks() {
    let manifest = manifest::parse(include_str!(
        "../../scripts/bench/manifests/m4a-ours-new.json"
    ))
    .unwrap();
    assert_eq!(manifest.cells.len(), 110);
    assert_eq!(manifest.order().len(), 546);
    assert!(manifest.retains_measured_timeouts());
    assert_eq!(manifest.cells.iter().filter(|c| c.covering).count(), 100);
    assert_eq!(manifest.cells.iter().filter(|c| c.fec).count(), 1);
    let soak = manifest::parse(include_str!("../../scripts/bench/manifests/m4-soak.json")).unwrap();
    assert_eq!(soak.order().len(), 2);
    assert!(
        soak.cells
            .iter()
            .all(|c| c.scenario == "M8" && c.runs == 1 && !c.covering)
    );
}

#[test]
fn m4_fec_requires_noncovering_enhanced_pair() {
    let mut raw: serde_json::Value = serde_json::from_str(include_str!(
        "../../scripts/bench/manifests/m4a-ours-new.json"
    ))
    .unwrap();
    raw["cells"][109]["covering"] = true.into();
    assert!(manifest::parse(&raw.to_string()).is_err());
}
