# mimalloc allocator — decision note

## Decision

**Keep `mimalloc` as the global allocator, default-on.**

The `mimalloc-allocator` Cargo feature is in `default`, so the shipped `.deb`
links mimalloc exactly as the previous unconditional `#[global_allocator]` did —
the device binary is byte-unchanged by this gating. The only new capability is an
opt-*out* for development, CI, and profiling.

`mimalloc` is built `default-features = false` with `secure` + `v3` (unchanged
from the prior unconditional dependency).

## How to disable (system allocator)

The single default feature is `mimalloc-allocator`, so `--no-default-features` is
the exact "off" build:

```bash
cargo build --release --no-default-features   # system allocator, no mimalloc linked
cargo bench  --bench alloc --no-default-features
```

Verify which allocator a binary links:

```bash
nm -C target/debug/srtla_send | rg 'mi_malloc|mi_heap'   # matches => mimalloc; no match => system
```

## Benchmark

`benches/alloc.rs` (criterion, `harness = false`) runs an SRT-sized packet-churn
workload under whichever global allocator the build selected, labeling each run
`allocator=mimalloc` or `allocator=system`. Compare:

```bash
cargo bench --bench alloc                       # mimalloc (default)
cargo bench --bench alloc --no-default-features  # system allocator
```

Measured ops/s for both configurations are captured by running
`cargo bench --bench alloc` and `cargo bench --bench alloc --no-default-features`
on the build host; results are not committed to the repo.

## Caveat — unmeasured on the device target

The benchmark runs on the build host, **not** on the constrained 4 GB SBC class the
device image targets. The original strata-port evaluation flagged mimalloc as an
"unmeasured allocator swap on a 4 GB SBC" (see
`docs/notes/strata-port-evaluation.md`). This bench validates that keeping
mimalloc is reasonable on a developer host; it does **not** by itself prove a win
on the constrained ARM target. The default-on gating is the conservative choice:
it preserves the historically shipped behavior while making an A/B comparison on
real device hardware a one-flag build away. Revisit the default if and when on-
device profiling provides evidence either direction.
