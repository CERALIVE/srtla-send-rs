use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};

use super::{NamespaceProcess, ProcessScope};

#[path = "process_identity.rs"]
mod identity;
pub(super) use identity::Identity;

static SPAWN_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy)]
pub(super) enum Teardown {
    Namespace,
    Process,
    None,
}

#[derive(Clone)]
pub(super) struct Launch {
    pub namespace: String,
    pub binary: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

#[derive(Debug)]
pub enum ProcessControlError {
    AlreadyExited,
    IdentityUnavailable,
    StopTimeout,
}

impl std::fmt::Display for ProcessControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "process control: {self:?}")
    }
}
impl std::error::Error for ProcessControlError {}

impl NamespaceProcess {
    pub(super) fn spawn_launch(launch: Launch, teardown: Teardown) -> Result<Self> {
        let _guard = SPAWN_LOCK
            .lock()
            .map_err(|_| anyhow::anyhow!("process spawn lock poisoned"))?;
        let before = identity::namespace_pids(&launch.namespace)?;
        let mut cmd = Command::new("sudo");
        cmd.args(["-n", "ip", "netns", "exec", &launch.namespace]);
        if !launch.env.is_empty() {
            cmd.arg("env");
            for (key, value) in &launch.env {
                cmd.arg(format!("{key}={value}"));
            }
        }
        let child = cmd
            .arg(&launch.binary)
            .args(&launch.args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("spawn namespace process")?;
        let mut process =
            Self::from_child(child, ProcessScope::Namespace(launch.namespace.clone()))?;
        process.teardown = teardown;
        process.label = format!("{} in ns:{}", launch.binary, launch.namespace);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            process.inner = identity::discover(&launch.namespace, &before, process.child.id())?;
            if process.inner.is_some() || process.child.try_wait()?.is_some() {
                break;
            }
            ensure!(
                Instant::now() < deadline,
                "namespace process PID discovery timed out"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        process.launch = Some(launch);
        tracing::debug!(label = %process.label, pid = ?process.inner, "spawned namespace process");
        Ok(process)
    }

    /// Exact inner PID, never the sudo wrapper. Short-lived exited launches may have none.
    pub fn pid(&self) -> Result<u32> {
        self.inner
            .as_ref()
            .map(|i| i.pid)
            .ok_or_else(|| ProcessControlError::IdentityUnavailable.into())
    }

    /// Auxiliary loads must never kill the sender/listener when their handle is dropped.
    pub fn spawn_process_only(ns: &crate::Namespace, binary: &str, args: &[&str]) -> Result<Self> {
        Self::spawn_launch(
            Launch {
                namespace: ns.name.clone(),
                binary: binary.into(),
                args: args.iter().map(|s| (*s).into()).collect(),
                env: Vec::new(),
            },
            Teardown::Process,
        )
    }

    pub fn signal_process_only(&mut self, signal: &str) -> Result<()> {
        self.require_live_identity()?;
        let pid = self.pid()?;
        let output = Command::new("sudo")
            .args(["-n", "kill", signal, "--", &pid.to_string()])
            .output()?;
        ensure!(
            output.status.success(),
            "signal PID {pid}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }

    /// Stop only this inner PID and disarm namespace-wide Drop for this retired handle.
    /// Ordinary handles retain namespace-wide teardown; the caller owns all descendants.
    pub fn stop_process_only(&mut self) -> Result<()> {
        if self.child.try_wait()?.is_some() {
            self.reaped = true;
            self.teardown = Teardown::None;
            self.join_drains();
            return Ok(());
        }
        self.require_live_identity()?;
        for signal in ["-TERM", "-KILL"] {
            self.signal_process_only(signal)?;
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                if self.child.try_wait()?.is_some() {
                    self.reaped = true;
                    self.teardown = Teardown::None;
                    self.join_drains();
                    return Ok(());
                }
                if Instant::now() >= deadline {
                    break;
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        Err(ProcessControlError::StopTimeout.into())
    }

    pub fn restart_process_only(&mut self) -> Result<()> {
        self.require_live_identity()?;
        let launch = self
            .launch
            .clone()
            .ok_or(ProcessControlError::IdentityUnavailable)?;
        let teardown = self.teardown;
        self.stop_process_only()?;
        let replacement = Self::spawn_launch(launch, teardown)?;
        *self = replacement;
        Ok(())
    }

    fn require_live_identity(&mut self) -> Result<()> {
        if self.child.try_wait()?.is_some() {
            return Err(ProcessControlError::AlreadyExited.into());
        }
        let inner = self
            .inner
            .as_ref()
            .ok_or(ProcessControlError::IdentityUnavailable)?;
        ensure!(
            identity::read(inner.pid)?.start_time == inner.start_time,
            ProcessControlError::IdentityUnavailable
        );
        Ok(())
    }
}

impl super::SrtlaTestStack {
    pub fn restart_receiver(&mut self) -> Result<()> {
        self.srtla_rec
            .as_mut()
            .context("receiver stack is stopped")?
            .restart_process_only()?;
        super::wait_for_udp_listener(&self.topo.receiver_ns, 5000, Duration::from_secs(5))
    }
}
