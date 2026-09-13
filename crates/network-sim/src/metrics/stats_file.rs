use std::path::Path;

use serde::{Deserialize, Serialize};

use super::MetricError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionSample {
    pub conn_id: String,
    pub weight_percent: u8,
    pub window: i32,
    pub in_flight: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatsDocument {
    pub schema_version: u32,
    pub last_updated_ms: u64,
    pub connections: Vec<ConnectionSample>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatsFileSample {
    pub t_ms: i64,
    pub document: StatsDocument,
}

impl StatsFileSample {
    pub fn parse(t_ms: i64, json: &str) -> Result<Self, MetricError> {
        let document: StatsDocument = serde_json::from_str(json)?;
        if document.schema_version != 1 {
            return Err(MetricError::InvalidField("telemetry schema_version".into()));
        }
        for conn in &document.connections {
            if conn.weight_percent > 100 || conn.priority.is_some_and(|p| !p.is_finite()) {
                return Err(MetricError::InvalidField(
                    "telemetry weight or priority".into(),
                ));
            }
        }
        Ok(Self { t_ms, document })
    }

    /// Invoke on the collector's 1 Hz clock. Missing opt-in telemetry is not a zero snapshot.
    pub fn sample(path: &Path, t_ms: i64) -> Result<Option<Self>, MetricError> {
        match std::fs::read_to_string(path) {
            Ok(json) => Self::parse(t_ms, &json).map(Some),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
