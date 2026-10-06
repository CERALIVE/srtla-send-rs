<!-- Moved verbatim from AGENTS.md on 2026-10-05 by lean-rules-docs-landing-latam -->

## BUILD / GATE

Run the full gate green on the pinned nightly before every PR (auto-selected via
`rust-toolchain.toml`):

```bash
cargo build --release
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo check && cargo check --release
cargo test --lib
timeout --foreground --kill-after=10s 600s cargo test --all-features
timeout --foreground --kill-after=10s 600s cargo test --features test-internals
cargo audit && cargo deny check advisories sources
```

`test-internals` exposes internal fields for assertions; `--all-features` enables it.
Most tests run in-process over loopback UDP. The `tests/netns_*.rs` targets are
privileged supplements (Linux network namespaces, passwordless `sudo`, `srtla_rec`,
`srt-live-transmit`) that self-skip when their dependencies are absent, so an ordinary
CI run does not prove the privileged topology. Run them only through
`scripts/netns_test_gate.sh` (90 s per target; `netns_twin` gets 420 s via
`NETNS_TWIN_TEST_TIMEOUT_SECONDS` because it waits out real sender timers).

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

**Miri lane (BLOCKING, not part of the default gate).** Five single-filter runs over the
pure pointer logic in `src/net/batch_recv.rs` (`test_recv_buffer_creation`,
`test_buffer_size`, `iter_clamps_oversized_msg_len_to_mtu`, `sockaddr_storage_roundtrip`,
`recv_retry_action_classifies_errors`), each as
`cargo miri test --lib --no-default-features --features test-internals <filter>`.
`--no-default-features` drops the mimalloc global allocator, which Miri cannot run. Miri
cannot execute `recvmmsg`/`sendmmsg`; the send-side header construction has no Miri
coverage on this base (it is built inline in `try_send_batch`). Both workflows must
carry exactly these five filters; `scripts/release_workflow_contract_test.py` checks.

Typed workflow contracts run under `uv`: `scripts/release_workflow_contract_test.py`,
`scripts/rust_cache_contract_test.py`, `scripts/workflow_authority_contract_test.py`,
plus `scripts/release_version_contract_test.sh`, `scripts/deb_version_ordering_test.sh`,
and `scripts/check-doc-refs.sh` (every `docs/` path named in this file and `README.md`
must resolve).

