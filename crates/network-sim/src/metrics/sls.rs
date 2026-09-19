use serde::{Deserialize, Serialize};

use super::identity::Hash256;
use crate::harness::SlsAssertions;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsPlayerStats {
    pub client_id: String,
    pub latency: Option<u32>,
    pub mbps_send_rate: Option<f64>,
    pub pkt_retrans_total: Option<u64>,
    pub pkt_snd_drop_total: Option<u64>,
    pub rtt: Option<f64>,
    pub uptime: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioGapTrack {
    pub channels: u32,
    pub format_detected: bool,
    pub gap_count: u64,
    pub last_gap_frames: u64,
    pub last_gap_pts_delta: i64,
    pub pid: u32,
    pub sample_rate: u32,
    pub silent_bytes_inserted: u64,
    pub silent_frames_inserted: u64,
    pub silent_packets_inserted: u64,
    pub stream_id: u32,
    pub stream_type: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioGapFill {
    pub audio_track_count: u32,
    pub enabled: bool,
    pub gap_count: u64,
    pub pmt_parsed: bool,
    pub silent_bytes_inserted: u64,
    pub silent_frames_inserted: u64,
    pub silent_packets_inserted: u64,
    pub tracks: Vec<AudioGapTrack>,
}

/// Native publisher snapshots; these are never covering-set measurements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlsPublisherStats {
    pub audio_gap_fill: Option<AudioGapFill>,
    pub bitrate: Option<u64>,
    pub bytes_rcv_drop: Option<u64>,
    pub bytes_rcv_loss: Option<u64>,
    pub ingest_discontinuities: Option<i64>,
    pub latency: Option<u32>,
    pub max_reader_backlog_bytes: Option<i64>,
    pub max_reader_backlog_ms: Option<u64>,
    pub mbps_bandwidth: Option<f64>,
    pub mbps_recv_rate: Option<f64>,
    pub ms_rcv_buf: Option<u64>,
    pub pkt_rcv_drop: Option<u64>,
    pub pkt_rcv_loss: Option<u64>,
    pub pkt_rcv_retrans: Option<u64>,
    #[serde(rename = "pktRecvNAKTotal")]
    pub pkt_recv_nak_total: Option<u64>,
    pub pkt_retrans_total: Option<u64>,
    #[serde(rename = "pktSentNAKTotal")]
    pub pkt_sent_nak_total: Option<u64>,
    pub players: Vec<SlsPlayerStats>,
    pub ring_overruns: Option<i64>,
    pub rtt: Option<f64>,
    pub send_backpressure: Option<i64>,
    pub uptime: Option<u64>,
    pub viewer_pkt_snd_drop: Option<i64>,
    #[serde(rename = "msSrtlaReorderHold")]
    pub reorder_hold_ms: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlsIdentity {
    pub binary_sha256: Hash256,
    pub template_sha256: Hash256,
    pub libsrt_sha256: Hash256,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceGate {
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlsConformance {
    pub assertions: SlsAssertions,
    pub offered_bytes: u64,
    pub player_bytes: u64,
    pub duration_ms: u32,
    pub device_preset_ms: u32,
    pub belated: Option<u64>,
    pub occupancy_pct: Option<f64>,
    pub belated_gate: ConformanceGate,
    pub occupancy_gate: ConformanceGate,
}

pub fn default_sink() -> String {
    "slt".into()
}

pub fn default_metrics() -> String {
    "full".into()
}
