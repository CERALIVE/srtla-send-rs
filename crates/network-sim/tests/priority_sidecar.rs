use network_sim::bond::PrioritySidecar;

#[test]
fn priority_sidecar_accepts_inclusive_bounds_and_sparse_rows() {
    // Given: positive and negative boundary values for a sparse selection.
    let json = r#"[{"link":2,"priority":-0.2},{"link":0,"priority":0.2}]"#;
    // When: parsing and projecting onto three topology indices.
    let priorities: PrioritySidecar = serde_json::from_str(json).unwrap();
    priorities.validate_links(3).unwrap();
    // Then: omitted links remain unconfigured, not zero-biased.
    assert_eq!(priorities.priority(0), Some(0.2));
    assert_eq!(priorities.priority(1), None);
    assert_eq!(priorities.priority(2), Some(-0.2));
}

#[test]
fn priority_sidecar_rejects_out_of_range_duplicate_and_unknown_rows() {
    // Given: invalid producer input classes.
    for json in [
        r#"[{"link":0,"priority":0.21}]"#,
        r#"[{"link":0,"priority":-0.21}]"#,
        r#"[{"link":0,"priority":0.1},{"link":0,"priority":0.2}]"#,
        r#"[{"link":0,"priority":0.1,"typo":1}]"#,
    ] {
        // When/Then: boundary parsing refuses the invalid sidecar.
        assert!(
            serde_json::from_str::<PrioritySidecar>(json).is_err(),
            "{json}"
        );
    }
}

#[test]
fn priority_sidecar_rejects_indices_outside_the_final_pool() {
    // Given: a priority naming a nonexistent topology index.
    let priorities: PrioritySidecar =
        serde_json::from_str(r#"[{"link":2,"priority":0.1}]"#).unwrap();
    // When/Then: reject before any namespaces or sender process exist.
    assert!(priorities.validate_links(2).is_err());
}
