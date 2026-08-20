//! The ADR-001 telemetry document: its model, its units, and its serializer.
//!
//! Split from [`crate::telemetry_file`], which owns only the publish mechanics
//! (temp sibling -> fsync -> `rename(2)`). Everything that decides what the
//! document *says* lives here, so the schema has one home.
//!
//! # Schema (ADR-001 + ADR-002 + ADR-003)
//!
//! A single newline-free JSON object, parsed by the `@ceralive/srtla-send` Zod
//! reader (`bindings/typescript/src/telemetry`):
//!
//! ```json
//! {"schema_version":1,"last_updated_ms":1749556546000,"connections":[
//!   {"conn_id":"0","rtt_ms":42,"nak_count":3,"weight_percent":85,
//!    "window":8192,"in_flight":100,"bitrate_bps":2500000,
//!    "bytes_sent_total":812000000,"iface":"wwan0","link_id":"modem-a"}],
//!  "bytes_sent_total":1620000000,
//!  "bind_map_status":{"state":"active"},"disposition":{"state":"mapped"}}
//! ```
//!
//! ## `schema_version` handling
//!
//! `schema_version` stays **1**. It names the shape of the *required* fields,
//! not the set of fields present: the schema grows only by ADDITION, and an
//! added field is always OPTIONAL. A consumer therefore keeps parsing a newer
//! document — Zod strips keys it does not know — and a producer that omits an
//! added field (an older build) still validates. The version is reserved for a
//! change no old consumer could survive: renaming, retyping, or REMOVING a
//! required field, or changing a unit. None of `iface`, `link_id`,
//! `bind_map_status`, or `disposition` does any of that, so none of them bumps
//! it.
//!
//! ## Identity: `conn_id` vs `link_id`
//!
//! `conn_id` is the uplink's 0-based index in IP-list order and is
//! **TRANSIENT**: a SIGHUP reload that reorders the file reorders it, so the
//! same physical modem can answer to a different `conn_id` across a reload. It
//! is retained for compatibility and for positional correlation within one
//! snapshot. **UI identity must use `link_id`**, the sidecar's writer-assigned
//! opaque id (ADR-003), which survives reloads, reconnects, IP changes, and
//! interface changes. The sender only ever echoes it and never invents one, so
//! an unmapped link has none.
//!
//! `bitrate_bps` is wire bytes/s x 8 — the mandated bits/s conversion has its
//! single home in [`ConnRecord::from`]. `bytes_sent_total` is BYTES and is
//! **not** multiplied: it is a count, not a rate. See
//! `docs/adr/ADR-002-session-bytes-telemetry.md`.

use serde::Serialize;

use crate::bind_map::BindMapReport;
use crate::stats::StatsSnapshot;

/// JSON schema version. See the module docs for what does and does not bump it.
pub const TELEMETRY_SCHEMA_VERSION: u32 = 1;

/// One per-uplink telemetry record in wire units.
///
/// Field names / units mirror the C `TelemetrySnapshot` (`sender_telemetry.h`).
/// `bitrate_bytes_per_sec` is the wire byte rate; the mandated x8 -> bits/s
/// conversion happens only at serialization, in [`ConnRecord::from`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct TelemetryConn {
    pub conn_id: u32,
    pub rtt_ms: u32,
    pub nak_count: u32,
    pub weight_percent: u8,
    pub window: i32,
    pub in_flight: i32,
    pub bitrate_bytes_per_sec: u32,
    /// Cumulative wire BYTES for this uplink (ADR-002). Serialized verbatim —
    /// unlike `bitrate_bytes_per_sec` there is no x8, because this is a byte
    /// count and not a rate.
    pub bytes_sent_total: u64,
    /// Egress interface this uplink is bound to; `None` when unmapped.
    pub iface: Option<String>,
    /// The sidecar's writer-assigned identity, echoed; `None` when unmapped.
    pub link_id: Option<String>,
}

/// Serialized per-connection record. `conn_id` is a string and `bitrate_bps` is
/// bits/s (the x8 conversion), matching the ADR-001 schema and the Zod reader.
/// Field order is fixed to mirror the C golden fixture; the two additive
/// identity fields come last and are omitted entirely when absent.
#[derive(Serialize)]
struct ConnRecord {
    conn_id: String,
    rtt_ms: u32,
    nak_count: u32,
    weight_percent: u8,
    window: i32,
    in_flight: i32,
    bitrate_bps: u64,
    bytes_sent_total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    iface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    link_id: Option<String>,
}

impl From<&TelemetryConn> for ConnRecord {
    fn from(c: &TelemetryConn) -> Self {
        Self {
            conn_id: c.conn_id.to_string(),
            rtt_ms: c.rtt_ms,
            nak_count: c.nak_count,
            weight_percent: c.weight_percent,
            window: c.window,
            in_flight: c.in_flight,
            // The single, testable home of the mandated bytes/s -> bits/s x8.
            bitrate_bps: u64::from(c.bitrate_bytes_per_sec) * 8,
            // No x8: a cumulative byte COUNT, not a rate.
            bytes_sent_total: c.bytes_sent_total,
            iface: c.iface.clone(),
            link_id: c.link_id.clone(),
        }
    }
}

/// Whole-document shape. `schema_version` first so the on-disk object leads with
/// the version tag; the rest mirrors the ADR-001 / C golden field order, with
/// the ADR-003 operating-mode pair flattened onto the end.
#[derive(Serialize)]
struct TelemetryDoc<'a> {
    schema_version: u32,
    last_updated_ms: u64,
    connections: Vec<ConnRecord>,
    bytes_sent_total: u64,
    #[serde(flatten)]
    bind_map: &'a BindMapReport,
}

/// Everything a snapshot document is built from, other than its timestamp.
///
/// `session_bytes_sent` is the bond-level ADR-002 cumulative byte count. It is
/// passed in rather than summed from `conns` because the sum of the LIVE links
/// regresses when a link is torn down; see [`StatsSnapshot::session_bytes_sent`].
pub struct TelemetryInputs<'a> {
    pub conns: &'a [TelemetryConn],
    pub session_bytes_sent: u64,
    pub bind_map: &'a BindMapReport,
}

/// Serialize one snapshot to the exact ADR-001 JSON object (compact,
/// newline-free). This is the single place the bytes/s -> bits/s x8 conversion
/// lives, so the mandated unit transform has one testable home.
#[must_use]
pub fn build_telemetry_json(last_updated_ms: u64, inputs: &TelemetryInputs<'_>) -> String {
    let doc = TelemetryDoc {
        schema_version: TELEMETRY_SCHEMA_VERSION,
        last_updated_ms,
        connections: inputs.conns.iter().map(ConnRecord::from).collect(),
        bytes_sent_total: inputs.session_bytes_sent,
        bind_map: inputs.bind_map,
    };
    // The doc is plain scalars / strings, so serialization cannot fail; fall back
    // to an empty object defensively rather than panicking on the hot path.
    serde_json::to_string(&doc).unwrap_or_else(|_| "{}".to_string())
}

/// Serialize a whole [`StatsSnapshot`] into the publishable telemetry document.
///
/// Both sinks (the `--stats-file` writer and the `subscribe-events` broadcaster)
/// go through here, so the per-link projection, the bond-level cumulative
/// counter, and the bind-map operating mode can never be paired inconsistently.
#[must_use]
pub fn build_telemetry_json_from_stats(last_updated_ms: u64, stats: &StatsSnapshot) -> String {
    build_telemetry_json(
        last_updated_ms,
        &TelemetryInputs {
            conns: &conns_from_stats(stats),
            session_bytes_sent: stats.session_bytes_sent,
            bind_map: &stats.bind_map,
        },
    )
}

/// Project the shared stats snapshot into per-uplink telemetry records.
///
/// `conn_id` is the link's 0-based position in IP-list order. `weight_percent`
/// is the link's share of total selection weight (`base_score x quality`) among
/// active links, normalized to 100; inactive links report 0. Active links with
/// no capacity signal yet fall back to an equal share so a freshly-registered
/// group is not reported as all-zero.
#[must_use]
pub fn conns_from_stats(stats: &StatsSnapshot) -> Vec<TelemetryConn> {
    let weights: Vec<f64> = stats
        .links
        .iter()
        .map(|l| {
            if l.connected && !l.timed_out {
                f64::from(l.base_score.max(0)) * l.quality_multiplier
            } else {
                0.0
            }
        })
        .collect();
    let total: f64 = weights.iter().sum();
    let active = stats
        .links
        .iter()
        .filter(|l| l.connected && !l.timed_out)
        .count();

    stats
        .links
        .iter()
        .enumerate()
        .map(|(idx, l)| {
            let is_active = l.connected && !l.timed_out;
            let weight_percent = if !is_active {
                0
            } else if total > 0.0 {
                weight_share_percent(weights[idx], total)
            } else {
                equal_share_percent(active)
            };
            TelemetryConn {
                conn_id: idx as u32,
                rtt_ms: l.rtt_ms,
                nak_count: l.nak_count.max(0) as u32,
                weight_percent,
                window: l.window,
                in_flight: l.in_flight,
                // LinkStats.bitrate_bps is already wire bytes/s; the x8 to bits/s
                // is applied once, at JSON serialization.
                bitrate_bytes_per_sec: l.bitrate_bps,
                bytes_sent_total: l.bytes_sent_total,
                iface: l.iface.clone(),
                link_id: l.link_id.clone(),
            }
        })
        .collect()
}

/// One link's percentage of the total selection weight, rounded and clamped to
/// the schema's `0..=100` range.
fn weight_share_percent(weight: f64, total: f64) -> u8 {
    let pct = (weight / total * 100.0).round();
    pct.clamp(0.0, 100.0) as u8
}

/// Equal share among `active` links (the no-capacity-signal fallback).
fn equal_share_percent(active: usize) -> u8 {
    100usize
        .checked_div(active)
        .map_or(0, |share| share.min(100) as u8)
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;
    use crate::bind_map::{
        BindMapDisposition, BindMapReport, BindMapStatus, CollisionGroup, DegradedReason,
        Resolution,
    };
    use crate::stats::LinkStats;

    /// The ADR-001 canonical 312500 B/s -> 2_500_000 bps example link.
    pub(crate) fn sample_conn() -> TelemetryConn {
        TelemetryConn {
            conn_id: 0,
            rtt_ms: 42,
            nak_count: 3,
            weight_percent: 85,
            window: 8192,
            in_flight: 100,
            bitrate_bytes_per_sec: 312_500,
            bytes_sent_total: 812_000_000,
            iface: None,
            link_id: None,
        }
    }

    fn legacy() -> BindMapReport {
        BindMapReport::default()
    }

    fn json(conns: &[TelemetryConn], session_bytes: u64) -> String {
        build_telemetry_json(
            1,
            &TelemetryInputs {
                conns,
                session_bytes_sent: session_bytes,
                bind_map: &legacy(),
            },
        )
    }

    fn link(score: i32, active: bool, bytes_per_sec: u32) -> LinkStats {
        LinkStats {
            ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            label: "test".to_string(),
            iface: None,
            link_id: None,
            connected: active,
            timed_out: !active,
            window: 100,
            in_flight: 0,
            rtt_ms: 20,
            nak_count: 0,
            bitrate_bps: bytes_per_sec,
            bytes_sent_total: u64::from(bytes_per_sec) * 10,
            rtt_min_ms: 0.0,
            rtt_velocity: 0.0,
            base_score: score,
            quality_multiplier: 1.0,
        }
    }

    // ---- Schema serialization: field names + types ------------------------

    #[test]
    fn schema_version_is_integer_one() {
        let doc = json(&[], 0);
        assert!(doc.contains("\"schema_version\":1"), "got {doc}");
        // It must be a number, never a string.
        assert!(!doc.contains("\"schema_version\":\"1\""));
    }

    #[test]
    fn document_is_newline_free() {
        let doc = json(&[sample_conn()], 0);
        assert!(
            !doc.contains('\n'),
            "telemetry must be a single line: {doc}"
        );
    }

    #[test]
    fn empty_connections_serialize_to_array() {
        let doc = build_telemetry_json(
            1_749_556_546_000,
            &TelemetryInputs {
                conns: &[],
                session_bytes_sent: 0,
                bind_map: &legacy(),
            },
        );
        assert!(doc.contains("\"connections\":[]"), "got {doc}");
        assert!(doc.contains("\"last_updated_ms\":1749556546000"));
    }

    #[test]
    fn conn_id_is_stringified() {
        assert!(json(&[sample_conn()], 0).contains("\"conn_id\":\"0\""));
    }

    #[test]
    fn all_schema_fields_present_and_typed() {
        let doc = json(&[sample_conn()], 0);
        for needle in [
            "\"rtt_ms\":42",
            "\"nak_count\":3",
            "\"weight_percent\":85",
            "\"window\":8192",
            "\"in_flight\":100",
            "\"bitrate_bps\":2500000",
        ] {
            assert!(doc.contains(needle), "missing {needle} in {doc}");
        }
    }

    // ---- The mandated x8 bytes/s -> bits/s conversion ---------------------

    #[test]
    fn bitrate_is_bytes_times_eight_bits_per_second() {
        // 312500 B/s -> 2500000 bps (the ADR-001 canonical example).
        let doc = json(&[sample_conn()], 0);
        assert!(doc.contains("\"bitrate_bps\":2500000"), "got {doc}");
        // The raw bytes/s value must never leak into the JSON.
        assert!(!doc.contains("312500"), "raw bytes/s leaked: {doc}");
    }

    #[test]
    fn bitrate_conversion_is_exactly_times_eight() {
        let cases = [
            (0u32, 0u64),
            (1, 8),
            (150_000, 1_200_000),
            (312_500, 2_500_000),
        ];
        for (bytes, bits) in cases {
            let conn = TelemetryConn {
                bitrate_bytes_per_sec: bytes,
                ..sample_conn()
            };
            let record = ConnRecord::from(&conn);
            assert_eq!(record.bitrate_bps, bits, "{bytes} B/s should be {bits} bps");
        }
    }

    // ---- Additive identity fields (ADR-003 echo) --------------------------

    #[test]
    fn an_unmapped_link_omits_iface_and_link_id_entirely() {
        // Given: a legacy uplink, which has neither an interface binding nor a
        // writer-assigned identity.
        // When/Then: the keys are ABSENT, not null and not empty-string — an old
        // consumer must see the document it has always seen.
        let doc = json(&[sample_conn()], 0);
        assert!(
            !doc.contains("iface"),
            "unmapped link leaked an iface: {doc}"
        );
        assert!(
            !doc.contains("link_id"),
            "unmapped link leaked a link_id: {doc}"
        );
    }

    #[test]
    fn a_mapped_link_echoes_the_writer_assigned_identity() {
        // Given: a link the sidecar named. The id is the WRITER's; the sender
        // must reproduce it byte-for-byte rather than deriving anything.
        let conn = TelemetryConn {
            iface: Some("wwan1".to_string()),
            link_id: Some("modem-b".to_string()),
            ..sample_conn()
        };

        // When: serialized.
        let doc = json(&[conn], 0);

        // Then: both fields ride along verbatim.
        assert!(doc.contains("\"iface\":\"wwan1\""), "got {doc}");
        assert!(doc.contains("\"link_id\":\"modem-b\""), "got {doc}");
    }

    #[test]
    fn twin_modems_on_one_ip_are_told_apart_by_link_id_not_conn_id() {
        // Given: the HiLink twin case — two uplinks that legacy identity would
        // have collapsed into one.
        let conns = vec![
            TelemetryConn {
                conn_id: 0,
                iface: Some("wwan0".to_string()),
                link_id: Some("modem-a".to_string()),
                ..sample_conn()
            },
            TelemetryConn {
                conn_id: 1,
                iface: Some("wwan1".to_string()),
                link_id: Some("modem-b".to_string()),
                ..sample_conn()
            },
        ];

        // When: serialized.
        let doc = json(&conns, 0);

        // Then: both identities are present and distinct, so a UI keying on
        // link_id renders two links rather than one.
        assert!(doc.contains("\"link_id\":\"modem-a\""), "got {doc}");
        assert!(doc.contains("\"link_id\":\"modem-b\""), "got {doc}");
    }

    // ---- Top-level operating mode (ADR-003 §6.4) --------------------------

    #[test]
    fn a_legacy_document_reports_absent_and_legacy_unique_only() {
        let doc = json(&[], 0);
        assert!(
            doc.contains("\"bind_map_status\":{\"state\":\"absent\"}"),
            "got {doc}"
        );
        assert!(
            doc.contains("\"disposition\":{\"state\":\"legacy_unique_only\"}"),
            "got {doc}"
        );
    }

    #[test]
    fn a_degraded_startup_publishes_the_collision_group_it_excluded() {
        // Given: the sender fell open at startup and dropped one of two modems
        // sharing an IP.
        let report = BindMapReport::from(&Resolution {
            status: BindMapStatus::Degraded(DegradedReason::RetryExhausted),
            disposition: BindMapDisposition::StartupCollisionExcluded,
            links: Vec::new(),
            excluded: vec![CollisionGroup {
                ip: "192.168.8.100".parse().expect("fixture IP"),
                effective_index: 0,
                excluded_indices: vec![1],
            }],
            applied: None,
        });

        // When: a snapshot is published.
        let doc = build_telemetry_json(
            1,
            &TelemetryInputs {
                conns: &[],
                session_bytes_sent: 0,
                bind_map: &report,
            },
        );

        // Then: the UI has everything it needs to explain "two modems, one
        // link" without reading a log line.
        assert!(
            doc.contains(
                "\"bind_map_status\":{\"state\":\"degraded\",\"reason\":\"retry_exhausted\"}"
            ),
            "got {doc}"
        );
        assert!(
            doc.contains("\"state\":\"startup_collision_excluded\""),
            "got {doc}"
        );
        assert!(doc.contains("\"ip\":\"192.168.8.100\""), "got {doc}");
        assert!(doc.contains("\"excluded_indices\":[1]"), "got {doc}");
    }

    #[test]
    fn the_operating_mode_pair_comes_after_the_frozen_fields() {
        // The ADR-001 prefix must stay byte-stable so a byte-diff of two
        // documents localizes a change to the additive tail.
        let doc = json(&[], 0);
        let frozen_end = doc
            .find("\"bytes_sent_total\"")
            .expect("frozen tail present");
        let additive = doc
            .find("\"bind_map_status\"")
            .expect("additive tail present");
        assert!(
            frozen_end < additive,
            "the additive pair must follow the frozen fields: {doc}"
        );
        assert!(
            doc.starts_with("{\"schema_version\":1,\"last_updated_ms\":"),
            "got {doc}"
        );
    }

    // ---- ADR-001 contract pins (T10) -------------------------------------
    //
    // These two pin the producer side of the ADR-001 stats-file contract in one
    // place: the full JSON shape + the mandated x8 bitrate, and the "running
    // but idle" empty-connections case.

    #[test]
    fn telemetry_json_shape_and_bitrate_x8() {
        // One snapshot with the ADR-001 canonical 312500 B/s link asserts the
        // whole shape at once: the schema tag, every field the frozen
        // `@ceralive/srtla` Zod reader requires, the x8 bitrate, and the
        // single-line invariant the atomic publish depends on.
        let doc = build_telemetry_json(
            1_749_556_546_000,
            &TelemetryInputs {
                conns: &[sample_conn()],
                session_bytes_sent: 0,
                bind_map: &legacy(),
            },
        );

        assert!(doc.starts_with("{\"schema_version\":1,"), "got {doc}");
        assert!(!doc.contains("\"schema_version\":\"1\""));

        // 312500 B/s x 8 == 2_500_000 bps; the raw bytes/s must never leak.
        assert!(doc.contains("\"bitrate_bps\":2500000"), "got {doc}");
        assert!(!doc.contains("312500"), "raw bytes/s leaked: {doc}");

        for needle in [
            "\"last_updated_ms\":1749556546000",
            "\"conn_id\":\"0\"",
            "\"rtt_ms\":42",
            "\"nak_count\":3",
            "\"weight_percent\":85",
            "\"window\":8192",
            "\"in_flight\":100",
            "\"bitrate_bps\":2500000",
        ] {
            assert!(doc.contains(needle), "missing {needle} in {doc}");
        }

        assert!(!doc.contains('\n'), "telemetry must be one line: {doc}");
    }

    #[test]
    fn telemetry_idle_connections_empty() {
        // "running but idle": a live process with no active uplinks still
        // serializes an empty array (distinct from an absent file), keeping the
        // schema tag and timestamp so a reader can tell idle from stale.
        let doc = build_telemetry_json(
            1_749_556_546_000,
            &TelemetryInputs {
                conns: &[],
                session_bytes_sent: 0,
                bind_map: &legacy(),
            },
        );
        assert!(doc.contains("\"connections\":[]"), "got {doc}");
        assert!(doc.contains("\"schema_version\":1"), "got {doc}");
        assert!(
            doc.contains("\"last_updated_ms\":1749556546000"),
            "got {doc}"
        );
        assert!(!doc.contains('\n'), "telemetry must be one line: {doc}");
    }

    // ---- Weight normalization --------------------------------------------

    #[test]
    fn weight_share_normalizes_to_one_hundred() {
        assert_eq!(weight_share_percent(5.0, 10.0), 50);
        assert_eq!(weight_share_percent(10.0, 10.0), 100);
        // 2:1 split rounds to 67 / 33.
        assert_eq!(weight_share_percent(2.0, 3.0), 67);
        assert_eq!(weight_share_percent(1.0, 3.0), 33);
    }

    #[test]
    fn equal_share_fallback_distributes_evenly() {
        assert_eq!(equal_share_percent(0), 0);
        assert_eq!(equal_share_percent(1), 100);
        assert_eq!(equal_share_percent(2), 50);
        assert_eq!(equal_share_percent(4), 25);
    }

    #[test]
    fn conns_from_stats_indexes_and_normalizes() {
        let snap = StatsSnapshot {
            // two equal active links + one timed-out link
            links: vec![link(10, true, 100), link(10, true, 200), link(0, false, 0)],
            ..Default::default()
        };
        let conns = conns_from_stats(&snap);

        assert_eq!(conns.len(), 3);
        // conn_id is the 0-based IP-list index.
        assert_eq!(conns[0].conn_id, 0);
        assert_eq!(conns[1].conn_id, 1);
        assert_eq!(conns[2].conn_id, 2);
        // Two equal active links split 50/50; the inactive link reports 0.
        assert_eq!(conns[0].weight_percent, 50);
        assert_eq!(conns[1].weight_percent, 50);
        assert_eq!(conns[2].weight_percent, 0);
        // Wire bytes/s carried through verbatim (x8 applied only at serialization).
        assert_eq!(conns[1].bitrate_bytes_per_sec, 200);
    }

    #[test]
    fn conns_from_stats_equal_share_when_no_capacity_signal() {
        // Active links whose base_score is 0 still get a non-zero equal share.
        let snap = StatsSnapshot {
            links: vec![link(0, true, 0), link(0, true, 0)],
            ..Default::default()
        };
        let conns = conns_from_stats(&snap);
        assert_eq!(conns[0].weight_percent, 50);
        assert_eq!(conns[1].weight_percent, 50);
    }

    #[test]
    fn conns_from_stats_carries_the_link_identity_through() {
        // Given: a mapped link in the shared stats snapshot.
        let mapped = LinkStats {
            iface: Some("wwan0".to_string()),
            link_id: Some("modem-a".to_string()),
            ..link(10, true, 100)
        };
        let snap = StatsSnapshot {
            links: vec![mapped, link(10, true, 100)],
            ..Default::default()
        };

        // When: projected into telemetry records.
        let conns = conns_from_stats(&snap);

        // Then: the identity is echoed, and the unmapped neighbour still has
        // none — the projection never invents one.
        assert_eq!(conns[0].iface.as_deref(), Some("wwan0"));
        assert_eq!(conns[0].link_id.as_deref(), Some("modem-a"));
        assert_eq!(conns[1].iface, None);
        assert_eq!(conns[1].link_id, None);
    }

    #[test]
    fn the_snapshots_operating_mode_reaches_the_document() {
        // Given: a stats snapshot taken while a degraded reload was retained.
        let snap = StatsSnapshot {
            bind_map: BindMapReport::degraded(
                DegradedReason::Malformed,
                BindMapDisposition::RetainedLastValid,
            ),
            ..Default::default()
        };

        // When/Then: the whole-snapshot serializer carries it, so the file sink
        // and the event broadcaster cannot disagree about the operating mode.
        let doc = build_telemetry_json_from_stats(1, &snap);
        assert!(doc.contains("\"reason\":\"malformed\""), "got {doc}");
        assert!(
            doc.contains("\"state\":\"retained_last_valid\""),
            "got {doc}"
        );
    }
}
