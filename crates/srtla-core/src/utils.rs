//! Utility functions shared across the codebase

use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Process-wide monotonic clock anchor.
///
/// `now_ms()` must never move backwards: every timeout, RTT sample, and
/// congestion-window deadline in this codebase is a difference between two
/// `now_ms()` reads (see the keepalive-echo RTT path in `connection::rtt`,
/// where `rtt = now_ms() - echoed_stamp` and *both* stamps are ours). A wall
/// clock (`SystemTime`) can step backwards on an NTP correction, which would
/// clamp an RTT to zero or falsely reset a link's timeout. `Instant` is
/// monotonic, so we anchor to it once and report `base_ms + elapsed`.
///
/// `base_ms` is captured from the wall clock at first read purely so the value
/// keeps an epoch-scale magnitude. Nothing depends on the absolute base (no
/// `now_ms()` value is interpreted by a peer or persisted), only on differences.
struct Clock {
    anchor: Instant,
    base_ms: u64,
}

fn clock() -> &'static Clock {
    static CLOCK: OnceLock<Clock> = OnceLock::new();
    CLOCK.get_or_init(|| Clock {
        anchor: Instant::now(),
        base_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
    })
}

/// Monotonic time in milliseconds, anchored to an epoch-scale base.
///
/// Guaranteed non-decreasing within a process. Not a true wall clock: use it
/// only for measuring elapsed time between two reads, never as a timestamp to
/// compare against another machine's clock.
pub fn now_ms() -> u64 {
    let c = clock();
    c.base_ms + c.anchor.elapsed().as_millis() as u64
}

/// True wall-clock time in milliseconds since the Unix epoch.
///
/// The deliberate carve-out from [`now_ms`]. Use it **only** for a timestamp
/// another process reads and compares against its own clock — today that is the
/// ADR-001 telemetry document's `last_updated_ms`, which a consumer diffs
/// against `Date.now()` to decide whether the snapshot is stale.
///
/// Anchoring that field to the monotonic clock would be wrong in exactly the
/// case the monotonic clock exists for: after a wall-clock step the two clocks
/// disagree permanently, so a perfectly fresh snapshot would read as stale (or
/// as coming from the future) on the consumer side. Conversely, never use this
/// for an interval: it can step backwards.
pub fn wall_clock_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_clock_ms_is_epoch_scale_and_advances() {
        // Sanity floor: 2020-01-01. A monotonic-anchored value would also pass
        // this (it borrows an epoch base), so the real assertion is below.
        let first = wall_clock_ms();
        assert!(first > 1_577_836_800_000, "not epoch-scale: {first}");

        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(wall_clock_ms() >= first, "wall clock went backwards");
    }

    #[test]
    fn wall_clock_ms_tracks_system_time_not_the_monotonic_anchor() {
        // The carve-out's whole point: this must be read straight off
        // SystemTime, so it agrees with an independent SystemTime read to
        // within scheduling noise rather than with `now_ms`'s frozen base.
        let system = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .expect("post-epoch");
        assert!(
            wall_clock_ms().abs_diff(system) < 1_000,
            "wall_clock_ms must track SystemTime"
        );
    }

    #[test]
    fn now_ms_is_non_decreasing() {
        let a = now_ms();
        std::thread::sleep(std::time::Duration::from_millis(2));
        assert!(now_ms() >= a);
    }
}
