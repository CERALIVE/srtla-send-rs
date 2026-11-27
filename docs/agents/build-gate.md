<!-- Moved verbatim from AGENTS.md on 2026-10-05 by lean-rules-docs-landing-latam -->

## BUILD / GATE

Run the full gate green on the pinned nightly before every PR (auto-selected via
`rust-toolchain.toml`):

```bash
cargo build --release
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check && cargo check --release
cargo check --workspace --lib --bins && cargo check --workspace --all-targets
cargo test --lib
timeout --foreground --kill-after=10s 600s cargo test --workspace --all-features --verbose
timeout --foreground --kill-after=10s 600s cargo test --workspace --features test-internals
cargo audit && cargo deny check advisories sources
```

`test-internals` exposes internal fields for assertions; `--all-features` enables it.
Every test build turns `srtla-core`'s copy on through `srtla_send`'s dev-dependency, so
`cargo check --workspace --lib --bins` is the only build of the crates with it off, and
`cargo check --workspace --all-targets` compiles the integration tests without
`srtla_send`'s own `test-internals`.
Most tests run in-process over loopback UDP. The `tests/netns_*.rs` targets are
privileged supplements (Linux network namespaces, passwordless `sudo`, `srtla_rec`,
`srt-live-transmit`) that self-skip when their dependencies are absent, so an ordinary
CI run does not prove the privileged topology. Run them only through
`scripts/netns_test_gate.sh` (90 s per target; `netns_twin` gets 420 s via
`NETNS_TWIN_TEST_TIMEOUT_SECONDS` because it waits out real sender timers). It runs every
`tests/netns_*.rs` target, one test at a time: the harness names veth pairs after the
process id, so two tests of one target would collide on `ip link add`.

`netns_stall_gate` injects SRT DATA at 2000 packets/s through the shared harness
injector, which sends from one encoder source port with clock-based sequence numbers,
so its 2/3/8-second phases never reuse an acknowledged sequence. It runs with
`--stall-min-in-flight 8`, keeps the load up until both links have left Warming, and
blacks out both directions of the dead link. At the 1-in-100 probe rate, 2000
packets/s supplies twenty probes/s, so ten-packet receiver ACK batches keep delivery
proof fresh through the dwell. Its focused log filter keeps lifecycle debug events on
stdout without evicting them from the bounded capture with per-packet debug noise.

RTT sampling follows Karn's ambiguity rule: a sequence repeated on one link or an
R-bit retransmit cannot supply an RTT sample, and a probe yields one only when the
link has probed that sequence once. Their ACK pruning, in-flight accounting,
delivery proof and congestion/window updates remain unchanged. The wire-loss test
holds the existing real SRT caller at 4 Mbps, below either 6 Mbps shaper: its
adaptive unacknowledged-packet limit otherwise stops the load near 1.7 Mbps. Each
link must clear both an absolute 0.5 Mbps floor and a 15% share of the bond.
Stock-receiver NAK counts reflect cross-link reordering; production-like receiver
options and adaptive-demand recovery coverage remain follow-ups.

**`tests/netns_twin.rs`** (8 scenarios) is the only target that reproduces two uplinks
sharing one source address, built on `crates/network-sim/src/twin/` (one NAT carrier
namespace per twin, per-device `rp_filter` cleared, `src`-hinted return routes). It
includes a falsifiability control: the same topology without `--bind-map` leaves the
second twin dead.

**Production subscription-concurrency invariant (BLOCKING, separate lane).**
`tests/subscription_loom.rs` drives upstream's real `SubscriptionHub`
(`src/subscriptions.rs`) under Loom, racing `publish` against `subscribe` and drop. The
invariants are the *next-publish* form: a subscriber racing a publish is never silently
dropped (its live read is `None` or the in-flight frame, and it always receives the next
publish), and a hung-up subscriber is pruned by the next publish. There is no
last-frame replay on this base; do not document one. Command contract:

```bash
RUSTFLAGS="--cfg loom" cargo test --test subscription_loom
```

**Miri lane (BLOCKING, not part of the default gate).** Eight single-filter runs over the
pure pointer logic in `src/net/batch_recv.rs` (`test_recv_buffer_creation`,
`test_buffer_size`, `iter_clamps_oversized_msg_len_to_mtu`, `sockaddr_storage_roundtrip`,
`recv_retry_action_classifies_errors`, and upstream `201e948`'s
`pointers_follow_the_buffer_after_a_move_out_of_the_box`,
`pointers_follow_both_buffers_after_a_swap`, `forged_packet_reads_correctly_after_a_move`),
each as
`cargo miri test --lib --no-default-features --features test-internals <filter>`.
`--no-default-features` no longer changes the build (mimalloc, which Miri cannot run, was
dropped and the crate has no default features); the contract test pins the command. Miri
cannot execute `recvmmsg`/`sendmmsg`; the send-side header construction has no Miri
coverage on this base (it is built inline in `try_send_batch`). Both workflows must
carry exactly these eight filters; `scripts/release_workflow_contract_test.py` checks.

Typed workflow contracts run under `uv`: `scripts/release_workflow_contract_test.py`,
`scripts/rust_cache_contract_test.py`, `scripts/workflow_authority_contract_test.py`,
plus `scripts/release_version_contract_test.sh`, `scripts/deb_version_ordering_test.sh`,
`scripts/netns_test_gate_contract_test.sh` (the netns gate against a stub `cargo`),
and `scripts/check-doc-refs.sh` (every `docs/` path named in this file and `README.md`
must resolve).

