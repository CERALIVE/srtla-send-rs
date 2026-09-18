use std::time::Duration;

use anyhow::{Context, Result, ensure};
use network_sim::metrics::sink::{SinkBucket, SinkSeries};
use network_sim::metrics::sls::{ConformanceGate, SlsConformance};
use network_sim::metrics::{Window, control};
use network_sim::profile::{BondRuntime, ProcessEndpoints, Scheduler};
use network_sim::scenarios::Profile;

use super::record::{Request, check_config};
use super::stack::Stack;

pub fn measure(request: &mut Request, profile: &Profile, stack: &mut Stack) -> Result<()> {
    let cell = &request.manifest.cells[request.work.cell];
    let candidate = request
        .manifest
        .candidates
        .iter()
        .find(|c| c.label == cell.candidate)
        .context("conformance candidate")?;
    if let Some(expected) = &candidate.effective_config {
        let control = control::query(&stack.control, Duration::from_secs(1))?;
        let observed = control
            .as_ref()
            .and_then(control::ControlMetrics::effective_config);
        check_config(Some(expected), observed)?;
        request.result.record.sender.effective_config = observed.cloned();
    }
    let capture = stack
        .sls_capture
        .as_mut()
        .context("SLS capture not launched")?;
    stack
        .source
        .rate(profile.warmup_offered_bps, Duration::ZERO)?;
    capture.attach_player(&stack.topo.receiver_ns, &request.srt_binary)?;
    let twinport_start = if super::twinport::measures_player(&request.manifest.campaign) {
        Some(super::twinport::begin(request, profile, stack)?)
    } else {
        None
    };
    let capture = stack.sls_capture.as_ref().context("SLS capture")?;
    let offered_start = stack.source.offered_bytes()?;
    let measurement = capture.begin_measurement()?;
    let mut scheduler = Scheduler::new(&profile.timeline)?;
    {
        let source = &mut stack.source;
        let ramp = std::cell::Cell::new(Duration::ZERO);
        let mut offered_rate = |bps| source.rate(bps, ramp.get());
        let mut runtime = BondRuntime::with_link_additions(
            &profile.timeline,
            &mut stack.topo,
            ProcessEndpoints {
                sender: &mut stack.sender,
                receiver: &mut stack.receiver,
                offered_rate: &mut offered_rate,
            },
        )?;
        scheduler.run(&mut |event| {
            ramp.set(
                profile
                    .source_ramp
                    .filter(|r| r.at == event.at)
                    .map_or(Duration::ZERO, |r| r.duration),
            );
            runtime.apply(event)
        })?;
    }
    request.result.record.events = scheduler.log().entries.clone();
    let offered_bytes = stack
        .source
        .offered_bytes()?
        .checked_sub(offered_start)
        .context("offered-byte capture truncated")?;
    let preset = crate::manifest::preset(&cell.srt_profile)?;
    let observed = capture.finish_measurement(
        (&stack.topo.receiver_ns, &stack.listener),
        measurement,
        (offered_bytes, preset.latency_ms),
    )?;
    let record = &mut request.result.record;
    record.window = Window::new(0, i64::from(observed.duration_ms))?;
    record.raw.sink_series = SinkSeries::new(vec![SinkBucket {
        t_ms: i64::from(observed.duration_ms),
        duration_ms: observed.duration_ms,
        bytes: observed.player_bytes,
        pkts: 0,
    }])?;
    record.useful_goodput_bps = record.raw.sink_series.useful_goodput_bps(record.window)?;
    record.no_traffic = observed.player_bytes == 0;
    record.sls_stats = observed.sls_stats;
    record.sls_conformance = Some(SlsConformance {
        assertions: observed.assertions.clone(),
        offered_bytes,
        player_bytes: observed.player_bytes,
        duration_ms: observed.duration_ms,
        device_preset_ms: preset.latency_ms,
        belated: None,
        occupancy_pct: None,
        belated_gate: ConformanceGate::NotApplicable,
        occupancy_gate: ConformanceGate::NotApplicable,
    });
    record.raw.stats_csv_path = request.artifacts.join("sls-stats.json");
    record.raw.sink_series_path = request.artifacts.join("player-series.json");
    crate::checkpoint::atomic(&record.raw.stats_csv_path, &record.sls_stats)?;
    crate::checkpoint::atomic(&record.raw.sink_series_path, &record.raw.sink_series)?;
    let state = |pass| if pass { "ok" } else { "FAILED" };
    eprintln!(
        "port={} registered={} carry={} latency={}",
        cell.port,
        state(observed.assertions.registered),
        state(observed.assertions.carry),
        state(observed.assertions.latency)
    );
    if let Some(start) = twinport_start {
        super::twinport::finish(request, profile, stack, start)?;
    }
    observed.assertions.require_pass()
}

pub fn smoke() -> Result<()> {
    if std::env::var_os("SLS_BIN").is_none() {
        eprintln!("SKIP conformance_smoke: SLS_BIN is unset");
        return Ok(());
    }
    if !network_sim::check_privileges()
        || !std::process::Command::new("sudo")
            .args(["-n", "unshare", "--net", "--", "ip", "link", "show", "lo"])
            .output()
            .is_ok_and(|output| output.status.success())
    {
        eprintln!("SKIP conformance_smoke: privileged netns unavailable");
        return Ok(());
    }
    let _guard = crate::measurement::measurement_lock();
    let mut manifest = crate::manifest::parse(&serde_json::json!({
        "campaign":"sls-conformance-smoke", "seed":20260917,
        "candidates":[{"label":"enhanced", "bin":env!("CARGO_BIN_EXE_srtla_send"), "args":["--mode","enhanced"]}],
        "receivers":[{"label":"ours-new", "lineage":"ours-new", "kind":"ceralive",
            "bin":"lock:ours-new", "srt_live_transmit_bin":"lock:ours-new"}],
        "cells":[
            {"candidate":"enhanced","scenario":"SLS","receiver":"ours-new","srt_profile":"strict",
             "runs":1,"sink":"sls","metrics":"none","covering":false,"port":4002},
            {"candidate":"enhanced","scenario":"SLS","receiver":"ours-new","srt_profile":"strict",
             "runs":1,"sink":"sls","metrics":"none","covering":false,"port":4003}
        ]
    }).to_string())?;
    manifest.receiver_defaults = Some(network_sim::harness::ReceiverSpec::from_env()?);
    let directory = tempfile::Builder::new()
        .prefix("sls-conformance-smoke-")
        .tempdir()?
        .keep();
    std::fs::create_dir(directory.join("artifacts"))?;
    std::fs::create_dir(directory.join("results"))?;
    crate::checkpoint::atomic(&directory.join("manifest.json"), &manifest)?;
    eprintln!("conformance_smoke artifacts={}", directory.display());
    for work in manifest.order() {
        let (record, receiver_spec) = super::record::prepare(&manifest, work, work.run)?;
        let artifacts = directory
            .join("artifacts")
            .join(format!("port-{}", manifest.cells[work.cell].port));
        std::fs::create_dir(&artifacts)?;
        let result = super::bounded::execute(Request {
            manifest: manifest.clone(),
            work,
            result: super::record::Attempt {
                record,
                attempt: 1,
                reason: None,
                detail: None,
            },
            artifacts,
            srt_binary: receiver_spec.srt_live_transmit_bin.clone(),
            receiver_spec,
        })?;
        eprintln!("{}", serde_json::to_string(&result.record)?);
        let result_dir = directory
            .join("results")
            .join(manifest.cells[work.cell].id());
        std::fs::create_dir(&result_dir)?;
        crate::checkpoint::atomic(&result_dir.join("run-0.json"), &result)?;
        ensure!(
            result.record.status == network_sim::metrics::record::RunStatus::Ok,
            "SLS cell failed: {:?} {:?}",
            result.reason,
            result.detail
        );
        let checks = result
            .record
            .sls_conformance
            .as_ref()
            .context("missing conformance assertions")?;
        let state = |pass| if pass { "ok" } else { "FAILED" };
        eprintln!(
            "port={} registered={} carry={} latency={}",
            manifest.cells[work.cell].port,
            state(checks.assertions.registered),
            state(checks.assertions.carry),
            state(checks.assertions.latency)
        );
        ensure!(
            result.record.window.duration_ms() >= 20_000,
            "incomplete 20s conformance window"
        );
        ensure!(
            result.record.sink == "sls" && result.record.useful_goodput_bps.is_finite(),
            "SLS RunRecord"
        );
        checks.assertions.require_pass()?;
    }
    Ok(())
}
