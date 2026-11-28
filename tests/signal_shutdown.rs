//! Signal shutdown integration tests (Unix).
//!
//! These spawn the real `srtla_send` binary and pin that SIGTERM and SIGINT
//! produce a prompt, clean exit (code 0) rather than death by the default
//! signal action, which a supervisor reports as a failure.
#![cfg(unix)]

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_srtla_send");
const START_TIMEOUT: Duration = Duration::from_secs(5);
/// Poll well past `PROMPT_EXIT` so a slow exit is reported as too slow rather
/// than as "never exited".
const EXIT_TIMEOUT: Duration = Duration::from_secs(10);
const PROMPT_EXIT: Duration = Duration::from_secs(2);
/// Local SRT listen port. 0 lets the OS pick a free one at bind time: probing for
/// a free port and handing it over left a window in which another socket, or a
/// parallel test, could take it first.
const ANY_PORT: &str = "0";

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

    /// The sender installs its signal handlers before it logs the listener, so
    /// once "listening for SRT" appears a signal is safe to send.
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

    fn wait_exit(&mut self, timeout: Duration) -> Option<i32> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                // A signal death has no exit code; report it as -1 so it can
                // never be mistaken for the clean 0 these tests require.
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

fn assert_signal_exits_cleanly(sig: &str) {
    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips.txt");
    std::fs::write(&ips, "127.0.0.1\n").expect("write ips file");
    let mut proc = SenderProc::spawn(&[ANY_PORT, "127.0.0.1", "9999", ips.to_str().unwrap()]);

    assert!(
        proc.wait_for_log("listening for SRT", START_TIMEOUT),
        "sender never started; logs:\n{}",
        proc.logs()
    );

    let t0 = Instant::now();
    proc.signal(sig);
    let code = proc.wait_exit(EXIT_TIMEOUT);
    let elapsed = t0.elapsed();

    assert_eq!(
        code,
        Some(0),
        "SIG{sig} must exit 0; logs:\n{}",
        proc.logs()
    );
    assert!(
        elapsed < PROMPT_EXIT,
        "SIG{sig} must exit promptly (< {PROMPT_EXIT:?}); took {elapsed:?}"
    );
}

#[test]
fn sigterm_exits_cleanly_and_promptly() {
    assert_signal_exits_cleanly("TERM");
}

#[test]
fn sigint_exits_cleanly_and_promptly() {
    assert_signal_exits_cleanly("INT");
}

/// A `SIGHUP` with `--bind-map` reads the IP file on the blocking pool. Here
/// that read never finishes: the IP file is swapped for a FIFO whose write end
/// is held open without data. `SIGTERM` must still exit 0 promptly rather than
/// wait for the read.
#[cfg(target_os = "linux")]
#[test]
fn sigterm_exits_cleanly_while_a_bind_map_reload_is_stuck() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips.txt");
    std::fs::write(&ips, "127.0.0.1\n").expect("write ips file");
    let sidecar = dir.path().join("bind-map.json");
    let mut proc = SenderProc::spawn(&[
        ANY_PORT,
        "127.0.0.1",
        "9999",
        ips.to_str().unwrap(),
        "--bind-map",
        sidecar.to_str().unwrap(),
    ]);
    // Startup has read the IP file once an uplink is added, so the FIFO swapped
    // in below can only be opened by the reload.
    assert!(
        proc.wait_for_log("added uplink", START_TIMEOUT),
        "sender never finished startup; logs:\n{}",
        proc.logs()
    );

    std::fs::remove_file(&ips).unwrap();
    let fifo = CString::new(ips.as_os_str().as_bytes()).unwrap();
    // SAFETY: a valid NUL-terminated path and a plain permission mode.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    proc.signal("HUP");

    // A non-blocking open of the write end succeeds only once the reload has
    // opened the FIFO for reading; keeping it open leaves that read waiting.
    let deadline = Instant::now() + START_TIMEOUT;
    let write_end = loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&ips)
        {
            Ok(end) => break end,
            Err(_) if Instant::now() < deadline => thread::sleep(Duration::from_millis(20)),
            Err(e) => panic!(
                "the reload never opened the IP file ({e}); logs:\n{}",
                proc.logs()
            ),
        }
    };

    let t0 = Instant::now();
    proc.signal("TERM");
    let code = proc.wait_exit(EXIT_TIMEOUT);
    let elapsed = t0.elapsed();
    drop(write_end);

    assert_eq!(
        code,
        Some(0),
        "SIGTERM must exit 0 while the reload is stuck; logs:\n{}",
        proc.logs()
    );
    assert!(
        elapsed < PROMPT_EXIT,
        "SIGTERM must exit promptly (< {PROMPT_EXIT:?}) while the reload is stuck; took \
         {elapsed:?}"
    );
}

/// Start the sender on an IP file that is a FIFO, so its startup read blocks
/// after the listener is bound and before the main loop runs. Returns once that
/// read has opened the FIFO, with the write end that releases it.
fn start_with_startup_held_open(ips: &std::path::Path) -> (SenderProc, std::fs::File) {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::OpenOptionsExt as _;

    let fifo = CString::new(ips.as_os_str().as_bytes()).unwrap();
    // SAFETY: a valid NUL-terminated path and a plain permission mode.
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    let proc = SenderProc::spawn(&[ANY_PORT, "127.0.0.1", "9999", ips.to_str().unwrap()]);

    // A non-blocking open of the write end succeeds only once the startup read
    // has opened the FIFO; until the test writes it, that read stays blocked.
    let deadline = Instant::now() + START_TIMEOUT;
    let write_end = loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(ips)
        {
            Ok(end) => break end,
            // ENXIO is the one expected answer: no reader has opened it yet.
            Err(e) if e.raw_os_error() == Some(libc::ENXIO) && Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(20))
            }
            Err(e) => panic!(
                "the startup read never opened the IP file ({e}); logs:\n{}",
                proc.logs()
            ),
        }
    };
    (proc, write_end)
}

/// Hand the blocked startup read `contents` and close the FIFO. A sender that
/// died from the signal has no reader left; that broken pipe is the only write
/// error expected here.
fn release_startup(mut write_end: std::fs::File, contents: &[u8]) {
    use std::io::Write as _;
    if let Err(e) = write_end.write_all(contents) {
        assert_eq!(
            e.kind(),
            std::io::ErrorKind::BrokenPipe,
            "write the IP file: {e}"
        );
    }
}

/// A `SIGTERM` or `SIGINT` while startup is stalled must neither kill the
/// sender with the default action nor wait for startup: it exits 0 promptly,
/// and the startup read is never released.
fn assert_shutdown_during_a_stalled_startup(sig: &str) {
    let dir = tempfile::tempdir().unwrap();
    let (mut proc, write_end) = start_with_startup_held_open(&dir.path().join("ips.txt"));

    let t0 = Instant::now();
    proc.signal(sig);
    let code = proc.wait_exit(EXIT_TIMEOUT);
    let elapsed = t0.elapsed();
    drop(write_end);

    assert_eq!(
        code,
        Some(0),
        "SIG{sig} during a stalled startup must exit 0; logs:\n{}",
        proc.logs()
    );
    assert!(
        elapsed < PROMPT_EXIT,
        "SIG{sig} during a stalled startup must exit promptly (< {PROMPT_EXIT:?}); took \
         {elapsed:?}"
    );
}

#[test]
fn sigterm_during_a_stalled_startup_exits_promptly() {
    assert_shutdown_during_a_stalled_startup("TERM");
}

#[test]
fn sigint_during_a_stalled_startup_exits_promptly() {
    assert_shutdown_during_a_stalled_startup("INT");
}

/// A `SIGHUP` that lands during startup (CeraUI signals as soon as a link
/// appears) must become a reload once the main loop runs, not kill the sender.
#[test]
fn sighup_during_startup_becomes_a_reload() {
    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips.txt");
    let (mut proc, write_end) = start_with_startup_held_open(&ips);

    proc.signal("HUP");
    // The startup read already holds the FIFO open, so swapping the path for a
    // regular file leaves that read alone and gives the reload something to read.
    let replacement = dir.path().join("ips.next");
    std::fs::write(&replacement, "127.0.0.1\n").expect("write replacement ips file");
    std::fs::rename(&replacement, &ips).expect("replace the FIFO");
    // Startup reads an empty list, so any uplink can only come from the reload.
    release_startup(write_end, b"");

    assert!(
        proc.wait_for_log("added uplink", START_TIMEOUT),
        "SIGHUP during startup must reload the IP list once startup finishes; logs:\n{}",
        proc.logs()
    );
    assert!(
        proc.logs().contains("no valid source IPs at startup"),
        "startup must have read the empty list; logs:\n{}",
        proc.logs()
    );
    proc.signal("TERM");
    assert_eq!(
        proc.wait_exit(EXIT_TIMEOUT),
        Some(0),
        "SIGTERM after the reload must exit 0; logs:\n{}",
        proc.logs()
    );
}
