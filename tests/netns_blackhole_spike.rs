//! Real-kernel size-class spike; run explicitly with `--ignored --nocapture`.
//! Only IPv4 lengths 1024..=2047 match 0x0400/0xfc00 (not arbitrary jumbo DATA).
//! UDP payloads 1316/38 become IP lengths 1344/66, without fragmentation.

#![cfg(target_os = "linux")]

use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::harness::{NamespaceProcess, SrtlaTestTopology, check_binary};
use network_sim::impairment::{ImpairmentConfig, apply_impairment};
use serde::Deserialize;

const LISTENER: &str = r#"
import json, socket, time
with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
    sock.bind(('0.0.0.0', 0))
    sock.settimeout(2)
    print('READY ' + str(sock.getsockname()[1]), flush=True)
    received = []
    while True:
        try:
            data, peer = sock.recvfrom(2048)
        except TimeoutError:
            break
        elapsed_ms = (time.monotonic_ns() - int.from_bytes(data[:8], 'big')) / 1e6
        received.append({'bytes': len(data), 'elapsed_ms': elapsed_ms})
    print('RECEIVED ' + json.dumps(received), flush=True)
"#;

const SEND: &str = r#"
import socket, sys, time
with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
    sock.bind((sys.argv[1], 0))
    for size in (1316, 38):
        data = time.monotonic_ns().to_bytes(8, 'big') + bytes(size - 8)
        assert sock.sendto(data, (sys.argv[2], int(sys.argv[3]))) == size
        print('SENT ' + str(size), flush=True)
"#;

#[derive(Debug, Deserialize)]
struct Received {
    bytes: usize,
    elapsed_ms: f64,
}

fn wait_line(process: &mut NamespaceProcess, prefix: &str) -> Result<String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(line) = process
            .log_snapshot()
            .iter()
            .find_map(|line| line.strip_prefix(prefix))
        {
            return Ok(line.to_owned());
        }
        ensure!(
            Instant::now() < deadline,
            "timeout waiting for {prefix}: {:?}",
            process.log_snapshot()
        );
        if let Some((code, stderr)) = process.check_exit() {
            // check_exit joins pipe drains, so take the final snapshot before failing.
            return process
                .log_snapshot()
                .iter()
                .find_map(|line| line.strip_prefix(prefix).map(str::to_owned))
                .with_context(|| format!("missing {prefix}, exit={code:?}: {stderr}"));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn probe(topology: &SrtlaTestTopology, label: &str, expected: &[usize]) -> Result<Vec<Received>> {
    // Declared after topology: the process guard kills/reaps Python before netns deletion,
    // including on assertion failure. Every probe gets a fresh ephemeral listener port.
    let mut listener =
        NamespaceProcess::spawn(&topology.receiver_ns, "python3", &["-u", "-c", LISTENER])?;
    let port = wait_line(&mut listener, "READY ")?.parse::<u16>()?;
    let output = topology.sender_ns.exec_checked(
        "timeout",
        &[
            "5s",
            "python3",
            "-u",
            "-c",
            SEND,
            &topology.sender_ips[0],
            &topology.receiver_ip,
            &port.to_string(),
        ],
    )?;
    println!(
        "{label}: {}",
        String::from_utf8_lossy(&output.stdout).trim()
    );
    let json = wait_line(&mut listener, "RECEIVED ")?;
    println!("{label}: RECEIVED {json}");
    let received: Vec<Received> = serde_json::from_str(&json)?;
    let mut sizes: Vec<usize> = received.iter().map(|packet| packet.bytes).collect();
    sizes.sort_unstable();
    assert_eq!(sizes, expected, "{label}: receiver byte counts");
    Ok(received)
}

fn tc(topology: &SrtlaTestTopology, command: &str) -> Result<()> {
    let command = command.replace("{dev}", &topology.sender_ifaces[0]);
    println!(
        "sudo ip netns exec {} tc {command}",
        topology.sender_ns.name
    );
    let args: Vec<&str> = command.split_whitespace().collect();
    let output = topology.sender_ns.exec_checked("tc", &args)?;
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

#[test]
#[ignore = "privileged tc/netns size-class spike; run explicitly with --ignored --nocapture"]
fn data_blackhole_survives_band_one_replacement() -> Result<()> {
    if !Command::new("sudo")
        .args(["-n", "true"])
        .status()?
        .success()
    {
        eprintln!("skipped: sudo -n true (requires an unattended sudo ticket)");
        return Ok(());
    }
    for tool in ["ip", "tc", "python3", "timeout"] {
        if check_binary(tool).is_none() {
            eprintln!("skipped: {tool} not found");
            return Ok(());
        }
    }

    // Given: one real link, with the harness's PID+atomic-counter names and RAII.
    let topology = SrtlaTestTopology::new("bh", 1)?;
    println!(
        "topology: sender={} receiver={} veths={:?}/{:?}",
        topology.sender_ns.name,
        topology.receiver_ns.name,
        topology.sender_ifaces,
        topology.receiver_ifaces
    );
    probe(&topology, "baseline", &[38, 1316])?;

    // Failure control: root loss works, but the real legacy impairment API erases it.
    tc(
        &topology,
        "qdisc add dev {dev} root handle 20: netem loss 100%",
    )?;
    probe(&topology, "root-blackhole", &[])?;
    tc(&topology, "-s qdisc show dev {dev}")?;
    println!("apply_impairment(delay_ms=60): executes the following two commands");
    println!(
        "sudo ip netns exec {} tc qdisc del dev {} root",
        topology.sender_ns.name, topology.sender_ifaces[0]
    );
    println!(
        "sudo ip netns exec {} tc qdisc add dev {} root netem delay 60ms",
        topology.sender_ns.name, topology.sender_ifaces[0]
    );
    apply_impairment(
        &topology.sender_ns,
        &topology.sender_ifaces[0],
        ImpairmentConfig {
            delay_ms: Some(60),
            ..Default::default()
        },
    )?;
    probe(&topology, "root-blackhole-erased", &[38, 1316])?;

    for rate_set in [false, true] {
        println!("COMPOSED rate_set={rate_set}");
        tc(&topology, "qdisc del dev {dev} root")?;
        tc(
            &topology,
            "qdisc add dev {dev} root handle 1: prio bands 2 priomap 0 0 0 0 0 0 0 0 0 0 0 0 0 0 \
             0 0",
        )?;
        if rate_set {
            tc(
                &topology,
                "qdisc add dev {dev} parent 1:1 handle 10: tbf rate 10000kbit burst 125000 \
                 latency 1s",
            )?;
            tc(
                &topology,
                "qdisc add dev {dev} parent 10:1 handle 11: netem delay 20ms 0ms loss 0% limit \
                 1000",
            )?;
        } else {
            tc(
                &topology,
                "qdisc add dev {dev} parent 1:1 handle 10: netem delay 20ms 0ms loss 0% limit 1000",
            )?;
        }
        tc(
            &topology,
            "qdisc add dev {dev} parent 1:2 handle 20: netem loss 100%",
        )?;

        // When: classify only the DATA-sized IPv4 datagram into band two.
        tc(
            &topology,
            "filter add dev {dev} parent 1: protocol ip prio 10 u32 match u16 0x0400 0xfc00 at 2 \
             flowid 1:2",
        )?;
        let before = probe(&topology, "filter-on-20ms", &[38])?;
        assert!(
            before[0].elapsed_ms >= 15.0,
            "band one bypassed: {before:?}"
        );

        if rate_set {
            tc(
                &topology,
                "qdisc replace dev {dev} parent 1:1 handle 10: tbf rate 20000kbit burst 250000 \
                 latency 1s",
            )?;
            tc(
                &topology,
                "qdisc replace dev {dev} parent 10:1 handle 11: netem delay 60ms 0ms loss 0% \
                 limit 1000",
            )?;
        } else {
            tc(
                &topology,
                "qdisc replace dev {dev} parent 1:1 handle 10: netem delay 60ms 0ms loss 0% limit \
                 1000",
            )?;
        }
        tc(&topology, "-s qdisc show dev {dev}")?;
        tc(&topology, "filter show dev {dev} parent 1:")?;
        let after = probe(&topology, "filter-on-replaced-60ms", &[38])?;
        assert!(
            after[0].elapsed_ms >= 50.0,
            "delay replacement bypassed: {after:?}"
        );
        assert!(
            after[0].elapsed_ms > before[0].elapsed_ms + 20.0,
            "delay did not increase: {before:?} -> {after:?}"
        );

        // Then: toggling ONLY the filter restores DATA through the delayed band.
        tc(
            &topology,
            "filter del dev {dev} parent 1: protocol ip prio 10",
        )?;
        let restored = probe(&topology, "filter-off-restored", &[38, 1316])?;
        assert!(restored.iter().all(|packet| packet.elapsed_ms >= 50.0));
    }
    // Revert all scratch qdiscs; topology Drop deletes both namespaces and veths.
    tc(&topology, "qdisc del dev {dev} root")?;
    Ok(())
}
