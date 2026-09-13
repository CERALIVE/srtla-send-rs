use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::{
    NamespaceProcess, SrtlaTestTopology, wait_for_registered_uplinks, wait_for_udp_listener,
};

pub struct Stack {
    pub processes: Vec<NamespaceProcess>,
    pub topo: SrtlaTestTopology,
    captures: [usize; 2],
}

pub fn wait_until(timeout: Duration, mut ready: impl FnMut() -> Result<bool>) -> Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if ready()? {
            return Ok(());
        }
        ensure!(Instant::now() < deadline, "readiness deadline expired");
        std::thread::sleep(Duration::from_millis(50));
    }
}

impl Stack {
    pub const fn script(&self) -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/dup_spike/traffic.py")
    }

    pub fn start(dir: &Path, variant: &str) -> Result<Self> {
        let repo = std::env::var_os("SRTLA_REPO")
            .context("set SRTLA_REPO to the CeraLive receiver checkout")?;
        let repo = std::fs::canonicalize(repo)?;
        let receiver = ["build-pkg/srtla_rec", "build/srtla_rec"]
            .iter()
            .map(|name| repo.join(name))
            .find(|p| p.is_file())
            .context("build the CeraLive receiver in SRTLA_REPO/build-pkg or build first")?;
        let receiver = receiver.canonicalize()?;
        ensure!(
            receiver.starts_with(&repo),
            "receiver must come from SRTLA_REPO"
        );
        std::fs::write(dir.join("receiver-path.txt"), receiver.to_str().unwrap())?;
        let topo = SrtlaTestTopology::new("dup4", 2)?;
        for (ns, ifaces) in [
            (&topo.sender_ns, &topo.sender_ifaces),
            (&topo.receiver_ns, &topo.receiver_ifaces),
        ] {
            for iface in ["all", "default"]
                .into_iter()
                .chain(ifaces.iter().map(String::as_str))
            {
                ns.exec_checked(
                    "sysctl",
                    &["-w", &format!("net.ipv4.conf.{iface}.rp_filter=0")],
                )?;
            }
        }
        topo.sender_ns.exec_checked(
            "ip",
            &[
                "route",
                "add",
                &topo.receiver_ip,
                "dev",
                &topo.sender_ifaces[1],
                "table",
                "102",
            ],
        )?;
        topo.sender_ns.exec_checked(
            "ip",
            &["rule", "add", "from", &topo.sender_ips[1], "lookup", "102"],
        )?;
        topo.receiver_ns.exec_checked(
            "ip",
            &[
                "route",
                "add",
                &topo.sender_ips[1],
                "dev",
                &topo.receiver_ifaces[1],
                "src",
                &topo.receiver_ip,
            ],
        )?;
        let mut stack = Self {
            processes: Vec::new(),
            topo,
            captures: [0, 0],
        };
        let ns = &stack.topo.receiver_ns;
        stack.processes.push(NamespaceProcess::spawn(
            ns,
            "python3",
            &[stack.script(), "sink", dir.to_str().unwrap()],
        )?);
        wait_for_udp_listener(ns, 4100, Duration::from_secs(5))?;
        stack.processes.push(NamespaceProcess::spawn(
            ns,
            "srt-live-transmit",
            &[
                "-s:1",
                "-f",
                "-pf:csv",
                &format!("-statsout:{}", dir.join("stats.csv").display()),
                "srt://:4001?mode=listener&latency=2000&lossmaxttl=40",
                "udp://127.0.0.1:4100",
            ],
        )?);
        wait_for_udp_listener(ns, 4001, Duration::from_secs(5))?;
        stack.processes.push(NamespaceProcess::spawn(
            ns,
            receiver.to_str().unwrap(),
            &[
                "--srtla_port",
                "5000",
                "--srt_hostname",
                "127.0.0.1",
                "--srt_port",
                "4001",
            ],
        )?);
        wait_for_udp_listener(ns, 5000, Duration::from_secs(5))?;
        for (index, iface, name, filter) in [
            (
                0,
                "lo",
                "loopback.pcap",
                "udp dst port 4001 and udp[8] < 0x80",
            ),
            (
                1,
                stack.topo.receiver_ifaces[0].as_str(),
                "unregistered.pcap",
                "udp dst port 5000 and udp src port 49000 and udp[8] < 0x80",
            ),
        ] {
            stack.captures[index] = stack.processes.len();
            stack.processes.push(NamespaceProcess::spawn(
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
                    dir.join(name).to_str().unwrap(),
                    filter,
                ],
            )?);
            let capture = stack.processes.last().unwrap();
            wait_until(Duration::from_secs(5), || {
                Ok(capture
                    .log_snapshot()
                    .iter()
                    .any(|line| line.contains("listening on")))
            })
            .with_context(|| format!("tcpdump readiness: {:?}", capture.log_snapshot()))?;
        }
        let ips = dir.join("ips.txt");
        std::fs::write(&ips, stack.topo.sender_ips.join("\n") + "\n")?;
        let env = match variant {
            "clear" => vec![
                ("RUST_LOG", "info"),
                ("SRTLA_TEST_DUP_EVERY", "10"),
                ("SRTLA_TEST_DUP_RETX_BIT", "0"),
            ],
            "set" => vec![
                ("RUST_LOG", "info"),
                ("SRTLA_TEST_DUP_EVERY", "10"),
                ("SRTLA_TEST_DUP_RETX_BIT", "1"),
            ],
            "unregistered" => vec![("RUST_LOG", "info"), ("SRTLA_TEST_DUP_EVERY", "0")],
            _ => anyhow::bail!("unknown spike variant"),
        };
        stack.processes.push(NamespaceProcess::spawn_with_env(
            &stack.topo.sender_ns,
            env!("CARGO_BIN_EXE_srtla_send"),
            &[
                "4000",
                &stack.topo.receiver_ip,
                "5000",
                ips.to_str().unwrap(),
                "--mode",
                "classic",
            ],
            &env,
        )?);
        let sender = stack.processes.last().unwrap();
        wait_for_udp_listener(&stack.topo.sender_ns, 4000, Duration::from_secs(5))?;
        wait_for_registered_uplinks(sender, 2, Duration::from_secs(15))?;
        stack.processes.push(NamespaceProcess::spawn(
            &stack.topo.sender_ns,
            "srt-live-transmit",
            &[
                "-v",
                "udp://:4002",
                "srt://127.0.0.1:4000?mode=caller&latency=2000&lossmaxttl=40",
            ],
        )?);
        let caller = stack.processes.last().unwrap();
        wait_until(Duration::from_secs(10), || {
            Ok(caller
                .log_snapshot()
                .iter()
                .any(|line| line.contains("SRT target connected")))
        })
        .with_context(|| format!("caller readiness: {:?}", caller.log_snapshot()))?;
        Ok(stack)
    }

    pub fn stop(&mut self, dir: &Path) -> Result<()> {
        for process in self.processes.iter_mut().rev() {
            process.kill();
        }
        for (idx, process) in self.processes.iter().enumerate() {
            std::fs::write(
                dir.join(format!("process-{idx}.log")),
                process.log_snapshot().join("\n"),
            )?;
        }
        for idx in self.captures {
            ensure!(
                self.processes[idx]
                    .log_snapshot()
                    .iter()
                    .any(|line| line == "0 packets dropped by kernel"),
                "capture dropped packets or did not shut down cleanly"
            );
        }
        Ok(())
    }
}
