use std::collections::BTreeMap;
use std::path::PathBuf;

use network_sim::metrics::control::EffectiveConfig;
use network_sim::metrics::record::ReceiverKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
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
#[serde(deny_unknown_fields)]
pub struct Receiver {
    pub name: String,
    pub bin: PathBuf,
    pub kind: Option<ReceiverKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell {
    pub candidate: String,
    pub scenario: String,
    pub receiver: String,
    pub srt_profile: String,
    pub runs: u32,
}

impl Cell {
    pub fn id(&self) -> String {
        format!(
            "{}--{}--{}--{}",
            self.candidate, self.scenario, self.receiver, self.srt_profile
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub cell: usize,
    pub run: u32,
}
