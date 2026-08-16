//! Utility functions shared across the codebase

use std::sync::{LazyLock, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::time::Instant;

/// Static startup instant for stable epoch-based timing calculations
/// This is initialized once at program startup and used for periodic operations
/// that need to be based on a stable reference point.
pub static STARTUP_INSTANT: LazyLock<Instant> = LazyLock::new(Instant::now);

/// Monotonic millisecond clock: a wall-clock base captured once, advanced by a
/// monotonic [`std::time::Instant`] anchor.
///
/// Deliberately `std::time::Instant`, NOT `tokio::time::Instant`: Tokio's clock
/// can be paused/advanced by tests, and the scheduling/RTT/timeout arithmetic
/// built on [`now_ms`] expects real elapsed time (existing tokio-paused-clock
/// tests rely on `now_ms` NOT being frozen).
struct Clock {
    /// Monotonic anchor taken at the same moment as `base_ms`.
    anchor: std::time::Instant,
    /// Wall-clock milliseconds since the Unix epoch at anchor time.
    base_ms: u64,
}

/// Process-wide clock, initialized on the first [`now_ms`] call.
static CLOCK: OnceLock<Clock> = OnceLock::new();

/// Compose a monotonic timestamp from a fixed base and a monotonic elapsed value.
///
/// Pure and total (saturating), so the clock composition can be tested
/// deterministically without manipulating the system clock.
pub(crate) fn compose_now_ms(base_ms: u64, elapsed_ms: u64) -> u64 {
    base_ms.saturating_add(elapsed_ms)
}

/// Get current time in milliseconds since the Unix epoch, **monotonically**.
///
/// The absolute value is `base_ms + monotonic elapsed`, where `base_ms` is a
/// single `SystemTime` reading captured at the first call. Consequences, by
/// design:
///
/// - **Differences are exact and never go backwards.** NTP steps, manual clock
///   changes, DST/timezone edits and leap-second smearing cannot make a later
///   call return a smaller value, so RTT samples, timeouts, backoffs and
///   window-growth intervals can never see a negative or absurd delta.
/// - **The absolute base may be wrong.** On an embedded device that boots
///   without RTC and syncs NTP later, `base_ms` is whatever the pre-NTP clock
///   said (and is `0` if the system clock is before the Unix epoch — the
///   documented fallback). This clock never re-syncs to correct it: only
///   differences are meaningful.
/// - Consumers that need a real wall-clock timestamp (i.e. one comparable
///   against another process's `Date.now()`) must use [`wall_clock_ms`]. The
///   telemetry `last_updated_ms` field is exactly that case.
pub fn now_ms() -> u64 {
    let clock = CLOCK.get_or_init(|| Clock {
        anchor: std::time::Instant::now(),
        base_ms: wall_clock_ms(),
    });
    compose_now_ms(clock.base_ms, clock.anchor.elapsed().as_millis() as u64)
}

/// Get the raw wall-clock time in milliseconds since the Unix epoch.
///
/// Returns 0 if system time is before the Unix epoch (fallback behavior).
///
/// Unlike [`now_ms`] this tracks the system clock, so it CAN jump backwards
/// (NTP step) — use it only where an absolute, externally comparable timestamp
/// is required. The telemetry snapshot's `last_updated_ms` is compared against
/// `Date.now()` by the TypeScript watcher for staleness, so it must come from
/// here; every intra-process duration must come from [`now_ms`].
pub fn wall_clock_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_millis(0))
        .as_millis() as u64
}

/// Get elapsed milliseconds since program startup
/// Uses the stable STARTUP_INSTANT for consistent periodic timing
pub fn elapsed_ms() -> u64 {
    STARTUP_INSTANT.elapsed().as_millis() as u64
}
