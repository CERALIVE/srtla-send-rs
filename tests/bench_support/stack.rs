use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, ensure};
use network_sim::bond::{BondTopology, LinkSpec, MappingMode};
use network_sim::scenarios::Profile;
use network_sim::{NamespaceProcess, wait_for_registered_uplinks, wait_for_udp_listener};

use super::clock::wait_log;
use super::record::{Request, binary_hash, receiver_kind};
use super::source::Source;
use crate::manifest;

pub struct Stack {
    pub source: Source,
    pub sender: NamespaceProcess,
    pub receiver: NamespaceProcess,
    pub listener: NamespaceProcess,
    pub caller: NamespaceProcess,
    pub sink: NamespaceProcess,
    pub pcaps: Vec<(NamespaceProcess, PathBuf)>,
    pub control: PathBuf,
    pub stats: PathBuf,
    pub sink_origin_ms: i64,
    pub topo: BondTopology,
    _control_directory: tempfile::TempDir,
}

impl Stack {
    pub fn start(request: &Request, profile: &Profile) -> Result<Self> {
        let cell = &request.manifest.cells[request.work.cell];
        let candidate = request
            .manifest
            .candidates
            .iter()
            .find(|c| c.label == cell.candidate)
            .context("candidate")?;
        let receiver = request
            .manifest
            .receivers
            .iter()
            .find(|r| r.name == cell.receiver)
            .context("receiver")?;
        ensure!(
            binary_hash(&candidate.bin)? == request.result.record.candidate.bin_sha256,
            "candidate binary changed after fingerprinting"
        );
        ensure!(
            binary_hash(&receiver.bin)? == request.result.record.receiver.sha256,
            "receiver binary changed after fingerprinting"
        );
        let links: Vec<_> = profile
            .timeline
            .links
            .iter()
            .map(|link| LinkSpec {
                carrier: link.carrier,
                shared_ip_with: None,
            })
            .collect();
        let topo = BondTopology::new("bench", &links, MappingMode::None)?;
        for (i, link) in profile.timeline.links.iter().enumerate() {
            topo.apply_impairment(i, &link.base)?;
        }
        let mut pcaps = Vec::new();
        if std::env::var("BENCH_PCAP_ON_FAIL").as_deref() == Ok("1") {
            for i in 0..topo.link_count() {
                let path = request.artifacts.join(format!("link-{i}.pcap"));
                let process = NamespaceProcess::spawn_process_only(
                    &topo.sender_ns,
                    "tcpdump",
                    &[
                        "-U",
                        "-n",
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
        let sink = network_sim::metrics::sink::spawn(
            &topo.receiver_ns,
            9999,
            &request.result.record.raw.sink_series_path,
        )?;
        let ready = wait_log(&sink, "ready,9999,", Duration::from_secs(5))?;
        let sink_origin_ms = ready
            .rsplit(',')
            .next()
            .context("sink origin")?
            .parse::<i64>()?
            / 1_000_000;
        let listener_args = manifest::preset(&cell.srt_profile)?
            .listener_argv(4001, Some(&request.result.record.raw.stats_csv_path))?;
        let listener = NamespaceProcess::spawn_process_only(
            &topo.receiver_ns,
            utf8(&request.srt_binary)?,
            &listener_args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topo.receiver_ns, 4001, Duration::from_secs(5))?;
        let args = receiver_kind(receiver)?.argv(5000, "127.0.0.1", 4001);
        let receiver = NamespaceProcess::spawn_process_only(
            &topo.receiver_ns,
            utf8(&receiver.bin)?,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topo.receiver_ns, 5000, Duration::from_secs(5))?;
        let control_directory = tempfile::Builder::new().prefix("bench-ctl-").tempdir()?;
        let control = control_directory.path().join("c.sock");
        crate::checkpoint::atomic(
            &request.artifacts.join("control-directory.json"),
            &control_directory.path(),
        )?;
        let stats = request.artifacts.join("sender.json");
        let mut extra = candidate.args.clone();
        extra.extend(["--control-socket".into(), utf8(&control)?.into()]);
        if candidate.stats_file {
            extra.extend(["--stats-file".into(), utf8(&stats)?.into()]);
        }
        let args = topo.sender_args(
            (5555, 5000),
            &extra.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        let mut launch = vec!["-i".into()];
        launch.extend(
            request
                .result
                .record
                .candidate
                .env
                .iter()
                .map(|(k, v)| format!("{k}={v}")),
        );
        launch.push(utf8(&candidate.bin)?.into());
        launch.extend(args);
        let sender = NamespaceProcess::spawn_process_only(
            &topo.sender_ns,
            "env",
            &launch.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topo.sender_ns, 5555, Duration::from_secs(5))?;
        wait_for_registered_uplinks(&sender, links.len(), Duration::from_secs(30))?;
        if control.exists() {
            ensure!(
                std::process::Command::new("sudo")
                    .args(["-n", "chmod", "0666", utf8(&control)?])
                    .status()?
                    .success(),
                "control socket access"
            );
        }
        let preset = manifest::preset(&cell.srt_profile)?;
        let caller_uri = if preset == network_sim::harness::SrtProfile::LEGACY_DEFAULT {
            "srt://127.0.0.1:5555?mode=caller".into()
        } else {
            format!(
                "srt://127.0.0.1:5555?mode=caller&latency={}&lossmaxttl={}",
                preset.latency_ms, preset.lossmaxttl
            )
        };
        let caller = NamespaceProcess::spawn_process_only(
            &topo.sender_ns,
            utf8(&request.srt_binary)?,
            &["udp://:6000", &caller_uri],
        )?;
        wait_for_udp_listener(&topo.sender_ns, 6000, Duration::from_secs(5))?;
        let source = Source::start(&topo.sender_ns, &request.artifacts.join("source.pipe"), 0)?;
        wait_log(&source.process, "ready", Duration::from_secs(5))?;
        Ok(Self {
            source,
            sender,
            receiver,
            listener,
            caller,
            sink,
            pcaps,
            control,
            stats,
            sink_origin_ms,
            topo,
            _control_directory: control_directory,
        })
    }

    pub fn logs(&self, directory: &Path) -> Result<()> {
        for (name, process) in [
            ("source", &self.source.process),
            ("sender", &self.sender),
            ("receiver", &self.receiver),
            ("listener", &self.listener),
            ("caller", &self.caller),
            ("sink", &self.sink),
        ] {
            std::fs::write(
                directory.join(format!("{name}.log")),
                process.log_snapshot().join("\n"),
            )?;
        }
        Ok(())
    }

    pub fn finish_pcaps(&mut self, keep: bool) -> Result<()> {
        for (process, path) in &mut self.pcaps {
            process.stop_process_only()?;
            if !keep {
                std::fs::remove_file(path)?;
            }
        }
        Ok(())
    }
}

pub fn utf8(path: &Path) -> Result<&str> {
    path.to_str().context("benchmark path must be UTF-8")
}
