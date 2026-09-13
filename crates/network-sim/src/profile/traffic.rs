use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};

use crate::bond::{BondTopology, RECEIVER_IP};
use crate::{NamespaceProcess, check_binary, wait_for_udp_listener};

pub struct CrossTraffic {
    client: NamespaceProcess,
    server: NamespaceProcess,
}

impl CrossTraffic {
    pub fn start(topology: &BondTopology, link: usize, mbit: u32) -> Result<Self> {
        match check_binary("iperf3") {
            Some(_) => Self::iperf(topology, link, mbit),
            None => Self::start_python(topology, link, mbit),
        }
    }

    fn iperf(topology: &BondTopology, link: usize, mbit: u32) -> Result<Self> {
        ensure!(
            mbit > 0 && link < topology.link_count(),
            "invalid cross-traffic rate/link"
        );
        let port = (5201 + u16::try_from(link)?).to_string();
        let mut server = NamespaceProcess::spawn_process_only(
            &topology.receiver_ns,
            "iperf3",
            &["-s", "-p", &port],
        )?;
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let out = topology
                .receiver_ns
                .exec_checked("ss", &["-H", "-ltn", "sport", "=", &format!(":{port}")])?;
            if !out.stdout.is_empty() {
                break;
            }
            ensure!(
                server.is_alive() && Instant::now() < deadline,
                "iperf server failed readiness"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let client = NamespaceProcess::spawn_process_only(
            &topology.sender_ns,
            "iperf3",
            &[
                "-c",
                RECEIVER_IP,
                "-p",
                &port,
                "-u",
                "-b",
                &format!("{mbit}M"),
                "-B",
                topology.sender_ip(link),
                "--bind-dev",
                topology.sender_iface(link),
                "-l",
                "1200",
                "-t",
                "0",
            ],
        )?;
        Ok(Self { client, server })
    }

    /// Explicit fallback entrypoint lets privileged tests exercise it even with iperf installed.
    pub fn start_python(topology: &BondTopology, link: usize, mbit: u32) -> Result<Self> {
        ensure!(
            mbit > 0 && link < topology.link_count(),
            "invalid cross-traffic rate/link"
        );
        let port = 5201 + u16::try_from(link)?;
        let server_script = r#"import socket,sys
with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as sock:
    sock.bind(('0.0.0.0',int(sys.argv[1])))
    while True:
        sock.recvfrom(65536)
"#;
        let server = NamespaceProcess::spawn_process_only(
            &topology.receiver_ns,
            "python3",
            &["-c", server_script, &port.to_string()],
        )?;
        wait_for_udp_listener(&topology.receiver_ns, port, Duration::from_secs(3))?;
        let script = r#"import socket,sys,time
with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as sock:
    sock.setsockopt(socket.SOL_SOCKET,socket.SO_BINDTODEVICE,sys.argv[1].encode()+b'\0')
    sock.bind((sys.argv[2],0))
    peer=('10.99.0.1',int(sys.argv[3]))
    interval=1200*8/(int(sys.argv[4])*1000000)
    deadline=time.monotonic()
    while True:
        sock.sendto(bytes(1200),peer)
        deadline+=interval
        time.sleep(max(0,deadline-time.monotonic()))
"#;
        let client = NamespaceProcess::spawn_process_only(
            &topology.sender_ns,
            "python3",
            &[
                "-c",
                script,
                topology.sender_iface(link),
                topology.sender_ip(link),
                &port.to_string(),
                &mbit.to_string(),
            ],
        )?;
        Ok(Self { client, server })
    }

    pub fn stop(&mut self) -> Result<()> {
        self.client
            .stop_process_only()
            .context("stop cross-traffic client")?;
        self.server
            .stop_process_only()
            .context("stop cross-traffic server")
    }
}

impl Drop for CrossTraffic {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            tracing::warn!(%error, "cross-traffic process-only teardown failed");
        }
    }
}
