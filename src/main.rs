#[cfg(not(loom))]
use anyhow::{Context, Result};
#[cfg(not(loom))]
use clap::Parser;
#[cfg(not(loom))]
use srtla_send::cli::{Cli, dry_run_resolve};
#[cfg(not(loom))]
use srtla_send::{config, sender, stats, subscription, telemetry_file, version};
#[cfg(not(loom))]
use tracing::{info, warn};
#[cfg(not(loom))]
use tracing_subscriber::EnvFilter;

// Use mimalloc as the global allocator for the binary (non-Windows only).
// Gated default-on via the `mimalloc-allocator` feature; --no-default-features
// builds fall back to the system allocator (docs/notes/mimalloc-decision.md).
#[cfg(all(not(windows), feature = "mimalloc-allocator"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(loom)]
fn main() {}

#[cfg(not(loom))]
#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Cli::parse();

    if args.print_version {
        println!("{}", version::version_line());
        return Ok(());
    }

    // `--verbose` raises the log level to debug (parity with the C sender's
    // `--verbose`); otherwise honor RUST_LOG via the env filter.
    let env_filter = if args.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::from_default_env()
    };
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .init();

    let local_srt_port = args.local_srt_port.expect("required");
    let receiver_host = args.receiver_host.as_deref().expect("required");
    let receiver_port = args.receiver_port.expect("required");
    let ips_file = args.ips_file.as_deref().expect("required");

    if args.dry_run {
        let report = dry_run_resolve(ips_file, receiver_host, receiver_port).await?;
        for (line_no, content) in &report.invalid_lines {
            warn!("ignoring invalid IP on line {line_no}: '{content}'");
        }
        println!("dry-run: configuration valid; no sockets bound");
        println!(
            "receiver {receiver_host}:{receiver_port} resolves to {} address(es):",
            report.receiver_addrs.len()
        );
        for addr in &report.receiver_addrs {
            println!("  {addr}");
        }
        println!("source uplink IPs ({}):", report.source_ips.len());
        for ip in &report.source_ips {
            println!("  {ip}");
        }
        return Ok(());
    }

    if let Some(stats_file) = args.stats_file.as_deref() {
        info!(
            "telemetry stats-file requested at {stats_file} (cadence {} ms)",
            args.stats_file_interval
        );
    }

    let config = config::DynamicConfig::from_cli(
        args.mode,
        args.no_quality,
        args.exploration,
        args.rtt_delta_ms,
        args.earned_ack_window,
        args.stall_deselect,
        args.stall_min_in_flight,
        args.stall_ack_stale_ms,
        args.stall_reprobe_ms,
    );

    // Create shared stats for telemetry export
    let shared_stats = stats::SharedStats::new();

    // Telemetry-event fan-out for `subscribe-events`; broadcast on the same tick
    // as the stats file (dual-publish), populated by the control-socket handler.
    let subscriptions = subscription::SubscriptionManager::new();

    // Start config listener (stdin or Unix socket)
    config::spawn_config_listener(
        config.clone(),
        args.control_socket,
        shared_stats.clone(),
        subscriptions.clone(),
    );

    let telemetry = args.stats_file.as_deref().map(|path| {
        telemetry_file::TelemetryWriter::new(path.to_string(), args.stats_file_interval)
    });

    let outcome = sender::run_sender_with_config(
        local_srt_port,
        receiver_host,
        receiver_port,
        ips_file,
        config,
        shared_stats,
        sender::TelemetrySinks {
            file: telemetry,
            subscriptions,
        },
    )
    .await;

    // On shutdown the telemetry writer thread is joined (via the writer's Drop as
    // `run_sender_with_config` returns): it drains its final snapshot, then
    // unlinks the live file + `.tmp` sibling. This backstop removal covers the
    // no-`--stats-file` and any-residue case so a stale snapshot never outlives
    // the process — CeraUI respawns srtla_send on every network change and would
    // read a leftover file as live.
    if let Some(stats_file) = args.stats_file.as_deref() {
        let _ = std::fs::remove_file(stats_file);
        let _ = std::fs::remove_file(format!("{stats_file}.tmp"));
    }

    outcome.context("srtla_send failed")
}
