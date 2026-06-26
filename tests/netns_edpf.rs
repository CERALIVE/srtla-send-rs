//! EDPF heterogeneous-RTT bonding scenario under network namespaces (opt-in).
//!
//! Runs `srtla_send --mode edpf` across two impaired uplinks with very
//! different latency (30 ms vs 150 ms RTT) and mild loss, both bandwidth-capped
//! below the offered rate so the scheduler *must* bond across them. It then
//! proves the realistic-impairment outcome the EDPF pipeline (BLEST → IoDS →
//! EDPF, fixed by T1/T3) targets:
//!
//!   - both uplinks carry forwarded DATA throughout the run (no link starvation,
//!     even the 150 ms one), and
//!   - the bonded stream keeps flowing in both halves of the run (no stall).
//!
//! This is the realistic netns SUPPLEMENT to T6's in-process golden correctness
//! oracle. It skips cleanly unless root/sudo + srtla_rec + srt-live-transmit +
//! tcpdump + python3 + netem are available — mirroring the other `netns_*`
//! suites via the harness `SkipReason`/privilege gate — so the default
//! `cargo test` gate stays green (skipped, never ignored) in CI.

mod common;

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use network_sim::{
    ImpairmentConfig, NamespaceProcess, SrtlaTestTopology, check_binary, check_privileges,
    wait_for_udp_listener,
};

const LOCAL_SRT_PORT: u16 = 5558;
const SRTLA_PORT: u16 = 5000;
const SRT_PORT: u16 = 4001;
const SINK_URI: &str = "udp://127.0.0.1:9999";

/// Per-link bandwidth cap (kbit). Both links are capped well below the offered
/// injection rate (~6.3 Mbps), so a single link cannot absorb the stream and
/// the EDPF scheduler is forced to bond across both — that is what makes the
/// 150 ms link's traffic share a genuine no-starvation signal rather than an
/// artifact of an uncapped fast path soaking up everything.
const LINK_RATE_KBIT: u64 = 4000;

fn deps_ok() -> bool {
    if !check_privileges() {
        eprintln!("Skipping netns_edpf: requires root / passwordless sudo");
        return false;
    }
    for bin in ["srtla_rec", "srt-live-transmit", "tcpdump", "python3"] {
        if check_binary(bin).is_none() {
            eprintln!("Skipping netns_edpf: '{bin}' not found in PATH");
            return false;
        }
    }
    let netem_ok = Command::new("sudo")
        .args(["modprobe", "sch_netem"])
        .output()
        .is_ok_and(|o| o.status.success());
    if !netem_ok {
        eprintln!("Skipping netns_edpf: sch_netem unavailable");
        return false;
    }
    true
}

fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

/// The full sender→receiver pipeline plus the per-link routing the bonding
/// topology needs: without it both bound source IPs resolve the receiver via
/// link 0's subnet and link 1 never carries traffic (see `start_stack`).
struct Stack {
    topo: SrtlaTestTopology,
    _srt: NamespaceProcess,
    _rec: NamespaceProcess,
    _send: NamespaceProcess,
}

fn start_stack(name: &str) -> Stack {
    common::build_srtla_send();
    let topo = SrtlaTestTopology::new(name, 2).expect("create topology");

    let recv_ip = topo.receiver_ip.clone();
    let src1 = topo.sender_ips[1].clone();
    let sif1 = topo.sender_ifaces[1].clone();
    let rif1 = topo.receiver_ifaces[1].clone();

    // Loose reverse-path: link 1's reply path is asymmetric by design.
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

    // routing: force link 1's source to egress its own veth, and make the
    // receiver answer link 1 *from* the IP the sender connect()-ed to, so the
    // connected uplink socket accepts the reply.
    let _ = topo.sender_ns.exec(
        "ip",
        &["route", "add", &recv_ip, "dev", &sif1, "table", "102"],
    );
    let _ = topo
        .sender_ns
        .exec("ip", &["rule", "add", "from", &src1, "lookup", "102"]);
    let _ = topo.receiver_ns.exec(
        "ip",
        &["route", "add", &src1, "dev", &rif1, "src", &recv_ip],
    );

    let srt_uri = format!("srt://:{SRT_PORT}?mode=listener");
    let srt = NamespaceProcess::spawn(
        &topo.receiver_ns,
        "srt-live-transmit",
        &[&srt_uri, SINK_URI],
    )
    .expect("start srt-live-transmit");
    wait_for_udp_listener(&topo.receiver_ns, SRT_PORT, Duration::from_secs(5))
        .expect("srt-live-transmit listener");

    let srtla_port = SRTLA_PORT.to_string();
    let srt_port = SRT_PORT.to_string();
    let rec = NamespaceProcess::spawn(
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
    wait_for_udp_listener(&topo.receiver_ns, SRTLA_PORT, Duration::from_secs(5))
        .expect("srtla_rec listener");

    let ips = topo.write_ip_list().expect("write ip list");
    let bin = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/debug/srtla_send");
    let local = LOCAL_SRT_PORT.to_string();
    let send = NamespaceProcess::spawn_with_env(
        &topo.sender_ns,
        bin.to_str().expect("bin path"),
        &[
            local.as_str(),
            recv_ip.as_str(),
            srtla_port.as_str(),
            ips.to_str().expect("ips path"),
            "--mode",
            "edpf",
        ],
        &[("RUST_LOG", "info")],
    )
    .expect("start srtla_send");

    // Readiness gate: srtla_send's local SRT listener must be up before any
    // injection — poll for it instead of sleeping a fixed interval.
    wait_for_udp_listener(&topo.sender_ns, LOCAL_SRT_PORT, Duration::from_secs(5))
        .expect("srtla_send local SRT listener");

    Stack {
        topo,
        _srt: srt,
        _rec: rec,
        _send: send,
    }
}

/// Kill every process inside a namespace, keeping teardown bounded (the
/// sudo-wrapped children are not process-group leaders).
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

impl Drop for Stack {
    fn drop(&mut self) {
        kill_netns_pids(&self.topo.sender_ns.name);
        kill_netns_pids(&self.topo.receiver_ns.name);
    }
}

/// A `tcpdump` writing newline-per-packet `-tt -n` records to a temp file
/// (a file, not a pipe, avoids the 64 KiB pipe stall on data-rate captures).
struct Capture {
    child: Child,
    path: PathBuf,
    iface: String,
}

fn start_capture(ns_name: &str, iface: &str, filter: &str) -> Capture {
    let path = std::env::temp_dir().join(format!("edpfcap_{}_{iface}.txt", std::process::id()));
    let file = std::fs::File::create(&path).expect("create capture file");
    let child = Command::new("sudo")
        .args([
            "ip", "netns", "exec", ns_name, "tcpdump", "-i", iface, "-tt", "-n", filter,
        ])
        .stdout(Stdio::from(file))
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn tcpdump");
    sleep(Duration::from_millis(300));
    Capture {
        child,
        path,
        iface: iface.to_string(),
    }
}

fn stop_capture(mut cap: Capture) -> Vec<f64> {
    let _ = Command::new("sudo")
        .args([
            "pkill",
            "-TERM",
            "-f",
            &format!("tcpdump -i {} ", cap.iface),
        ])
        .output();
    sleep(Duration::from_millis(400));
    let _ = cap.child.wait();
    let text = std::fs::read_to_string(&cap.path).unwrap_or_default();
    let _ = std::fs::remove_file(&cap.path);
    text.lines()
        .filter_map(|l| l.split_whitespace().next()?.parse::<f64>().ok())
        .collect()
}

fn spawn_injector(ns_name: &str, secs: f64) -> Child {
    let script = format!(
        "import socket,struct,time  # \
         EDPFINJ\ns=socket.socket(socket.AF_INET,socket.SOCK_DGRAM)\npay=bytes(1316); seq=0; \
         t0=time.time()\nwhile time.time()-t0<{secs}: \
         s.sendto(struct.pack('>I',seq&0x7fffffff)+pay[4:],('127.0.0.1',{LOCAL_SRT_PORT})); \
         seq+=1; time.sleep(1/600)\ns.close()"
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
        .args(["pkill", "-TERM", "-f", "EDPFINJ"])
        .output();
    let _ = child.wait();
}

fn count_in(ts: &[f64], lo: f64, hi: f64) -> usize {
    ts.iter().filter(|&&t| t >= lo && t <= hi).count()
}

/// Warm-up + readiness gate: returns once BOTH uplinks carry forwarded DATA, so
/// measurement never starts before the high-RTT link finishes registration.
fn wait_links_active(stack: &Stack) -> bool {
    let if0 = stack.topo.sender_ifaces[0].clone();
    let if1 = stack.topo.sender_ifaces[1].clone();
    let ns = stack.topo.sender_ns.name.clone();
    let data = format!("udp and dst port {SRTLA_PORT} and udp[8] < 0x80");
    for _ in 0..15 {
        let c0 = start_capture(&ns, &if0, &data);
        let c1 = start_capture(&ns, &if1, &data);
        let inj = spawn_injector(&ns, 1.6);
        sleep(Duration::from_secs(2));
        stop_injector(inj);
        let n0 = stop_capture(c0).len();
        let n1 = stop_capture(c1).len();
        if n0 > 5 && n1 > 5 {
            return true;
        }
    }
    false
}

#[test]
fn edpf_bonds_heterogeneous_rtt_links_without_starvation() {
    if !deps_ok() {
        return;
    }
    let stack = start_stack("edpfhet");

    // Heterogeneous RTT + mild loss, both capped below the offered rate so the
    // EDPF scheduler must spill onto the slow link (genuine bonding).
    stack
        .topo
        .impair_link(
            0,
            ImpairmentConfig {
                delay_ms: Some(30),
                loss_percent: Some(0.3),
                rate_kbit: Some(LINK_RATE_KBIT),
                tbf_shaping: true,
                ..Default::default()
            },
        )
        .expect("impair link 0 (fast)");
    stack
        .topo
        .impair_link(
            1,
            ImpairmentConfig {
                delay_ms: Some(150),
                loss_percent: Some(0.3),
                rate_kbit: Some(LINK_RATE_KBIT),
                tbf_shaping: true,
                ..Default::default()
            },
        )
        .expect("impair link 1 (slow)");

    assert!(
        wait_links_active(&stack),
        "both uplinks failed to carry data before measurement — EDPF bonding never started"
    );

    let ns = stack.topo.sender_ns.name.clone();
    let if0 = stack.topo.sender_ifaces[0].clone();
    let if1 = stack.topo.sender_ifaces[1].clone();
    let data = format!("udp and dst port {SRTLA_PORT} and udp[8] < 0x80");

    let window = 16.0;
    let inj = spawn_injector(&ns, window + 3.0);
    let cap0 = start_capture(&ns, &if0, &data);
    let cap1 = start_capture(&ns, &if1, &data);

    sleep(Duration::from_secs(1));
    let t_start = now();
    let t_mid = t_start + window / 2.0;
    sleep(Duration::from_secs_f64(window));
    let t_end = now();

    let t0 = stop_capture(cap0);
    let t1 = stop_capture(cap1);
    stop_injector(inj);

    // Per-link DATA forwarded over the measurement window.
    let n0 = count_in(&t0, t_start, t_end);
    let n1 = count_in(&t1, t_start, t_end);
    let total = n0 + n1;

    // No-stall proof: the bonded path keeps flowing in BOTH halves of the run.
    let first_half = count_in(&t0, t_start, t_mid) + count_in(&t1, t_start, t_mid);
    let second_half = count_in(&t0, t_mid, t_end) + count_in(&t1, t_mid, t_end);

    println!(
        "[edpf] window={window:.0}s link0(30ms)={n0} link1(150ms)={n1} total={total} \
         first_half={first_half} second_half={second_half}"
    );

    // No-stall: a meaningful volume of DATA was forwarded across the bonded path
    // over the window, and it did not seize up in either half.
    assert!(
        total >= 200,
        "bonded path forwarded too little ({total} pkts in {window:.0}s) — stream stalled"
    );
    assert!(
        first_half > 0 && second_half > 0,
        "bonded path stalled mid-run (first_half={first_half}, second_half={second_half})"
    );

    // No-starvation: the high-RTT link is not abandoned. With both links capped
    // below the offered rate, a healthy EDPF spills onto link 1; the fast link
    // still carries the larger share.
    assert!(
        n0 > 0,
        "fast link (30ms) carried zero DATA — scheduler starved it"
    );
    assert!(
        n1 > 0,
        "slow link (150ms) carried zero DATA — EDPF starved the high-RTT uplink"
    );
}
