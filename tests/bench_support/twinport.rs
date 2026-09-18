use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::harness::SlsCapture;
use network_sim::metrics::srt_stats::{CaptureSemantics, SrtStats};
use network_sim::scenarios::Profile;
use serde::{Deserialize, Serialize};

use super::record::{Attempt, Request, RunFailure};
use super::stack::Stack;

#[derive(Clone, Copy, Serialize)]
struct PlayerLeg {
    loss: u64,
    drop: u64,
    mbps_recv_rate: f64,
}

impl PlayerLeg {
    fn valid(self, offered_bps: u32) -> bool {
        self.loss == 0
            && self.drop == 0
            && self.mbps_recv_rate.is_finite()
            && self.mbps_recv_rate * 1_000_000.0 >= 0.98 * f64::from(offered_bps)
    }
}

#[derive(Deserialize, Serialize)]
struct Telemetry {
    schema_version: u32,
    last_updated_ms: u64,
    bytes_sent_total: u64,
}

pub struct Start {
    telemetry: Telemetry,
    csv: SrtStats,
    settled: bool,
    started: Instant,
}

pub fn enabled(campaign: &str) -> bool {
    matches!(campaign, "twinport-default" | "twinport-legacy-l2")
}

pub fn measures_player(campaign: &str) -> bool {
    enabled(campaign) || campaign == "m4b-lineages"
}

pub fn attempt_budget(reason: Option<&str>) -> u32 {
    match reason {
        None | Some("player_leg_invalid") => 2,
        Some(_) => 1,
    }
}

pub fn budget(directory: &Path, run: u32) -> Result<u32> {
    let path = directory.join(format!("run-{run}.failed-1.json"));
    match std::fs::read(path) {
        Ok(bytes) => {
            let previous: Attempt = serde_json::from_slice(&bytes)?;
            Ok(attempt_budget(previous.reason.as_deref()))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(2),
        Err(error) => Err(error.into()),
    }
}

fn telemetry(path: &Path) -> Result<Telemetry> {
    let value: Telemetry = serde_json::from_slice(&std::fs::read(path)?)?;
    ensure!(value.schema_version == 1, "telemetry schema");
    Ok(value)
}

fn csv(capture: &SlsCapture) -> Result<SrtStats> {
    Ok(SrtStats::parse_with_semantics(
        &super::clock::complete_csv(&capture.player_stats_path())?,
        0,
        CaptureSemantics::Interval,
    )?)
}

pub fn begin(request: &Request, profile: &Profile, stack: &Stack) -> Result<Start> {
    let expected = request
        .manifest
        .campaign
        .strip_prefix("twinport-")
        .or_else(|| (request.manifest.campaign == "m4b-lineages").then_some("default"))
        .context("campaign")?;
    let override_value = std::env::var("SLS_BONDED_PROFILE_OVERRIDE").ok();
    ensure!(
        override_value.as_deref().unwrap_or("default") == expected,
        "SLS profile mismatch"
    );
    let capture = stack.sls_capture.as_ref().context("SLS capture")?;
    let mut settle = crate::manifest::Settle::for_profile(profile);
    let warmup = Instant::now();
    let mut previous = capture.player_bytes()?;
    let mut settled = false;
    for second in 1..=30 {
        std::thread::sleep(
            (warmup + Duration::from_secs(second)).saturating_duration_since(Instant::now()),
        );
        let bytes = capture.player_bytes()?;
        let bps = f64::from(u32::try_from(
            bytes.checked_sub(previous).context("player truncated")?,
        )?) * 8.0;
        previous = bytes;
        if settle.observe(second, bps) && second >= 10 {
            settled = true;
            break;
        }
    }
    std::fs::write(
        request.artifacts.join("sls-start.json"),
        SlsCapture::raw_stats(&stack.topo.receiver_ns)?,
    )?;
    std::fs::copy(&stack.stats, request.artifacts.join("sender-start.json"))?;
    Ok(Start {
        telemetry: telemetry(&request.artifacts.join("sender-start.json"))?,
        csv: csv(capture)?,
        settled,
        started: Instant::now(),
    })
}

pub fn finish(request: &Request, profile: &Profile, stack: &Stack, start: Start) -> Result<()> {
    let capture = stack.sls_capture.as_ref().context("SLS capture")?;
    let end = telemetry(&stack.stats)?;
    std::fs::copy(&stack.stats, request.artifacts.join("sender-end.json"))?;
    std::fs::write(
        request.artifacts.join("sls-end.json"),
        SlsCapture::raw_stats(&stack.topo.receiver_ns)?,
    )?;
    let stats = csv(capture)?;
    let first = start.csv.rows.last().context("player start row")?;
    let last = stats.rows.last().context("player end row")?;
    ensure!(
        first.socket_id == last.socket_id && last.t_ms > first.t_ms,
        "player socket continuity"
    );
    let mut previous_ms = first.t_ms;
    let mut weighted_rate = 0.0;
    for row in stats.rows.iter().filter(|r| r.t_ms > first.t_ms) {
        ensure!(row.socket_id == first.socket_id, "player reconnected");
        weighted_rate += row.mbps_recv_rate * f64::from(u32::try_from(row.t_ms - previous_ms)?);
        previous_ms = row.t_ms;
    }
    let leg = PlayerLeg {
        loss: last
            .pkt_loss_total
            .checked_sub(first.pkt_loss_total)
            .context("player loss reset")?,
        drop: last
            .pkt_drop_total
            .checked_sub(first.pkt_drop_total)
            .context("player drop reset")?,
        mbps_recv_rate: weighted_rate / f64::from(u32::try_from(last.t_ms - first.t_ms)?),
    };
    let sent = end
        .bytes_sent_total
        .checked_sub(start.telemetry.bytes_sent_total)
        .context("sender counter reset")?;
    ensure!(sent > 0, "no sender bytes");
    let observed = request
        .result
        .record
        .sls_conformance
        .as_ref()
        .context("conformance")?;
    let offered = u32::try_from(profile.offered_bps)?;
    let player_leg_valid = leg.valid(offered);
    crate::checkpoint::atomic(
        &request.artifacts.join("twinport.json"),
        &serde_json::json!({
            "settled": start.settled, "player_leg_valid": player_leg_valid,
            "player_leg": leg, "player_latency_ms": 200, "offered_bps": offered,
            "player_bytes": observed.player_bytes, "sender_bytes": sent,
            "sender_start": start.telemetry, "sender_end": end,
            "player_csv_start_ms": first.t_ms, "player_csv_end_ms": last.t_ms,
            "elapsed_ms": start.started.elapsed().as_millis(),
            "sls_override": std::env::var("SLS_BONDED_PROFILE_OVERRIDE").ok(),
        }),
    )?;
    if enabled(&request.manifest.campaign) {
        ensure!(player_leg_valid, RunFailure::PlayerLegInvalid);
    }
    ensure!(start.settled, RunFailure::SettleTimeout);
    Ok(())
}

#[test]
fn player_leg_requires_all_three_predicates_at_the_exact_boundary() {
    let clean = PlayerLeg {
        loss: 0,
        drop: 0,
        mbps_recv_rate: 9.408,
    };
    assert!(clean.valid(9_600_000));
    assert!(!PlayerLeg { loss: 1, ..clean }.valid(9_600_000));
    assert!(!PlayerLeg { drop: 1, ..clean }.valid(9_600_000));
    assert!(
        !PlayerLeg {
            mbps_recv_rate: 9.407999,
            ..clean
        }
        .valid(9_600_000)
    );
    assert!(
        !PlayerLeg {
            mbps_recv_rate: f64::NAN,
            ..clean
        }
        .valid(9_600_000)
    );
}

#[test]
fn lineage_measurement_has_settling_without_twinport_retries() {
    assert!(measures_player("m4b-lineages"));
    assert!(!enabled("m4b-lineages"));
    assert!(measures_player("twinport-default"));
    assert!(!measures_player("sls-conformance-smoke"));
}

#[test]
fn only_player_leg_invalid_gets_one_retry() {
    assert_eq!(attempt_budget(Some("player_leg_invalid")), 2);
    assert_eq!(attempt_budget(Some("settle_timeout")), 1);
    assert_eq!(attempt_budget(Some("execution_error")), 1);
    assert_eq!(attempt_budget(None), 2);
}

#[test]
fn twinport_manifests_pair_ports_before_advancing_index() {
    for text in [
        include_str!("../../scripts/bench/manifests/twinport-default.json"),
        include_str!("../../scripts/bench/manifests/twinport-legacy-l2.json"),
    ] {
        let manifest = crate::manifest::parse(text).unwrap();
        let work = manifest.order();
        assert_eq!(work.len(), 8);
        for pair in work.chunks_exact(2) {
            assert_eq!(pair[0].run, pair[1].run);
            assert_ne!(
                manifest.cells[pair[0].cell].port,
                manifest.cells[pair[1].cell].port
            );
        }
        assert_eq!(manifest.seed, 20260913);
        assert_eq!(
            manifest
                .cell_profile(&manifest.cells[0])
                .unwrap()
                .timeline
                .duration
                .as_secs(),
            90
        );
    }
}
