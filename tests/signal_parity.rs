//! Signal & startup parity integration tests (Unix).
//!
//! These spawn the real `srtla_send` binary and drive it with POSIX signals to
//! pin the CeraLive signal/startup contract CeraUI depends on:
//!   * a missing / empty `BIND_IPS_FILE` at startup yields an EMPTY uplink pool
//!     (listener still bound), not a crash;
//!   * a SIGHUP that follows the empty start adds the newly listed uplinks;
//!   * SIGTERM and SIGINT exit cleanly (code 0) well inside CeraUI's 10s
//!     SIGKILL window.
#![cfg(unix)]

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_srtla_send");
const START_TIMEOUT: Duration = Duration::from_secs(5);
const LOG_TIMEOUT: Duration = Duration::from_secs(5);
const KILL_WINDOW: Duration = Duration::from_secs(10);
/// Clean shutdown must be prompt: the contract says "well within" CeraUI's 10s
/// SIGKILL window. Poll up to `KILL_WINDOW` so a slow exit is reported as a
/// too-slow failure, and additionally assert this tighter bound.
const PROMPT_EXIT: Duration = Duration::from_secs(2);

const EMPTY_START_LOG: &str = "no valid source IPs at startup";

fn free_udp_port() -> u16 {
    let sock = std::net::UdpSocket::bind("127.0.0.1:0").expect("bind ephemeral udp");
    sock.local_addr().unwrap().port()
}

struct SenderProc {
    child: Child,
    logs: Arc<Mutex<String>>,
}

impl SenderProc {
    fn spawn(args: &[&str]) -> Self {
        let mut child = Command::new(BIN)
            .args(args)
            .env("RUST_LOG", "info")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn srtla_send");

        // tracing writes to stdout; panics/anyhow errors land on stderr. Pump both
        // into one buffer so log assertions see everything the binary emitted.
        let logs = Arc::new(Mutex::new(String::new()));
        let streams: [Box<dyn std::io::Read + Send>; 2] = [
            Box::new(child.stdout.take().expect("capture stdout")),
            Box::new(child.stderr.take().expect("capture stderr")),
        ];
        for stream in streams {
            let sink = logs.clone();
            thread::spawn(move || {
                for line in BufReader::new(stream).lines().map_while(Result::ok) {
                    let mut buf = sink.lock().unwrap();
                    buf.push_str(&line);
                    buf.push('\n');
                }
            });
        }

        Self { child, logs }
    }

    fn logs(&self) -> String {
        self.logs.lock().unwrap().clone()
    }

    fn wait_for_log(&self, needle: &str, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.logs().contains(needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(50));
        }
        false
    }

    fn signal(&self, sig: &str) {
        let status = Command::new("kill")
            .args([format!("-{sig}"), self.child.id().to_string()])
            .status()
            .expect("run kill");
        assert!(status.success(), "kill -{sig} failed");
    }

    fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn wait_exit(&mut self, timeout: Duration) -> Option<i32> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(status)) => return Some(status.code().unwrap_or(-1)),
                Ok(None) => thread::sleep(Duration::from_millis(20)),
                Err(_) => return None,
            }
        }
        None
    }
}

impl Drop for SenderProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Spawn a sender against `ips`, bound to `127.0.0.1`, and return the process
/// plus the ips-file path so a test can rewrite it before signalling.
fn spawn(dir: &std::path::Path, ips_contents: &str) -> (SenderProc, std::path::PathBuf) {
    let ips = dir.join("ips.txt");
    std::fs::write(&ips, ips_contents).expect("write ips file");
    let port = free_udp_port().to_string();
    let proc = SenderProc::spawn(&[
        port.as_str(),
        "127.0.0.1",
        "9999",
        ips.to_str().unwrap(),
        "--verbose",
    ]);
    (proc, ips)
}

#[test]
fn empty_ip_file_at_startup_not_fatal() {
    let dir = tempfile::tempdir().unwrap();
    // The file exists but resolves to zero valid IPs (blank lines only).
    let (mut proc, _ips) = spawn(dir.path(), "\n   \n\n");

    assert!(
        proc.wait_for_log(EMPTY_START_LOG, START_TIMEOUT),
        "an empty ips file must start an empty pool with the empty-start log; logs:\n{}",
        proc.logs()
    );
    assert!(
        proc.wait_for_log("listening for SRT", START_TIMEOUT),
        "the listener must be bound even with no uplinks; logs:\n{}",
        proc.logs()
    );
    thread::sleep(Duration::from_secs(1));
    assert!(
        proc.is_alive(),
        "empty-start must keep running (no crash-loop); logs:\n{}",
        proc.logs()
    );

    proc.signal("TERM");
    assert_eq!(
        proc.wait_exit(KILL_WINDOW),
        Some(0),
        "SIGTERM after an empty start must still exit 0; logs:\n{}",
        proc.logs()
    );
}

#[test]
fn missing_ips_file_at_startup_starts_empty_and_reloads_on_sighup() {
    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips.txt"); // intentionally not created
    let port = free_udp_port().to_string();
    let mut proc = SenderProc::spawn(&[
        port.as_str(),
        "127.0.0.1",
        "9999",
        ips.to_str().unwrap(),
        "--verbose",
    ]);

    assert!(
        proc.wait_for_log(EMPTY_START_LOG, START_TIMEOUT),
        "a missing ips file must yield an empty pool, not an error exit; logs:\n{}",
        proc.logs()
    );
    assert!(
        proc.wait_for_log("listening for SRT", START_TIMEOUT),
        "the sender must bind the listener even with no uplinks; logs:\n{}",
        proc.logs()
    );
    thread::sleep(Duration::from_secs(1));
    assert!(
        proc.is_alive(),
        "sender must not crash on a missing ips file"
    );

    // Writing the file and signalling must populate the pool.
    std::fs::write(&ips, "127.0.0.1\n").unwrap();
    proc.signal("HUP");
    assert!(
        proc.wait_for_log("added uplink", LOG_TIMEOUT),
        "SIGHUP after writing the file must add the uplink; logs:\n{}",
        proc.logs()
    );
    assert!(proc.is_alive());

    proc.signal("TERM");
    assert_eq!(
        proc.wait_exit(KILL_WINDOW),
        Some(0),
        "SIGTERM after the reload must still exit 0; logs:\n{}",
        proc.logs()
    );
}

#[test]
fn sigterm_exits_cleanly_and_promptly() {
    let dir = tempfile::tempdir().unwrap();
    let (mut proc, _ips) = spawn(dir.path(), "127.0.0.1\n");

    assert!(
        proc.wait_for_log("listening for SRT", START_TIMEOUT),
        "sender never started; logs:\n{}",
        proc.logs()
    );

    let t0 = Instant::now();
    proc.signal("TERM");
    let code = proc.wait_exit(KILL_WINDOW);
    let elapsed = t0.elapsed();

    assert_eq!(code, Some(0), "SIGTERM must exit 0; logs:\n{}", proc.logs());
    assert!(
        elapsed < PROMPT_EXIT,
        "SIGTERM must exit promptly (< {PROMPT_EXIT:?}); took {elapsed:?}"
    );
}

#[test]
fn sigint_exits_cleanly_and_promptly() {
    let dir = tempfile::tempdir().unwrap();
    let (mut proc, _ips) = spawn(dir.path(), "127.0.0.1\n");

    assert!(
        proc.wait_for_log("listening for SRT", START_TIMEOUT),
        "sender never started; logs:\n{}",
        proc.logs()
    );

    let t0 = Instant::now();
    proc.signal("INT");
    let code = proc.wait_exit(KILL_WINDOW);
    let elapsed = t0.elapsed();

    assert_eq!(code, Some(0), "SIGINT must exit 0; logs:\n{}", proc.logs());
    assert!(
        elapsed < PROMPT_EXIT,
        "SIGINT must exit promptly (< {PROMPT_EXIT:?}); took {elapsed:?}"
    );
}
