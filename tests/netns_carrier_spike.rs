//! Verify-first N-link carrier spike; no production topology generalization.
//! Run with a live sudo ticket and a 180-second outer timeout, with --ignored
//! --nocapture --test-threads=1. Explicit runs fail rather than silently skip.

use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::harness::{NamespaceProcess, wait_for_registered_uplinks};
use network_sim::topology::Namespace;
use network_sim::twin::{RECEIVER_IP, TwinTopology};
use network_sim::wait_for_udp_listener;
use serde::Deserialize;

const REGISTERED: Duration = Duration::from_secs(45);
const SOURCES: &str = "10.30.1.1\n10.30.2.1\n10.30.3.1\n";

#[derive(Deserialize)]
struct Route {
    dev: String,
}

fn ip(ns: &Namespace, args: &str) -> Result<()> {
    ns.exec_checked("ip", &args.split_ascii_whitespace().collect::<Vec<_>>())?;
    Ok(())
}

fn route_device(ns: &Namespace, destination: &str) -> Result<String> {
    let output = ns.exec_checked("ip", &["-j", "route", "get", destination])?;
    let routes: Vec<Route> = serde_json::from_slice(&output.stdout)?;
    Ok(routes
        .into_iter()
        .next()
        .context("route has no device")?
        .dev)
}

fn readdress(ns: &Namespace, iface: &str, cidr: &str) -> Result<()> {
    ns.exec_checked("ip", &["-4", "addr", "flush", "dev", iface])?;
    ns.exec_checked("ip", &["addr", "add", cidr, "dev", iface])?;
    ns.exec_checked(
        "sysctl",
        &["-qw", &format!("net.ipv4.conf.{iface}.rp_filter=0")],
    )?;
    Ok(())
}

fn distinct_carriers() -> Result<TwinTopology> {
    // Reuse the actual twin builder: five RAII namespaces, six veth pairs,
    // forwarding, MASQUERADE, receiver loopback, and all/default rp_filter=0.
    let topo = TwinTopology::new("carrier_spike", 3)?;
    for idx in 0..3 {
        let n = idx + 1;
        let gateway = &topo.gateways[idx];
        let sender = topo.sender_iface(idx);
        let lan = route_device(gateway, "10.30.9.1")?;
        let wan = route_device(gateway, &format!("10.31.{n}.2"))?;
        let receiver = route_device(&topo.receiver_ns, &format!("10.31.{n}.1"))?;
        topo.blackhole(idx)?; // Remove the inherited duplicate-IP default.
        topo.receiver_ns
            .exec_checked("ip", &["route", "del", &format!("10.31.{n}.0/24")])?;
        readdress(&topo.sender_ns, sender, &format!("10.30.{n}.1/24"))?;
        readdress(gateway, &lan, &format!("10.30.{n}.2/24"))?;
        readdress(gateway, &wan, &format!("100.64.{n}.1/24"))?;
        readdress(&topo.receiver_ns, &receiver, &format!("100.64.{n}.2/24"))?;
        ip(
            gateway,
            &format!("route replace {RECEIVER_IP} via 100.64.{n}.2 dev {wan}"),
        )?;
        ip(
            &topo.receiver_ns,
            &format!("route replace 100.64.{n}.0/24 dev {receiver} scope link src {RECEIVER_IP}"),
        )?;
        // A source bind is not a device bind: each source needs its own table.
        let table = (100 + n).to_string();
        ip(
            &topo.sender_ns,
            &format!("route add 10.30.{n}.0/24 dev {sender} table {table}"),
        )?;
        ip(
            &topo.sender_ns,
            &format!("route add default via 10.30.{n}.2 dev {sender} table {table}"),
        )?;
        ip(
            &topo.sender_ns,
            &format!("rule add from 10.30.{n}.1/32 table {table}"),
        )?;
        topo.shape(idx, 2000, 25)?;
        eprintln!("link={idx} source=10.30.{n}.1 carrier=100.64.{n}.0/24 iface={sender}");
    }
    Ok(topo)
}

fn start_stack(topo: &TwinTopology, ips: &Path) -> Result<[NamespaceProcess; 3]> {
    let listener = NamespaceProcess::spawn(
        &topo.receiver_ns,
        "srt-live-transmit",
        &["srt://:4001?mode=listener", "udp://127.0.0.1:9999"],
    )?;
    wait_for_udp_listener(&topo.receiver_ns, 4001, Duration::from_secs(5))?;
    let receiver = NamespaceProcess::spawn(
        &topo.receiver_ns,
        "srtla_rec",
        &[
            "--srtla_port",
            "5000",
            "--srt_hostname",
            "127.0.0.1",
            "--srt_port",
            "4001",
        ],
    )?;
    wait_for_udp_listener(&topo.receiver_ns, 5000, Duration::from_secs(5))?;
    let sender = NamespaceProcess::spawn(
        &topo.sender_ns,
        env!("CARGO_BIN_EXE_srtla_send"),
        &[
            "5555",
            RECEIVER_IP,
            "5000",
            ips.to_str().context("UTF-8 ips path")?,
            "--mode",
            "classic",
            "--verbose",
        ],
    )?;
    // Processes drop before the borrowed topology, including on assertion failure.
    Ok([sender, receiver, listener])
}

fn carried_bytes(topo: &TwinTopology) -> Result<Vec<u64>> {
    let before = (0..3)
        .map(|idx| topo.tx_bytes(idx))
        .collect::<Result<Vec<_>>>()?;
    // Raw SRT-shaped DATA, as in netns_twin: wire carriage, not SRT goodput.
    // Absolute monotonic deadlines avoid accumulating sleep drift: 600 x 1316 B/s.
    let script = r"import socket,time
s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM)
start=time.monotonic()
for i in range(6000):
    time.sleep(max(0,start+i/600-time.monotonic()))
    s.sendto(i.to_bytes(4,'big')+bytes(1312),('127.0.0.1',5555))
time.sleep(max(0,start+10-time.monotonic()))
";
    let mut traffic = NamespaceProcess::spawn(&topo.sender_ns, "python3", &["-c", script])?;
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some((code, stderr)) = traffic.check_exit() {
            ensure!(code == Some(0), "CBR failed: {code:?}: {stderr}");
            break;
        }
        ensure!(Instant::now() < deadline, "CBR exceeded 15-second deadline");
        sleep(Duration::from_millis(20));
    }
    before
        .iter()
        .enumerate()
        .map(|(idx, start)| Ok(topo.tx_bytes(idx)? - start))
        .collect()
}

#[test]
#[ignore = "privileged three-carrier spike; sudo, iptables, tc, Python, SRT tools"]
fn three_distinct_carriers_register_and_carry_with_reverse_path_qa() -> Result<()> {
    // Given: the twin primitives adapted to distinct IPs and CGNAT transits.
    let topo = distinct_carriers()?;
    let files = tempfile::tempdir()?;
    let ips = files.path().join("ips");
    std::fs::write(&ips, SOURCES)?;

    // When: classic is launched on the legacy three-line IP file, with no map.
    let processes = start_stack(&topo, &ips)?;
    wait_for_registered_uplinks(&processes[0], 3, REGISTERED)?;
    let carried = carried_bytes(&topo)?;
    eprintln!("baseline REG3=3 tx_bytes={carried:?}");
    // Then: every physical uplink carries substantially more than control traffic.
    for (idx, bytes) in carried.iter().enumerate() {
        ensure!(
            *bytes > 100_000,
            "link {idx} carried only {bytes} bytes: {carried:?}"
        );
    }
    drop(processes);

    topo.gateways[2].exec_checked("sysctl", &["-qw", "net.ipv4.conf.all.rp_filter=1"])?;
    let processes = start_stack(&topo, &ips)?;
    wait_for_registered_uplinks(&processes[0], 3, REGISTERED)?;
    eprintln!("symmetric carrier 2 rp_filter=1 REG3=3 (strict alone does not break symmetry)");
    drop(processes);
    topo.gateways[2].exec_checked("sysctl", &["-qw", "net.ipv4.conf.all.rp_filter=0"])?;

    // Marked replies use table 200 (LAN), while src_valid_mark=0 leaves reverse
    // validation on the main table's WAN host route. QA-only, NOT BondTopology.
    let carrier = &topo.gateways[2];
    let lan = route_device(carrier, "10.30.3.1")?;
    let wan = route_device(carrier, "100.64.3.2")?;
    ip(
        carrier,
        &format!("route add 10.30.3.1/32 dev {lan} table 200"),
    )?;
    ip(carrier, "rule add priority 100 fwmark 200 lookup 200")?;
    carrier.exec_checked(
        "iptables",
        &[
            "-t",
            "mangle",
            "-A",
            "PREROUTING",
            "-i",
            &wan,
            "-j",
            "MARK",
            "--set-mark",
            "200",
        ],
    )?;
    for device in ["all", lan.as_str(), wan.as_str()] {
        carrier.exec_checked(
            "sysctl",
            &["-qw", &format!("net.ipv4.conf.{device}.src_valid_mark=0")],
        )?;
    }
    ip(carrier, &format!("route add 10.30.3.1/32 dev {wan}"))?;
    let processes = start_stack(&topo, &ips)?;
    wait_for_registered_uplinks(&processes[0], 3, REGISTERED)?;
    eprintln!("asymmetric carrier 2 rp_filter=0 REG3=3");
    drop(processes);

    carrier.exec_checked("sysctl", &["-qw", "net.ipv4.conf.all.rp_filter=1"])?;
    let processes = start_stack(&topo, &ips)?;
    wait_for_registered_uplinks(&processes[0], 2, REGISTERED)?;
    let strict = wait_for_registered_uplinks(&processes[0], 3, REGISTERED);
    let log = processes[0].log_snapshot().join("\n");
    ensure!(
        strict.is_err(),
        "asymmetric strict carrier unexpectedly registered"
    );
    ensure!(
        !log.contains("REG3 from uplink #2"),
        "carrier 2 registered:\n{log}"
    );
    ensure!(
        log.contains("REG3 from uplink #0") && log.contains("REG3 from uplink #1"),
        "unaffected carriers must register:\n{log}"
    );
    eprintln!("asymmetric carrier 2 rp_filter=1 REG3=[0,1], missing=2 for 45s");
    drop(processes);

    carrier.exec_checked("sysctl", &["-qw", "net.ipv4.conf.all.rp_filter=0"])?;
    let processes = start_stack(&topo, &ips)?;
    wait_for_registered_uplinks(&processes[0], 3, REGISTERED)?;
    let carried = carried_bytes(&topo)?;
    eprintln!("restored asymmetric carrier 2 rp_filter=0 REG3=3 tx_bytes={carried:?}");
    for (idx, bytes) in carried.iter().enumerate() {
        ensure!(
            *bytes > 100_000,
            "restored link {idx} carried only {bytes} bytes"
        );
    }
    Ok(())
}
