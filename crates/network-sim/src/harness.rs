//! Test harness for end-to-end SRTLA integration tests.
//!
//! Provides [`SrtlaTestTopology`] for network namespace setup,
//! [`NamespaceProcess`] for managed child processes inside namespaces,
//! and [`SrtlaTestStack`] for the full 3-process test pipeline
//! (srt-live-transmit + srtla_rec + srtla_send).

// allow: SIZE_OK — Existing harness compatibility surface; new process-control logic lives in its own module.

use std::collections::{HashSet, VecDeque};
use std::io::Read;
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::process::Stdio;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use crate::impairment::{ImpairmentConfig, apply_impairment};
use crate::test_util::unique_ns_name;
use crate::topology::Namespace;

mod process_control;
pub use process_control::ProcessControlError;

// ---------------------------------------------------------------------------
// Dependency checking
// ---------------------------------------------------------------------------

/// Check if a binary exists in PATH.
pub fn check_binary(name: &str) -> Option<PathBuf> {
    Command::new("sh")
        .args(["-c", &format!("command -v {name}")])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string()))
}

/// Reason why integration tests must be skipped.
#[derive(Debug)]
pub enum SkipReason {
    NotRoot,
    MissingBinary(String),
    MissingTool(String),
    InvalidConfiguration(String),
    NoNetem,
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::NotRoot => write!(f, "requires root / passwordless sudo"),
            SkipReason::MissingBinary(b) => write!(f, "{b} not found in PATH"),
            SkipReason::MissingTool(t) => write!(f, "system tool '{t}' not found"),
            SkipReason::InvalidConfiguration(message) => f.write_str(message),
            SkipReason::NoNetem => write!(
                f,
                "sch_netem kernel module not available (try: sudo modprobe sch_netem)"
            ),
        }
    }
}

/// Check all dependencies needed for integration tests.
///
/// Returns `Ok(())` if everything is available, or `Err(SkipReason)` with
/// the first missing dependency.
pub fn check_integration_deps() -> std::result::Result<(), SkipReason> {
    // Root / sudo check
    if !crate::test_util::check_privileges() {
        return Err(SkipReason::NotRoot);
    }

    // External binaries
    for (key, bin) in [
        ("SRTLA_REC_BIN", "srtla_rec"),
        ("SRT_LIVE_TRANSMIT_BIN", "srt-live-transmit"),
    ] {
        let resolved = resolve_external_binary(key, bin)
            .map_err(|error| SkipReason::InvalidConfiguration(error.to_string()))?;
        tracing::info!(binary = bin, path = %resolved.display(), "resolved integration dependency");
    }
    ReceiverKind::from_env()
        .map_err(|error| SkipReason::InvalidConfiguration(error.to_string()))?;

    // System tools
    for tool in &["ip", "tc", "ss"] {
        if check_binary(tool).is_none() {
            return Err(SkipReason::MissingTool(tool.to_string()));
        }
    }

    Ok(())
}

/// Check deps including netem (for tests that apply impairment).
pub fn check_impairment_deps() -> std::result::Result<(), SkipReason> {
    check_integration_deps()?;

    // Try to load sch_netem and check if it succeeded
    let modprobe_ok = Command::new("sudo")
        .args(["modprobe", "sch_netem"])
        .output()
        .is_ok_and(|o| o.status.success());

    if !modprobe_ok {
        return Err(SkipReason::NoNetem);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// NamespaceProcess
// ---------------------------------------------------------------------------

const OUTPUT_TAIL_BYTES: usize = 64 * 1024;

type OutputTail = Arc<Mutex<VecDeque<u8>>>;

fn new_output_tail() -> OutputTail {
    Arc::new(Mutex::new(VecDeque::with_capacity(OUTPUT_TAIL_BYTES)))
}

fn drain_pipe<R>(mut reader: R, tail: OutputTail) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut buffer = [0_u8; 8192];
        while let Ok(bytes_read) = reader.read(&mut buffer) {
            if bytes_read == 0 {
                break;
            }
            if let Ok(mut output) = tail.lock() {
                output.extend(&buffer[..bytes_read]);
                while output.len() > OUTPUT_TAIL_BYTES {
                    output.pop_front();
                }
            }
        }
    })
}

fn output_lines(tail: &OutputTail) -> Vec<String> {
    let Ok(mut output) = tail.lock() else {
        return Vec::new();
    };
    String::from_utf8_lossy(output.make_contiguous())
        .lines()
        .map(str::to_owned)
        .collect()
}

/// A child process running inside a network namespace.
///
/// Captures stdout+stderr and kills the process on drop.
enum ProcessScope {
    Namespace(String),
    #[cfg(test)]
    Pids(Vec<u32>),
}

pub struct NamespaceProcess {
    child: Child,
    label: String,
    scope: ProcessScope,
    reaped: bool,
    stdout_tail: OutputTail,
    stderr_tail: OutputTail,
    drain_threads: Vec<JoinHandle<()>>,
    launch: Option<process_control::Launch>,
    inner: Option<process_control::Identity>,
    teardown: process_control::Teardown,
}

impl NamespaceProcess {
    fn from_child(mut child: Child, scope: ProcessScope) -> Result<Self> {
        let stdout_tail = new_output_tail();
        let stderr_tail = new_output_tail();
        let stdout = child.stdout.take().context("capture child stdout")?;
        let stderr = child.stderr.take().context("capture child stderr")?;
        let drain_threads = vec![
            drain_pipe(stdout, Arc::clone(&stdout_tail)),
            drain_pipe(stderr, Arc::clone(&stderr_tail)),
        ];
        Ok(Self {
            child,
            label: "test child".to_string(),
            scope,
            reaped: false,
            stdout_tail,
            stderr_tail,
            drain_threads,
            launch: None,
            inner: None,
            teardown: process_control::Teardown::Namespace,
        })
    }

    /// Spawn `binary args...` inside `ns` via `sudo ip netns exec`.
    pub fn spawn(ns: &Namespace, binary: &str, args: &[&str]) -> Result<Self> {
        Self::spawn_with_env(ns, binary, args, &[])
    }

    /// Spawn with additional environment variables as `(key, value)` pairs.
    pub fn spawn_with_env(
        ns: &Namespace,
        binary: &str,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<Self> {
        Self::spawn_launch(
            process_control::Launch {
                namespace: ns.name.clone(),
                binary: binary.into(),
                args: args.iter().map(|s| (*s).into()).collect(),
                env: env
                    .iter()
                    .map(|(k, v)| ((*k).into(), (*v).into()))
                    .collect(),
            },
            process_control::Teardown::Namespace,
        )
    }

    /// Read all captured stdout lines (non-blocking snapshot via `try_wait`).
    /// Only meaningful after the process has exited.
    pub fn stdout_lines(&mut self) -> Vec<String> {
        output_lines(&self.stdout_tail)
    }

    /// Read all captured stderr lines. Only meaningful after exit.
    pub fn stderr_lines(&mut self) -> Vec<String> {
        output_lines(&self.stderr_tail)
    }

    /// Snapshot the live stdout+stderr tail without waiting for exit. The drain
    /// threads keep both tails current, so this is safe to poll while running.
    pub fn log_snapshot(&self) -> Vec<String> {
        let mut lines = output_lines(&self.stdout_tail);
        lines.extend(output_lines(&self.stderr_tail));
        lines
    }

    fn join_drains(&mut self) {
        for drain_thread in self.drain_threads.drain(..) {
            let _ = drain_thread.join();
        }
    }

    /// Send SIGTERM to the namespace's exact PIDs, then SIGKILL if needed.
    pub fn kill(&mut self) {
        if self.reaped && self.scope_pids().is_empty() {
            self.join_drains();
            return;
        }

        self.signal_scope("-TERM");
        if self.wait_until_stopped(Duration::from_secs(2)) {
            self.join_drains();
            return;
        }

        self.signal_scope("-KILL");
        let _ = self.child.kill();
        if !self.wait_until_stopped(Duration::from_secs(2)) {
            tracing::warn!(label = %self.label, "namespace process teardown exceeded grace period");
            drop(self.child.stdout.take());
            drop(self.child.stderr.take());
        }
        self.join_drains();
    }

    /// Check if the process is still running.
    pub fn is_alive(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }

    /// If the process has exited, return its exit code and stderr.
    /// Returns `None` if still running.
    pub fn check_exit(&mut self) -> Option<(Option<i32>, String)> {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                self.join_drains();
                let stderr = self.stderr_lines().join("\n");
                Some((status.code(), stderr))
            }
            _ => None,
        }
    }

    fn signal_scope(&self, signal: &str) {
        let pids = self.scope_pids();
        if pids.is_empty() {
            return;
        }

        let mut command = match &self.scope {
            ProcessScope::Namespace(_) => {
                let mut command = Command::new("sudo");
                command.args(["-n", "kill"]);
                command
            }
            #[cfg(test)]
            ProcessScope::Pids(_) => Command::new("kill"),
        };
        command.args([signal, "--"]);
        command.args(pids.iter().map(u32::to_string));
        let _ = command.output();
    }

    fn wait_until_stopped(&mut self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        loop {
            if let Ok(Some(_)) = self.child.try_wait() {
                self.reaped = true;
            }
            if self.reaped && self.scope_pids().is_empty() {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn scope_pids(&self) -> Vec<u32> {
        match &self.scope {
            ProcessScope::Namespace(namespace) => {
                let Ok(output) = Command::new("sudo")
                    .args(["-n", "ip", "netns", "pids", namespace])
                    .output()
                else {
                    return Vec::new();
                };
                if !output.status.success() {
                    return Vec::new();
                }
                String::from_utf8_lossy(&output.stdout)
                    .split_whitespace()
                    .filter_map(|pid| pid.parse().ok())
                    .collect()
            }
            #[cfg(test)]
            ProcessScope::Pids(pids) => pids
                .iter()
                .copied()
                .filter(|pid| {
                    Command::new("kill")
                        .args(["-0", "--", &pid.to_string()])
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status()
                        .is_ok_and(|status| status.success())
                })
                .collect(),
        }
    }
}

impl Drop for NamespaceProcess {
    fn drop(&mut self) {
        match self.teardown {
            process_control::Teardown::Namespace => self.kill(),
            process_control::Teardown::Process => {
                if let Err(error) = self.stop_process_only() {
                    tracing::warn!(%error, "process-only teardown failed");
                }
            }
            process_control::Teardown::None => {}
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod namespace_process_tests {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::{NamespaceProcess, ProcessScope, new_output_tail};

    #[test]
    fn restart_process_only_when_already_exited_preserves_other_pids() {
        // Given: a reaped child and a separate live process in its teardown scope.
        let mut other = Command::new("sleep").arg("30").spawn().unwrap();
        let child = Command::new("true")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut process =
            NamespaceProcess::from_child(child, ProcessScope::Pids(vec![other.id()])).unwrap();
        process.child.wait().unwrap();
        // When: attempting a receiver-only restart after exit.
        let error = process.restart_process_only().unwrap_err();
        let alive = other.try_wait().unwrap().is_none();
        other.kill().unwrap();
        other.wait().unwrap();
        // Then: the error is typed and the unrelated process received no signal.
        assert!(matches!(
            error.downcast_ref(),
            Some(super::ProcessControlError::AlreadyExited)
        ));
        assert!(alive);
    }

    fn signal_exact(signal: &str, target: &str) {
        let _ = Command::new("kill").args([signal, "--", target]).status();
    }

    fn is_alive(pid: u32) -> bool {
        Command::new("kill")
            .args(["-0", "--", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    #[test]
    fn kill_returns_when_wrapper_and_inner_process_have_mismatched_groups() {
        let mut child = Command::new("sh")
            .args([
                "-c",
                "setsid sh -c 'trap \"\" TERM INT; echo ready; exec sleep 30' & echo $!; wait",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn mismatched process groups");
        let wrapper_pid = child.id();
        let mut reader = BufReader::new(child.stdout.take().expect("child stdout"));
        let mut pid_line = String::new();
        reader.read_line(&mut pid_line).expect("read inner pid");
        let inner_pid = pid_line.trim().parse::<u32>().expect("parse inner pid");
        let mut readiness_line = String::new();
        reader
            .read_line(&mut readiness_line)
            .expect("read inner readiness");
        assert_eq!(readiness_line.trim(), "ready");

        let wrapper_pgid = Command::new("ps")
            .args(["-o", "pgid=", "-p", &wrapper_pid.to_string()])
            .output()
            .expect("read wrapper pgid");
        let wrapper_pgid = String::from_utf8_lossy(&wrapper_pgid.stdout)
            .trim()
            .parse::<u32>()
            .expect("parse wrapper pgid");
        assert_ne!(
            wrapper_pid, wrapper_pgid,
            "wrapper unexpectedly leads its PGID"
        );
        assert_ne!(
            wrapper_pgid, inner_pid,
            "inner process did not create a new group"
        );
        for signal in ["-TERM", "-INT", "-TERM", "-INT"] {
            signal_exact(signal, &inner_pid.to_string());
        }
        assert!(is_alive(inner_pid), "inner process did not ignore TERM/INT");

        let mut process = NamespaceProcess {
            child,
            label: "mismatched process groups".to_string(),
            scope: ProcessScope::Pids(vec![inner_pid]),
            reaped: false,
            stdout_tail: new_output_tail(),
            stderr_tail: new_output_tail(),
            drain_threads: vec![],
            launch: None,
            inner: None,
            teardown: super::process_control::Teardown::Namespace,
        };
        let (done_tx, done_rx) = mpsc::channel();
        let teardown = thread::spawn(move || {
            process.kill();
            process.kill();
            drop(process);
            let _ = done_tx.send(());
        });

        let completed = done_rx.recv_timeout(Duration::from_secs(4)).is_ok();
        if !completed {
            signal_exact("-KILL", &wrapper_pid.to_string());
            signal_exact("-KILL", &format!("-{inner_pid}"));
        }
        teardown.join().expect("teardown thread");

        assert!(
            completed,
            "NamespaceProcess::kill blocked on mismatched process groups"
        );
        assert!(
            !is_alive(wrapper_pid),
            "wrapper process leaked after teardown"
        );
        assert!(!is_alive(inner_pid), "inner process leaked after teardown");
    }

    #[test]
    fn child_pipes_are_drained_during_sustained_output() {
        let child = Command::new("sh")
            .args([
                "-c",
                "head -c 131072 /dev/zero; head -c 131072 /dev/zero >&2",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn pipe stress child");
        let pid = child.id();
        let mut process = NamespaceProcess::from_child(child, ProcessScope::Pids(vec![pid]))
            .expect("wrap pipe stress child");

        let deadline = Instant::now() + Duration::from_secs(2);
        let mut exit = None;
        while Instant::now() < deadline {
            if let Some(status) = process.check_exit() {
                exit = Some(status);
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        let completed = exit.is_some();
        if !completed {
            process.kill();
        }
        assert!(
            completed,
            "child did not complete while both pipes were written"
        );
        assert!(!process.stdout_lines().is_empty(), "stdout tail was empty");
        assert!(!process.stderr_lines().is_empty(), "stderr tail was empty");
    }
}

// ---------------------------------------------------------------------------
// SrtlaTestTopology
// ---------------------------------------------------------------------------

/// Network topology with sender and receiver namespaces connected by N veth links.
pub struct SrtlaTestTopology {
    pub sender_ns: Namespace,
    pub receiver_ns: Namespace,
    /// Sender-side IPs, e.g. `["10.10.1.1", "10.10.2.1"]`.
    pub sender_ips: Vec<String>,
    /// Receiver-side IP on the first link (used for srtla_rec bind address).
    pub receiver_ip: String,
    /// Sender-side veth interface names (for applying impairment).
    pub sender_ifaces: Vec<String>,
    /// Receiver-side veth interface names.
    pub receiver_ifaces: Vec<String>,
}

impl SrtlaTestTopology {
    /// Create a new topology with `num_links` veth pairs.
    ///
    /// Addresses use `10.10.{i+1}.{1|2}/24` where `i` is the link index.
    pub fn new(test_name: &str, num_links: usize) -> Result<Self> {
        assert!(num_links > 0, "need at least one link");

        let sender_ns = Namespace::new(&unique_ns_name(&format!("{test_name}_s")))?;
        let receiver_ns = Namespace::new(&unique_ns_name(&format!("{test_name}_r")))?;

        let mut sender_ips = Vec::with_capacity(num_links);
        let mut sender_ifaces = Vec::with_capacity(num_links);
        let mut receiver_ifaces = Vec::with_capacity(num_links);

        for i in 0..num_links {
            let subnet = i + 1;
            let s_ip = format!("10.10.{subnet}.1");
            let r_ip = format!("10.10.{subnet}.2");
            let s_iface = unique_ns_name(&format!("vs{i}"));
            let r_iface = unique_ns_name(&format!("vr{i}"));

            sender_ns.add_veth_link(
                &receiver_ns,
                &s_iface,
                &r_iface,
                &format!("{s_ip}/24"),
                &format!("{r_ip}/24"),
            )?;

            sender_ips.push(s_ip);
            sender_ifaces.push(s_iface);
            receiver_ifaces.push(r_iface);
        }

        let receiver_ip = "10.10.1.2".to_string();

        Ok(Self {
            sender_ns,
            receiver_ns,
            sender_ips,
            receiver_ip,
            sender_ifaces,
            receiver_ifaces,
        })
    }

    /// Apply impairment to sender-side veth link at `idx`.
    pub fn impair_link(&self, idx: usize, config: ImpairmentConfig) -> Result<()> {
        let iface = self
            .sender_ifaces
            .get(idx)
            .with_context(|| format!("link index {idx} out of range"))?;
        apply_impairment(&self.sender_ns, iface, config)
    }

    /// Write sender IPs to a temp file and return the path.
    pub fn write_ip_list(&self) -> Result<PathBuf> {
        let dir = tempfile::tempdir().context("create temp dir for IP list")?;
        let path = dir.keep().join("srtla_ips.txt");
        std::fs::write(&path, self.sender_ips.join("\n") + "\n").context("write IP list")?;
        Ok(path)
    }
}

// ---------------------------------------------------------------------------
// Waiting helpers
// ---------------------------------------------------------------------------

/// Poll `ss -uln` inside `ns` until `port` appears as a UDP listener.
pub fn wait_for_udp_listener(ns: &Namespace, port: u16, timeout: Duration) -> Result<()> {
    let start = Instant::now();
    let port_str = format!(":{port}");
    let mut last_ss_output;

    loop {
        let out = ns.exec("ss", &["-uln"])?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        if stdout.lines().any(|line| line.contains(&port_str)) {
            return Ok(());
        }
        last_ss_output = stdout.to_string();

        if start.elapsed() > timeout {
            bail!(
                "timeout waiting for UDP listener on port {port} in ns {}\nlast ss -uln \
                 output:\n{last_ss_output}",
                ns.name
            );
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Count the uplinks that have reached REG3 according to the sender's own log.
///
/// `srtla_send` logs `REG3 from uplink #N` per uplink and
/// `connection established (active=N)` for the aggregate, so registration
/// readiness is observable from the log alone. Both signals are read because the
/// aggregate line is only emitted on change.
/// Block until at least `min_count` of `process`'s uplinks have completed
/// registration (REG3), or `timeout` elapses.
///
/// The free-function form is for stacks assembled outside `SrtlaTestStack`
/// (custom routing, extra CLI flags); `SrtlaTestStack::wait_for_registered_uplinks`
/// delegates here so both paths share one readiness definition.
pub fn wait_for_registered_uplinks(
    process: &NamespaceProcess,
    min_count: usize,
    timeout: Duration,
) -> Result<()> {
    let start = Instant::now();
    loop {
        let log = process.log_snapshot();
        let registered = registered_uplink_count(&log);
        if registered >= min_count {
            return Ok(());
        }
        if start.elapsed() > timeout {
            bail!(
                "timeout waiting for {min_count} registered uplink(s) (saw \
                 {registered})\nsrtla_send log:\n{}",
                log.join("\n")
            );
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn registered_uplink_count(log: &[String]) -> usize {
    let mut reg3_uplinks: HashSet<String> = HashSet::new();
    let mut max_active = 0usize;

    for line in log {
        if let Some(rest) = line.split("REG3 from uplink #").nth(1) {
            let idx: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if !idx.is_empty() {
                let _ = reg3_uplinks.insert(idx);
            }
        }
        if let Some(rest) = line.split("connection established (active=").nth(1) {
            let count: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(parsed) = count.parse::<usize>() {
                max_active = max_active.max(parsed);
            }
        }
    }

    reg3_uplinks.len().max(max_active)
}

// ---------------------------------------------------------------------------
// SrtlaTestStack
// ---------------------------------------------------------------------------

/// Explicit libsrt listener tuning. Only `LEGACY_DEFAULT` omits URI parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SrtProfile {
    pub latency_ms: u32,
    pub lossmaxttl: u32,
    pub name: &'static str,
}

impl SrtProfile {
    pub const PRODUCTION: Self = Self {
        latency_ms: 2000,
        lossmaxttl: 40,
        name: "production",
    };
    pub const STRICT: Self = Self {
        latency_ms: 500,
        lossmaxttl: 10,
        name: "strict",
    };
    /// Sentinel for the old URI, not a request to configure libsrt with zeroes.
    pub const LEGACY_DEFAULT: Self = Self {
        latency_ms: 0,
        lossmaxttl: 0,
        name: "legacy-default",
    };

    pub fn listener_uri(&self, port: u16) -> String {
        let uri = format!("srt://:{port}?mode=listener");
        if *self == Self::LEGACY_DEFAULT {
            uri
        } else {
            let receiver_options = if *self == Self::PRODUCTION {
                "&reorderfreeze=1"
            } else {
                ""
            };
            format!(
                "{uri}&latency={}&lossmaxttl={}{receiver_options}",
                self.latency_ms, self.lossmaxttl
            )
        }
    }

    /// Listener arguments; `-stats 1000` counts packets, not milliseconds.
    /// Capture directories must already exist; non-UTF-8 paths are rejected.
    pub fn listener_argv(&self, port: u16, stats_csv: Option<&Path>) -> Result<Vec<String>> {
        let mut args = Vec::new();
        if let Some(path) = stats_csv {
            args.extend([
                "-statsout".into(),
                path.to_str()
                    .context("stats_csv path must be UTF-8")?
                    .into(),
                "-statspf:csv".into(),
                "-stats".into(),
                "1000".into(),
            ]);
        }
        args.extend([self.listener_uri(port), "udp://127.0.0.1:9999".into()]);
        Ok(args)
    }
}

/// Receiver CLI dialect; selected by `SRTLA_REC_KIND` (default `ceralive`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiverKind {
    CeraLive,
    Irlserver,
    Belabox,
}

impl ReceiverKind {
    pub fn from_env() -> Result<Self> {
        Self::parse(std::env::var_os("SRTLA_REC_KIND").as_deref())
    }

    fn parse(value: Option<&std::ffi::OsStr>) -> Result<Self> {
        match value {
            None => Ok(Self::CeraLive),
            Some(value) => match value.to_str() {
                Some("ceralive") => Ok(Self::CeraLive),
                Some("irlserver") => Ok(Self::Irlserver),
                Some("belabox") => Ok(Self::Belabox),
                Some(_) | None => bail!(
                    "invalid SRTLA_REC_KIND {value:?}; expected ceralive, irlserver or belabox"
                ),
            },
        }
    }

    /// The three endpoint values follow each receiver's native CLI contract.
    pub fn argv(&self, srtla_port: u16, srt_host: &str, srt_port: u16) -> Vec<String> {
        match self {
            Self::CeraLive | Self::Irlserver => vec![
                "--srtla_port".into(),
                srtla_port.to_string(),
                "--srt_hostname".into(),
                srt_host.into(),
                "--srt_port".into(),
                srt_port.to_string(),
            ],
            Self::Belabox => vec![
                srtla_port.to_string(),
                srt_host.into(),
                srt_port.to_string(),
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReceiverSpec {
    pub label: String,
    pub lineage: String,
    pub srtla_rec_kind: String,
    pub srtla_rec_bin: PathBuf,
    pub srt_live_transmit_bin: PathBuf,
    pub listener_uri_extra: String,
}

impl ReceiverSpec {
    pub fn from_env() -> Result<Self> {
        let kind = std::env::var("SRTLA_REC_KIND").unwrap_or_else(|_| "ceralive".into());
        ReceiverKind::parse(Some(std::ffi::OsStr::new(&kind)))?;
        Ok(Self {
            label: kind.clone(),
            lineage: kind.clone(),
            srtla_rec_kind: kind,
            srtla_rec_bin: std::env::var_os("SRTLA_REC_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| "srtla_rec".into()),
            srt_live_transmit_bin: std::env::var_os("SRT_LIVE_TRANSMIT_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| "srt-live-transmit".into()),
            listener_uri_extra: String::new(),
        })
    }

    pub fn resolved(mut self) -> Result<Self> {
        fn resolve(path: &Path) -> Result<PathBuf> {
            let found = if path.components().count() == 1 {
                if path.is_file() {
                    path.to_owned()
                } else {
                    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                        .map(|directory| directory.join(path))
                        .find(|candidate| candidate.is_file())
                        .with_context(|| {
                            format!("receiver binary {} not on PATH", path.display())
                        })?
                }
            } else {
                path.to_owned()
            };
            anyhow::ensure!(
                found.is_file(),
                "receiver binary {} is not a file",
                found.display()
            );
            found
                .canonicalize()
                .with_context(|| format!("receiver binary {}", path.display()))
        }
        self.kind()?;
        Self::validate_options(&self.listener_uri_extra)?;
        self.srtla_rec_bin = resolve(&self.srtla_rec_bin)?;
        self.srt_live_transmit_bin = resolve(&self.srt_live_transmit_bin)?;
        Ok(self)
    }

    pub fn kind(&self) -> Result<ReceiverKind> {
        ReceiverKind::parse(Some(std::ffi::OsStr::new(&self.srtla_rec_kind)))
    }

    pub fn validate_options(extra: &str) -> Result<()> {
        if extra.is_empty() {
            return Ok(());
        }
        anyhow::ensure!(
            extra.starts_with('&'),
            "listener_uri_extra must begin with &"
        );
        let mut names = std::collections::BTreeSet::new();
        for option in extra[1..].split('&') {
            let (key, value) = option
                .split_once('=')
                .context("listener option requires key=value")?;
            anyhow::ensure!(
                !key.is_empty()
                    && !value.is_empty()
                    && names.insert(key)
                    && key.bytes().all(|b| b.is_ascii_alphanumeric())
                    && value
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
                    && !["mode", "adapter", "port", "packetfilter"].contains(&key),
                "invalid or duplicate listener option {key}"
            );
        }
        Ok(())
    }

    pub fn listener_argv(
        &self,
        profile: SrtProfile,
        port: u16,
        stats_csv: Option<&Path>,
    ) -> Result<Vec<String>> {
        Self::validate_options(&self.listener_uri_extra)?;
        let mut args = profile.listener_argv(port, stats_csv)?;
        let index = args.len() - 2;
        let base = args[index].clone();
        let (endpoint, query) = base.split_once('?').context("listener URI query")?;
        let mut options: Vec<&str> = query.split('&').collect();
        for option in self.listener_uri_extra.split('&').filter(|s| !s.is_empty()) {
            let (key, _) = option.split_once('=').context("listener option")?;
            options.retain(|existing| existing.split_once('=').is_none_or(|(name, _)| name != key));
            options.push(option);
        }
        args[index] = format!("{endpoint}?{}", options.join("&"));
        Ok(args)
    }
}

/// Full 3-process SRTLA test stack: srt-live-transmit + srtla_rec + srtla_send.
pub struct SrtlaTestStack {
    pub topo: SrtlaTestTopology,
    srt_server: Option<NamespaceProcess>,
    srtla_rec: Option<NamespaceProcess>,
    srtla_send: Option<NamespaceProcess>,
    _ip_list_path: PathBuf,
}

/// Output collected from all processes after stopping the stack.
pub struct StackOutput {
    pub srt_server_stdout: Vec<String>,
    pub srt_server_stderr: Vec<String>,
    pub srtla_rec_stdout: Vec<String>,
    pub srtla_rec_stderr: Vec<String>,
    pub srtla_send_stdout: Vec<String>,
    pub srtla_send_stderr: Vec<String>,
}

/// Ports used by the test stack.
const SRT_SERVER_PORT: u16 = 4001;
const SRTLA_REC_PORT: u16 = 5000;
const SRTLA_SEND_SRT_PORT: u16 = 5555;

impl SrtlaTestStack {
    /// Start the full stack: srt-live-transmit → srtla_rec → srtla_send.
    ///
    /// `sender_extra_args` are appended to the srtla_send command line.
    /// Pass `SrtProfile::LEGACY_DEFAULT` and `None` to retain the old listener argv.
    /// Profile and capture remain explicit arguments so existing tests cannot opt in silently.
    pub fn start(
        test_name: &str,
        num_links: usize,
        sender_extra_args: &[&str],
        srt_profile: SrtProfile,
        stats_csv: Option<PathBuf>,
    ) -> Result<Self> {
        let receiver = ReceiverSpec::from_env()?.resolved()?;
        let srt_binary = &receiver.srt_live_transmit_bin;
        let rec_binary = &receiver.srtla_rec_bin;
        let receiver_kind = receiver.kind()?;
        let listener_args =
            receiver.listener_argv(srt_profile, SRT_SERVER_PORT, stats_csv.as_deref())?;
        let receiver_args = receiver_kind.argv(SRTLA_REC_PORT, "127.0.0.1", SRT_SERVER_PORT);
        let topo = SrtlaTestTopology::new(test_name, num_links)?;
        let ip_list_path = topo.write_ip_list()?;

        // 1. Start srt-live-transmit in receiver NS
        let mut srt_server = NamespaceProcess::spawn(
            &topo.receiver_ns,
            srt_binary
                .to_str()
                .context("SRT tool binary path must be UTF-8")?,
            &listener_args.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .context("start srt-live-transmit")?;

        // Brief pause for listener setup, then check it's alive
        std::thread::sleep(Duration::from_millis(500));
        if let Some((code, stderr)) = srt_server.check_exit() {
            bail!("srt-live-transmit exited immediately (code: {code:?})\nstderr:\n{stderr}");
        }
        wait_for_udp_listener(&topo.receiver_ns, SRT_SERVER_PORT, Duration::from_secs(5))
            .context("wait for srt-live-transmit")?;

        // 2. Start srtla_rec in receiver NS
        let mut srtla_rec = NamespaceProcess::spawn(
            &topo.receiver_ns,
            rec_binary
                .to_str()
                .context("receiver binary path must be UTF-8")?,
            &receiver_args.iter().map(String::as_str).collect::<Vec<_>>(),
        )
        .context("start srtla_rec")?;

        // Wait for srtla_rec to be listening
        std::thread::sleep(Duration::from_millis(500));
        if let Some((code, stderr)) = srtla_rec.check_exit() {
            bail!("srtla_rec exited immediately (code: {code:?})\nstderr:\n{stderr}");
        }
        wait_for_udp_listener(&topo.receiver_ns, SRTLA_REC_PORT, Duration::from_secs(5))
            .context("wait for srtla_rec")?;

        // 3. Start srtla_send in sender NS
        let send_port = SRTLA_SEND_SRT_PORT.to_string();
        let rec_port = SRTLA_REC_PORT.to_string();
        let ip_list_str = ip_list_path.to_string_lossy().to_string();

        let mut send_args = vec![
            send_port.as_str(),
            topo.receiver_ip.as_str(),
            rec_port.as_str(),
            ip_list_str.as_str(),
        ];
        send_args.extend_from_slice(sender_extra_args);

        // Find the srtla_send binary built by cargo
        let srtla_send_bin = find_srtla_send_binary()?;
        let bin_str = srtla_send_bin.to_string_lossy().to_string();

        let srtla_send = NamespaceProcess::spawn_with_env(
            &topo.sender_ns,
            &bin_str,
            &send_args,
            &[("RUST_LOG", "debug")],
        )
        .context("start srtla_send")?;

        Ok(Self {
            topo,
            srt_server: Some(srt_server),
            srtla_rec: Some(srtla_rec),
            srtla_send: Some(srtla_send),
            _ip_list_path: ip_list_path,
        })
    }

    /// Snapshot srtla_send's live log without stopping the stack.
    pub fn sender_log_snapshot(&self) -> Vec<String> {
        self.srtla_send
            .as_ref()
            .map(NamespaceProcess::log_snapshot)
            .unwrap_or_default()
    }

    /// Block until at least `min_count` uplinks have completed registration
    /// (REG3), or `timeout` elapses.
    ///
    /// Registration is the readiness signal because uplink sockets are
    /// unconnected — `ss` can no longer report a connected UDP peer per uplink.
    pub fn wait_for_registered_uplinks(&self, min_count: usize, timeout: Duration) -> Result<()> {
        match self.srtla_send.as_ref() {
            Some(process) => wait_for_registered_uplinks(process, min_count, timeout),
            None => bail!("srtla_send is not running"),
        }
    }

    /// Apply impairment to sender-side link at `idx`.
    pub fn impair_link(&self, idx: usize, config: ImpairmentConfig) -> Result<()> {
        self.topo.impair_link(idx, config)
    }

    /// The local SRT port that srtla_send listens on (for injecting test data).
    pub fn sender_srt_port(&self) -> u16 {
        SRTLA_SEND_SRT_PORT
    }

    /// The receiver-side srtla_rec port the sender's uplink sockets connect to.
    pub fn receiver_srtla_port(&self) -> u16 {
        SRTLA_REC_PORT
    }

    /// Stop all processes and collect their output.
    pub fn stop(&mut self) -> StackOutput {
        let mut send_out = (vec![], vec![]);
        let mut rec_out = (vec![], vec![]);
        let mut srt_out = (vec![], vec![]);

        // Kill in reverse order: sender → receiver → srt server
        if let Some(mut p) = self.srtla_send.take() {
            p.kill();
            send_out = (p.stdout_lines(), p.stderr_lines());
        }
        if let Some(mut p) = self.srtla_rec.take() {
            p.kill();
            rec_out = (p.stdout_lines(), p.stderr_lines());
        }
        if let Some(mut p) = self.srt_server.take() {
            p.kill();
            srt_out = (p.stdout_lines(), p.stderr_lines());
        }

        StackOutput {
            srt_server_stdout: srt_out.0,
            srt_server_stderr: srt_out.1,
            srtla_rec_stdout: rec_out.0,
            srtla_rec_stderr: rec_out.1,
            srtla_send_stdout: send_out.0,
            srtla_send_stderr: send_out.1,
        }
    }
}

impl Drop for SrtlaTestStack {
    fn drop(&mut self) {
        // Ensure all processes are killed even if stop() wasn't called.
        // Dropping NamespaceProcess triggers its Drop impl which calls kill().
        drop(self.srtla_send.take());
        drop(self.srtla_rec.take());
        drop(self.srt_server.take());
    }
}

// ---------------------------------------------------------------------------
// UDP injection
// ---------------------------------------------------------------------------

/// Inject `count` UDP packets into a port inside `ns`.
///
/// Sends from within the namespace using a bound local socket. Each packet
/// is 188 bytes (MPEG-TS packet size) of zeroes to simulate SRT data.
pub fn inject_udp_packets(ns: &Namespace, target_ip: &str, port: u16, count: usize) -> Result<()> {
    let addr = format!("{target_ip}:{port}");
    let script = format!(
        "import socket; s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM); \
         [s.sendto(b'\\x00'*188,('{target_ip}',{port})) for _ in range({count})]; s.close()"
    );
    ns.exec_checked("python3", &["-c", &script])
        .with_context(|| format!("inject {count} UDP packets to {addr}"))?;
    Ok(())
}

/// Inject UDP datagrams from a SPECIFIC source address.
///
/// Lets a test reproduce a multi-homed receiver replying from another of its own
/// addresses — the interop case unconnected uplink sockets exist for.
pub fn inject_udp_packets_from(
    ns: &Namespace,
    source_ip: &str,
    source_port: u16,
    target_ip: &str,
    target_port: u16,
    payload_len: usize,
    count: usize,
) -> Result<()> {
    let script = format!(
        "import socket; s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM); \
         s.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1); \
         s.bind(('{source_ip}',{source_port})); \
         [s.sendto(b'\\x00'*{payload_len},('{target_ip}',{target_port})) for _ in \
         range({count})]; s.close()"
    );
    ns.exec_checked("python3", &["-c", &script])
        .with_context(|| {
            format!(
                "inject {count} UDP packets {source_ip}:{source_port} -> {target_ip}:{target_port}"
            )
        })?;
    Ok(())
}

/// Local UDP ports bound to `local_ip` inside `ns`, newest listing order.
///
/// Uplink sockets are unconnected, so `ss` reports them as `UNCONN` with no
/// peer; this reads the ephemeral source port a test needs to address them.
pub fn bound_udp_ports(ns: &Namespace, local_ip: &str) -> Result<Vec<u16>> {
    let out = ns.exec("ss", &["-uan"])?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let needle = format!("{local_ip}:");
    let mut ports = Vec::new();
    for line in stdout.lines() {
        for field in line.split_whitespace() {
            if let Some(port) = field.strip_prefix(&needle)
                && let Ok(parsed) = port.parse::<u16>()
            {
                ports.push(parsed);
            }
        }
    }
    Ok(ports)
}

/// Inject UDP packets at a steady rate (packets/sec) for `duration`.
pub fn inject_udp_stream(
    ns: &Namespace,
    target_ip: &str,
    port: u16,
    packets_per_sec: u32,
    duration: Duration,
) -> Result<()> {
    if packets_per_sec == 0 {
        bail!("packets_per_sec must be > 0");
    }
    let interval_us = 1_000_000 / packets_per_sec;
    let dur_secs = duration.as_secs_f64();

    let script = format!(
        r#"import socket,time
s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM)
d=b'\x00'*188
start=time.monotonic(); i=0
while time.monotonic()-start<{dur_secs}:
    s.sendto(d,('{target_ip}',{port}))
    i+=1
    time.sleep({interval_us}/1e6)
s.close()
print(f'sent {{i}} packets')"#
    );
    ns.exec_checked("python3", &["-c", &script])
        .context("inject UDP stream")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve an explicit receiver path, or the `srtla_rec` PATH entry when unset.
pub fn find_srtla_rec_binary() -> Result<PathBuf> {
    resolve_external_binary("SRTLA_REC_BIN", "srtla_rec")
}

/// Resolve an explicit SRT tool path, or `srt-live-transmit` when unset.
pub fn find_srt_live_transmit_binary() -> Result<PathBuf> {
    resolve_external_binary("SRT_LIVE_TRANSMIT_BIN", "srt-live-transmit")
}

fn resolve_external_binary(key: &str, fallback: &str) -> Result<PathBuf> {
    match std::env::var_os(key) {
        Some(explicit) => {
            let path = PathBuf::from(explicit);
            if !path.is_file() {
                bail!("{key}={} is not a file", path.display());
            }
            std::fs::canonicalize(&path)
                .with_context(|| format!("resolve {key}={}", path.display()))
        }
        None => check_binary(fallback)
            .with_context(|| format!("{fallback} not found in PATH ({key} unset)")),
    }
}

#[cfg(test)]
mod srt_profile_tests {
    use super::*;

    #[test]
    #[ignore = "requires netns privileges, real SRT tools and a built srtla_send"]
    fn profile_stats_stack_writes_csv() {
        // Given a registered single-link stack with explicit strict tuning and capture.
        check_integration_deps().expect("live stack dependencies");
        let dir = tempfile::tempdir().expect("capture directory");
        let csv = dir.path().join("listener stats.csv");
        let mut stack =
            SrtlaTestStack::start("profile_csv", 1, &[], SrtProfile::STRICT, Some(csv.clone()))
                .expect("profile stack");
        stack
            .wait_for_registered_uplinks(1, Duration::from_secs(20))
            .expect("registered uplink");
        let tool = find_srt_live_transmit_binary().expect("SRT tool");
        let caller_uri = format!(
            "srt://127.0.0.1:{}?mode=caller&latency=500",
            stack.sender_srt_port()
        );
        let caller = NamespaceProcess::spawn(
            &stack.topo.sender_ns,
            tool.to_str().expect("tool path"),
            &["udp://:6000", &caller_uri],
        )
        .expect("real SRT caller");
        wait_for_udp_listener(&stack.topo.sender_ns, 6000, Duration::from_secs(10))
            .expect("caller input ready");
        // When real libsrt carries enough messages to cross its stats packet count.
        // A burst can overflow the caller's UDP receive buffer before SRT reads 1000 messages.
        inject_udp_stream(
            &stack.topo.sender_ns,
            "127.0.0.1",
            6000,
            500,
            Duration::from_secs(6),
        )
        .expect("source datagrams");
        // Then the listener publishes real CSV, including a data row rather than just a header.
        let deadline = Instant::now() + Duration::from_secs(10);
        let captured = loop {
            let text = std::fs::read_to_string(&csv).expect("read listener stats CSV");
            if text.ends_with('\n') && text.lines().count() >= 2 {
                break text;
            }
            assert!(
                Instant::now() < deadline,
                "no stats rows; listener={:?}; caller={:?}; sender={:?}",
                stack.srt_server.as_ref().expect("listener").log_snapshot(),
                caller.log_snapshot(),
                stack.sender_log_snapshot()
            );
            std::thread::sleep(Duration::from_millis(100));
        };
        let mut lines = captured.lines();
        let header = lines.next().expect("CSV header");
        let recv_column = header
            .split(',')
            .position(|field| field == "pktRecv")
            .expect("pktRecv column");
        let received = lines
            .next()
            .expect("CSV data row")
            .split(',')
            .nth(recv_column)
            .expect("pktRecv value")
            .parse::<u64>()
            .expect("numeric pktRecv");
        assert!(received > 0, "listener must report real received packets");
        eprintln!(
            "listener CSV captured: pktRecv={received}, data_rows={}, path={}",
            captured.lines().count() - 1,
            csv.display()
        );
        stack.stop();
        drop(caller);
    }

    #[test]
    fn srt_uri_includes_profile_params() {
        for (profile, expected) in [
            (
                SrtProfile::PRODUCTION,
                "srt://:4001?mode=listener&latency=2000&lossmaxttl=40&reorderfreeze=1",
            ),
            (
                SrtProfile::STRICT,
                "srt://:4001?mode=listener&latency=500&lossmaxttl=10",
            ),
        ] {
            // Given an explicit preset; when composing the listener URI.
            let uri = profile.listener_uri(4001);
            // Then tuning is exact: production freezes reordering without disabling NAK reports.
            assert_eq!(uri, expected);
        }
    }

    #[test]
    fn legacy_default_uri_unchanged() {
        // Given legacy behavior without stats capture; when building the invocation.
        let args = SrtProfile::LEGACY_DEFAULT
            .listener_argv(4001, None)
            .expect("legacy args");
        // Then the entire listener argument vector is byte-identical to the old stack.
        assert_eq!(args, ["srt://:4001?mode=listener", "udp://127.0.0.1:9999"]);
    }

    #[test]
    fn stats_capture_uses_csv_and_packet_count_flags() {
        // Given an explicit capture path containing a space; when building the invocation.
        let args = SrtProfile::PRODUCTION
            .listener_argv(4001, Some(std::path::Path::new("capture dir/stats.csv")))
            .expect("stats args");
        // Then capture is one path argument and the cadence is 1000 packets, not milliseconds.
        assert_eq!(
            args,
            [
                "-statsout",
                "capture dir/stats.csv",
                "-statspf:csv",
                "-stats",
                "1000",
                "srt://:4001?mode=listener&latency=2000&lossmaxttl=40&reorderfreeze=1",
                "udp://127.0.0.1:9999"
            ]
        );
    }

    #[test]
    fn legacy_profile_can_capture_stats_without_tuning_latency() {
        // Given a legacy profile with opt-in capture; when composing the invocation.
        let args = SrtProfile::LEGACY_DEFAULT
            .listener_argv(1234, Some(std::path::Path::new("stats.csv")))
            .expect("legacy stats args");
        // Then capture does not opt into either profile knob.
        assert_eq!(
            args,
            [
                "-statsout",
                "stats.csv",
                "-statspf:csv",
                "-stats",
                "1000",
                "srt://:1234?mode=listener",
                "udp://127.0.0.1:9999"
            ]
        );
    }
}

#[cfg(test)]
mod launch_policy_tests {
    use super::*;

    #[test]
    fn belabox_argv_is_positional() {
        // Given a BELABOX receiver and distinct listener/forwarding ports.
        let receiver = ReceiverKind::Belabox;
        // When composing its invocation.
        let args = receiver.argv(15000, "127.0.0.2", 15001);
        // Then only the three ordered positionals are emitted.
        assert_eq!(args, ["15000", "127.0.0.2", "15001"]);
    }

    #[test]
    fn ceralive_and_irlserver_argv_use_flags() {
        for receiver in [ReceiverKind::CeraLive, ReceiverKind::Irlserver] {
            // Given either flags-based receiver; when composing its invocation.
            let args = receiver.argv(15000, "127.0.0.2", 15001);
            // Then the receiver's own CLI names and order are preserved.
            assert_eq!(
                args,
                [
                    "--srtla_port",
                    "15000",
                    "--srt_hostname",
                    "127.0.0.2",
                    "--srt_port",
                    "15001"
                ]
            );
        }
    }

    #[test]
    fn receiver_kind_parses_known_values_and_legacy_default() {
        for (value, expected) in [
            (None, ReceiverKind::CeraLive),
            (Some("ceralive"), ReceiverKind::CeraLive),
            (Some("irlserver"), ReceiverKind::Irlserver),
            (Some("belabox"), ReceiverKind::Belabox),
        ] {
            // Given a supported environment value; when parsing it.
            let kind = ReceiverKind::parse(value.map(std::ffi::OsStr::new));
            // Then it selects the corresponding CLI dialect.
            assert_eq!(kind.expect("known receiver kind"), expected);
        }
    }

    #[test]
    fn receiver_kind_rejects_unknown_values() {
        // Given a typo rather than a supported dialect; when parsing it.
        let error = ReceiverKind::parse(Some(std::ffi::OsStr::new("belaboxx"))).unwrap_err();
        // Then the environment setting is diagnosed instead of silently defaulted.
        assert!(error.to_string().contains("SRTLA_REC_KIND"));
    }

    #[test]
    fn env_override_wins_over_path() {
        // Given isolated child-process environment overrides (no global env mutation).
        let executable = std::env::current_exe().expect("test executable");
        if std::env::var_os("NETWORK_SIM_OVERRIDE_CHILD").is_some() {
            // When resolving the real environment boundary.
            let rec = find_srtla_rec_binary().expect("receiver override");
            let srt = find_srt_live_transmit_binary().expect("SRT tool override");
            let kind = ReceiverKind::from_env().expect("receiver kind override");
            // Then explicit paths win even with no usable PATH.
            assert_eq!(rec, executable);
            assert_eq!(srt, executable);
            assert_eq!(kind, ReceiverKind::Belabox);
            return;
        }
        let output = Command::new(&executable)
            .args([
                "--exact",
                "harness::launch_policy_tests::env_override_wins_over_path",
            ])
            .env("NETWORK_SIM_OVERRIDE_CHILD", "1")
            .env("SRTLA_REC_BIN", &executable)
            .env("SRT_LIVE_TRANSMIT_BIN", &executable)
            .env("SRTLA_REC_KIND", "belabox")
            .env("PATH", "")
            .output()
            .expect("run isolated override test");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    #[test]
    fn env_override_missing_file_is_reported() {
        // Given a missing explicit path and an otherwise available fallback.
        let dir = tempfile::tempdir().expect("isolated missing binary path");
        let missing = dir.path().join("missing-receiver");
        if let Some(path) = std::env::var_os("NETWORK_SIM_MISSING_CHILD") {
            // When resolving, no PATH fallback is allowed for an explicit override.
            let error = resolve_external_binary("SRTLA_REC_BIN", "sh").unwrap_err();
            // Then diagnostics retain both the environment key and missing filename.
            let message = error.to_string();
            assert!(message.contains("SRTLA_REC_BIN"), "{message}");
            assert!(
                message.contains(&path.to_string_lossy().to_string()),
                "{message}"
            );
            assert!(message.contains("not a file"), "{message}");
            return;
        }
        let output = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "harness::launch_policy_tests::env_override_missing_file_is_reported",
            ])
            .env("NETWORK_SIM_MISSING_CHILD", &missing)
            .env("SRTLA_REC_BIN", &missing)
            .output()
            .expect("run isolated missing override test");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    #[test]
    fn absent_override_resolves_path_binary() {
        // Given an isolated environment without the override.
        if std::env::var_os("NETWORK_SIM_PATH_CHILD").is_some() {
            // When resolving a standard tool through the same resolver.
            let binary = resolve_external_binary("SRTLA_REC_BIN", "sh").expect("PATH fallback");
            // Then the normal PATH resolution is retained.
            assert_eq!(Some(binary), check_binary("sh"));
            return;
        }
        let output = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "harness::launch_policy_tests::absent_override_resolves_path_binary",
            ])
            .env("NETWORK_SIM_PATH_CHILD", "1")
            .env_remove("SRTLA_REC_BIN")
            .output()
            .expect("run isolated PATH fallback test");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

/// Locate the srtla_send binary from a cargo build.
pub(crate) fn find_srtla_send_binary() -> Result<PathBuf> {
    // An explicit path always wins. `tests/common/build_srtla_send` publishes
    // `CARGO_BIN_EXE_srtla_send` here, which is the only location that is
    // correct under a redirected CARGO_TARGET_DIR — the hardcoded
    // `<workspace>/target/...` candidates below silently resolve to a STALE
    // binary in that case, which would invalidate every netns observation.
    if let Ok(explicit) = std::env::var("SRTLA_SEND_BIN") {
        let path = PathBuf::from(explicit);
        if path.exists() {
            return Ok(path);
        }
    }

    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        for profile in ["debug", "release"] {
            let path = PathBuf::from(&target_dir).join(profile).join("srtla_send");
            if path.exists() {
                return Ok(path);
            }
        }
    }

    // Check common cargo build output locations
    let candidates = [
        // Debug build
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("target/debug/srtla_send"),
        // Release build
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("target/release/srtla_send"),
    ];

    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    // Fall back to PATH
    check_binary("srtla_send").ok_or_else(|| {
        anyhow::anyhow!(
            "srtla_send binary not found. Run `cargo build` first. Checked: {:?}",
            candidates
        )
    })
}
