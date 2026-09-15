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

> **Status:** current source v3.3.0; CeraLive parity milestone v1.0.0 complete. Fork created from upstream HEAD;
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
> **Upstream sync landed (2026-09): 0 commits behind `irlserver/srtla_send` at
> `df0b3938791ff24eced4aed8b29e3d49d0efb639`.** All seven commits in
> `c9f6bb2..df0b393` were evaluated
> (`docs/notes/upstream-sync-2026-09-evaluation.md`) and merged with a true
> two-parent merge commit. NAK/ACK wrap safety, recovery, and registration gates were
> already stronger in the fork; the remaining RTT unit labels, DNS diagnostic liveness,
> and registration-log amplification fixes were adapted in fork-native follow-ups.
> Upstream's default-on whole-bond receiver re-home remains DEFERRED after review found
> socket-coherence and stale-reader-generation gaps, and its `4.0.1` bump was not imported:
> the published fork release stays `3.3.0`.
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
- **Last-merged upstream SHA (2026-09 sync):** `df0b3938791ff24eced4aed8b29e3d49d0efb639`.
  Merged with a normal two-parent merge commit after compatibility resolution; the next
  sync's merge base is computed from this SHA. The seven-commit verdict table, exact-set
  validation, selective adaptations, and explicit re-home/version exclusions are in
  `docs/notes/upstream-sync-2026-09-evaluation.md`. The prior 138-commit sync remains
  documented in `docs/notes/upstream-sync-2026-08-evaluation.md`.

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
- **Release ABI floor is Debian 12 / GLIBC 2.36 for BOTH architectures.** The tag
  workflow's `build-deb` matrix runs inside `debian:bookworm-slim`; never move those
  builds back onto the raw `ubuntu-latest` userspace. The runner label currently maps to
  Ubuntu 24.04 / glibc 2.39, which produces binaries that cannot start on the Bookworm
  device image. The job inspects each final ELF's versioned imports and rejects anything
  above `GLIBC_2.36`. Its Rust target cache key includes `bookworm` so an older raw-runner
  build script or target artifact cannot be restored into the container.

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
- **Optional bind-map sidecar (`--bind-map <path>`, ADR-003) — ADDITIVE, never required.**
  `BIND_IPS_FILE` stays **byte-unchanged**; the mapping rides a *separate* versioned JSON
  sidecar that describes it **positionally** (the Nth row describes the Nth accepted IP
  line, which is what disambiguates duplicate-IP twin modems). Header carries
  `{schema_version, generation, ips_file_sha256}`; rows carry
  `{link_id, ip, iface, id_path?}`. **Absent `--bind-map` ⇒ byte-identical legacy
  behavior** — the module is not entered at all, pinned by
  `a_legacy_invocation_without_bind_map_produces_byte_identical_output`
  (`tests/bind_map_contract.rs`, literal stdout). Coherence is one-directional: the
  sidecar names the exact ips-file bytes it describes. A mismatch is retried
  (5 attempts × 400 ms, ≤ 2 s ceiling) because the writer's two-rename publication window
  produces exactly that transient; a mismatch that outlives the budget **fails open,
  duplicate-safe** — at STARTUP a same-IP collision group keeps one deterministic
  representative and the rest are excluded **and reported**; on a valid→degraded RELOAD
  the sender **retains the last valid mapped pool** rather than silently un-binding a live
  bond. Full contract: [`docs/adr/ADR-003-bind-map-contract.md`](docs/adr/ADR-003-bind-map-contract.md).
  The mapping is **acted on**: a mapped link's socket is bound with
  `SO_BINDTODEVICE` **and** `bind(ip, 0)` (`DeviceBinder`, `src/connection/socket.rs`),
  so egress leaves the named interface *and* the wire source address is deterministic —
  neither half substitutes for the other, and an unmapped link still takes the
  `SourceIpBinder` path verbatim.
- **Link identity is the sidecar's `link_id`; `(ip, iface)` is only the current socket
  key.** Registration, stats, and telemetry state attach to `link_id`, which is stable
  across reloads, reconnects, and interface changes; dedup runs on the socket key, which
  is what tells two same-IP twin modems apart. A reload that moves a `link_id` onto a
  different `(ip, iface)` **recreates the socket and the registration** rather than
  carrying window/packet-log/in-flight state across — every one of those is scoped to the
  interface it was measured on. `src/connection/spec.rs`, `src/sender/connections.rs`.
- **The interface is re-resolved by NAME on every socket creation, and a stale socket is
  never reused.** `SO_BINDTODEVICE` resolves a name to an ifindex **once**, at
  `setsockopt` time, so a replugged modem leaves the socket holding an index that no
  longer names it — `sendto` then answers `ENODEV` (gone) or `ENETUNREACH` (down), and
  neither heals. Housekeeping re-resolves each tick (bounded detection: one interval, not
  a `CONN_TIMEOUT` wait): a changed ifindex forces a rebind, a vanished interface puts the
  link in a **`removed`** state that waits for a reload instead of burning the reconnect
  backoff, and an `ENODEV` send does the same from the data path.
  `src/connection/egress.rs`, `src/sender/egress_tick.rs`.
- **Route invariant is link health, and it is DISTINCT from ACK liveness.** A device-bound
  socket whose interface has lost its default route does not fail: IPv4 assumes the
  destination is on-link, ARPs for the receiver's public address, and drops the packet
  while `sendto` reports success. So per-interface default-route presence is **observed**
  (read-only, `/proc/net/route`) and reported on its own axis in the status log — never
  inferred from send success, and never merged into the ACTIVE/TIMED_OUT line. **No policy
  routing is introduced**: nothing installs a rule, a route, or a table.
  Because the invariant is re-read every housekeeping tick but the status log only prints
  every 30 s, each **crossing** is additionally announced when it happens: losing the route
  is a `WARN`, regaining it an `INFO`. A crossing into or out of `Unknown` is deliberately
  silent — an unreadable route table is not evidence either way, and every link's first
  observation leaves `Unknown`. `src/connection/route.rs`
  (`classify_route_transition`), `src/sender/egress_tick.rs`.
- **A `SIGHUP` with `--bind-map` runs the ADR-003 read protocol off the forwarding loop.**
  The bounded pair read (hash-coherent pair, or fail-open) is spawned and answered back
  into the event loop, because a retried hash mismatch would otherwise stall packet
  forwarding for up to 2 s. Without `--bind-map` the SIGHUP path is the legacy
  reload guard, unchanged. `src/sender/links.rs`.
- **`--capabilities-json` is the pre-spawn probe (ADR-003 §7).** One-shot, side-effect
  free, exits `0` with a single-line JSON capability document on stdout (before logging is
  initialized). **The load-bearing half is the caller's:** non-zero exit, unparseable
  output, or a timeout means NO SUPPORT — fall back to the legacy spawn and never pass
  `--bind-map`. The shipped `3.2.0` binary answers this flag with `error: unexpected
  argument` and exit `2`, which is precisely that signal; callers must treat *any*
  non-zero exit the same way rather than matching on the code or the message.
- **The runtime `get-capabilities` returns the SAME document, plus `methods`.** The
  JSON-RPC method on `--control-socket` emits every key of `capability_document()`
  verbatim and adds an additive `methods` array (the control methods and event topics
  ADR-001 requires, which previously occupied the `capabilities` key). Pre-spawn probe and
  live socket must never disagree about what a build can do; pinned by
  `get_capabilities_matches_the_pre_spawn_probe_document`. **`hello`'s `capabilities`
  stays a string array** — the TS control binding feature-detects with
  `hello.capabilities.includes(...)`, so that field is frozen.
  `get-status` additionally returns `bind_map_status`, `disposition`, and a `links` array
  of `{conn_id, iface?, link_id?}` in telemetry order.
- **`-v/--version` IS operator-visible, and its build metadata is OPTIONAL.** CeraUI
  shells out to `srtla_send -v` and renders the raw stdout in Settings → Versions
  (`apps/backend/src/modules/system/revisions.ts`), so this line is read by humans, not
  only by scripts. Shape: `<version> [(<branch>@<hash>[-dirty>])] [<package>]` — the
  parenthetical is emitted ONLY when `build.rs` resolved a commit, so a build with no
  git context prints a bare `3.3.0 [srtla_send]`.
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
  strips it. The document model lives in `src/telemetry_doc.rs` (schema, units,
  serializer) and the publish mechanics in `src/telemetry_file.rs`, which re-exports the
  model so existing `telemetry_file::` import paths are unchanged; CeraUI parses this
  verbatim.
- **ADR-003 telemetry echo — four OPTIONAL additive fields, `schema_version` STAYS 1.**
  Per connection: `iface` (the interface the socket is bound to) and `link_id` (the
  sidecar's writer-assigned opaque identity, **echoed** — the sender never mints one).
  Top level: `bind_map_status` `{state: active|absent|degraded, reason?}` with the seven
  frozen ADR-003 §6.4 reasons, and `disposition`
  `{state: mapped|retained_last_valid|legacy_unique_only|startup_collision_excluded,
  collisions?}`. The two are **orthogonal**: a degraded RELOAD keeps the last valid mapped
  pool running (`retained_last_valid`) while a degraded STARTUP excludes the ambiguous
  rows (`startup_collision_excluded`) and publishes the group it broke up — the colliding
  IP plus the **`BIND_IPS_FILE` line positions** (NOT `conn_id`s) that are effective vs
  excluded. That startup-exclusion / reload-retention split is now directly observable
  instead of inferable from log text. Every field is omitted (never `null`, never `""`)
  when it does not apply, so an unmapped/legacy run's document is byte-identical to the
  pre-ADR-003 producer's plus the top-level pair. **`schema_version` names the shape of
  the REQUIRED fields, not the set of fields present** — the schema grows only by
  addition, added fields are always optional, and the version is reserved for renaming,
  retyping, or REMOVING a required field or changing a unit. Do not bump it for an
  additive field. Types come from `src/bind_map/report.rs`, which projects the existing
  `BindMapStatus`/`BindMapDisposition`/`CollisionGroup` — do NOT introduce a parallel
  status type. `SharedStats::set_bind_map` holds the mode on its own lock and
  `SharedStats::get` composes it, because `update` rebuilds the snapshot on every
  housekeeping tick while the mode changes only on a reload.
- **`conn_id` is RETAINED but TRANSIENT; UI identity is `link_id`.** `conn_id` is a
  position in `BIND_IPS_FILE`, so a SIGHUP reorder hands the same modem a different one.
  It stays in the schema for compatibility and for correlating records within one
  snapshot. Anything that must survive a reload, reorder, reconnect, lease change, or
  interface move MUST key on `link_id`. Two twin modems on one source IP are
  distinguishable only by it. Pinned by the `telemetry-reordered` / `telemetry-reconnect`
  fixtures on both sides.
- **Cross-language fixture matrix — Rust writes, TypeScript parses THE SAME BYTES.**
  Nine fixtures, each committed twice (`tests/fixtures/<name>.json` and
  `bindings/typescript/tests/fixtures/<name>.json`) and asserted byte-identical by
  `tests/telemetry_fixture_parity.rs`. The producer half is `tests/telemetry_fixtures.rs`
  (regenerate deliberately with `UPDATE_GOLDEN=1 cargo test --test telemetry_fixtures`,
  which rewrites BOTH copies); the consumer half is
  `bindings/typescript/tests/telemetry-fixtures.test.ts`. `telemetry-legacy-producer.json`
  is the **frozen** pre-ADR-003 producer document and is NEVER regenerated — it is the
  old-shape side of the compatibility proof, and a test asserts it contains none of the
  four additive keys. Do not `biome check --write` any fixture (see TS BINDING TOOLING).
- **Cumulative session bytes (`bytes_sent_total`, ADR-002).** Additive at BOTH scopes:
  top-level (whole bond) and per-connection. **Unit is BYTES, and no ×8 is applied** —
  it is a count, not a rate, and it sits directly beside `bitrate_bps` (bits/s), which
  is the one place a consumer is most likely to introduce a factor-of-8 bug. Counted at
  the same call site as `bitrate_bps` (`queue_data_packet` for originals and SRT-level
  retransmits; accepted-prefix processing for duplicate probes). Both DATA forms are
  IN and control frames are OUT, by construction. **Monotonic for the
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

**Hard-admission correction (2026-09-15), live acceptance still blocked:**
`sender/wire_admission.rs::configure` now returns `None` for Adaptive's hard
budget, retaining the estimator update but never installing `WireBudget`.
`RateCap`'s soft BDP ranking multiplier is unchanged and does not read
`WireRateEstimator::rate_bps`. Without the budget, `wire_sample()` is absent and
the retained estimator update rebases instead of actively searching; do not claim
continued attempted-wire measurement. The four frozen wire-rate/queue/search/budget
implementation files remain byte-identical. The new fixed-clock regression first
failed with `funded_link == None` after a queue-induced 500 kbit/s estimate and
two-MTU reservation; it now admits and actually flushes three MTUs. Existing
learned-rate/reconnect and forwarding tests assert the new no-hard-gate contract.
Three isolated A indices (two attempts each, unchanged criteria) still fail
`settle_timeout`; this correction is not sufficient for Scenario-A acceptance.
Both full feature-suite invocations fail G's demotion and Twins sustained-health
checks, while I passes; D remains ignored in those invocations. Build, Clippy and
library tests pass. Formatting still reports the two pre-existing spans in unchanged
`tests/netns_adaptive.rs`. No all-green gate, no C1 restart, and no scheduler
acceptance is claimed. Historical hard-admission descriptions below are superseded.

**ACK-frame RTT correction (2026-09-15), also insufficient for Scenario A:**
Adaptive credits every valid link-local original/probe ACK as before, but supplies
at most one RTT sample per SRTLA ACK frame, from its final receiver-order entry.
A final probe, missing/expired original, replay, or Karn-ambiguous retransmission
supplies none; never substitute an earlier entry that includes coalescing delay.
`SrtlaIncoming` retains frame boundaries even when draining multiple datagrams.
Legacy per-sequence RTT/window behavior is unchanged. `rtt.rs` adds only a test
module, not an estimator change. The real-dispatch staggered-ten-original regression
first recorded ten samples (870 down to 60 ms), Kalman 112.40 ms/jitter 170 ms; it now
records only 60 ms, Kalman 60 ms/jitter 0. Seven tests cover mixed/probe-only frames,
Karn exclusion, receiver order, replay, separate frames, and legacy behavior.

The admission isolation comparison stashed only the three inherited Rust changes:
HEAD `5d54052` failed G (30/61 demoted); baseline Twins passed once and failed a
second run, including both sustained-health assertions. Restored admission-only
Twins passed. Thus both failure signatures predate admission removal; this does
not establish equal failure rates or waive either blocking assertion.
The combined immutable candidate's isolated A smoke still failed all six attempts
(three indices, unchanged two-attempt budget) with `settle_timeout`. Sink data
shows transient 23.78–24.20 Mbps one-second peaks but only 7.02–11.07 Mbps in final
one-second buckets. Admission-only evidence likewise had transient ~24 Mbps peaks,
so neither zero-goodput failure placeholders nor peaks establish sustained delivery.
Retained sender log tails contain NAK floods but no health/RTT status lines and no
literal 1000-sequence truncation warnings; truncated logs cannot prove their absence
throughout a run. Stable health and RTT improvement are unverified live. No C1
restart, active-search redesign, telemetry expansion, or acceptance-criteria change.
Final gate: release build, Clippy, changed-file formatting, nine changed Rust files'
LSP diagnostics, and library tests (883 passed, one ignored) pass. Both feature
suites pass 908 library tests (one ignored), then fail G (34/61 and 49/61 demoted)
and Twins sustained health; I passes. Later integration targets are not reached.
Whole-tree formatting still reports only the unchanged `netns_adaptive.rs` spans.
Both fixes remain uncommitted pending the owner's decision on the failed live gate.

**Scenario-A limitation and owner decision:** the estimator retains the exact `d168aa6`
wire-rate implementation. Post-Todo-28 provisional-learning, DemandLimited-utilization and
repeat-reset experiments passed deterministic tests but failed live A acceptance and
have been reverted. Their mechanisms, full provisional test source, measured limits,
and the C1/C2 follow-up are archived in
[`docs/notes/scheduler-evaluation-2026-09.md`](docs/notes/scheduler-evaluation-2026-09.md#known-limitation-adaptive-mode-baseline-topology-throughput-instability-scenario-a-discovered-post-todo-28).
Do not reintroduce them as accepted fixes or describe an unidentified fourth gate as
proven. Todo 28's acceptance remains scoped, including D's accepted4/5 residual.
For Todo29 only, the final owner calibration makes all D cells and adaptive/A informational
and requires at least one of two successful planned indices in each of classic/A and
enhanced/A. All twelve index outcomes and provenance are validated; failures are never
relabelled. `report.py --smoke-coverage` accepts only the two canonical required smoke
cells and preserves actual indices/counts; default complete-manifest reporting remains
unchanged. This is coverage, not a strict majority or performance proof. C1/C2 remain
strict; no settling predicate, scheduler constant or runtime source has changed.
The existing final campaign was retrospectively rescored without new live runs:
classic/A2/2 and enhanced/A1/2 pass this final scope. All D cells and adaptive/A
remain0/2; the original full-matrix exit101 remains a failure. The
[portable report](docs/notes/smoke-final-report-2026-09.md) and
[receipt](docs/notes/smoke-final-receipt-2026-09.json) preserve that distinction.
Todo29's measurement-path acceptance does not accept an adaptive scheduler fix.

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
each target at 90 s by default. **`netns_bond` gets 120 s; `netns_twin` gets 420 s
(`NETNS_TWIN_TEST_TIMEOUT_SECONDS`)**: its scenarios wait out real sender timers no other
target touches — the 15 s `CONN_TIMEOUT` and the 30 s status-log interval — so a shared
budget would make it flake at exit 124. Separately,
`stall_deselect_real_starlink_repro` is one intentionally ignored hardware-only test; run
it with `--ignored` only on the bonded Starlink/cellular validation rig.

**Adaptive integration target:** `tests/netns_adaptive.rs` uses generic
`BondTopology`/temporal profiles, the real FIFO-controlled SRT source, and the shared
host measurement lock. Four privileged scenarios pin D, G (60s), I, and duplicate-IP
twins including the legacy falsifiability control and both in-blackhole priority
mutations. Two existing unprivileged helper tests are included through shared modules.
`NETNS_ADAPTIVE_TEST_TIMEOUT_SECONDS` defaults to 360s in the bounded gate. Set
`SRTLA_REC_BIN` explicitly for CeraLive receiver provenance;
`NETNS_ADAPTIVE_ARTIFACT_DIR` optionally preserves observations/sink/log evidence.
The protected legacy twin suite is unchanged. **Todo 30 closes only as a scoped
evaluation with documented findings, not a green adaptive gate.** Nine non-adaptive
targets have historical privileged-pass evidence (27 tests); no fresh ten-target
pass is claimed. I passes with its blocking precondition restored. D remains
explicitly ignored, Twins-share informational, Twins-health unresolved, and G's
wide reliability variance deferred to Wave 6 / Todo 32's N=5+ C2 campaign.
G and Twins-health assertions stay blocking. No self-skip is integration validation.

**Oracle consult #16 / Todo 30: scoped completion; unconditional success claim withdrawn.**
The priority-snapshot fix is retained. Removing I's blocking precondition in
`58e7207` was unauthorized: the required wire-budget attribution had not been done.
The final correction restores that commit's parent version of the test, including
the legitimate settling window, in a separate restoration commit. The owner ended
single-run investigation after the baseline Twins comparison. Accepted D and
Twins-share dispositions do not waive G or Twins-health, and the literal
every-target-passes bar remains unmet. Final findings are under KNOWN LIMITATION
in `docs/notes/scheduler-evaluation-2026-09.md`; later round narratives are historical.

1. **Twins preferred-share assertion removed as blocking.** The priority mechanism is a
   bounded RANKING bias (max 1.2x multiplier via `score × (1 + p·clamp(...))`), not a
   share allocator. Theoretical best case: `1.2/(1+1.2) = 54.545%`, which is BELOW the
   old 55% floor. The consistent ~50% result across 3 isolated runs does NOT demonstrate
   a priority-plumbing bug — it demonstrates the test's target range was never guaranteed
   by the implemented contract. The `(0.55..0.70).contains(&preferred)` assertion is
   removed as a blocking requirement; the observed share is logged for informational
   purposes only. The rest of the twins test (mapping, dual REG3, legacy falsifiability
   control, priority set/clear snapshots, feasible-load recovery, final health) remains
   blocking.

2. **D's obstruction-recovery test marked `#[ignore]` with explicit reason.** The
   wire-rate/stall-detector coupling issue (documented in rounds 8, 14, 17, 18) produces
   a 4/5 historical pass rate on isolated runs. A single hard `assert!` cannot honestly
   represent that behavior. The test is marked ignored with the reason string
   `"wire-rate/stall-detector coupling: 4/5 historical pass rate; N-run statistical
   evaluation deferred to Todo 32"`, moving its N-run statistical evaluation into the
   planned scheduler-redesign scope.

3. **I's receiver-restart precondition is BLOCKING again.** The legitimate settling
   window (`t >= 17.0 && t < 20.0`) remains; every sampled sink rate in that window
   must reach 90% of 12.8 Mbit/s. The 70% sample-fraction rule and informational-only
   treatment are removed. Rolling-window measurement alone is not evidence of a
   brittle assertion. The fresh isolated run passed all 15 settling samples
   (12.738880–12.886272 Mbit/s), with REG3 recovery in 17.455938485s and sink recovery
   in 19.773828501s. Both links' final-five-second wire-budget events report
   11,313,708.49898476 bps and `DemandLimited`, not low/Draining. This run does not
   reproduce or explain the earlier precondition failures; it does not justify
   extending scenario A's known limitation to I. Exact event counters and measurement
   semantics are recorded in `docs/notes/scheduler-evaluation-2026-09.md`. The owner
   closes I as resolved/non-issue for Todo 30 on this restored-check evidence,
   not as a claim of permanent reliability or a reason to remove the precondition.

**Priority-snapshot bug fix (commit 361d644):** After applying a pool control RPC request
(e.g., `set-link-priority`), the telemetry snapshot was not refreshed immediately. The
snapshot was only updated at the next housekeeping tick, causing it to lag behind the RPC
application by up to one housekeeping interval. Fix: call `adaptive_state.update_stats()`
immediately after applying a pool control request in the event loop. The priority snapshot
now correctly reflects the applied value immediately after the RPC.

**G's wide-variance reliability characteristic:** identical current source produced
29/61, 44/61 and 0/61 demoted snapshots; exact `d168aa6` produced 4/61 in the endpoint
comparison. All met 3MB carriage, but the first two failed the unchanged ≤10% demotion
condition. No test namespace/process contamination was observed. The earlier 0–13%
band and 0/61 ×3 sampled too little to characterize the now-observed 72.13% tail.
The owner assigns this to deeper scheduler reliability, not an assertion defect or
an identified commit regression. Both endpoints passed; that is not causal proof
excluding all code or host effects. Wave 6 / Todo 32 must characterize it with N=5+
statistical runs; more single-run diagnostic cycles are out of scope. No new waiver.

**Final Twins baseline comparison:** exact `d168aa6` failed overall, but full-rate
health PASSED (all 93 final-window samples Healthy; recovery at restore+4.854316s).
Failures were the old clear-RPC snapshot staleness (`Some(0.2)` after clear) and
50.105526% preferred share versus the old 55% floor. The harness accumulates failures,
so those failures did not hide the health assertions. This does NOT confirm a
pre-existing full-rate health failure. Earlier current-state health failures remain
unresolved; a single baseline pass identifies no causal mechanism. The owner ended
diagnosis here, not by ignoring or loosening health checks. `361d644` stays; all
`src/connection/wire_rate*` files remain at `d168aa6`. Scoped completion records the
findings and future work honestly; it is not scheduler acceptance or a full-gate pass.

**Recovery-load scope and new evidence (Todo 28 round 8):** four oracle consults
justify testing recovery under feasible load, not assuming aggregate admission that
does not exist. D now holds 6.4 Mbit/s from t=20 to t=46 (28s restoration +17s
deadline +1 telemetry tick), then restores its actual 22.4 Mbit/s offered rate;
twins hold 6.4 Mbit/s until t=38 (28+9+1), then restore 12.8 Mbit/s. Bounded
Healthy recovery, no relapse, and separate full-rate health/share assertions remain
blocking. The ignored `immediate_full_rate_restoration_stress` records the old
schedules without a health/share pass gate. The test launcher suppresses the
per-NAK congestion log flood so the bounded process tail retains health/RTT lines.
No production mechanism or threshold changed in round 8.

**This correction is NOT yet sufficient:** final real runs still fail D/G/twins;
I passes. D reaches Healthy at restore+15.5916s, but misses Stalled/0 and loses
Healthy after full load. Twins do not rejoin until restore+15.2844s (>9s), and the
first priority-set snapshot again omits the applied key. Instrumented feasible-load
twins have two successful probe rounds and zero queue/loss on the probe path, yet
the oldest round's age (e.g. 2697ms) exceeds the 2060.24ms freshness allowance;
this is a separately exposed recovery timing problem, not overload during that
low-load interval. Do not widen the gate without a separately verified correction.

G remains an independent failure, NOT an approved threshold-expectation exemption.
The installed GE model is `5% 0.1% 0.8% 0.02%` (~0.7847% stationary loss), not
5% aggregate loss or 80% bad-state loss. A real zero-loss marginal-link control
still incurs 43,690 qdisc drops and 36/61 demoted snapshots. Final unchanged G
carries 3,937,450B but is demoted 45/61; none of its three 5Mbit companions has a
Healthy sample. Per-link allocation/recovery collapse needs investigation; the
evidence does not justify relaxing ≤10%. See
`docs/notes/scheduler-evaluation-2026-09.md`. Todo 28 remains open.

**Sampled recovery freshness (Todo 28 round 9):** the acquisition formula was
correct for ONE held/probed link in the two-link twin topology, but its consumer
is a 1Hz housekeeping sampler, not an immediate ACK callback. `HealthSignals`
now carries `observation_interval_ms`: runtime supplies the existing
`HOUSEKEEPING_INTERVAL_MS`, immediate policy fixtures use zero. Only the Stalled
recovery freshness limit adds that interval to `rejoin_span`; train ACK deadlines,
epoch ordering, two qualified rounds, stall/queue/loss thresholds and the rejoin
ramp are unchanged. This also covers sampled original-DATA recovery evidence.
Do not substitute delayed wall-clock execution time or an unbounded allowance.
The real scheduler/log regression (105ms opportunities, 60ms ACK RTT, oldest-start
age2697ms, completion-to-observation642ms) first failed Stalled versus Rejoining;
it now passes, with stale/pre-epoch and four-ACK negative guards. Final twins
reach Rejoining+3.071572s and Healthy+5.086991s within9s, before full-rate restoration,
without relapse. Later full-rate health/share remains red; this is a partial fix.

G's original source load is12.8Mbit; actual original DATA is1332B for1316B payload
(12.9556Mbit), below the companions'15Mbit total even without the marginal link.
Round9 accepted-prefix captures show the marginal initially assigned2.2768Mbit
of DATA, then companion original+retry rates6.0017/6.2453/5.9146Mbit before their
health demotions. Early traffic is ranked admission, with zero pre-registration,
sole or fallback bytes in those intervals; retransmissions and real queues grow.
This falsifies the simple aggregate-rate explanation but is not yet a proven
specific ranking-code defect or an approved startup-control redesign. G's scenario
and≤10%assertion remain unchanged. Final I passes; D/G/twins remain failing targets.

**Retransmission accounting audit (Todo28 round10):** no R-specific admission or
accounting bypass was found. The sender forwards NAKs back to the local SRT caller;
it does not generate payload retries. R-marked DATA re-enters `handle_srt_packet`
and the normal adaptive selector/queue/accepted-prefix path. Every queued retry
counts in BitrateTracker, every accepted retry in delivery attempts/loss-send
accounting, and ACKed retry DATA in `DeliveryLedger::delivered_bps`. Karn exclusion
affects RTT sampling only. Four new real-UDP/production-dispatch tests pass against
unchanged production behavior (`retransmit_accounting_tests.rs`, adaptive forwarding).

Keep the accounting distinctions explicit: queued count is per datagram; flight
and the delivery ledger are keyed by sequence. Repeated same-sequence outstanding
copies count all wire bytes but one unresolved sequence and one ambiguous ACK
credit. `RateCap` intentionally consumes ACK-derived delivered throughput, NOT
BitrateTracker's transmitted rate; its positive BDP multiplier is not a sent-byte
pacer. This does not prove accurate physical queue occupancy or convergence under
retry amplification. No speculative controller change, G scenario change or
threshold relaxation was made. Latest real gates remain I PASS, D/G/Twins FAIL;
round9's bounded twin recovery still passes. Todo28 remains unchecked.

**Historical attempted-wire admission (Todo 28 round 11 — NOT acceptance-green;
superseded by the hard-admission correction above):** adaptive
mode acquired a separate hard gate, `connection/wire_budget.rs`, integrated at
`BatchSender::flush` and `sender/wire_admission.rs`. Its input is the existing
per-link `RateCap.target_bps()` estimate; RateCap's code and positive ranking
multiplier are unchanged. This is an enforced estimate, NOT a configured or known
physical link capacity. Monotonic credit has a two-MTU (3000-byte) burst bound;
every kernel-accepted queued datagram debits it, including originals, R-marked
retries and probes. Failed/unaccepted suffixes do not spend credit; an unfunded
suffix remains queued and is not a socket error. Same-sequence copies each pay.
SRTLA registration/keepalive control remains on its existing unpaced control path.

Before enqueue, an exhausted preferred link yields to another funded link from
the existing health/deadline-admitted set; budget pressure cannot reopen a held
link or override sole-carrier retention. With none funded, the input returns
`SrtPacketOutcome::Backpressured`, retaining ONE datagram and pausing local UDP
reads. A 1ms timer drains funded prefixes/retries that datagram while ACKs,
housekeeping, control and signals remain live. This bounds user-space backlog,
not end-to-end UDP loss: kernel receive buffering is finite and the producer is
not guaranteed lossless backpressure. Legacy modes disable the gate. No feature
bit was added; adaptive wire enforcement is always on, including when the eight
older ablation bits are disabled. Default feature bits themselves are unchanged.

DEBUG housekeeping records `wire_budget_bps` and `attempted_wire_bytes` (UDP-payload
bytes, not IP/link framing or confirmed goodput). This diagnostic counter is scoped
to the enabled budget epoch, reset on socket/batch reset or leaving adaptive mode;
it is NOT ADR-002 session bytes. Stats-file schema/weights remain unchanged.
Optional netns artifacts now retain final qdisc JSON alongside these debug logs.

**Known new limitation/regression:** enforcing the soft controller's target exposed
its unsuitable bootstrap/capacity estimate. In the one round-11 G run all three
companions stayed Healthy (61/61 each), their peak sampled attempted rates were
2.908096/2.222816/2.855872 Mbit/s, and all four qdiscs had zero non-model drops.
But marginal budget rose to 1.104081 Mbit/s against its 1Mbit capacity, and it
remained Degraded in31/61 samples (all queue-labelled), despite4,369,830B carriage.
I now FAILS throughput: pre-restart sink2,684,640bps and no90% recovery, although
process/REG3 timing passes. D passes unchanged; twins pass recovery/final health
but fail share and first-post-RPC snapshots. This experiment is NOT ready to merge
or deploy. No G recalibration or round12 tuning is authorized; Todo28 stays open.

**Queue investigation (Todo 28 round 6):** temporary explicit-now minima captures in
G and twins found slow floors of 60–61 ms and elevated fast minima, not isolated low
outliers. Twin kernel qdisc capture confirmed actual TBF backlog/drop growth after
restoring 12.8 Mbit/s while only one 8 Mbit/s link was admitted, followed by overload of
the recovering link. `RateCap` is a positive soft ranking penalty, NOT pacing;
G also queued at Bootstrap and while Holding below 1 Mbit/s, so target-ramp growth
alone does not explain it. No RTT filter, health threshold or rate policy was
changed in this round. The new G diagnostic also had two loss-labelled demotions:
the prior run's queue-only attribution must not be generalized to every run.

**Integration correctness follow-up (still incomplete):** adaptive cumulative SRT
ACKs retain congestion pruning but do not supply per-path RTT; specific ACKs sample
from the delivery ledger's kernel-acceptance timestamp only when unambiguous.
`FlushOutcome.retransmitted` is aligned one-for-one with the unchanged accepted
tuple and marks the SRT R bit. Retransmitted/repeated outstanding sequences still
credit DATA proof and delivered bytes, but cannot poison adaptive RTT minima.
All legacy RTT paths and frozen traces are unchanged. A pending probe train does
not erase preceding successful rounds, and Degraded/Stalled→Rejoining begins a new
normal-loss epoch only after health policy has qualified recovery. A validated
changed receiver full ID invalidates old-ID pending REG3 grants; connected links
still skip the broadcast and REG_NGP acceptance is unchanged. These mechanisms have
five failing-first regressions. D/G/twin live integration remains red; receiver
restart passes after separating process respawn from its downstream-SRT preflight.
`BondRuntime::receiver_restart_elapsed()` excludes UDP readiness; deadlines in the
adaptive test are anchored at respawn, not at the later readiness observation.

**Original-traffic recovery and temporal loss attribution:** the plan's prohibition
on duplicate probes to the elected sole carrier is preserved. A separate bounded
original-DATA witness in `connection/original_recovery.rs` observes accepted sends
while Stalled and their arrival/generation-exact ACKs. It reuses two ten-packet,
at-least-five-ACK train qualification with the existing one-link deadline and health
freshness checks; zero/four-ACK controls stay Stalled. It emits nothing and does not
alter ordinary delivery/window/byte accounting. Socket resets and new health epochs
cannot reuse previous evidence. This fixes the missing original-traffic recovery
path, but live D/twins can still relapse after reaching Rejoining; no all-green claim.

**Generic bond topology (`crates/network-sim/src/bond.rs`).**
`BondTopology::new(test_name, &[LinkSpec], MappingMode)` accepts 1–253 links;
`LinkSpec` carries explicit `CarrierMode::{Direct,Nat}` and an optional zero-based
`shared_ip_with` reference to an earlier link. Shared groups require NAT on every
member, one carrier per link, and an explicit `BindMap { rows }` or `LegacyControl`
policy. `MappingMode::None` with sharing returns typed `BondConfigError` before any
namespace creation; distinct-IP users explicitly pass `None`. `BondRow` supplies
`link_id`, `iface_index`, and optional finite `priority`; row order defines IP-file
order. The priority is only an additive fixture field, not scheduler support.
`TwinRow::new` stays unchanged; `with_priority` is additive, and absent priorities
retain exact legacy sidecar bytes. Heterogeneous-IP publication extends the existing
`BindMapPublisher` under `bond/publication.rs`, keeping the twin files independent.
The bond owns namespaces and launch files, not process handles: drop every
`NamespaceProcess` before dropping its topology. `sender_args`/`spawn_sender` apply
the mapping policy; receiver/profile selection stays at the caller layer.

NAT setup uses the proven `100.64.N.0/24` transit, carrier MASQUERADE, receiver
loopback `10.99.0.1` and source-hinted return routes. Distinct links all get source
tables (including link 0); shared links use the twin-style per-device defaults.
Both namespace and per-device `rp_filter` are cleared. No asymmetric fault routes
or packet marks are installed. All names use the shared PID+counter helper.
`replug(i)` deletes/recreates the access veth (new ifindex, shaping resets);
`delete_default_route`/`restore_default_route` operate on the link's source-table
and main-table defaults. `tx_bytes` is raw netdev traffic, including the ARP probes
that continue without DATA in a route blackhole. `tests/netns_bond.rs` has its own
**120 s** gate budget (`NETNS_BOND_TEST_TIMEOUT_SECONDS`), separate from the twin
budget; it covers mixed and shared-IP carriage, the legacy control, route failure
and restoration, and replug. The A/B runner's routing helper is now exported from
`network_sim::bond`; its legacy topology and measurement protocol are unchanged.

**Temporal profiles (`crates/network-sim/src/profile.rs`, Todo 10).** Model,
expansion, monotonic scheduler, qdisc commands, runtime adapter and cross-traffic
live in separate `profile/` modules. `Periodic.until` is the inclusive last onset,
not an observation endpoint; each actual onset + hold + horizon must fit the run.
Defaults: 30s recovery tail, 5s periodic tail, zero for load edges. One-shot holds
end at an explicit restoration of the previous property value; otherwise they are
instantaneous. `graded=false` never disables horizon validation. Periodic restores
use the pre-onset state, not blindly the initial base. Nested periodic events and
overlapping writes to held properties are rejected. Equal-time restores execute
first, then stable declaration order. The synchronous `Scheduler` anchors a
monotonic clock on first run, records successful actual timestamps, waits through
the observation tail, and never retries a failed/partially applied event.

**Generic bonds exclusively use `LinkQdisc`, NEVER the legacy root-clearing API.**
Root `prio` handle `1:` has sixteen zero priomap entries; band `1:1` is netem `10:`
or TBF `10:` → netem `11:`, band `1:2` is netem `20:` loss 100%. Blackhole toggles
only `tc filter add/del ... parent 1: protocol ip prio 10`, using exactly
`u32 match u16 0x0400 0xfc00 at 2 flowid 1:2` on add. That is IPv4 length
1024–2047. An update replaces only band one, including while DATA is blackholed;
kind changes detach only band one. `ImpairmentConfig.queue_limit` is a netem packet
limit; `delay_distribution` supports Normal/Pareto; `tbf_latency_ms` defaults to
1s. With a netem child, its packet limit owns the backlog (TBF latency is not an
additional total-delay bound). Legacy `apply_impairment` retains root deletion for
legacy/twin callers; never pass a generic bond interface to it.

`BondRuntime` borrows the topology and exact sender/receiver handles and requires
an offered-rate callback that changes the real source. It uses receiver port 5000.
Replug restores the current impairment/filter and explicit route/link state;
link-up restores source routes the kernel removed on link-down. Reorder publishes
topology indices with stable link IDs, a coherent sidecar/hash and increasing
generation before HUP. Cross-traffic uses iperf3 when present, otherwise paced
Python UDP; both bind source IP AND device and use a server in the receiver ns.

**Process-only restart is distinct from final teardown.** NamespaceProcess now
stores argv/env and discovers the exact inner PID using before/after namespace PID
sets plus wrapper ancestry (serialized harness spawns), with a process start-time
identity check before signaling. `pid()` never returns the sudo wrapper in its
place. `restart_process_only()` rejects an exited child with downcastable
`ProcessControlError::AlreadyExited` before signaling anyone; it stops only the
inner PID and respawns identical argv/env. Auxiliary cross-traffic uses
`spawn_process_only` so error/drop cleanup cannot invoke namespace-wide kill.
The existing `kill()` and ordinary handle Drop remain namespace-wide teardown.
The original stack exposes `restart_receiver()`. The `netns_bond` 120s target now
also covers mid-blackhole impairment updates (plain and TBF), keepalive RTT while
DATA stalls, listener/sink PID preservation, both load backends and event dispatch.

**`tests/netns_twin.rs` — duplicate-IP twin-modem scenarios (8 tests).** This target
that reproduces two uplinks sharing ONE source address, which is what the bind-map exists
for. It needs a topology no other target has, built by `crates/network-sim/src/twin/`:
each twin sits behind its **own NAT carrier namespace**, because a plain veth pair would
answer both uplinks at the same address and route every reply down one interface, so the
second device-bound socket would never register — a topology artifact that reads as a
sender bug. Two settings are load-bearing and were both found the hard way: per-device
`rp_filter` must be cleared (the effective value is `max(all, dev)`, and strict
reverse-path silently eats the shared address's ARP replies on the second twin), and the
receiver answers from the address the sender dialed via a `src` hint on its return routes.
Scenarios: both twins register and carry simultaneously; the same topology WITHOUT
`--bind-map` leaves the second twin dead weight (the falsifiability control — without it
the bonding assertion proves nothing); reload remove/re-add under a stable `link_id`; a
degraded reload retaining the mapped pool; a file-order swap that recreates no socket; a
well-formed but unorderable republication refused as `stale-generation`; an
unplug/replug recovering on a genuinely new ifindex; and a route-removal blackhole
reported on the route axis, confirmed by ACK timeout, never reading healthy.

**`tests/netns_hsrsp_spike.rs` — ignored live HSRSP visibility spike.** Requires
an explicit `SRTLA_REC_BIN` pointing at an out-of-tree CeraLive receiver build,
unattended sudo, `srt-live-transmit`, `tcpdump`, and `tshark`; explicit execution
fails on absent prerequisites. Run under a 90-second outer timeout with `--ignored`.
Fresh one-link pairs at listener `latency=2000` and `latency=500` yield matching
HSRSP delays, with identical UDP payloads captured on the sender uplink and loopback.
The spike's clean-room decoder stays test-only (`tests/support/hsrsp.rs`); Todo 18
adds the separate production parser below. The committed 80-byte raw-payload fixture
`tests/fixtures/srt-hsrsp-latency2000.bin` is regenerated only with `UPDATE_GOLDEN=1`.
Offsets are zero-based from the SRT/UDP-payload start: extension type `[64,66)` = 2,
body length `[66,68)` = 3 words, latency `[76,80)` = `07 d0 07 d0`.
The high half `[76,78)` and low half `[78,80)` both decode to 2000 here; the
500ms control changes both to 500. This symmetric capture alone does not distinguish
directional half semantics. Extension lengths are walked, not assumed fixed by
the decoder. `HSRSP_CAPTURE_DIR` optionally preserves unique pcap/log directories.

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
cd bindings/typescript && bun install --frozen-lockfile && bun run lint && bun run typecheck && bun run test && bun run build
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
binding workflows use a pinned **Bun 1.4.2** for both package management and the
binding's Bun-native tests/API; they share no triggers with the Rust workflows:

- **`ci.yml`** (push/PR) — the gate (`fmt`, `clippy -D warnings` lib+bin, `check`,
  bounded tests with the privileged-netns self-skip boundary, `cargo audit`) plus typed
  workflow/cache/version contracts and a `build-deb` matrix that
  cross-compiles `aarch64-unknown-linux-gnu` (device) and `x86_64-unknown-linux-gnu`
  and packages each `.deb` so a packaging break is caught before any tag. Upstream's
  stable/beta/windows/macOS jobs are kept; under the pin they must call `cargo +<channel>`
  (explicit `+` outranks `rust-toolchain.toml`) to actually exercise that channel. The
  uv contract step uses the published `astral-sh/setup-uv@v8.3.2` release tag because
  setup-uv does not publish a `v8` major alias; do not shorten this ref to `@v8`.
  It also carries the **`bindings` job — the PR-gated TypeScript binding lane**
  (`bindings_release_ref_contract_test.sh` + `bindings_package_manager_contract_test.sh`
  at `working-directory: .`, then `bun install --frozen-lockfile` and
`bun run lint|typecheck|test|build` from `bindings/typescript/`, under **Bun 1.4.2**).
  The two contract scripts live here, not in the Rust `test` job, because both evaluate
  JavaScript and `test` declares no JS runtime — see TS BINDING TOOLING. It is
  **REQUIRED, not a canary**: no `continue-on-error`, so a red binding blocks the PR
  like any Rust lane. `publish-bindings.yml` runs the same commands, but only on a
  `bindings-v*` tag — by then a break is already on `main`; this lane moves the gate
  onto the PR. **This job installs no Node at all**, which is deliberate: the root
  `AGENTS.md` → CI/CD STANDARD's Node 26 baseline exempts binding gates that execute
  under Bun (the same exemption `cerastream` and `srtla` bindings hold), and every
  command here does. Do not re-add `setup-node` to this job. Where Node genuinely is
  required — the `npm pack` tarball guard and the OIDC publish in
  `publish-bindings.yml` — it stays pinned at 26; never pin Node 24 or older.
- **`release.yml`** (tag push `v*`) — runs the full Rust gate plus the blocking
  `loom` contract job (production subscription-concurrency invariant) and Miri lane in
  parallel; `build-deb` needs all three before rebuilding both
  arches inside `debian:bookworm-slim`, enforcing a maximum `GLIBC_2.36` import before
  packaging and attaching both `.deb`s + `.sha256`s to the GitHub release.
  No crates.io publish; no scheduled upstream-sync.
- **`publish-bindings.yml`** (tag push **`bindings-v*`**) — publishes
  `@ceralive/srtla-send` to the **public npm registry** (`@ceralive` scope,
  `registry-url: https://registry.npmjs.org/`) via npm **OIDC trusted publishing**
  (the `publish` job grants `id-token: write`, `npm publish --access public` — **no `NODE_AUTH_TOKEN`**;
  npm is pinned to `11.18.0`, above the trusted-publishing minimum of 11.5.1; Node is 26).
  Mirrors `@ceralive/cerastream`'s publish flow.
  The `test-bindings` job uses the committed `bun.lock` to run lint, typecheck, tests,
  and build under Bun, then runs the tarball guard and uploads validated `dist/`. Node 26
  + npm `11.18.0` remain in that job for the guard alone (it parses `npm pack --dry-run
  --json`, keeping BOTH the npm-11 array and npm-12 object shapes), not to run the gate.
  The OIDC `publish` job
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
The release `.deb` matrix additionally keys on `bookworm`; do not remove that ABI
dimension or it may restore host-built executables that require a newer GLIBC before
Cargo gets a chance to rebuild them.

The CI `test` job runs `uv run scripts/rust_cache_contract_test.py`, which
locks coverage for all nightly, Loom, Miri, `.deb`, stable, beta, Windows, and
macOS Rust lanes in both workflows. The Rust cache action is the only cache
owner for those lanes; the binding lanes are outside its scope entirely and rely on
`oven-sh/setup-bun`'s own dependency cache.

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
Current source package version: `3.3.0`. The workspace `versions.yaml` and the latest
published GitHub release are both pinned at `v3.3.0`.

Rationale: this repo is a fork of `irlserver/srtla_send`; keeping the upstream semver
line in `Cargo.toml` preserves direct traceability to upstream releases. An upstream
version-only commit is still a deliberate fork release decision, not an automatic bump.

The GitHub release **tag** namespace is `v<package-version>`. A tag-triggered package
build must match the committed `Cargo.toml` version; `ci/build-deb.sh` rejects a tag ref
whose `GITHUB_REF_NAME` differs from `v<package-version>`. For this source version, the
only valid release tag is `v3.3.0`.

The `@ceralive/srtla-send` npm binding ships on its own `bindings-vYYYY.M.P` tag
namespace and uses CalVer independently of the Rust crate version.

See `CeraUI/docs/APT_VERSION_CONTROL.md` → "Exception: srtla-send-rs (upstream semver)"
for the full rationale and Debian version-ordering notes.

aarch64 cross-build env (mirrors the PINNED TOOLCHAIN note): linker
`CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc`, apt
`gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross binutils-aarch64-linux-gnu pkg-config`,
`PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig`.

## BENCHMARK METRICS (network-sim)

**Reference-backed scenarios (Todo 12):** `network_sim::scenarios::all()` returns
13 profiles (A–L, B1/B2). `scenarios::Profile` wraps the unchanged temporal `Profile`
in `timeline` with explicit offered/warm-up rates, production SRT settings and optional
source-ramp/restart-budget metadata. Pass `timeline` to the scheduler/metrics, and
the explicit aggregate capacity to `load_intervals::evaluate`. Definitions do not
launch the runner: it must start the SRT source before settling (90% warm-up rate,
three consecutive seconds, 30s deadline/`settle_timeout`), retain ten seconds of
prehistory, apply the 400ms source ramp through live control, and enforce I's 2s
receiver kill/respawn budget. `Profile::validate()` checks FULL periodic expansion.
C/D use 215ms combined spike/half-rate + 285ms half-rate holds, every 15s with 5s
tails and `until=duration−6s`; never overlap SetImpairment holds. D is 75s, not 60s.
H is 90s with a 20s final reorder horizon. K preserves seeded random-walk samples
while retaining TBF enforcement; samples are ungraded zero-tail diagnostics.
L uses feasible warm-up, overload at t=0, ungraded idle, and ONE burst LoadInterval;
do not log intermediate source ramp writes as additional load boundaries. Its
explicit overload/burst targets are 14.4/9Mbit for the 16Mbit bond; a dead sink
must have `reached_ms=None`, `recovered=false`. Catalog and citations:
[`docs/notes/bench-scenarios.md`](docs/notes/bench-scenarios.md). Gate:
`cargo test -p network-sim --lib scenarios` and
`cargo clippy -p network-sim --all-targets -- -D warnings` (include test-module lints).
No hardware validation is implied, and no production sender behavior changes.

`crates/network-sim/src/metrics/` owns the dev-only benchmark collectors and serde
`RunRecord` v1. The authoritative definitions and runner integration contract are
[`docs/notes/bench-scenarios.md#metrics`](docs/notes/bench-scenarios.md#metrics).
The JSON fixture includes BOTH `episodes` and `load_intervals`; its summaries are
recomputed from embedded evidence by tests. Never omit load intervals from reports.

- CSV wire names are `pktRecv`, `pktRecvUnique`, `pktRcvLoss`, `pktRcvDrop`,
  `pktRcvRetrans`, `byteRecv`, and `pktRcvBelated`. Use the SECOND header-position
  `Time` column, not SocketID. `pktReorderDistance` is genuinely absent on host
  libsrt 1.5.5 and must stay optional.
- Default `SrtStats::parse` implements the frozen cumulative-packet/interval-belated
  contract. The later live smoke proved the current non-fullstats listener emits
  interval counters; its runner MUST choose `CaptureSemantics::Interval` explicitly.
  This normalizes packet counters before delta windowing while preserving raw rows.
  No auto-detection, no saturating counter resets, no assumptions about outage cadence.
- Sink and event clocks are aligned explicitly; signed ms retain warm-up. Episodes
  use each originating event's horizon, 100 ms impact/outage detection, and complete
  post-impact one-second recovery buckets. An unfinished, buffered-but-unimpacted
  episode cannot report zero failover. A completed unimpacted episode can.
- Source-load intervals have positive capacity-clamped targets, never an idle
  baseline. Explicit ungraded fault holds/restart gaps are excluded from no-collapse;
  zero-tail restores do not exclude the following healthy interval.
- Optional control metrics skip unsupported candidates. Process-owned future
  adaptive configuration is preserved opaquely as `effective_config`, included in
  the fingerprint, and never guessed from flags. Telemetry health/priority remain
  optional; existing producer JSON is not changed.
- CPU uses proc ticks with explicit CLK_TCK, PID/start-time coherence and VmHWM.
  Link counters retain ifindex generations; periodic sampling cannot recover bytes
  lost with an unsampled destroyed netdev. Final-before-replug sampling is the
  campaign runner's responsibility.
- Gate: `cargo test -p network-sim --lib metrics` and
  `cargo clippy -p network-sim -- -D warnings`. Tests are unprivileged; they include
  real ephemeral UDP/Unix-socket boundaries. No packet capture or new unsafe code.

Profile event/impairment types derive serde for the result's actual event log. Shared
initial-state/channel helpers are crate-visible so restore inference uses the
scheduler's definitions, not a duplicate set of defaults. Scenario C/D waveform
fixtures include both the 215 ms delay spike and full 500 ms capacity dip; whole
property holds cannot overlap, so their combined waveform uses adjacent updates.

### Paired campaign runner

C1 retains all seven arms: five fork modes and upstream `df0b393` classic/enhanced,
across thirteen profiles, production SRT, five runs, seed 20260913 (455 successes
required). Upstream's intentional 4.0.1/fork 3.3.0 version divergence is not an
exclusion criterion. Set upstream `stats_file: false` and omit `effective_config`:
the harness otherwise appends unsupported `--stats-file`, causing argument-parse
exit disguised as a UDP-listener timeout. All five fork test-internals modes expose
the complete adaptive configuration through metrics, even in legacy modes; declare
that observed configuration without enabling adaptive mode or adding env overrides.
`sender-startup.log` preserves captured child output before propagating listener
readiness failure. The readiness timeout and all measurement criteria are unchanged.
The C1 manifest contract tests pin these distinctions; successful startup alone is
not warm-up settling or campaign acceptance.

`tests/bench_scheduler.rs` and `tests/bench_support/` integrate the scenario, topology,
profile and metric APIs. [Campaign contract](docs/notes/bench-campaign.md): required
explicit cells, per-pair seeded candidate interleaving, atomic RunRecord checkpoints,
fingerprint-based stale archival, and bounded retries (default **two total attempts**).
Only matching `status: "ok"` records count; `.failed-N` and `.exhausted` never do.
Records add `attempt`, optional `reason`/`detail` without changing RunRecord v1.
Expected adaptive configuration is explicitly supplied by the manifest and compared
against observed metrics, never synthesized as an observation.

Source traffic starts after all-link REG3 and shaping initialization, at the scenario's
warm-up rate. Settle at ≥90% for three consecutive seconds (30s bound), retain ten
seconds of prehistory, then apply t=0 OfferedRate and run the temporal scheduler.
Smoke forces A+D/two runs/full windows; never truncate D. CSV parsing explicitly uses
`CaptureSemantics::Interval`, never fullstats. The first CSV-row clock bracket and
its uncertainty are recorded; no report-cadence assumption. Ramps use real FIFO source
control; pre-replug counters are sampled; receiver restart has its explicit budget.

The old A/B `measurement_lock` is extracted to `tests/support/measurement_lock.rs`.
Its local mutex is retained and a kernel file lock serializes separate binaries and
worktrees. The file-lock functions have a test-only Clippy MSRV of 1.89 under the
pinned nightly (no production MSRV bump or new FFI/dependency). Each live run is a
GNU-timeout worker (duration+60s, kill-after 10s), with PID-suffixed namespace cleanup.
No pcap unless explicitly enabled; successful captures are removed, failures retained.

Gate: `cargo test --features test-internals --test bench_scheduler` and
`cargo clippy --test bench_scheduler --features test-internals -- -D warnings`.
Live `campaign`/`smoke` are intentionally ignored and require separate privileged
validation; the unit gate does not establish live performance or hardware behavior.

Smoke tooling lives in `scripts/bench/`: `build_candidate.sh` rejects tracked and
untracked dirt, builds release + `test-internals` in the artifact root’s `target-ti`,
and publishes copied, read-only, SHA-addressed binaries without replacement.
`manifests/smoke.json` is a concrete local artifact receipt for classic/enhanced/adaptive,
A/D, CeraLive production, two runs, seed 1. Rebuild and update paths for later revisions.
`assert_smoke.sh` validates all twelve terminal RunRecord outcomes and compares summary
aggregates with the actual successful classic/A and enhanced/A records. CSV packet
deltas live in raw records, not summary v1; all D and adaptive/A outcomes are informational.
Config expectations apply to successful records in all three modes. Shell mutation tests use disposable evidence
copies. See `scripts/bench/README.md` for invocation and actual retry semantics.

### Statistical reporting and D-1 retention

`scripts/bench/report.py` and `scripts/bench/decide.py` are executable, standalone uv
scripts with inline `numpy==2.*`/`pydantic==2.*` dependencies and embedded unittest
gates (`uv run scripts/bench/<script>.py --self-test`). The [operator/schema contract](README.md#statistical-reports-and-retention-decisions)
defines their JSON artifacts. The explicit manifest cells and per-cell run counts
are authoritative: no missing-cell pass, duplicate-success inflation, stale archive
reuse, or campaign/receiver/profile pooling. Reporter failures invalidate the previous
summary; successful publication puts the summary last. Bootstrap is exactly 10,000
paired resamples at seed 20260913, with median-based percentile CIs. Failed episode
durations remain +infinity, represented as `"+inf"`, not silently discarded.

D-1 is fail-closed at 0.95 goodput CI lower / 0.1 pp viewer loss / 1.10 recovery ratio
and no worse non-recovery rate. Any requested N-mismatched cell blocks a verdict;
pair/configuration/episode defects block coverage. Insufficient evidence returns a
nonzero exit with `verdict: null` and **no retirement/default fields**. J/L pass rates
are reported separately; forced outage/idle cannot erase post-restore/burst failures.
Ablation scoring requires each configuration's own A control (C2 lower-CI guard 0.98)
and uses geometric target-median ratios. Target sets are explicit per invocation;
campaign execution and feature/constant publication remain separate tasks. Neither
script changes Rust code or proves that a scheduler should actually be retired.

## ADAPTIVE SELECTION (scheduler evaluation, Todo 22) [PARTIAL]

`SchedulingMode::Adaptive` is additive (`adaptive`, atomic value 4); Enhanced stays
the default and existing encodings/spellings remain unchanged. The send loop owns
`AdaptiveState::new(shared_stats.clone())` beside `EdpfSchedulerState`. Its six-argument
`adaptive::select(conns, last_idx, last_switch_ms, now_ms, cfg, state)` mutates only
admission/election history and quality caches, never health/rate ticks. The live
dispatcher `select_connection_idx_with_state` takes both persistent stores. Test-only
six-argument adapters keep the frozen legacy traces unmodified on that same dispatcher;
the old inline selection tests now live in `selection/tests.rs` without changed assertions.

`AdaptiveState` carries public `features`, `sole_carrier: Option<(usize,u64)>`,
`probe: ProbeScheduler`, and reused `targets: SmallVec<ProbeTarget,4>`, plus private
SharedStats and `(internal conn_id, socket_generation)` sole identity. The internal
random u64 connection ID is NOT telemetry's positional conn_id. Resolve that identity
after reorders; do not restart the incumbent's hold just because its index moved.
`AdaptiveFeatures` is a u8 bitset with STALL/LOSS/QUEUE/DEADLINE/REJOIN/SOLE/PREF/RATECAP,
ALL/NONE, `contains`, `bits`, subtraction and union; Default is ALL.

Eligibility requires connected, not timed out, not Down. Stalled/Degraded are held;
Rejoining is admitted with ramp. Loss ablation uses `HealthMachine::loss_latched()`,
not stale EWMA inference. QUEUE also gates queue delay in the deadline prediction.
The deadline reads the bond atomic directly (unknown →500ms), holds above 0.5L and
releases only continuously below 0.4L for τ. Socket-owned `AdaptiveLinkState` keeps
deadline dwell, failure mark, last election and observed proof; a generation change
invalidates that history on its next observation. `latest_data_proof_ms()->Option<u64>`
distinguishes real DATA proof at clock zero from the first-attempt age anchor.

Rank = existing get_score × cached quality × ramp × Healthy-only preference × soft cap.
Cooldown 15ms / hysteresis 1.10 apply ONLY to an admitted incumbent. Empty admission
elects an eligible sole carrier by ascending measured sRTT (unknown last), holds ≥2000ms,
and switches only for a measured ≤0.5× challenger, hard failure, or expired DATA proof.
After max(2000,τ) without recent proof, mark `sole_failed_until_proof`, force another
eligible link when possible, and rank failed incumbents last with oldest election first.
Only new DATA proof clears the mark. Two unknown RTTs do NOT prove a faster challenger.
No eligible links → highest base among connected, even timed-out/Down links.

`ProbeTarget::eligible` is soft-health OR deadline hold, with hard/sole vetoes.
The packet handler invokes `state.probe.maybe_emit` after selection/forwarding:
only a due probe can force primary flushing; a failure or queued suffix prevents its
copy. Primary and alternate flush errors both return the failed internal conn_id for
normal recovery + SequenceTracker removal. Probes never alter switch history or probe
the elected carrier. **`stall_deselect` is a strict no-op in adaptive**, and adaptive
ACK attribution uses the existing generation-fenced arrival-scoped policy regardless
of the legacy earned-ACK flag.

Todos 23/24 are integrated: health/rate housekeeping, registration/recovery resets,
lifecycle status, and the adaptive control surface now run in production. Health
still initializes Down and enters Rejoining after registration. Todo 25's connector
refreshes actual admission/ranking before the initial and every housekeeping stats
snapshot, even without DATA. No default, wire format, frozen trace, or hardware
performance claim changes here. Tests: `cargo test --lib adaptive` and the real-binary
`cargo test --test adaptive_cli`, including the ALL−STALL scenario-D control.

## PRIORITY PLUMBING (scheduler evaluation, Todo 21)

### Wave-4 shared foundation

- **Todo 23 seam:** `HealthSignals.route_health: connection::route::RouteHealth`
  uses `Unknown | DefaultRoutePresent | NoDefaultRoute`, not a parallel enum or
  synthetic loss/queue sample. `HealthMachine::route_latched()` is an independent
  cause: NoDefaultRoute latches it, DefaultRoutePresent clears it, Unknown retains
  it. A latched route blocks Degraded clearance; after restoration all causes must
  clear continuously for the existing τ. Hard Down and stall precedence are intact.
  `HealthConstants` is re-exported from `health_constants.rs` without API/default
  changes. This foundation does NOT add housekeeping health ticks.
- **Todo 25 seam:** `TelemetryConn` and `LinkStats` gain optional `health` and
  `priority`. The document orders them after `link_id`, omitting None entirely;
  `TelemetryConn` retains PartialEq but cannot derive Eq with raw optional f64.
  `LinkStats.effective_multiplier` now reads the socket-scoped selection cache in
  adaptive mode; legacy modes retain `quality_multiplier`. The single
  `selection/adaptive/ranking.rs::refresh` pass owns admission, deadline hysteresis,
  election, and the quality × ramp × preference × soft-cap product. Both packet
  selection and `AdaptiveState::update_stats` consume that pass; stats never
  reconstructs admission. The cache pairs the base score with its multiplier, is
  cleared on socket reset, and follows the connection through reorder. Held links
  publish zero; a sole carrier retains the product, while the last-resort connected
  fallback retains its base-only rank. Adaptive zero-total snapshots stay zero,
  never using the legacy equal-share fallback. Percentages describe ranking shares,
  not packet counters or a one-hot representation of cooldown/hysteresis decisions.
  Health is now populated only in adaptive mode (`healthy|degraded|stalled|rejoining|down`);
  priority echoes `effective_priority()` whenever configured, including in legacy
  modes where it has no scheduling effect. Absent values remain omitted. Schema
  stays 1. `telemetry-adaptive` adds a mapped healthy +0.2 carrier and zero-weight
  stalled neighbour without priority; all eight older fixture pairs are byte-unchanged.
  TS declares both fields after `link_id` and byte-roundtrips the new fixture.
  The fully populated record alone has the exact 12-key assertion; its neighbour
  uses subset checks. Old golden key equality/additivity remain frozen. A stripped
  health control falsifies byte parity. `tests/telemetry_adaptive.rs` also drives the
  real binary over two loopback uplinks with DATA ACKs withheld on one while
  keepalives continue, requiring that link's live stats-file weight to reach zero.
  That test executes the exact Cargo artifact through a same-filesystem hard-link
  alias `atel-<test-pid>` in a private target-directory temp folder. Never launch it
  with the production process name: concurrent host control-plane tests can issue
  `killall srtla_send` and terminate an unrelated test child. Linux checks the actual
  kernel process name. Child logs accompany exit/deadline failures; the 15s deadline
  and telemetry assertions are unchanged, with no spawn/test retries or signal masks.
- **Todo 24 seam:** `SharedStats::pool_control() -> Option<PoolControlHandle>`
  reaches `sender::pool_control`. `submit(PoolControlRequest::SetLinkPriority {
  key: LinkKey::LinkId(LinkId) | LinkKey::ConnId(usize), priority: Option<Priority>
  })` is synchronous/nonblocking and returns a Tokio oneshot receiver of
  `PoolControlResult`. The sender owns/drains the bounded 64-message mpsc channel
  in its cross-platform event loop. **ConnId is the telemetry position, never the
  internal random connection ID.** Application changes only the addressed override,
  calls the existing clear methods for None, and replies AFTER mutation with
  `{applied, key, link_id, conn_id, priority, effective_priority}` (typed priorities).
  Errors: `UnknownLink(LinkKey)`, `Busy`, `Unavailable`, `PoolReloaded`.
  The RPC adapter must parse boundary types, bound its reply wait, and never report
  enqueue success as `applied`. Do not block an async runtime on `blocking_recv`.
- **Reload ordering:** close/reject the old channel BEFORE `apply_link_changes`'s
  first await; publish a fresh handle afterward. Queued requests of either key kind
  get PoolReloaded, all retained old handles stay closed, and requests during reload
  fail Unavailable. This conservatively fences positional writes even on unchanged
  order. Existing baseline/link/conn reload semantics are unchanged. Dropping a
  reply cancels queued work observed before application; racing cancellation after
  application cannot roll it back. Sender teardown disconnects waiting replies.
- Tests: `health::route_tests` proves route-only and mixed-cause clearance;
  `telemetry_doc::optional_tests` pins order/omission; `sender::pool_control_tests`
  mutates actual connections, and `pool_control_runtime_tests` drives the real sender
  loop using initial snapshot readiness. JSON-RPC, fixture expansion, live health
  ticks, and scheduler-derived telemetry remain the later lanes' work.

`bind_map::Priority` is a private-field `f64` newtype, constructed by
`TryFrom<f64>` only for finite −0.20..=+0.20; `get()` exposes the numeric bias.
Raw sidecar rows use `#[serde(default)] Option<f64>`; after hash coherence,
`validate::parse_rows` maps conversion failure into the existing
`BindMapError::InvalidRow { index, field: "priority", detail }`. No new degraded
reason, retry rule, partial acceptance, or schema-version change. The validated
`Option<Priority>` flows through `BindMapRow`, `EffectiveLink`, and `UplinkSpec`.

`SrtlaConnection` owns THREE independent `Option<Priority>` fields:
`priority_baseline`, `priority_override_link`, `priority_override_conn`.
`effective_priority()` reads conn > link > baseline. The methods
`clear_priority_override_conn()` and `clear_priority_override_link()` clear ONLY
their own layer; a conn clear must expose a remaining link override, not jump to
baseline. `connect` initializes baseline from the spec and both overrides to None;
`spec()` projects the baseline, never an effective override back into a sidecar value.

The pool rebuild refreshes survivors' baseline (including None) and clears only
their conn layer, keeping the persistent link layer attached to identity through
reorders. A fresh socket under the same `link_id` copies the old link override before
retiring the old connection; transport history is still discarded. Ordinary
recovery/reconnect resets do not clear priority fields. Runtime override commands
remain Todo 24; do not merge the two override slots when implementing them.

`sender::preference_multiplier(priority, window, health)` (implemented in
`sender::selection::adaptive::preference`) is
pure, allocation-free and called only by adaptive ranking.
Healthy uses `1 + p * clamp((window−10000)/10000, 0, 1)`; all other health states
return 1.0. Convert the signed window before subtraction to avoid integer overflow.
No CLI, telemetry, capabilities, ADR document, or legacy-byte contract edits here.
Tests remain in the existing bind-map parse/validate and link-identity homes, plus
the pure multiplier's local table. README carries the matching integration boundary.

## PURE LINK HEALTH (scheduler evaluation, Todo 15)

`connection::health` is a production-compiled, allocation-free policy
module. Todo 15 added only `pub mod health;` to the connection implementation;
Todo 16 separately adds the DATA evidence ledger below, not runtime health calls,
scheduling modes, flags or telemetry. Policy tests are test-only companion modules in
`src/tests/health_*_tests.rs`, loaded
by `health.rs` without changing `src/lib.rs` or its existing `not(loom)` gate.

API: `HealthState::{Healthy, Degraded, Stalled, Rejoining, Down}` with lowercase
`as_str()`, `HealthSignals`, `HealthConstants`, `HealthMachine::new(state, now_ms)`,
`step(&signals, &constants) -> Option<Transition { from, to, at_ms }>`, and
`ramp_multiplier(now_ms, held_links)`. State/timestamps/dwell have read-only getters;
`step` delegates to `step_with_originals`, the production transition entry point;
the latter owns transitions and maintains dwell in 1..=16.

Defaults: stall_attempts=32; τ=clamp(4×sRTT,1000,3000) ms, unknown=3000;
loss_enter=0.10, loss_clear=0.05, loss_cohort_min_sends=100,
loss_stale_after_ms=10_000; queue enter=max(10,0.25×slow_min),
clear=max(5,0.125×slow_min) ms; rejoin_rounds=2; probe_train_len=10;
probe_max_pps=10 bond-wide; dwell_backoff_max=16. Formula-valued defaults are public
methods `stall_tau`, `queue_enter`, `queue_clear`; `train_period_ms` and `rejoin_span`
expose the cadence calculations to future probe consumers.

**Round-8 keepalive detector (2026-09-15), uncommitted experiment / acceptance FAILED.**
`HealthSignals.keepalive_silence_ms` comes from the socket-scoped
`connection::keepalive::KeepaliveLiveness`, not RTT's intermittently armed waiting
flag. The original attempts≥32 AND DATA-proof-age≥τ condition remains; an OR arm
adds control silence≥3×IDLE_TIME (3000ms) with DATA proof unknown or ≥τ old.
Healthy controls never veto DATA-stall detection; fresh DATA still proves liveness.
Unknown control age means no accepted keepalive send, not expiry. First accepted
send anchors the no-reply deadline; subsequent sends cannot postpone it. Pending
timestamps use a fixed 16-slot ring, matched timestamped replies are one-shot and
monotonic, and bare two-byte replies prove control liveness only. Zero-ms replies
do not change the existing RTT rejection rule. Core recovery clears this evidence.
No keepalive-only Stalled→Rejoining bypass is added: recovery still needs DATA.
Keepalive sends remain independent of selection, checked every housekeeping tick;
actual intervals include tick/I/O scheduling, not a hard real-time guarantee.

This does **not** resolve D: its classifier deliberately passes small controls while
dropping DATA. Ten unchanged sole-ON release trials give3/10 timely Stalled/0,
versus fresh baseline2/10 and historical4/5; full D0/10. All nine logged Stalled
transitions meet the original DATA condition. No substantial detector improvement
is established, so admission/ranking/sole remain untouched and conditional A was
not run. Dedicated G fails3/3; release Twins passes3/3, while both broader feature
gates and the netns script fail G/Twins. No full-green or C1/Todo31 completion claim.
Nine new tests include two behavioral failures before implementation, healthy idle
controls, replay/interop/reset boundaries and preserved DATA-only failure handling.
Details and per-run timings: [`docs/notes/adaptive-keepalive-round8.md`](docs/notes/adaptive-keepalive-round8.md).

**Queue-entry persistence (Todo 28 round 7):** `step_with_originals` starts one
private `queue_entry_pending: Option<QueueEntryPending>` episode on the first
Healthy/Rejoining evaluation at or above `queue_enter`. It latches `since_ms` and
`required_ms=τ`; later RTT changes do not move that deadline. Queue degradation
requires continuous qualifying evidence for the latched duration. Below-entry
observations and entry into Down/Stalled/Degraded clear the episode; the
Rejoining→Healthy edge MUST preserve it if queue remains high. Hard failure wins,
then stall, then immediate route/finalized-loss degradation, then matured queue.
Starting the timer does not double dwell; only an actual soft rejoin relapse does.
The Degraded `queue_clear`/`clear_since_ms` path is UNCHANGED and separate storage.

This deliberately absorbs short full-load restoration transients from a source
unaware of the rejoin ramp. It does not reinterpret round 6's proven kernel backlog
as measurement noise, widen queue thresholds, or change HealthSignals/HealthConstants.
Never use `ramp_span_effective` or `dwell_multiplier` for entry persistence: sustained
evidence must mature in 1–3s from the first qualifying evaluation (design budget
roughly 5s from physical onset including detector/housekeeping quantization, not
a hard real-time guarantee). Remaining overload is an admission/sole-carrier
coordination investigation, not permission to extend this window.

The first round-7 live gate remains RED: I passes, D/G/twins fail unchanged
assertions. G carries 3,727,772B but is demoted in 48/61 distinct snapshots
(36Degraded/12Stalled). Twins' retained survivor and rejoining-link demotions both
say **normal DATA loss**, not queue; no causal admission/qdisc capture was added
this round. D/G transition causes are unretained and must not be guessed from NAK
warnings. The gate is unit-proven (seven new tests), not a live-convergence fix.

Integration obligations:
- Pass normalized finite/nonnegative timing observations and loss fractions in [0,1]
  in one monotonic-ms domain. `proof_age_ms=None` means unknown, not infinite;
  before first DATA proof pass elapsed-since-registration/first-attempt instead.
- `loss_cohort_ok` means a qualifying normal-DATA cohort, never a probe cohort.
  `last_cohort_ms: Option<u64>` dates the last qualifying EWMA. Fresh retained EWMA
  can clear loss even if this tick's cohort is below the floor. Stale (age ≥10s),
  absent, or undated EWMA requires `probe_loss`; supply it only after the last two
  complete trains. Loss demotion latches its cause; queue-only demotion with unknown
  loss does not invent a loss requirement. Loss, queue, or the independent route
  cause can reset clear dwell.
- `probe_rounds_started_ms: Option<u64>` is the start of the oldest train counted
  by `probe_rounds_ok`; expire the pair together. This additional timestamp is
  necessary to enforce the plan's freshness requirement from a pure snapshot.
  Stalled accepts two rounds only within max(2τ,rounds×train_period+sRTT), inclusive,
  with a start not preceding this Stalled entry or lying in the future. This is a
  rolling evidence window, NOT a deadline measured from initial failure.
- Fresh runtime machines start Down; restored eligibility takes Down→Rejoining.
  Explicit Degraded initialization conservatively requires loss clearance; normal
  transitions record the actual loss cause. Direct Rejoining initialization uses
  default unknown-RTT timing; runtime rejoin captures the supplied entry RTT/tuning.
- Ramp multiplier and Healthy promotion share one effective-span calculation:
  max(2τ,train_period)×dwell. Entry RTT/tuning stay fixed during a ramp, held count
  stays live. Rejoining→Stalled/Degraded doubles dwell, never Down; Healthy resets
  it. Do not substitute a shorter promotion timer after a relapse.

Run `cargo test --lib health`, `cargo fmt --all -- --check`, and the real lib+bin
`cargo clippy -- -D warnings`. This is policy coverage only; later delivery/probe
tracking and housekeeping integration must establish the live behavior separately.

## DATA DELIVERY LEDGER (scheduler evaluation, Todo 16)

`SrtlaConnection::delivery` is `pub(crate)` and independent of `packet_log`.
`connection::delivery::DeliveryLedger` owns a preallocated 4096-entry sequence map
with O(1) linked LRU removal, 6000ms expiry (exactly 6000ms remains valid), saturating
`attempts_since_proof: u32`, `last_data_proof_ms: u64`, a first-attempt/proof-age anchor,
a two-second delivered-byte ring and `socket_generation: u32`. Retransmitting a
sequence refreshes its timestamp/length/LRU position and adds one attempt; a hit is
consumed once. Eviction never resets attempts or moves the proof-age anchor.

`FlushOutcome.accepted` is now `SmallVec<(Option<i32>, u64, usize), 4>`:
sequence, existing QUEUE timestamp, actual accepted datagram length. The touched
`flush_batch` implementation was extracted from the oversized connection façade to
`connection/transmit.rs`; its method name and callers are unchanged. It registers
the accepted prefix in the legacy log using queue timestamps, and separately records
`DataSend { sent_ms: acceptance_now_ms, len: checked_u16_length }` in the ledger.
This also runs on a prefix followed by a hard error. Unsent suffixes, queue-only
packets and sequence-less control packets do not enter the ledger. No telemetry
accounting call was moved.

`handle_srtla_ack_specific` consults the ledger FIRST. A valid `DeliveryAck { seq,
socket_generation }` hit stamps proof, resets attempts and credits its length even
if a cumulative ACK or NAK already pruned `packet_log`. Its existing bool return,
RTT sampling, legacy stamp and window growth remain packet-log-dependent. Never
make these legacy effects depend on the new ledger, or require a packet-log hit
for DATA proof. Keepalive, cumulative ACK and NAK never stamp DATA proof. Duplicate
probe hits must use the probe log rather than inserting copies into this ledger.

`proof_age_ms(now_ms)` returns elapsed since first accepted attempt before first
proof, then elapsed since proof; a fresh/reset idle ledger returns `None`. Zero is
a valid clock timestamp, so absence is tracked separately from the public zero
stamp. `delivered_bps(now_ms)` is a fixed 2s wire-bits/s signal over `(now−2000, now]`,
not telemetry's queued-send rate or useful viewer goodput. Equal-ms credits aggregate,
so the retained ring is bounded by 2000 buckets without losing high-PPS samples.

`reset_core_state` clears all evidence and wrapping-increments generation for both
`mark_for_recovery` and successful socket replacement. Ledger lookups check BOTH the
caller token and entry generation. An old token cannot consume a reused current
sequence. Todo 19 propagates captured reader generations through `UplinkPacket`
and fences them in the adaptive ACK policy. The legacy sequence-only handler still
supplies the current generation intentionally. SRTLA wire ACKs carry no generation:
local reader fencing is not wire authentication.

Tests: `cargo test --lib health_delivery` includes the seven required named cases,
the permanent `legacy_stall_predicate_misses_scenario_d` control, expiry/LRU/rate/token
boundaries, and real kernel prefix/partial-error sends. Scenario D uses 40 accepted
sends, five real keepalive-handler replies at 10–14s, and three NAK frames removing
40 packet-log entries. It remains Stalled with 40 attempts and zero in-flight packets.
`utils::test_clock` is a cfg(test)-only, scoped, thread-local !Send override for these
current-thread tests; production/test-internals-only builds retain the real monotonic
clock. No sleeps or global clock replacement; existing frozen flag tests are unchanged.
HealthMachine housekeeping wiring and hardware validation remain later work; adaptive
selection/probe integration is described above.

## LOSS / QUEUE EVIDENCE (scheduler evaluation, Todo 17)

`connection::loss::LossTracker` owns normal-DATA loss independently of legacy
congestion counters. `new(now_ms)` anchors 1000ms `[start, end)` cohorts;
`record_send(now_ms)` runs beside the DeliveryLedger insertion in
`connection/transmit.rs::flush_batch`, inside the accepted-prefix `Some(seq)` arm.
Queue-only packets, failed suffixes, sequence-less control and direct probe copies
never count. `record_data_nak_for_send(sent_ms, now_ms)` runs only after `handle_nak` successfully
removes a normal `packet_log` entry; duplicate/missing NAKs do not count. Keep
probe-log lookup outside this branch. Existing NAK/window effects stay intact.

`advance(now_ms)` closes elapsed cohorts before counting a boundary event. It feeds
`cohort_naks / cohort_sends` to the existing alpha-0.2 `Ewma` only at
`LOSS_COHORT_MIN_SENDS = 100` or more. Below-floor cohorts are discarded, not pooled.
`last_value() -> Option<f64>` and `last_cohort_ms() -> Option<u64>` retain the
last qualifying estimate and its cohort END, even across long idle gaps. NAKs use the
accepted-send timestamp from the delivery ledger (congestion-log timestamp fallback),
never arrival time to select a denominator. A bounded ten-second closed-cohort history
corrects older cohorts and recomputes EWMA chronologically from an evicted-prefix
checkpoint. This does not redetermine the load floor, pool sub-floor cohorts, refresh
old evidence dates, or clamp loss ratios. Pre-epoch/expired feedback is never charged
to new traffic. `record_data_nak(now_ms)` remains a same-cohort convenience API.
**Close and settle are distinct.** Closure freezes the accepted-send denominator;
the closed cohort cannot feed EWMA until the latest of its accepted-send deadlines
has elapsed. Adaptive admission supplies the SAME `negotiated_latency_ms` observation
as its deadline gate (one shared 500ms unknown-fallback constant). Each accepted send
captures an epoch/cohort/deadline `LossSend` token: deadline = acceptance time + that
budget. A changed budget or a retry never extends an older debit's deadline.
For adaptive normal ACKs, `LossDebit`s in the bounded delivery entry can be credited
exactly once in either the open or closed-but-unsettled cohort, before EACH debit's
deadline. Retries retain distinct cohort/deadline groups; only identical groups merge.
At/after deadline, unresolved NAKs (including a first late observation) are final and
cannot be forgiven. Probe/cross-link/stale-generation/old-epoch ACKs and missing
ledger entries cannot receive credit. Socket replacement resets both ledgers.
Expired debit groups are pruned on retry/NAK; the delivery ledger's 4096-entry/6s
bound and ten-second cohort history still apply. An unsettled cohort expiring from
history is discarded, never prematurely finalized for an exceptionally large budget.
`last_settlement_ms` controls the one-second fresh-settlement qualification, while
`last_cohort_ms` remains the ORIGINAL cohort end for the ten-second stale rule.
Late final NAK revisions recompute retained EWMA without refreshing either date.
Raw congestion NAK/window effects, legacy ACK behaviour, and ALL health transition
thresholds/dwell logic remain unchanged. This is deadline-budget accounting, not a
receiver playback timestamp measurement or blanket eventual-delivery forgiveness.
The health sampler must call `advance(now)` even for idle links, then read
`loss_cohort_ok(now, stale_after_ms)` (qualified fresh settlement, original evidence not stale)
and `is_stale(now, stale_after_ms)` (unknown or age >= threshold). Discarding a
cohort disables qualification without deleting a retained clearance estimate.
Recovery/socket replacement reconstruct loss state at `reset_core_state`.
`probe_loss() -> Option<f64>` reads the default-None `pub(crate)` field populated by
Todo 19's `advance_probes(now_ms)` from the last two finished probe trains.

`RttTracker` adds separate `rtt_obs_fast` / `rtt_obs_slow` time windows over raw RTT
observations. `queue_delay_ms() -> f64` is `max(0, (fast_min - slow_min) / 2)`
over `(now-1000, now]` / `(now-30000, now]`; no fast evidence returns 0.
`slow_min_rtt_ms() -> f64` exposes the time-based floor (0 if absent) for future
queue thresholds. Expiry is checked on reads as well as inserts. The child
`rtt/queue_delay.rs` keeps monotonic minimum candidates, coalescing same-ms samples;
at most one candidate per millisecond bounds retained entries without sample-count
eviction. Reset clears both new windows. **The existing filtered sample-count
windows, `rtt_min_ms` computation, and in-file RTT tests are byte-unchanged.**
Do not substitute these time windows for the baseline consumed by BLEST/legacy EDPF.

Run `cargo test --lib loss`, `cargo test --lib queue_delay`, and the unchanged
legacy `cargo test --lib rtt` cases. Tests cover the 50-send/50-NAK guard, EWMA
convergence, expiry, raw RTT ramp/jitter, real accepted-prefix errors, attribution,
and lifecycle resets. No HealthMachine/HealthSignals runtime wiring, telemetry,
CLI, probe production or scheduler-selection change is part of this tracker work.

## NEGOTIATED SRT LATENCY (scheduler evaluation, Todo 18)

`protocol::srt_handshake::parse_hsrsp_tsbpd_delay_ms(&[u8]) -> Option<u32>` is a
clean-room, allocation-free passive decoder using the real committed 80-byte
`tests/fixtures/srt-hsrsp-latency2000.bin`. Header checks: type `[0,2)` = 0x8000,
version `[16,20)` = 5, extension flags `[22,24)` has HS bit 1, conclusion request
`[36,40)` = 0xffffffff. Extensions start at 64; each advances `4+4*body_words`.
Command 2 HSRSP requires at least three complete body words; receiver delay is the
big-endian high half of the latency word at descriptor offset E+12 (not the low
sender half). All descriptors/bodies must fit, including trailing extensions.
Absent HSRSP, other types/versions/phases, and truncation silently return `None`.

`process_packet_internal` sniffs ONLY the existing forwarding else-branch when
type is `SRT_TYPE_HANDSHAKE`, then executes the original byte-copy push regardless
of parse success. Both `process_packet` and `drain_incoming` borrow `&SharedStats`;
the sender passes the same bond handle through every uplink/queue-drain entry,
including SIGHUP. Do not attach this bond-wide state to individual connection
lifecycles or alter registration, liveness, forwarding, or source acceptance.

`SharedStats` owns a new private `Arc<AtomicU32>` with relaxed loads/stores: the
single word is the whole observation and publishes no associated memory. Zero is
unknown; the public `negotiated_latency_ms() -> Option<u32>` reads directly without
any snapshot/configuration lock. Its crate-private setter is called on successful
decode only. Housekeeping/reload/reconnect retain the last observation; a later
valid handshake replaces it (including zero → unknown). There is no authentication,
stream-identity binding or generation/freshness fencing: Todo 22 must not invent
those guarantees. No adaptive deadline gate is enabled by this plumbing alone.

`get-status` adds optional `negotiated_latency_ms`, omitted for unknown (not null).
The 30-second status log prints the value or `None`. `StatsSnapshot`, ADR-001 file
telemetry, CLI and TS bindings remain unchanged. Stats tests were extracted into
`src/stats_tests.rs`; module/test names retain the `stats::tests` path.
Gates: `cargo test --lib srt_handshake` (real fixture + byte-flip/truncation controls),
`cargo test --lib packet_io` (byte-preserving receive paths), atomic/status tests,
`cargo test --test parser_proptest` (arbitrary bytes and captured-header extensions),
and `cargo clippy -- -D warnings`.

`tests/negotiated_latency.rs` runs the actual binary without privileges, completing
registration with a loopback UDP test peer, checking the returned datagram verbatim
and querying the live Unix `get-status`. Both captured-HSRSP and flipped-type cases
are bounded to ten seconds; temporary paths and ports are per test. This exercises
production handle propagation, not a second live-libsrt capture or hardware gate.

## DUPLICATE DATA PROBES (scheduler evaluation, Todo 19)

`connection::probe::ProbeScheduler` is owned per adaptive send loop, never global.
`next(targets, now_ms)` uses a one-token bucket capped at `PROBE_MAX_PPS=10`:
no idle burst, `with_rate(0)` disables emission, overrides cannot exceed the cap.
Ten-slot trains (`PROBE_TRAIN_LEN=10`) rotate by stable connection ID; a generation
change or eligibility loss abandons the current train. `ProbeTarget` explicitly
carries `{conn_id, socket_generation, health, deadline_held, sole_carrier, srtt_ms}`.
Stalled/Degraded OR deadline-held targets are eligible, never Down or sole carrier.
Adaptive calls `maybe_emit(connections, ProbeOpportunity { primary_conn_id,
packet, targets })`, which establishes primary acceptance before invoking `emit`.
Skipped opportunities
consume pacing slots conservatively; incomplete trains never qualify for recovery.

Production emission reuses the spike's wire-copy primitive in
`sender/duplicate_data.rs`, clearing **SRT byte 4 mask 0x04** and changing no other
byte. `queue_probe_packet` inserts a distinct probe kind into the normal unpadded
BatchSender. It rejects stale dispatches and same-link original/probe sequence
collisions. Queued probes do not enter `queued_count()`'s congestion load. Normal
`FlushOutcome.accepted` stays unchanged; new `probes` metadata reports only the
kernel-accepted probe prefix. Probe sends never call `register_packet`, insert
into DeliveryLedger/SequenceTracker, or modify selector switch history. The emitter
refuses a queued suffix and rebases pacing after I/O to avoid delayed-send bursts.
`ProbeEmission { conn_id, outcome }` identifies the link the caller must recover
and remove from SequenceTracker on an error, exactly as for normal flush errors.

Each connection owns `probes: ProbeLog`, containing `probe_log: FxHashMap<i32,u64>`
(sequence → acceptance ms), a bounded 256-entry LRU, and at most 32 train records
with inline ten-sequence lists. A train's inclusive ACK deadline is its start plus
`2000*m + srtt_ms`, with m captured from eligible held links. Only ten distinct
accepted sequences with at least five ACKs qualify as OK; replays cannot re-credit
a train. Expired trains fail when below that threshold. Loss is missing slots /20
over the last two finished (all-ACKed or expired) trains; incomplete trains count
their missing slots as losses. Dividing the integer loss count avoids rounding
5% just above the clearance threshold. `rounds_ok()` returns the consecutive
count and oldest contributing start. History expires after twice its train timeout;
HealthSignals must still enforce its own rejoin-span age bound. Todo 23 must call
`advance_probes(now)` on idle links before reading loss/rounds. Recovery and socket
replacement clear logs/trains with the delivery generation; `probes_sent` remains
cumulative for the connection lifetime.

`UplinkPacket` is now `{conn_id, reader_generation: u32, bytes}`; both reader send
sites capture the generation at spawn, and sync/restart pass it explicitly.
`packet_handler::apply_srtla_ack` re-exports the implementation in `sender/ack.rs`:
`AckContext { arrival_idx, reader_generation, policy: AckPolicy }`. Adaptive first
rejects a stale token, then consumes ONLY the arrival link's probe log or original
delivery ledger. Probe proof refreshes DATA health but never original delivered
bitrate, window, packet_log or in-flight. No cross-link scan exists in this arm.
`AckPolicy::from_config` maps the four established modes to the unchanged legacy
first-match/global-growth arm and Adaptive to the arrival-scoped arm.
Test-only adapters preserve the frozen ACK-RTT, batch-I/O and earned-ACK suites
byte-for-byte while calling the same production implementations.

Probe-only NAKs never enter normal loss accounting. Accepted probe bytes DO update
BitrateTracker's total and rate, including an accepted prefix before a flush error
(ADR-002); unsent probe suffixes do not. `probes_sent` appears only as a structured
field in the existing 30s status log when nonzero. No telemetry JSON keys, schema
version, runtime flags, selection policy or hardware-performance claim are added.
Gate: `cargo test --lib probe`; separate `cargo test --lib ack_rtt` and
`cargo test --lib batch_io`; `cargo clippy -- -D warnings`.

## PURE DELIVERED-RATE CONTROLLER (scheduler evaluation, Todo 20)

`connection::rate_cap::{RateCap, RateState, ClimbMode, RateSignals}` is a standalone
policy module. Adaptive now owns a connection field and reads its ranking multiplier;
housekeeping ticks/resets and telemetry remain pending. `connection/congestion/enhanced.rs` is untouched.
`RateCap::default()` belongs to one socket lifetime. The later lifecycle owner must
reconstruct it alongside that link's delivery ledger on recovery/socket replacement.

`tick(&mut self, &DeliveryLedger, &RateSignals)` MUST run exactly once per 1s
housekeeping tick, even when idle, never per packet or in a catch-up loop. It reads
`delivery.delivered_bps(signals.now_ms)` internally: callers cannot substitute TX
bitrate. Signals supply monotonic ledger time, finite nonnegative `srtt_ms`,
`rtt_min_ms`, `queue_delay_ms`, `jitter_ms`, finite `velocity_ms_per_update`, and
`loss_ewma: Option<f64>` (fraction, unknown is not zero). Integration should use
the same link's RTT/queue trackers and advanced normal-loss tracker, not probe loss.

States are exactly `Bootstrap | Climbing { sub } | Holding | BackingOff | Drain`;
submodes are `Normal | Hai | FastRecovery { ticks_left }`. State/target/rate fields
are private and exposed via read-only `state()`, `target_bps()`, `delivered_bps()`.
The first positive observation seeds `max(1_000_000, delivered)` and enters normal
Climbing without applying a growth increment on the same tick. Idle before that
preserves Bootstrap, so the first later observation still seeds correctly.

- Normal grows 2%; Hai grows 6% only with measured RTT, absolute velocity ≤0.1
  ms/Kalman update, jitter ≤10% sRTT, and zero queue delay. The explicit 0.1
  tolerance and queue veto resolve the plan's unspecified near-zero criterion.
- BackingOff needs loss ≥0.015 AND delivered ≥0.3×target; the next target is exactly
  `max(0.85*prev, min(delivered, prev))`. The first observation AFTER three completed
  cuts tests efficacy; loss ≥0.8×entry starts a 30-tick cut-suppression latch, including
  that observation tick. Strictly lower loss permits continued backoff. Efficacy is
  tested once per uninterrupted backoff episode, not repeatedly against moving entry
  loss. `is_uncongestive()` exposes the latch; it ages through idle as well.
- Measured sRTT >1.5×baseline holds. At ≥2×baseline with `Some(0.0)` loss, Drain
  applies ×0.75 only at episode entry AND with no cut in the preceding ten ticks.
  Episode identity is separate from the guard: expiry cannot recut sustained Drain.
  A guarded/suppressed episode is not cut later merely because the timer expires.
- Backoff/Drain arm five recovery growth ticks. They survive Holding and idle.
  Mode selection occurs before decrement: reported `ticks_left` is 4,3,2,1,0 after
  the five full ×1.04 ticks. Only the next climbing tick can choose Normal/Hai.
- Idle freezes target, active mode, Drain identity and recovery budget; it clears
  incomplete backoff efficacy evidence. `stale_since` counts the first idle tick
  inclusively, so `bdp_cap_suspended()` becomes true on tick ten, stays true through
  indefinite idle, and clears on the next positive delivery without rebootstrap.

`bdp_cap_packets(rtt_min_ms) -> u32` returns
`max(32, floor(target_bps*rtt_min_ms/1000/8*1.5/1316))`, or `u32::MAX` when suspended.
Packet conversion intentionally saturates overflow; growth saturates at finite f64
maximum. `soft_cap_multiplier(in_flight: u32)` uses the last tick's cached baseline:
1.0 at/below cap (including suspension), otherwise `cap/in_flight`, always positive.
It is RANKING ONLY, never an eligibility predicate. There are no share-tier verdicts.

Tests are split into `rate_cap_tests.rs` (growth/ledger/ranking),
`rate_cap_backoff_tests.rs` (loss/latch) and `rate_cap_episode_tests.rs` (timing/idle).
They populate a real DeliveryLedger with SRTLA ACK credits, not a mock send rate.
The five audit-named regressions include an exhaustive wildcard-free match over
both enums; adding any new verdict fails compilation. Gate:
`cargo test --lib rate_cap` and `cargo clippy -- -D warnings`. This is pure policy
coverage; no live-bond performance or runtime integration claim is implied.

## CODEBASE (inherited from upstream)

```
src/
  main.rs            CLI entry point (clap)
  lib.rs             library exports
  config.rs / config/    runtime config (DynamicConfig, ConfigSnapshot); stdin + Unix-socket control
  mode.rs            SchedulingMode (Classic | Enhanced | RttThreshold | Edpf | Adaptive)
  bind_map/          optional versioned bind-map sidecar (ADR-003): parser, coherence,
                     bounded retry, fail-open duplicate-safe resolution
    report.rs        telemetry projection of a Resolution (bind_map_status + disposition)
  capabilities.rs    --capabilities-json pre-spawn probe document
  telemetry_doc.rs   ADR-001 document model + units + serializer (schema lives here)
  telemetry_file.rs  opt-in --stats-file publish mechanics (temp -> fsync -> rename)
  connection/        SrtlaConnection, bind/resolve, incoming packet handling, RTT (Kalman)
    socket.rs        SourceIpBinder (legacy) + DeviceBinder (SO_BINDTODEVICE + source bind)
    spec.rs          UplinkSpec/SocketKey — link_id identity vs (ip, iface) socket key
    egress.rs        ifindex staleness: re-resolve, re-enumeration, ENODEV -> removed
    route.rs         read-only per-iface default-route observation (blackhole check)
  protocol.rs        SRTLA protocol constants/structures
  registration.rs    REG1/REG2/REG3 flow + ID propagation
  sender/            packet forwarding + selection/ (BLEST → IoDS → EDPF), status logging
    links.rs         where the uplink set comes from (legacy ips file, or the bind-map pair)
    connections.rs   pool rebuild: dedup on socket key, survive on link_id
    egress_tick.rs   the per-tick egress re-resolution + route observation
  tests/             unit / integration / e2e / protocol / registration suites
crates/network-sim/  dev-only network simulation harness (workspace member)
  twin/            duplicate-IP twin topology (NAT carrier per link), bind-map
                   sidecar publisher, and the twin process stack
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
scripts/bindings_package_manager_contract_test.sh  bun-only package-manager policy contract
scripts/netns_test_gate.sh  bounded privileged network-namespace test runner
```

Conventions (enforced by the gate): edition 2024, `anyhow::Result`, `tracing` macros,
Tokio async, imports grouped std → external → crate (module granularity), constants
`SCREAMING_SNAKE_CASE`. Four established modes plus the partial adaptive integration
above; enhanced (default) adds NAK-decay quality scoring + optional exploration.
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
`src/telemetry_doc.rs` (document model) and `src/telemetry_file.rs` (publish mechanics):

- **`tests/telemetry_edge_cases.rs`** (9 tests): zero connections (`connections:[]`
  idle-not-absent), active link with zero traffic (`bitrate_bps:0` present not absent),
  very-high RTT 5000 ms verbatim, `schema_version==1` pinned (constant + JSON,
  number-not-string, leads the document), `bitrate_bps == wire_bytes*8` on fixed
  inputs (0, 1, 150k, 312.5k, 1M bytes/s).
- **`tests/telemetry_fixture_parity.rs`** (5 tests): every fixture in the matrix asserted
  byte-identical between `tests/fixtures/` and `bindings/typescript/tests/fixtures/`,
  plus structural parity on the golden (top-level keys, `schema_version==constant`,
  frozen 7-key per-conn set) and the additivity proof (`telemetry-golden` minus the two
  top-level ADR-003 keys == `telemetry-legacy-producer` exactly). All paths anchored at
  `CARGO_MANIFEST_DIR` -- inside the repo, Rule D clean.
- **`tests/telemetry_fixtures.rs`** (8 tests): the producer half of the cross-language
  matrix -- legacy / golden / mapped / reordered / reconnect / degraded-startup /
  degraded-reload / unknown-fields. Regenerate with
  `UPDATE_GOLDEN=1 cargo test --test telemetry_fixtures` (rewrites BOTH copies).

Key seam: `build_telemetry_json(last_updated_ms, &TelemetryInputs { .. })` takes an
explicit ms arg. Tests call it with a fixed timestamp (`1_749_556_546_000`) -- never
`publish()` -- to stay non-flaky. Do not conflate with the tokio virtual-clock seam
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
-   `bindings/typescript/tests/telemetry-fixtures.test.ts` (13 tests) is the consumer half
  of the cross-language matrix: it parses the Rust-written fixtures and asserts
  old-producer tolerance (every added field reads `undefined`, and `undefined` is NOT
  conflated with the positive `'absent'` state), twin-modem disambiguation by `link_id`,
  reorder/reconnect identity stability, both degraded modes, the seven frozen degraded
  reasons, and forward tolerance of a future producer's unknown keys.
- `bindings/typescript/tests/telemetry-roundtrip.test.ts` (14 tests) is the
  **byte-parity passthrough proof** — see BYTE-PARITY ROUND TRIP below.

> **`extra_fields_stripped_or_rejected` was ADAPTED, not weakened.** That test used
> `iface` as its stand-in for "a field a future producer might add"; `iface` has since
> BECOME a known optional field, so the placeholder moved to a genuinely unknown key and
> the now-known field is additionally asserted to be PRESERVED. Same contract, read from
> both sides.

### BYTE-PARITY ROUND TRIP — the identity-passthrough proof

`bindings/typescript/tests/telemetry-roundtrip.test.ts` asserts
`JSON.stringify(telemetrySchema.parse(bytes)) === bytes` for all seven
producer-ordered fixtures, so nothing the sender emits — `iface` and `link_id`
included — is dropped by the reader.

**Why the suite exists at all:** a Zod object strips undeclared keys *silently and
successfully*. A reader that has never heard of `iface`/`link_id` parses a mapped
snapshot with no error and hands the consumer a document with both twins' identities
deleted — which is what the released `@ceralive/srtla-send@2026.6.2` reader did, and
what no "does it parse?" assertion can see. Re-serializing and comparing bytes does
see it.

Two properties keep the suite honest and must survive any edit:

- **The schema declares its keys in the producer's own field order**, which is why
  byte parity (not mere deep-equality) holds. Reordering `connectionTelemetrySchema`
  or `telemetrySchema` breaks it — that is the intended alarm, not a test bug: fix
  the order, do not relax the assertion to `toEqual`.
- **A falsifiability control** (`a stripped identity field falsifies the byte-parity
  assertion`) deletes exactly what a stripping parser would delete and requires the
  comparison to FAIL. Without it, byte parity would also pass for a reader that
  strips fields the producer never emitted.

`telemetry-unknown-fields` is excluded from the byte-parity set by construction (it
is hand-ordered and carries a future producer's keys); it gets an idempotence
assertion — one pass reaches the fixed point — plus the known additive fields kept.
The old-shape half asserts a pre-identity payload parses, round-trips byte-stably,
and reports both fields as `undefined` with **no key materialized** (no `null`, no
`""`), because an omitted optional must stay omitted for "absent" to stay
distinguishable from "empty".

`tsconfig.json` fix: added `tests/**/*` to `include`; moved `rootDir: "src"` into
`tsconfig.build.json` only. This ensures `bun run typecheck` typechecks tests (not
just `src/`), while `bun run build` still emits only `dist/{index,sender/index,
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

### Test-only duplicate-DATA receiver experiment

`src/sender/duplicate_data.rs` and its forwarding call site are compiled ONLY with
`test-internals`. `SRTLA_TEST_DUP_EVERY=<positive n>` plus
`SRTLA_TEST_DUP_RETX_BIT=0|1` enable a per-process cadence; absent/invalid settings
disable it. The hook selects every nth complete DATA header, flushes the original
batch, and sends a copy only if that flush succeeded and left no unsent suffix.
The alternate must be registered and not timed out. `send_copy` borrows the
connection immutably and calls its existing bound socket directly: NO batch queue,
packet-log, in-flight, bitrate or `SequenceTracker` insert. Preserve this bypass
when reusing the primitive for future probe work; single-owner tracker semantics
are unchanged. Only byte 4 bit 2 (`0x04`, second word bit 26) is changed on the copy,
per `draft-sharabayko-srt-01` §3.1.

`tests/netns_dup_spike.rs` is explicitly ignored and requires real sudo/netns,
tcpdump, Python, libsrt's `srt-live-transmit`, and a built CeraLive receiver under
`SRTLA_REPO` (never an ambient PATH receiver). Its A/B/control runs use classic mode,
200 pps × 15 s, 1316-byte messages, latency 2000 ms and lossmaxttl 40. Loopback pcap
duplicates prove libsrt admission; the fresh-port control separately proves 50
replays arrived at SRTLA while none reached libsrt. Source/sink bytes must match.
`-fullstats` disables counter clearing, so all four end-minus-zero counters,
including belated, are cumulative even though CSV names omit `Total`. Wire choice
minimizes the sum of belated and retransmitted events (tie → clear). Use the bounded
command in README; `DUP_SPIKE_OUTPUT` retains artifacts. `DUP_SPIKE_EXPECT_DAMAGE=1`
is solely for the scratch sequence-corruption falsifiability run and is not a
passing substitute for the normal three-variant spike. Non-feature release binaries
must contain no `SRTLA_TEST_DUP` strings. No CLI/telemetry contract changes.

## TS BINDING TOOLING

The binding package manager is **Bun `1.4.2`**, pinned by `packageManager` and locked by
`bindings/typescript/bun.lock`. Run package commands from `bindings/typescript/` with Bun
(`bun install --frozen-lockfile`, `bun run lint`, `bun run typecheck`, `bun run test`,
`bun run build`). Bun is now BOTH the dependency manager and the test runtime — the
package API and tests were always Bun-native, and the package manager finally matches.

**`bun.lock` is the ONLY lockfile.** `pnpm-lock.yaml`, `package-lock.json`, and
`yarn.lock` are forbidden: a second lockfile resolves a different dependency graph than
the one CI installs, and it does so silently. `scripts/bindings_package_manager_contract_test.sh`
enforces this (bun `packageManager`, `bun.lock` present, the other three absent, and no
pnpm invocation anywhere in `.github/workflows/`) and runs in the `ci.yml` **`bindings`**
job, alongside `scripts/bindings_release_ref_contract_test.sh`.

**Both binding contract scripts run in the `bindings` job, NOT the Rust `test` job, and
both evaluate JavaScript with `bun`.** They used to run in `test`, which installs no JS
runtime at all — so they executed only on whatever Node the GitHub runner image happened
to ship. Masking `node` from `PATH` reproduced exit `127` on both. They now run after
`setup-bun` with `working-directory: .` (the job's default is `bindings/typescript`, but
these are repo-root scripts). Do NOT move them back, and do NOT add `setup-node` to
either job. The ONE sanctioned Node island is `ci/verify-bindings-release-ref.sh`, which
runs in `publish-bindings.yml`'s Node-26 OIDC publish job and keeps its `node`
invocation verbatim; `bindings_release_ref_contract_test.sh` exercises it through a
Bun-backed `node` shim. Pinned by
`test_binding_contracts_run_under_bun_never_on_an_ambient_node`
(`scripts/release_workflow_contract_test.py`).

**That shim is prepended to `PATH` UNCONDITIONALLY — never behind an
`if ! command -v node` guard.** A GitHub runner ships an ambient Node, so a
conditional shim never fires on the one machine that matters: the contract test would
resolve the island's `node -p` to an unpinned ambient runtime and go on reporting OK,
proving nothing about the Bun-only job it now lives in. The script therefore prepends
the shim every run and then **asserts** the result — `command -v node` must be the shim
AND `process.versions.bun` must be set, which is a behavioral check no real Node can
pass — failing closed otherwise. The resolved runtime is echoed on the success line
(`node-runtime=bun@<version>`) so every CI log carries the proof. The `export` is
scoped to that process; the production publish job resolves its own real Node 26 and is
untouched.

The `bindings/typescript/` package uses Biome **2.5.9** via `@ceralive/biome-config` **2026.8.0** (the workspace canon — keep `biome.json`'s `$schema` on the same Biome patch) as its first linter/formatter. The declared range is `^2.5.9` and the lockfile deliberately HOLDS 2.5.9 rather than floating to a newer published patch, because the canon version is set workspace-wide by `@ceralive/biome-config`, not per-repo; bump it here only when the canon package bumps. The `biome.json` in `bindings/typescript/` extends `@ceralive/biome-config` (`"extends": ["@ceralive/biome-config"]`). ESLint and Prettier are not used. Run `bun run lint` from `bindings/typescript/` (check) or `bunx biome check --write .` (apply fixes). The binding gate includes `bun run lint && bun run typecheck && bun run test && bun run build`.

**Golden fixtures are excluded from Biome** — `biome.json` sets `files.includes` to `["**", "!dist", "!**/tests/fixtures"]`. `tests/fixtures/telemetry-golden.json` is a deliberately byte-identical copy of the Rust producer golden (`tests/fixtures/telemetry-golden.json` at the crate root): the single-line, newline-free atomic-publish telemetry shape (ADR-001). If Biome pretty-prints it (multi-line + trailing newline), the cross-language parity test (`tests/telemetry_fixture_parity.rs` — `rust_and_ts_goldens_are_byte_identical` plus the newline-free assertion) fails every Rust test job in CI. **Do not remove this exclude, and never `biome check --write` the fixtures** — re-sync the two goldens by editing both byte-for-byte instead.

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
  applied in adaptive mode, which owns its stall handling; also never
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

After rebuilding a reconnecting socket against its cached peer, the sender starts a
detect-only DNS diagnostic off the housekeeping loop. The blocking system resolver runs
on one detached standard thread at a time; a Tokio task waits up to 3 seconds for its
result, while the actual resolver retains the process-wide permit until it returns even if
that wait times out or is cancelled. This prevents overlapping resolver work without
joining Tokio runtime shutdown. An empty or failed answer is inconclusive rather than
drift. The existing peer is kept when it remains among fresh
answers; a genuine drift emits at most one receiver-identity warning per minute across the
whole process. **DEFERRED, not implemented: coordinated whole-bond receiver migration.** SRTLA's
receiver-generated full ID makes swapping a single uplink to a different receiver
instance unsafe — it would split the bond, with some uplinks registered against one
receiver identity and some against another. `apply_connection_changes` deliberately
preserves surviving sockets/registrations across a SIGHUP reload, so SIGHUP is not a
substitute mechanism either. A later, separately-scoped change must specify and
implement any coordinated multi-uplink migration; do not add a single-uplink swap in
the meantime. Upstream `171ddc1` was explicitly deferred because its metadata-first
socket replacement can leave an old-peer socket live after a rebuild failure, reload
additions do not share one authoritative bond endpoint, and queued old-reader traffic has
no generation guard. Full acceptance requirements are recorded in
`docs/notes/upstream-sync-2026-09-evaluation.md`. `src/connection/socket.rs`,
`src/connection/mod.rs`.

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

Six registration-hardening bugfixes, found by the post-merge verification waves.
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
- **Out-of-phase control logging is bounded without weakening accounting (2026-09 sync).**
  Every rejected REG3/REG_ERR still increments its diagnostic counter and returns the
  explicit no-op event. The first rejection of each kind logs at `WARN`; later rejections
  are `DEBUG`, so a spoof flood cannot amplify default-level logs.

Pinned by `replayed_reg3_does_not_wipe_a_live_connection`,
`reg2_broadcast_retry_skips_already_connected_uplinks`,
`out_of_phase_reg3_is_counted_and_ignored`,
`out_of_phase_reg_err_does_not_disconnect_a_live_uplink`,
`out_of_phase_reg_err_does_not_damage_another_uplinks_handshake`,
`in_phase_reg_err_still_aborts_the_registration`, and
`failed_reg2_resend_revokes_a_stale_pre_existing_grant`
(`src/tests/batch_io_tests.rs`) plus the SIGHUP reset assertions in
`src/tests/sender_tests.rs`.

### REVERTED: the REG_NGP acceptance gate and its REG3-wait expiry (rounds 4-5)

Two further rounds of the same arc were written, gated green, and then **REVERTED**
(`ed7e74f` and `1680670`, reverted by `813832b` and `b74da1a`). They are recorded here
so they are not re-attempted from the same premise. **Rounds 1-3 above are untouched and
remain in force.**

- **Round 4** additionally required `awaiting_reg3.is_empty()` and the uplink's own
  `connected` flag before accepting a REG_NGP, closing a narrow window in which a forged
  REG_NGP could re-open `pending_reg2_idx` on a just-connected link (`active_connections`
  is refreshed only by a housekeeping tick, so it lags `SrtlaConnection::connected`) and
  thereby walk back into the round-3 REG_ERR teardown.
- **Round 5** tried to bound round 4 by expiring the REG3 wait from housekeeping.

**Why they were reverted: a live-proven liveness regression.** With the gate in place, an
ordinary `srtla_rec` **restart** while the sender is fully connected and forwarding —
a routine operational event on a production device fleet — **never recovered**. Measured
hands-on on loopback against a real `srtla_rec`: 180 s after the receiver came back, 32
REG2 retries, 0 REG1, 0 `connection established`. The receiver legitimately no longer
knows the group and answers every REG2 with a REG_NGP; the gate refuses all of them, and
round 5's expiry did not reach that path. With rounds 4-5 reverted the same scenario
recovers deterministically in ~18 s (15 s `CONN_TIMEOUT` + a ~1 s REG1→REG2→REG3 cycle),
reproduced twice at 18.07 s / 18.09 s with 1 REG1, 2 REG2, 1 `connection established`.

**The residual risk is KNOWN and ACCEPTED.** The uplink sockets are deliberately
unconnected (see the accept-any-source rationale above), so any host that can reach an
uplink's ephemeral port can already inject registration-adjacent traffic — that is the
standing baseline, not something round 4 introduced. What round 4 closed on top of it is
strictly narrower than the one-packet DoS rounds 1-3 closed: it needs a forged REG_NGP
landing inside a sub-second-to-few-second window right after a REG3, **and** a subsequent
forged REG_ERR, and it yields only a re-registration. A guaranteed loss of
receiver-restart recovery is the worse trade, so the gate is out.

**Tracked follow-up (do NOT re-attempt round 4 in isolation).** Rounds 1-5 are five
instances of one defect class: registration phase lives in five uncoordinated fields
(`pending_reg2_idx: Option<usize>`, `awaiting_reg3: HashSet<usize>`,
`reg1_target_idx: Option<usize>`, `pending_timeout_at_ms`, and the lagging
`active_connections` counter), and every acceptance check re-derives "what phase is this
uplink in?" from a different subset. Rounds 3→4→5 each fixed a hazard the previous round
created. The proper fix — independently flagged by multiple reviewers across rounds 4-6 —
is **one per-connection registration-phase enum with a single transition function**
(REG_NGP/REG1/REG2/REG3 phases explicit, every gate exhaustive by construction) plus
removing `active_connections` as an acceptance input. That is a state-machine rewrite
needing its own verification wave and its own receiver-restart hands-on test; it is the
only sanctioned way to revisit this window.

## DOCS DISCIPLINE (Rule A)

Any behavior/structure change updates this `AGENTS.md` and `README.md` in the SAME PR.
Keep the parity contract section authoritative — it is the device-integration contract.
