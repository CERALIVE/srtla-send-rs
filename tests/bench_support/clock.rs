use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::NamespaceProcess;
use network_sim::metrics::sink::SinkSeries;
use network_sim::metrics::srt_stats::{CaptureSemantics, SrtStats};

pub struct Clock {
    instant: Instant,
    mono_ms: i64,
}

impl Clock {
    pub fn calibrate() -> Result<Self> {
        let mut child = Command::new("python3")
            .args(["-u", "-c", "import time; print(time.monotonic_ns())"])
            .stdout(Stdio::piped())
            .spawn()?;
        let mut line = String::new();
        BufReader::new(child.stdout.take().context("clock pipe")?).read_line(&mut line)?;
        let instant = Instant::now();
        ensure!(child.wait()?.success(), "clock calibration failed");
        Ok(Self {
            instant,
            mono_ms: line.trim().parse::<i64>()? / 1_000_000,
        })
    }

    pub fn now_ms(&self) -> i64 {
        self.mono_ms
            + i64::try_from(self.instant.elapsed().as_millis()).expect("elapsed ms fits i64")
    }
}

pub fn wait_log(process: &NamespaceProcess, prefix: &str, timeout: Duration) -> Result<String> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(line) = process
            .log_snapshot()
            .into_iter()
            .find(|l| l.starts_with(prefix))
        {
            return Ok(line);
        }
        ensure!(
            Instant::now() < deadline,
            "readiness timeout: {prefix}; {:?}",
            process.log_snapshot()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub fn complete_csv(path: &Path) -> Result<String> {
    let mut text = std::fs::read_to_string(path)?;
    text.truncate(text.rfind('\n').map_or(0, |last| last + 1));
    Ok(text)
}

pub fn sink(path: &Path, offset: i64) -> Result<SinkSeries> {
    Ok(SinkSeries::parse(&complete_csv(path)?, offset)?)
}

pub struct CsvClock {
    pub offset_ms: i64,
    pub uncertainty_ms: i64,
}

impl CsvClock {
    // Bracket the first flushed row during warm-up. Time is socket-relative, not row-index time.
    pub fn observe(path: &Path, clock: &Clock) -> Result<Self> {
        let deadline = Instant::now() + network_sim::scenarios::WARMUP_TIMEOUT;
        let mut before = clock.now_ms();
        loop {
            let text = match complete_csv(path) {
                Ok(text) => text,
                Err(e)
                    if e.downcast_ref::<std::io::Error>()
                        .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound) =>
                {
                    String::new()
                }
                Err(e) => return Err(e),
            };
            let after = clock.now_ms();
            if text.lines().count() >= 2 {
                let stats = SrtStats::parse_with_semantics(&text, 0, CaptureSemantics::Interval)?;
                let row = stats.rows.last().context("first CSV row")?;
                return Ok(Self {
                    offset_ms: before + (after - before) / 2 - row.t_ms,
                    uncertainty_ms: after - before,
                });
            }
            before = after;
            ensure!(
                Instant::now() < deadline,
                super::record::RunFailure::SettleTimeout
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
