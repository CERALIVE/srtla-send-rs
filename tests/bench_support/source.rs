use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result, ensure};
use network_sim::{Namespace, NamespaceProcess};

pub struct Source {
    pub process: NamespaceProcess,
    control: File,
    #[allow(dead_code)]
    offered: Option<PathBuf>,
}

impl Source {
    pub fn start(ns: &Namespace, pipe: &Path, warmup_bps: u64) -> Result<Self> {
        Self::start_capture(ns, pipe, (warmup_bps, None))
    }

    #[allow(dead_code)]
    pub fn start_sls(ns: &Namespace, pipe: &Path, offered: PathBuf) -> Result<Self> {
        std::fs::File::create(&offered)?;
        Self::start_capture(ns, pipe, (0, Some(offered)))
    }

    fn start_capture(ns: &Namespace, pipe: &Path, source: (u64, Option<PathBuf>)) -> Result<Self> {
        let (warmup_bps, offered) = source;
        ensure!(
            std::process::Command::new("mkfifo")
                .arg(pipe)
                .status()?
                .success(),
            "mkfifo"
        );
        let control = OpenOptions::new().read(true).write(true).open(pipe)?;
        let process = NamespaceProcess::spawn_process_only(
            ns,
            "python3",
            &[
                "-u",
                "-c",
                PROGRAM,
                pipe.to_str().context("control pipe UTF-8")?,
                &warmup_bps.to_string(),
                "6000",
                offered
                    .as_deref()
                    .map(|p| p.to_str().context("offered path UTF-8"))
                    .transpose()?
                    .unwrap_or(""),
            ],
        )?;
        Ok(Self {
            process,
            control,
            offered,
        })
    }

    #[allow(dead_code)]
    pub fn offered_bytes(&self) -> Result<u64> {
        Ok(std::fs::metadata(
            self.offered
                .as_ref()
                .context("source has no byte capture")?,
        )?
        .len())
    }

    pub fn rate(&mut self, bps: u64, ramp: Duration) -> Result<()> {
        writeln!(self.control, "{bps},{}", ramp.as_millis())?;
        Ok(())
    }
}

// Monotonic deadlines avoid relative-sleep drift. FIFO ramp commands do not add timeline events.
pub const PROGRAM: &str = r#"
import os
import select
import socket
import sys
import time
from contextlib import ExitStack

def run() -> None:
    rate = float(sys.argv[2])
    port = int(sys.argv[3])
    with ExitStack() as resources:
        control = resources.enter_context(open(sys.argv[1], 'rb', buffering=0))
        sock = resources.enter_context(socket.socket(socket.AF_INET, socket.SOCK_DGRAM))
        capture = resources.enter_context(open(sys.argv[4], 'wb', buffering=0)) if len(sys.argv) > 4 and sys.argv[4] else None
        os.set_blocking(control.fileno(), False)
        payload = (b'\x47\x1f\xff\x10' + b'\xff' * 184) * 7 if capture else bytes(1316)
        pending = b''
        deadline = time.monotonic()
        ramp_start = deadline
        ramp_end = deadline
        ramp_from = rate
        ramp_to = rate
        print('ready', flush=True)
        while True:
            now = time.monotonic()
            if ramp_end > ramp_start:
                progress = min(1.0, max(0.0, (now - ramp_start) / (ramp_end - ramp_start)))
                rate = ramp_from + (ramp_to - ramp_from) * progress
            wait = max(0.0, min(0.001, deadline - now)) if rate > 0 else 0.001
            readable, _, _ = select.select([control], [], [], wait)
            if readable:
                chunk = control.read(4096)
                if chunk:
                    pending += chunk
                    while b'\n' in pending:
                        line, pending = pending.split(b'\n', 1)
                        target, milliseconds = (float(value) for value in line.split(b','))
                        ramp_from = rate
                        ramp_to = target
                        ramp_start = time.monotonic()
                        ramp_end = ramp_start + milliseconds / 1000
                        if milliseconds == 0:
                            rate = target
                        deadline = ramp_start
                        print(f'rate,{target},{milliseconds}', flush=True)
            now = time.monotonic()
            if rate > 0 and now >= deadline:
                sock.sendto(payload, ('127.0.0.1', port))
                if capture:
                    capture.write(payload)
                deadline = max(deadline + len(payload) * 8 / rate, now - 0.001)

run()
"#;

#[test]
fn control_pipe_starts_real_udp_from_idle_without_restarting_source() {
    use std::io::{BufRead, BufReader};
    use std::process::{Child, Command, Stdio};
    struct Process(Child);
    impl Drop for Process {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    // Given the real generator, an idle rate and a unique FIFO/UDP socket.
    let directory = tempfile::tempdir().unwrap();
    let pipe = directory.path().join("source.pipe");
    assert!(
        Command::new("mkfifo")
            .arg(&pipe)
            .status()
            .unwrap()
            .success()
    );
    let mut control = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&pipe)
        .unwrap();
    let sink = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    sink.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut process = Process(
        Command::new("python3")
            .args(["-u", "-c", PROGRAM])
            .arg(&pipe)
            .arg("0")
            .arg(sink.local_addr().unwrap().port().to_string())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut reader = BufReader::new(process.0.stdout.take().unwrap());
    let mut ready = String::new();
    reader.read_line(&mut ready).unwrap();
    assert_eq!(ready.trim(), "ready");
    // When a rate command is sent, then the same process delivers a full payload.
    writeln!(control, "1000000,0").unwrap();
    let mut payload = [1_u8; 1500];
    assert_eq!(sink.recv(&mut payload).unwrap(), 1316);
    assert_eq!(&payload[..1316], &[0_u8; 1316]);
}
