use std::process::Command;

#[test]
fn adaptive_dry_run_accepts_the_mode_and_preserves_legacy_output() {
    // Given a valid source list and the two explicit mode spellings.
    let directory = tempfile::tempdir().unwrap();
    let ips = directory.path().join("ips");
    std::fs::write(&ips, "127.0.0.1\n").unwrap();
    let run = |mode| {
        Command::new(env!("CARGO_BIN_EXE_srtla_send"))
            .args(["5000", "127.0.0.1", "5001"])
            .arg(&ips)
            .args(["--dry-run", "--mode", mode])
            .output()
            .unwrap()
    };
    let legacy = run("enhanced");
    // When the real binary parses adaptive, then validation succeeds byte-identically.
    let adaptive = run("adaptive");
    assert!(legacy.status.success());
    assert!(
        adaptive.status.success(),
        "{}",
        String::from_utf8_lossy(&adaptive.stderr)
    );
    assert!(!adaptive.stdout.is_empty());
    assert_eq!(adaptive.stdout, legacy.stdout);
}

#[test]
fn adaptive_dry_run_preserves_the_invalid_ip_list_error() {
    // Given an empty source list accepted by neither mode's dry-run contract.
    let directory = tempfile::tempdir().unwrap();
    let ips = directory.path().join("ips");
    std::fs::write(&ips, "").unwrap();
    let run = |mode| {
        Command::new(env!("CARGO_BIN_EXE_srtla_send"))
            .args(["5000", "127.0.0.1", "5001"])
            .arg(&ips)
            .args(["--dry-run", "--mode", mode])
            .output()
            .unwrap()
    };
    let legacy = run("enhanced");
    // When adaptive parses successfully, then it reaches the same IP validation error.
    let adaptive = run("adaptive");
    assert!(!legacy.status.success());
    assert_eq!(adaptive.status.code(), legacy.status.code());
    assert_eq!(adaptive.stderr, legacy.stderr);
}
