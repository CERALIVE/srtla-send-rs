//! The `srtla_send` process boundary for ADR-003: the pre-spawn capability probe
//! and the parity guarantee that a legacy invocation is untouched.
//!
//! Scope: what only a spawned binary can prove — exit codes, stdout bytes, and
//! that the probe writes nothing. The file-level contract lives in
//! `bind_map_contract`.

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_srtla_send");

// ---- the pre-spawn probe, at the process boundary --------------------

#[test]
fn capabilities_json_exits_zero_with_a_parseable_document() {
    let out = Command::new(BIN)
        .arg("--capabilities-json")
        .env("RUST_LOG", "off")
        .output()
        .expect("spawn srtla_send");

    assert_eq!(
        out.status.code(),
        Some(0),
        "the probe must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let doc: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout must be the capability document alone");
    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["binary"], "srtla_send");
    assert_eq!(doc["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(doc["capabilities"]["bind_map"], cfg!(target_os = "linux"));
    assert_eq!(doc["capabilities"]["bind_map_schema_version"], 1);
    assert_eq!(doc["capabilities"]["capabilities_json"], true);
}

#[test]
fn the_probe_needs_no_configuration_and_touches_nothing() {
    // Side-effect free and bounded: no positionals, no sockets, no files.
    let dir = tempfile::tempdir().expect("tempdir");
    let before: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read dir")
        .map(|e| e.expect("entry").file_name())
        .collect();

    let out = Command::new(BIN)
        .arg("--capabilities-json")
        .current_dir(dir.path())
        .output()
        .expect("spawn srtla_send");
    assert_eq!(out.status.code(), Some(0));

    let after: Vec<_> = std::fs::read_dir(dir.path())
        .expect("read dir")
        .map(|e| e.expect("entry").file_name())
        .collect();
    assert_eq!(before, after, "the probe must not create files");
}

#[test]
fn an_unknown_flag_exits_nonzero_which_is_the_old_binary_signal() {
    // Callers MUST treat any non-zero exit as "no support" and fall back to the
    // legacy spawn — never match on the code or the message. This pins the
    // behavior a pre-ADR-003 binary gives for `--capabilities-json` itself.
    let out = Command::new(BIN)
        .arg("--a-flag-this-binary-does-not-have")
        .output()
        .expect("spawn srtla_send");

    assert!(
        !out.status.success(),
        "an unknown flag must never exit 0 — that is the whole downgrade signal"
    );
    assert!(
        serde_json::from_slice::<serde_json::Value>(&out.stdout).is_err(),
        "an unknown flag must not emit a parseable capability document"
    );
}

// ---- the parity guarantee, at the process boundary -------------------

#[test]
fn a_legacy_invocation_without_bind_map_produces_byte_identical_output() {
    // Four positionals + an IP-only file. Any deviation here is a parity break,
    // so the expected stdout is pinned literally rather than pattern-matched.
    let ips = tempfile::NamedTempFile::new().expect("temp ips file");
    std::fs::write(ips.path(), "10.0.0.1\n10.0.1.2\n").expect("write ips");

    let out = Command::new(BIN)
        .args([
            "5000",
            "127.0.0.1",
            "5001",
            ips.path().to_str().unwrap(),
            "--dry-run",
        ])
        .env("RUST_LOG", "off")
        .output()
        .expect("spawn srtla_send");

    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "dry-run: configuration valid; no sockets bound\nreceiver 127.0.0.1:5001 resolves to 1 \
         address(es):\n\x20 127.0.0.1:5001\nsource uplink IPs (2):\n\x20 10.0.0.1\n\x20 10.0.1.2\n",
        "the legacy dry-run output must not gain a single byte from the bind-map work"
    );
}

#[test]
fn dry_run_validates_the_sidecar_too_and_refuses_an_unusable_one() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ips = dir.path().join("srtla_ips");
    let sidecar = dir.path().join("bind_map.json");
    std::fs::write(&ips, "10.0.0.1\n").expect("write ips");
    std::fs::write(&sidecar, r#"{"schema_version":1,"generation":1}"#).expect("write sidecar");

    let out = Command::new(BIN)
        .args([
            "5000",
            "127.0.0.1",
            "5001",
            ips.to_str().unwrap(),
            "--dry-run",
            "--bind-map",
            sidecar.to_str().unwrap(),
        ])
        .env("RUST_LOG", "off")
        .output()
        .expect("spawn srtla_send");

    assert!(
        !out.status.success(),
        "--dry-run exists to answer 'is this configuration valid?'; a broken map is a no"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("bind-map sidecar") && stderr.contains("unusable"),
        "the failure must name the sidecar; got: {stderr}"
    );
}
