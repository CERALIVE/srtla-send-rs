use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use super::error::{delta, number};
use super::{MetricError, rate_number};

pub struct ProcText<'a> {
    pub stat: &'a str,
    pub status: &'a str,
    pub loadavg: &'a str,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuSample {
    pub t_ms: i64,
    pub pid: u32,
    pub start_ticks: u64,
    pub cpu_ticks: u64,
    pub peak_rss_kb: u64,
    pub loadavg_1m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuWindow {
    pub cpu_ms: f64,
    pub peak_rss_kb: u64,
    pub ticks_per_second: NonZeroU64,
    pub start: CpuSample,
    pub end: CpuSample,
}

impl CpuSample {
    pub fn parse(t_ms: i64, files: ProcText<'_>) -> Result<Self, MetricError> {
        let (identity, tail) = files
            .stat
            .rsplit_once(") ")
            .ok_or_else(|| MetricError::InvalidField("proc stat comm".into()))?;
        let (pid, _) = identity
            .split_once(" (")
            .ok_or_else(|| MetricError::InvalidField("proc stat pid".into()))?;
        let fields: Vec<_> = tail.split_whitespace().collect();
        let count = |idx: usize, name: &str| {
            number::<u64>(
                fields
                    .get(idx)
                    .ok_or_else(|| MetricError::InvalidField(name.into()))?,
                name,
            )
        };
        let cpu_ticks = count(11, "utime")?
            .checked_add(count(12, "stime")?)
            .ok_or_else(|| MetricError::InvalidField("cpu ticks overflow".into()))?;
        let rss = files
            .status
            .lines()
            .find_map(|line| line.strip_prefix("VmHWM:"))
            .ok_or_else(|| MetricError::InvalidField("VmHWM".into()))?;
        let rss = rss
            .trim()
            .strip_suffix("kB")
            .ok_or_else(|| MetricError::InvalidField("VmHWM unit".into()))?;
        let loadavg_1m: f64 = number(
            files.loadavg.split_whitespace().next().unwrap_or(""),
            "loadavg",
        )?;
        if !loadavg_1m.is_finite() || loadavg_1m < 0.0 {
            return Err(MetricError::InvalidField("loadavg".into()));
        }
        Ok(Self {
            t_ms,
            pid: number(pid, "pid")?,
            start_ticks: count(19, "starttime")?,
            cpu_ticks,
            peak_rss_kb: number(rss, "VmHWM")?,
            loadavg_1m,
        })
    }

    pub fn sample(pid: u32, t_ms: i64) -> Result<Self, MetricError> {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
        let status = std::fs::read_to_string(format!("/proc/{pid}/status"))?;
        let loadavg = std::fs::read_to_string("/proc/loadavg")?;
        Self::parse(
            t_ms,
            ProcText {
                stat: &stat,
                status: &status,
                loadavg: &loadavg,
            },
        )
    }

    pub fn delta(
        &self,
        start: &Self,
        ticks_per_second: NonZeroU64,
    ) -> Result<CpuWindow, MetricError> {
        if self.pid != start.pid || self.start_ticks != start.start_ticks {
            return Err(MetricError::CounterReset("process generation".into()));
        }
        if self.t_ms <= start.t_ms {
            return Err(MetricError::InvalidWindow);
        }
        Ok(CpuWindow {
            cpu_ms: rate_number(delta(self.cpu_ticks, start.cpu_ticks, "cpu_ticks")?) * 1000.0
                / rate_number(ticks_per_second.get()),
            peak_rss_kb: self.peak_rss_kb.max(start.peak_rss_kb),
            ticks_per_second,
            start: start.clone(),
            end: self.clone(),
        })
    }
}

pub fn clock_ticks_per_second() -> Result<NonZeroU64, MetricError> {
    let output = std::process::Command::new("getconf")
        .arg("CLK_TCK")
        .output()?;
    if !output.status.success() {
        return Err(MetricError::InvalidField("getconf CLK_TCK".into()));
    }
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|_| MetricError::InvalidField("CLK_TCK encoding".into()))?;
    NonZeroU64::new(number(text, "CLK_TCK")?)
        .ok_or_else(|| MetricError::InvalidField("CLK_TCK".into()))
}
