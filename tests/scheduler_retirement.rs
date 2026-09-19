use std::process::Command;

use clap::Parser;
use srtla_send::cli::Cli;
use srtla_send::{DynamicConfig, SchedulingMode};

#[test]
fn retired_modes_are_clap_errors_before_startup() {
    // Given every retired spelling and otherwise valid positional arguments.
    for mode in ["classic", "rtt-threshold", "edpf", "adaptive"] {
        // When the real executable parses the request.
        let output = Command::new(env!("CARGO_BIN_EXE_srtla_send"))
            .args([
                "0",
                "127.0.0.1",
                "5000",
                "missing-ips",
                "--dry-run",
                "--mode",
                mode,
            ])
            .output()
            .unwrap();
        // Then clap rejects it without starting the sender and identifies the migration.
        assert_eq!(output.status.code(), Some(2), "{mode}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("4.0.0"), "{error}");
        assert!(error.contains("release-notes-4.0.0.md"), "{error}");
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn enhanced_is_the_only_mode_and_the_default() {
    // Given the public parser, enum and dynamic configuration defaults.
    let cli = Cli::try_parse_from(["srtla_send", "0", "127.0.0.1", "5000", "ips"]).unwrap();
    // When enumerating accepted values.
    let modes = <SchedulingMode as clap::ValueEnum>::value_variants();
    // Then every entry point agrees on the sole mode.
    assert_eq!(modes, &[SchedulingMode::Enhanced]);
    assert_eq!(cli.mode, SchedulingMode::Enhanced);
    assert_eq!(SchedulingMode::default(), SchedulingMode::Enhanced);
    assert_eq!(DynamicConfig::new().mode(), SchedulingMode::Enhanced);
}

#[test]
fn explicitly_supplied_retired_flags_warn_once_even_at_default_values() {
    // Given valid dry-run inputs and all retired flags, including numeric defaults.
    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips");
    std::fs::write(&ips, "127.0.0.1\n").unwrap();
    // When startup parses the flags (dry-run keeps this test socket-free).
    let output = Command::new(env!("CARGO_BIN_EXE_srtla_send"))
        .args(["0", "127.0.0.1", "5000"])
        .arg(&ips)
        .args([
            "--dry-run",
            "--no-quality",
            "--exploration",
            "--rtt-delta-ms=30",
            "--stall-deselect",
            "--stall-min-in-flight=32",
            "--stall-ack-stale-ms=3000",
            "--stall-reprobe-ms=1000",
        ])
        .env("RUST_LOG", "warn")
        .output()
        .unwrap();
    // Then exactly one warning identifies each ignored input, not its defaulted siblings.
    assert!(output.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    for flag in [
        "--no-quality",
        "--exploration",
        "--rtt-delta-ms",
        "--stall-deselect",
        "--stall-min-in-flight",
        "--stall-ack-stale-ms",
        "--stall-reprobe-ms",
    ] {
        assert_eq!(
            text.lines()
                .filter(|line| line.contains("WARN") && line.contains(flag))
                .count(),
            1,
            "{flag}: {text}"
        );
    }
    assert_eq!(text.lines().filter(|line| line.contains("WARN")).count(), 7);
}
