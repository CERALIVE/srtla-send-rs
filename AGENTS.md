# srtla-send-rs

CERALIVE's hard fork of [`irlserver/srtla_send`](https://github.com/irlserver/srtla_send),
the Rust SRTLA bonding sender. It reads local SRT (UDP) on a listen port and forwards it
over one bound UDP socket per source IP to an SRTLA receiver. On the device it is spawned
by CeraUI and feeds the bonded path into `irl-srt-server`.

The rule of this fork is **track upstream, differentiate on top**. The tree is upstream's
code plus a thin, individually committed CERALIVE layer that CeraUI actually consumes.
Everything upstream ships (scheduler, control socket, metrics, re-home, stall deselect)
is used as-is. Do not reintroduce anything from the pre-hard-fork `legacy` branch that is
not listed under PARITY CONTRACT below.

Credits: upstream builds on ideas from [Moblin](https://github.com/eerimoq/moblin) and the
[BELABOX SRTLA reference](https://github.com/BELABOX/srtla). Keep upstream's `LICENSE`
(MIT, Thomas Lekanger) and its credits intact; CERALIVE layers AGPLv3 at distribution.

## UPSTREAM RELATIONSHIP

One permanent remote, `origin` (`https://github.com/CERALIVE/srtla-send-rs.git`). The
upstream remote is **transient**: added for a merge, removed before any push or PR.

- **Fork base:** upstream `df0b3938791ff24eced4aed8b29e3d49d0efb639` (v4.0.1). That is
  also the last-merged upstream SHA; the next sync's merge base is computed from it.
  The 2026-09 hard-fork closure ledger (bases across all four repos, ports/drops, A/B
  verdicts, erasure receipts, rollback) is `docs/notes/upstream-hardfork-2026-09.md`.
- **Merges are MANUAL and COMPAT-GATED.** No auto-sync, no scheduled merge, no bot PRs.
  Pull upstream deliberately, in a dedicated PR, when there is a reason to.
- Each merge runs the full gate (below) on the pinned toolchain and must not regress the
  parity contract. If upstream HEAD is red on formatting, green it with a mechanical
  `cargo fmt` + `clippy --fix` pass inside the merge PR; never import red state.
- Never `git fetch --tags` from upstream. The only release tags here are `v<version>`.

```bash
git remote add irlserver https://github.com/irlserver/srtla_send.git   # never 'upstream'
git fetch irlserver main:refs/remotes/irlserver/main
git rev-parse refs/remotes/irlserver/main            # pin-verify the SHA first
git merge refs/remotes/irlserver/main --no-ff -m "chore: merge upstream irlserver/srtla_send <SHA>"
git remote remove irlserver                          # BEFORE any push or PR
git remote -v                                        # must show origin only
```

## PINNED TOOLCHAIN

`rust-toolchain.toml` pins an exact nightly, `nightly-2026-06-12`, with `rustfmt`,
`clippy`, `rust-src`, and the `aarch64-unknown-linux-gnu` target. Nightly is mandatory:
`rustfmt.toml` uses unstable features (edition 2024, `group_imports`, `format_strings`).
Upstream floats on `nightly`; the fork pins a date so the device image is reproducible.
Bump only in a deliberate toolchain or upstream-merge PR, and re-run the full gate after.

Two dependency pins are load-bearing and must survive an upstream merge or a
`cargo upgrade` sweep: `smallvec = "=2.0.0-alpha.12"` (exact; pre-release line) and
`libc = "0.2"` (never a `1.0` pre-release).

## PARITY CONTRACT (CeraUI depends on every bullet; change only with a versioned decision)

- **Binary name** is exactly `srtla_send`, installed at `/usr/bin/srtla_send`.
- **CLI positional order:** `srtla_send <SRT_LISTEN_PORT> <SRTLA_HOST> <SRTLA_PORT>
  <BIND_IPS_FILE> [OPTIONS]`. CeraUI's `buildSrtlaSendArgs` emits them in that order.
- **`--verbose`** turns on debug-level logging. **`--dry-run`** parses the IP list,
  resolves the receiver, prints both, and exits `0` without binding a socket; a
  missing, unreadable, empty, or all-invalid IP list (or, with `--bind-map`, an unusable
  sidecar) exits non-zero with a specific error. `tests/cli_surface.rs`.
- **Control dialect is upstream's JSON-RPC 2.0, extended additively (ADR-004).**
  `--control-socket <path>` speaks upstream's snake_case methods (`get_status`,
  `get_stats`, `set_mode`, `set_quality`, `set_stall_deselect`, `set_conn_timeout`,
  `subscribe`, `unsubscribe`, ...). CERALIVE adds `get_capabilities` and optional fields
  inside upstream's existing payloads (`get_stats` and the `stats` topic return
  upstream's `StatsSnapshot`, whose rate field is `bitrate_bytes_per_sec`, plus
  `bytes_sent_total`, `iface`, `link_id`, `bind_map_status`, `disposition`; the ADR-001
  document with `bitrate_bps` is the FILE shape only). There is **no `hello`, no `subscribe-events`, and
  no kebab-case method**; ADR-001's transport section is superseded. Upstream's
  `SubscriptionHub` (`src/subscriptions.rs`) keeps no last frame, so a subscriber that
  needs immediate state calls `get_stats` once after subscribing. `src/control.rs`,
  `docs/CONTROL_PROTOCOL.md`, `docs/adr/ADR-004-control-dialect.md`.
- **`--capabilities-json` is the pre-spawn probe.** One line of JSON on stdout, exit
  `0`, before logging is initialized, no socket bound, no file written. Frozen key set:
  `bind_map`, `stats_file`, `dry_run`, `control_socket_jsonrpc`, `conn_timeout_ms`,
  `modes`. The runtime `get_capabilities` returns the **same document plus `methods`**,
  pinned by `get_capabilities_matches_the_pre_spawn_probe_document`. The load-bearing
  half is the caller's: any non-zero exit, unparseable output, or timeout means NO
  SUPPORT, so fall back to the legacy spawn. Never match on the code or the message.
  `src/capabilities.rs`, `tests/capabilities_probe.rs`.
- **Telemetry contract (`--stats-file <path>`, `--stats-file-interval <ms>`, ADR-001 +
  ADR-002 + ADR-003).** Opt-in: absent means no file is ever written. A newline-free
  JSON document, atomically published (temp sibling, `fsync`, `rename(2)`, with the
  fsync off the forwarding loop), shape
  `{"schema_version":1,"last_updated_ms":<wall-clock ms>,"connections":[{"conn_id","rtt_ms","nak_count","weight_percent","window","in_flight","bitrate_bps", ...}],"bytes_sent_total":<bytes>}`.
  `bitrate_bps` is wire bytes/s **× 8** (mandatory). `conn_id` is the string IP-list index
  (transient across a SIGHUP reorder). `window` and `in_flight` are required. Cadence
  defaults to 1000 ms. The live file and its `.tmp` sibling are unlinked on clean
  shutdown. `last_updated_ms` comes from `wall_clock_ms()`, not the monotonic
  `now_ms()`, because the consumer compares it against `Date.now()`. The document model
  is `src/telemetry_doc.rs`; publish mechanics are `src/telemetry_file.rs`; the snapshot
  is fed from upstream's `src/stats.rs`, not a parallel collector.
  Writer-thread creation is fallible: `TelemetryWriter::new` returns `anyhow::Result`,
  propagated through `spawn_telemetry_sink` to `main` as a contextual startup error,
  never a panic. Once started, filesystem publish failures remain best-effort warnings.
- **`bytes_sent_total` (ADR-002)** is additive at both scopes, counted in **bytes** (no
  ×8), counted at the same call site as `bitrate_bps`, and monotonic for the process
  lifetime: it does not reset on a per-link socket replacement and does not regress
  when a SIGHUP reload drops a link (the bond figure is a delta-banking accumulator,
  `SessionBytes` in `src/stats.rs`, not a sum of the live links). `schema_version` stays
  `1`. Absent means unknown, never zero.
- **ADR-003 telemetry echo: four OPTIONAL additive fields, `schema_version` stays 1.**
  Per connection `iface` and `link_id` (echoed from the sidecar, never minted here); top
  level `bind_map_status {state: active|absent|degraded, reason?}` (seven frozen
  reasons) and `disposition {state: mapped|retained_last_valid|legacy_unique_only|
  startup_collision_excluded, collisions?}`. The top-level pair is always present (an
  unmapped run reads `absent` / `legacy_unique_only`); the per-connection pair and
  `reason`/`collisions` are omitted (never `null`, never `""`) when they do not apply, so
  an unmapped run's document is the pre-ADR-003 producer's plus the top-level pair.
  `schema_version` names the shape of the REQUIRED fields;
  never bump it for an additive field. Types are projected from `src/bind_map/report.rs`;
  do not introduce a parallel status type.
- **Fixtures.** `tests/fixtures/*.json` are the producer goldens, written by
  `tests/telemetry_fixtures.rs` (regenerate deliberately with
  `UPDATE_GOLDEN=1 cargo test --test telemetry_fixtures`). `telemetry-legacy-producer.json`
  is the frozen pre-ADR-003 document and is never regenerated. The TypeScript reader and
  its copies of these fixtures live in CeraUI, not here.
- **`--bind-map <path>` (ADR-003) is additive and never required.** `BIND_IPS_FILE`
  stays byte-unchanged; the sidecar is a separate versioned JSON file describing it
  **positionally** (`{schema_version, generation, ips_file_sha256}` header, rows of
  `{link_id, ip, iface, id_path?}`). Absent `--bind-map` means byte-identical legacy
  behavior, pinned by `a_legacy_invocation_without_bind_map_produces_byte_identical_output`
  (`tests/bind_map_contract.rs`). A hash mismatch is retried (5 × 400 ms, 2 s ceiling)
  and then fails open, duplicate-safe: at startup a same-IP collision group keeps one
  deterministic representative and reports the rest excluded; on a valid-to-degraded
  reload the last valid mapped pool is retained. A mapped link binds with
  `SO_BINDTODEVICE` **and** `bind(ip, 0)` (`DeviceBinder`, `src/net/socket.rs`); an
  unmapped link takes `SourceIpBinder` verbatim. Full contract:
  `docs/adr/ADR-003-bind-map-contract.md`.
- **Link identity is `link_id`; `(ip, iface)` is only the socket key**
  (`src/net/spec.rs`). Anything that must survive a reload, reorder, reconnect, or
  interface move keys on `link_id`. A reload that moves a `link_id` to a different
  socket key recreates the socket and the registration rather than carrying
  interface-scoped state across (`src/sender/connections.rs`).
- **The interface is re-resolved by name every housekeeping tick.** A changed ifindex
  forces a rebind; a vanished interface puts the link in a `removed` state that waits
  for a reload; an `ENODEV` send does the same from the data path. Per-interface
  default-route presence is observed read-only from `/proc/net/route` and reported on
  its own axis (a lost route is a `WARN`, a regained one an `INFO`; `Unknown` crossings
  are silent). No policy routing is ever installed. `src/net/egress.rs`,
  `src/net/route.rs`, `src/sender/egress_tick.rs`.
- **A `SIGHUP` with `--bind-map` runs the sidecar read off the forwarding loop**
  (`src/sender/links.rs`), because a retried mismatch could otherwise stall forwarding
  for up to 2 s. Without `--bind-map`, SIGHUP is upstream's reload plus the guard below.
- **IP-list reload (`SIGHUP`)** keeps surviving uplinks' sockets and registrations (no
  re-handshake) and rebuilds the pool in file order. A reload resolving to zero valid
  source IPs is refused with a specific log and the stream keeps running on the
  existing links (`src/sender/reload.rs`).
- **Empty start is not fatal.** A missing, empty, or all-invalid `BIND_IPS_FILE` at
  startup binds the local listener, starts with an empty uplink pool, and waits for a
  `SIGHUP`. It must never crash-loop the device. `tests/signal_parity.rs`.
- **Startup bind ordering.** The local `SRT_LISTEN_PORT` listener is bound before the
  IP list is read and before any uplink is dialed, because CeraUI dials that port
  immediately after spawn with no readiness handshake. Never move the bind below uplink
  setup. `tests/startup_bind_ordering.rs`.
- **Clean shutdown (`SIGTERM`/`SIGINT`)** exits `0` well inside CeraUI's 10 s SIGKILL
  window and unlinks the stats file. `tests/signal_parity.rs`.
- **NAT-keepalive control padding.** Every control-plane send (keepalive, REG1/REG2)
  is zero-padded to at least `MIN_CONTROL_PKT_LEN = 32` bytes
  (`crates/srtla-protocol/src/constants.rs`, applied in `src/sender/uplink.rs`), parity
  with the C `pad_sendto`. DATA is never padded.
- **`--conn-timeout-ms` is upstream's flag; CeraUI passes `15000`.** Upstream's default
  is `CONN_TIMEOUT = 5` s (`crates/srtla-protocol/src/constants.rs`), clamped to
  `1000..=60000` ms and also settable at runtime via `set_conn_timeout`. The 15 s value
  that matches the receiver's `CONN_TIMEOUT` is **not baked into the binary**; the CeraUI
  package sets it on the command line. Do not change the constant here.
- **Re-home and stall deselect are upstream defaults, ON.** `--no-rehome` and
  `--no-stall-deselect` are the opt-outs. Neither is a CERALIVE experimental flag; they
  are upstream behavior and are documented by upstream's README sections below.
- **`-v/--version`** is operator-visible (CeraUI renders it in Settings → Versions).
  Shape: `<version> [(<branch>@<hash>[-dirty])] [srtla_send]`; the parenthetical is
  omitted, not placeholdered, when there is no git context. `src/version.rs`.
- **Scheduling modes are upstream's two:** `classic` and `enhanced` (default). No other
  value is accepted, pinned by `mode_accepts_only_the_upstream_value_set`.

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

## CI / PACKAGING

Two workflows. Both build on the pinned nightly; neither publishes a binding.

- **`ci.yml`** (push/PR): the gate above, the `loom` and `miri` lanes, upstream's
  stable/beta/Windows/macOS lanes (they call `cargo +<channel>` explicitly so the pin
  does not shadow them), and a `build-deb` matrix that cross-compiles
  `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-gnu` and packages each `.deb`
  so a packaging break is caught before any tag. The CI `.deb` job runs on
  `ubuntu-latest` on purpose; its artifacts are never shipped.
- **`release.yml`** (tag push `v*`): the full gate plus `loom` and `miri`, then
  `build-deb` inside **`debian:bookworm-slim`** for both arches. Each ELF's versioned
  imports are inspected and anything above **`GLIBC_2.36`** (Debian 12, the device
  image) fails the job. Both `.deb`s and `.sha256`s are attached to the GitHub release
  and the APT reindex is dispatched for component `srtla`. Never move the release build
  back onto the raw runner userspace.

`ci/build-deb.sh` is the single source of truth for the package: **Package `srtla`**,
binary at `/usr/bin/srtla_send`, `Architecture` `arm64`/`amd64`, filename
**`srtla_<ver>_<arch>.deb`** (it re-runs the image pipeline's `*${ARCH}*.deb` fetch glob
as a self-test), `Conflicts`/`Replaces` on the retired transitional package name. The
version comes from `Cargo.toml` (**`4.1.0`**), follows upstream's semver line (not
CalVer), and a tag build must be `v<version>`; the script rejects a mismatch.

aarch64 cross-build: linker `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc`,
apt `gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross binutils-aarch64-linux-gnu pkg-config`,
`PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig`.

`mimalloc` is the global allocator behind a default-on feature; `--no-default-features`
is the system-allocator build for Miri and profiling. `docs/notes/mimalloc-decision.md`.

## CODEBASE (upstream layout + the CERALIVE additions)

```
crates/srtla-protocol/   wire constants and packet types (CONN_TIMEOUT, MIN_CONTROL_PKT_LEN)
crates/srtla-core/       connection, registration, selection, config snapshot, utils
crates/network-sim/      dev-only netns harness; twin/ = duplicate-IP topology + sidecar publisher
src/main.rs              clap CLI (positionals, upstream flags, CERALIVE flags)
src/control.rs           JSON-RPC dispatch (+ get_capabilities)      src/subscriptions.rs  SubscriptionHub
src/capabilities.rs      --capabilities-json document                src/version.rs        -v line
src/telemetry_doc.rs     ADR-001 document model                      src/telemetry_file.rs --stats-file sink
src/stats.rs             upstream snapshot + SessionBytes + bind-map report slot
src/bind_map/            ADR-003 sidecar: parser, coherence, retry, resolve, report
src/net/                 sockets (SourceIpBinder/DeviceBinder), spec, egress, route, batch I/O
src/sender/              forwarding loop, links (ips file vs bind-map pair), connections,
                         reload guard, egress_tick, housekeeping, rehome, status
docs/adr/                ADR-001 (historical), ADR-002, ADR-003, ADR-004
ci/build-deb.sh          the .deb packager      scripts/  contract tests + netns gate
```

Conventions (enforced by the gate): edition 2024, `anyhow::Result`, `tracing` macros,
Tokio, imports grouped std → external → crate, constants `SCREAMING_SNAKE_CASE`.
Commit messages follow Conventional Commits. `CLAUDE.md` is a symlink to this file.

## ANTI-PATTERNS

- **No scheduler or mode work.** No new scheduling modes, no alternative selectors, no
  path-prediction or in-order-delivery pipelines, no experimental window-growth or
  link-exclusion flags. Upstream's `classic`/`enhanced` selector is the scheduler. If a
  ported test references a removed mode, delete the reference; never re-add the mode.
- **No bindings in this repo.** There is no `bindings/` directory, no npm package, and no
  binding tag namespace. The TypeScript sender/telemetry/control helper lives in CeraUI
  as a workspace package and is tested against the real binary there.
- **No auto-sync with upstream** and no transient remote left attached at push time.
- **No second control dialect.** No `hello`, no `subscribe-events`, no kebab-case
  methods, no replay-on-subscribe. Extend upstream's JSON-RPC additively only.
- **Don't unpin or silently bump the toolchain**, and don't "modernize" the `smallvec`
  or `libc` pins.
- **Don't break the parity contract** (binary name, positional order, telemetry shape,
  `bitrate_bps` ×8, additive-only telemetry fields, capability key set, SIGHUP reload,
  empty start, bind ordering) without a deliberate versioned change.
- **Don't strip upstream MIT/credits.** Layer AGPLv3 at distribution only.
- **No path above the repo root in any tracked file.** The repo builds and releases
  standalone in CI; the workspace parent does not exist there.
- **Don't bake `CONN_TIMEOUT = 15` into the binary.** CeraUI passes `--conn-timeout-ms`.
- **Don't add silent source filtering to the uplink sockets.** They are unconnected by
  upstream design (NAT and multi-homed receivers reply from other addresses).

## DOCS DISCIPLINE (Rule A)

Any behavior or structure change updates this `AGENTS.md` and `README.md` in the same
PR, and `scripts/check-doc-refs.sh` must stay green. Keep the parity contract section
authoritative: it is the device-integration contract.
