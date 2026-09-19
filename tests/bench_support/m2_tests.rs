use std::time::Duration;

use crate::manifest;

#[test]
fn m2_matrix_keeps_catalog_windows_and_four_diagnostic_lineages() {
    let manifest =
        manifest::parse(include_str!("../../scripts/bench/manifests/m2-sender.json")).unwrap();
    assert_eq!(manifest.order().len(), 47);
    assert_eq!(manifest.cells.len(), 19);
    for cell in &manifest.cells {
        let profile = manifest.cell_profile(cell).unwrap();
        if cell.scenario == "S-HSRSP" {
            assert_eq!(profile.timeline.duration, Duration::from_secs(20));
            assert!(!cell.covering);
        } else {
            let catalog = network_sim::scenarios::all()
                .into_iter()
                .find(|(id, _)| *id == cell.scenario)
                .unwrap()
                .1;
            assert_eq!(profile.timeline.duration, catalog.timeline.duration);
            assert_eq!(profile.offered_bps, catalog.offered_bps);
        }
    }
}
