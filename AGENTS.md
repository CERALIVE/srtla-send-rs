# srtla-send-rs

Parent: [`../AGENTS.md`](../AGENTS.md)

## ROLE IN THE GROUP

CERALIVE's **fork** of [`irlserver/srtla_send`](https://github.com/irlserver/srtla_send) —
the Rust SRTLA bonding **sender**. It reads local SRT (UDP) on a listen port and forwards
it over multiple bonded uplinks (one bound UDP socket per source IP) to an SRTLA receiver,
balancing traffic by link capacity/quality (EDPF/BLEST/IoDS scheduling, Kalman-smoothed
RTT). On the device it is driven by CeraUI and feeds the bonded path into
`irl-srt-server`. Canonical branch `main`; sibling checkout under the workspace root
(see CRITICAL CONSTRAINTS below).

> **Status:** v1.0.0 — CeraLive parity layer complete. Fork created from upstream HEAD;
> nightly pinned; full gate green on the pinned toolchain. Landed: CLI parity contract
> (Task 9: `--verbose`/`--dry-run`/`--stats-file`/`--stats-file-interval`), the opt-in
> ADR-001 telemetry sink (Task 10: `src/telemetry_file.rs`), signal/startup parity
> (Task 11: SIGHUP reload guard, empty-start, clean SIGTERM/SIGINT), PR#19 behavior
> verification (Task 12: keepalive cadence + jitter demotion), CI/packaging (Task 13:
> aarch64 + x86_64 cross-build producing pipeline-compatible `.deb`s -- see CI / PACKAGING),
> telemetry test hardening (Task 7: `tests/telemetry_edge_cases.rs` + `tests/telemetry_fixture_parity.rs`),
> TS binding test hardening (Task 8: `bindings/typescript/tests/telemetry-reader.test.ts`, 52 tests total),
> sendmmsg triage (Task 25: TODO converted to tracked DEFERRED note), and robustness
> pass (2026-06-19: S9 all-links-failed timeout fix, S5 dead-reader restart, S6 RTT
> clamp + classifier fix, S7 zero-RTT keepalive rejection — see ROBUSTNESS FIXES).
> Hardening pass (2026-06-25): EDPF scheduler state moved off thread-local + EDPF
> pipeline tests, loom/miri/proptest model+fuzz lanes, telemetry fsync moved off the
> packet-forwarding loop, keepalive interop (BELABOX 2-byte vs extended) + wire
> conformance goldens, mimalloc gated behind a default-on feature, netns de-flake to
> bounded readiness polling, and a final docs-consistency audit (T21: dangling-ref
> sweep via `scripts/check-doc-refs.sh`, version-drift + Rule-A sync).
> CeraUI integration lands in follow-up tasks.

**Relationship to `srtla/`:** this is the **sender** engine (Rust). The existing
`srtla/` repo holds the C `srtla_send`/`srtla_rec` pair plus the bonding receiver and
its own **TypeScript bindings** (`@ceralive/srtla`, consumed by CeraUI via the sibling
`link:`). This repo additionally ships its **own** pure-TS sender binding,
`@ceralive/srtla-send`, under `bindings/typescript/` — published to the **public npm
registry** (`@ceralive` scope) via npm **OIDC trusted publishing** and consumed
**registry-only**: **no sibling `link:`, no `.tgz` vendoring**. It is a thin,
registry-distributed helper layer over this repo's binary
(args/validation/telemetry reader); the binary itself remains the primary artifact.
The `@ceralive/srtla-send` sender/telemetry exports mirror `@ceralive/srtla`'s
`./sender` + `./telemetry` subpaths and must not import or share types with
`@ceralive/cerastream`.

## UPSTREAM RELATIONSHIP

One permanent remote: `origin` (`https://github.com/CERALIVE/srtla-send-rs.git`). The
irlserver upstream remote is **TRANSIENT** — added only for the duration of a merge PR,
then removed before any push or PR is opened.

```
origin    https://github.com/CERALIVE/srtla-send-rs.git   (our fork; push here; always present)
irlserver https://github.com/irlserver/srtla_send.git     (merge source; TRANSIENT — add, fetch, merge, remove)
```

**Transient-remote recipe** (use [`scripts/upstream-merge.sh`](../scripts/upstream-merge.sh)):

```bash
# 1. Add the upstream remote under the name 'irlserver' (NEVER 'upstream')
git remote add irlserver https://github.com/irlserver/srtla_send.git

# 2. Fetch with an explicit destination refspec
git fetch irlserver main:refs/remotes/irlserver/main

# 3. Pin-verify the fetched SHA before merging
git rev-parse refs/remotes/irlserver/main   # confirm expected SHA

# 4. Merge (true-merge commit — never squash an upstream-sync PR)
git merge refs/remotes/irlserver/main --no-ff -m "chore: merge upstream irlserver/srtla_send <SHA>"

# 5. Remove the remote BEFORE any push or PR
git remote remove irlserver
```

The working clone is back to `origin`-only at PR time. Never leave a remote pointing at
the fork parent attached when opening a PR. Verify: `git remote -v` must show only
`origin` before `git push`.

- **License:** upstream is **MIT**; CERALIVE ships under **AGPLv3**. MIT → AGPLv3
  incorporation is compatible. Keep upstream's `LICENSE` and credits intact; add CERALIVE
  licensing at the workspace/distribution layer, not by stripping upstream notices.
- **Fork start point:** upstream `80cd0c4` ("feat: use Kalman-smoothed RTT in EDPF
  arrival time prediction").

### Upstream-merge policy — MANUAL & COMPAT-GATED

- **Upstream merges are MANUAL and COMPAT-GATED. Never set up auto-sync / scheduled
  upstream merges / bots that open update PRs.** Pull upstream deliberately, in a
  dedicated PR, only when there is a reason to.
- Each merge runs the **full gate green** (below) on the **pinned** toolchain before
  it can land, and must not regress the CLI parity contract or the telemetry contract.
- Upstream HEAD is sometimes red on its own CI (e.g. an unformatted commit failing
  "Check formatting"). Do **not** import red upstream state — green it with a
  **mechanical-only** `cargo fmt` + `clippy --fix` pass (zero behavior change) as part
  of the merge PR, exactly as the bootstrap did.

## PINNED TOOLCHAIN

`rust-toolchain.toml` pins an **exact** nightly: `channel = "nightly-2026-06-12"`
(rustc 1.98.0-nightly, `b30f3df3b`), components `rustfmt clippy rust-src`, target
`aarch64-unknown-linux-gnu`.

- **Nightly is mandatory** — `rustfmt.toml` enables unstable formatting features
  (edition 2024, `group_imports`, `format_strings`, …). Upstream CI floats on
  `nightly`; the fork pins a date so the **device image build is reproducible**.
- **Bump only deliberately** (a toolchain-bump or upstream-merge PR), and re-run the
  full gate after any bump. Never let the pin drift silently.
- **aarch64 cross-build** (device target) needs the GNU cross linker; mirror upstream's
  `build-debian.yml`:
  - linker: `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc`
  - apt: `gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross binutils-aarch64-linux-gnu pkg-config`
  - `PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig`

## DEPENDENCY PINS

Most deps use caret floors (`tokio = "1.52"`, `clap = "4.6"`, `rand = "0.10"`,
`libc = "0.2"`). Two deliberate exceptions, both load-bearing — do not "modernize"
them in an upstream merge or a `cargo upgrade` sweep without a deliberate PR:

- **`smallvec = "=2.0.0-alpha.12"` (EXACT pin).** smallvec 2.x is still a pre-release
  line; each `2.0.0-alpha.*` can ship breaking API/layout changes, so we pin one
  known-good alpha rather than float across the alpha range. Mirrored by a comment on
  the dep in `Cargo.toml`. **Revisit and unpin to `"2"` once smallvec 2.0 is stable.**
  Do not migrate off smallvec 2.x while the pin stands.
- **`libc = "0.2"`.** Stay on the 0.2 line — do **not** bump to a `1.0` alpha/pre-release.

**`rand` is 0.10 (workspace + `crates/network-sim`), single version in the shipped
binary (0.10.2).** rand 0.9→0.10 was an API break: `rand_core::RngCore` was renamed to
`rand_core::Rng` (re-exported as `rand::Rng`) and the old `rand::Rng` ext trait became
`rand::RngExt`. Call sites use `use rand::Rng;` for `fill_bytes`/`next_u64`
(`src/registration/`, `src/connection/`) and `rand::RngExt` for `.random()`
(`crates/network-sim/src/scenario.rs`). A `rand 0.9.4` duplicate persists **only** via
the `proptest` dev-dependency (latest 1.11.0 has no rand-0.10 release); it is dev/test
-only and absent from the release binary — see the `deny.toml` RUSTSEC-2026-0097 note.

## PARITY CONTRACT (do not break without a versioned change)

CeraUI and the device integration depend on these staying stable:

- **Binary name:** exactly `srtla_send`.
- **CLI positional order:**
  `srtla_send <SRT_LISTEN_PORT> <SRTLA_HOST> <SRTLA_PORT> <BIND_IPS_FILE> [OPTIONS]` —
  the four positionals are load-bearing and resolved in exactly the order CeraUI's
  `buildSrtlaSendArgs` emits them.
- **CeraLive control-plane flags:** `--verbose` (debug logging), `--dry-run` (parse the
  IP list and resolve the receiver, print them, then exit `0` without binding any socket;
  an unusable IP list — missing/unreadable, empty, or zero valid IPs — exits non-zero with
  a specific error), `--stats-file <path>` and `--stats-file-interval <ms>` (default
  `1000`). The `--stats-file` telemetry sink is **implemented** (`src/telemetry_file.rs`)
  and opt-in — absent means no file is ever written.
- **Upstream scheduler/control-socket flags** (`--mode`, `--no-quality`, `--exploration`,
  `--rtt-delta-ms`, `--control-socket`, `-v/--version`) stay present and functional but
  are **not** surfaced in CeraUI.
- **Telemetry contract (`--stats-file <path>`, ADR-001):** opt-in (absent ⇒ no file is
  ever written). Newline-free JSON document, atomically published (temp sibling →
  `fsync` → `rename(2)`), shape
  `{"schema_version":1,"last_updated_ms":<ms>,"connections":[{"conn_id","rtt_ms","nak_count","weight_percent","window","in_flight","bitrate_bps"}]}`.
  `bitrate_bps` is wire-bytes/s × 8 (the ×8 bits-per-second conversion is mandatory);
  `conn_id` is the string IP-list index (stable until a SIGHUP reorder); `window` and
  `in_flight` are **required** by the frozen `@ceralive/srtla` Zod reader. The cadence is
  `--stats-file-interval` ms (default 1000). The live file is unlinked on clean shutdown
  (SIGTERM/SIGINT). `schema_version` is additive over the C producer — the Zod reader
  strips it. Implemented in `src/telemetry_file.rs`; CeraUI parses this verbatim.
- **IP-list reload (`SIGHUP`, Unix):** reloads `BIND_IPS_FILE` without restart.
  Surviving uplinks keep their socket + registration (no re-handshake, zero
  disconnect); the pool is rebuilt in **ips-file order** so `conn_id` tracks the
  file (reorder reorders `conn_id`, matching the telemetry contract above). A
  reload that resolves to **zero valid source IPs** — missing/unreadable, empty,
  or all-garbage — is **refused** (the stream keeps running on the existing
  links) with a specific log: `ips file not found/unreadable`, `ips file is
  empty`, `invalid IP on line N` (mixed valid+invalid still applies), or `no
  valid source IPs … keeping existing connections`. Mirrors the C reload guard
  (`srtla/src/sender_logic.h`).
- **Empty start:** a missing / empty / all-invalid `BIND_IPS_FILE` at startup is
  **not** fatal — the sender binds the local listener, starts with an empty
  uplink pool, and waits for a `SIGHUP` (CeraUI writes the file and signals once
  interfaces appear). It must not crash-loop the device.
- **Clean shutdown (`SIGTERM`/`SIGINT`, Unix):** exit `0` well within CeraUI's
  10s SIGKILL window; the `--stats-file` telemetry file (and its `.tmp` sibling)
  is unlinked so no stale snapshot outlives the process.
- **NAT-keepalive control padding (`MIN_CONTROL_PKT_LEN = 32`,
  `src/connection/packet_io.rs`):** every control-plane send (keepalive,
  REG1/REG2) routes through `send_control_padded`, which zero-pads frames
  smaller than 32 bytes up to a 32-byte wire frame — parity with the C
  `pad_sendto` (`srtla/src/protocol/pad_sendto.h`) so cellular/carrier NAT
  keepalive thresholds don't silently drop tiny control frames. **DATA is never
  padded** — the batch DATA path (`src/connection/batch_send.rs`) deliberately
  bypasses it; padding DATA would corrupt the SRT byte stream. Current control
  frames are already ≥32 B (extended keepalive 38 B, REG1/REG2 258 B) so this is
  a passthrough today, but the floor is now enforced. Pinned by
  `control_packet_padded_to_32b` + `data_not_padded` (`src/tests/connection_tests.rs`).
- **Keepalive divergence (BELABOX 2-byte vs our timestamped 10/38 B):** BELABOX
  (`BELABOX/srtla` `srtla_send.c` L589-593) sends a **bare 2-byte** keepalive
  (type only, no timestamp); this fork inherited irlserver's **timestamped**
  keepalive — standard 10 B (`type + u64 ms`) and the backwards-compatible
  **extended 38 B** (`+ 0xC01F`-tagged `ConnectionInfo` telemetry) — and always
  emits the extended form. The timestamp is what powers RTT-from-keepalive
  (`RttTracker::handle_keepalive_response`); a bare echo carries none. Interop
  rests on the **receiver-echo ASSUMPTION**: receivers echo keepalives and
  tolerate trailing bytes (proven for irlserver/CeraLive; **assumed, not proven,
  for BELABOX** — needs a live BELABOX receiver). The sender defensively accepts
  a bare 2-byte echo (no RTT, no panic). A BELABOX-compatible 2-byte *send* mode
  is a **deferred follow-up** — do not add it without live-interop evidence, and
  never change the keepalive wire format or the 32 B NAT-padding floor to do it.
  Full write-up: `docs/KEEPALIVE_INTEROP.md`. Pinned by
  `keepalive_extended_round_trip` / `keepalive_bare_2byte_accepted` /
  `keepalive_truncated_graceful` (`src/tests/keepalive_interop_tests.rs`).
- **Link-liveness timeout (`CONN_TIMEOUT = 15`, `src/protocol/constants.rs`):**
  seconds of inbound silence before an established uplink is declared failed and
  re-registered. It is deliberately set to **15**, matching the bonding receiver's
  `CONN_TIMEOUT` (`srtla/src/receiver_config.h:28`) and the C sender's
  `SENDER_CONN_TIMEOUT` (`srtla/src/sender_logic.h:66`). **Upstream irlserver ships
  5 s** — that value was inherited verbatim at fork time and is the accidental drift
  T12 reconciled (the receiver holds a link for 15 s while echoing keepalives, so a
  sender that gives up at 5 s falsely re-registers and resets the window on a link
  that is merely mid radio-stall). Real dead-link detection is unaffected — the
  send-failure path (`sender/packet_handler.rs` → `mark_for_recovery`) and quality
  scoring drop a dead/struggling link in ~1 s. **Do not let an upstream merge revert
  15 → 5**; the value (and the sender == receiver relationship) is pinned by
  `conn_timeout_value_pinned` (`src/tests/integration_tests.rs`).

## BUILD / GATE

Run the **full gate green on the pinned nightly** before every PR (it auto-selects via
`rust-toolchain.toml`):

```bash
cargo build --release
cargo fmt --all -- --check
cargo clippy -- -D warnings          # lib + bin (matches upstream ci.yml)
cargo test --lib
cargo test --all-features
cargo test --features test-internals # upstream's full-coverage suite
```

`test-internals` exposes internal fields for assertions; `--all-features` enables it.
The whole sender test corpus runs in-process over loopback UDP — **no root / netns /
CAP_NET_ADMIN required** (0 tests are gated/ignored).

**Loom model test (NOT part of the default gate).** `tests/subscription_loom.rs`
exhaustively model-checks the `SubscriptionManager` (`src/subscription.rs`)
`broadcast` (telemetry tick) vs `subscribe`/drop (control thread) interleavings
over its `Mutex<Inner>` + capacity-1 `sync_channel`. It is gated `#![cfg(loom)]`,
so it compiles to nothing — and the `loom` dev-dep stays unused — unless the
`loom` cfg is set. Run it deliberately (it is a dedicated BLOCKING CI job, not
part of `cargo test`):

```bash
RUSTFLAGS="--cfg loom" cargo test --features test-internals --test subscription_loom
```

It asserts four invariants under all schedules: no deadlock; `last_frame` equals
the broadcast frame; a non-full subscriber registered before a broadcast never
loses that frame (no lost wakeup); and a disconnected subscriber is pruned on the
next broadcast (no panic, no unbounded growth).

**Miri lane (BLOCKING CI job, NOT part of the default gate).** The only `unsafe`
FFI in the tree is the `recvmmsg` batch-receive path
(`src/connection/batch_recv.rs`). A dedicated BLOCKING `miri` job in `ci.yml`
runs miri over its **pure pointer logic** — three single-filter invocations (miri
takes one substring filter per run):

```bash
cargo miri test --lib --no-default-features --features test-internals init_rebuilds_self_pointers_after_move
cargo miri test --lib --no-default-features --features test-internals iter_clamps_oversized_msg_len_to_mtu
cargo miri test --lib --no-default-features --features test-internals sockaddr_storage_roundtrip
```

These vet, with no UB: the self-referential `iovec`/`mmsghdr` pointer rebuild
after a value move (`rebuild_pointers`/`init`), the `msg_len`→`MTU` clamp in the
iterator, and the `sockaddr_storage`→`SocketAddr` cast + big-endian decode.
**HARD LIMIT — miri CANNOT execute the real `recvmmsg` syscall/FFI.** It validates
only the Rust-side pointer arithmetic and decode *around* the syscall, never the
live kernel transition; any test that binds a socket, issues `recvmmsg`, or spawns
tokio must NOT run under miri (carry `#[cfg_attr(miri, ignore)]` if added — the
current `batch_recv.rs` tests are all syscall-free, so none need it). Two flags
are load-bearing: `--no-default-features` drops the mimalloc `#[global_allocator]`
(C FFI miri cannot run — it aborts on `mi_malloc_aligned`), and `--lib` scopes to
the unit-test binary holding the three pure tests. Install with
`rustup component add miri` on the pinned nightly.

The `@ceralive/srtla-send` TS binding (`bindings/typescript/`) has its own gate:

```bash
cd bindings/typescript && bun install --frozen-lockfile && bun run lint && bun run typecheck && bun test
```

`tsc --noEmit` typechecks **everything** via `tsconfig.json` (tests included). The
shipped build, however, emits via `tsconfig.build.json` (`extends tsconfig.json`,
`exclude: ["src/**/*.test.ts"]`) so compiled tests never land in `dist/` — the
published tarball is `dist/` non-test output + `package.json` only. Control tarball
contents at the **build-emit** layer, not `.npmignore`: with `files: ["dist"]` an
allowlist, `.npmignore`'s test-source pattern can't strip already-compiled
`dist/**/*.test.js`.

## CI / PACKAGING

Three workflows. The two Rust `.deb` workflows build on the **pinned nightly**
(`setup-rust-toolchain` with no `toolchain` input reads `rust-toolchain.toml`); the
binding-publish workflow is Node/Bun and shares no triggers with them:

- **`ci.yml`** (push/PR) — the gate (`fmt`, `clippy -D warnings` lib+bin, `check`,
  the full test fan-out, `cargo audit`) **plus** a `build-deb` matrix that
  cross-compiles `aarch64-unknown-linux-gnu` (device) and `x86_64-unknown-linux-gnu`
  and packages each `.deb` so a packaging break is caught before any tag. Upstream's
  stable/beta/windows/macOS jobs are kept; under the pin they must call `cargo +<channel>`
  (explicit `+` outranks `rust-toolchain.toml`) to actually exercise that channel.
- **`release.yml`** (tag push `v*`) — runs the full Rust gate plus the blocking loom
  and miri lanes in parallel; `build-deb` needs all three before rebuilding both
  arches, packaging, and attaching both `.deb`s + `.sha256`s to the GitHub release.
  No crates.io publish; no scheduled upstream-sync.
- **`publish-bindings.yml`** (tag push **`bindings-v*`**) — publishes
  `@ceralive/srtla-send` to the **public npm registry** (`@ceralive` scope,
  `registry-url: https://registry.npmjs.org/`) via npm **OIDC trusted publishing**
  (the `publish` job grants `id-token: write`, `npm publish --access public` — **no `NODE_AUTH_TOKEN`**;
  requires npm ≥ 11.5.1 / Node ≥ 22.14). Mirrors `@ceralive/cerastream`'s publish flow.
  The `test-bindings` job runs lint, typecheck, tests, build, and the tarball guard,
  uploads the validated `dist/`, and the separate `publish` job needs it before
  publishing. A `workflow_dispatch` run with `dry_run=true` performs a registry
  dry-run. **Binding version source:** the binding ships on its **own** tag namespace
  `bindings-vYYYY.M.P` (CalVer, matching `@ceralive/cerastream`), deliberately distinct
  from the Rust crate's `v*` release tags — the two namespaces keep the `.deb` release
  and the binding publish fully decoupled (no shared trigger). A `-rc.N` suffix publishes
  under the `next` dist-tag; a plain version under `latest`. The published version **is**
  the committed `bindings/typescript/package.json` `version`; the tag does not mint it.
  A guard step refuses to publish unless the tag's version equals `package.json` version,
  so a tag can never ship a stale/mismatched version. Cut a binding release: bump
  `package.json` `version` → commit → `git tag bindings-vYYYY.M.P && git push --tags`.

**`ci/build-deb.sh` is the single source of truth** for the `.deb` and is called by both
workflows. It pins the contract the device image depends on:

- **Package:** `srtla-send-rs`; **binary at** `/usr/bin/srtla_send`; **Architecture**
  `arm64` (aarch64 build) / `amd64` (x86_64 build).
- **Filename** `srtla-send-rs_<ver>_<arch>.deb` — matches `image-building-pipeline`
  `fetch-debs.sh`'s `*${ARCH}*.deb` glob; the script re-runs that exact glob as a
  self-test so a rename fails the build, not the image fetch.
- **`Conflicts: srtla (<< <cutover>)`** (and matching `Replaces:`) — pre-cutover srtla
  shipped the C `/usr/bin/srtla_send`, so the two packages file-conflict for any srtla
  release that still ships it. The bound is `SRTLA_CUTOVER_VERSION` (default `2026.6.2`),
  the first **receiver-only** srtla release (ADR-003 accepted): srtla `<< 2026.6.2`
  conflicts (C sender present); `2026.6.2` and later coexist (receiver only).

**Package versioning — upstream semver, NOT CalVer (approved exception):**
`srtla-send-rs` is the one first-party component that does NOT follow the CeraLive
CalVer (`YYYY.MINOR.PATCH`) scheme. Its `.deb` version comes directly from
`Cargo.toml` `[package] version`, which tracks upstream irlserver semver.
Current source package version: `3.2.0`. The workspace `versions.yaml` remains pinned at
the last published release, `v3.1.0`, until the 3.2.0 release is published and adopted.

Rationale: this repo is a fork of `irlserver/srtla_send`; keeping the upstream semver
line in `Cargo.toml` preserves direct traceability to upstream releases.

The GitHub release **tag** namespace is `v<package-version>`. A tag-triggered package
build must match the committed `Cargo.toml` version; `ci/build-deb.sh` rejects a tag ref
whose `GITHUB_REF_NAME` differs from `v<package-version>`. For this source version, the
only valid release tag is `v3.2.0`.

The `@ceralive/srtla-send` npm binding ships on its own `bindings-vYYYY.M.P` tag
namespace and uses CalVer independently of the Rust crate version.

See `CeraUI/docs/APT_VERSION_CONTROL.md` → "Exception: srtla-send-rs (upstream semver)"
for the full rationale and Debian version-ordering notes.

aarch64 cross-build env (mirrors the PINNED TOOLCHAIN note): linker
`CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc`, apt
`gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross binutils-aarch64-linux-gnu pkg-config`,
`PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig`.

## CODEBASE (inherited from upstream)

```
src/
  main.rs            CLI entry point (clap)
  lib.rs             library exports
  config.rs / config/    runtime config (DynamicConfig, ConfigSnapshot); stdin + Unix-socket control
  mode.rs            SchedulingMode (Classic | Enhanced | RttThreshold | Edpf)
  connection/        SrtlaConnection, bind/resolve, incoming packet handling, RTT (Kalman)
  protocol.rs        SRTLA protocol constants/structures
  registration.rs    REG1/REG2/REG3 flow + ID propagation
  sender/            packet forwarding + selection/ (BLEST → IoDS → EDPF), status logging
  tests/             unit / integration / e2e / protocol / registration suites
crates/network-sim/  dev-only network simulation harness (workspace member)
rust-toolchain.toml  pinned nightly (CERALIVE)
rustfmt.toml         unstable nightly fmt config (edition 2024)
ci/build-deb.sh      single-source .deb packager (control + filename + glob self-test)
.github/workflows/   ci.yml (gate + cross-build/package) + release.yml (tag-triggered) +
  publish-bindings.yml (binding gate + npm publish)
scripts/release_workflow_contract_test.py  release graph and failure-propagation checks
scripts/release_version_contract_test.sh  v3.2.0 tag/package/.deb version contract
```

Conventions (enforced by the gate): edition 2024, `anyhow::Result`, `tracing` macros,
Tokio async, imports grouped std → external → crate (module granularity), constants
`SCREAMING_SNAKE_CASE`. Four scheduling modes (classic, enhanced, rtt-threshold,
edpf); enhanced (default) adds NAK-decay quality scoring + optional exploration.
EDPF (`--mode edpf`) is Earliest Delivery Path First — a BLEST (static-OWD HoL
guard) → IoDS (bounded in-order constraint) → EDPF (lowest predicted arrival)
pipeline with per-loop owned scheduler state (no thread-local). See `README.md`
for the full operator/runtime reference (modes, runtime commands, tuning constants).

## ANTI-PATTERNS

- **Bindings are registry-only.** This repo ships its own pure-TS sender binding
  `@ceralive/srtla-send` under `bindings/typescript/` (public npm, `@ceralive`
  scope). It is consumed **registry-only** — **never add a sibling `link:` for it and
  never vendor a `.tgz`** (that is the `srtla/` → CeraUI pattern, not this one).
  `@ceralive/srtla` (the C-pair bindings) still lives in `srtla/`; do not duplicate it
  here.
- **No auto-sync with upstream.** Merges are manual + compat-gated (see policy above).
- **Don't unpin / silently bump the toolchain.** The exact nightly is load-bearing for a
  reproducible device build.
- **Don't break the parity contract** (binary name, CLI positional order, telemetry
  JSON shape, `bitrate_bps` ×8, SIGHUP reload) without a deliberate versioned change —
  CeraUI depends on it.
- **Don't strip upstream MIT/credits.** Layer AGPLv3 at distribution, keep notices.
- **No path above the repo root in any tracked file.** This repo builds, tests, and
  releases **standalone** in CI; the workspace parent does not exist there. The local
  orchestration scratch dir is gitignored and must appear in no other tracked file
  (Rule D).

## TEST HARDENING (Tasks 7-8, 25)

### Task 7 — Telemetry Rust test hardening

Two new integration-level test files complement the in-module unit tests in
`src/telemetry_file.rs`:

- **`tests/telemetry_edge_cases.rs`** (9 tests): zero connections (`connections:[]`
  idle-not-absent), active link with zero traffic (`bitrate_bps:0` present not absent),
  very-high RTT 5000 ms verbatim, `schema_version==1` pinned (constant + JSON,
  number-not-string, leads the document), `bitrate_bps == wire_bytes*8` on fixed
  inputs (0, 1, 150k, 312.5k, 1M bytes/s).
- **`tests/telemetry_fixture_parity.rs`** (3 tests): Rust golden
  `tests/fixtures/telemetry-golden.json` vs TS-binding golden
  `bindings/typescript/tests/fixtures/telemetry-golden.json` asserted byte-identical
  + structural (top-level keys, `schema_version==constant`, frozen 7-key per-conn set).
  Both anchored at `CARGO_MANIFEST_DIR` -- inside the repo, Rule D clean.

Key seam: `build_telemetry_json(last_updated_ms, conns)` takes an explicit ms arg.
Tests call it with a fixed timestamp (`1_749_556_546_000`) -- never `publish()` --
to stay non-flaky. Do not conflate with the tokio virtual-clock seam
(`advance_test_clock`), which is for timeout/keepalive tests only.

Gate note: `cargo clippy --features test-internals` is NOT a gate command (fails on
a pre-existing `tokio::time::advance` issue in `src/test_helpers.rs`). The real gate
is `cargo clippy -- -D warnings` (lib+bin only, matches `ci.yml:32`).

### Task 8 -- TS binding test hardening

New `bindings/typescript/tests/telemetry-reader.test.ts` (24 tests, 52 total after
adding to the existing 28):

- Valid golden fixture: full ADR-001 typed shape (both uplinks, all 7 per-link fields),
  `bitrate_bps` x8 invariant.
- Malformed input: non-JSON, truncated, empty string, non-object, absent file, each
  missing required field via `test.each`, wrong types, out-of-domain numerics -- all
  return graceful `null`.
- Schema version: `schema_version` 2/0/missing/non-numeric all return `null`.

`tsconfig.json` fix: added `tests/**/*` to `include`; moved `rootDir: "src"` into
`tsconfig.build.json` only. This ensures `bun tsc --noEmit` typechecks tests (not
just `src/`), while `bun run build` still emits only `dist/{index,sender/index,
telemetry/index}.js` with no test files. Tarball stays clean (`files: ["dist"]`
allowlist + build-emit excludes `*.test.ts`).

### Task 25 -- sendmmsg triage

The `// TODO: On Linux, could use sendmmsg ...` in `src/connection/batch_send.rs`
`flush()` has been converted to a tracked DEFERRED note. The note captures:

- What `sendmmsg(2)` would do (multi-datagram single syscall)
- Why it is deferred: marginal gain at current rates (~60-67 flushes/s at 10 Mbps,
  already a ~15x reduction from raw per-packet sends), Linux-only unsafe FFI, and
  complexity not justified without profiling evidence on the constrained device target
- When to revisit: profiling shows syscall overhead, or Tokio adds native support

Full rationale in `docs/notes/sendmmsg-deferred.md`.
**Do not implement sendmmsg** without profiling evidence and a deliberate PR.

## TS BINDING TOOLING

The binding package manager is **Bun**, verified by `bindings/typescript/bun.lock`;
run package commands from `bindings/typescript/` with Bun (`bun install --frozen-lockfile`,
`bun run typecheck`, `bun test`, `bun run build`). Do not introduce pnpm/npm/yarn
lockfiles for this package.

The `bindings/typescript/` package uses Biome 2.5 via `@ceralive/biome-config` as its first linter/formatter. The `biome.json` in `bindings/typescript/` extends `@ceralive/biome-config` (`"extends": ["@ceralive/biome-config"]`). ESLint and Prettier are not used. Run `biome check .` from `bindings/typescript/` (check) or `biome check --write .` (apply fixes). The binding gate includes `bun tsc --noEmit && bun test` — Biome is not a separate gate step but is expected clean before PR.

**Golden fixtures are excluded from Biome** — `biome.json` sets `files.includes` to `["**", "!**/tests/fixtures"]`. `tests/fixtures/telemetry-golden.json` is a deliberately byte-identical copy of the Rust producer golden (`tests/fixtures/telemetry-golden.json` at the crate root): the single-line, newline-free atomic-publish telemetry shape (ADR-001). If Biome pretty-prints it (multi-line + trailing newline), the cross-language parity test (`tests/telemetry_fixture_parity.rs` — `rust_and_ts_goldens_are_byte_identical` plus the newline-free assertion) fails every Rust test job in CI. **Do not remove this exclude, and never `biome check --write` the fixtures** — re-sync the two goldens by editing both byte-for-byte instead.

## EXPERIMENTAL SCHEDULER-HARDENING FLAGS (consolidated-flows-and-satellite, Todos 14-15)

Two CLI flags harden the default `enhanced` mode against a satellite/LAN failure signature
(a link that keeps a high scheduling weight while it silently degrades). Both are
**`[EXPERIMENTAL]` in their `--help` text and default OFF everywhere** (CLI parse default,
`DynamicConfig` atomic default, `ConfigSnapshot` default). Neither is validated against real
bond hardware — see the HARDWARE-VALIDATION GATE below. Full operator-facing description:
`README.md` → "Experimental Scheduler-Hardening Flags".

- **`earned_ack_window`** (`--earned-ack-window`, Todo 14) — gates broadcast-ACK window
  growth to the link that actually earned the ACK, with the rest growing at most once per
  `PROBE_GROWTH_INTERVAL_MS` (1000ms, `src/config.rs`) instead of unconditionally on every
  broadcast ACK. Wired through `apply_srtla_ack` (`sender/packet_handler.rs`) — ONE code path
  shared by production and tests, so flag-off is byte-identical to pre-flag behavior (proven
  by a golden-trace test). Tests: `src/tests/earned_ack_tests.rs` (16 tests).
- **`stall_deselect`** (`--stall-deselect`, Todo 15) — a selection-time-only penalty (never
  touches `CONN_TIMEOUT`/housekeeping/re-registration) that excludes a link from selection
  for one tick when its in-flight count exceeds `--stall-min-in-flight` (default 32,
  `STALL_MIN_IN_FLIGHT_PACKETS`) AND it has no earned ACK/RTT sample within
  `--stall-ack-stale-ms` (default 3000, `STALL_ACK_STALE_MS`) — tracked via the new
  `SrtlaConnection.last_ack_or_rtt_sample_ms` field, stamped only when a link actually earns
  an ACK or a keepalive RTT reply (never on generic inbound traffic, which is the exact gap
  this closes). Re-probed every `--stall-reprobe-ms` (default 1000,
  `STALL_REPROBE_INTERVAL_MS`) so a recovered link re-enters. All-stalled falls back to the
  normal selector so a link is always returned. Tests: `src/tests/stall_deselect_tests.rs`
  (11 tests + 1 `#[ignore]`d hardware-repro test).

**HARDWARE-VALIDATION GATE (unrun):** both flags are unit- and golden-trace-tested for
flag-off byte-identical behavior against the pre-flag code, but neither has been exercised
against a real bonded link (e.g. Starlink + cellular) outside this repo's in-process test
harness. Do not enable either flag in production, and do not cite either as a proven
improvement, until validated on real bond hardware. Mirrors the hardware-validation-gate
pattern used elsewhere in this workspace (see `docs/notes/sendmmsg-deferred.md` for how this
repo tracks a deferred/unrun item, and the [workspace diagnosis](https://github.com/CERALIVE/ceralive/blob/master/docs/notes/srtla-starlink-lan-diagnosis.md)
§6 for the mode-scoped mechanism analysis both flags address).

## ROBUSTNESS FIXES (robustness-pass, 2026-06-19)

Four behavior changes landed in the robustness pass. None alter the parity contract.

### S9 — all-links-failed timeout now measures elapsed-since-failure

`housekeeping.rs` armed the all-uplinks-failed global timeout with a helper
`instant_to_elapsed_ms(failed_at)` that computed `STARTUP.elapsed() -
failed_at.elapsed()` — effectively the uptime at the moment of failure, not the
time elapsed since it. On any session that had been running for more than 10 s, the
very first all-down tick tripped the timeout immediately, returning `Err` on what
could be a transient radio blip.

Fix: replaced the call with `failed_at.elapsed().as_millis() as u64` directly.
The helper `instant_to_elapsed_ms` was removed (it had no other callers). The
sender now correctly waits 10 s of continuous all-down before declaring a fatal
failure. Pinned by `all_failed_timeout_measures_elapsed_since_failure`
(`src/tests/integration_tests.rs`).

### S5 — dead uplink reader tasks are detected and restarted proactively

Previously, if a per-uplink reader `JoinHandle` exited unexpectedly (e.g. due to
a socket error), the connection stayed in the active pool but received no inbound
traffic. The 15 s `CONN_TIMEOUT` would eventually evict it, but in the meantime
the link appeared live to the scheduler.

The housekeeping per-tick loop now polls each active connection's reader
`JoinHandle.is_finished()` and calls `restart_reader_for()` on any that have
exited. Dead readers are detected and respawned within one housekeeping tick
rather than waiting for the liveness timeout. The hot path and `CONN_TIMEOUT`
value are unchanged.

### S6 — Kalman RTT clamped to ≥0; RTT-threshold classifier uses has_rtt_sample()

`get_smooth_rtt_ms()` (`connection/mod.rs`) now clamps the Kalman filter output
to `0.0_f64.max(value)` before returning. A negative Kalman estimate (possible
during filter warm-up on a link with high jitter) can no longer propagate to
callers.

The RTT-threshold mode classifier previously used `rtt <= 0.0` to detect links
with no measurement yet. That check is wrong after clamping: a link with a
clamped-zero RTT would satisfy `rtt <= threshold` and be auto-classified as fast.
The classifier now uses `has_rtt_sample()` (= `kalman_rtt.is_initialized()`) to
distinguish "no sample yet" from "measured but low". A recovering link whose
clamped RTT reads 0 is NOT auto-classified as fast; it routes via the capacity
fallback until a real sample arrives.

The clamp is placed at `get_smooth_rtt_ms()`, not inside `kalman.rs::value()`,
so the raw Kalman value remains available to the telemetry path (which already
saturates negative `f64 as u32` to 0) and to EDPF (which reads `value()` directly
and has its own floor).

### S7 — zero-RTT keepalive samples rejected (parity with ACK path)

The keepalive RTT guard in `rtt.rs` was `if rtt <= 10_000`. The ACK path in
`ack_nak.rs` already required `rtt > 0 && rtt <= 10_000`. A keepalive that
measured an RTT of exactly 0 (possible when the reply arrives within the same
scheduler tick) would previously feed a zero sample into the Kalman filter,
biasing it downward.

The guard is now `if rtt > 0 && rtt <= 10_000`, matching the ACK path. Zero-RTT
keepalive samples are silently discarded. Pinned by
`test_keepalive_zero_rtt_rejected` (`src/tests/`).

## DOCS DISCIPLINE (Rule A)

Any behavior/structure change updates this `AGENTS.md` and `README.md` in the SAME PR.
Keep the parity contract section authoritative — it is the device-integration contract.
