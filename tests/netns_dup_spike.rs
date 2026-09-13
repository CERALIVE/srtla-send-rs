//! Privileged, opt-in wire experiment against the CeraLive receiver and real libsrt.
#![cfg(all(target_os = "linux", feature = "test-internals"))]

#[path = "dup_spike/measurement.rs"]
mod measurement;
#[path = "dup_spike/stack.rs"]
mod stack;

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use anyhow::{Result, ensure};
use measurement::{RunResult, Stats, packets};
use network_sim::NamespaceProcess;
use stack::{Stack, wait_until};

fn run(root: &Path, variant: &str) -> Result<RunResult> {
    let dir = root.join(variant);
    std::fs::create_dir(&dir)?;
    let mut stack = Stack::start(&dir, variant)?;
    let mut source = NamespaceProcess::spawn(
        &stack.topo.sender_ns,
        "python3",
        &[stack.script(), "source", dir.to_str().unwrap()],
    )?;
    wait_until(Duration::from_secs(20), || {
        Ok(source
            .check_exit()
            .map(|status| {
                assert_eq!(status.0, Some(0), "source: {:?}", source.log_snapshot());
                true
            })
            .unwrap_or(false))
    })?;
    if variant == "unregistered" {
        let data = packets(&dir.join("loopback.pcap"))?;
        ensure!(data.len() >= 50, "need genuine SRT frames for replay");
        let mut replay = Vec::new();
        for packet in data.iter().take(50) {
            replay.extend_from_slice(&u16::try_from(packet.len())?.to_be_bytes());
            replay.extend_from_slice(packet);
        }
        std::fs::write(dir.join("replay.bin"), replay)?;
        let mut injector = NamespaceProcess::spawn(
            &stack.topo.sender_ns,
            "python3",
            &[
                stack.script(),
                "replay",
                dir.to_str().unwrap(),
                &stack.topo.sender_ips[0],
                &stack.topo.receiver_ip,
            ],
        )?;
        wait_until(Duration::from_secs(5), || {
            Ok(injector
                .check_exit()
                .map(|status| {
                    assert_eq!(status.0, Some(0), "replay: {:?}", injector.log_snapshot());
                    true
                })
                .unwrap_or(false))
        })?;
        // Keep the reaped handle alive: NamespaceProcess::drop tears down its namespace.
        stack.processes.push(injector);
    }
    // Real TSBPD playout is 2 seconds; allow the final frame and duplicate to settle.
    std::thread::sleep(Duration::from_secs(4));
    stack.stop(&dir)?;
    let data = packets(&dir.join("loopback.pcap"))?;
    let sequences: HashSet<_> = data.iter().map(|p| &p[..4]).collect();
    let result = RunResult {
        variant: variant.to_owned(),
        duplicate_sequences: data.len() - sequences.len(),
        unique_sequences: sequences.len(),
        retransmit_flags: data.iter().filter(|p| p[4] & 4 != 0).count(),
        unregistered_ingress: packets(&dir.join("unregistered.pcap"))?.len(),
        bytes_equal: std::fs::read(dir.join("source.bin"))? == std::fs::read(dir.join("sink.bin"))?,
        source_bytes: std::fs::metadata(dir.join("source.bin"))?.len(),
        sink_bytes: std::fs::metadata(dir.join("sink.bin"))?.len(),
        stats: Stats::read(&dir.join("stats.csv"))?,
    };
    std::fs::write(dir.join("result.json"), serde_json::to_vec_pretty(&result)?)?;
    eprintln!("{}", serde_json::to_string(&result)?);
    Ok(result)
}

#[test]
#[ignore = "requires sudo, CeraLive SRTLA_REPO, tcpdump, srt-live-transmit and ~70 seconds"]
fn registered_duplicates_are_deduplicated_by_libsrt() -> Result<()> {
    // Given fresh two-link bonds, a real SRT stream, and a known receiver build.
    let temp = tempfile::Builder::new()
        .prefix("srtla-dup-spike-")
        .tempdir()?;
    let root = match std::env::var_os("DUP_SPIKE_OUTPUT") {
        Some(path) => {
            let path = std::path::PathBuf::from(path);
            std::fs::create_dir_all(&path)?;
            path
        }
        None => temp.path().to_owned(),
    };
    // When DATA copies traverse the other registered uplink with the R bit clear/set.
    let clear = run(&root, "clear")?;
    if std::env::var_os("DUP_SPIKE_EXPECT_DAMAGE").is_some() {
        ensure!(
            !clear.bytes_equal || clear.stats.dropped > 0,
            "corrupt sequence was not detected"
        );
        return Ok(());
    }
    let set = run(&root, "set")?;
    let control = run(&root, "unregistered")?;
    // Then only libsrt dedups the registered copies; SRTLA rejects the fresh tuple.
    for result in [&clear, &set, &control] {
        ensure!(
            result.bytes_equal,
            "{}: sink stream differs",
            result.variant
        );
        ensure!(
            result.source_bytes == 3000 * 1316,
            "source must contain 15s at 200pps"
        );
        ensure!(
            result.unique_sequences == 3000,
            "pcap must cover all source DATA"
        );
        ensure!(
            result.stats.unique == 3000,
            "stats must cover the entire stream"
        );
    }
    ensure!(
        clear.duplicate_sequences >= 250 && set.duplicate_sequences >= 250,
        "copies never reached libsrt"
    );
    ensure!(
        clear.retransmit_flags == 0 && set.retransmit_flags >= 250,
        "R bit did not reach libsrt as configured"
    );
    ensure!(
        control.unregistered_ingress == 50,
        "negative replay did not reach SRTLA"
    );
    ensure!(
        control.duplicate_sequences == 0,
        "unregistered DATA reached libsrt"
    );
    let choice = measurement::wire_form(&clear.stats, &set.stats);
    let decision = format!("PROBE_WIRE_FORM=retransmit_bit_{choice}\n");
    std::fs::write(root.join("decision.txt"), &decision)?;
    eprintln!("{decision}");
    Ok(())
}
