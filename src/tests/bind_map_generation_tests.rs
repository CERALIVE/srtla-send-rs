//! Bind-map generation ordering, the telemetry reason set, and the retry
//! schedule's constants (ADR-003 §3, §4.2, §6.4).
//!
//! Scope: how two mappings are ordered against each other, and the frozen
//! vocabulary a rejection leaves the process as.
use assert_matches::assert_matches;

use crate::bind_map::{
    BIND_MAP_RETRY_ATTEMPTS, BIND_MAP_RETRY_BUDGET_MS, BIND_MAP_RETRY_DELAY_MS,
    BIND_MAP_SCHEMA_VERSION, BindMapError, DegradedReason, IpsFile, MappedPool, StaticIfaces,
    ValidateCtx, parse_sidecar, validate_pair,
};

// ---- fixtures ---------------------------------------------------------

fn ifaces(names: &[&str]) -> StaticIfaces {
    StaticIfaces::new(names.iter().copied())
}

/// A two-modem twin bond: one IP, two interfaces — the case the whole ADR exists for.
const TWIN_IPS: &[u8] = b"192.168.8.100\n192.168.8.100\n";

fn twin_sidecar(generation: u64, sha: &str) -> String {
    format!(
        r#"{{"schema_version":1,"generation":{generation},"ips_file_sha256":"{sha}",
           "links":[
             {{"link_id":"modem-a","ip":"192.168.8.100","iface":"wwan0"}},
             {{"link_id":"modem-b","ip":"192.168.8.100","iface":"wwan1"}}
           ]}}"#
    )
}

/// Validate a sidecar body against `TWIN_IPS` with both twin interfaces present.
fn validate_twin(body: &str, prior: Option<&MappedPool>) -> Result<MappedPool, BindMapError> {
    let ips = IpsFile::from_bytes(TWIN_IPS).expect("twin ips file parses");
    let doc = parse_sidecar(body.as_bytes())?;
    let oracle = ifaces(&["wwan0", "wwan1"]);
    validate_pair(
        &ips,
        doc,
        ValidateCtx {
            ifaces: &oracle,
            prior,
        },
    )
}

fn twin_sha() -> String {
    IpsFile::from_bytes(TWIN_IPS)
        .expect("twin ips file parses")
        .sha256()
        .to_string()
}

// ---- generation semantics --------------------------------------------

#[test]
fn a_mapping_only_change_keeps_the_digest_and_bumps_the_generation() {
    let prior = validate_twin(&twin_sidecar(4, &twin_sha()), None).expect("first apply");

    // Same ips_file bytes, same digest, interfaces swapped, generation bumped.
    let sha = twin_sha();
    let remapped = format!(
        r#"{{"schema_version":1,"generation":5,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"modem-a","ip":"192.168.8.100","iface":"wwan1"}},
            {{"link_id":"modem-b","ip":"192.168.8.100","iface":"wwan0"}}]}}"#
    );
    let next = validate_twin(&remapped, Some(&prior))
        .expect("same digest + new generation is a VALID pair");
    assert_eq!(next.generation, 5);
    assert_eq!(next.rows[0].iface.as_str(), "wwan1");
}

#[test]
fn changing_the_mapping_without_bumping_the_generation_is_stale() {
    let prior = validate_twin(&twin_sidecar(4, &twin_sha()), None).expect("first apply");

    let sha = twin_sha();
    let unbumped = format!(
        r#"{{"schema_version":1,"generation":4,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"modem-a","ip":"192.168.8.100","iface":"wwan1"}},
            {{"link_id":"modem-b","ip":"192.168.8.100","iface":"wwan0"}}]}}"#
    );
    let err = validate_twin(&unbumped, Some(&prior))
        .expect_err("the reader cannot order two mappings that share a generation");
    assert_matches!(err, BindMapError::StaleGeneration { generation: 4 });
    assert_eq!(err.reason(), DegradedReason::Malformed);
}

#[test]
fn an_identical_reread_is_a_no_op_rather_than_an_error() {
    let prior = validate_twin(&twin_sidecar(4, &twin_sha()), None).expect("first apply");
    let again = validate_twin(&twin_sidecar(4, &twin_sha()), Some(&prior))
        .expect("re-reading the same pair is not a failure");
    assert_eq!(again.generation, prior.generation);
    assert_eq!(again.rows.len(), prior.rows.len());
}

#[test]
fn a_writer_restart_that_resets_the_generation_is_accepted() {
    // A decrease is indistinguishable from a legitimate restart, so it must never
    // be a rejection on its own.
    let prior = validate_twin(&twin_sidecar(97, &twin_sha()), None).expect("first apply");
    let sha = twin_sha();
    let restarted = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"modem-a","ip":"192.168.8.100","iface":"wwan1"}},
            {{"link_id":"modem-b","ip":"192.168.8.100","iface":"wwan0"}}]}}"#
    );
    let next = validate_twin(&restarted, Some(&prior)).expect("a writer restart is legitimate");
    assert_eq!(next.generation, 1);
}

// ---- the reason set is total -----------------------------------------

#[test]
fn every_error_class_maps_onto_exactly_one_telemetry_reason() {
    let cases: [(BindMapError, DegradedReason, &str); 7] = [
        (
            BindMapError::HashMismatch {
                sidecar: String::new(),
                computed: String::new(),
            },
            DegradedReason::HashMismatch,
            "hash_mismatch",
        ),
        (
            BindMapError::MalformedJson("x".into()),
            DegradedReason::Malformed,
            "malformed",
        ),
        (
            BindMapError::UnknownIface {
                index: 0,
                iface: "wwan9".into(),
            },
            DegradedReason::UnknownIface,
            "unknown_iface",
        ),
        (
            BindMapError::RetryExhausted { attempts: 5 },
            DegradedReason::RetryExhausted,
            "retry_exhausted",
        ),
        (
            BindMapError::MissingFile {
                path: "/nope".into(),
            },
            DegradedReason::MissingFile,
            "missing_file",
        ),
        (
            BindMapError::Unreadable {
                path: "/nope".into(),
                detail: "eperm".into(),
            },
            DegradedReason::Unreadable,
            "unreadable",
        ),
        (
            BindMapError::UnsupportedSchemaVersion { found: 2 },
            DegradedReason::Unsupported,
            "unsupported",
        ),
    ];

    for (err, reason, token) in cases {
        assert_eq!(err.reason(), reason);
        assert_eq!(
            reason.as_str(),
            token,
            "telemetry token is the wire contract"
        );
    }
}

// ---- constants --------------------------------------------------------

#[test]
fn the_retry_schedule_fits_inside_the_two_second_budget() {
    let worst_case = u64::from(BIND_MAP_RETRY_ATTEMPTS - 1) * BIND_MAP_RETRY_DELAY_MS;
    assert!(
        worst_case <= BIND_MAP_RETRY_BUDGET_MS,
        "retry schedule {worst_case}ms exceeds the {BIND_MAP_RETRY_BUDGET_MS}ms ceiling"
    );
    assert!(BIND_MAP_RETRY_ATTEMPTS >= 2, "a retry budget needs a retry");
    assert_eq!(BIND_MAP_SCHEMA_VERSION, 1);
}

#[test]
fn the_static_iface_oracle_answers_only_for_names_it_was_given() {
    let oracle = ifaces(&["wwan0"]);
    assert!(crate::bind_map::IfaceOracle::exists(&oracle, "wwan0"));
    assert!(!crate::bind_map::IfaceOracle::exists(&oracle, "wwan1"));
}
