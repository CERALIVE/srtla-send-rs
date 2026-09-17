use std::collections::BTreeMap;
use std::path::PathBuf;

use network_sim::metrics::control::EffectiveConfig;
use network_sim::metrics::record::ReceiverKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver_defaults: Option<network_sim::harness::ReceiverSpec>,
    pub campaign: String,
    pub candidates: Vec<Candidate>,
    pub receivers: Vec<Receiver>,
    pub cells: Vec<Cell>,
    pub seed: u64,
    pub window_secs_override: Option<u64>,
    #[serde(default)]
    pub scenarios: Vec<String>,
    #[serde(default)]
    pub srt_profiles: Vec<String>,
    pub runs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub label: String,
    pub bin: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub effective_config: Option<EffectiveConfig>,
    #[serde(default)]
    pub stats_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ReceiverInput")]
pub struct Receiver {
    pub name: String,
    pub bin: Option<PathBuf>,
    pub kind: Option<ReceiverKind>,
    pub lineage: Option<String>,
    pub srt_live_transmit_bin: Option<PathBuf>,
    pub listener_uri_extra: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ReceiverInput {
    Legacy(String),
    Object(ReceiverObject),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiverObject {
    #[serde(alias = "label")]
    name: String,
    #[serde(alias = "srtla_rec_bin")]
    bin: Option<PathBuf>,
    #[serde(alias = "srtla_rec_kind")]
    kind: Option<ReceiverKind>,
    lineage: Option<String>,
    srt_live_transmit_bin: Option<PathBuf>,
    #[serde(default)]
    listener_uri_extra: String,
}

impl From<ReceiverInput> for Receiver {
    fn from(input: ReceiverInput) -> Self {
        let object = match input {
            ReceiverInput::Legacy(name) => ReceiverObject {
                name,
                bin: None,
                kind: None,
                lineage: None,
                srt_live_transmit_bin: None,
                listener_uri_extra: String::new(),
            },
            ReceiverInput::Object(object) => object,
        };
        Self {
            name: object.name,
            bin: object.bin,
            kind: object.kind,
            lineage: object.lineage,
            srt_live_transmit_bin: object.srt_live_transmit_bin,
            listener_uri_extra: object.listener_uri_extra,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority_sidecar: Option<network_sim::bond::PrioritySidecar>,
    #[serde(default)]
    pub cell_id: String,
    #[serde(default = "covering_default")]
    pub covering: bool,
    #[serde(default)]
    pub variant: String,
    #[serde(default = "sink_default")]
    pub sink: String,
    #[serde(default = "network_sim::metrics::sls::default_metrics")]
    pub metrics: String,
    #[serde(default = "port_default")]
    pub port: u16,
    #[serde(default)]
    pub fec: bool,
    pub candidate: String,
    pub scenario: String,
    pub receiver: String,
    pub srt_profile: String,
    pub runs: u32,
}

impl Cell {
    pub fn id(&self) -> String {
        format!("{}--{}", self.cell_id, self.candidate)
    }
}

pub const fn covering_default() -> bool {
    true
}
pub fn sink_default() -> String {
    "slt".into()
}
pub const fn port_default() -> u16 {
    4001
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub cell: usize,
    pub run: u32,
}
