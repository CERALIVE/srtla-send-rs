use std::cell::Cell;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::metrics::cpu::CpuSample;
use network_sim::metrics::{Window, control};
use network_sim::profile::{Action, BondRuntime, ProcessEndpoints, Scheduler};
use network_sim::scenarios::{self, Profile};

use super::clock::{self, Clock, CsvClock};
use super::collect::{self, Edges, Measurement, Sampler};
use super::csv_capture::Capture;
use super::record::{Request, RunFailure, check_config};
use super::stack::Stack;
use crate::manifest::{Settle, measurement_rate};

pub fn execute(request: &mut Request) -> Result<()> {
    let profile = request
        .manifest
        .scenario(&request.manifest.cells[request.work.cell].scenario)?;
    let mut stack = Stack::start(request, &profile)?;
    let result = match request.result.record.sink.as_str() {
        "sls" => super::conformance::measure(request, &profile, &mut stack),
        "slt" => measure(request, &profile, &mut stack),
        _ => anyhow::bail!("unsupported sink"),
    };
    stack.logs(&request.artifacts)?;
    stack.finish_pcaps(result.is_err())?;
    result
}

fn settle(
    profile: &Profile,
    timing: (&Clock, i64),
    capture: (&std::path::Path, i64),
) -> Result<()> {
    let (clock, started) = timing;
    let (path, sink_origin_ms) = capture;
    let mut settle = Settle::for_profile(profile);
    let mut second = 1_u64;
    loop {
        ensure!(
            clock.now_ms() - started < i64::try_from(scenarios::WARMUP_TIMEOUT.as_millis())?,
            RunFailure::SettleTimeout
        );
        let sink = clock::sink(path, sink_origin_ms)?;
        let end = started + i64::try_from(second)? * 1000;
        let window = Window::new(end - 1000, end)?;
        if sink.covers(window) {
            let ready = settle.observe(second, sink.useful_goodput_bps(window)?);
            if ready && second >= 10 {
                return Ok(());
            }
            second += 1;
        } else {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn measure(request: &mut Request, profile: &Profile, stack: &mut Stack) -> Result<()> {
    let clock = &Clock::calibrate()?;
    let ramp_duration = Cell::new(Duration::ZERO);
    let source = &mut stack.source;
    let mut offered_rate = |bps| source.rate(bps, ramp_duration.get());
    let sender_ns = std::sync::Arc::clone(&stack.topo.sender_ns);
    let mut runtime = BondRuntime::with_link_additions(
        &profile.timeline,
        &mut stack.topo,
        ProcessEndpoints {
            sender: &mut stack.sender,
            receiver: &mut stack.receiver,
            offered_rate: &mut offered_rate,
        },
    )?;
    let warmup_start = clock.now_ms();
    (runtime.processes.offered_rate)(profile.warmup_offered_bps)?;
    let csv_clock = CsvClock::observe(&request.result.record.raw.stats_csv_path, clock)?;
    settle(
        profile,
        (clock, warmup_start),
        (
            &request.result.record.raw.sink_series_path,
            stack.sink_origin_ms,
        ),
    )?;
    let requested = request
        .manifest
        .candidates
        .iter()
        .find(|c| c.label == request.result.record.candidate.label)
        .context("candidate")?;
    let preflight_control = control::query(&stack.control, Duration::from_secs(1))?;
    let observed = preflight_control
        .as_ref()
        .and_then(control::ControlMetrics::effective_config);
    request.result.record.sender.effective_config = observed.cloned();
    check_config(requested.effective_config.as_ref(), observed)?;
    let pid = runtime.processes.sender.pid()?;
    let gate = Mutex::new(
        (0..runtime.topology().link_count())
            .map(|i| runtime.topology().sender_iface(i).to_owned())
            .collect(),
    );
    let sampler = Sampler {
        ns: &sender_ns,
        stats_path: &stack.stats,
        clock,
        gate: &gate,
    };
    let mut samples = vec![sampler.sample()?];
    let mut scheduler = Scheduler::new(&profile.timeline)?;
    let mut origin_ms = None;
    let mut before_replug = Vec::new();
    let duration = profile.timeline.duration;
    let start_control = control::query(&stack.control, Duration::from_secs(1))?;
    let start_cpu = CpuSample::sample(pid, clock.now_ms())?;
    let (start_tx, start_rx) = std::sync::mpsc::channel();
    let run_result = std::thread::scope(|scope| -> Result<(Edges, Capture)> {
        let (stop_csv, csv_stopped) = std::sync::mpsc::channel();
        let csv_path = &request.result.record.raw.stats_csv_path;
        let initial_clock = &csv_clock;
        let csv_collector =
            scope.spawn(move || Capture::run(csv_path, (clock, initial_clock), csv_stopped));
        let sampler_ref = &sampler;
        let collector = scope.spawn(move || {
            start_rx.recv().context("measurement never started")?;
            sampler_ref.run(duration + Duration::from_secs(1))
        });
        let scheduled = scheduler.run(&mut |event| {
            if origin_ms.is_none() {
                ensure!(
                    event.at.is_zero()
                        && event.action
                            == Action::OfferedRate {
                                bps: measurement_rate(profile)?
                            },
                    "first measurement event must apply the offered rate"
                );
                origin_ms = Some(clock.now_ms());
            }
            if matches!(event.action, Action::Replug) {
                before_replug.push(sampler.sample()?);
            }
            let mut ifaces = gate.lock().expect("event sampling gate");
            ramp_duration.set(
                profile
                    .source_ramp
                    .filter(|r| r.at == event.at)
                    .map_or(Duration::ZERO, |r| r.duration),
            );
            let applied = match event.action {
                Action::ReceiverRestart => {
                    let start = Instant::now();
                    runtime.processes.receiver.restart_process_only()?;
                    if let Some(budget) = profile.receiver_restart_budget {
                        ensure!(start.elapsed() <= budget, RunFailure::RestartBudget);
                    }
                    Ok(())
                }
                _ => runtime.apply(event),
            };
            applied?;
            if matches!(event.action, Action::AddLink(_)) {
                let iface = runtime
                    .topology()
                    .sender_iface(runtime.topology().link_count() - 1);
                before_replug.push(
                    sampler.new_link_baseline(iface, origin_ms.context("measurement origin")?)?,
                );
                ifaces.push(iface.to_owned());
            }
            if event.at.is_zero() {
                start_tx.send(()).context("start collector")?;
            }
            Ok(())
        });
        drop(start_tx);
        let edges = (|| -> Result<Edges> {
            Ok(Edges {
                start_cpu,
                end_cpu: CpuSample::sample(pid, clock.now_ms())?,
                start_control,
                end_control: control::query(&stack.control, Duration::from_secs(1))?,
            })
        })();
        let collected = collector
            .join()
            .map_err(|_| anyhow::anyhow!("collector panicked"))?;
        drop(stop_csv);
        let capture = csv_collector
            .join()
            .map_err(|_| anyhow::anyhow!("CSV collector panicked"))?;
        samples.extend(collected?);
        scheduled?;
        Ok((edges?, capture?))
    });
    request.result.record.events = scheduler.log().entries.clone();
    let (edges, capture) = run_result?;
    drop(runtime);
    samples.extend(before_replug);
    check_config(
        requested.effective_config.as_ref(),
        edges
            .end_control
            .as_ref()
            .and_then(control::ControlMetrics::effective_config),
    )?;
    let origin_ms = origin_ms.context("measurement clock was not started")?;
    crate::checkpoint::atomic(
        &request.artifacts.join("clocks.json"),
        &serde_json::json!({
            "measurement_monotonic_ms": origin_ms, "sink_origin_monotonic_ms": stack.sink_origin_ms,
            "csv_offset_monotonic_ms": csv_clock.offset_ms, "csv_calibration_uncertainty_ms": csv_clock.uncertainty_ms,
            "csv_socket_clocks": capture.clocks,
            "capture_semantics": "interval"
        }),
    )?;
    collect::finish(
        &mut request.result.record,
        Measurement {
            profile,
            log: scheduler.log(),
            stats: capture.stats,
            origin_ms,
            sink_origin_ms: stack.sink_origin_ms,
            samples,
            edges,
        },
    )
}
