//! Bind-map IP-file and sidecar *document* parsing (ADR-003 §1, §2, §4.1).
//!
//! Scope: reading the two files into typed values. Row-level validation, row
//! correspondence, and generation ordering live in `bind_map_validate_tests`.
use std::net::IpAddr;

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

// ---- the ips file: legacy parsing rules, shared verbatim --------------

#[tokio::test]
async fn ips_file_parsing_matches_the_legacy_read_ip_list_verbatim() {
    // "Identical parsing rules" (ADR-003 §2) is the load-bearing claim that makes
    // positional row correspondence meaningful. Prove it against the real legacy
    // parser over every shape it treats specially.
    let cases: [&str; 6] = [
        "10.0.0.1\n10.0.1.2\n",
        "10.0.0.1\n\n  \nnot-an-ip\n10.0.2.3\n",
        "  10.0.0.1  \n\t10.0.0.2\t\n",
        "192.168.8.100\n192.168.8.100\n",
        "",
        "garbage\nstill-garbage\n",
    ];

    for case in cases {
        let file = tempfile::NamedTempFile::new().expect("temp ips file");
        std::fs::write(file.path(), case).expect("write ips file");
        let legacy = crate::sender::read_ip_list(file.path().to_str().unwrap())
            .await
            .expect("legacy parser reads the fixture");

        let parsed = IpsFile::from_bytes(case.as_bytes()).expect("bind-map parser reads it too");

        assert_eq!(
            parsed.accepted(),
            legacy.as_slice(),
            "bind-map ips parsing diverged from read_ip_list on {case:?}"
        );
    }
}

#[test]
fn ips_file_sha256_matches_the_published_nist_vectors() {
    // Independent known-answer vectors, so this is a real check on the hash and
    // not a tautology against our own implementation.
    assert_eq!(
        IpsFile::from_bytes(b"")
            .expect("empty file parses")
            .sha256(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    // "abc" is not a valid IP line, so it is skipped — but the hash covers the
    // RAW BYTES, which is exactly the property the coherence check depends on.
    let abc = IpsFile::from_bytes(b"abc").expect("garbage file still parses");
    assert_eq!(
        abc.sha256(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert!(abc.accepted().is_empty());
}

// ---- happy path -------------------------------------------------------

#[test]
fn a_coherent_twin_pair_validates_with_every_field_carried() {
    let pool = validate_twin(&twin_sidecar(7, &twin_sha()), None).expect("twin pair is valid");

    assert_eq!(pool.generation, 7);
    assert_eq!(pool.ips_file_sha256, twin_sha());
    assert_eq!(pool.rows.len(), 2);
    assert_eq!(pool.rows[0].link_id.as_str(), "modem-a");
    assert_eq!(pool.rows[0].iface.as_str(), "wwan0");
    assert_eq!(pool.rows[0].ip, ip("192.168.8.100"));
    assert_eq!(pool.rows[1].link_id.as_str(), "modem-b");
    assert_eq!(
        pool.rows[1].iface.as_str(),
        "wwan1",
        "same IP on a different interface is the legal case this contract exists for"
    );
}

#[test]
fn id_path_is_optional_and_carried_verbatim_when_present() {
    let sha = twin_sha();
    let body = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"a","ip":"192.168.8.100","iface":"wwan0","id_path":"/sys/class/net/wwan0"}},
            {{"link_id":"b","ip":"192.168.8.100","iface":"wwan1"}}]}}"#
    );
    let pool = validate_twin(&body, None).expect("id_path is optional");
    assert_eq!(
        pool.rows[0].id_path.as_deref(),
        Some("/sys/class/net/wwan0")
    );
    assert_eq!(pool.rows[1].id_path, None);
}

#[test]
fn unknown_additive_fields_are_ignored_within_the_same_schema_version() {
    let sha = twin_sha();
    let body = format!(
        r#"{{"schema_version":1,"generation":1,"ips_file_sha256":"{sha}","future_header":9,"links":[
            {{"link_id":"a","ip":"192.168.8.100","iface":"wwan0","future_row":"x"}},
            {{"link_id":"b","ip":"192.168.8.100","iface":"wwan1"}}]}}"#
    );
    validate_twin(&body, None).expect("unknown keys are ignored, not fatal");
}

// ---- rejection: document shape ---------------------------------------

#[test]
fn malformed_json_is_rejected_as_malformed() {
    let err = parse_sidecar(b"{not json").expect_err("garbage must not parse");
    assert_matches!(err, BindMapError::MalformedJson(_));
    assert_eq!(err.reason(), DegradedReason::Malformed);
}

#[test]
fn an_unsupported_schema_version_is_refused_rather_than_guessed_at() {
    for version in ["2", "0"] {
        let body = format!(
            r#"{{"schema_version":{version},"generation":1,"ips_file_sha256":"{}","links":[]}}"#,
            twin_sha()
        );
        let err = parse_sidecar(body.as_bytes()).expect_err("only version 1 is readable");
        assert_matches!(err, BindMapError::UnsupportedSchemaVersion { .. });
        assert_eq!(err.reason(), DegradedReason::Unsupported);
    }
}

#[test]
fn generation_zero_is_malformed_because_zero_is_the_reserved_unset_sentinel() {
    let err = parse_sidecar(twin_sidecar(0, &twin_sha()).as_bytes())
        .expect_err("generation 0 is reserved");
    assert_matches!(err, BindMapError::InvalidHeader { .. });
    assert_eq!(err.reason(), DegradedReason::Malformed);
}

#[test]
fn a_sha256_that_is_not_64_lowercase_hex_is_malformed() {
    let upper = twin_sha().to_uppercase();
    for bad in [upper.as_str(), "abc", ""] {
        let err = parse_sidecar(twin_sidecar(1, bad).as_bytes())
            .expect_err("the digest field has a fixed lexical form");
        assert_matches!(err, BindMapError::InvalidHeader { .. });
    }
}
