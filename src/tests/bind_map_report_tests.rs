//! The telemetry projection of an ADR-003 resolution (ADR-003 §6.4).
//!
//! These pin the exact snake_case tokens a UI dispatches on. They are a wire
//! contract, so every one of the three statuses, seven reasons, and four
//! dispositions is asserted literally rather than through the enum that
//! produced it — a rename would otherwise pass silently.

use std::net::IpAddr;

use crate::bind_map::{
    BindMapDisposition, BindMapError, BindMapReport, BindMapStatus, CollisionGroup, DegradedReason,
    IpsFile, MappedPool, Resolution, ResolvePhase, StaticIfaces, ValidateCtx, parse_sidecar,
    resolve, resolve_absent, validate_pair,
};

fn ip(s: &str) -> IpAddr {
    s.parse().expect("test fixture is a valid IP")
}

const TWIN_IPS: &[u8] = b"192.168.8.100\n192.168.8.100\n";

fn ips(bytes: &[u8]) -> IpsFile {
    IpsFile::from_bytes(bytes).expect("fixture ips file parses")
}

fn twin_pool() -> MappedPool {
    let ips = ips(TWIN_IPS);
    let sha = ips.sha256();
    let body = format!(
        r#"{{"schema_version":1,"generation":3,"ips_file_sha256":"{sha}","links":[
            {{"link_id":"modem-a","ip":"192.168.8.100","iface":"wwan0"}},
            {{"link_id":"modem-b","ip":"192.168.8.100","iface":"wwan1"}}]}}"#
    );
    let doc = parse_sidecar(body.as_bytes()).expect("fixture sidecar parses");
    let oracle = StaticIfaces::new(["wwan0", "wwan1"]);
    validate_pair(
        &ips,
        doc,
        ValidateCtx {
            ifaces: &oracle,
            prior: None,
        },
    )
    .expect("fixture pair is valid")
}

fn report_of(resolution: &Resolution) -> BindMapReport {
    BindMapReport::from(resolution)
}

fn json_of(report: &BindMapReport) -> serde_json::Value {
    serde_json::to_value(report).expect("the report is plain scalars")
}

// ---- the three statuses -----------------------------------------------

#[test]
fn a_legacy_run_reports_absent_with_no_reason() {
    // Given: no --bind-map, so the legacy pool builder runs.
    let ips = ips(TWIN_IPS);

    // When: the resolution is projected for telemetry.
    let json = json_of(&report_of(&resolve_absent(&ips)));

    // Then: a UI can see the sender is legacy, not merely "not degraded", and
    // there is no reason field to misread.
    assert_eq!(json["bind_map_status"]["state"], "absent");
    assert!(json["bind_map_status"].get("reason").is_none());
    assert_eq!(json["disposition"]["state"], "legacy_unique_only");
}

#[test]
fn a_default_report_is_the_legacy_one() {
    // Given/When: a sender that has never resolved a map (the shipped default).
    let json = json_of(&BindMapReport::default());

    // Then: it reports the same thing an explicit legacy run does, so a snapshot
    // taken before the first resolution never claims a map is active.
    assert_eq!(json["bind_map_status"]["state"], "absent");
    assert_eq!(json["disposition"]["state"], "legacy_unique_only");
}

#[test]
fn an_applied_map_reports_active_and_mapped() {
    // Given: a coherent pair.
    let ips = ips(TWIN_IPS);
    let resolution = resolve(Ok(twin_pool()), &ips, ResolvePhase::Startup);

    // When: projected.
    let json = json_of(&report_of(&resolution));

    // Then: active/mapped with nothing excluded.
    assert_eq!(json["bind_map_status"]["state"], "active");
    assert_eq!(json["disposition"]["state"], "mapped");
    assert!(json["disposition"].get("collisions").is_none());
}

#[test]
fn every_degraded_reason_reaches_telemetry_verbatim() {
    // Given: the seven frozen reasons (ADR-003 §6.4) paired with their tokens.
    let cases = [
        (DegradedReason::HashMismatch, "hash_mismatch"),
        (DegradedReason::Malformed, "malformed"),
        (DegradedReason::UnknownIface, "unknown_iface"),
        (DegradedReason::RetryExhausted, "retry_exhausted"),
        (DegradedReason::MissingFile, "missing_file"),
        (DegradedReason::Unreadable, "unreadable"),
        (DegradedReason::Unsupported, "unsupported"),
    ];

    for (reason, token) in cases {
        // When: a degraded status carrying that reason is projected.
        let report = BindMapReport::from(&Resolution {
            status: BindMapStatus::Degraded(reason),
            disposition: BindMapDisposition::LegacyUniqueOnly,
            links: Vec::new(),
            excluded: Vec::new(),
            applied: None,
        });

        // Then: the token crosses to the wire unchanged.
        let json = json_of(&report);
        assert_eq!(json["bind_map_status"]["state"], "degraded");
        assert_eq!(json["bind_map_status"]["reason"], token);
    }
}

// ---- the four dispositions ---------------------------------------------

#[test]
fn a_degraded_reload_reports_the_pool_it_retained() {
    // Given: a live bond whose reload degraded.
    let ips = ips(TWIN_IPS);
    let last_valid = twin_pool();
    let resolution = resolve(
        Err(BindMapError::RetryExhausted { attempts: 5 }),
        &ips,
        ResolvePhase::Reload {
            last_valid: &last_valid,
        },
    );

    // When: projected.
    let json = json_of(&report_of(&resolution));

    // Then: the operator sees BOTH that the map is degraded AND that the bond is
    // still interface-pinned — the two facts a single field could not carry.
    assert_eq!(json["bind_map_status"]["state"], "degraded");
    assert_eq!(json["bind_map_status"]["reason"], "retry_exhausted");
    assert_eq!(json["disposition"]["state"], "retained_last_valid");
    assert!(json["disposition"].get("collisions").is_none());
}

#[test]
fn a_degraded_startup_names_the_colliding_group_it_broke_up() {
    // Given: two modems on one IP and a map that never became coherent.
    let ips = ips(TWIN_IPS);
    let resolution = resolve(
        Err(BindMapError::RetryExhausted { attempts: 5 }),
        &ips,
        ResolvePhase::Startup,
    );

    // When: projected.
    let json = json_of(&report_of(&resolution));

    // Then: the disposition names the exact group — which IP, which ips-file
    // line runs, which do not — so "two modems, one visible link" is explained
    // by typed data rather than a log line.
    assert_eq!(json["disposition"]["state"], "startup_collision_excluded");
    let collisions = json["disposition"]["collisions"]
        .as_array()
        .expect("a collision group must be reported");
    assert_eq!(collisions.len(), 1);
    assert_eq!(collisions[0]["ip"], "192.168.8.100");
    assert_eq!(collisions[0]["effective_index"], 0);
    assert_eq!(collisions[0]["excluded_indices"], serde_json::json!([1]));
}

#[test]
fn a_degraded_startup_without_ambiguity_stays_legacy_unique_only() {
    // Given: a degraded startup whose IP list has no duplicates at all.
    let ips = ips(b"10.0.0.1\n10.0.0.2\n");
    let resolution = resolve(
        Err(BindMapError::MissingFile {
            path: "/nope".to_string(),
        }),
        &ips,
        ResolvePhase::Startup,
    );

    // When/Then: nothing was excluded, so the disposition must not claim it was.
    let json = json_of(&report_of(&resolution));
    assert_eq!(json["bind_map_status"]["reason"], "missing_file");
    assert_eq!(json["disposition"]["state"], "legacy_unique_only");
    assert!(json["disposition"].get("collisions").is_none());
}

#[test]
fn multiple_collision_groups_are_all_reported() {
    // Given: two independent same-IP groups, one of them three deep.
    let resolution = Resolution {
        status: BindMapStatus::Degraded(DegradedReason::Malformed),
        disposition: BindMapDisposition::StartupCollisionExcluded,
        links: Vec::new(),
        excluded: vec![
            CollisionGroup {
                ip: ip("192.168.8.100"),
                effective_index: 0,
                excluded_indices: vec![1, 2],
            },
            CollisionGroup {
                ip: ip("192.168.9.100"),
                effective_index: 3,
                excluded_indices: vec![4],
            },
        ],
        applied: None,
    };

    // When: projected.
    let json = json_of(&BindMapReport::from(&resolution));

    // Then: no group is summarized away — an operator with four dark modems is
    // told about all of them.
    let collisions = json["disposition"]["collisions"]
        .as_array()
        .expect("collision groups must be reported");
    assert_eq!(collisions.len(), 2);
    assert_eq!(
        collisions[0]["excluded_indices"],
        serde_json::json!([1, 2]),
        "a three-deep group must list both excluded lines"
    );
    assert_eq!(collisions[1]["effective_index"], 3);
}

// ---- the token tables have exactly one home ----------------------------

#[test]
fn the_disposition_tokens_match_the_adr_table() {
    // Given/When/Then: the log line and the telemetry projection read the same
    // accessor, so this is the single assertion that pins ADR-003 §6.4's table.
    assert_eq!(BindMapDisposition::Mapped.as_str(), "mapped");
    assert_eq!(
        BindMapDisposition::RetainedLastValid.as_str(),
        "retained_last_valid"
    );
    assert_eq!(
        BindMapDisposition::LegacyUniqueOnly.as_str(),
        "legacy_unique_only"
    );
    assert_eq!(
        BindMapDisposition::StartupCollisionExcluded.as_str(),
        "startup_collision_excluded"
    );
}

#[test]
fn a_non_degraded_status_has_no_reason_to_report() {
    // Given/When/Then: `reason()` is what keeps the absent/active records from
    // carrying a stale reason field.
    assert!(BindMapStatus::Active.reason().is_none());
    assert!(BindMapStatus::Absent.reason().is_none());
    assert_eq!(
        BindMapStatus::Degraded(DegradedReason::Unreadable).reason(),
        Some(DegradedReason::Unreadable)
    );
}
