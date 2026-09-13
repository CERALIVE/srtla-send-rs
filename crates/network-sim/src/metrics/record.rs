use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Window;
use super::control::{ControlMetrics, EffectiveConfig};
use super::cpu::CpuWindow;
use super::episodes::Episode;
use super::identity::{Hash256, RunId, SchemaVersion};
use super::link_counters::{LinkCounterSample, LinkShare};
use super::load_intervals::LoadInterval;
use super::sink::SinkSeries;
use super::srt_stats::SrtStats;
use super::stats_file::StatsFileSample;
use crate::harness::SrtProfile;
use crate::profile::EventRecord;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunRecord {
    pub schema_version: SchemaVersion,
    pub campaign: String,
    pub cell_id: String,
    pub run_id: RunId,
    pub scenario: ScenarioIdentity,
    pub candidate: Candidate,
    pub receiver: Receiver,
    pub srt_profile: RecordedSrtProfile,
    pub fingerprint: Hash256,
    pub run_index: u32,
    pub seed: u64,
    pub order_index: u32,
    pub started_at: String,
    pub status: RunStatus,
    pub window: Window,
    pub useful_goodput_bps: f64,
    pub offered_bps: u64,
    pub viewer_loss_ratio: f64,
    pub no_traffic: bool,
    pub diagnostics: Diagnostics,
    pub per_link: Vec<LinkShare>,
    pub sender: Sender,
    pub events: Vec<EventRecord>,
    pub episodes: Vec<Episode>,
    pub load_intervals: Vec<LoadInterval>,
    pub loadavg_1m: f64,
    pub raw: RawMeasurements,
    pub warnings: Vec<String>,
}

impl RunRecord {
    /// SHA256 of a JSON tuple prevents ambiguous concatenation; env and opaque JSON maps are sorted.
    pub fn compute_fingerprint(&self) -> Result<Hash256, serde_json::Error> {
        let bytes = serde_json::to_vec(&(
            &self.candidate,
            &self.receiver,
            &self.scenario.hash,
            &self.srt_profile,
            &self.sender.effective_config,
        ))?;
        Ok(Hash256::digest(&bytes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioIdentity {
    pub id: String,
    pub family: String,
    pub hash: Hash256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub label: String,
    pub bin_sha256: Hash256,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReceiverKind {
    Ceralive,
    Irlserver,
    Belabox,
}

impl From<crate::harness::ReceiverKind> for ReceiverKind {
    fn from(kind: crate::harness::ReceiverKind) -> Self {
        match kind {
            crate::harness::ReceiverKind::CeraLive => Self::Ceralive,
            crate::harness::ReceiverKind::Irlserver => Self::Irlserver,
            crate::harness::ReceiverKind::Belabox => Self::Belabox,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Receiver {
    pub kind: ReceiverKind,
    pub sha256: Hash256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedSrtProfile {
    pub latency_ms: u32,
    pub lossmaxttl: u32,
    pub name: String,
}

impl From<SrtProfile> for RecordedSrtProfile {
    fn from(profile: SrtProfile) -> Self {
        Self {
            latency_ms: profile.latency_ms,
            lossmaxttl: profile.lossmaxttl,
            name: profile.name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Ok,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostics {
    pub pkt_belated_sum: u64,
    pub loss_ratio: f64,
    pub retrans_ratio: f64,
    pub reorder_distance_max: Option<u64>,
    pub ms_rtt_median: Option<f64>,
    pub mbps_recv_rate_mean: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sender {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switch_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nak_count: Option<u64>,
    pub cpu_ms: f64,
    pub peak_rss_kb: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_config: Option<EffectiveConfig>,
    pub stats_file_samples: Vec<StatsFileSample>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlSample {
    pub t_ms: i64,
    pub metrics: ControlMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawMeasurements {
    pub stats_csv_path: PathBuf,
    pub sink_series_path: PathBuf,
    pub stats_csv: Option<SrtStats>,
    pub sink_series: SinkSeries,
    pub link_counters: Vec<LinkCounterSample>,
    pub control_metrics: Vec<ControlSample>,
    pub cpu: Option<CpuWindow>,
}
