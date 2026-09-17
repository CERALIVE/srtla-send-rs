use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Result, ensure};
use network_sim::metrics::cpu::CpuSample;
use network_sim::metrics::record::{ControlSample, Diagnostics, RunRecord};
use network_sim::metrics::stats_file::StatsFileSample;
use network_sim::metrics::{
    MetricError, control, cpu, episodes, link_counters, load_intervals, sampling,
};
use network_sim::profile::EventLog;
use network_sim::{Namespace, scenarios};

use super::clock::{self, Clock};
use super::record::{RunFailure, check_config};

pub struct Sampler<'a> {
    pub ns: &'a Namespace,
    pub stats_path: &'a Path,
    pub clock: &'a Clock,
    pub gate: &'a Mutex<Vec<String>>,
}

pub struct Sample {
    links: Vec<link_counters::LinkCounterSample>,
    stats: Option<StatsFileSample>,
}

impl Sampler<'_> {
    pub fn sample(&self) -> Result<Sample> {
        let ifaces = self.gate.lock().expect("sampling gate");
        let t = self.clock.now_ms();
        Ok(Sample {
            links: ifaces
                .iter()
                .map(|iface| link_counters::sample(Some(self.ns), iface, t))
                .collect::<Result<_>>()?,
            stats: StatsFileSample::sample(self.stats_path, t)?,
        })
    }

    pub fn new_link_baseline(&self, iface: &str, origin_ms: i64) -> Result<Sample> {
        let mut birth = link_counters::sample(Some(self.ns), iface, origin_ms)?;
        // AddLink creates this netdev after measurement start: its pre-birth counters are exactly zero.
        birth.tx_bytes = 0;
        birth.tx_packets = 0;
        Ok(Sample {
            links: vec![birth],
            stats: None,
        })
    }

    pub fn run(&self, duration: Duration) -> Result<Vec<Sample>> {
        Ok(sampling::sample_for(duration, |_| {
            self.sample()
                .map_err(|e| MetricError::InvalidField(format!("collector: {e:#}")))
        })?)
    }
}

pub struct Edges {
    pub start_cpu: CpuSample,
    pub end_cpu: CpuSample,
    pub start_control: Option<control::ControlMetrics>,
    pub end_control: Option<control::ControlMetrics>,
}

pub struct Measurement<'a> {
    pub profile: &'a scenarios::Profile,
    pub log: &'a EventLog,
    pub stats: network_sim::metrics::srt_stats::SrtStats,
    pub origin_ms: i64,
    pub sink_origin_ms: i64,
    pub samples: Vec<Sample>,
    pub edges: Edges,
}

pub fn finish(record: &mut RunRecord, measurement: Measurement<'_>) -> Result<()> {
    let m = measurement;
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    loop {
        record.raw.sink_series =
            clock::sink(&record.raw.sink_series_path, m.sink_origin_ms - m.origin_ms)?;
        if record.raw.sink_series.covers(record.window) {
            break;
        }
        ensure!(
            std::time::Instant::now() < deadline,
            RunFailure::MissingMetric
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut stats = m.stats;
    stats.clock_offset_ms -= m.origin_ms;
    for row in &mut stats.rows {
        row.t_ms -= m.origin_ms;
    }
    let window = stats.window(record.window, record.raw.sink_series.bytes(record.window)?)?;
    record.useful_goodput_bps = record.raw.sink_series.useful_goodput_bps(record.window)?;
    record.viewer_loss_ratio = window.viewer_loss_ratio;
    record.no_traffic = window.no_traffic;
    record.diagnostics = Diagnostics {
        pkt_drop_delta: Some(window.pkt_drop_total),
        pkt_belated_delta: Some(window.pkt_belated_delta),
        ms_rcv_buf_min: window.ms_rcv_buf_min,
        ms_rcv_tsbpd_delay: window.ms_rcv_tsbpd_delay,
        pkt_belated_sum: window.pkt_belated_sum,
        loss_ratio: window.loss_ratio,
        retrans_ratio: window.retrans_ratio,
        reorder_distance_max: window.reorder_distance_max,
        ms_rtt_median: window.ms_rtt_median,
        mbps_recv_rate_mean: window.mbps_recv_rate_mean,
    };
    record.raw.stats_csv = Some(stats);
    for sample in m.samples {
        record
            .raw
            .link_counters
            .extend(sample.links.into_iter().map(|mut link| {
                link.t_ms -= m.origin_ms;
                link
            }));
        if let Some(mut stats) = sample.stats {
            stats.t_ms -= m.origin_ms;
            record.sender.stats_file_samples.push(stats);
        }
    }
    record
        .raw
        .link_counters
        .sort_by(|a, b| (&a.iface, a.t_ms).cmp(&(&b.iface, b.t_ms)));
    record
        .raw
        .link_counters
        .dedup_by(|a, b| a.iface == b.iface && a.t_ms == b.t_ms);
    record.per_link = link_counters::shares(&record.raw.link_counters, record.window)?;
    let mut cpu = m
        .edges
        .end_cpu
        .delta(&m.edges.start_cpu, cpu::clock_ticks_per_second()?)?;
    cpu.start.t_ms -= m.origin_ms;
    cpu.end.t_ms -= m.origin_ms;
    record.sender.cpu_ms = cpu.cpu_ms;
    let sampled_ms = f64::from(u32::try_from(cpu.end.t_ms - cpu.start.t_ms)?);
    record.sender.cpu_percent = Some(100.0 * cpu.cpu_ms / sampled_ms);
    record.sender.peak_rss_kb = cpu.peak_rss_kb;
    for (t_ms, metrics) in [
        (cpu.start.t_ms, m.edges.start_control.as_ref()),
        (cpu.end.t_ms, m.edges.end_control.as_ref()),
    ] {
        if let Some(metrics) = metrics {
            record.raw.control_metrics.push(ControlSample {
                t_ms,
                metrics: metrics.clone(),
            });
        }
    }
    if let (Some(start), Some(end)) = (&m.edges.start_control, &m.edges.end_control) {
        check_config(start.effective_config(), end.effective_config())?;
        let delta = end.delta(start)?;
        record.sender.switch_count = Some(delta.switch_count);
        record.sender.switches_per_second =
            Some(f64::from(u32::try_from(delta.switch_count)?) * 1000.0 / sampled_ms);
        record.sender.nak_count = Some(delta.nak_count);
    }
    record.raw.cpu = Some(cpu);
    record.events = m.log.entries.clone();
    record.episodes = episodes::evaluate(&m.profile.timeline, m.log, &record.raw.sink_series)?;
    record.load_intervals = load_intervals::evaluate(
        &m.profile.timeline,
        m.log,
        (&record.raw.sink_series, m.profile.aggregate_capacity_bps()),
    )?;
    ensure!(
        !record.no_traffic && record.diagnostics.ms_rtt_median.is_some(),
        RunFailure::MissingMetric
    );
    Ok(())
}
