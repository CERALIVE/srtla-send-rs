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
    #[serde(default = "super::sls::default_sink")]
    pub sink: String,
    #[serde(default = "super::sls::default_metrics")]
    pub metrics: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sls_stats: Option<super::sls::SlsPublisherStats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sls_conformance: Option<super::sls::SlsConformance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sls_identity: Option<super::sls::SlsIdentity>,
    pub schema_version: SchemaVersion,
    pub campaign: String,
    pub cell_id: String,
    #[serde(default)]
    pub covering: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<RunId>,
    #[serde(default)]
    pub receiver_label: String,
    #[serde(default)]
    pub receiver_lineage: String,
    #[serde(default)]
    pub srtla_rec_sha256: Option<Hash256>,
    #[serde(default)]
    pub srt_live_transmit_sha256: Option<Hash256>,
    #[serde(default)]
    pub listener_uri_extra: String,
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
        let identity = (
            &self.candidate,
            &self.receiver,
            &self.scenario.hash,
            &self.srt_profile,
            &self.sender.effective_config,
        );
        let bytes = if self.receiver_label.is_empty() {
            serde_json::to_vec(&identity)?
        } else {
            serde_json::to_vec(&(
                identity,
                &self.cell_id,
                &self.receiver_label,
                &self.receiver_lineage,
                &self.listener_uri_extra,
                &self.srtla_rec_sha256,
                &self.srt_live_transmit_sha256,
            ))?
        };
        let hash = Hash256::digest(&bytes);
        match &self.sls_identity {
            Some(sls) => Ok(Hash256::digest(&serde_json::to_vec(&(
                hash,
                &self.sink,
                &self.metrics,
                sls,
            ))?)),
            None => Ok(hash),
        }
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
    #[serde(default)]
    pub pkt_drop_delta: Option<u64>,
    #[serde(default)]
    pub pkt_belated_delta: Option<u64>,
    #[serde(default)]
    pub ms_rcv_buf_min: Option<f64>,
    #[serde(default)]
    pub ms_rcv_tsbpd_delay: Option<f64>,
    pub pkt_belated_sum: u64,
    pub loss_ratio: f64,
    pub retrans_ratio: f64,
    pub reorder_distance_max: Option<u64>,
    pub ms_rtt_median: Option<f64>,
    pub mbps_recv_rate_mean: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sender {
    #[serde(default)]
    pub cpu_percent: Option<f64>,
    #[serde(default)]
    pub switches_per_second: Option<f64>,
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
