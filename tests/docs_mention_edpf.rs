//! Documentation-parity guard for the EDPF scheduling mode (Task 19).
//!
//! EDPF is a shipped, selectable scheduling mode (`--mode edpf`). These tests
//! pin that it stays documented in both operator-facing surfaces: the
//! `README.md` "Scheduling Modes" reference and the binary's `--help` text.
//! A regression that adds/renames the mode without documenting it fails here.

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_srtla_send");

#[test]
fn readme_mentions_edpf() {
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("read README.md");
    assert!(
        readme.to_lowercase().contains("edpf"),
        "README.md must document the EDPF scheduling mode"
    );
}

#[test]
fn help_text_mentions_edpf() {
    let output = Command::new(BIN)
        .arg("--help")
        .output()
        .expect("run srtla_send --help");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.to_lowercase().contains("edpf"),
        "--help output must list edpf as a valid --mode value"
    );
}
