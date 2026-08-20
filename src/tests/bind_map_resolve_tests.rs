//! Fail-open, duplicate-safe resolution (ADR-003 §6).
//!
//! Startup and reload are deliberately different: startup has nothing to retain
//! so it excludes ambiguity and reports it; reload has a known-good pool so it
//! keeps it rather than silently un-binding a live bond.

use std::net::IpAddr;

use crate::bind_map::{
    BindMapDisposition, BindMapError, BindMapStatus, DegradedReason, IpsFile, MappedPool,
    ResolvePhase, StaticIfaces, ValidateCtx, parse_sidecar, resolve, resolve_absent, validate_pair,
};

fn ip(s: &str) -> IpAddr {
    s.parse().expect("test fixture is a valid IP")
}

const TWIN_IPS: &[u8] = b"192.168.8.100\n192.168.8.100\n";
const DISTINCT_IPS: &[u8] = b"10.0.0.1\n10.0.0.2\n";

fn ips(bytes: &[u8]) -> IpsFile {
    IpsFile::from_bytes(bytes).expect("fixture ips file parses")
}

/// A valid twin mapping, as if a coherent pair had just been read.
fn twin_pool(generation: u64) -> MappedPool {
    let ips = ips(TWIN_IPS);
    let sha = ips.sha256();
    let body = format!(
        r#"{{"schema_version":1,"generation":{generation},"ips_file_sha256":"{sha}","links":[
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

fn degraded() -> BindMapError {
    BindMapError::RetryExhausted { attempts: 5 }
}

// ---- absent: the parity guarantee ------------------------------------

#[test]
fn an_absent_map_reproduces_legacy_behavior_including_duplicate_rows() {
    // Without --bind-map the sender must hand the legacy pool builder exactly the
    // list read_ip_list produced — duplicates included, nothing excluded, nothing
    // reported. Any filtering here would change shipped behavior.
    let ips = ips(TWIN_IPS);
    let out = resolve_absent(&ips);

    assert_eq!(out.status, BindMapStatus::Absent);
    assert_eq!(out.disposition, BindMapDisposition::LegacyUniqueOnly);
    assert_eq!(
        out.links.iter().map(|l| l.ip).collect::<Vec<_>>(),
        vec![ip("192.168.8.100"), ip("192.168.8.100")],
        "the absent path must not dedup — that is the legacy pool builder's job"
    );
    assert!(out.links.iter().all(|l| l.iface.is_none()));
    assert!(out.links.iter().all(|l| l.link_id.is_none()));
    assert!(out.excluded.is_empty());
    assert!(out.applied.is_none());
}

// ---- active ----------------------------------------------------------

#[test]
fn a_valid_pair_is_active_and_mapped_at_startup() {
    let ips = ips(TWIN_IPS);
    let out = resolve(Ok(twin_pool(3)), &ips, ResolvePhase::Startup);

    assert_eq!(out.status, BindMapStatus::Active);
    assert_eq!(out.disposition, BindMapDisposition::Mapped);
    assert_eq!(out.links.len(), 2);
    assert_eq!(
        out.links[0].iface.as_ref().map(|i| i.as_str()),
        Some("wwan0")
    );
    assert_eq!(
        out.links[1].iface.as_ref().map(|i| i.as_str()),
        Some("wwan1")
    );
    assert_eq!(
        out.links[1].link_id.as_ref().map(|l| l.as_str()),
        Some("modem-b")
    );
    assert!(out.excluded.is_empty());
    assert_eq!(
        out.applied.as_ref().map(|p| p.generation),
        Some(3),
        "an active resolution hands back the pool the caller must remember"
    );
}

// ---- startup degraded ------------------------------------------------

#[test]
fn startup_degraded_without_collisions_runs_every_unique_row() {
    let ips = ips(DISTINCT_IPS);
    let out = resolve(Err(degraded()), &ips, ResolvePhase::Startup);

    assert_eq!(
        out.status,
        BindMapStatus::Degraded(DegradedReason::RetryExhausted)
    );
    assert_eq!(out.disposition, BindMapDisposition::LegacyUniqueOnly);
    assert_eq!(
        out.links.iter().map(|l| l.ip).collect::<Vec<_>>(),
        vec![ip("10.0.0.1"), ip("10.0.0.2")]
    );
    assert!(out.excluded.is_empty());
}

#[test]
fn startup_degraded_with_a_collision_keeps_one_representative_and_reports_the_rest() {
    let ips = ips(b"192.168.8.100\n10.0.0.7\n192.168.8.100\n192.168.8.100\n");
    let out = resolve(Err(degraded()), &ips, ResolvePhase::Startup);

    assert_eq!(
        out.disposition,
        BindMapDisposition::StartupCollisionExcluded
    );
    assert_eq!(
        out.links.iter().map(|l| l.ip).collect::<Vec<_>>(),
        vec![ip("192.168.8.100"), ip("10.0.0.7")],
        "the FIRST occurrence in file order is the deterministic representative"
    );

    assert_eq!(out.excluded.len(), 1, "one colliding IP, one group");
    let group = &out.excluded[0];
    assert_eq!(group.ip, ip("192.168.8.100"));
    assert_eq!(group.effective_index, 0);
    assert_eq!(
        group.excluded_indices,
        vec![2, 3],
        "the ambiguity is named, not silently swallowed"
    );
}

#[test]
fn startup_degraded_never_attaches_an_interface_it_had_to_guess() {
    let ips = ips(TWIN_IPS);
    let out = resolve(Err(degraded()), &ips, ResolvePhase::Startup);
    assert!(
        out.links
            .iter()
            .all(|l| l.iface.is_none() && l.link_id.is_none()),
        "a degraded read must never invent a binding"
    );
    assert!(out.applied.is_none());
}

#[test]
fn the_degraded_reason_is_carried_verbatim_into_the_status() {
    let ips = ips(TWIN_IPS);
    let cases = [
        (
            BindMapError::MissingFile {
                path: "/nope".into(),
            },
            DegradedReason::MissingFile,
        ),
        (
            BindMapError::UnknownIface {
                index: 0,
                iface: "wwan9".into(),
            },
            DegradedReason::UnknownIface,
        ),
        (
            BindMapError::MalformedJson("bad".into()),
            DegradedReason::Malformed,
        ),
    ];
    for (err, reason) in cases {
        let out = resolve(Err(err), &ips, ResolvePhase::Startup);
        assert_eq!(out.status, BindMapStatus::Degraded(reason));
    }
}

// ---- reload degraded: retention --------------------------------------

#[test]
fn a_degraded_reload_retains_the_last_valid_pool_instead_of_tearing_the_bond_down() {
    let last_valid = twin_pool(4);
    let ips = ips(TWIN_IPS);
    let out = resolve(
        Err(degraded()),
        &ips,
        ResolvePhase::Reload {
            last_valid: &last_valid,
        },
    );

    assert_eq!(
        out.status,
        BindMapStatus::Degraded(DegradedReason::RetryExhausted)
    );
    assert_eq!(out.disposition, BindMapDisposition::RetainedLastValid);
    assert_eq!(
        out.links
            .iter()
            .map(|l| l.iface.as_ref().map(|i| i.as_str().to_string()))
            .collect::<Vec<_>>(),
        vec![Some("wwan0".to_string()), Some("wwan1".to_string())],
        "the retained pool keeps its interface bindings"
    );
    assert!(out.excluded.is_empty());
    assert!(
        out.applied.is_none(),
        "retention does not re-apply; the caller keeps the pool it already has"
    );
}

#[test]
fn retention_applies_even_when_the_new_ip_list_has_no_ambiguity_at_all() {
    // Falling back to legacy here would silently drop per-interface egress pinning
    // from every link of a live bond — worse than running the last good mapping.
    let last_valid = twin_pool(4);
    let ips = ips(DISTINCT_IPS);
    let out = resolve(
        Err(BindMapError::MalformedJson("truncated".into())),
        &ips,
        ResolvePhase::Reload {
            last_valid: &last_valid,
        },
    );

    assert_eq!(out.disposition, BindMapDisposition::RetainedLastValid);
    assert_eq!(
        out.links.iter().map(|l| l.ip).collect::<Vec<_>>(),
        vec![ip("192.168.8.100"), ip("192.168.8.100")],
        "the RETAINED pool's links run, not the rejected reload's"
    );
}

// ---- reload recovery -------------------------------------------------

#[test]
fn a_coherent_pair_arriving_after_degradation_replaces_the_retained_pool() {
    let last_valid = twin_pool(4);
    let ips = ips(TWIN_IPS);

    // Degrade …
    let degraded_out = resolve(
        Err(degraded()),
        &ips,
        ResolvePhase::Reload {
            last_valid: &last_valid,
        },
    );
    assert_eq!(
        degraded_out.disposition,
        BindMapDisposition::RetainedLastValid
    );

    // … then recover.
    let recovered = twin_pool(5);
    let out = resolve(
        Ok(recovered),
        &ips,
        ResolvePhase::Reload {
            last_valid: &last_valid,
        },
    );

    assert_eq!(out.status, BindMapStatus::Active);
    assert_eq!(out.disposition, BindMapDisposition::Mapped);
    assert_eq!(
        out.applied.as_ref().map(|p| p.generation),
        Some(5),
        "recovery hands back the NEW pool so the caller stops retaining the old one"
    );
}
