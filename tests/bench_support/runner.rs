use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use network_sim::metrics::record::RunStatus;

use super::record::{Attempt, Request, prepare};
use super::{bounded, live};
use crate::checkpoint::{Next, Store, atomic};
use crate::manifest::{Manifest, Work};

pub fn campaign(smoke: bool) -> Result<()> {
    ensure!(
        cfg!(feature = "test-internals"),
        "campaign requires --features test-internals"
    );
    let input =
        std::env::var("BENCH_MANIFEST").context("set BENCH_MANIFEST to JSON or a JSON file")?;
    let text = if input.trim_start().starts_with('{') {
        input
    } else {
        std::fs::read_to_string(input)?
    };
    let mut manifest: Manifest = crate::manifest::parse(&text)?;
    let smoke = smoke || std::env::var("BENCH_SMOKE").as_deref() == Ok("1");
    if smoke {
        manifest.smoke();
    }
    manifest.validate()?;
    for candidate in &mut manifest.candidates {
        candidate.bin = candidate.bin.canonicalize()?;
    }
    let defaults = network_sim::harness::ReceiverSpec::from_env()?;
    for receiver in &mut manifest.receivers {
        let spec = receiver.resolve(&defaults)?;
        receiver.kind = Some(spec.kind()?.into());
        receiver.bin = Some(spec.srtla_rec_bin);
        receiver.srt_live_transmit_bin = Some(spec.srt_live_transmit_bin);
    }
    manifest.receiver_defaults = Some(defaults);
    let max_attempts = std::env::var("BENCH_MAX_RETRIES")
        .unwrap_or_else(|_| "2".into())
        .parse::<u32>()?;
    ensure!(max_attempts > 0, "BENCH_MAX_RETRIES must be positive");
    ensure!(
        !super::twinport::enabled(&manifest.campaign) || max_attempts == 2,
        "TWINPORT requires two attempts, the second only for player_leg_invalid"
    );
    ensure!(
        !manifest.retains_measured_timeouts() || max_attempts == 1,
        "measurement spikes require exactly one attempt per planned index (BENCH_MAX_RETRIES=1)"
    );
    let output = std::env::var_os("BENCH_OUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".omo/evidence/bench").join(&manifest.campaign));
    std::fs::create_dir_all(&output)?;
    let output = output.canonicalize()?;
    let artifacts = std::env::var_os("BENCH_ARTIFACT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| output.join("artifacts"));
    std::fs::create_dir_all(&artifacts)?;
    let artifacts = artifacts.canonicalize()?;
    let _guard = crate::measurement::measurement_lock();
    super::candidate_lock::record(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/bench/receivers.lock.json"),
        &manifest,
    )?;
    atomic(&output.join("manifest.json"), &manifest)?;
    let work = manifest.order();
    let mut order_index = 0_u32;
    let same_pair = |a: &Work, b: &Work| {
        let (a_cell, b_cell) = (&manifest.cells[a.cell], &manifest.cells[b.cell]);
        a.run == b.run && a_cell.cell_id == b_cell.cell_id
    };
    for pair in work.chunk_by(same_pair) {
        loop {
            let mut attempted = false;
            for work in pair {
                let (record, receiver_spec) = prepare(&manifest, *work, order_index)?;
                let directory = output.join(manifest.cells[work.cell].id());
                let budget = if super::twinport::enabled(&manifest.campaign) {
                    super::twinport::budget(&directory, work.run)?
                } else {
                    max_attempts
                };
                let store = Store::new(&directory, record.fingerprint.clone(), budget);
                match store.next(work.run)? {
                    Next::Complete | Next::Exhausted => continue,
                    Next::Attempt(attempt) => {
                        attempted = true;
                        let directory = tempfile::Builder::new()
                            .prefix("run-")
                            .tempdir_in(&artifacts)?
                            .keep();
                        let mut request = Request {
                            manifest: manifest.clone(),
                            work: *work,
                            result: Attempt {
                                record,
                                attempt,
                                reason: None,
                                detail: None,
                            },
                            artifacts: directory,
                            srt_binary: receiver_spec.srt_live_transmit_bin.clone(),
                            receiver_spec,
                        };
                        request.result.record.raw.stats_csv_path =
                            request.artifacts.join("receiver.csv");
                        request.result.record.raw.sink_series_path =
                            request.artifacts.join("sink.csv");
                        let result = bounded::execute(request)?;
                        store.save(work.run, attempt, &result)?;
                        order_index = order_index.checked_add(1).context("order index overflow")?;
                    }
                }
            }
            if !attempted {
                break;
            }
        }
    }
    let mut missing = 0;
    for (cell_index, cell) in manifest.cells.iter().enumerate() {
        let (record, _) = prepare(
            &manifest,
            Work {
                cell: cell_index,
                run: 0,
            },
            0,
        )?;
        let store = Store::new(&output.join(cell.id()), record.fingerprint, max_attempts);
        missing += cell.runs - store.count_ok(cell.runs)?;
    }
    ensure!(
        missing == 0,
        "campaign exhausted with {missing} missing successful runs"
    );
    Ok(())
}

pub fn worker() -> Result<()> {
    let Some(path) = std::env::var_os("BENCH_WORKER_REQUEST") else {
        return Ok(());
    };
    let mut request: Request = serde_json::from_slice(&std::fs::read(path)?)?;
    atomic(&request.artifacts.join("owner.json"), &std::process::id())?;
    request.manifest.validate()?;
    let result = live::execute(&mut request);
    match result {
        Ok(()) => {
            ensure!(
                request.result.record.compute_fingerprint()? == request.result.record.fingerprint,
                "observed fingerprint differs from requested fingerprint"
            );
            request.result.record.status = RunStatus::Ok;
        }
        Err(error) => {
            request.result.reason = Some(
                error
                    .downcast_ref::<super::record::RunFailure>()
                    .map_or_else(|| "execution_error".into(), ToString::to_string),
            );
            request.result.detail = Some(format!("{error:#}"));
        }
    }
    atomic(&request.artifacts.join("result.json"), &request.result)?;
    Ok(())
}
