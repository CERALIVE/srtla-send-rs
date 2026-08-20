//! The three-process stack driven over a [`TwinTopology`].
//!
//! Deliberately separate from [`crate::SrtlaTestStack`]: that one builds its own
//! one-subnet-per-link topology and writes a plain IP list, which is exactly the
//! shape a duplicate-IP bond cannot use. Everything below the process
//! management — `NamespaceProcess`, the registration readiness gate — is shared.

use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use super::publish::{BindMapPublisher, TwinRow};
use super::topology::{RECEIVER_IP, TWIN_IP, TwinTopology};
use crate::harness::{NamespaceProcess, find_srtla_send_binary, wait_for_registered_uplinks};
use crate::wait_for_udp_listener;

const SRT_SERVER_PORT: u16 = 4001;
const SRTLA_REC_PORT: u16 = 5000;
const SRTLA_SEND_SRT_PORT: u16 = 5555;

/// Whether the sender is launched with the ADR-003 sidecar or on the legacy
/// IP-only path. The legacy arm is what makes a twin assertion falsifiable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mapping {
    BindMap,
    Legacy,
}

pub struct TwinStack {
    pub topo: TwinTopology,
    pub publisher: BindMapPublisher,
    rows: Vec<TwinRow>,
    srt_server: Option<NamespaceProcess>,
    srtla_rec: Option<NamespaceProcess>,
    srtla_send: Option<NamespaceProcess>,
    traffic: Option<NamespaceProcess>,
}

impl TwinStack {
    /// Start the stack with one twin per `link_ids` entry, the Nth identity
    /// pinned to the Nth interface.
    pub fn start(test_name: &str, link_ids: &[&str], mapping: Mapping) -> Result<Self> {
        let topo = TwinTopology::new(test_name, link_ids.len())?;
        let publisher = BindMapPublisher::new()?;
        let rows: Vec<TwinRow> = link_ids
            .iter()
            .enumerate()
            .map(|(idx, link_id)| TwinRow::new(link_id, topo.sender_iface(idx)))
            .collect();
        match mapping {
            Mapping::BindMap => {
                publisher.publish(TWIN_IP, &rows)?;
            }
            Mapping::Legacy => publisher.publish_ips_only(TWIN_IP, rows.len())?,
        }

        let (srt_server, srtla_rec) = start_receiver(&topo)?;
        let ips = publisher.ips_path().to_string_lossy().to_string();
        let sidecar = publisher.sidecar_path().to_string_lossy().to_string();
        let send_port = SRTLA_SEND_SRT_PORT.to_string();
        let rec_port = SRTLA_REC_PORT.to_string();
        let mut args = vec![
            send_port.as_str(),
            RECEIVER_IP,
            rec_port.as_str(),
            ips.as_str(),
        ];
        if mapping == Mapping::BindMap {
            args.extend_from_slice(&["--bind-map", sidecar.as_str()]);
        }
        let binary = find_srtla_send_binary()?;
        let srtla_send = NamespaceProcess::spawn_with_env(
            &topo.sender_ns,
            &binary.to_string_lossy(),
            &args,
            &[("RUST_LOG", "debug")],
        )
        .context("start srtla_send")?;

        Ok(Self {
            topo,
            publisher,
            rows,
            srt_server: Some(srt_server),
            srtla_rec: Some(srtla_rec),
            srtla_send: Some(srtla_send),
            traffic: None,
        })
    }

    /// The rows published at startup, with interface names resolved.
    #[must_use]
    pub fn rows(&self) -> &[TwinRow] {
        &self.rows
    }

    /// How this uplink names itself in every log line it appears in.
    #[must_use]
    pub fn label(&self, idx: usize) -> String {
        format!(
            "{RECEIVER_IP}:{SRTLA_REC_PORT} via {TWIN_IP} on {} [{}]",
            self.rows[idx].iface, self.rows[idx].link_id
        )
    }

    /// Republish a coherent pair and signal the sender to reload it.
    pub fn reload(&self, rows: &[TwinRow]) -> Result<()> {
        self.publisher.publish(TWIN_IP, rows)?;
        self.sighup()
    }

    /// Signal `srtla_send` specifically — never every PID in the namespace,
    /// which would also hit a running traffic generator.
    pub fn sighup(&self) -> Result<()> {
        let pids = self.namespace_pids()?;
        let target = pids
            .into_iter()
            .find(|pid| process_name(*pid).as_deref() == Some("srtla_send"))
            .context("srtla_send is not running in the sender namespace")?;
        let status = Command::new("sudo")
            .args(["-n", "kill", "-HUP", "--", &target.to_string()])
            .status()
            .context("signal srtla_send")?;
        if !status.success() {
            bail!("SIGHUP to srtla_send (pid {target}) failed");
        }
        Ok(())
    }

    /// Block until at least `count` uplinks have completed registration.
    pub fn wait_for_registered(&self, count: usize, timeout: Duration) -> Result<()> {
        let process = self
            .srtla_send
            .as_ref()
            .context("srtla_send is not running")?;
        wait_for_registered_uplinks(process, count, timeout)
    }

    /// Block until `needle` appears in the sender's log, returning the snapshot.
    pub fn wait_for_log(&self, needle: &str, timeout: Duration) -> Result<String> {
        let deadline = Instant::now() + timeout;
        loop {
            let log = self.sender_log();
            if log.contains(needle) {
                return Ok(log);
            }
            if Instant::now() >= deadline {
                bail!("timeout waiting for `{needle}` in the srtla_send log:\n{log}");
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    #[must_use]
    pub fn sender_log(&self) -> String {
        self.srtla_send
            .as_ref()
            .map(|p| p.log_snapshot().join("\n"))
            .unwrap_or_default()
    }

    /// Cap every twin so no single link can absorb the whole stream, and give
    /// keepalive round trips enough delay to produce a non-zero RTT sample.
    pub fn shape_all(&self, rate_kbit: u64, delay_ms: u32) -> Result<()> {
        for idx in 0..self.topo.link_count() {
            self.topo.shape(idx, rate_kbit, delay_ms)?;
        }
        Ok(())
    }

    /// Run the local SRT producer for `seconds`, reporting each twin's wire
    /// bytes over the window. Read from the netdev counters, so a *recreated*
    /// interface reports only what it carried since it came back.
    pub fn carried_bytes(&mut self, packets_per_sec: u32, seconds: u64) -> Result<Vec<u64>> {
        let before = (0..self.topo.link_count())
            .map(|idx| self.topo.tx_bytes(idx))
            .collect::<Result<Vec<_>>>()?;
        self.start_traffic(packets_per_sec, seconds)?;
        std::thread::sleep(Duration::from_secs(seconds + 2));
        before
            .iter()
            .enumerate()
            .map(|(idx, start)| Ok(self.topo.tx_bytes(idx)? - start))
            .collect()
    }

    /// Start a steady local SRT producer that outlives the call.
    pub fn start_traffic(&mut self, packets_per_sec: u32, seconds: u64) -> Result<()> {
        let script = format!(
            "import socket,time\ns=socket.socket(socket.AF_INET,socket.SOCK_DGRAM)\nd=b'\\x00'*\
             188\nend=time.time()+{seconds}\nwhile time.time()<end:\n    \
             s.sendto(d,('127.0.0.1',{SRTLA_SEND_SRT_PORT}))\n    \
             time.sleep(1/{packets_per_sec})\n"
        );
        self.traffic = Some(
            NamespaceProcess::spawn(&self.topo.sender_ns, "python3", &["-c", &script])
                .context("start the local SRT producer")?,
        );
        Ok(())
    }

    fn namespace_pids(&self) -> Result<Vec<u32>> {
        let out = Command::new("sudo")
            .args(["-n", "ip", "netns", "pids", &self.topo.sender_ns.name])
            .output()
            .context("list sender-namespace pids")?;
        Ok(String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .filter_map(|pid| pid.parse().ok())
            .collect())
    }
}

impl Drop for TwinStack {
    fn drop(&mut self) {
        drop(self.traffic.take());
        drop(self.srtla_send.take());
        drop(self.srtla_rec.take());
        drop(self.srt_server.take());
    }
}

fn start_receiver(topo: &TwinTopology) -> Result<(NamespaceProcess, NamespaceProcess)> {
    let srt_uri = format!("srt://:{SRT_SERVER_PORT}?mode=listener");
    let mut srt_server = NamespaceProcess::spawn(
        &topo.receiver_ns,
        "srt-live-transmit",
        &[&srt_uri, "udp://127.0.0.1:9999"],
    )
    .context("start srt-live-transmit")?;
    wait_for_udp_listener(&topo.receiver_ns, SRT_SERVER_PORT, Duration::from_secs(5))
        .context("wait for srt-live-transmit")?;
    if let Some((code, stderr)) = srt_server.check_exit() {
        bail!("srt-live-transmit exited immediately (code {code:?})\n{stderr}");
    }

    let srtla_port = SRTLA_REC_PORT.to_string();
    let srt_port = SRT_SERVER_PORT.to_string();
    let mut srtla_rec = NamespaceProcess::spawn(
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
    .context("start srtla_rec")?;
    wait_for_udp_listener(&topo.receiver_ns, SRTLA_REC_PORT, Duration::from_secs(5))
        .context("wait for srtla_rec")?;
    if let Some((code, stderr)) = srtla_rec.check_exit() {
        bail!("srtla_rec exited immediately (code {code:?})\n{stderr}");
    }
    Ok((srt_server, srtla_rec))
}

fn process_name(pid: u32) -> Option<String> {
    std::fs::read_to_string(format!("/proc/{pid}/comm"))
        .ok()
        .map(|name| name.trim().to_string())
}
