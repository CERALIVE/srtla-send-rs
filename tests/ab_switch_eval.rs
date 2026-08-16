//! Paired A/B evaluation runner for the switch-cooldown / flush-on-switch
//! upstream candidates (`0cc0c6d`, `24b5f64`).
//!
//! Both tests are `#[ignore]`d: they are a measurement harness, not a gate, and
//! a single invocation takes tens of minutes. Run explicitly with
//!
//! ```text
//! timeout --foreground --kill-after=30s 1800s \
//!   cargo test --features test-internals --test ab_switch_eval -- --ignored --nocapture
//! ```
//!
//! `test-internals` is REQUIRED: the switch/NAK/cooldown counters and the
//! `metrics` runtime command that reads them live behind that feature.
//!
//! # What is measured
//!
//! A real SRT path inside a network-namespace topology:
//! `srt-live-transmit` (caller) → `srtla_send` (bonded) → `srtla_rec` →
//! `srt-live-transmit` (listener) → UDP sink. Goodput is the sink byte count
//! over a fixed steady-state window; the retransmit proxy and the treatment
//! activation signals come from `srtla_send`'s own monotonic counters, read via
//! the control socket at window start and window end and differenced, so
//! registration and warm-up traffic are excluded by construction.
//!
//! # Frozen scenario (no runtime-chosen parameters)
//!
//! 3 bonded veth uplinks; per-link netem FIXED delay 20/60/120 ms with no loss
//! and no jitter (a randomized impairment would make the paired comparison
//! non-deterministic); per-link TBF rate limit 4 Mbit; netem queue limit at its
//! default; offered load a constant 6 Mbit stream of 1316-byte payloads;
//! warm-up of 5 s after registration readiness; a 30 s measurement window; 3
//! paired runs per mode with the variants alternating baseline, candidate,
//! baseline, candidate, ….
//!
//! # Binaries
//!
//! The two variants are IMMUTABLE pre-built binaries passed by path, never a
//! shared target directory that is rebuilt between runs (a rebuild racing a
//! measurement window produces numbers that lie). Build them first:
//!
//! ```text
//! CARGO_TARGET_DIR=/tmp/srtla-ab-baseline  cargo build --release --features test-internals
//! # ... apply the step, then ...
//! CARGO_TARGET_DIR=/tmp/srtla-ab-candidate cargo build --release --features test-internals
//! ```
//!
//! and point the runner at them:
//! `AB_STEP_A_BASELINE_BIN` / `AB_STEP_A_CANDIDATE_BIN` (step A),
//! `AB_STEP_B_BASELINE_BIN` / `AB_STEP_B_CANDIDATE_BIN` (step B).
//! A step whose two variables are unset is skipped.

#![cfg(unix)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use network_sim::{
    ImpairmentConfig, NamespaceProcess, SrtlaTestTopology, check_binary, check_privileges,
    wait_for_registered_uplinks, wait_for_udp_listener,
};

const LINKS: usize = 3;
const DELAYS_MS: [u32; LINKS] = [20, 60, 120];
const LINK_RATE_KBIT: u64 = 4000;
const OFFERED_KBIT: u64 = 6000;
const PAYLOAD_BYTES: usize = 1316;
const WARMUP_SECS: u64 = 5;
const WINDOW_SECS: u64 = 30;
const PAIRED_RUNS: usize = 3;

const LOCAL_SRT_PORT: u16 = 5560;
const CALLER_UDP_PORT: u16 = 7100;
const SRTLA_PORT: u16 = 5000;
const SRT_PORT: u16 = 4003;
const SINK_PORT: u16 = 9999;

const STEP_A_MODES: [&str; 4] = ["enhanced", "classic", "rtt-threshold", "edpf"];
const STEP_B_MODES: [&str; 2] = ["enhanced", "rtt-threshold"];

/// Adoption rule, pre-committed and applied mechanically per mode.
const GOODPUT_FLOOR_RATIO: f64 = 0.99;
const NAK_CEILING_RATIO: f64 = 1.05;
const NAK_CEILING_ABSOLUTE_SLACK: f64 = 5.0;
const SWITCH_THRASH_MULTIPLIER: f64 = 3.0;

fn offered_pps() -> u64 {
    OFFERED_KBIT * 1000 / (PAYLOAD_BYTES as u64 * 8)
}

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_secs_f64()
}

/// Measurement runs must never overlap: CPU contention between two live
/// namespaces silently corrupts throughput numbers while both runs still
/// "succeed".
fn measurement_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn deps_ok(step: &str) -> bool {
    if !check_privileges() {
        eprintln!("Skipping {step}: requires root / passwordless sudo");
        return false;
    }
    for bin in ["srtla_rec", "srt-live-transmit", "tcpdump", "python3"] {
        if check_binary(bin).is_none() {
            eprintln!("Skipping {step}: '{bin}' not found in PATH");
            return false;
        }
    }
    let netem_ok = Command::new("sudo")
        .args(["modprobe", "sch_netem"])
        .output()
        .is_ok_and(|o| o.status.success());
    if !netem_ok {
        eprintln!("Skipping {step}: sch_netem unavailable");
        return false;
    }
    true
}

// ---------------------------------------------------------------------------
// Counters read from the sender's control socket
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default)]
struct Counters {
    switch_count: u64,
    nak_count: u64,
    cooldown_hold_count: u64,
}

impl Counters {
    fn delta(self, start: Self) -> Self {
        Self {
            switch_count: self.switch_count.saturating_sub(start.switch_count),
            nak_count: self.nak_count.saturating_sub(start.nak_count),
            cooldown_hold_count: self
                .cooldown_hold_count
                .saturating_sub(start.cooldown_hold_count),
        }
    }
}

fn query_metrics(socket_path: &str) -> Counters {
    let mut stream = match UnixStream::connect(socket_path) {
        Ok(s) => s,
        Err(e) => panic!("connect control socket {socket_path}: {e}"),
    };
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set read timeout");
    stream.write_all(b"metrics\n").expect("write metrics query");
    stream.flush().expect("flush metrics query");

    let mut line = String::new();
    BufReader::new(&stream)
        .read_line(&mut line)
        .expect("read metrics reply");
    let parsed: serde_json::Value = serde_json::from_str(line.trim()).unwrap_or_else(|e| {
        panic!(
            "metrics reply is not JSON ({e}); did you build with --features test-internals? \
             {line:?}"
        )
    });
    let field = |k: &str| {
        parsed[k]
            .as_u64()
            .unwrap_or_else(|| panic!("metrics reply missing {k}: {line:?}"))
    };
    Counters {
        switch_count: field("switch_count"),
        nak_count: field("nak_count"),
        cooldown_hold_count: field("cooldown_hold_count"),
    }
}

// ---------------------------------------------------------------------------
// One measured run
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct RunMetrics {
    sink_bytes: u64,
    switch_count: u64,
    nak_count: u64,
    cooldown_hold_count: u64,
}

/// Owns every process and namespace of one run so a panic still tears down.
struct Stack {
    topo: SrtlaTestTopology,
    _srt_listener: NamespaceProcess,
    _srtla_rec: NamespaceProcess,
    srtla_send: NamespaceProcess,
    _caller: NamespaceProcess,
    control_socket: String,
}

impl Drop for Stack {
    fn drop(&mut self) {
        kill_netns_pids(&self.topo.sender_ns.name);
        kill_netns_pids(&self.topo.receiver_ns.name);
        let _ = std::fs::remove_file(&self.control_socket);
    }
}

fn kill_netns_pids(ns_name: &str) {
    let Ok(out) = Command::new("sudo")
        .args(["ip", "netns", "pids", ns_name])
        .output()
    else {
        return;
    };
    let pids: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .map(String::from)
        .collect();
    if !pids.is_empty() {
        let mut args = vec!["kill".to_string(), "-9".to_string()];
        args.extend(pids);
        let _ = Command::new("sudo").args(&args).output();
    }
}

/// Give every uplink beyond link 0 its own egress path and a symmetric reply
/// route. Without this, all three bound source IPs resolve the receiver through
/// link 0's subnet and only one link ever carries traffic — the bond would be
/// nominal and the A/B comparison meaningless.
fn configure_bond_routing(topo: &SrtlaTestTopology) {
    for iface in ["all", "default"]
        .into_iter()
        .chain(topo.sender_ifaces.iter().map(String::as_str))
    {
        let _ = topo.sender_ns.exec(
            "sysctl",
            &["-w", &format!("net.ipv4.conf.{iface}.rp_filter=0")],
        );
    }
    for iface in ["all", "default"]
        .into_iter()
        .chain(topo.receiver_ifaces.iter().map(String::as_str))
    {
        let _ = topo.receiver_ns.exec(
            "sysctl",
            &["-w", &format!("net.ipv4.conf.{iface}.rp_filter=0")],
        );
    }

    let recv_ip = topo.receiver_ip.clone();
    for idx in 1..topo.sender_ips.len() {
        let table = (101 + idx).to_string();
        let src = topo.sender_ips[idx].as_str();
        let sif = topo.sender_ifaces[idx].as_str();
        let rif = topo.receiver_ifaces[idx].as_str();
        let _ = topo.sender_ns.exec(
            "ip",
            &["route", "add", &recv_ip, "dev", sif, "table", &table],
        );
        let _ = topo
            .sender_ns
            .exec("ip", &["rule", "add", "from", src, "lookup", &table]);
        let _ = topo
            .receiver_ns
            .exec("ip", &["route", "add", src, "dev", rif, "src", &recv_ip]);
    }
}

fn start_stack(name: &str, bin: &str, mode: &str) -> Stack {
    let topo = SrtlaTestTopology::new(name, LINKS).expect("create topology");
    configure_bond_routing(&topo);

    for (idx, delay_ms) in DELAYS_MS.iter().enumerate() {
        topo.impair_link(
            idx,
            ImpairmentConfig {
                delay_ms: Some(*delay_ms),
                rate_kbit: Some(LINK_RATE_KBIT),
                tbf_shaping: true,
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("impair link {idx}: {e}"));
    }

    let srt_uri = format!("srt://:{SRT_PORT}?mode=listener");
    let sink_uri = format!("udp://127.0.0.1:{SINK_PORT}");
    let srt_listener = NamespaceProcess::spawn(
        &topo.receiver_ns,
        "srt-live-transmit",
        &[&srt_uri, &sink_uri],
    )
    .expect("start srt-live-transmit listener");
    wait_for_udp_listener(&topo.receiver_ns, SRT_PORT, Duration::from_secs(10))
        .expect("srt-live-transmit listener");

    let srtla_port = SRTLA_PORT.to_string();
    let srt_port = SRT_PORT.to_string();
    let srtla_rec = NamespaceProcess::spawn(
        &topo.receiver_ns,
        "srtla_rec",
        &[
            "--srtla_port",
            &srtla_port,
            "--srt_hostname",
            "127.0.0.1",
            "--srt_port",
            &srt_port,
        ],
    )
    .expect("start srtla_rec");
    wait_for_udp_listener(&topo.receiver_ns, SRTLA_PORT, Duration::from_secs(10))
        .expect("srtla_rec listener");

    let ips = topo.write_ip_list().expect("write ip list");
    let local_port = LOCAL_SRT_PORT.to_string();
    let control_socket = format!(
        "/tmp/ab_switch_eval_{}_{}.sock",
        std::process::id(),
        topo.sender_ns.name
    );
    let _ = std::fs::remove_file(&control_socket);

    let srtla_send = NamespaceProcess::spawn_with_env(
        &topo.sender_ns,
        bin,
        &[
            local_port.as_str(),
            topo.receiver_ip.as_str(),
            srtla_port.as_str(),
            ips.to_str().expect("ips path"),
            "--mode",
            mode,
            "--control-socket",
            control_socket.as_str(),
        ],
        &[("RUST_LOG", "info")],
    )
    .expect("start srtla_send");

    wait_for_udp_listener(&topo.sender_ns, LOCAL_SRT_PORT, Duration::from_secs(10))
        .expect("srtla_send local SRT listener");
    wait_for_registered_uplinks(&srtla_send, LINKS, Duration::from_secs(30))
        .expect("all uplinks registered");

    // srtla_send runs as root under `ip netns exec`, so its control socket is
    // not writable by the (unprivileged) test process without this.
    let _ = Command::new("sudo")
        .args(["chmod", "0666", &control_socket])
        .output();

    let caller_src = format!("udp://:{CALLER_UDP_PORT}");
    let caller_dst = format!("srt://127.0.0.1:{LOCAL_SRT_PORT}?mode=caller");
    let caller = NamespaceProcess::spawn(
        &topo.sender_ns,
        "srt-live-transmit",
        &[&caller_src, &caller_dst],
    )
    .expect("start srt-live-transmit caller");
    sleep(Duration::from_secs(2));

    Stack {
        topo,
        _srt_listener: srt_listener,
        _srtla_rec: srtla_rec,
        srtla_send,
        _caller: caller,
        control_socket,
    }
}

struct SinkCapture {
    child: Child,
    path: PathBuf,
    ns: String,
}

fn start_sink_capture(ns_name: &str) -> SinkCapture {
    let path = std::env::temp_dir().join(format!("absink_{}_{ns_name}.txt", std::process::id()));
    let file = std::fs::File::create(&path).expect("create sink capture file");
    let filter = format!("udp and dst port {SINK_PORT}");
    let child = Command::new("sudo")
        .args([
            "ip", "netns", "exec", ns_name, "tcpdump", "-i", "lo", "-tt", "-n", "-q", &filter,
        ])
        .stdout(Stdio::from(file))
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sink tcpdump");
    sleep(Duration::from_millis(500));
    SinkCapture {
        child,
        path,
        ns: ns_name.to_string(),
    }
}

/// Sum the UDP payload bytes the sink received inside `[lo, hi]`.
///
/// `tcpdump -tt -n -q` emits one line per packet beginning with the epoch
/// timestamp and ending with `UDP, length <n>`, so both the window filter and
/// the byte count come from the same record.
fn stop_sink_capture(mut cap: SinkCapture, lo: f64, hi: f64) -> u64 {
    let _ = Command::new("sudo")
        .args([
            "ip",
            "netns",
            "exec",
            &cap.ns,
            "pkill",
            "-TERM",
            "-f",
            "tcpdump -i lo",
        ])
        .output();
    sleep(Duration::from_millis(500));
    let _ = cap.child.wait();
    let text = std::fs::read_to_string(&cap.path).unwrap_or_default();
    let _ = std::fs::remove_file(&cap.path);

    text.lines()
        .filter_map(|line| {
            let ts: f64 = line.split_whitespace().next()?.parse().ok()?;
            if ts < lo || ts > hi {
                return None;
            }
            let len: u64 = line.rsplit_once("length ")?.1.trim().parse().ok()?;
            Some(len)
        })
        .sum()
}

fn spawn_injector(ns_name: &str, secs: f64) -> Child {
    let pps = offered_pps();
    let script = format!(
        r#"
import socket, time  # ABSWITCHINJ
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
pay = bytes({PAYLOAD_BYTES})
t0 = time.time()
while time.time() - t0 < {secs}:
    s.sendto(pay, ('127.0.0.1', {CALLER_UDP_PORT}))
    time.sleep(1 / {pps})
s.close()
"#
    );
    Command::new("sudo")
        .args(["ip", "netns", "exec", ns_name, "python3", "-c", &script])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn injector")
}

fn stop_injector(mut child: Child) {
    let _ = Command::new("sudo")
        .args(["pkill", "-TERM", "-f", "ABSWITCHINJ"])
        .output();
    let _ = child.wait();
}

fn run_once(bin: &str, mode: &str, label: &str) -> RunMetrics {
    let _guard = measurement_lock();
    let ns_tag = format!("ab{}", mode.replace('-', ""));
    let stack = start_stack(&ns_tag, bin, mode);

    let injector = spawn_injector(
        &stack.topo.sender_ns.name,
        (WARMUP_SECS + WINDOW_SECS + 5) as f64,
    );
    let capture = start_sink_capture(&stack.topo.receiver_ns.name);

    sleep(Duration::from_secs(WARMUP_SECS));
    let start_counters = query_metrics(&stack.control_socket);
    let t_start = now();
    sleep(Duration::from_secs(WINDOW_SECS));
    let t_end = now();
    let end_counters = query_metrics(&stack.control_socket);

    let sink_bytes = stop_sink_capture(capture, t_start, t_end);
    stop_injector(injector);

    let delta = end_counters.delta(start_counters);
    let metrics = RunMetrics {
        sink_bytes,
        switch_count: delta.switch_count,
        nak_count: delta.nak_count,
        cooldown_hold_count: delta.cooldown_hold_count,
    };

    println!(
        "[ab] {label} mode={mode} goodput_bytes={} goodput_mbps={:.3} switches={} naks={} \
         cooldown_holds={}",
        metrics.sink_bytes,
        (metrics.sink_bytes as f64 * 8.0) / (WINDOW_SECS as f64 * 1e6),
        metrics.switch_count,
        metrics.nak_count,
        metrics.cooldown_hold_count
    );

    assert!(
        metrics.sink_bytes > 0,
        "{label} mode={mode}: sink received nothing — the measurement path is broken, not the \
         treatment.\nsrtla_send log tail:\n{}",
        stack.srtla_send.log_snapshot().join("\n")
    );

    metrics
}

// ---------------------------------------------------------------------------
// Verdict
// ---------------------------------------------------------------------------

fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let v: Vec<f64> = values.collect();
    if v.is_empty() {
        return 0.0;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

#[derive(Debug)]
struct ModeOutcome {
    mode: String,
    baseline: Vec<RunMetrics>,
    candidate: Vec<RunMetrics>,
}

impl ModeOutcome {
    fn mean_goodput(runs: &[RunMetrics]) -> f64 {
        mean(runs.iter().map(|r| r.sink_bytes as f64))
    }
    fn mean_naks(runs: &[RunMetrics]) -> f64 {
        mean(runs.iter().map(|r| r.nak_count as f64))
    }
    fn mean_switches(runs: &[RunMetrics]) -> f64 {
        mean(runs.iter().map(|r| r.switch_count as f64))
    }

    fn goodput_ok(&self) -> bool {
        Self::mean_goodput(&self.candidate)
            >= GOODPUT_FLOOR_RATIO * Self::mean_goodput(&self.baseline)
    }

    fn nak_ok(&self) -> bool {
        let base = Self::mean_naks(&self.baseline);
        Self::mean_naks(&self.candidate)
            <= (base * NAK_CEILING_RATIO).max(base + NAK_CEILING_ABSOLUTE_SLACK)
    }

    fn switch_ok(&self) -> bool {
        Self::mean_switches(&self.candidate)
            <= SWITCH_THRASH_MULTIPLIER * Self::mean_switches(&self.baseline)
    }

    fn passes_adoption_rule(&self) -> bool {
        self.goodput_ok() && self.nak_ok() && self.switch_ok()
    }

    fn switch_activated(&self) -> bool {
        self.baseline
            .iter()
            .chain(self.candidate.iter())
            .all(|r| r.switch_count >= 1)
    }

    fn cooldown_activated(&self) -> bool {
        self.baseline.iter().all(|r| r.cooldown_hold_count >= 1)
    }
}

fn print_table(step: &str, outcomes: &[ModeOutcome]) {
    println!("\n=== {step}: per-run metrics ===");
    println!(
        "{:<14} {:<9} {:<4} {:>14} {:>10} {:>9} {:>9}",
        "mode", "variant", "run", "goodput_bytes", "mbps", "switches", "naks"
    );
    for o in outcomes {
        for (variant, runs) in [("baseline", &o.baseline), ("candidate", &o.candidate)] {
            for (i, r) in runs.iter().enumerate() {
                println!(
                    "{:<14} {:<9} {:<4} {:>14} {:>10.3} {:>9} {:>9}",
                    o.mode,
                    variant,
                    i + 1,
                    r.sink_bytes,
                    (r.sink_bytes as f64 * 8.0) / (WINDOW_SECS as f64 * 1e6),
                    r.switch_count,
                    r.nak_count
                );
            }
        }
    }

    println!("\n=== {step}: means + adoption rule ===");
    for o in outcomes {
        println!(
            "{:<14} goodput {:.0} -> {:.0} ({:.4}x, ok={})  naks {:.1} -> {:.1} (ok={})  switches \
             {:.1} -> {:.1} (ok={})  switch_activated={} cooldown_activated={}",
            o.mode,
            ModeOutcome::mean_goodput(&o.baseline),
            ModeOutcome::mean_goodput(&o.candidate),
            if ModeOutcome::mean_goodput(&o.baseline) > 0.0 {
                ModeOutcome::mean_goodput(&o.candidate) / ModeOutcome::mean_goodput(&o.baseline)
            } else {
                0.0
            },
            o.goodput_ok(),
            ModeOutcome::mean_naks(&o.baseline),
            ModeOutcome::mean_naks(&o.candidate),
            o.nak_ok(),
            ModeOutcome::mean_switches(&o.baseline),
            ModeOutcome::mean_switches(&o.candidate),
            o.switch_ok(),
            o.switch_activated(),
            o.cooldown_activated(),
        );
    }
}

/// Alternating baseline, candidate, baseline, candidate, … per mode.
fn run_matrix(modes: &[&str], baseline_bin: &str, candidate_bin: &str) -> Vec<ModeOutcome> {
    let mut outcomes = Vec::new();
    for mode in modes {
        let mut baseline = Vec::new();
        let mut candidate = Vec::new();
        for run in 0..PAIRED_RUNS {
            baseline.push(run_once(
                baseline_bin,
                mode,
                &format!("baseline run{}", run + 1),
            ));
            candidate.push(run_once(
                candidate_bin,
                mode,
                &format!("candidate run{}", run + 1),
            ));
        }
        outcomes.push(ModeOutcome {
            mode: (*mode).to_string(),
            baseline,
            candidate,
        });
    }
    outcomes
}

fn binaries(step: &str) -> Option<(String, String)> {
    let baseline = std::env::var(format!("AB_STEP_{step}_BASELINE_BIN")).ok()?;
    let candidate = std::env::var(format!("AB_STEP_{step}_CANDIDATE_BIN")).ok()?;
    for path in [&baseline, &candidate] {
        assert!(
            PathBuf::from(path).exists(),
            "configured binary does not exist: {path}"
        );
    }
    Some((baseline, candidate))
}

/// One frozen-scenario run against a single binary, to prove the measurement
/// path itself works before spending tens of minutes on a full matrix: real SRT
/// traffic reaches the sink, and the control socket answers `metrics`.
#[test]
#[ignore = "A/B measurement harness: needs netns privileges"]
fn measurement_path_smoke() {
    if !deps_ok("measurement_path_smoke") {
        return;
    }
    let Ok(bin) = std::env::var("AB_SMOKE_BIN").or_else(|_| std::env::var("SRTLA_SEND_BIN")) else {
        eprintln!("Skipping smoke: set AB_SMOKE_BIN (a --features test-internals binary)");
        return;
    };
    let m = run_once(&bin, "enhanced", "smoke");
    assert!(m.sink_bytes > 0, "no goodput at the sink");
}

#[test]
#[ignore = "A/B measurement harness: tens of minutes, needs netns privileges"]
fn step_a_flush_on_switch() {
    if !deps_ok("step_a_flush_on_switch") {
        return;
    }
    let Some((baseline, candidate)) = binaries("A") else {
        eprintln!("Skipping step A: set AB_STEP_A_BASELINE_BIN and AB_STEP_A_CANDIDATE_BIN");
        return;
    };

    let outcomes = run_matrix(&STEP_A_MODES, &baseline, &candidate);
    print_table(
        "STEP A (remove flush-on-switch, upstream 0cc0c6d)",
        &outcomes,
    );

    let unactivated: Vec<&str> = outcomes
        .iter()
        .filter(|o| !o.switch_activated())
        .map(|o| o.mode.as_str())
        .collect();
    let failing: Vec<&str> = outcomes
        .iter()
        .filter(|o| !o.passes_adoption_rule())
        .map(|o| o.mode.as_str())
        .collect();

    if !unactivated.is_empty() {
        println!(
            "\nSTEP A VERDICT: DEFER-FOLLOWUP — treatment unexercised under deterministic \
             scenario (switch_count < 1 in some run for: {unactivated:?})"
        );
    } else if failing.is_empty() {
        println!("\nSTEP A VERDICT: ADOPT — adoption rule satisfied in every measured mode");
    } else {
        println!("\nSTEP A VERDICT: REJECT — adoption rule violated in: {failing:?}");
    }
}

#[test]
#[ignore = "A/B measurement harness: tens of minutes, needs netns privileges"]
fn step_b_switch_cooldown() {
    if !deps_ok("step_b_switch_cooldown") {
        return;
    }
    let Some((baseline, candidate)) = binaries("B") else {
        eprintln!(
            "Skipping step B: set AB_STEP_B_BASELINE_BIN and AB_STEP_B_CANDIDATE_BIN (step B runs \
             only when step A was adopted)"
        );
        return;
    };

    let outcomes = run_matrix(&STEP_B_MODES, &baseline, &candidate);
    print_table(
        "STEP B (remove MIN_SWITCH_INTERVAL_MS cooldown, upstream 24b5f64)",
        &outcomes,
    );

    let unactivated: Vec<&str> = outcomes
        .iter()
        .filter(|o| !o.cooldown_activated())
        .map(|o| o.mode.as_str())
        .collect();
    let failing: Vec<&str> = outcomes
        .iter()
        .filter(|o| !o.passes_adoption_rule())
        .map(|o| o.mode.as_str())
        .collect();

    if !unactivated.is_empty() {
        println!(
            "\nSTEP B VERDICT: DEFER-FOLLOWUP — treatment unexercised under deterministic \
             scenario (baseline cooldown_hold_count < 1 in some run for: {unactivated:?})"
        );
    } else if failing.is_empty() {
        println!("\nSTEP B VERDICT: ADOPT — adoption rule satisfied in every measured mode");
    } else {
        println!("\nSTEP B VERDICT: REJECT — adoption rule violated in: {failing:?}");
    }
}
