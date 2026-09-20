//! Bind-map row validation, correspondence, uniqueness, and generation ordering
//! (ADR-003 §2, §3, §4.1).
//!
//! Scope: deciding whether a parsed pair may be applied. Reading the files into
//! typed values lives in `bind_map_parse_tests`.
use std::net::{IpAddr, Ipv4Addr};

use assert_matches::assert_matches;

use crate::bind_map::{
    BindMapError, DegradedReason, IpsFile, MappedPool, StaticIfaces, ValidateCtx, parse_sidecar,
    validate_pair,
};

// ---- fixtures ---------------------------------------------------------

fn ip(s: &str) -> IpAddr {
    s.parse().expect("test fixture is a valid IP")
}

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

// ---- rejection: row syntax -------------------------------------------

#[test]
fn a_malformed_row_is_rejected_with_its_index_and_field() {
    let sha = twin_sha();
    let cases: [(&str, &str); 6] = [
        (
            r#"{"link_id":"","ip":"192.168.8.100","iface":"wwan0"}"#,
            "link_id",
        ),
        (
            r#"{"link_id":"has space","ip":"192.168.8.100","iface":"wwan0"}"#,
            "link_id",
        ),
        (r#"{"link_id":"a","ip":"not-an-ip","iface":"wwan0"}"#, "ip"),
        (
            r#"{"link_id":"a","ip":"192.168.8.100","iface":"wwan0/../wwan1"}"#,
            "iface",
        ),
        (
            r#"{"link_id":"a","ip":"192.168.8.100","iface":"an-interface-name-far-too-long"}"#,
            "iface",
        ),
        (
            r#"{"link_id":"a","ip":"192.168.8.100","iface":"wwan0","id_path":"relative/path"}"#,
            "id_path",
        ),
    ];

    for (row, field) in cases {
        let body = format!(
            r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[{row},
               {{"link_id":"b","ip":"192.168.8.100","iface":"wwan1"}}]}}"#
        );
        let err = validate_twin(&body, None).expect_err("row {row} must be rejected");
        assert_matches!(
            err,
            BindMapError::InvalidRow { index: 0, field: f, .. } if f == field,
            "row {row} should fail on `{field}`"
        );
        assert_eq!(err.reason(), DegradedReason::Malformed);
    }
}

#[test]
fn an_oversized_link_id_is_malformed() {
    let sha = twin_sha();
    let long = "x".repeat(65);
    let body = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"{long}","ip":"192.168.8.100","iface":"wwan0"}},
            {{"link_id":"b","ip":"192.168.8.100","iface":"wwan1"}}]}}"#
    );
    assert_matches!(
        validate_twin(&body, None).expect_err("64 bytes is the ceiling"),
        BindMapError::InvalidRow {
            field: "link_id",
            ..
        }
    );
}

// ---- rejection: uniqueness -------------------------------------------

#[test]
fn a_duplicate_ip_iface_pair_names_the_same_socket_twice_and_is_rejected() {
    let sha = twin_sha();
    let body = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"a","ip":"192.168.8.100","iface":"wwan0"}},
            {{"link_id":"b","ip":"192.168.8.100","iface":"wwan0"}}]}}"#
    );
    let err = validate_twin(&body, None).expect_err("duplicate (ip,iface) is not a bond");
    assert_matches!(err, BindMapError::DuplicatePair { index: 1, .. });
    assert_eq!(err.reason(), DegradedReason::Malformed);
}

#[test]
fn a_duplicate_link_id_is_rejected_because_identity_must_be_unique() {
    let sha = twin_sha();
    let body = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"same","ip":"192.168.8.100","iface":"wwan0"}},
            {{"link_id":"same","ip":"192.168.8.100","iface":"wwan1"}}]}}"#
    );
    let err = validate_twin(&body, None).expect_err("link_id is the identity key");
    assert_matches!(err, BindMapError::DuplicateLinkId { index: 1, .. });
}

// ---- rejection: interfaces -------------------------------------------

#[test]
fn a_row_naming_an_absent_interface_is_rejected_as_unknown_iface() {
    let ips = IpsFile::from_bytes(TWIN_IPS).expect("twin ips file parses");
    let doc = parse_sidecar(twin_sidecar(1, &twin_sha()).as_bytes()).expect("parses");
    let oracle = ifaces(&["wwan0"]); // wwan1 was unplugged
    let err = validate_pair(
        &ips,
        doc,
        ValidateCtx {
            ifaces: &oracle,
            prior: None,
        },
    )
    .expect_err("an interface that does not exist cannot be bound");

    assert_matches!(err, BindMapError::UnknownIface { index: 1, .. });
    assert_eq!(err.reason(), DegradedReason::UnknownIface);
}

// ---- rejection: row correspondence -----------------------------------

#[test]
fn a_row_count_mismatch_invalidates_the_whole_pair() {
    let sha = twin_sha();
    let short = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"a","ip":"192.168.8.100","iface":"wwan0"}}]}}"#
    );
    assert_matches!(
        validate_twin(&short, None).expect_err("a missing row is not a partial map"),
        BindMapError::RowCountMismatch {
            sidecar: 1,
            ips_file: 2
        }
    );

    let long = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"a","ip":"192.168.8.100","iface":"wwan0"}},
            {{"link_id":"b","ip":"192.168.8.100","iface":"wwan1"}},
            {{"link_id":"c","ip":"192.168.8.100","iface":"wwan0"}}]}}"#
    );
    assert_matches!(
        validate_twin(&long, None).expect_err("an extra row is not a partial map"),
        BindMapError::RowCountMismatch {
            sidecar: 3,
            ips_file: 2
        }
    );
}

#[test]
fn a_reordered_or_mismatched_row_ip_invalidates_the_whole_pair() {
    // Distinct IPs, so a swap is observable positionally.
    let ips_bytes = b"10.0.0.1\n10.0.0.2\n";
    let ips = IpsFile::from_bytes(ips_bytes).expect("parses");
    let sha = ips.sha256().to_string();
    let body = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"a","ip":"10.0.0.2","iface":"wwan0"}},
            {{"link_id":"b","ip":"10.0.0.1","iface":"wwan1"}}]}}"#
    );
    let doc = parse_sidecar(body.as_bytes()).expect("parses");
    let oracle = ifaces(&["wwan0", "wwan1"]);
    let err = validate_pair(
        &ips,
        doc,
        ValidateCtx {
            ifaces: &oracle,
            prior: None,
        },
    )
    .expect_err("the Nth row must describe the Nth ips_file entry");

    assert_matches!(
        err,
        BindMapError::RowIpMismatch { index: 0, sidecar, ips_file }
            if sidecar == ip("10.0.0.2") && ips_file == ip("10.0.0.1")
    );
    assert_eq!(err.reason(), DegradedReason::Malformed);
}

// ---- rejection: hash coherence ---------------------------------------

#[test]
fn a_digest_naming_different_content_is_a_hash_mismatch_not_a_malformed_file() {
    let stale = "0".repeat(64);
    let err = validate_twin(&twin_sidecar(1, &stale), None)
        .expect_err("the sidecar must name the exact ips_file content it describes");
    assert_matches!(err, BindMapError::HashMismatch { .. });
    assert_eq!(
        err.reason(),
        DegradedReason::HashMismatch,
        "hash mismatch is the retryable class and must stay distinguishable"
    );
}

#[test]
fn hash_coherence_is_checked_before_row_validation() {
    // Rows in hand during a mismatch may be about to be replaced; reporting a row
    // error for content that is already known stale would send a writer chasing
    // the wrong bug.
    let sha = "0".repeat(64);
    let body = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"a","ip":"10.9.9.9","iface":"nonexistent0"}}]}}"#
    );
    assert_matches!(
        validate_twin(&body, None).expect_err("stale content"),
        BindMapError::HashMismatch { .. }
    );
}

#[test]
fn an_ipv6_row_is_as_legal_as_an_ipv4_row() {
    let ips_bytes = b"fd00::1\n";
    let ips = IpsFile::from_bytes(ips_bytes).expect("parses");
    assert_eq!(ips.accepted(), [ip("fd00::1")]);
    let sha = ips.sha256().to_string();
    let body = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"v6","ip":"fd00::1","iface":"wwan0"}}]}}"#
    );
    let doc = parse_sidecar(body.as_bytes()).expect("parses");
    let oracle = ifaces(&["wwan0"]);
    let pool = validate_pair(
        &ips,
        doc,
        ValidateCtx {
            ifaces: &oracle,
            prior: None,
        },
    )
    .expect("IPv6 uplinks are ordinary uplinks");
    assert_eq!(
        pool.rows[0].ip,
        IpAddr::from("fd00::1".parse::<std::net::Ipv6Addr>().unwrap())
    );
    assert_ne!(pool.rows[0].ip, IpAddr::V4(Ipv4Addr::LOCALHOST));
}
