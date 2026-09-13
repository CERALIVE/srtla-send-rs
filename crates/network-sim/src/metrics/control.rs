use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::MetricError;
use super::error::delta;

/// Opaque process-owned JSON: Todo 27 owns these schemas; collectors must not guess their keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectiveConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptive_features: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adaptive_tuning: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlMetrics {
    pub switch_count: u64,
    pub nak_count: u64,
    pub cooldown_hold_count: u64,
    #[serde(flatten)]
    pub config: EffectiveConfig,
}

impl ControlMetrics {
    pub fn effective_config(&self) -> Option<&EffectiveConfig> {
        (self.config.adaptive_features.is_some() || self.config.adaptive_tuning.is_some())
            .then_some(&self.config)
    }

    pub fn delta(&self, start: &Self) -> Result<Self, MetricError> {
        Ok(Self {
            switch_count: delta(self.switch_count, start.switch_count, "switch_count")?,
            nak_count: delta(self.nak_count, start.nak_count, "nak_count")?,
            cooldown_hold_count: delta(
                self.cooldown_hold_count,
                start.cooldown_hold_count,
                "cooldown_hold_count",
            )?,
            config: self.config.clone(),
        })
    }
}

pub fn parse_reply(reply: &str) -> Result<Option<ControlMetrics>, MetricError> {
    #[derive(Deserialize)]
    struct Probe {
        switch_count: Option<u64>,
        nak_count: Option<u64>,
        cooldown_hold_count: Option<u64>,
    }
    let reply = reply.trim();
    if reply.is_empty() || reply.to_ascii_lowercase().starts_with("unknown command") {
        return Ok(None);
    }
    let probe: Probe = serde_json::from_str(reply)?;
    if probe.switch_count.is_none()
        && probe.nak_count.is_none()
        && probe.cooldown_hold_count.is_none()
    {
        return Ok(None);
    }
    Ok(Some(serde_json::from_str(reply)?))
}

#[cfg(unix)]
pub fn query(path: &Path, timeout: Duration) -> Result<Option<ControlMetrics>, MetricError> {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::net::UnixStream;
    let mut stream = match UnixStream::connect(path) {
        Ok(stream) => stream,
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Ok(None);
        }
        Err(e) => return Err(e.into()),
    };
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    stream.write_all(b"metrics\n")?;
    let mut reply = String::new();
    match BufReader::new(stream.take(65537)).read_line(&mut reply) {
        Ok(_) if reply.len() > 65536 => {
            return Err(MetricError::InvalidField("metrics reply size".into()));
        }
        Ok(_) => {}
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
            ) && reply.is_empty() =>
        {
            return Ok(None);
        }
        Err(e) => return Err(e.into()),
    }
    parse_reply(&reply)
}

#[cfg(not(unix))]
pub fn query(_path: &Path, _timeout: Duration) -> Result<Option<ControlMetrics>, MetricError> {
    Ok(None)
}
