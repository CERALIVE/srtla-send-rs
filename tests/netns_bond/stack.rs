use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use network_sim::bond::BondTopology;
use network_sim::harness::{
    ReceiverKind, SrtProfile, find_srt_live_transmit_binary, find_srtla_rec_binary,
};
use network_sim::{
    ImpairmentConfig, NamespaceProcess, check_binary, check_integration_deps,
    wait_for_registered_uplinks, wait_for_udp_listener,
};

pub fn available() -> bool {
    match check_integration_deps() {
        Ok(()) => {}
        Err(reason) => {
            eprintln!("Skipping: {reason}");
            return false;
        }
    }
    for tool in ["iptables", "python3"] {
        if check_binary(tool).is_none() {
            eprintln!("Skipping: missing {tool}");
            return false;
        }
    }
    true
}

// Handles precede the topology: NamespaceProcess kills exact namespace PIDs on drop.
pub struct Stack {
    pub sender: NamespaceProcess,
    processes: Vec<NamespaceProcess>,
    pub topo: BondTopology,
}

impl Stack {
    pub fn start(topo: BondTopology) -> Result<Self> {
        for i in 0..topo.link_count() {
            topo.apply_impairment(
                i,
                &ImpairmentConfig {
                    rate_kbit: Some(2000),
                    delay_ms: Some(25),
                    tbf_shaping: true,
                    ..Default::default()
                },
            )?;
        }
        let binary = find_srt_live_transmit_binary()?;
        let argv = SrtProfile::LEGACY_DEFAULT.listener_argv(4001, None)?;
        let server = NamespaceProcess::spawn(
            &topo.receiver_ns,
            binary.to_str().context("listener path")?,
            &argv.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topo.receiver_ns, 4001, Duration::from_secs(5))?;
        let binary = find_srtla_rec_binary()?;
        let argv = ReceiverKind::from_env()?.argv(5000, "127.0.0.1", 4001);
        let receiver = NamespaceProcess::spawn(
            &topo.receiver_ns,
            binary.to_str().context("receiver path")?,
            &argv.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topo.receiver_ns, 5000, Duration::from_secs(5))?;
        let sender = topo.spawn_sender(env!("CARGO_BIN_EXE_srtla_send"), &["--mode", "classic"])?;
        Ok(Self {
            sender,
            processes: vec![server, receiver],
            topo,
        })
    }

    pub fn registered(&self, count: usize) -> Result<()> {
        wait_for_registered_uplinks(&self.sender, count, Duration::from_secs(30))
    }

    pub fn start_traffic(&mut self) -> Result<()> {
        let script = r#"import socket,struct,time
s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM)
start=time.monotonic()
for seq in range(90000):
    s.sendto(struct.pack('!I',seq)+bytes(1312),('127.0.0.1',5555))
    time.sleep(max(0,start+(seq+1)/1000-time.monotonic()))
"#;
        self.processes.push(NamespaceProcess::spawn(
            &self.topo.sender_ns,
            "python3",
            &["-c", script],
        )?);
        Ok(())
    }

    pub fn counters(&self) -> Result<Vec<u64>> {
        (0..self.topo.link_count())
            .map(|i| self.topo.tx_bytes(i))
            .collect()
    }

    pub fn wait_for_carriage(&self, before: &[u64]) -> Result<Vec<u64>> {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let delta = self
                .counters()?
                .iter()
                .zip(before)
                .map(|(end, start)| end - start)
                .collect::<Vec<_>>();
            if delta.iter().all(|bytes| *bytes >= 100_000) {
                return Ok(delta);
            }
            if Instant::now() >= deadline {
                bail!(
                    "carriage timed out: {delta:?}\n{}",
                    self.sender.log_snapshot().join("\n")
                );
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}
