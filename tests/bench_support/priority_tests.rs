use super::parse;

#[test]
fn manifest_preserves_optional_priority_sidecar() {
    // Given: two paired cells with the same per-link policy.
    let mut json = crate::manifest_json();
    for i in [0, 1] {
        json["cells"][i]["priority_sidecar"] = serde_json::json!([{"link":0,"priority":0.2}]);
    }
    // When: parsing the real campaign boundary.
    let manifest = parse(&json.to_string()).unwrap();
    let serialized = serde_json::to_value(&manifest).unwrap();
    // Then: configured values survive and legacy cells materialize no field.
    assert_eq!(
        serialized["cells"][0]["priority_sidecar"][0]["priority"],
        0.2
    );
    assert!(serialized["cells"][2].get("priority_sidecar").is_none());
}

#[test]
fn manifest_rejects_priority_outside_scenario_links() {
    // Given: a priority whose index exceeds Scenario A's three links.
    let mut json = crate::manifest_json();
    for i in [0, 1] {
        json["cells"][i]["priority_sidecar"] = serde_json::json!([{"link":3,"priority":0.2}]);
    }
    // When/Then: fail at validation, not during namespace setup.
    assert!(parse(&json.to_string()).is_err());
}

#[test]
fn paired_cells_cannot_silently_use_different_priorities() {
    // Given: the same canonical cell identity but different link policies.
    let mut json = crate::manifest_json();
    json["cells"][0]["priority_sidecar"] = serde_json::json!([{"link":0,"priority":0.2}]);
    // When/Then: refuse a confounded candidate comparison.
    assert!(parse(&json.to_string()).is_err());
}
