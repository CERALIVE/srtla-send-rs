//! Command-line surface for the `srtla_send` binary.
//!
//! The clap derive definition lives in the LIBRARY (not in `src/main.rs`) so
//! integration tests — which link the library crate and cannot see a
//! binary-private item — can introspect the parser structurally. See
//! `tests/cli_surface.rs`, which walks the [`clap::Command`] returned by
//! [`cli_command`] to pin the load-bearing parity contract:
//!
//! ```text
//! srtla_send <SRT_LISTEN_PORT> <SRTLA_HOST> <SRTLA_PORT> <BIND_IPS_FILE> [OPTIONS]
//! ```
//!
//! The positional order and every flag name/default here are consumed verbatim
//! by CeraUI (`buildSrtlaSendArgs`); see `AGENTS.md` → PARITY CONTRACT.

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use anyhow::{Context, Result, anyhow};
use clap::{CommandFactory, Parser};

use crate::config;
use crate::mode::SchedulingMode;

/// Default telemetry write cadence in milliseconds (`--stats-file-interval`).
pub const DEFAULT_STATS_FILE_INTERVAL_MS: u64 = 1000;

#[derive(Parser, Debug)]
#[command(
    name = "srtla_send",
    author,
    version,
    disable_version_flag = true,
    about = "SRTLA sender CLI",
    override_usage = "srtla_send [OPTIONS] SRT_LISTEN_PORT SRTLA_HOST SRTLA_PORT BIND_IPS_FILE"
)]
pub struct Cli {
    /// Print the version and exit
    #[arg(short = 'v', long = "version", action = clap::ArgAction::SetTrue)]
    pub print_version: bool,

    /// Local UDP port to listen for SRT packets (from srt-live-transmit or SRT
    /// app)
    #[arg(required_unless_present = "print_version")]
    pub local_srt_port: Option<u16>,
    /// Receiver host (srtla_rec or SRT listener)
    #[arg(required_unless_present = "print_version")]
    pub receiver_host: Option<String>,
    /// Receiver UDP port to send SRTLA packets to
    #[arg(required_unless_present = "print_version")]
    pub receiver_port: Option<u16>,
    /// Path to file containing newline-separated local source IPs to use for
    /// uplinks
    #[arg(required_unless_present = "print_version")]
    pub ips_file: Option<String>,

    /// Enable verbose (debug-level) logging
    #[arg(long = "verbose")]
    pub verbose: bool,

    /// Validate the IP list and resolve the receiver, then exit without binding
    /// sockets (non-zero exit if the IP list is unusable)
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Write per-uplink telemetry JSON to this path (ADR-001 stats file).
    /// Opt-in: absent means no telemetry file is ever created.
    #[arg(long = "stats-file")]
    pub stats_file: Option<String>,

    /// Telemetry write cadence in milliseconds for `--stats-file` (default 1000).
    #[arg(long = "stats-file-interval", default_value_t = DEFAULT_STATS_FILE_INTERVAL_MS)]
    pub stats_file_interval: u64,

    /// Unix domain socket path for remote toggle control (e.g.,
    /// /tmp/srtla.sock)
    #[arg(long = "control-socket")]
    pub control_socket: Option<String>,

    /// Scheduling mode: classic, enhanced (default), rtt-threshold, edpf
    #[arg(long = "mode", value_enum, default_value = "enhanced")]
    pub mode: SchedulingMode,
    /// Disable quality scoring (enhanced/rtt-threshold only)
    #[arg(long = "no-quality")]
    pub no_quality: bool,
    /// Enable connection exploration (enhanced only)
    #[arg(long = "exploration")]
    pub exploration: bool,
    /// RTT delta threshold in ms (rtt-threshold only, links within min_rtt + delta are "fast")
    #[arg(long = "rtt-delta-ms", default_value = "30")]
    pub rtt_delta_ms: u32,
    /// [EXPERIMENTAL] Gate broadcast-ACK window growth to the earning link, with
    /// rate-limited probe growth for the rest (default OFF; unvalidated on hardware)
    #[arg(long = "earned-ack-window")]
    pub earned_ack_window: bool,
    /// [EXPERIMENTAL] Deselect a stalled link (high in-flight + no earned ACK/RTT
    /// sample) so healthy links carry traffic, re-probing so a recovered link
    /// re-enters (default OFF; unvalidated on hardware)
    #[arg(long = "stall-deselect")]
    pub stall_deselect: bool,
    /// [EXPERIMENTAL] In-flight threshold that marks a link stall-eligible for
    /// --stall-deselect
    #[arg(long = "stall-min-in-flight", default_value_t = config::STALL_MIN_IN_FLIGHT_PACKETS)]
    pub stall_min_in_flight: i32,
    /// [EXPERIMENTAL] Earned-ACK/RTT staleness window in ms for --stall-deselect
    #[arg(long = "stall-ack-stale-ms", default_value_t = config::STALL_ACK_STALE_MS)]
    pub stall_ack_stale_ms: u64,
    /// [EXPERIMENTAL] Re-probe interval in ms for --stall-deselect
    #[arg(long = "stall-reprobe-ms", default_value_t = config::STALL_REPROBE_INTERVAL_MS)]
    pub stall_reprobe_ms: u64,
}

/// The fully-built [`clap::Command`] for `srtla_send`.
///
/// Exposed so tests (and any tooling) can introspect the CLI surface —
/// positional order, flag names, defaults — without parsing an argv vector.
#[must_use]
pub fn cli_command() -> clap::Command {
    Cli::command()
}

/// Result of a `--dry-run` resolution: the parsed source IPs, any invalid
/// lines that were skipped, and the receiver addresses the host resolved to.
#[derive(Debug)]
pub struct DryRunReport {
    pub source_ips: Vec<IpAddr>,
    pub invalid_lines: Vec<(usize, String)>,
    pub receiver_addrs: Vec<SocketAddr>,
}

/// Validate the run configuration without binding any sockets.
///
/// Reads and parses `ips_file`, then resolves `receiver_host:receiver_port`.
/// Returns an error with a specific, actionable message when the IP list is
/// unusable (missing/unreadable, empty, or zero valid IPs) or when the
/// receiver address cannot be resolved. On success no sockets are bound — the
/// caller is expected to print the report and exit 0.
pub async fn dry_run_resolve(
    ips_file: &str,
    receiver_host: &str,
    receiver_port: u16,
) -> Result<DryRunReport> {
    let text = std::fs::read_to_string(ips_file)
        .with_context(|| format!("ips file not found or unreadable: {ips_file}"))?;

    let mut source_ips = Vec::new();
    let mut invalid_lines = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match IpAddr::from_str(trimmed) {
            Ok(ip) => source_ips.push(ip),
            Err(_) => invalid_lines.push((idx + 1, trimmed.to_string())),
        }
    }

    if source_ips.is_empty() {
        return match invalid_lines.first() {
            Some((line_no, content)) => Err(anyhow!(
                "no valid source IPs in {ips_file}: first invalid entry on line {line_no} \
                 ('{content}')"
            )),
            None => Err(anyhow!("no valid source IPs in {ips_file}: file is empty")),
        };
    }

    let receiver_addrs: Vec<SocketAddr> = tokio::net::lookup_host((receiver_host, receiver_port))
        .await
        .with_context(|| {
            format!("failed to resolve receiver address {receiver_host}:{receiver_port}")
        })?
        .collect();

    if receiver_addrs.is_empty() {
        return Err(anyhow!(
            "receiver host {receiver_host}:{receiver_port} resolved to no addresses"
        ));
    }

    Ok(DryRunReport {
        source_ips,
        invalid_lines,
        receiver_addrs,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use clap::error::ErrorKind;

    use super::*;

    // ---- CLI parsing: positional contract --------------------------------

    #[test]
    fn parses_positional_contract_in_order() {
        let cli =
            Cli::try_parse_from(["srtla_send", "5000", "127.0.0.1", "5001", "/tmp/srtla_ips"])
                .expect("positional contract should parse");
        assert_eq!(cli.local_srt_port, Some(5000));
        assert_eq!(cli.receiver_host.as_deref(), Some("127.0.0.1"));
        assert_eq!(cli.receiver_port, Some(5001));
        assert_eq!(cli.ips_file.as_deref(), Some("/tmp/srtla_ips"));
        // Defaults for the additive flags.
        assert!(!cli.verbose);
        assert!(!cli.dry_run);
        assert_eq!(cli.stats_file, None);
        assert_eq!(cli.stats_file_interval, DEFAULT_STATS_FILE_INTERVAL_MS);
        assert!(
            !cli.earned_ack_window,
            "the EXPERIMENTAL earned-ack-window flag defaults OFF"
        );
        assert!(
            !cli.stall_deselect,
            "the EXPERIMENTAL stall-deselect flag defaults OFF"
        );
        assert_eq!(
            cli.stall_min_in_flight,
            config::STALL_MIN_IN_FLIGHT_PACKETS,
            "stall-min-in-flight defaults to the named constant"
        );
        assert_eq!(cli.stall_ack_stale_ms, config::STALL_ACK_STALE_MS);
        assert_eq!(cli.stall_reprobe_ms, config::STALL_REPROBE_INTERVAL_MS);
    }

    #[test]
    fn stall_deselect_flag_defaults_off_and_parses_when_present() {
        let cli =
            Cli::try_parse_from(["srtla_send", "5000", "127.0.0.1", "5001", "/tmp/srtla_ips"])
                .expect("baseline should parse");
        assert!(!cli.stall_deselect, "absent flag => OFF");

        let cli = Cli::try_parse_from([
            "srtla_send",
            "5000",
            "127.0.0.1",
            "5001",
            "/tmp/srtla_ips",
            "--stall-deselect",
            "--stall-min-in-flight",
            "48",
            "--stall-ack-stale-ms",
            "4000",
            "--stall-reprobe-ms",
            "1500",
        ])
        .expect("--stall-deselect and tunables should parse");
        assert!(cli.stall_deselect, "present flag => ON");
        assert_eq!(cli.stall_min_in_flight, 48);
        assert_eq!(cli.stall_ack_stale_ms, 4000);
        assert_eq!(cli.stall_reprobe_ms, 1500);
    }

    #[test]
    fn stall_deselect_is_marked_experimental_in_help() {
        let mut cmd = cli_command();
        let mut help = Vec::new();
        cmd.write_long_help(&mut help).expect("render long help");
        let help = String::from_utf8(help).expect("help is utf8");
        assert!(
            help.contains("--stall-deselect"),
            "help must list the stall-deselect flag"
        );
        assert!(
            help.contains("--stall-min-in-flight")
                && help.contains("--stall-ack-stale-ms")
                && help.contains("--stall-reprobe-ms"),
            "help must list all three stall tunables"
        );
        assert!(
            help.matches("EXPERIMENTAL").count() >= 2,
            "the stall-deselect help text must be marked EXPERIMENTAL"
        );
    }

    #[test]
    fn earned_ack_window_flag_defaults_off_and_parses_when_present() {
        let cli =
            Cli::try_parse_from(["srtla_send", "5000", "127.0.0.1", "5001", "/tmp/srtla_ips"])
                .expect("baseline should parse");
        assert!(!cli.earned_ack_window, "absent flag => OFF");

        let cli = Cli::try_parse_from([
            "srtla_send",
            "5000",
            "127.0.0.1",
            "5001",
            "/tmp/srtla_ips",
            "--earned-ack-window",
        ])
        .expect("--earned-ack-window should parse");
        assert!(cli.earned_ack_window, "present flag => ON");
    }

    #[test]
    fn earned_ack_window_is_marked_experimental_in_help() {
        let mut cmd = cli_command();
        let mut help = Vec::new();
        cmd.write_long_help(&mut help).expect("render long help");
        let help = String::from_utf8(help).expect("help is utf8");
        assert!(
            help.contains("--earned-ack-window"),
            "help must list the flag"
        );
        assert!(
            help.contains("EXPERIMENTAL"),
            "the earned-ack-window help text must be marked EXPERIMENTAL"
        );
    }

    #[test]
    fn parses_the_ceraui_arg_vector_verbatim() {
        // The exact arg vector buildSrtlaSendArgs emits for a verbose stream
        // (see CeraUI srtla-bindings-skew.test.ts): positionals then --verbose.
        let cli = Cli::try_parse_from([
            "srtla_send",
            "9000",
            "relay.example.com",
            "8890",
            "/tmp/srtla_ips",
            "--verbose",
        ])
        .expect("CeraUI arg vector should parse");
        assert_eq!(cli.local_srt_port, Some(9000));
        assert_eq!(cli.receiver_host.as_deref(), Some("relay.example.com"));
        assert_eq!(cli.receiver_port, Some(8890));
        assert_eq!(cli.ips_file.as_deref(), Some("/tmp/srtla_ips"));
        assert!(cli.verbose);
    }

    #[test]
    fn flags_may_precede_positionals() {
        // clap accepts options before positionals; the positional order is
        // still load-bearing and must resolve identically.
        let cli = Cli::try_parse_from([
            "srtla_send",
            "--verbose",
            "5000",
            "127.0.0.1",
            "5001",
            "/tmp/srtla_ips",
        ])
        .expect("flags before positionals should parse");
        assert!(cli.verbose);
        assert_eq!(cli.local_srt_port, Some(5000));
        assert_eq!(cli.ips_file.as_deref(), Some("/tmp/srtla_ips"));
    }

    // ---- CLI parsing: individual flags -----------------------------------

    #[test]
    fn parses_stats_file_flag() {
        let cli = Cli::try_parse_from([
            "srtla_send",
            "5000",
            "127.0.0.1",
            "5001",
            "/tmp/srtla_ips",
            "--stats-file",
            "/tmp/srtla-send-stats-5000.json",
        ])
        .expect("--stats-file should parse");
        assert_eq!(
            cli.stats_file.as_deref(),
            Some("/tmp/srtla-send-stats-5000.json")
        );
    }

    #[test]
    fn parses_stats_file_interval_flag() {
        let cli = Cli::try_parse_from([
            "srtla_send",
            "5000",
            "127.0.0.1",
            "5001",
            "/tmp/srtla_ips",
            "--stats-file-interval",
            "500",
        ])
        .expect("--stats-file-interval should parse");
        assert_eq!(cli.stats_file_interval, 500);
    }

    #[test]
    fn parses_dry_run_flag() {
        let cli = Cli::try_parse_from([
            "srtla_send",
            "5000",
            "127.0.0.1",
            "5001",
            "/tmp/srtla_ips",
            "--dry-run",
        ])
        .expect("--dry-run should parse");
        assert!(cli.dry_run);
    }

    #[test]
    fn upstream_scheduler_flags_still_parse() {
        // The upstream scheduler / control-socket flags remain intact (kept
        // working, just undocumented in user-facing docs).
        let cli = Cli::try_parse_from([
            "srtla_send",
            "5000",
            "127.0.0.1",
            "5001",
            "/tmp/srtla_ips",
            "--mode",
            "classic",
            "--no-quality",
            "--exploration",
            "--rtt-delta-ms",
            "50",
            "--control-socket",
            "/tmp/srtla.sock",
        ])
        .expect("upstream flags should still parse");
        assert_eq!(cli.mode, SchedulingMode::Classic);
        assert!(cli.no_quality);
        assert!(cli.exploration);
        assert_eq!(cli.rtt_delta_ms, 50);
        assert_eq!(cli.control_socket.as_deref(), Some("/tmp/srtla.sock"));
    }

    #[test]
    fn version_flag_short_circuits_required_positionals() {
        for flag in ["-v", "--version"] {
            let cli = Cli::try_parse_from(["srtla_send", flag])
                .unwrap_or_else(|e| panic!("{flag} should parse without positionals: {e}"));
            assert!(cli.print_version);
            assert_eq!(cli.local_srt_port, None);
        }
    }

    // ---- CLI parsing: error paths (usage + non-zero exit) ----------------

    #[test]
    fn no_arguments_is_a_missing_required_argument_error() {
        let err = Cli::try_parse_from(["srtla_send"]).expect_err("missing positionals must error");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn partial_positionals_is_an_error() {
        // Only listen_port + host given; receiver_port + ips_file missing.
        let err = Cli::try_parse_from(["srtla_send", "5000", "127.0.0.1"])
            .expect_err("partial positionals must error");
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn non_numeric_port_is_an_error() {
        let err = Cli::try_parse_from(["srtla_send", "notaport", "127.0.0.1", "5001", "/tmp/ips"])
            .expect_err("non-numeric port must error");
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
    }

    // ---- --dry-run resolution: happy path --------------------------------

    fn write_temp_ips(contents: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().expect("create temp ips file");
        f.write_all(contents.as_bytes()).expect("write ips file");
        f.flush().expect("flush ips file");
        f
    }

    #[tokio::test]
    async fn dry_run_resolves_valid_ip_list() {
        let f = write_temp_ips("10.0.0.1\n10.0.1.2\n");
        let report = dry_run_resolve(f.path().to_str().unwrap(), "127.0.0.1", 5001)
            .await
            .expect("valid ip list should resolve");
        assert_eq!(
            report.source_ips,
            vec![
                IpAddr::from_str("10.0.0.1").unwrap(),
                IpAddr::from_str("10.0.1.2").unwrap(),
            ]
        );
        assert!(report.invalid_lines.is_empty());
        // 127.0.0.1:5001 must resolve to at least one socket address.
        assert!(!report.receiver_addrs.is_empty());
        assert!(report.receiver_addrs.iter().all(|a| a.port() == 5001));
    }

    #[tokio::test]
    async fn dry_run_skips_blank_and_invalid_lines_but_keeps_valid_ips() {
        let f = write_temp_ips("10.0.0.1\n\n  \nnot-an-ip\n10.0.2.3\n");
        let report = dry_run_resolve(f.path().to_str().unwrap(), "127.0.0.1", 5001)
            .await
            .expect("a list with one valid IP should still resolve");
        assert_eq!(
            report.source_ips,
            vec![
                IpAddr::from_str("10.0.0.1").unwrap(),
                IpAddr::from_str("10.0.2.3").unwrap(),
            ]
        );
        // The garbage line ("not-an-ip" on line 4) is reported but non-fatal.
        assert_eq!(report.invalid_lines, vec![(4, "not-an-ip".to_string())]);
    }

    // ---- --dry-run resolution: error paths -------------------------------

    #[tokio::test]
    async fn dry_run_missing_file_errors_with_specific_message() {
        let missing = "/tmp/srtla-dry-run-definitely-missing-file.txt";
        let err = dry_run_resolve(missing, "127.0.0.1", 5001)
            .await
            .expect_err("missing file must error");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("not found or unreadable") && msg.contains(missing),
            "unexpected error message: {msg}"
        );
    }

    #[tokio::test]
    async fn dry_run_empty_file_errors() {
        let f = write_temp_ips("\n   \n\n");
        let err = dry_run_resolve(f.path().to_str().unwrap(), "127.0.0.1", 5001)
            .await
            .expect_err("empty file must error");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("no valid source IPs") && msg.contains("empty"),
            "unexpected error message: {msg}"
        );
    }

    #[tokio::test]
    async fn dry_run_all_garbage_file_errors_with_line_number() {
        let f = write_temp_ips("garbage\nstill-not-an-ip\n");
        let err = dry_run_resolve(f.path().to_str().unwrap(), "127.0.0.1", 5001)
            .await
            .expect_err("all-garbage file must error");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("no valid source IPs") && msg.contains("line 1"),
            "unexpected error message: {msg}"
        );
    }
}
