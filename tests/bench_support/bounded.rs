use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, ensure};
use network_sim::metrics::record::RunStatus;

use super::record::{Attempt, Request};
use crate::checkpoint::atomic;

pub fn execute(mut request: Request) -> Result<Attempt> {
    let request_path = request.artifacts.join("request.json");
    atomic(&request_path, &request)?;
    let log = std::fs::File::create(request.artifacts.join("worker.log"))?;
    let seconds = request
        .manifest
        .cell_profile(&request.manifest.cells[request.work.cell])?
        .timeline
        .duration
        .as_secs()
        .checked_add(60)
        .context("timeout overflow")?;
    let status = Command::new("timeout")
        .args(["--kill-after=10s", &format!("{seconds}s")])
        .arg(std::env::current_exe()?)
        .args(["--exact", "worker", "--ignored", "--nocapture"])
        .env("BENCH_WORKER_REQUEST", &request_path)
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .status()?;
    cleanup(&request.artifacts)?;
    let result_path = request.artifacts.join("result.json");
    if status.success() && result_path.exists() {
        let result: Attempt = serde_json::from_slice(&std::fs::read(result_path)?)?;
        ensure!(
            result.record.fingerprint == request.result.record.fingerprint
                && result.record.run_id == request.result.record.run_id,
            "worker identity mismatch"
        );
        return Ok(result);
    }
    request.result.record.status = RunStatus::Failed;
    request.result.reason = Some(
        if matches!(status.code(), Some(124 | 137)) {
            "run_timeout"
        } else {
            "worker_failed"
        }
        .into(),
    );
    request.result.detail = Some(format!("worker exit {status}; see worker.log"));
    Ok(request.result)
}

fn command(args: &[&str]) -> Result<std::process::Output> {
    Ok(Command::new("timeout")
        .args(["--kill-after=2s", "5s", "sudo", "-n"])
        .args(args)
        .output()?)
}

fn cleanup(artifacts: &Path) -> Result<()> {
    let owner = artifacts.join("owner.json");
    if !owner.exists() {
        return Ok(());
    }
    let pid: u32 = serde_json::from_slice(&std::fs::read(owner)?)?;
    let output = command(&["ip", "netns", "list"])?;
    ensure!(
        output.status.success(),
        "list worker namespaces for cleanup"
    );
    let pid = pid.to_string();
    for line in std::str::from_utf8(&output.stdout)?.lines() {
        let name = line.split_whitespace().next().context("namespace name")?;
        let mut suffix = name.rsplitn(3, '_');
        let counter = suffix.next().unwrap_or("");
        if suffix.next() != Some(&pid) || counter.parse::<u64>().is_err() {
            continue;
        }
        // Only namespaces whose PID+counter suffix belongs to this exact worker are eligible.
        let pids = command(&["ip", "netns", "pids", name])?;
        ensure!(pids.status.success(), "read worker namespace PIDs");
        let pids = std::str::from_utf8(&pids.stdout)?
            .split_whitespace()
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;
        for process in pids {
            let output = command(&["kill", "-KILL", "--", &process.to_string()])?;
            ensure!(
                output.status.success() || !Path::new(&format!("/proc/{process}")).exists(),
                "kill worker process"
            );
        }
        ensure!(
            command(&["ip", "netns", "del", name])?.status.success(),
            "delete worker namespace"
        );
    }
    let control_directory = artifacts.join("control-directory.json");
    if control_directory.exists() {
        let path: std::path::PathBuf = serde_json::from_slice(&std::fs::read(control_directory)?)?;
        ensure!(
            path.parent() == Some(std::env::temp_dir().as_path())
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("bench-ctl-")),
            "invalid worker control directory"
        );
        match std::fs::remove_dir_all(path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
