use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::number;
use super::{MetricError, Window, rate_number};
use crate::{Namespace, NamespaceProcess};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SinkBucket {
    pub t_ms: i64,
    pub duration_ms: u32,
    pub bytes: u64,
    pub pkts: u64,
}

impl SinkBucket {
    pub fn bps(&self) -> f64 {
        rate_number(self.bytes) * 8000.0 / f64::from(self.duration_ms)
    }
    pub fn start_ms(&self) -> i64 {
        self.t_ms - i64::from(self.duration_ms)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<SinkBucket>", into = "Vec<SinkBucket>")]
pub struct SinkSeries {
    buckets: Vec<SinkBucket>,
}

impl TryFrom<Vec<SinkBucket>> for SinkSeries {
    type Error = MetricError;
    fn try_from(buckets: Vec<SinkBucket>) -> Result<Self, Self::Error> {
        Self::new(buckets)
    }
}

impl From<SinkSeries> for Vec<SinkBucket> {
    fn from(series: SinkSeries) -> Self {
        series.buckets
    }
}

impl SinkSeries {
    pub fn new(buckets: Vec<SinkBucket>) -> Result<Self, MetricError> {
        for bucket in &buckets {
            if bucket.duration_ms == 0
                || bucket
                    .t_ms
                    .checked_sub(i64::from(bucket.duration_ms))
                    .is_none()
            {
                return Err(MetricError::IncompleteSeries);
            }
        }
        if buckets
            .windows(2)
            .any(|pair| pair[0].t_ms != pair[1].start_ms())
        {
            return Err(MetricError::IncompleteSeries);
        }
        Ok(Self { buckets })
    }

    pub fn parse(csv: &str, clock_offset_ms: i64) -> Result<Self, MetricError> {
        let mut lines = csv.lines();
        if lines.next() != Some("t,bytes,pkts") {
            return Err(MetricError::MissingColumn("t,bytes,pkts".into()));
        }
        let buckets = lines
            .map(|line| {
                let fields: Vec<_> = line.split(',').collect();
                if fields.len() != 3 {
                    return Err(MetricError::InvalidField("sink row".into()));
                }
                Ok(SinkBucket {
                    t_ms: number::<i64>(fields[0], "t")?
                        .checked_add(clock_offset_ms)
                        .ok_or(MetricError::InvalidWindow)?,
                    duration_ms: 100,
                    bytes: number(fields[1], "bytes")?,
                    pkts: number(fields[2], "pkts")?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::new(buckets)
    }

    pub fn buckets(&self) -> &[SinkBucket] {
        &self.buckets
    }

    pub fn covers(&self, window: Window) -> bool {
        self.buckets
            .first()
            .is_some_and(|b| b.start_ms() <= window.start_ms())
            && self
                .buckets
                .last()
                .is_some_and(|b| b.t_ms >= window.end_ms())
    }

    pub fn bytes(&self, window: Window) -> Result<u64, MetricError> {
        if !self.covers(window) {
            return Err(MetricError::IncompleteSeries);
        }
        self.buckets
            .iter()
            .filter(|b| window.contains(b.t_ms))
            .try_fold(0_u64, |sum, b| sum.checked_add(b.bytes))
            .ok_or_else(|| MetricError::InvalidField("sink bytes sum".into()))
    }

    pub fn useful_goodput_bps(&self, window: Window) -> Result<f64, MetricError> {
        Ok(rate_number(self.bytes(window)?) * 8000.0
            / rate_number(window.duration_ms().unsigned_abs()))
    }

    /// Complete one-second buckets, aligned to the run clock, never to CSV row indices.
    pub fn seconds(&self, window: Window) -> Result<Vec<SinkBucket>, MetricError> {
        let mut result = Vec::new();
        let mut start = window.start_ms();
        let remainder = start.rem_euclid(1000);
        if remainder > 0 {
            start = start
                .checked_add(1000 - remainder)
                .ok_or(MetricError::InvalidWindow)?;
        }
        while let Some(end) = start
            .checked_add(1000)
            .filter(|end| *end <= window.end_ms())
        {
            let part = Window::new(start, end)?;
            if self.covers(part) {
                result.push(SinkBucket {
                    t_ms: end,
                    duration_ms: 1000,
                    bytes: self.bytes(part)?,
                    pkts: 0,
                });
            }
            start = end;
        }
        Ok(result)
    }
}

/// The ready line carries the bound port and absolute monotonic origin in ns.
/// Calibrate that origin against the event clock, then pass its offset to `parse`.
pub fn spawn(ns: &Namespace, port: u16, output: &Path) -> anyhow::Result<NamespaceProcess> {
    use anyhow::Context;
    NamespaceProcess::spawn_process_only(
        ns,
        "python3",
        &[
            "-u",
            "-c",
            PROGRAM,
            &port.to_string(),
            output.to_str().context("sink path must be UTF-8")?,
            "0",
        ],
    )
}

pub const PROGRAM: &str = r#"
import select
import socket
import sys
import time

port = int(sys.argv[1])
path = sys.argv[2]
limit = int(sys.argv[3])
with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock, open(path, 'w', buffering=1) as output:
    sock.bind(('127.0.0.1', port))
    sock.setblocking(False)
    origin = time.monotonic_ns()
    print(f'ready,{sock.getsockname()[1]},{origin}', flush=True)
    output.write('t,bytes,pkts\n')
    bucket = 1
    received_bytes = 0
    received_packets = 0
    while limit == 0 or bucket <= limit:
        deadline = origin + bucket * 100_000_000
        remaining = (deadline - time.monotonic_ns()) / 1_000_000_000
        if remaining <= 0:
            output.write(f'{bucket * 100},{received_bytes},{received_packets}\n')
            received_bytes = 0
            received_packets = 0
            bucket += 1
            continue
        readable, _, _ = select.select([sock], [], [], remaining)
        if readable:
            payload = sock.recv(65535)
            received_bytes += len(payload)
            received_packets += 1
"#;
