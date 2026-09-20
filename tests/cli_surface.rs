//! Machine-readable pin of the `srtla_send` command-line surface.
//!
//! These tests spawn the real binary via `CARGO_BIN_EXE_srtla_send` and pin the
//! CeraLive device-integration contract (`AGENTS.md` → PARITY CONTRACT):
//!
//! ```text
//! srtla_send <SRT_LISTEN_PORT> <SRTLA_HOST> <SRTLA_PORT> <BIND_IPS_FILE> [OPTIONS]
//! ```
//!
//! They cover the `--verbose` / `--dry-run` arms only (the flags this port
//! adds). `--dry-run` must print the resolved receiver + uplink IPs and exit
//! `0` without binding any socket; an unusable IP list (missing/unreadable,
//! empty, all-invalid) or an unresolvable host must exit non-zero with a
//! specific stderr message.
//!
//! Changing the positional order, the flag names, or the mode value set is a
//! deliberate, versioned CLI change — never an incidental refactor.

use std::io::Write;
use std::net::UdpSocket;
use std::process::{Command, Output};

use tempfile::NamedTempFile;

const BIN: &str = env!("CARGO_BIN_EXE_srtla_send");

/// `main` returns `anyhow::Result`, so any `Err` becomes `ExitCode::FAILURE`.
const FAILURE_CODE: i32 = 1;

fn run(args: &[&str]) -> Output {
    Command::new(BIN)
        .args(args)
        .output()
        .expect("spawn srtla_send")
}

/// Bind an ephemeral UDP port and hold the socket so the spawned binary cannot
/// bind it. A dry-run that succeeds against this occupied port proves the
/// dry-run path never touched the local SRT listener socket.
fn occupied_udp_port() -> (UdpSocket, u16) {
    let sock = UdpSocket::bind("127.0.0.1:0").expect("bind ephemeral udp");
    let port = sock.local_addr().unwrap().port();
    (sock, port)
}

fn write_temp_ips(contents: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().expect("create temp ips file");
    f.write_all(contents.as_bytes()).expect("write ips file");
    f.flush().expect("flush ips file");
    f
}

/// Spawn `srtla_send <listen_port> <host> <recv_port> <ips_file> --dry-run`.
/// `RUST_LOG` is silenced so stdout carries only the dry-run report lines.
fn run_dry_run(listen_port: u16, host: &str, recv_port: u16, ips_file: &str) -> Output {
    Command::new(BIN)
        .args([
            &listen_port.to_string(),
            host,
            &recv_port.to_string(),
            ips_file,
            "--dry-run",
        ])
        .env("RUST_LOG", "off")
        .output()
        .expect("spawn srtla_send")
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

// ---- CLI surface: flag presence and positional contract ------------------

#[test]
fn help_lists_the_ceraui_control_plane_flags() {
    let out = run(&["--help"]);
    assert_eq!(out.status.code(), Some(0), "clap renders help to stdout");
    let help = stdout_of(&out);
    assert!(
        help.contains("--verbose"),
        "help must list --verbose: {help}"
    );
    assert!(
        help.contains("--dry-run"),
        "help must list --dry-run: {help}"
    );
    assert!(
        help.contains("--stats-file"),
        "help must list --stats-file: {help}"
    );
    assert!(
        help.contains("--stats-file-interval"),
        "help must list --stats-file-interval: {help}"
    );
    assert!(
        help.contains("--bind-map"),
        "help must list --bind-map: {help}"
    );
    assert!(
        help.contains("srtla_send [OPTIONS] SRT_LISTEN_PORT SRTLA_HOST SRTLA_PORT BIND_IPS_FILE"),
        "usage must keep the documented positional shape: {help}"
    );
}

#[test]
fn version_flag_short_circuits_the_required_positionals() {
    for flag in ["-v", "--version"] {
        let out = run(&[flag]);
        assert_eq!(out.status.code(), Some(0), "{flag} must exit 0");
        assert!(
            !stdout_of(&out).trim().is_empty(),
            "{flag} must print the version line"
        );
    }
}

#[test]
fn omitting_the_positionals_is_a_usage_error() {
    let out = run(&[]);
    assert_ne!(
        out.status.code(),
        Some(0),
        "missing positionals must be a usage error"
    );
    assert!(
        stderr_of(&out).contains("Usage"),
        "clap must render usage on stderr; got: {}",
        stderr_of(&out)
    );
}

#[test]
fn mode_accepts_only_the_upstream_value_set() {
    let ips = write_temp_ips("192.0.2.10\n");
    let ips_path = ips.path().to_str().unwrap().to_string();

    let classic = run(&[
        "--mode",
        "classic",
        "6100",
        "127.0.0.1",
        "5000",
        &ips_path,
        "--dry-run",
    ]);
    assert_eq!(
        classic.status.code(),
        Some(0),
        "--mode classic must parse; stderr: {}",
        stderr_of(&classic)
    );

    let rejected = run(&[
        "--mode",
        "rtt-threshold",
        "6100",
        "127.0.0.1",
        "5000",
        &ips_path,
    ]);
    assert_ne!(
        rejected.status.code(),
        Some(0),
        "adding mode values is out of scope: rtt-threshold must be rejected"
    );
}

// ---- --dry-run: success path --------------------------------------------

#[test]
fn dry_run_valid_exits_zero_and_prints_both_ips() {
    let (_held, listen_port) = occupied_udp_port();
    let ips = write_temp_ips("192.0.2.10\n198.51.100.23\n");

    let out = run_dry_run(listen_port, "127.0.0.1", 5000, ips.path().to_str().unwrap());

    assert_eq!(
        out.status.code(),
        Some(0),
        "valid dry-run must exit 0; stderr: {}",
        stderr_of(&out)
    );
    let stdout = stdout_of(&out);
    assert!(stdout.contains("no sockets bound"), "got: {stdout}");
    assert!(
        stdout.contains("receiver 127.0.0.1:5000 resolves to"),
        "got: {stdout}"
    );
    assert!(stdout.contains("192.0.2.10"), "first IP missing: {stdout}");
    assert!(
        stdout.contains("198.51.100.23"),
        "second IP missing: {stdout}"
    );
    assert!(stdout.contains("source uplink IPs (2):"), "got: {stdout}");
}

#[test]
fn dry_run_does_not_bind_the_listener_socket() {
    // The occupied port is the strongest no-bind proof: the binary would fail
    // with "address in use" if the dry-run path bound the listener.
    let (_held, listen_port) = occupied_udp_port();
    let ips = write_temp_ips("127.0.0.1\n");

    let out = run_dry_run(listen_port, "127.0.0.1", 5000, ips.path().to_str().unwrap());

    assert_eq!(
        out.status.code(),
        Some(0),
        "dry-run must succeed while the listener port is held; stderr: {}",
        stderr_of(&out)
    );
}

#[test]
fn dry_run_mixed_valid_and_invalid_still_succeeds() {
    let (_held, listen_port) = occupied_udp_port();
    let ips = write_temp_ips("192.0.2.10\nnot-an-ip\n198.51.100.23\n");

    let out = run_dry_run(listen_port, "127.0.0.1", 5000, ips.path().to_str().unwrap());

    assert_eq!(
        out.status.code(),
        Some(0),
        "a mixed list with valid IPs must succeed; stderr: {}",
        stderr_of(&out)
    );
    assert!(stdout_of(&out).contains("source uplink IPs (2):"));
}

#[test]
fn verbose_and_dry_run_parse_together_after_the_positionals() {
    let ips = write_temp_ips("192.0.2.10\n");
    let out = run(&[
        "6100",
        "127.0.0.1",
        "5000",
        ips.path().to_str().unwrap(),
        "--verbose",
        "--dry-run",
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "--verbose must not disturb the positional contract; stderr: {}",
        stderr_of(&out)
    );
    assert!(stdout_of(&out).contains("source uplink IPs (1):"));
}

#[test]
fn dry_run_flag_may_precede_the_positionals() {
    let ips = write_temp_ips("192.0.2.10\n");
    let out = run(&[
        "--dry-run",
        "6100",
        "127.0.0.1",
        "5000",
        ips.path().to_str().unwrap(),
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "options before positionals must parse; stderr: {}",
        stderr_of(&out)
    );
}

// ---- --dry-run: error paths ---------------------------------------------

#[test]
fn dry_run_empty_ip_file_exits_nonzero() {
    let (_held, listen_port) = occupied_udp_port();
    let ips = write_temp_ips("\n   \n\n");

    let out = run_dry_run(listen_port, "127.0.0.1", 5000, ips.path().to_str().unwrap());

    assert_eq!(
        out.status.code(),
        Some(FAILURE_CODE),
        "empty ip file must exit {FAILURE_CODE}"
    );
    let stderr = stderr_of(&out);
    assert!(
        stderr.contains("ips file is empty"),
        "stderr must name the empty file; got: {stderr}"
    );
    assert!(
        stderr.contains("no valid source IPs"),
        "stderr must name the empty-list cause; got: {stderr}"
    );
}

#[test]
fn dry_run_missing_ip_file_exits_nonzero() {
    let (_held, listen_port) = occupied_udp_port();
    let missing = "/tmp/srtla-send-rs-task12-definitely-missing.txt";

    let out = run_dry_run(listen_port, "127.0.0.1", 5000, missing);

    assert_eq!(
        out.status.code(),
        Some(FAILURE_CODE),
        "missing ip file must exit {FAILURE_CODE}"
    );
    let stderr = stderr_of(&out);
    assert!(
        stderr.contains("not found or unreadable") && stderr.contains(missing),
        "stderr must name the unreadable file; got: {stderr}"
    );
}

#[test]
fn dry_run_all_invalid_ip_file_exits_nonzero() {
    let (_held, listen_port) = occupied_udp_port();
    let ips = write_temp_ips("garbage\nstill-not-an-ip\n");

    let out = run_dry_run(listen_port, "127.0.0.1", 5000, ips.path().to_str().unwrap());

    assert_eq!(
        out.status.code(),
        Some(FAILURE_CODE),
        "all-invalid ip file must exit {FAILURE_CODE}"
    );
    let stderr = stderr_of(&out);
    assert!(
        stderr.contains("no valid source IPs") && stderr.contains("line 1"),
        "stderr must name the first invalid line; got: {stderr}"
    );
}

#[test]
fn dry_run_unresolvable_host_exits_nonzero() {
    let (_held, listen_port) = occupied_udp_port();
    // `.invalid` is reserved by RFC 6761 and must never resolve.
    let ips = write_temp_ips("127.0.0.1\n");

    let out = run_dry_run(
        listen_port,
        "srtla-send-rs-task12.invalid",
        5000,
        ips.path().to_str().unwrap(),
    );

    assert_eq!(
        out.status.code(),
        Some(FAILURE_CODE),
        "unresolvable host must exit {FAILURE_CODE}; stdout: {}",
        stdout_of(&out)
    );
    assert!(
        stderr_of(&out).contains("failed to resolve receiver address"),
        "stderr must name the resolution failure; got: {}",
        stderr_of(&out)
    );
}
