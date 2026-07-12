# strata-port Branch Evaluation

**Audited ref:** `refs/_audit/strata-port` @ `1866c6c` (irlserver upstream HEAD, pin-verified)
**Our shipping HEAD:** `c6f8834` (CERALIVE `main`)
**Fork point:** `80cd0c4`
**Audit date:** 2026-06-23
**Method:** 3 parallel deep code-audits (CC engine, scheduler/classifier, surface/parity/safety) + independent spot-verification. Every claim cites `file:line` or SHA.

> **Baseline note:** CERALIVE `origin/strata-port` resolves to `3c5da0c` — an older, divergent mirror that predates several upstream commits. It was NOT used as the audit baseline. All findings below are against the irlserver upstream tip `1866c6c`, anchored locally as `refs/_audit/strata-port`.

---

## VERDICT

**REJECT the wholesale port. REIMPLEMENT 2-3 discrete ideas clean-room (NEVER cherry-pick).**

The branch is genuine R&D, not junk. But it is structurally incompatible with our parity contract (11 breaking changes, P1-P11), carries two behavioral regressions that hit our exact use case (heterogeneous cellular bonding), and ships two unauthenticated on-device network listeners. The thing that looked like the headline advantage — the CC/scheduler rework — is where the worst regressions live. Adopting it wholesale would mean re-forking onto a new baseline, breaking CeraUI integration on day one, and inheriting bugs we've already designed around. The right path is to extract 3-4 good ideas and reimplement them clean-room in our codebase, beating the audited flaws.

---

## Section 1: What strata-port Is (Architecturally)

strata-port is not an additive patch set. It is a partial brain transplant of the scheduling and congestion-control layer.

Commit `548ef85` deletes the entire EDPF + BLEST + IoDS + RTT-threshold scheduler stack:
- `edpf.rs` -163 lines
- `blest.rs` -160 lines
- `iods.rs` -114 lines
- `rtt_threshold.rs` -140 lines
- SBD module -467 lines

In their place it introduces a new subsystem:
- `link_cc.rs` +1005 lines — per-link AIMD+delay congestion controller
- `classifier.rs` +416 lines — weak-link gating and classification
- `metrics.rs` +379 lines — Prometheus `/metrics` endpoint
- `control.rs` / `control_socket.rs` / `subscriptions.rs` — JSON-RPC 2.0 control protocol replacing the text protocol
- `keyframe.rs` + `priority.rs` — keyframe-priority steering via size heuristic
- `toml_config.rs`, `ewma.rs`, `reload.rs` — supporting infrastructure

Total diff: 57 files, +6493 / -1781. Adopting it is equivalent to re-forking onto a new baseline. There is no incremental path.

---

## Section 2: Where to Find It

The branch is anchored locally as:

```
refs/_audit/strata-port  ->  1866c6c
```

Fetch command (read-only, does not touch any working branch):

```bash
git fetch https://github.com/irlserver/srtla_send strata-port:refs/_audit/strata-port
```

Pin-verify:

```bash
git rev-parse refs/_audit/strata-port
# must print: 1866c6cd2314488f1df02910bee4de4acd9f4091
```

CERALIVE `origin/strata-port` is `3c5da0c` — an older, divergent mirror. Do not use it as a reference for this branch's current state.

---

## Section 3: Commit Taxonomy

### Already-convergent (present in our `main` via independent work)

These commits exist in strata-port but their substance is already on our `main` through the robustness pass and parity work. Do not re-import.

| SHA | Subject | Our equivalent |
|-----|---------|----------------|
| `22c6966` | EINTR retry on recvmmsg | Robustness pass 2026-06-19 |
| `fcd9d7c` | MTU clamp on send path | Robustness pass 2026-06-19 |
| `261d0e0` | Zero-RTT keepalive rejection | S7 (`test_keepalive_zero_rtt_rejected`) |
| `f61e53e` | Kalman RTT clamp to >=0 | S6 (`get_smooth_rtt_ms` clamp) |
| `ad70b4c` | Dead reader task restart | S5 (housekeeping `is_finished()` poll) |

### Genuinely new (not on our `main`)

These commits introduce behavior or structure we do not have. Each is reachable from `refs/_audit/strata-port` (`1866c6c`).

| SHA | Subject | Category |
|-----|---------|----------|
| `6286e0f` | `link_cc.rs` — per-link AIMD+delay CC engine | New CC subsystem |
| `1def164` | `classifier.rs` — weak-link gating + LowShare/NoTraffic states | New scheduler |
| `feb2b95` | `UplinkBinder` trait — pluggable egress steering, Android callback | New abstraction |
| `72eac3c` | `metrics.rs` — Prometheus `/metrics` TCP listener | New surface |
| `73a410e` | `priority.rs` + `keyframe.rs` — keyframe-priority steering | New routing |
| `1bb18a0` | `rtt.rs` dual-window RTT-min gradient (jitter-immune queue detector) | Good idea |
| `1866c6c` | `control.rs` — JSON-RPC 2.0 control protocol, replaces text protocol | Breaking change |
| `3919115` | `toml_config.rs` — TOML config loading (dead code) | Dead code |
| `f870bda` | `ewma.rs` — EWMA helper | Supporting |
| `81250c5` | Protocol conformance tests (ack/nak/registration) | Good tests |

---

## Section 4: Per-Item Benefit and Risk (Genuinely New)

### `6286e0f` — per-link AIMD+delay CC engine (`link_cc.rs`)

**Benefit:** Introduces per-link congestion awareness. The BDP-relative in-flight cap concept (`~2849 pkts` derived from `rtt_min * bandwidth`) is path-correct and scales with link RTT. The `rtt_min` 30-second windowing with single-sample outlier rejection is handover-aware. The saturating window-growth overflow guard (`link_cc.rs:~380`) prevents integer wrap on very-high-bandwidth links.

**Bugs (empirically confirmed):**
- **B-2 SEVERE — idle-link target inflation (inverted logic).** `link_cc.rs:497-507`: the `else` arm grows `target_bps` by `step` unconditionally when a link is idle (`sane_observed==0`), the exact opposite of its own comment "prevents ramp on idle links". Probe: idle link reaches `MAX_TARGET_BPS=200 Mbps` in ~120 ticks. Effect: the BDP in-flight cap and soft-cap multiplier both go inert, so a previously-idle link can be flooded with no congestion brake on its first real traffic.
- **B-3 MEDIUM — FastRecovery off-by-one.** `link_cc.rs:454-462` decrements before `pick_climb_mode`, so the documented 5-tick window fires 4 ticks. Telemetry (`cc_climb_mode`) contradicts the docs.
- **B-4 MEDIUM — Drain compounds.** `link_cc.rs:517-520` applies ×0.75 every Drain tick (docs say "one-shot"). Probe: 11 ticks collapse target to the 100 kbps floor, then the seed condition re-fires, producing saw-tooth oscillation under sustained RTT inflation.

**Parity conflicts:** P1 (CONN_TIMEOUT reverted to 5), P2 (NAT padding removed), P6 (ADR-001 sink removed), P7 (bitrate_bps is bytes/s not bits/s — 8× off).

**Verdict:** Do not import. Reimplement the BDP cap concept and window-growth guard clean-room, designing out B-2/B-3/B-4 from the start.

---

### `1def164` — weak-link gating + classifier (`classifier.rs`)

**Benefit:** The `loss_degraded` graded demotion latch (`classifier.rs:~290`) is cellular-tolerant and self-clears when starved — a genuinely good design. The HighRtt tier concept is sound.

**Bugs (empirically confirmed):**
- **R1 CRITICAL — marginal-link starvation latch.** `classifier.rs:276-282` + `enhanced.rs:186-191`: a usable-but-weak uplink (< 0.25× fair share, e.g. a 1 Mbps modem beside three 5 Mbps modems at the `enter=62‰` boundary) is flagged LowShare, score crushed to `GATED_LINK_PENALTY=0.02`, traffic drops to ~0, bitrate reads `0.0` within 2 s (`BITRATE_UPDATE_INTERVAL_MS=2000`), re-flagged NoTraffic, dead latch. Clearing needs `share >= 0.75/N`, which needs traffic it can't get. Exploration is OFF by default (`config.rs:62`) and even ON only probes 2nd-best, never a 2%-crushed link when >=2 healthy links exist. Net: strata-port drops the marginal capacity that bonding exists to aggregate — strictly worse than our baseline for the product's core scenario.
- **B-5 LOW-MED — loss threshold too hot for cellular.** `LOSS_BACKOFF_PERMILLE=5` (0.5%) triggers BackingOff at 0.8% sustained HARQ loss, pinning the controller at -84% target for 30/30 ticks.
- HighRtt classifier tier is self-referential (`budget=longest_rtt×3`) and effectively inert below ~2 s RTT.

**Parity conflicts:** P3 (`--mode rtt-threshold` removed), P4 (`--rtt-delta-ms` removed).

**Verdict:** Do not import. The `loss_degraded` latch idea is worth reimplementing clean-room. R1 is a fundamental design flaw for our use case.

---

### `feb2b95` — `UplinkBinder` trait (egress steering)

**Benefit:** Clean, selection-neutral abstraction for pluggable uplink binding. The Android callback pattern is additive and portable. No parity conflicts. No bugs found.

**Parity conflicts:** None.

**Verdict:** Good idea. Reimplement clean-room as `srtla-sender-egress-steering` (T12). Default-off, additive.

---

### `72eac3c` — Prometheus `/metrics` TCP listener (`metrics.rs`)

**Benefit:** Structured telemetry export for external consumers.

**Risk:** Hand-rolled `tokio::net::TcpListener` accepting `0.0.0.0`, no auth, no TLS, no rate-limit, `tokio::spawn` per connection (SYN-flood vector if exposed). No on-device consumer exists. Opt-in, but the binary CeraUI launches would carry this surface.

**Parity conflicts:** P6 (replaces ADR-001 sink without shim).

**Verdict:** Reject as-is. The direction (structured telemetry) is already covered by ADR-001. If a push-telemetry endpoint is needed, it belongs on a Unix socket, not a TCP listener.

---

### `73a410e` — keyframe-priority steering (`keyframe.rs` + `priority.rs`)

**Benefit:** Attempts to route keyframe bursts preferentially to reduce decoder stalls.

**Risk:** Pure size heuristic — 5 consecutive 1316-byte packets = "keyframe burst" (`keyframe.rs:1-20`). Never parses TS/NAL headers. False-positives at sustained high bitrate route non-keyframe traffic down a single link, defeating bonding. The `--priority-bind` UDP sidecar accepts `0.0.0.0`, no auth — spoofable to pin all traffic to one link.

**Parity conflicts:** None directly, but the single-link routing defeats the bonding contract.

**Verdict:** Reject. The heuristic is too coarse for production use. A real keyframe detector requires TS/NAL parsing.

---

### `1bb18a0` — dual-window RTT-min gradient (`rtt.rs:198-221`)

**Benefit:** Jitter-immune queue detector. Compares short-window RTT-min against long-window RTT-min to detect queue buildup while ignoring pure jitter spikes. Correctly scoped, no parity conflicts.

**Parity conflicts:** None.

**Verdict:** Good idea. Reimplement clean-room as part of `srtla-sender-quality-signals` (T11).

---

### `1866c6c` — JSON-RPC 2.0 control protocol (`control.rs`)

**Benefit:** Structured, versioned control protocol. Consistent with cerastream's existing JSON-RPC + `subscribeEvents` interface.

**Risk:** Replaces the text control protocol with no shim (`control.rs:35-39,329-336`). CeraUI cannot drive this binary today — every `--dry-run` preflight, `--stats-file` telemetry read, and runtime mode/quality command fails. No Unix-socket-only constraint; the implementation accepts arbitrary transports.

**Parity conflicts:** P3/P4/P11 (CeraUI text control protocol replaced, no shim).

**Verdict:** The direction is sound (promoted to its own proposal: `srtla-control-protocol-alignment`). The specific implementation is rejected as an import source. Reimplement clean-room: Unix-socket only, dual-support + capability-gated cutover, superseding ADR-001's transport.

---

### `3919115` — TOML config loading (`toml_config.rs`)

**Benefit:** None observable. The config is loaded then discarded (`main.rs:124-126`). Dead code.

**Risk:** Adds `toml` dep and 6 lock entries for zero behavior.

**Verdict:** Reject outright.

---

### `f870bda` — `ewma.rs` EWMA helper

**Benefit:** Clean EWMA implementation, useful for smoothing metrics.

**Parity conflicts:** None.

**Verdict:** Neutral. Reimplement if needed as part of quality-signals work.

---

### `81250c5` — protocol conformance tests

**Benefit:** ACK/NAK/registration conformance tests are genuinely useful. No parity conflicts.

**Parity conflicts:** None.

**Verdict:** Good tests. Reimplement the test patterns clean-room (do not cherry-pick — the test harness assumes strata-port's module layout).

---

## Section 5: Recommendation

**Stance (2026-06-23):** Do NOT cherry-pick strata-port commits. Reimplement each adopted idea clean-room in our codebase, beating the audited flaw. strata-port is a cited design reference, not an import source.

### Adopt (reimplement clean-room)

**T11 — `srtla-sender-quality-signals` (OpenSpec proposal, queued):**
- Our own jitter-immune queue detector (from `1bb18a0` `rtt.rs:198-221`), beating the dual-window concept
- A NON-starving BDP-relative in-flight signal (designing out R1 from the start)
- Adaptive batch-send regimes WITH the hysteresis strata-port lacks
- Our own saturating window-growth guard (from `6286e0f`)
- Verify the EINTR+MTU fix is already on `main` before doing anything there (it is — robustness pass 2026-06-19)

**T12 — `srtla-sender-egress-steering` (OpenSpec proposal, queued):**
- Our own pluggable uplink binder + Android callback (from `feb2b95`)
- Selection-neutral, default-off, additive

**T13 — `srtla-sender-link-cc` (OpenSpec proposal, queued, deferred prototype):**
- Our own per-link CC + classifier that designs out B-2 (idle inflation), B-4 (Drain compounding), R1 (starvation latch), B-5 (cellular loss tuning), B-3 (FastRecovery count) from the start
- Keeps all parity divergences: CONN_TIMEOUT=15, NAT padding, ADR-001 sink, rtt-threshold mode, SIGTERM, empty-start
- Shadow-mode + default-OFF, validated in the `network-sim` harness before any default flip
- Multi-week build, not a merge

**Promoted (not rejected) — `srtla-control-protocol-alignment`:**
- The JSON-RPC control + push-telemetry direction is sound (consistent with cerastream)
- Reimplement clean-room: Unix-socket only, dual-support + capability-gated cutover
- strata-port's specific JSON-RPC code (`1866c6c`) stays rejected as an import source

### Defer

- `loss_degraded` graded demotion latch concept (from `1def164`) — good idea, defer to T13 where it can be designed without R1
- Protocol conformance test patterns (from `81250c5`) — reimplement when the protocol layer is stable

### Reject outright

- `--metrics-bind` Prometheus TCP listener (`72eac3c`) — unauthenticated `0.0.0.0` surface, no on-device consumer
- `--priority-bind` UDP sidecar (`73a410e`) — spoofable, defeats bonding
- Keyframe size heuristic (`73a410e` `keyframe.rs:1-20`) — too coarse, false-positives at high bitrate
- TOML dead code (`3919115`) — loaded and discarded, zero behavior
- `mimalloc` as `#[global_allocator]` (`main.rs:5-7`) — unmeasured allocator swap on a 4 GB SBC; revisit only with constrained-device profiling evidence

---

## Section 6: Parity-Conflict Check Table

The following table records every parity contract item against strata-port's state at `1866c6c`. Items marked CONFLICT would break CeraUI integration or device behavior if imported.

| # | Contract item | Our value / behavior | strata-port @ `1866c6c` | Status |
|---|--------------|---------------------|------------------------|--------|
| P1 | `CONN_TIMEOUT` (`src/protocol/constants.rs`) | **15** (receiver parity, pinned by `conn_timeout_value_pinned`) | **5** (`protocol/constants.rs:31`) | CONFLICT |
| P2 | NAT padding `MIN_CONTROL_PKT_LEN=32` / `send_control_padded` | Present (`src/connection/packet_io.rs`), pinned by `control_packet_padded_to_32b` | **Removed** — all control sends direct (`connection/mod.rs:338,354,362`; grep=0) | CONFLICT |
| P3 | `--mode rtt-threshold` present and functional | Present, functional | **Removed** — parser rejects it (`mode.rs:9-22,62-67`) | CONFLICT |
| P4 | `--rtt-delta-ms` flag | Present | **Removed** — `from_cli` 4 args reduced to 3 (`config.rs:67`) | CONFLICT |
| P5 | `--verbose`, `--dry-run` | Both present | **Both removed** (grep dry-run=0) | CONFLICT |
| P6 | `--stats-file` / `--stats-file-interval` ADR-001 sink | Present (`src/telemetry_file.rs`) | **Flags and `telemetry_file.rs` absent** (ls-tree=empty) | CONFLICT |
| P7 | Telemetry JSON 7-key shape; `bitrate_bps` = wire-bytes/s × 8 | 7-key shape, `bitrate_bps` in bits/s | LinkStats ~30 fields; **`bitrate_bps` is bytes/s (8× off)** (`stats.rs:288`, `metrics.rs:117`) | CONFLICT |
| P8 | Clean SIGTERM/SIGINT shutdown | Handler present; stats file unlinked | **No handler** (only SIGHUP) (grep SIGTERM=0) | CONFLICT |
| P9 | SIGHUP reload guard | Present | **Present** (the one parity item respected) | OK |
| P10 | Empty-start non-fatal | Non-fatal; waits for SIGHUP | **Hard-errors on empty start** (`sender/mod.rs:96-101`) — crash-loop on device | CONFLICT |
| P11 | CeraUI text control protocol | Text protocol over stdin/Unix socket | **Replaced by JSON-RPC 2.0**, no shim (`control.rs:35-39,329-336`) | CONFLICT |

**SRTLA_CUTOVER_VERSION:** `2026.6.2` — the first receiver-only srtla release (ADR-003 accepted). `srtla << 2026.6.2` conflicts (C sender present); `2026.6.2` and later coexist. strata-port does not affect this version pin.

**Impact summary:** CeraUI cannot drive the strata-port binary today. Every `--dry-run` preflight, `--stats-file` telemetry read, and runtime mode/quality command fails. Bonded throughput regresses on cellular (P1). The device crash-loops during bring-up (P10).

---

## Appendix A: Bugs Summary

| ID | Severity | Location | Description |
|----|----------|----------|-------------|
| B-2 | SEVERE | `link_cc.rs:497-507` | Idle-link target inflation — `else` arm grows `target_bps` unconditionally when `sane_observed==0`, opposite of its own comment. Idle link reaches `MAX_TARGET_BPS=200 Mbps` in ~120 ticks. |
| R1 | CRITICAL | `classifier.rs:276-282` + `enhanced.rs:186-191` | Marginal-link starvation latch — weak uplink flagged LowShare, score crushed to 0.02, traffic drops, re-flagged NoTraffic, dead latch. Drops the marginal capacity bonding exists to aggregate. |
| B-3 | MEDIUM | `link_cc.rs:454-462` | FastRecovery off-by-one — decrements before `pick_climb_mode`, 5-tick window fires 4 ticks. |
| B-4 | MEDIUM | `link_cc.rs:517-520` | Drain compounds — ×0.75 applied every tick (docs say one-shot). 11 ticks collapse to floor, saw-tooth oscillation. |
| B-5 | LOW-MED | `LOSS_BACKOFF_PERMILLE=5` | Loss threshold too hot for cellular — 0.5% threshold pins controller in BackingOff at 0.8% HARQ loss. |

---

## Appendix B: New On-Device Surface

Both listeners are opt-in but ship in the binary CeraUI launches:

- **`--metrics-bind`:** hand-rolled Prometheus `/metrics` over `tokio::net::TcpListener`, accepts `0.0.0.0`, no auth/TLS/rate-limit, `tokio::spawn` per connection (SYN-flood vector if exposed). No on-device consumer.
- **`--priority-bind`:** UDP sidecar, 5-byte `[0xC1][u32 window_ms]` datagrams, accepts `0.0.0.0`, no auth. Spoofable to pin all traffic to one link, defeating bonding.

---

## Appendix C: Dependency Changes

- `+toml` in manifest; lock +6 entries (mimalloc/cc/toml*)
- `smallvec = =2.0.0-alpha.12` pre-release pin (inherited upstream) — worth review before any adoption
- `rand` RUSTSEC-2026-0097 resolved on runtime 0.10.2 and dev-only 0.9.4; no ignore remains
- `mimalloc` as `#[global_allocator]` (`main.rs:5-7`, secure+v3) — unmeasured allocator swap on a 4 GB SBC
