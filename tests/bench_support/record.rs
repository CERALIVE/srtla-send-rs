use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use network_sim::bond::CarrierMode;
use network_sim::harness::ReceiverSpec;
use network_sim::metrics::Window;
use network_sim::metrics::control::EffectiveConfig;
use network_sim::metrics::identity::{Hash256, RunId, SchemaVersion};
use network_sim::metrics::record::{
    self, Diagnostics, RawMeasurements, RunRecord, RunStatus, Sender,
};
use network_sim::metrics::sink::SinkSeries;
use serde::{Deserialize, Serialize};

use crate::manifest::{self, Manifest, Work};

#[derive(Debug, Serialize, Deserialize)]
pub struct Attempt {
    #[serde(flatten)]
    pub record: RunRecord,
    pub attempt: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub manifest: Manifest,
    pub work: Work,
    pub result: Attempt,
    pub artifacts: PathBuf,
    pub srt_binary: PathBuf,
    pub receiver_spec: ReceiverSpec,
}

#[derive(Debug)]
pub enum RunFailure {
    ConfigMismatch,
    SettleTimeout,
    RestartBudget,
    MissingMetric,
}

impl std::fmt::Display for RunFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ConfigMismatch => "config_mismatch",
            Self::SettleTimeout => "settle_timeout",
            Self::RestartBudget => "receiver_restart_budget",
            Self::MissingMetric => "missing_assertion_metric",
        })
    }
}
impl std::error::Error for RunFailure {}

pub fn check_config(
    requested: Option<&EffectiveConfig>,
    observed: Option<&EffectiveConfig>,
) -> Result<()> {
    ensure!(requested == observed, RunFailure::ConfigMismatch);
    Ok(())
}

pub fn binary_hash(path: &Path) -> Result<Hash256> {
    Ok(Hash256::digest(&std::fs::read(path).with_context(
        || format!("read binary {}", path.display()),
    )?))
}

pub fn prepare(
    manifest: &Manifest,
    work: Work,
    order_index: u32,
) -> Result<(RunRecord, ReceiverSpec)> {
    let cell = &manifest.cells[work.cell];
    let candidate = manifest
        .candidates
        .iter()
        .find(|c| c.label == cell.candidate)
        .context("candidate")?;
    let receiver = manifest
        .receivers
        .iter()
        .find(|r| r.name == cell.receiver)
        .context("receiver")?;
    let profile = manifest.scenario(&cell.scenario)?;
    let receiver = receiver.resolve(
        manifest
            .receiver_defaults
            .as_ref()
            .context("campaign receiver defaults were not resolved")?,
    )?;
    let srt = &receiver.srt_live_transmit_bin;
    let bases: Vec<_> = profile
        .timeline
        .links
        .iter()
        .map(|link| {
            (
                &link.base,
                match link.carrier {
                    CarrierMode::Direct => "direct",
                    CarrierMode::Nat => "nat",
                },
            )
        })
        .collect();
    let ramp = profile
        .source_ramp
        .map(|r| (r.at, r.duration, r.from_bps, r.to_bps));
    let scenario_bytes = serde_json::to_vec(&(
        bases,
        &profile.timeline.events,
        profile.timeline.duration,
        profile.offered_bps,
        profile.warmup_offered_bps,
        ramp,
        profile.receiver_restart_budget,
        manifest.seed,
        candidate.stats_file,
        binary_hash(srt)?,
    ))?;
    let mut env = candidate.env.clone();
    env.entry("RUST_LOG".into())
        .or_insert_with(|| "info".into());
    env.entry("PATH".into())
        .or_insert_with(|| "/usr/bin:/bin".into());
    let timestamp = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%S.%3NZ"])
        .output()?;
    ensure!(timestamp.status.success(), "UTC timestamp");
    let mut record = RunRecord {
        schema_version: SchemaVersion,
        campaign: manifest.campaign.clone(),
        cell_id: cell.cell_id.clone(),
        covering: cell.covering,
        supersedes: None,
        receiver_label: receiver.label.clone(),
        receiver_lineage: receiver.lineage.clone(),
        srtla_rec_sha256: Some(binary_hash(&receiver.srtla_rec_bin)?),
        srt_live_transmit_sha256: Some(binary_hash(srt)?),
        listener_uri_extra: receiver.listener_uri_extra.clone(),
        run_id: RunId::try_from(
            std::fs::read_to_string("/proc/sys/kernel/random/uuid")?
                .trim()
                .to_owned(),
        )?,
        scenario: record::ScenarioIdentity {
            id: cell.scenario.clone(),
            family: cell.scenario.chars().take(1).collect(),
            hash: Hash256::digest(&scenario_bytes),
        },
        candidate: record::Candidate {
            label: candidate.label.clone(),
            bin_sha256: binary_hash(&candidate.bin)?,
            args: candidate.args.clone(),
            env,
        },
        receiver: record::Receiver {
            kind: receiver.kind()?.into(),
            sha256: binary_hash(&receiver.srtla_rec_bin)?,
        },
        srt_profile: manifest::preset(&cell.srt_profile)?.into(),
        fingerprint: Hash256::digest(b"pending"),
        run_index: work.run,
        seed: manifest.seed,
        order_index,
        started_at: String::from_utf8(timestamp.stdout)?.trim().to_owned(),
        status: RunStatus::Failed,
        window: Window::new(0, i64::try_from(profile.timeline.duration.as_millis())?)?,
        useful_goodput_bps: 0.0,
        offered_bps: profile.offered_bps,
        viewer_loss_ratio: 0.0,
        no_traffic: true,
        diagnostics: Diagnostics {
            pkt_drop_delta: None,
            pkt_belated_delta: None,
            ms_rcv_buf_min: None,
            ms_rcv_tsbpd_delay: None,
            pkt_belated_sum: 0,
            loss_ratio: 0.0,
            retrans_ratio: 0.0,
            reorder_distance_max: None,
            ms_rtt_median: None,
            mbps_recv_rate_mean: None,
        },
        per_link: Vec::new(),
        sender: Sender {
            cpu_percent: None,
            switches_per_second: None,
            switch_count: None,
            nak_count: None,
            cpu_ms: 0.0,
            peak_rss_kb: 0,
            effective_config: candidate.effective_config.clone(),
            stats_file_samples: Vec::new(),
        },
        events: Vec::new(),
        episodes: Vec::new(),
        load_intervals: Vec::new(),
        loadavg_1m: loadavg()?,
        raw: RawMeasurements {
            stats_csv_path: PathBuf::new(),
            sink_series_path: PathBuf::new(),
            stats_csv: None,
            sink_series: SinkSeries::new(Vec::new())?,
            link_counters: Vec::new(),
            control_metrics: Vec::new(),
            cpu: None,
        },
        warnings: Vec::new(),
    };
    record.fingerprint = record.compute_fingerprint()?;
    record.sender.effective_config = None;
    if record.loadavg_1m > 2.0 {
        record.warnings.push("loadavg>2".into());
    }
    Ok((record, receiver))
}

pub fn loadavg() -> Result<f64> {
    let text = std::fs::read_to_string("/proc/loadavg")?;
    let value: f64 = text.split_whitespace().next().context("loadavg")?.parse()?;
    ensure!(value.is_finite() && value >= 0.0, "invalid loadavg");
    Ok(value)
}
