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

> **Status:** current source v3.2.0; CeraLive parity milestone v1.0.0 complete. Fork created from upstream HEAD;
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
> pipeline tests, production-linked concurrency/Miri/proptest gates, telemetry fsync moved off the
> packet-forwarding loop, keepalive interop (BELABOX 2-byte vs extended) + wire
> conformance goldens, mimalloc gated behind a default-on feature, netns de-flake to
> bounded readiness polling, and a final docs-consistency audit (T21: dangling-ref
> sweep via `scripts/check-doc-refs.sh`, version-drift + Rule-A sync).
> Startup bind ordering (2026-07-27: S10 — the local SRT listener binds before any uplink is
> dialed, closing a live-reproduced `SRT_REJ_TIMEOUT` start race; see ROBUSTNESS FIXES).
> **Upstream sync landed (2026-08): 0 commits behind `irlserver/srtla_send` at
> `c9f6bb2296f236d60802f2ec3b79d9da4dac6e28`.** All 138 commits in
> `80cd0c4..c9f6bb2` were triaged (`docs/notes/upstream-sync-2026-08-evaluation.md`)
> and merged history-only (`-s ours`); adopted substance landed as individually
> gated follow-up commits, not merge auto-application. Ported: NAK loss-list
> offset-16 with 31-bit wrap safety, unconnected uplink sockets +
> `sendmmsg(2)` batch flush with prefix-commit semantics, monotonic `now_ms()`
> (wall-clock carve-out for telemetry), ACK-RTT ownership attribution (wrap-aware),
> the RTT-velocity window-recovery gate, and an EDPF velocity+BDP-overrun ranking
> penalty (redesigned from upstream's hard-exclusion shape) — see ROBUSTNESS FIXES
> and EXPERIMENTAL SCHEDULER-HARDENING FLAGS below for the hardware-validation
> caveats that apply to some of these. DNS drift detection on reconnect landed;
> coordinated whole-bond receiver migration is DEFERRED (see ROBUSTNESS FIXES).
> Two candidate perf changes (switch-cooldown removal, flush-on-switch removal)
> were measured via a real A/B harness and REJECTED — see the triage doc.
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
- **Last-merged upstream SHA (2026-08 sync):** `c9f6bb2296f236d60802f2ec3b79d9da4dac6e28`.
  Merged history-only via `-s ours` (see `docs/notes/upstream-sync-2026-08-evaluation.md`
  for why); the next sync's merge base is computed from this SHA. Full 138-commit triage
  table and the EXACT-SET VALIDATION proving no commit in the range was missed or
  double-counted live in that same doc.

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
-only and absent from the release binary. Both 0.10.2 and 0.9.4 are patched for
RUSTSEC-2026-0097, so `deny.toml` no longer carries an advisory ignore.

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
  `--rtt-delta-ms`, `--control-socket`) stay present and functional but are **not**
  surfaced in CeraUI.
- **`-v/--version` IS operator-visible, and its build metadata is OPTIONAL.** CeraUI
  shells out to `srtla_send -v` and renders the raw stdout in Settings → Versions
  (`apps/backend/src/modules/system/revisions.ts`), so this line is read by humans, not
  only by scripts. Shape: `<version> [(<branch>@<hash>[-dirty>])] [<package>]` — the
  parenthetical is emitted ONLY when `build.rs` resolved a commit, so a build with no
  git context prints a bare `3.2.0 [srtla_send]`.
  **A build outside a git checkout is NORMAL, not broken** — an exported source tarball,
  a container that copies only `src/`, a vendored crate. The retired `build.rs` answered
  that case with the literal string `"unknown"` for both branch and hash, and — because
  `git diff --quiet` exits `128`/`129` (not `0`) with no repository at all, which the old
  code read through `!status.success()` as "dirty" — appended `-dirty` on top. The
  shipped device binary therefore read
  `3.2.0 (unknown@unknown-dirty) [srtla_send]`, asserting a branch, a commit, and
  uncommitted changes that all did not exist. CI was never the cause: `actions/checkout`
  provides `.git`, and both `.deb` workflows build in the checkout.
  `build.rs` now emits an EMPTY string for anything it could not resolve, treats a
  detached HEAD (`--abbrev-ref HEAD` == `"HEAD"`, i.e. every tag build) as "no branch"
  rather than a branch literally named `HEAD`, and only calls the tree dirty on an exact
  exit code of `1`. `src/version.rs` `compose_version_line()` owns the composition and is
  pinned by unit tests including the no-git-context case. Do NOT reintroduce a placeholder
  word for missing metadata, and do NOT infer "dirty" from a non-zero `git diff` exit.
- **Telemetry contract (`--stats-file <path>`, ADR-001 + ADR-002):** opt-in (absent ⇒ no file is
  ever written). Newline-free JSON document, atomically published (temp sibling →
  `fsync` → `rename(2)`), shape
  `{"schema_version":1,"last_updated_ms":<ms>,"connections":[{"conn_id","rtt_ms","nak_count","weight_percent","window","in_flight","bitrate_bps","bytes_sent_total"}],"bytes_sent_total":<bytes>}`.
  `bitrate_bps` is wire-bytes/s × 8 (the ×8 bits-per-second conversion is mandatory);
  `conn_id` is the string IP-list index (stable until a SIGHUP reorder); `window` and
  `in_flight` are **required** by the frozen `@ceralive/srtla` Zod reader. The cadence is
  `--stats-file-interval` ms (default 1000). The live file is unlinked on clean shutdown
  (SIGTERM/SIGINT). `schema_version` is additive over the C producer — the Zod reader
  strips it. Implemented in `src/telemetry_file.rs`; CeraUI parses this verbatim.
- **Cumulative session bytes (`bytes_sent_total`, ADR-002).** Additive at BOTH scopes:
  top-level (whole bond) and per-connection. **Unit is BYTES, and no ×8 is applied** —
  it is a count, not a rate, and it sits directly beside `bitrate_bps` (bits/s), which
  is the one place a consumer is most likely to introduce a factor-of-8 bug. Counted at
  the same call site as `bitrate_bps` (`queue_data_packet`), so DATA and SRT-level
  retransmits are IN and control frames are OUT, by construction. **Monotonic for the
  process lifetime:** it does NOT reset on a per-link socket replacement
  (`BitrateTracker::reset` rebases the rate window instead of zeroing the total) and
  does NOT regress when a SIGHUP reload drops a link — the bond figure is a delta-banking
  accumulator (`SessionBytes`, `src/stats.rs`), **not** a sum of the live links. It
  restarts at 0 only when the process does, i.e. on a genuinely new stream; it therefore
  survives a CeraUI backend restart that re-adopts a running stream. `schema_version`
  stays `1`. The TS schema marks it OPTIONAL: absent means UNKNOWN, never zero. Full
  rationale and the reset table: `docs/adr/ADR-002-session-bytes-telemetry.md`.
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
- **Startup listener bind ordering:** the local `SRT_LISTEN_PORT` listener is bound
  **first** in `run_sender_with_config`, before the ips file is read and before any
  uplink is dialed. CeraUI spawns the process and immediately dials that port with
  no readiness handshake, so anything awaited ahead of the bind is a window in which
  the encoder's SRT connect fails with `SRT_REJ_TIMEOUT`. **Never move the bind back
  below uplink setup** — the sequential per-link connect loop makes the window grow
  with the number of bonded modems. Pinned by `tests/startup_bind_ordering.rs`.
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
timeout --foreground --kill-after=10s 300s cargo test --all-features
timeout --foreground --kill-after=10s 300s cargo test --features test-internals
```

`test-internals` exposes internal fields for assertions; `--all-features` enables it.
Most tests run in-process over loopback UDP. The `tests/netns_*.rs` integration targets
are privileged supplements: they require Linux network namespaces, passwordless `sudo`
or equivalent `CAP_NET_ADMIN`, and external tools including `srtla_rec` and
`srt-live-transmit` (plus `tcpdump`/netem for relevant scenarios). They self-skip when
their dependency checks fail, so an ordinary CI run does not prove the privileged
topology. `NamespaceProcess` teardown is namespace-scoped: it signals only the exact PIDs
reported by `ip netns pids`, waits on bounded TERM/KILL grace periods, and never assumes
the tracked `sudo` child PID is a process-group leader or performs an unbounded `wait`.
`kill_returns_when_wrapper_and_inner_process_have_mismatched_groups` locks this behavior
without privileges and covers repeated teardown calls. Both namespace and veth names use
the shared PID+atomic-counter uniqueness suffix; do not replace the veth suffix with the
test-binary PID alone because scenarios inside one integration target run in parallel.
Never run the privileged targets unbounded: use `scripts/netns_test_gate.sh`, which caps
each target at 90 s by default. Separately, `stall_deselect_real_starlink_repro` is one
intentionally ignored hardware-only test; run it with `--ignored` only on the bonded
Starlink/cellular validation rig.

**Production subscription-concurrency invariant (BLOCKING, separate target).**
`tests/subscription_loom.rs` uses Loom to enumerate schedules while racing the real
`SubscriptionManager` (`src/subscription.rs`) `broadcast` path against its real
`subscribe`/drop operations. Under `cfg(loom)`, only the synchronization primitives
and capacity-one channel adapter change; the production manager state and methods are
the code under test. The behavioral invariants are that a concurrent subscriber
always receives the broadcast either live or through last-frame replay, and that a
disconnected subscriber is pruned by the next broadcast. Run the dedicated target
with the exact cross-repo command contract:

```bash
RUSTFLAGS="--cfg loom" cargo test --test subscription_loom
```

The `loom` job ID, display name, step name, environment, test filename, and command
are retained as a cross-repo CI contract. The former test copied the manager/channel
algorithm into an independent model, so it could stay green after production drift.
The replacement drives the production manager directly and checks externally visible
delivery, replay, and pruning behavior across the schedules Loom explores.

**Miri lane (BLOCKING CI job, NOT part of the default gate).** The only `unsafe`
FFI in the tree is the **`recvmmsg` + `sendmmsg` batch I/O** paths
(`src/connection/batch_recv.rs`). A dedicated BLOCKING `miri` job in `ci.yml`
**and `release.yml`** runs miri over their **pure pointer logic** — five
single-filter invocations (miri takes one substring filter per run):

```bash
cargo miri test --lib --no-default-features --features test-internals init_rebuilds_self_pointers_after_move
cargo miri test --lib --no-default-features --features test-internals iter_clamps_oversized_msg_len_to_mtu
cargo miri test --lib --no-default-features --features test-internals sockaddr_storage_roundtrip
cargo miri test --lib --no-default-features --features test-internals sendmmsg_pointers_rebuilt_after_move
cargo miri test --lib --no-default-features --features test-internals sendmmsg_prefix_extraction_bounded
```

These vet, with no UB: the receive-side self-referential `iovec`/`mmsghdr` pointer
rebuild after a value move (`rebuild_pointers`/`init`), the `msg_len`→`MTU` clamp
in the iterator, the `sockaddr_storage`→`SocketAddr` cast + big-endian decode, the
send-side `SendMmsgBatch` pointer rebuild after a move (`msg_iov` into its own
`iov`, `msg_name`/`msg_namelen` at the socket-owned peer), and the `sendmmsg`
accepted-prefix bounds plus short-`msg_len` hard error
(`validate_sent_prefix`). **HARD LIMIT — miri CANNOT execute the real `recvmmsg`
or `sendmmsg` syscall/FFI.** It validates only the Rust-side pointer arithmetic
and decode *around* the syscalls, never the live kernel transition; any test that
binds a socket, issues `recvmmsg`/`sendmmsg`, or spawns tokio must NOT run under
miri (carry `#[cfg_attr(miri, ignore)]` if added — the current `batch_recv.rs`
tests are all syscall-free, so none need it). Two flags are load-bearing:
`--no-default-features` drops the mimalloc `#[global_allocator]` (C FFI miri
cannot run — it aborts on `mi_malloc_aligned`), and `--lib` scopes to the
unit-test binary holding the five pure tests. `scripts/release_workflow_contract_test.py`
requires all five filters in BOTH workflows. Install with
`rustup component add miri` on the pinned nightly.

The `@ceralive/srtla-send` TS binding (`bindings/typescript/`) has its own gate:

```bash
cd bindings/typescript && pnpm install --frozen-lockfile && pnpm lint && pnpm typecheck && pnpm test && pnpm build
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
binding-publish workflow uses pnpm for package management and a pinned Bun runtime for
the binding's Bun-native tests/API; it shares no triggers with the Rust workflows:

- **`ci.yml`** (push/PR) — the gate (`fmt`, `clippy -D warnings` lib+bin, `check`,
  bounded tests with the privileged-netns self-skip boundary, `cargo audit`) plus typed
  workflow/ref/version contracts and a `build-deb` matrix that
  cross-compiles `aarch64-unknown-linux-gnu` (device) and `x86_64-unknown-linux-gnu`
  and packages each `.deb` so a packaging break is caught before any tag. Upstream's
  stable/beta/windows/macOS jobs are kept; under the pin they must call `cargo +<channel>`
  (explicit `+` outranks `rust-toolchain.toml`) to actually exercise that channel. The
  uv contract step uses the published `astral-sh/setup-uv@v8.3.2` release tag because
  setup-uv does not publish a `v8` major alias; do not shorten this ref to `@v8`.
  It also carries the **`bindings` job — the PR-gated TypeScript binding lane**
  (`pnpm install --frozen-lockfile`, `lint`, `typecheck`, `test`, `build` from
  `bindings/typescript/`, under **Node 26**, pnpm 10.30.3 and Bun 1.3.14). It is
  **REQUIRED, not a canary**: no `continue-on-error`, so a red binding blocks the PR
  like any Rust lane. `publish-bindings.yml` runs the same commands, but only on a
  `bindings-v*` tag — by then a break is already on `main`; this lane moves the gate
  onto the PR. Node 26 is the CeraLive CI baseline as of 2026-08-14 (root `AGENTS.md`
  → CI/CD STANDARD); do not pin Node 24 or older in any workflow here.
- **`release.yml`** (tag push `v*`) — runs the full Rust gate plus the blocking
  `loom` contract job (production subscription-concurrency invariant) and Miri lane in
  parallel; `build-deb` needs all three before rebuilding both
  arches, packaging, and attaching both `.deb`s + `.sha256`s to the GitHub release.
  No crates.io publish; no scheduled upstream-sync.
- **`publish-bindings.yml`** (tag push **`bindings-v*`**) — publishes
  `@ceralive/srtla-send` to the **public npm registry** (`@ceralive` scope,
  `registry-url: https://registry.npmjs.org/`) via npm **OIDC trusted publishing**
  (the `publish` job grants `id-token: write`, `npm publish --access public` — **no `NODE_AUTH_TOKEN`**;
  npm is pinned to `11.18.0`, above the trusted-publishing minimum of 11.5.1; Node is 26).
  Mirrors `@ceralive/cerastream`'s publish flow.
  The `test-bindings` job uses the committed pnpm lockfile to run lint, typecheck, tests,
  build, and the tarball guard, then uploads validated `dist/`. The OIDC `publish` job
  needs both that gate and `verify-release-ref`, which accepts only a tag-push event whose
  `bindings-v*` tag, package version, ref, checked-out commit, and event SHA agree. A
  `workflow_dispatch` run can reach only the separate non-OIDC `npm publish --dry-run`
  job; it cannot publish. **Binding version source:** the binding ships on its **own** tag namespace
  `bindings-vYYYY.M.P` (CalVer, matching `@ceralive/cerastream`), deliberately distinct
  from the Rust crate's `v*` release tags — the two namespaces keep the `.deb` release
  and the binding publish fully decoupled (no shared trigger). A `-rc.N` suffix publishes
  under the `next` dist-tag; a plain version under `latest`. The published version **is**
  the committed `bindings/typescript/package.json` `version`; the tag does not mint it.
  The provenance guard refuses stale, malformed, branch, dispatch, or mismatched refs.
  The tarball guard reads `npm pack --dry-run --json` **shape-agnostically** (array on
  npm 11, object keyed by package name on npm 12) — the sibling
  `publish-biome-config.yml` failed a release on exactly that shape change; keep both
  branches when touching it.
  Cut a binding release: bump `package.json` `version` → commit →
  `git tag bindings-vYYYY.M.P && git push origin bindings-vYYYY.M.P`.

### Rust dependency-cache policy (Wave 6.6)

Every Rust job in `ci.yml` and `release.yml` owns one explicit
`Swatinem/rust-cache@v2` step. `actions-rust-lang/setup-rust-toolchain` has
`cache: false` in these jobs so its implicit cache cannot compete with the
explicit key. The cache action covers Cargo's registry and git stores and the
workspace target directory (`. -> target`).

Keys include `runner.os`, `runner.arch`, the lane toolchain, the target/arch
matrix values for `.deb` jobs, the pinned `rust-toolchain.toml`, `Cargo.lock`,
the root `build.rs`, and relevant Rust source/manifests. `add-job-id-key: false` allows CI and release
lanes with the same semantics to reuse one cache. `cache-bin` and
`cache-on-failure` are disabled. The action saves dependency artifacts only,
removes stale/old target content, and disables incremental artifacts; this
keeps target reuse bounded instead of accumulating full workspace builds toward
GitHub's 10 GB cache limit. Miri intentionally caches only registry/git data
(`cache-targets: false`) because its target artifacts are specialized.

The CI `test` job runs `uv run scripts/rust_cache_contract_test.py`, which
locks coverage for all nightly, Loom, Miri, `.deb`, stable, beta, Windows, and
macOS Rust lanes in both workflows. The Rust cache action is the only cache
owner for those lanes; the binding workflow separately uses setup-node's pnpm
cache.

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
scripts/workflow_authority_contract_test.py  adversarial write/secret publication-authority checks
scripts/workflow_contract.py  typed semantic GitHub workflow graph/model
scripts/workflow_yaml.py  typed YAML-node and publication-authority parser
scripts/release_version_contract_test.sh  manifest-derived tag/package/.deb version contract
scripts/bindings_release_ref_contract_test.sh  binding tag/ref/version/SHA provenance contract
scripts/bindings_package_manager_contract_test.sh  root pnpm policy contract
scripts/netns_test_gate.sh  bounded privileged network-namespace test runner
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
- **Unconnected uplink sockets' accept-any-source behavior is deliberate — do not add
  silent source filtering without a versioned decision.** Uplink sockets are
  unconnected (`recvmmsg`-based), so any host that can reach an uplink's ephemeral
  port can inject ACK/NAK/keepalive traffic that influences scheduling and liveness;
  REG2 ID matching only hardens registration, not the data-plane receive path. This
  is a deliberate interop choice (matches the upstream C `srtla_send`/`_rec` pair and
  BELABOX, and tolerates NAT/multi-homed receivers replying from a different source
  address than the one dialed), not an oversight. The existing mitigation is defense
  in depth, not exclusion: a `foreign_source_datagrams` counter + rate-limited (1/s)
  `debug!` log on source mismatch, status-log-only (never in the frozen ADR-001
  telemetry JSON), and nothing is ever dropped. Adding a hard source filter, dropping
  foreign-source datagrams, or connect()-ing the uplink sockets per-peer would each
  break this interop case and must go through a deliberate, versioned decision — not
  a drive-by hardening patch. See `docs/notes/upstream-sync-2026-08-evaluation.md`
  ("Security rationale for accept-any sockets") for the full rationale.

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

New `bindings/typescript/tests/telemetry-reader.test.ts` (24 tests; 68 binding tests total):

- Valid golden fixture: full ADR-001 typed shape (both uplinks, all 7 per-link fields),
  `bitrate_bps` x8 invariant.
- Malformed input: non-JSON, truncated, empty string, non-object, absent file, each
  missing required field via `test.each`, wrong types, out-of-domain numerics -- all
  return graceful `null`.
- Schema version: `schema_version` 2/0/missing/non-numeric all return `null`.
- `src/telemetry/watch.test.ts` keeps six distinct watcher contracts without
  fixed sleeps: absent, stale-boundary, stop, file-appears, invalid-schema, and
  parsed-payload behavior. Callback/event-loop completion replaces timed windows.

`tsconfig.json` fix: added `tests/**/*` to `include`; moved `rootDir: "src"` into
`tsconfig.build.json` only. This ensures `pnpm typecheck` typechecks tests (not
just `src/`), while `pnpm build` still emits only `dist/{index,sender/index,
telemetry/index}.js` with no test files. Tarball stays clean (`files: ["dist"]`
allowlist + build-emit excludes `*.test.ts`).

### Task 25 -- sendmmsg triage (superseded: sendmmsg IS now implemented)

The original DEFERRED note for `sendmmsg(2)` batch send was **superseded by
adoption** in the 2026-08 upstream sync (todo 9), which ported upstream `673138d`
*feat(srtla_send): flush batches with sendmmsg* with fork fixes. `BatchSender::flush`
now submits up to `BATCH_SEND_SIZE = 32` datagrams per kernel entry via `sendmmsg(2)`
on Linux, with a sequential fallback on other platforms, over now-unconnected uplink
sockets (see the ANTI-PATTERNS note on accept-any-source above). Prefix-commit
semantics guarantee a partial send can neither duplicate nor drop a datagram; a hard
flush error routes into `mark_for_recovery()` + `SequenceTracker::remove_connection()`.
`docs/notes/sendmmsg-deferred.md` is retained as an **adoption record** (its title
is `ADOPTED: sendmmsg(2) batch send (was: DEFERRED)`), not a deferred-item pointer —
do not cite it as an example of an unimplemented/deferred feature. The triage row of
record is `docs/notes/upstream-sync-2026-08-evaluation.md` → `673138d`.

## TS BINDING TOOLING

The binding package manager is **pnpm**, pinned by `packageManager` and
`bindings/typescript/pnpm-lock.yaml`. Run package commands from
`bindings/typescript/` with pnpm (`pnpm install --frozen-lockfile`, `pnpm lint`,
`pnpm typecheck`, `pnpm test`, `pnpm build`). The package API and tests remain Bun-native,
so `pnpm test` invokes the pinned Bun test runtime; Bun is not the dependency manager.
Do not add Bun/npm/yarn lockfiles for this package.

The `bindings/typescript/` package uses Biome **2.5.8** via `@ceralive/biome-config` **2026.8.0** (the workspace canon — keep `biome.json`'s `$schema` on the same Biome patch) as its first linter/formatter. The `biome.json` in `bindings/typescript/` extends `@ceralive/biome-config` (`"extends": ["@ceralive/biome-config"]`). ESLint and Prettier are not used. Run `pnpm lint` from `bindings/typescript/` (check) or `pnpm exec biome check --write .` (apply fixes). The binding gate includes `pnpm lint && pnpm typecheck && pnpm test && pnpm build`.

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

## ROBUSTNESS FIXES (startup bind ordering, 2026-07-27)

### S10 — the local SRT listener binds before any uplink is dialed

`run_sender_with_config` (`src/sender/mod.rs`) used to bind the local SRT UDP
listener **after** `create_connections_from_ips()`. That loop is sequential —
one `resolve_remote` + `bind` + `connect` await per uplink
(`sender/connections.rs`) — so the local port CeraUI's encoder dials stayed
closed for the whole duration of uplink setup.

CeraUI spawns `srtla_send` and then immediately calls `start()` on the streaming
engine, which opens an SRT connection to `127.0.0.1:<SRT_LISTEN_PORT>` with no
readiness handshake in between (`apps/backend` `streamloop/start-stream.ts`).
Whenever uplink setup outran that connect, the handshake hit a closed port and
the operator got a hard, non-retriable stream-start failure — SRT rejection code
16, `SRT_REJ_TIMEOUT`, surfaced as `engine_internal`. It reproduced live on
device on 2 of 3 start→stop→start cycles.

The window scales with the bond: N modems means up to N sequential resolve +
connect round trips before the bind, so the failure gets **more** likely on the
multi-link deployments this sender exists for, not less.

Fix: the bind moved to the top of `run_sender_with_config`, ahead of
`read_ip_list` and the connect loop. Binding a UDP port depends on nothing the
uplinks provide, so the listener is open from the first instant the process is
alive regardless of how long the bond takes to come up. Nothing else moved —
uplink setup, `start_probing`, SIGHUP reload, `--dry-run` (which never reaches
this function), and the `main.rs` telemetry/config-listener startup order are
all unchanged, and no parity-contract behavior is affected.

Pinned by `tests/startup_bind_ordering.rs` (3 tests, unprivileged): the
`listening for SRT` log must precede the first `added uplink`, must precede
every uplink line of a multi-link bond including a `failed to add uplink`
attempt, and the port must genuinely be held (`AddrInUse`) once that log is
emitted. The first two fail on the pre-fix ordering.

## ROBUSTNESS FIXES (EDPF bonding, 2026-08-15)

`--mode edpf` did not work at all, and `tests/netns_edpf.rs` had been red since it
was written. Two independent defects in the EDPF pipeline, both inherited from the
upstream commits that introduced it (`27c6c00`, `80cd0c4`). Neither touches the
parity contract; the other three modes are unaffected (their selectors never call
the EDPF predictor).

### E1 — EDPF could never bootstrap (`src/sender/selection/edpf.rs`)

`predicted_arrival` returned `None` when `conn.bitrate.current_bitrate_bps <= 0.0`.
That field is a **measurement** of bytes this uplink has already sent, so it is
`0.0` on every link at process start. Every link therefore had no predicted
arrival, EDPF selected nothing, and the sender logged `no available connection to
forward packet` for every single DATA packet, forever — nothing sent means nothing
measured means nothing ever sent. Reproduced in a two-namespace bond: `--mode edpf`
forwarded **0** packets while `--mode enhanced` on the identical topology forwarded
1913.

Unmeasured links now use a flat `BOOTSTRAP_CAPACITY_BPS` (1 Mbps) placeholder. It
is deliberately flat, not modelled: every unmeasured link gets the same number, so
ordering among them falls to in-flight bytes and OWD, and a real measurement
replaces it within one 2 s bitrate window. This also covers a link idle longer than
that window, whose measurement decays back to `0.0`.

### E2 — BLEST permanently starved the high-latency uplink (`selection/mod.rs`)

`BlestFilter` is a static, capacity-blind OWD guard: a link more than 50 ms of OWD
behind the fastest is excluded on **every** tick regardless of congestion, and the
pipeline's fallback chain (`select_from_indices(candidates).or_else(select_from)`)
is only reached when the admitted set yields nothing — which never happens while
one fast link is admitted. A 30 ms + 150 ms bond therefore never bonded: after E1
was fixed, the 150 ms link still carried **0** packets over a 16 s window while the
30 ms link ran flat against its 4 Mbit cap and the excess was dropped.

`with_congestion_escape` re-admits a BLEST-excluded link while its predicted arrival
is **earlier** than every admitted link's. That is consistent with the guard BLEST
exists to enforce — a packet that lands first cannot head-of-line-block anything —
and it re-engages automatically once the fast link drains. Measured effect on the
netns scenario: link1 0 → ~3100 packets, aggregate 5860 → ~8960 packets per 16 s
window (~3.9 → ~5.9 Mbps), stable across 4 consecutive runs.

Do NOT restore "zero bitrate ⇒ not selectable" or drop the escape in an upstream
merge; `tests/netns_edpf.rs` plus the EDPF arms in `src/tests/edpf_tests.rs` and
`src/sender/selection/edpf.rs` pin both. Todo 11's EDPF velocity/BDP-penalty work
supersedes the flat bootstrap constant if it introduces a real capacity estimate;
the BLEST escape is orthogonal and should survive it.

## ROBUSTNESS FIXES (upstream sync, 2026-08)

Seven behavior changes landed as individually gated port commits during the 2026-08
upstream sync (todos 4-13). None alter the parity contract; the EDPF cold-start and
BLEST-starvation fixes are covered separately above under "ROBUSTNESS FIXES (EDPF
bonding, 2026-08-15)".

### NAK loss-list offset-16 (todo 4, ported from upstream `71f4ecc`)

The SRT NAK control frame's loss list starts at byte offset 16, not 4 — the naive
upstream port is risky without hardening (a range end of `0x7fff_ffff` wraps `seq`
to 0 under `seq <= end` with `wrapping_add`, potentially emitting up to 999 bogus
loss IDs before the 1000-entry cap silently truncates). The fork's port introduces
`SrtSeq`, a 31-bit modular sequence type (`src/protocol/srt_seq.rs`) with wrap-aware
`serial_lt`/`serial_le`/`serial_gt`/`serial_ge` comparisons (RFC 1982 semantics —
values exactly `2^30` apart are unordered in both directions, so `serial_lt` is
**not** a total order and must never back a `sort_by`), a directional `distance`,
and two constructors (`new` masks the protocol-flag bit; `from_u32_checked` rejects
it — using the wrong one on a NAK range-end word silently accepts a corrupt frame as
a huge valid range). `parse_srt_nak` returns a `NakList` with a global,
per-packet truncation cap and a rate-limited truncation warning
(`NAK_TRUNC_WARN_INTERVAL_MS = 1000`). Pinned by `src/protocol/srt_seq.rs` unit
tests plus migrated fixtures across `protocol_tests.rs`, `integration_tests.rs`,
`end_to_end_tests.rs`, and `tests/parser_proptest.rs`.

### Monotonic `now_ms()` with a wall-clock carve-out (todo 5, ported from upstream `bd6fad8`)

`src/utils.rs::now_ms()` is now backed by a monotonic clock so internal timing
deltas (NAK decay, window recovery, liveness timeouts) survive a wall-clock step
(NTP correction, manual clock change) without producing a spurious jump. A
**deliberate carve-out**: `wall_clock_ms()` stays on `SystemTime` and is the one
used for telemetry's `last_updated_ms`, because the TS-side telemetry watcher
compares that field against `Date.now()` — anchoring it to the monotonic clock
would drift after any wall-clock step and produce false staleness on the consumer
side. `std::time::Instant` (not `tokio::time::Instant`) backs the monotonic clock
specifically so it is NOT frozen by `tokio::time::pause()` in tests that rely on
`now_ms()` moving independently of the virtual clock. Pinned in
`src/tests/utils_tests.rs`.

### RTT-velocity window-recovery gate (todo 6, ported from upstream `a8d8a37`) — SIM-TESTED, NOT HARDWARE-VALIDATED

`perform_window_recovery` (enhanced mode) halves its window increment while the
Kalman-filtered RTT velocity exceeds `RTT_VELOCITY_GATE_THRESHOLD = 2.0`
**ms per Kalman update** (corrected from upstream's own mislabeled "ms/s" — the
filter's predict step has no `dt` term, so the unit is inherently per-sample, not
per-second). The existing 10s/7s/5s/else recovery schedule and the
`fast_recovery_mode` 2x bonus are unchanged; only the resulting increment is
scaled, and only ever downward, so the failure mode is slower recovery, never
window inflation. **This gate ships without the experimental-flag hardware
gate** used by `--earned-ack-window`/`--stall-deselect` precisely because it can
only reduce growth — but it is still sim-tested only (unit + golden-trace tests),
not exercised against real bonded hardware, and should be read with that caveat.

### ACK-RTT ownership attribution (todo 10, ported from upstream `a094863` + `3b2c425`)

An SRT cumulative ACK is broadcast to every uplink (all links still prune their
packet logs), but only the uplink the `SequenceTracker` says actually carried the
acknowledged sequence turns it into an RTT sample — previously every holding link
would have reported a round trip it never observed. A tracker miss means no RTT
sample on any link (conservative, intentional), not a fallback guess. An SRTLA ACK
names one specific sequence, so a packet-log hit is itself the ownership proof and
feeds the smoothed RTT directly with no additional gate. The `ack <=
highest_acked_seq` comparison that gates fast-path pruning is now the 31-bit
modular `serial_gt`/`distance`-based comparison (todo 4's `SrtSeq`), not a raw
integer `<=` — the raw form silently mishandled the sequence-space wrap. **Known
limitation, not a bug:** `record_round_trip` discards a measured RTT of exactly 0
ms (`saturating_sub`), so a genuine sub-millisecond round trip (loopback, LAN, a
colocated receiver) never contributes a sample. The fix is a finer clock, not a
looser gate — relaxing the `rtt == 0` rejection would reintroduce the S7
downward-bias regression on every real (non-sub-ms) link. `src/connection/rtt.rs`,
`src/connection/ack_nak.rs`.

### EDPF velocity + BDP-overrun RANKING penalty (todo 11, ported from upstream `57525c7` + `d53d8bc`) — SIM-TESTED, NOT HARDWARE-VALIDATED

Upstream's own shape (`DO-NOT-PORT-AS-IS` per the triage doc's frozen bug list) hard
-excludes a link once its bandwidth-delay product is exceeded — if every candidate
link is over the cap, selection returns `None` and the connection pool empties,
violating the fork's own "pool never empties" invariant (`src/sender/selection/mod.rs`).
The fork redesigns both as **ranking** penalties instead of exclusions:
`VELOCITY_PENALTY_FACTOR = 0.005` adds an RTT-velocity-proportional term to
predicted arrival time, and `BDP_OVERRUN_MULT = 1.5` (with a `propagation_s.max(0.001)`
1ms floor guarding the zero-RTT bypass) penalizes over-cap links in the EDPF argmin
without removing them from consideration — `edpf_all_links_over_cap_selects_the_least_
overrun_link` proves this is ranking, not filtering. EDPF is opt-in (`--mode edpf`,
never the default), and both constants are heuristics carried from upstream, not
field-derived — sim-tested (unit/golden tests plus the `netns_edpf` netem topology)
but **not** exercised on real bonded hardware. Same hardware-validation-gate
convention as `--earned-ack-window`/`--stall-deselect` applies: do not cite either
constant as a proven improvement until validated on real bond hardware.

### DNS drift detection on reconnect (todo 12, ported from upstream, MEDIUM-4 of `c9f6bb2`'s bug list) — single-uplink swap DEFERRED

Reconnect now re-resolves the receiver hostname (`resolve_remote_all`) instead of
reusing the previously-resolved `SocketAddr` forever (the pre-sync tree had the
identical flaw upstream also carried). The existing peer is kept when re-resolution
fails, when it remains among the fresh answers, or when drift simply omits it from
the answer set without another candidate being clearly preferred; a genuine drift
emits a rate-limited (at most once/minute) receiver-identity warning. **DEFERRED,
not implemented: coordinated whole-bond receiver migration.** SRTLA's
receiver-generated full ID makes swapping a single uplink to a different receiver
instance unsafe — it would split the bond, with some uplinks registered against one
receiver identity and some against another. `apply_connection_changes` deliberately
preserves surviving sockets/registrations across a SIGHUP reload, so SIGHUP is not a
substitute mechanism either. A later, separately-scoped change must specify and
implement any coordinated multi-uplink migration; do not add a single-uplink swap in
the meantime. `src/connection/mod.rs` (`host`/`port` fields alongside `remote`).

### Partial-send prefix-commit invariant + foreign-source counter (todo 9, alongside unconnected sockets — see ANTI-PATTERNS)

`flush_batch`'s `sendmmsg(2)` path commits (`register_packet`) only the
kernel-**accepted prefix** of a queued batch, in order, even when the overall call
returns an error — a partial send can neither duplicate nor silently drop a
datagram, and the caller (`sender/packet_handler.rs`, `sender/housekeeping.rs`) must
`mark_for_recovery()` + `seq_tracker.remove_connection(conn_id)` on any `Err`. This
replaces upstream's HIGH-2 bug (draining the whole queue and registering it as sent
before confirming the I/O succeeded). Because uplink sockets are now unconnected, a
datagram from a source other than the dialed receiver can reach protocol state; the
`foreign_source_datagrams` counter plus a 1/s rate-limited `debug!` (status logs
only, never the frozen ADR-001 telemetry JSON) is the mitigation — see the
ANTI-PATTERNS entry above for the full accept-any-source rationale and why nothing
is ever dropped. `src/connection/batch_send.rs`, `src/connection/mod.rs`.

### REG3 authorization is one-shot; index-scoped registration state resets on SIGHUP (Final Verification Wave, `eaced59` + this round)

Six registration-hardening bugfixes, all found by the post-merge verification wave.
None alters the parity contract.

- **REG3 is a ONE-SHOT grant.** `handle_reg3` consumes the uplink's `awaiting_reg3`
  entry on success, so a duplicate or replayed REG3 falls through to the out-of-phase
  branch (counted in `out_of_phase_reg3`, `RegistrationEvent::Reg3OutOfPhase`) instead
  of re-firing `clear_pre_registration_state()` and wiping a live uplink's packet log,
  in-flight count, congestion state, and batch queue. A legitimate reconnect re-arms
  the gate through `send_reg2_to`.
- **A SIGHUP pool reorder resets index-scoped registration state**
  (`reset_index_scoped_state`: `awaiting_reg3`, `pending_reg2_idx`, `reg1_target_idx`,
  probe results). Those are positional indices into the connection vector, which SIGHUP
  rebuilds in ips-file order, so an in-flight grant could otherwise authorize a REG3 on
  whichever uplink inherited the index. Only *incomplete* attempts are discarded —
  established links keep their own `SrtlaConnection::connected` state, socket, and
  window, so the "no re-handshake, zero disconnect" reload contract is unaffected
  (confirmed by manual reorder QA).
- **The REG2 broadcast retry skips uplinks that no longer need one.** A partially
  failed broadcast retries on the next tick; the retry pass now skips any uplink that
  is already `connected` or already holds a live `awaiting_reg3` grant. Without the
  skip, the retry re-`insert`ed the index and re-armed the one-shot gate on an
  already-connected, actively-forwarding link, reintroducing the first bug through a
  slow multi-uplink broadcast instead of a raw duplicate packet.
- **REG_ERR is phase-gated the same way REG3 is (round 3).** `handle_reg_err` used to
  act on ANY REG_ERR: it set `connected = false` on the receiving link and cleared the
  GLOBAL `pending_reg2_idx`/`reg1_target_idx`. SRTLA control frames are unauthenticated
  and the uplink sockets are unconnected, so a forged 2-byte `SRTLA_TYPE_REG_ERR` from
  anything that could reach an uplink's ephemeral port was a one-packet remote DoS
  against an established, forwarding link — and collaterally aborted an *unrelated*
  uplink's concurrent handshake. It is now honored only when that index is genuinely
  mid-registration (`pending_reg2_idx == Some(idx)` OR `awaiting_reg3` member);
  otherwise it is counted in `out_of_phase_reg_err` and returned as
  `RegistrationEvent::RegErrOutOfPhase`, which `packet_io.rs` treats as a no-op. When
  in phase, the clearing is SCOPED: the global REG1/REG2 fields are cleared only when
  this index owns that single in-flight slot; an `awaiting_reg3`-phase REG_ERR revokes
  only its own grant.
- **A failed REG2 send revokes any stale pre-existing grant (round 3).** `send_reg2_to`
  arms `awaiting_reg3` only on a successful send, but on failure it used to leave an
  older grant for the same index untouched — so a failed RESEND could keep a REG3
  authorization alive for a socket generation that was never re-armed. The `Err` branch
  now removes the entry, so "armed only by a send that left the host" holds for the
  current generation.
- **REG_NGP is accepted only when nothing is in flight anywhere (round 4).** Accepting a
  REG_NGP restarts the handshake at REG1, which re-opens the `pending_reg2_idx` window
  the round-3 REG_ERR gate honors a REG_ERR in. The old condition
  (`active_connections == 0 && pending_reg2_idx.is_none()`) let a forged REG_NGP do that
  on a link that had just completed REG3: `active_connections` is recomputed only by a
  later housekeeping tick, so it still read zero while `SrtlaConnection::connected` was
  already `true`. `handle_reg_ngp` now additionally requires `awaiting_reg3.is_empty()`
  (no outstanding REG3 grant on ANY index) and takes the uplink's own `connected` flag —
  threaded through `process_registration_packet` from `packet_io.rs` — because that flag,
  not the manager's counter, is authoritative in that window. This supersedes the
  "deliberately NOT changed" REG_NGP residual recorded in the round-3 evidence.
- **The REG3 wait now actually times out (round 5).** `handle_reg2` clears
  `pending_reg2_idx` and re-points `pending_timeout_at_ms` at a `REG3_TIMEOUT` (4s)
  deadline in the same statement block, but `clear_pending_if_timed_out` only fires
  `if let Some(idx) = self.pending_reg2_idx` — already `None` by then — so that deadline
  was set and never consulted and an `awaiting_reg3` grant never expired. Combined with
  round 4's gate that made the blockage PERMANENT: a receiver that restarts between our
  REG2 and its REG3 answers the group it no longer knows with a perfectly legitimate
  fresh REG_NGP, which the sender then refuses for the life of the process. The new
  sibling `clear_awaiting_reg3_if_timed_out` — called from the same housekeeping tick,
  right after `clear_pending_if_timed_out` — revokes every outstanding grant at the
  deadline, zeroes `pending_timeout_at_ms`, drops a queued REG2 rebroadcast (it would
  re-arm the grants just revoked, with an id the receiver may no longer know), and
  re-opens the REG1 path. `pending_timeout_at_ms` is one slot that `handle_reg2` sets
  exactly once per successful REG2, so it is one deadline for the whole REG3-wait phase
  and the grants expire together; a pending REG2 cedes ownership back to
  `clear_pending_if_timed_out`. This is a timeout-bounded recovery, not an open door —
  the fresh REG1 → REG2 → REG3 cycle re-arms and re-gates, and a merely late link keeps
  its socket and is re-registered by the housekeeping `CONN_TIMEOUT` path.

Pinned by `replayed_reg3_does_not_wipe_a_live_connection`,
`reg2_broadcast_retry_skips_already_connected_uplinks`,
`out_of_phase_reg3_is_counted_and_ignored`,
`out_of_phase_reg_err_does_not_disconnect_a_live_uplink`,
`out_of_phase_reg_err_does_not_damage_another_uplinks_handshake`,
`in_phase_reg_err_still_aborts_the_registration`, and
`failed_reg2_resend_revokes_a_stale_pre_existing_grant`
(`src/tests/batch_io_tests.rs`), `reg_ngp_rejected_while_any_uplink_awaits_reg3`,
`reg_ngp_rejected_on_connected_uplink_with_stale_active_count`,
`forged_reg_ngp_cannot_reopen_the_reg_err_window_on_a_live_uplink`,
`reg3_timeout_fires_at_4s_logical` (`src/tests/registration_tests.rs`),
`expired_reg3_grant_lets_a_fresh_reg_ngp_restart_registration`
(`src/tests/batch_io_tests.rs`, driven end-to-end through
`SrtlaConnection::process_packet`), `housekeeping_expires_a_stale_reg3_grant`
(`src/sender/housekeeping.rs`, pinning the production call site), plus the SIGHUP reset
assertions in `src/tests/sender_tests.rs`.

## DOCS DISCIPLINE (Rule A)

Any behavior/structure change updates this `AGENTS.md` and `README.md` in the SAME PR.
Keep the parity contract section authoritative — it is the device-integration contract.
