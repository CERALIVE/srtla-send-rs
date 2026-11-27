# mimalloc allocator — decision note

## Decision (2026-11-27)

**The binary uses the system allocator.** Upstream `adc1f4a` (irlserver #34) removed
the `mimalloc` feature and dependency while cutting hot-path allocations, and the
owner chose to follow upstream in the `1481507` sync rather than carry a fork-only
allocator ([`upstream-sync-2026-11-1481507.md`](upstream-sync-2026-11-1481507.md)).
The crate has no default features any more, so `--no-default-features` builds the
same binary; the Miri lane keeps the flag only because its contract test pins the
command.

## Caveat — unmeasured on the device target

Neither allocator has been measured on the 4 GB RK3588 SBC class the device image
targets. The previous default kept mimalloc only because it was the shipped status
quo; this change carries the same evidence gap in the other direction. Revisit with
on-device profiling.

## History

From the 2026-07 gating until the `1481507` sync, mimalloc (`default-features =
false`, `secure` + `v3`) was the global allocator behind a default-on `mimalloc`
feature, and `--no-default-features` was the system-allocator build for Miri and
profiling. Its benchmark (`benches/alloc.rs` on the legacy branch) ran on the build
host only.
