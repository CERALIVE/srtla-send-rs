use std::path::PathBuf;

use anyhow::{Result, ensure};
use network_sim::NamespaceProcess;
use network_sim::bond::BondTopology;

use super::record::Request;
use super::stack::utf8;

pub fn start(request: &Request, topo: &BondTopology) -> Result<Vec<(NamespaceProcess, PathBuf)>> {
    let freeze = request.result.record.scenario.id == "S-FREEZE-NORDR";
    if freeze {
        let seed = (42 + request.work.run).to_string();
        let output = topo.sender_ns.exec(
            "tc",
            &[
                "qdisc",
                "change",
                "dev",
                topo.sender_iface(0),
                "parent",
                "10:1",
                "handle",
                "11:",
                "netem",
                "delay",
                "60ms",
                "loss",
                "1%",
                "limit",
                "500",
                "seed",
                &seed,
            ],
        )?;
        ensure!(
            output.status.success(),
            "seed freeze netem: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut pcaps = Vec::new();
    if request.manifest.cells[request.work.cell].variant == "rexmit-capture" {
        let path = request.artifacts.join("caller-srt.pcap");
        let process = NamespaceProcess::spawn_process_only(
            &topo.sender_ns,
            "tcpdump",
            &[
                "-U",
                "-n",
                "-s",
                "128",
                "-i",
                "lo",
                "-w",
                utf8(&path)?,
                "udp",
                "dst",
                "port",
                "5555",
            ],
        )?;
        super::clock::wait_log(
            &process,
            "tcpdump: listening on lo",
            std::time::Duration::from_secs(5),
        )?;
        pcaps.push((process, path));
    }
    if std::env::var("BENCH_PCAP_ON_FAIL").as_deref() == Ok("1") || freeze {
        for i in 0..topo.link_count() {
            let path = request.artifacts.join(format!("link-{i}.pcap"));
            let process = NamespaceProcess::spawn_process_only(
                &topo.sender_ns,
                "tcpdump",
                &[
                    "-U",
                    "-n",
                    "-s",
                    "128",
                    "-i",
                    topo.sender_iface(i),
                    "-w",
                    utf8(&path)?,
                    "udp",
                ],
            )?;
            pcaps.push((process, path));
        }
    }
    if freeze {
        let path = request.artifacts.join("receiver-srt.pcap");
        let process = NamespaceProcess::spawn_process_only(
            &topo.receiver_ns,
            "tcpdump",
            &[
                "-U",
                "-n",
                "-s",
                "256",
                "-i",
                "lo",
                "-w",
                utf8(&path)?,
                "udp",
                "port",
                "4001",
            ],
        )?;
        pcaps.push((process, path));
    }
    Ok(pcaps)
}
