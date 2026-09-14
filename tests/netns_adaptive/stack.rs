use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::bond::{BondTopology, LinkSpec, MappingMode};
use network_sim::harness::{ReceiverKind, find_srt_live_transmit_binary, find_srtla_rec_binary};
use network_sim::metrics::sink::{self, SinkSeries};
use network_sim::scenarios::Profile;
use network_sim::{
    NamespaceProcess, check_binary, check_integration_deps, wait_for_registered_uplinks,
    wait_for_udp_listener,
};

use crate::source::Source;

pub fn available() -> bool {
    if let Err(reason) = check_integration_deps() {
        eprintln!("Skipping: {reason}");
        return false;
    }
    for tool in ["iptables", "python3", "mkfifo"] {
        if check_binary(tool).is_none() {
            eprintln!("Skipping: missing {tool}");
            return false;
        }
    }
    true
}

// Process-only RAII guards drop before namespaces, including on assertion failure.
pub struct Stack {
    pub registered_uplinks: usize,
    pub source: Source,
    pub sender: NamespaceProcess,
    pub receiver: NamespaceProcess,
    pub auxiliaries: Vec<NamespaceProcess>,
    pub topo: BondTopology,
    pub stats: PathBuf,
    pub control: PathBuf,
    pub sink_path: PathBuf,
    pub directory: tempfile::TempDir,
    _binary_directory: tempfile::TempDir,
}

impl Stack {
    pub fn start(profile: &Profile, mapping: MappingMode, shared: bool) -> Result<Self> {
        let directory = tempfile::Builder::new().prefix("adpt-").tempdir()?;
        let links: Vec<_> = profile
            .timeline
            .links
            .iter()
            .enumerate()
            .map(|(i, link)| LinkSpec {
                carrier: link.carrier,
                shared_ip_with: (shared && i == 1).then_some(0),
            })
            .collect();
        let registered = if matches!(mapping, MappingMode::LegacyControl) {
            1
        } else {
            links.len()
        };
        let topo = BondTopology::new("adpt", &links, mapping)?;
        for (i, link) in profile.timeline.links.iter().enumerate() {
            topo.apply_impairment(i, &link.base)?;
        }
        let sink_path = directory.path().join("sink.csv");
        let sink = sink::spawn(&topo.receiver_ns, 9999, &sink_path)?;
        wait_for_udp_listener(&topo.receiver_ns, 9999, Duration::from_secs(5))?;
        let srt = find_srt_live_transmit_binary()?;
        let args = profile.srt_profile.listener_argv(4001, None)?;
        let listener = NamespaceProcess::spawn_process_only(
            &topo.receiver_ns,
            utf8(&srt)?,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topo.receiver_ns, 4001, Duration::from_secs(5))?;
        ensure!(
            matches!(ReceiverKind::from_env()?, ReceiverKind::CeraLive),
            "adaptive pinning requires the CeraLive receiver"
        );
        let receiver_binary = find_srtla_rec_binary()?;
        eprintln!("adaptive receiver={}", receiver_binary.display());
        let args = ReceiverKind::CeraLive.argv(5000, "127.0.0.1", 4001);
        let receiver = NamespaceProcess::spawn_process_only(
            &topo.receiver_ns,
            utf8(&receiver_binary)?,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topo.receiver_ns, 5000, Duration::from_secs(5))?;
        let stats = directory.path().join("stats.json");
        let control = directory.path().join("c.sock");
        // Same inode as Cargo's artifact, private comm: unrelated production killall is isolated.
        let binary = Path::new(env!("CARGO_BIN_EXE_srtla_send"));
        let binary_directory = tempfile::Builder::new()
            .prefix("adpt-bin-")
            .tempdir_in(binary.parent().context("binary parent")?)?;
        let alias = binary_directory
            .path()
            .join(format!("adpt-{}", std::process::id()));
        std::fs::hard_link(binary, &alias)?;
        let mut args = vec![
            "-i".to_owned(),
            "RUST_LOG=info,srtla_send::connection::rtt=debug,\
             srtla_send::connection::ack_nak=error,srtla_send::connection::congestion=error,\
             srtla_send::sender::housekeeping=debug"
                .to_owned(),
            utf8(&alias)?.to_owned(),
        ];
        args.extend(topo.sender_args(
            (5555, 5000),
            &[
                "--mode",
                "adaptive",
                "--stats-file",
                utf8(&stats)?,
                "--control-socket",
                utf8(&control)?,
            ],
        )?);
        let sender = NamespaceProcess::spawn_process_only(
            &topo.sender_ns,
            "env",
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_registered_uplinks(&sender, registered, Duration::from_secs(30))?;
        ensure!(
            std::process::Command::new("sudo")
                .args(["-n", "chmod", "0666", utf8(&control)?])
                .status()?
                .success(),
            "control socket access"
        );
        let caller_uri = format!(
            "srt://127.0.0.1:5555?mode=caller&latency={}&lossmaxttl={}",
            profile.srt_profile.latency_ms, profile.srt_profile.lossmaxttl
        );
        let caller = NamespaceProcess::spawn_process_only(
            &topo.sender_ns,
            utf8(&srt)?,
            &["udp://:6000", &caller_uri],
        )?;
        wait_for_udp_listener(&topo.sender_ns, 6000, Duration::from_secs(5))?;
        let source = Source::start(
            &topo.sender_ns,
            &directory.path().join("source.pipe"),
            profile.warmup_offered_bps,
        )?;
        let stack = Self {
            registered_uplinks: registered,
            source,
            sender,
            receiver,
            auxiliaries: vec![caller, listener, sink],
            topo,
            stats,
            control,
            sink_path,
            directory,
            _binary_directory: binary_directory,
        };
        // A real ten-second prehistory, not synthetic DATA without SRT ACKs.
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(200));
        }
        ensure!(
            stack.sink()?.buckets().iter().any(|b| b.bytes > 0),
            "real SRT source never reached sink"
        );
        Ok(stack)
    }

    pub fn sink(&self) -> Result<SinkSeries> {
        read_sink(&self.sink_path)
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        if let Ok(root) = std::env::var("NETNS_ADAPTIVE_ARTIFACT_DIR") {
            let output = Path::new(&root).join(&self.topo.sender_ns.name);
            let save = || -> Result<()> {
                std::fs::create_dir_all(&output)?;
                for i in 0..self.topo.link_count() {
                    let qdisc = self.topo.sender_ns.exec_checked(
                        "tc",
                        &[
                            "-s",
                            "-j",
                            "qdisc",
                            "show",
                            "dev",
                            self.topo.sender_iface(i),
                        ],
                    )?;
                    std::fs::write(output.join(format!("qdisc-{i}.json")), qdisc.stdout)?;
                }
                for entry in std::fs::read_dir(self.directory.path())? {
                    let entry = entry?;
                    if entry.file_type()?.is_file() {
                        std::fs::copy(entry.path(), output.join(entry.file_name()))?;
                    }
                }
                std::fs::write(
                    output.join("sender-tail.log"),
                    self.sender.log_snapshot().join("\n"),
                )?;
                std::fs::write(
                    output.join("source.log"),
                    self.source.process.log_snapshot().join("\n"),
                )?;
                for (i, process) in self.auxiliaries.iter().enumerate() {
                    std::fs::write(
                        output.join(format!("aux-{i}.log")),
                        process.log_snapshot().join("\n"),
                    )?;
                }
                Ok(())
            };
            if let Err(error) = save() {
                eprintln!("artifact save failed: {error:#}");
            }
            eprintln!("adaptive artifacts={}", output.display());
        }
    }
}

pub fn read_sink(path: &Path) -> Result<SinkSeries> {
    let text = std::fs::read_to_string(path)?;
    // Only consume flushed complete rows while the sink writer is running.
    let end = text.rfind('\n').context("sink header")? + 1;
    Ok(SinkSeries::parse(&text[..end], 0)?)
}

pub fn utf8(path: &Path) -> Result<&str> {
    path.to_str().context("UTF-8 test path")
}
