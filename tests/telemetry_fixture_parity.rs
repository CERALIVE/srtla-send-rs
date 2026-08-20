//! Golden-fixture parity: the Rust producer fixtures and the TS-binding fixtures
//! are a single source kept in lockstep (Task 7, extended by the ADR-003
//! telemetry evolution).
//!
//! Each fixture is committed twice, under the same name:
//!   * `tests/fixtures/<name>.json` — written by the Rust producer
//!     (`tests/telemetry_fixtures.rs`, regenerated with `UPDATE_GOLDEN=1`);
//!   * `bindings/typescript/tests/fixtures/<name>.json` — the copy the
//!     `@ceralive/srtla-send` Zod reader parses.
//!
//! They MUST stay identical — that byte-equality is what makes "Rust writes it,
//! TypeScript parses it" a real cross-language proof rather than two files that
//! merely started out the same. A contract regression that lands in one but not the
//! other would silently corrupt the CeraUI console + ingest panel that consume
//! this telemetry — exactly the drift these assertions exist to catch. Both
//! paths are anchored at `CARGO_MANIFEST_DIR` and live inside this repo, so the
//! test never reaches above its own checkout (Rule D).

use std::path::PathBuf;

use srtla_send::telemetry_file::TELEMETRY_SCHEMA_VERSION;

/// The frozen per-connection key set the `@ceralive/srtla` Zod reader requires.
/// Sorted. These may never be dropped or renamed; the schema grows only by
/// addition, so this is asserted as a SUBSET, not as equality.
const FROZEN_CONN_KEYS: [&str; 7] = [
    "bitrate_bps",
    "conn_id",
    "in_flight",
    "nak_count",
    "rtt_ms",
    "weight_percent",
    "window",
];

/// Every per-connection key the current producer emits: the frozen ADR-001 set
/// plus ADR-002's `bytes_sent_total`. Asserted for equality so a NEW field can
/// never land in the goldens without a deliberate edit here.
const CURRENT_CONN_KEYS: [&str; 8] = [
    "bitrate_bps",
    "bytes_sent_total",
    "conn_id",
    "in_flight",
    "nak_count",
    "rtt_ms",
    "weight_percent",
    "window",
];

/// Every fixture in the cross-language matrix. `tests/telemetry_fixtures.rs`
/// documents what each one proves.
const FIXTURES: [&str; 8] = [
    "telemetry-legacy-producer",
    "telemetry-golden",
    "telemetry-mapped",
    "telemetry-reordered",
    "telemetry-reconnect",
    "telemetry-degraded-startup",
    "telemetry-degraded-reload",
    "telemetry-unknown-fields",
];

fn rust_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{name}.json"))
}

fn ts_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("bindings/typescript/tests/fixtures")
        .join(format!("{name}.json"))
}

fn rust_golden_path() -> PathBuf {
    rust_fixture_path("telemetry-golden")
}

fn ts_golden_path() -> PathBuf {
    ts_fixture_path("telemetry-golden")
}

fn read(path: &PathBuf) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("golden fixture missing at {}: {e}", path.display()))
}

fn parse(path: &PathBuf) -> serde_json::Value {
    serde_json::from_str(&read(path)).unwrap_or_else(|e| {
        panic!(
            "golden fixture at {} is not valid JSON: {e}",
            path.display()
        )
    })
}

/// Sorted top-level object keys.
fn sorted_keys(v: &serde_json::Value) -> Vec<String> {
    let mut keys: Vec<String> = v
        .as_object()
        .expect("expected a JSON object")
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

// ---- Single source: byte-for-byte identical --------------------------------

#[test]
fn rust_and_ts_goldens_are_byte_identical() {
    // The two fixtures are maintained as identical copies (one source of truth).
    // If this fails they have drifted — re-sync them rather than editing one.
    let rust = read(&rust_golden_path());
    let ts = read(&ts_golden_path());
    assert_eq!(
        rust, ts,
        "Rust and TS golden fixtures have drifted; re-sync them (single source)"
    );
    // The shared document is the single-line atomic-publish shape.
    assert!(
        !rust.contains('\n'),
        "golden fixtures must be newline-free: {rust}"
    );
}

// ---- Structural parity (survives even if byte-equality is ever relaxed) -----

#[test]
fn goldens_share_schema_version_and_top_level_keys() {
    let rust = parse(&rust_golden_path());
    let ts = parse(&ts_golden_path());

    // schema_version matches the producer constant on BOTH sides.
    let expected = serde_json::json!(TELEMETRY_SCHEMA_VERSION);
    assert_eq!(
        rust["schema_version"], expected,
        "rust schema_version drift"
    );
    assert_eq!(ts["schema_version"], expected, "ts schema_version drift");

    // Identical top-level key sets.
    assert_eq!(
        sorted_keys(&rust),
        sorted_keys(&ts),
        "top-level key sets differ between the two goldens"
    );
    assert_eq!(
        sorted_keys(&rust),
        vec![
            "bind_map_status",
            "bytes_sent_total",
            "connections",
            "disposition",
            "last_updated_ms",
            "schema_version"
        ],
        "top-level contract keys changed"
    );
}

#[test]
fn goldens_share_per_connection_key_structure() {
    let rust = parse(&rust_golden_path());
    let ts = parse(&ts_golden_path());

    let rust_conns = rust["connections"].as_array().expect("rust connections[]");
    let ts_conns = ts["connections"].as_array().expect("ts connections[]");
    assert_eq!(
        rust_conns.len(),
        ts_conns.len(),
        "connection counts differ between the two goldens"
    );
    assert!(
        !rust_conns.is_empty(),
        "the golden must exercise at least one connection"
    );

    let current: Vec<String> = CURRENT_CONN_KEYS.iter().map(|s| s.to_string()).collect();
    for (i, (r, t)) in rust_conns.iter().zip(ts_conns).enumerate() {
        let keys = sorted_keys(r);
        assert_eq!(
            keys,
            sorted_keys(t),
            "per-connection key sets differ at index {i}"
        );
        for frozen in FROZEN_CONN_KEYS {
            assert!(
                keys.iter().any(|k| k == frozen),
                "connection {i} dropped frozen ADR-001 key `{frozen}` — the schema may only grow"
            );
        }
        assert_eq!(
            keys, current,
            "connection {i} keys drifted from the current producer contract"
        );
    }
}

// ---- The whole matrix, not just the golden ---------------------------------

#[test]
fn every_fixture_has_a_byte_identical_binding_copy() {
    // The cross-language claim is "TypeScript parses the bytes Rust wrote". That
    // only holds if the two committed copies are the same bytes, for EVERY case
    // in the matrix — not only the golden.
    for name in FIXTURES {
        let rust = read(&rust_fixture_path(name));
        let ts = read(&ts_fixture_path(name));
        assert_eq!(
            rust, ts,
            "`{name}` has drifted between the two fixture directories; regenerate with \
             UPDATE_GOLDEN=1 rather than editing one side"
        );
        assert!(
            !rust.contains('\n'),
            "`{name}` must be the newline-free atomic-publish document"
        );
    }
}

#[test]
fn the_current_golden_is_the_legacy_document_plus_the_additive_tail() {
    // The additivity claim, asserted rather than asserted-about: strip the four
    // keys this change introduced from the current producer's golden and what
    // remains must be BYTE-for-byte the document the pre-ADR-003 producer wrote.
    let mut current: serde_json::Value =
        serde_json::from_str(&read(&rust_golden_path())).expect("golden is valid JSON");
    let object = current.as_object_mut().expect("golden is an object");
    object.remove("bind_map_status");
    object.remove("disposition");

    let legacy: serde_json::Value =
        serde_json::from_str(&read(&rust_fixture_path("telemetry-legacy-producer")))
            .expect("legacy fixture is valid JSON");

    assert_eq!(
        current, legacy,
        "the golden must differ from the legacy producer document ONLY by the additive tail"
    );
}
