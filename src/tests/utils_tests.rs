//! Clock contract tests for `src/utils.rs`.
//!
//! `now_ms()` is a monotonic clock (wall-clock base + `Instant` anchor); the
//! telemetry `last_updated_ms` field is the one carve-out that must stay on the
//! raw wall clock. Both halves are pinned structurally so a later refactor
//! cannot silently revert either one.

use crate::utils::{compose_now_ms, now_ms, wall_clock_ms};

/// Extract the body of `fn <name>` from a Rust source string by brace matching.
///
/// Deterministic: scans from the first `{` after the signature and returns the
/// slice up to its matching `}`. Sufficient here because neither target
/// function contains a brace inside a string or char literal.
fn function_body<'a>(src: &'a str, signature: &str) -> &'a str {
    let start = src
        .find(signature)
        .unwrap_or_else(|| panic!("signature not found: {signature}"));
    let open = src[start..]
        .find('{')
        .unwrap_or_else(|| panic!("no opening brace after: {signature}"))
        + start;

    let bytes = src.as_bytes();
    let mut depth = 0usize;
    for (offset, byte) in bytes.iter().enumerate().skip(open) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &src[open + 1..offset];
                }
            }
            _ => {}
        }
    }
    panic!("unbalanced braces after: {signature}");
}

#[test]
fn compose_now_ms_is_monotone_and_saturating() {
    // Given: a fixed base and a table of (elapsed, expected) pairs.
    let base = 1_749_556_546_000u64;
    let table = [
        (0u64, base),
        (1, base + 1),
        (1_000, base + 1_000),
        (86_400_000, base + 86_400_000),
    ];

    // Then: composition is a plain offset from the base.
    for (elapsed, expected) in table {
        assert_eq!(
            compose_now_ms(base, elapsed),
            expected,
            "compose_now_ms({base}, {elapsed})"
        );
    }

    // Given: a non-decreasing elapsed sequence with irregular steps.
    // Then: the composed output never decreases.
    let mut elapsed = 0u64;
    let mut previous = compose_now_ms(base, elapsed);
    for step in 0..10_000u64 {
        elapsed += step % 7;
        let current = compose_now_ms(base, elapsed);
        assert!(
            current >= previous,
            "compose_now_ms went backwards: {previous} -> {current} at elapsed={elapsed}"
        );
        previous = current;
    }

    // Then: overflow saturates instead of wrapping (which would go backwards).
    assert_eq!(compose_now_ms(u64::MAX, 0), u64::MAX);
    assert_eq!(compose_now_ms(u64::MAX, 1), u64::MAX);
    assert_eq!(compose_now_ms(u64::MAX - 1, 5), u64::MAX);
    assert_eq!(compose_now_ms(u64::MAX, u64::MAX), u64::MAX);
}

#[test]
fn now_ms_never_goes_backwards() {
    // Given: the live process clock.
    // Then: 10k consecutive readings are non-decreasing.
    let mut previous = now_ms();
    for iteration in 0..10_000u32 {
        let current = now_ms();
        assert!(
            current >= previous,
            "now_ms() went backwards at iteration {iteration}: {previous} -> {current}"
        );
        previous = current;
    }

    // And: it is still anchored near the wall clock (same epoch, not uptime).
    let wall = wall_clock_ms();
    assert!(
        previous.abs_diff(wall) < 60_000,
        "now_ms() ({previous}) drifted from wall_clock_ms() ({wall}) by more than a minute"
    );
}

#[test]
fn now_ms_does_not_read_system_time() {
    // Given: the source of the clock module.
    let src = include_str!("../utils.rs");

    // When: the body of `fn now_ms` is isolated by brace matching.
    let body = function_body(src, "pub fn now_ms() -> u64");

    // Then: it composes from the process-wide monotonic clock...
    assert!(
        body.contains("CLOCK"),
        "now_ms() must read the monotonic CLOCK; body was:\n{body}"
    );
    assert!(
        body.contains("compose_now_ms"),
        "now_ms() must compose through the pure seam; body was:\n{body}"
    );
    // ...and never touches the system clock directly.
    assert!(
        !body.contains("SystemTime"),
        "now_ms() must not read SystemTime (monotonicity); body was:\n{body}"
    );

    // And: the wall-clock carve-out is the only SystemTime reader.
    let wall_body = function_body(src, "pub fn wall_clock_ms() -> u64");
    assert!(
        wall_body.contains("SystemTime"),
        "wall_clock_ms() must read SystemTime; body was:\n{wall_body}"
    );
}

#[test]
fn telemetry_uses_wall_clock() {
    // Given: the sender event loop source.
    let src = include_str!("../sender/mod.rs");

    // Then: both telemetry snapshots are stamped with the wall clock...
    assert_eq!(
        src.matches("build_telemetry_json_from_stats(wall_clock_ms")
            .count(),
        2,
        "both telemetry snapshot call sites must stamp last_updated_ms with wall_clock_ms()"
    );
    // ...and none with the monotonic clock (the TS watcher compares
    // last_updated_ms against Date.now()).
    assert_eq!(
        src.matches("build_telemetry_json_from_stats(now_ms")
            .count(),
        0,
        "telemetry last_updated_ms must not come from the monotonic now_ms()"
    );
}
