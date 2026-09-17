//! Verify-first HSRSP wire spike; no production parser or instrumentation.
//! Run with SRTLA_REC_BIN pointing at an out-of-tree CeraLive receiver build:
//! `timeout --foreground --kill-after=10s 90s cargo test --test netns_hsrsp_spike -- --ignored --nocapture`
//! Set UPDATE_GOLDEN=1 deliberately to replace the 2000ms raw UDP-payload fixture.

#![cfg(target_os = "linux")]

use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::harness::{
    NamespaceProcess, SrtlaTestTopology, check_binary, wait_for_registered_uplinks,
    wait_for_udp_listener,
};
use network_sim::topology::Namespace;

#[path = "support/hsrsp.rs"]
mod hsrsp;
use hsrsp::decode_hsrsp;

fn start_capture(ns: &Namespace, iface: &str, path: &Path) -> Result<NamespaceProcess> {
    let filter = if iface == "lo" {
        "udp src port 5555"
    } else {
        "udp src port 5000"
    };
    let mut capture = NamespaceProcess::spawn(
        ns,
        "tcpdump",
        &[
            "-n",
            "-U",
            "-s",
            "0",
            "-Z",
            "root",
            "-i",
            iface,
            "-w",
            path.to_str().context("UTF-8 pcap path")?,
            filter,
        ],
    )?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while !capture
        .log_snapshot()
        .iter()
        .any(|line| line.contains("listening on"))
    {
        ensure!(
            capture.check_exit().is_none(),
            "tcpdump exited: {:?}",
            capture.log_snapshot()
        );
        ensure!(Instant::now() < deadline, "tcpdump readiness timeout");
        sleep(Duration::from_millis(20));
    }
    Ok(capture)
}

fn handshake_payloads(path: &Path) -> Result<Vec<Vec<u8>>> {
    // tshark only unwraps pcap/Ethernet/IP/UDP; the Rust helper decodes SRT itself.
    let output = Command::new("timeout")
        .args(["5s", "tshark", "-n", "-r"])
        .arg(path)
        .args([
            "-Y",
            "udp.payload[0:2] == 80:00",
            "-T",
            "fields",
            "-e",
            "udp.payload",
        ])
        .output()
        .context("extract raw UDP payloads")?;
    ensure!(
        output.status.success(),
        "tshark: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)?
        .lines()
        .map(|line| {
            let hex = line.replace(':', "");
            ensure!(
                hex.is_ascii() && hex.len().is_multiple_of(2),
                "invalid payload hex"
            );
            (0..hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).context("UDP payload hex byte"))
                .collect()
        })
        .collect()
}

fn capture_latency(latency_ms: u16, nak_report: bool) -> Result<()> {
    // Given: an isolated one-link bond and real caller/listener, ready before dialing.
    ensure!(
        network_sim::test_util::check_privileges(),
        "requires unattended sudo"
    );
    for tool in [
        "srt-live-transmit",
        "tcpdump",
        "tshark",
        "ip",
        "ss",
        "timeout",
    ] {
        ensure!(check_binary(tool).is_some(), "missing dependency: {tool}");
    }
    let receiver =
        std::env::var("SRTLA_REC_BIN").context("set SRTLA_REC_BIN to a CeraLive receiver build")?;
    ensure!(
        Path::new(&receiver).is_absolute() && Path::new(&receiver).is_file(),
        "receiver must be an absolute file path"
    );
    let topo = SrtlaTestTopology::new("hsrsp", 1)?;
    let artifacts = std::env::var_os("HSRSP_CAPTURE_DIR");
    let mut builder = tempfile::Builder::new();
    builder.prefix("hsrsp-");
    let dir = match &artifacts {
        Some(parent) => builder.tempdir_in(parent)?,
        None => builder.tempdir()?,
    };
    let ips = dir.path().join("ips.txt");
    fs::write(&ips, format!("{}\n", topo.sender_ips[0]))?;
    let uri = format!("srt://:4001?mode=listener&latency={latency_ms}&nakreport={nak_report}");
    let listener = NamespaceProcess::spawn(
        &topo.receiver_ns,
        "srt-live-transmit",
        &[&uri, "udp://127.0.0.1:9999"],
    )?;
    wait_for_udp_listener(&topo.receiver_ns, 4001, Duration::from_secs(5))?;
    let rec = NamespaceProcess::spawn(
        &topo.receiver_ns,
        &receiver,
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
            &topo.receiver_ip,
            "5000",
            ips.to_str().context("UTF-8 IP-list path")?,
            "--verbose",
        ],
    )?;
    wait_for_registered_uplinks(&sender, 1, Duration::from_secs(10))?;
    let uplink_path = dir.path().join("uplink.pcap");
    let client_path = dir.path().join("client.pcap");
    let mut uplink = start_capture(&topo.sender_ns, &topo.sender_ifaces[0], &uplink_path)?;
    let mut client = start_capture(&topo.sender_ns, "lo", &client_path)?;
    // When: the caller negotiates during the first three seconds, with captures ready.
    let caller = NamespaceProcess::spawn(
        &topo.sender_ns,
        "srt-live-transmit",
        &["udp://:9000", "srt://127.0.0.1:5555?mode=caller"],
    )?;
    sleep(Duration::from_secs(3));
    // NamespaceProcess::kill stops ALL sender-namespace PIDs, flushing both pcaps.
    uplink.kill();
    client.kill();
    for (name, process) in [
        ("sender", &sender),
        ("caller", &caller),
        ("receiver", &rec),
        ("listener", &listener),
    ] {
        fs::write(
            dir.path().join(format!("{name}.log")),
            process.log_snapshot().join("\n"),
        )?;
    }
    let incoming = handshake_payloads(&uplink_path)?;
    let forwarded = handshake_payloads(&client_path)?;
    let (raw, decoded) = incoming
        .iter()
        .find_map(|raw| decode_hsrsp(raw).map(|hs| (raw, hs)))
        .with_context(|| {
            format!(
                "no conclusion HSRSP; sender={:?}; caller={:?}",
                sender.log_snapshot(),
                caller.log_snapshot()
            )
        })?;
    // Then: live latency and byte-identical client-side forwarding, without debug hooks.
    assert_eq!(decoded.high_ms, latency_ms);
    let flags_offset = decoded.extension_offset + 8;
    let flags = u32::from_be_bytes(raw[flags_offset..flags_offset + 4].try_into()?);
    assert_eq!(flags & (1 << 4) != 0, nak_report);
    assert!(
        forwarded.contains(raw),
        "HSRSP never reached the caller-side capture"
    );
    assert!(raw.len() < 2048);
    println!(
        "latency={latency_ms}ms decoded={decoded:?} bytes={} uplink_handshakes={} \
         client_handshakes={} forwarded_identical=true",
        raw.len(),
        incoming.len(),
        forwarded.len()
    );
    println!("raw={:02x?}", raw);
    if latency_ms == 2000 && std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        let fixture = if nak_report {
            "tests/fixtures/srt-hsrsp-latency2000.bin"
        } else {
            "tests/fixtures/srt-hsrsp-nak-off.bin"
        };
        fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join(fixture), raw)?;
    }
    if artifacts.is_some() {
        println!("capture_artifacts={}", dir.keep().display());
    }
    Ok(())
}

#[test]
#[ignore = "privileged live SRT pair; requires SRTLA_REC_BIN, tcpdump and tshark"]
fn hsrsp_reports_listener_latency_2000() -> Result<()> {
    capture_latency(2000, true)
}

#[test]
#[ignore = "privileged failure QA: different listener latency must change the field"]
fn hsrsp_reports_listener_latency_500() -> Result<()> {
    capture_latency(500, true)
}

#[test]
fn hsrsp_reports_listener_nak_off() -> Result<()> {
    // Given a real listener with periodic NAK disabled (optional privileged lane).
    if !network_sim::test_util::check_privileges()
        || std::env::var_os("SRTLA_REC_BIN").is_none()
        || [
            "srt-live-transmit",
            "tcpdump",
            "tshark",
            "ip",
            "ss",
            "timeout",
        ]
        .iter()
        .any(|tool| check_binary(tool).is_none())
    {
        eprintln!("SKIP: HSRSP capture requires sudo, capture tools and SRTLA_REC_BIN");
        return Ok(());
    }
    // When negotiating, Then the captured receiver flags have bit 4 clear.
    capture_latency(2000, false)
}
