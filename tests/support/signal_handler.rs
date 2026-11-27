//! Waiting for the sender to install a signal handler before signalling it.

use std::time::Duration;

/// Block until process `pid` has installed its handler for `signum`.
///
/// The handlers are registered after the "listening for SRT" line is
/// logged, so signalling straight off that line can still hit the default
/// action and kill the process. `SigCgt` in `/proc/<pid>/status` is the
/// kernel's own record of caught signals, so it closes that window
/// without a guessed sleep.
#[cfg(target_os = "linux")]
pub fn installed(pid: u32, signum: u32, timeout: Duration) -> bool {
    let path = format!("/proc/{pid}/status");
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        let caught = std::fs::read_to_string(&path).ok().and_then(|status| {
            let mask = status.lines().find_map(|l| l.strip_prefix("SigCgt:"))?;
            u64::from_str_radix(mask.trim(), 16).ok()
        });
        if caught.is_some_and(|mask| mask & (1 << (signum - 1)) != 0) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

/// No portable way to observe the handler off Linux; give startup a moment.
#[cfg(not(target_os = "linux"))]
pub fn installed(_pid: u32, _signum: u32, _timeout: Duration) -> bool {
    std::thread::sleep(Duration::from_millis(500));
    true
}
