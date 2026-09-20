# mimalloc allocator — decision note

## Decision

**Keep `mimalloc` as the global allocator, default-on.**

The `mimalloc` Cargo feature is in `default`, so the shipped `.deb` links
mimalloc exactly as the previous unconditional `#[global_allocator]` did — the
device binary's allocator is unchanged by this gating. The only new capability is
an opt-*out* for development, CI, and profiling.

`mimalloc` is built `default-features = false` with `secure` + `v3` (unchanged
from the prior unconditional dependency).

## How to disable (system allocator)

The default feature set is `["mimalloc"]`, so `--no-default-features` is the
exact "off" build:

```bash
cargo build --release --no-default-features   # system allocator, no mimalloc linked
```

Verify which allocator a binary links:

```bash
nm -C target/release/srtla_send | rg 'mi_malloc|mi_heap'   # matches => mimalloc; no match => system
```

## Benchmark

Measuring the two allocators is one build flag apart — `cargo build --release`
builds with mimalloc, `cargo build --release --no-default-features` with the system
allocator. An SRT-sized packet-churn criterion benchmark accompanied the original
gating on the legacy branch (`benches/alloc.rs`, labeling each run
`allocator=mimalloc` or `allocator=system`); it is not carried on this branch.

## Caveat — unmeasured on the device target

Any such benchmark runs on the build host, **not** on the constrained 4 GB SBC class
the device image targets. The original strata-port evaluation flagged mimalloc as
an "unmeasured allocator swap on a 4 GB SBC". A host bench validates that keeping
mimalloc is reasonable on a developer machine; it does **not** by itself prove a win
on the constrained ARM target. The default-on gating is the conservative choice: it
preserves the historically shipped behavior while making an A/B comparison on real
device hardware a one-flag build away. Revisit the default if and when on-device
profiling provides evidence either direction.
