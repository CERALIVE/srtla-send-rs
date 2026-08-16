# Upstream sync triage: irlserver/srtla_send `80cd0c4..c9f6bb2` (2026-08)

Successor to `docs/notes/strata-port-evaluation.md` (2026-06-23 audit @ `1866c6c`). That
audit rejected a wholesale port of upstream's sans-IO/strata restructure and listed 11
parity conflicts (P1-P11). This document re-triages the fork point to upstream HEAD,
covers every commit in the range, and records the merge-strategy deviation used to land
the history.

Range: `80cd0c45851ef2b3d9ee48864a8a63e7fcd3bf74..c9f6bb2296f236d60802f2ec3b79d9da4dac6e28`
(irlserver/srtla_send `main`). Commit count verified twice: `git log --format='%h'
80cd0c45851ef2b3d9ee48864a8a63e7fcd3bf74..c9f6bb2296f236d60802f2ec3b79d9da4dac6e28 | wc -l`
returns **138** both before the merge (plan-time) and after it (this worktree, on top of
`todo/2-wave2` cut from merge commit `b4e149747ae32e68f4c46a03605e094b67ab5819`).

## Verdict taxonomy

Two tiers. Every row carries exactly one label in column 4.

### FINAL verdicts

- **ADOPT** — imported as-is, no fork-specific change needed beyond the mechanical merge
  conflict resolution.
- **ADOPT-WITH-FORK-FIX** — imported, but with a named fix or redesign applied on top
  (either because the upstream code itself has a bug we found, or because the raw
  upstream shape does not fit our layout/contract and needs adapting).
- **ALREADY-CONVERGENT** — the substance is already on our `main`, independently
  (clean-room fix, pre-existing design, or our own earlier port). The evidence column
  names our equivalent file/commit/test.
- **DEFER-FOLLOWUP** — valuable but blocked on a named precondition: the `link_cc`/
  classifier subsystem we don't have, live BELABOX interop we can't prove, or a
  measurement environment we don't have access to.
- **REJECT** — parity conflict, security conflict, dead code upstream, or a measured
  regression. The evidence column names the reason.

### PROVISIONAL state

- **PENDING-EVAL** — allowed ONLY for `0cc0c6d` and `24b5f64`. A later todo (13) runs the
  A/B measurement infra and resolves both to a FINAL verdict per this mapping:
  adopted ⇒ `ADOPT-WITH-FORK-FIX`; measured regression ⇒ `REJECT` with the numbers;
  environment unavailable ⇒ `DEFER-FOLLOWUP`. Every other row in this document already
  carries a FINAL verdict.

### Mapping from the draft's analysis labels

The research draft (`.omo/drafts/upstream-sync-irlserver.md`, main repo) used
MECHANICAL / TEST-ONLY / CI-ONLY as *analysis* labels for the wave-3 completeness sweep's
29 leftover commits, not as verdicts. They map into the taxonomy above as follows:
a mechanical/style/docs/CI commit whose substance our tree already reflects, or does not
need, becomes `ALREADY-CONVERGENT` (if we have an equivalent) or `REJECT` (if it's not
applicable — e.g. upstream's own CI branch policy, or a restructure we didn't adopt);
a test-only commit our suite already covers becomes `ALREADY-CONVERGENT`, naming our test
file; a CI-only commit for a workflow we replaced becomes `REJECT` with the reason.

## Merge-strategy deviation (`-s ours`)

Recorded first by todo 3 in `.omo/notepads/upstream-sync-irlserver/decisions.md` (main
repo); folded in here per the plan's instruction.

`scripts/upstream-merge.sh` (and `AGENTS.md` → UPSTREAM RELATIONSHIP) documents a
real-content merge: `git merge refs/remotes/irlserver/main --no-ff` with no strategy
override, letting git auto-apply every non-conflicting upstream hunk. This sync used
`-s ours` instead, for two reasons:

1. Upstream `c9f6bb2` carries a sans-IO workspace restructure (strata-port, PR #15/#16).
   Auto-application would land an uncontrollable, unreviewed subset of it on a fork whose
   layout has already diverged (the CLI moved into `src/cli.rs`, the private `main.rs`
   module tree was deleted).
2. Fork-only production fixes live in files upstream also touches —
   `src/sender/selection/edpf.rs` (`BOOTSTRAP_CAPACITY_BPS`) and
   `src/sender/selection/mod.rs` (`with_congestion_escape`), both from `b83f97b`
   (see "EDPF fork-side fixes" below). A content merge is precisely the mechanism that
   would silently revert them.

`-s ours` records only the history relationship — upstream is now an ancestor, so the
next sync computes a correct merge base — while provably changing zero bytes of the tree
(`git diff --exit-code MERGE_SHA^1 MERGE_SHA` is empty; both trees are OID
`678ddb5e53830a4cbc761481244457b7bd5c8e68`). Consequence: every adopted upstream commit in
this table arrives, or will arrive, as an explicit follow-up port commit (todos 4-13),
individually reviewed and gated — nothing is adopted by merge auto-application. This
deviation is local to this sync's merge invocation; `scripts/upstream-merge.sh` itself was
not edited.

## EDPF fork-side fixes (context for the EDPF rows below)

Todo 1 found that upstream's own EDPF pipeline never worked: `--mode edpf` forwarded zero
DATA packets on any topology, from the moment the mode was introduced. Two bugs, both
inherited from upstream, both fixed on our side as commit `b83f97b` on
`merge/upstream-2026-08` (full story:
`.omo/notepads/upstream-sync-irlserver/issues.md`, "Todo 1 FOLLOW-UP"):

- **E1 — cold-start deadlock** (`src/sender/selection/edpf.rs`): `predicted_arrival`
  returned `None` whenever `current_bitrate_bps <= 0.0`, which is true for every link at
  startup, so EDPF selected nothing, forwarded nothing, measured nothing, forever. Fixed
  with a flat `BOOTSTRAP_CAPACITY_BPS` placeholder for unmeasured links.
- **E2 — BLEST permanent starvation** (`src/sender/selection/mod.rs`): the static OWD
  guard excluded the high-RTT uplink on every tick with no escape, so a 30ms+150ms bond
  never used its slow link and dropped excess instead of spilling to it. Fixed with
  `with_congestion_escape`.

Because upstream's own EDPF code was broken and the fork fixed it independently (not
upstream), every EDPF-introducing commit in this range is `ADOPT-WITH-FORK-FIX`, not a
clean `ADOPT` — the note on each such row references `b83f97b`.

## Wave-2/wave-3 audit outcomes referenced below

- **Scheduler audit** (in-repo `strata-port-evaluation.md` R1-B5 flaws, re-checked at
  `c9f6bb2`): R1 starvation **FIXED** upstream (`cc5e6ed` probation retest + `616047e`
  rankable-not-excluded, `GATED_LINK_PENALTY=0.02` trickle); **B-2** idle inflation
  **FIXED** (`3b6cf8e`); **B-3 FIXED**; **B-4 FIXED** (`6286e0f` Drain one-shot); **B-5
  PARTIAL** (threshold still 5‰, but consequences mitigated by a ≥30%-load precondition +
  `loss_uncongestive` + a delivered-throughput floor, `cf7144a`). All of these fixes live
  in the `link_cc`/classifier subsystem this fork does not have (culled by our own
  `548ef85`-equivalent history — we never ported `link_cc` in the first place), so they
  stay `DEFER-FOLLOWUP` pending the deferred multi-week T13 clean-room build, which will
  consume this refreshed evidence as input.
- **Frozen bug lists per adopted commit** (from the third research wave):
  - `71f4ecc` (NAK offset-16): the naive port is risky without hardening — `end =
    0x7fff_ffff` wraps `seq` to 0 while `seq <= end` with `wrapping_add`, emitting up to
    999 bogus loss IDs; the 1000-entry cap silently truncates. Fix ships with 31-bit
    endpoint validation, wrap-safe stop-after-end, and a truncation warning.
  - `bd6fad8` (monotonic `now_ms`): sound for internal deltas; `last_updated_ms` in
    telemetry must stay wall-clock `SystemTime` (the TS watcher compares it against
    `Date.now()`; an anchored monotonic clock would drift after a wall-clock step and
    produce false staleness).
  - `a8d8a37` (RTT-velocity recovery gate): sound with fixes — the upstream velocity unit
    is ms/update, not ms/s (the Kalman filter has no `dt` term, and upstream's own log
    line mislabels the unit); the `2.0` threshold is uncalibrated; the TOML config key is
    dead weight for us. Adopt with corrected-unit docs and a pinning test.
  - `57525c7` + `d53d8bc` (EDPF velocity + BDP cap): **DO-NOT-PORT-AS-IS**. HIGH: if every
    link is over the BDP cap, selection returns `None` and empties the pool (violates the
    fork's own `04140d2`-style "pool never empties" invariant). MEDIUM: zero-RTT samples
    bypass the cap entirely. MEDIUM: the `0.005` constant embeds an unrecorded update
    cadence. LOW: unguarded division can produce `NaN`. Redesign as a ranking penalty (or
    least-over-cap fallback) instead of a hard exclusion, with finite-value guards.
  - `a094863` + `3b2c425` (ACK-RTT ownership): sound with fixes — HIGH: 31-bit sequence
    wrap breaks the `ack <= highest_acked_seq` guard at `0x7fff_ffff → 0` (needs
    serial-number comparison, not raw `<=`); MEDIUM: `rtt == 0` rejection starves sub-ms
    LAN RTT (accepted limitation for our cellular/Starlink device target — consistent
    with our own S7 zero-RTT rejection; recorded as a known limitation, not a bug).
  - `c9f6bb2` (unconnected uplink sockets): HIGH-1 foreign-source datagrams reach protocol
    state unvalidated (source address discarded in `recvmmsg`) — port keeps accept-any
    (security rationale below) but passes the source through with a rate-limited warn +
    counter on mismatch, never silently dropping. HIGH-2 the periodic batch-flush path
    registers a drained queue as sent (`packet_log`/`in_flight`/`last_sent`) BEFORE
    confirming the I/O succeeded, so a hard flush error silently loses the claim of
    delivery — port uses structured progress (register only the confirmed prefix, requeue
    the rest) and routes periodic-flush errors into `mark_for_recovery` consistently, the
    same path the send-loop already uses. MEDIUM-3 recovery leaves stale seq-tracker
    ownership for up to 5s — add `SequenceTracker::remove_connection` on recovery.
    MEDIUM-4 reconnect never re-resolves DNS — our tree has the identical flaw
    (`connection/mod.rs:530` reuses `self.remote`); fix ships alongside: re-resolve on
    reconnect, swap only on success. LOW-5/6: single-datagram `send_to` lacks the batch
    path's EINTR retry and `BATCH_SEND_SIZE` cap parity in the fallback path.
- **Security rationale for accept-any sockets**: unconnected sockets mean any source can
  send datagrams that reach protocol state. This is deliberately kept (not hardened into
  a connect()-per-peer model) for C-reference interop — the upstream C `srtla_send`/`_rec`
  pair and BELABOX's implementation both operate accept-any, and NAT/multi-homed receivers
  legitimately reply from a different source address than the one dialed. The mitigation
  is defense in depth, not exclusion: source-mismatch datagrams are still processed (so a
  legitimate NAT-remapped reply is never dropped) but are rate-limited-logged and counted,
  so an operator can see abuse without breaking the interop case the accept-any model
  exists for.
- **Revisit-notes — `/metrics` + priority sidecar**: both stay `REJECT` per the 2026-06
  audit (unauthenticated network-facing surfaces, no on-device consumer — CeraUI does not
  scrape Prometheus and has no UDP-sidecar priority client). Revisit only if a concrete
  on-device consumer is scoped; until then upstream only warns on non-loopback binds
  (`2009a0a`), which is not a substitute for authentication.

## Table

| SHA | date | subject | VERDICT | evidence/note |
|---|---|---|---|---|
| c9f6bb2 | 2026-08-01 | fix(srtla_send): leave uplink sockets unconnected | ADOPT-WITH-FORK-FIX | Q-B decision; guards: source-mismatch rate-limited warn+counter (not silent drop), structured-progress batch flush → `mark_for_recovery`, `SequenceTracker::remove_connection` on recovery, reconnect DNS re-resolve fix, netns reply-from-different-source + dead-link-latency + miri-green tests |
| 71f4ecc | 2026-08-01 | fix(srtla-protocol): read the SRT NAK loss list at offset 16 | ADOPT-WITH-FORK-FIX | 31-bit endpoint validation + wrap-safe stop-after-end + truncation warn; migrates offset-4 fixtures in `src/tests/protocol_tests.rs:144-180,386-403`, `tests/parser_proptest.rs:117-145`, `src/tests/integration_tests.rs:49-59,167-188`, `src/tests/end_to_end_tests.rs:198-242,314` |
| a608584 | 2026-08-01 | Merge pull request #17 from irlserver/feat/librist-mr375-followups | ALREADY-CONVERGENT | merge commit only, no independent content; constituent commits triaged individually in this table |
| 7cd22b2 | 2026-08-01 | refactor(srtla_send): build the CLI on the library instead of a second copy | ALREADY-CONVERGENT | substance already landed on our side as a forced side effect of todo 1's type-unification cascade (see `.omo/notepads/upstream-sync-irlserver/learnings.md` "Todo 1" + "Todo 7"): `src/main.rs` no longer redeclares the module tree, `src/lib.rs` owns it, pinned by `main_has_no_duplicate_module_tree` |
| 3768cf2 | 2026-08-01 | fix(srtla_send): scope the dead-code allowance to the binary's copy of net | REJECT | tied to the sans-IO `net` module restructure we did not adopt |
| d81b13c | 2026-08-01 | fix(srtla_send): omit git metadata instead of printing "unknown" | ALREADY-CONVERGENT | our `src/version.rs` `compose_version_line()` already omits the parenthetical entirely rather than printing a placeholder, and treats detached HEAD / non-`1` `git diff` exit correctly (AGENTS.md Version output parity contract) |
| 1f96b03 | 2026-07-28 | fix(srtla-core): judge links against the peer's real buffer, not a guess | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| a134701 | 2026-07-28 | feat(srtla_send): read the SRT delivery budget off the peer's handshake | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 795d365 | 2026-07-27 | fix(srtla-core): back off the stall rejoin dwell for a link that keeps failing | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| d146fd9 | 2026-07-27 | fix(srtla-core): back off the probation interval when a re-test fails | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 14b13b7 | 2026-07-27 | fix(srtla-core): let a delay verdict cancel the probation re-test | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| d4ca575 | 2026-07-27 | fix(srtla-core): keep duplicate probes out of the cumulative-ACK sweep | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 0ba3edf | 2026-07-27 | fix(srtla_send): flag duplicate probes as retransmissions | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 3b8071a | 2026-07-27 | fix(srtla-core): make the rejoin ramp and the sole-carrier role churn-proof | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 09f0847 | 2026-07-27 | fix(srtla-core): keep the critical-window override off held-out links | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 3b2c425 | 2026-07-27 | fix(srtla-core): only measure RTT on the link that carried the acked seq | ADOPT-WITH-FORK-FIX | paired with `a094863`; needs serial-number-aware `ack <= highest_acked_seq` guard (31-bit wrap fix) |
| 5068d6f | 2026-07-27 | fix(srtla_send): bind the local SRT listener before dialing uplinks | ALREADY-CONVERGENT | our S10 fix — `tests/startup_bind_ordering.rs`, AGENTS.md "ROBUSTNESS FIXES (startup bind ordering, 2026-07-27)" |
| f267592 | 2026-07-27 | chore: remove obsolete impeccable hook configuration | REJECT | n/a — no impeccable hook config exists in this fork |
| 34b6b2b | 2026-07-27 | feat(srtla_send): probe late links with duplicates instead of unique payload | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have, AND mutates SRT R-bit on probe copies — needs live BELABOX interop proof before touching wire payload semantics |
| 66de4e0 | 2026-07-27 | feat(srtla-core): elect a sticky sole carrier when every link is quality gated | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 4e012da | 2026-07-27 | feat(srtla-core): ramp a rejoining link's share instead of restoring it whole | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| a094863 | 2026-07-27 | fix(srtla-core): feed SRTLA ACK round trips into the smoothed RTT | ADOPT-WITH-FORK-FIX | genuine quality improvement (our SRT-ACK RTT is broadcast to all conns holding the seq, `packet_handler.rs:87-90`, and SRTLA-ACK feeds no RTT); port via our `SequenceTracker` for seq-owner gating + 31-bit wrap-safe comparison |
| 86b90aa | 2026-07-23 | feat(srtla_send): bind uplinks by interface index on apple targets | DEFER-FOLLOWUP | optional, `cfg(apple)`-gated Darwin/iOS interface-binding fix — no todo in this plan ported it (`git grep -i apple`/`IP_BOUND_IF` finds nothing under `src/`); low priority given the Linux aarch64 device target; tracked for a future PR if macOS/iOS bonding support becomes a real requirement |
| f0a354c | 2026-07-22 | feat(srtla_send): fast silence pull and runtime-scaled liveness timeout | REJECT | breaks `CONN_TIMEOUT=15` receiver-parity contract (AGENTS.md pinned by `conn_timeout_value_pinned`); upstream ships 5s, drifted from our 15s intentionally |
| 04140d2 | 2026-07-22 | test(srtla_send): prove cross-mechanism gates never empty the pool | DEFER-FOLLOWUP | tests the `link_cc`/classifier subsystem we don't have; the "pool never empties" invariant itself is honored in our EDPF redesign note above |
| e65e0fe | 2026-07-22 | chore: update dependencies | REJECT | upstream deps (rand 0.9, tokio 1.49, clap 4.5) are all behind ours (0.10/1.52/4.6); dependency direction stays ours everywhere |
| b8f05cd | 2026-07-22 | chore: update anyhow | REJECT | same dependency-direction reason as `e65e0fe` |
| ca22c78 | 2026-07-17 | fix(srtla_send): gate unix-only tests so windows cargo test compiles | ALREADY-CONVERGENT | our test suite already gates Unix-only tests (SIGHUP reload etc.) properly for the Windows build, per README "Windows this arm is disabled" |
| 138fa5b | 2026-07-17 | test(srtla_send): netns stall-gate lifecycle test | DEFER-FOLLOWUP | tests the stall-gate `link_cc`/classifier subsystem we don't have (distinct from our own `--stall-deselect` flag) |
| bc8a038 | 2026-07-17 | chore: fmt | REJECT | mechanical fix for upstream's own red CI (fmt failure); n/a — we never import red upstream state, and this fork has its own pinned-nightly fmt gate |
| 3f525b6 | 2026-07-17 | fix(batch_recv): correct import placement of BATCH_SEND_SIZE for Linux | ADOPT-WITH-FORK-FIX | part of the sendmmsg package with `673138d`, evaluate-first per D5 |
| 4eedb41 | 2026-07-17 | Merge pull request #16 from irlserver/sans-io | REJECT | sans-IO workspace restructure rejected wholesale (in-repo `strata-port-evaluation.md` VERDICT: REJECT wholesale port, never cherry-pick) |
| a844258 | 2026-07-17 | Merge pull request #15 from irlserver/strata-port | REJECT | strata-port restructure rejected wholesale, same audit precedent |
| 9314fdf | 2026-07-17 | feat(srtla_send): export stall gate state in stats and metrics | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have; also touches ADR-001-frozen telemetry shape |
| 3e0037e | 2026-07-17 | feat(srtla_send): pin retransmits and probe gated links with duplicates | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| a3621e6 | 2026-07-17 | feat(srtla-protocol): detect the srt retransmit flag | DEFER-FOLLOWUP | supports the duplicate-probe `link_cc` subsystem we don't have |
| fc6ec58 | 2026-07-17 | feat(srtla-core): latch stall gate with rtt-adaptive window and rejoin dwell | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 924609b | 2026-07-17 | style(workspace): apply configured rustfmt import grouping | REJECT | mechanical style-only, tied to upstream's own workspace layout |
| 681215e | 2026-07-15 | refactor(srtla_send): name srtla-core/srtla-protocol by real crate paths, drop boundary shims | REJECT | sans-IO restructure not adopted |
| f2734c9 | 2026-07-15 | refactor(srtla-core): extract the sans-IO core into its own crate | REJECT | sans-IO restructure not adopted |
| 7ae4108 | 2026-07-15 | refactor(srtla_send): split ConfigSnapshot out of the shell-coupled DynamicConfig | ALREADY-CONVERGENT | we already carry both `DynamicConfig` and `ConfigSnapshot` in `src/config.rs`/`src/config/` |
| 975df88 | 2026-07-15 | refactor(srtla_send): peel uplink socket I/O out of connection into a net module | REJECT | sans-IO restructure not adopted |
| 9461bf7 | 2026-07-15 | refactor(srtla_send): relocate selection out of the sender shell | ALREADY-CONVERGENT | we already carry selection as its own module tree, `src/sender/selection/` |
| 9bd6f75 | 2026-07-15 | refactor(srtla_send): split the priority sidecar listener out of the pure CriticalWindow | REJECT | priority sidecar feature itself rejected |
| f727904 | 2026-07-15 | refactor(srtla_send): move uplink receive processing to the shell | REJECT | sans-IO restructure not adopted |
| 6381312 | 2026-07-15 | refactor(srtla-protocol): extract the wire protocol into its own dependency-free crate | REJECT | sans-IO restructure not adopted; we keep `src/protocol.rs` in-tree |
| adadc84 | 2026-07-15 | refactor(srtla_send): lift the socket out of SrtlaConnection into a shell-owned conn_io map | REJECT | sans-IO restructure not adopted |
| ba7712e | 2026-07-15 | refactor(srtla_send): convert uplink sends to pure builders | REJECT | sans-IO restructure not adopted |
| 87c7c31 | 2026-07-15 | refactor(srtla_send): make BatchSender pure (drain + injected clock) | REJECT | sans-IO restructure not adopted |
| aafc966 | 2026-07-15 | refactor(srtla_send): inject clock into registration packet handlers | REJECT | part of the sans-IO clock-injection wave; superseded for us by the standalone `bd6fad8` monotonic `now_ms` port, which achieves the same goal without the restructure |
| e117016 | 2026-07-15 | refactor(srtla_send): inject clock into the ack/nak handlers | REJECT | same reason as `aafc966` |
| 92c2548 | 2026-07-15 | refactor(srtla_send): inject clock into is_timed_out | REJECT | same reason as `aafc966` |
| 8d830d7 | 2026-07-15 | refactor(srtla_send): inject clock into time_since_last_nak_ms | REJECT | same reason as `aafc966` |
| fcada23 | 2026-07-15 | refactor(srtla_send): inject clock into needs_keepalive and perform_window_recovery | REJECT | same reason as `aafc966` |
| 9f4c8c7 | 2026-07-15 | refactor(srtla_send): collapse the dual clock onto now_ms | REJECT | same reason as `aafc966`; this is the setup commit for `bd6fad8`, which we port standalone instead |
| c40eeb8 | 2026-07-15 | refactor(srtla_send): inject clock into connection phase transitions | REJECT | same reason as `aafc966` |
| 5a8e7c9 | 2026-07-15 | refactor(srtla_send): inject clock into ReconnectionState | REJECT | same reason as `aafc966` |
| 62d2e3c | 2026-07-15 | refactor(srtla_send): inject clock into CongestionControl | REJECT | same reason as `aafc966` |
| 32e3cff | 2026-07-15 | refactor(srtla_send): inject clock into RttTracker | REJECT | same reason as `aafc966` |
| 2958108 | 2026-07-15 | refactor(srtla_send): inject clock into BitrateTracker | REJECT | same reason as `aafc966` |
| bd6fad8 | 2026-07-15 | fix(srtla_send): make now_ms monotonic to survive wall-clock steps | ADOPT-WITH-FORK-FIX | ported standalone (not the clock-injection wave above); `SystemTime` carve-out kept for telemetry `last_updated_ms` (TS watcher compares against `Date.now()`) |
| 7ccbfd0 | 2026-07-15 | test(network-sim): adaptive SRT sender to load the bond realistically | ALREADY-CONVERGENT | our `crates/network-sim` + `tests/netns_*.rs` already load the bond realistically |
| a99f672 | 2026-07-15 | test(srtla_send): assert the CC invariant, not retransmit-noise goodput | DEFER-FOLLOWUP | asserts a `link_cc`-subsystem invariant we don't have |
| 6e0d57d | 2026-07-15 | test(network-sim): keep the bonded netem run out of congestion collapse | ALREADY-CONVERGENT | our netns scenarios already avoid congestion collapse via existing scenario tuning |
| eec33b6 | 2026-07-15 | test(network-sim): make the netns topology actually bond | ALREADY-CONVERGENT | our `tests/netns_*.rs` bonds correctly, including the `netns_edpf` fix from todo 1 |
| b909220 | 2026-07-15 | fix(network-sim): drain child pipes so the process under test cannot wedge | ADOPT-WITH-FORK-FIX | landed by todo 8 as `d08c517` — the wedge is real against our `crates/network-sim/src/harness.rs` too; ported as dedicated stdout/stderr drain threads shaped for our harness (not a verbatim upstream copy), with the `child_pipes_are_drained_during_sustained_output` regression test |
| cf7144a | 2026-07-15 | fix(srtla_send): floor the loss backoff at delivered throughput | DEFER-FOLLOWUP | the B-5 mitigation from the scheduler audit; lives in the `link_cc`/classifier subsystem we don't have (partial fix, threshold still 5‰) |
| 87d8564 | 2026-07-14 | test(srtla_send): netem coverage for wire loss on a bonded link | ALREADY-CONVERGENT | our netns netem-loss scenarios already cover this (TEST INVENTORY finding: all 8 upstream test areas covered) |
| a1b4037 | 2026-07-14 | fix(srtla_send): name the stats bitrate field for its actual unit | ALREADY-CONVERGENT | our `bitrate_bps` field is already correctly named and scoped per ADR-001/ADR-002 |
| ace481b | 2026-07-14 | fix(srtla_send): only back off for loss we caused | DEFER-FOLLOWUP | `link_cc`/classifier loss-attribution subsystem we don't have |
| b686ecc | 2026-07-14 | style(srtla_send): apply rustfmt line wrapping | REJECT | mechanical style-only, n/a to our own rustfmt config |
| 417df1c | 2026-07-14 | refactor(srtla_send): remove connection exploration | REJECT | parity contract requires the `--exploration` flag (D2: keep ours) |
| 9b14dd7 | 2026-07-14 | fix(srtla_send): restore the windows build | ALREADY-CONVERGENT | our Windows build is already maintained, with the SIGHUP-reload arm correctly Unix-only |
| f5de6c0 | 2026-07-14 | fix(srtla_send): make LinkPhase a scheduling weight, not an admission gate | DEFER-FOLLOWUP | R1-fix, `link_cc`/classifier subsystem we don't have |
| 3b9f0a2 | 2026-07-14 | feat(srtla_send): probe starved links instead of the second-best one | REJECT | superseded/culled by upstream itself (later replaced by the `34b6b2b` duplicate-probe mechanism); moot, no code to port |
| 24b5f64 | 2026-07-14 | fix(srtla_send): remove the switch cooldown that was costing 15 points | REJECT | dependent-on-rejected: requires 0cc0c6d. Unmeasured, per the taxonomy's explicit rule that Step B runs only if Step A was adopted. `0cc0c6d` was REJECTed, so this step was never exercised. Full A/B evidence: `.omo/evidence/task-13-upstream-sync-irlserver.md` |
| 0cc0c6d | 2026-07-14 | refactor(srtla_send): drop the batch flush on connection switch | REJECT | MEASURED (todo 13, 3 uplinks, 20/60/120ms delay, 4Mbit/link, 6Mbit offered load, 3 paired runs/mode). Mean goodput ratio candidate/baseline vs the pre-committed ≥99% floor: enhanced 0.9713 FAIL, rtt-threshold 0.9758 FAIL, classic 0.9981 pass, edpf 1.0339 pass. The rule requires the floor to hold in EVERY measured mode, so this is REJECT despite two modes passing. Mechanism: without the switch flush, the previous link's partial batch waits for the 15ms `FLUSH_INTERVAL_MS` cooldown instead of flushing immediately; on a 20/60/120ms heterogeneous bond that delay lands on SRT's TSBPD deadline and shows up as sink drops (every `enhanced` baseline run beat every candidate run bar one overlapping pair). The step also silently drops the flush's error-recovery arm (`mark_for_recovery` + `seq_tracker.remove_connection`), so it is not a pure performance trade. NAK and switch-thrash gates passed in all four modes; the activation gate (`switch_count` delta ≥ 1) was met in every gating run, so this is a genuine measured rejection. Full per-run table: `.omo/evidence/task-13-upstream-sync-irlserver.md` |
| 673138d | 2026-07-14 | feat(srtla_send): flush batches with sendmmsg | ADOPT-WITH-FORK-FIX | Q-A user decision (adopt in this merge, overriding the prior deferred-gate note); `BATCH_SEND_SIZE=32` iovec/mmsghdr, `docs/notes/sendmmsg-deferred.md` + AGENTS.md anti-pattern note updated in the same port PR |
| e3f936f | 2026-07-13 | ci(srtla_send): add miri lane over batch_recv unsafe pointer logic | ALREADY-CONVERGENT | we already have our own dedicated BLOCKING miri job (AGENTS.md "Miri lane") covering `batch_recv.rs`'s pure pointer logic |
| c1946f8 | 2026-07-13 | style(srtla_send): apply nightly rustfmt import wrapping | REJECT | mechanical style-only |
| 598706b | 2026-07-06 | feat(srtla_send): stalled-link deselect guard (on by default) | ALREADY-CONVERGENT | substance already on our main as `--stall-deselect`, deliberately default OFF (not upstream's default ON) pending the hardware-validation gate (D2) |
| 1763a9e | 2026-06-26 | test(srtla_send): satisfy clippy --tests on ported test code | REJECT | mechanical, tied to strata-ported test code we don't have |
| d041e3f | 2026-06-26 | test(srtla_send): gate netns tests on observed readiness, not fixed sleeps | ALREADY-CONVERGENT | our `tests/netns_*.rs` already use bounded readiness polling (AGENTS.md "netns de-flake to bounded readiness polling") |
| 195fe78 | 2026-06-26 | test(srtla_send): unit coverage for kalman filter and bitrate tracker | ALREADY-CONVERGENT | covered by our kalman/bitrate-tracker unit tests (TEST INVENTORY finding) |
| 38646f7 | 2026-06-26 | test(srtla_send): srtla wire-conformance + keepalive interop coverage | ALREADY-CONVERGENT | `src/tests/keepalive_interop_tests.rs` + wire-conformance goldens |
| 4714d95 | 2026-06-26 | test(srtla_send): property-fuzz the srt/srtla packet parsers | ALREADY-CONVERGENT | `tests/parser_proptest.rs` |
| 2009a0a | 2026-06-23 | feat(srtla_send): warn when a sidecar binds a non-loopback address | REJECT | priority sidecar feature itself rejected wholesale (revisit-note above) |
| afecf99 | 2026-06-23 | fix(srtla_send): drop the size-based keyframe heuristic, trust the hint | REJECT | the keyframe heuristic feature it fixes is itself rejected/deleted upstream (`b419ef6`) |
| cc5e6ed | 2026-06-23 | fix(srtla_send): break weak-link starvation latch with probation re-test | DEFER-FOLLOWUP | R1 fix, `link_cc`/classifier subsystem we don't have |
| 3b6cf8e | 2026-06-23 | fix(srtla_send): correct cc target-update logic bugs | DEFER-FOLLOWUP | B-2 idle-inflation fix, `link_cc`/classifier subsystem we don't have |
| 1866c6c | 2026-06-20 | fix(srtla_send): retry recvmmsg on EINTR and clamp msg_len to MTU | ALREADY-CONVERGENT | our `batch_recv.rs` already has both, pinned by miri tests `iter_clamps_oversized_msg_len_to_mtu` and the EINTR-retry path |
| 3919115 | 2026-06-20 | fix(srtla_send): saturate the in-flight window-growth multiplication | ALREADY-CONVERGENT | already present in `classic.rs:23` and `enhanced.rs:40` (PORT SURFACE finding) |
| 22c6966 | 2026-06-20 | fix(srtla_send): reject zero-rtt keepalive samples and clamp smoothed rtt | ALREADY-CONVERGENT | our S6 (Kalman RTT clamp) + S7 (zero-RTT keepalive rejection) fixes, AGENTS.md ROBUSTNESS FIXES |
| fcd9d7c | 2026-06-20 | fix(srtla_send): restart silently-dead uplink reader tasks proactively | ALREADY-CONVERGENT | our S5 fix, AGENTS.md ROBUSTNESS FIXES |
| 261d0e0 | 2026-06-20 | fix(srtla_send): measure elapsed-since-failure for all-links-failed timeout | ALREADY-CONVERGENT | our S9 fix, AGENTS.md ROBUSTNESS FIXES |
| 403a442 | 2026-06-15 | fix(srtla_send): de-twitch the link admission gate with sustained signals | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 81250c5 | 2026-06-15 | test(srtla_send): add protocol, ack/nak, and registration conformance tests | ALREADY-CONVERGENT | `src/tests/protocol_tests.rs` and sibling conformance suites |
| f870bda | 2026-06-15 | ci(srtla_send): add cargo-deny supply-chain gate | ALREADY-CONVERGENT | our `ci.yml` already runs `cargo audit`/`deny.toml` |
| f61e53e | 2026-06-15 | fix(srtla_send): refuse zero-valid-ip sighup reload | ALREADY-CONVERGENT | our SIGHUP reload guard, AGENTS.md IP-list reload parity contract |
| ad70b4c | 2026-06-15 | test(srtla_send): add deterministic fake-clock seam for timing tests | ALREADY-CONVERGENT | our `advance_test_clock` seam in `src/test_helpers.rs` |
| 3c5da0c | 2026-06-03 | fix(srtla_send): window the cc rtt-min so handovers don't pin inflation | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 1bb18a0 | 2026-06-03 | fix(srtla_send): reject outlier throughput samples on the cc soft cap | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 73a410e | 2026-06-03 | fix(srtla_send): make the in-flight cap bandwidth-delay-relative | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 72eac3c | 2026-06-03 | feat(srtla_send): jitter-immune queue-building detector | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 616047e | 2026-06-03 | fix(srtla_send): keep quality-gated links rankable, not excluded | DEFER-FOLLOWUP | R1 fix, `link_cc`/classifier subsystem we don't have |
| 35c4d3b | 2026-06-03 | fix(srtla_send): demote lossy links continuously, never hard-kill | DEFER-FOLLOWUP | R1 fix, `link_cc`/classifier subsystem we don't have |
| 5add55a | 2026-06-02 | style(srtla_send): drop unused BatchRegime re-export and clippy nits | REJECT | mechanical, tied to the `link_cc` `batch_regime` feature we don't have |
| 379affe | 2026-06-01 | feat(srtla_send): in-flight cap soft admission gate | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| f35f395 | 2026-06-01 | feat(srtla_send): surface batch_regime in per-link stats | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 79e727b | 2026-06-01 | feat(srtla_send): cc_target_bps as soft cap on enhanced score | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 0a03fef | 2026-06-01 | feat(srtla_send): wire NAK deltas into link_cc loss path | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| a229f8f | 2026-05-27 | docs(srtla_send): sync docs with current scheduling modes | REJECT | upstream's own docs for their `link_cc` scheduler stack; our README/AGENTS.md are maintained independently per Rule A |
| 274c971 | 2026-05-27 | style(srtla_send): cargo fmt | REJECT | mechanical style-only |
| d61d965 | 2026-05-27 | style(srtla_send): clear clippy across all targets | REJECT | mechanical style-only |
| feb2b95 | 2026-05-27 | feat(srtla_send): pluggable uplink binder for egress steering | ALREADY-CONVERGENT | already executed clean-room on our side, our own uplink binder commit `5a73a13` |
| 1def164 | 2026-05-05 | feat(srtla_send): adaptive batch-send regimes | DEFER-FOLLOWUP | `link_cc`/classifier batch-regime ecosystem we don't have; superseded on our side by the standalone `673138d` sendmmsg port |
| 6286e0f | 2026-05-05 | feat(srtla_send): link_cc HAI + FastRecovery + Drain | DEFER-FOLLOWUP | the core `link_cc` engine; the B-4 Drain-fix reference in the scheduler audit above |
| bcfd0f8 | 2026-05-05 | feat(srtla_send): gate weak + cc-backing-off links in enhanced mode | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 19a73b7 | 2026-05-05 | feat(srtla_send): per-link CC soft-cap in shadow mode | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| dd83f55 | 2026-05-05 | feat(srtla_send): weak-link classifier in shadow mode | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| 548ef85 | 2026-05-04 | refactor(srtla_send): cull unproven selection modes | REJECT | culled EDPF/BLEST/IoDS/rtt-threshold/SBD; our parity contract requires those flags (D2: keep ours) |
| 03ed91f | 2026-04-20 | feat(control)!: add JSON-RPC subscriptions on the async Unix socket | ALREADY-CONVERGENT | already covered by our `src/subscription.rs` + jsonrpc stats subscriptions |
| 9e3b4ff | 2026-04-20 | feat(metrics): add Prometheus /metrics endpoint | REJECT | unauthenticated network-facing surface, no on-device consumer (revisit-note above) |
| 15052b1 | 2026-04-20 | feat(priority)!: replace mark_critical RPC with UDP sidecar windows | REJECT | unauthenticated network-facing surface, no on-device consumer (revisit-note above) |
| 809423e | 2026-04-20 | feat(control)!: replace text control protocol with JSON-RPC 2.0 | ALREADY-CONVERGENT | already exists clean-room on our side, `src/jsonrpc.rs` |
| 4fc707f | 2026-04-20 | feat(config): add mark-critical hint command for encoder-driven priority | REJECT | feeds the priority-sidecar feature, itself rejected wholesale |
| 20d081c | 2026-03-22 | ci: add 'feat/improvements' branch to build triggers | REJECT | upstream's own CI branch policy, n/a to our workflows |
| e64e0b5 | 2026-03-15 | refactor: replace #[allow(dead_code)] with #[cfg(test)] for test-only items | ALREADY-CONVERGENT | mechanical hygiene already enforced on our side by the `clippy -D warnings` gate |
| cfa41cb | 2026-03-15 | feat: add shared bottleneck detection across links (RFC 8382) | REJECT | deleted upstream in the later `548ef85`-class cull; moot |
| b419ef6 | 2026-03-15 | feat: add heuristic keyframe detection for priority scheduling | REJECT | deleted upstream (superseded by `afecf99`, itself also rejected) |
| d744c4a | 2026-03-15 | chore: update Cargo.lock for toml dependency | REJECT | dependency of the rejected TOML `--config` feature |
| 14a0398 | 2026-03-15 | feat: add TOML config with --config CLI arg | REJECT | dead code per the in-repo `strata-port-evaluation.md` audit (P-series parity conflict) |
| 870b4e0 | 2026-03-15 | feat: add AsymmetricEwma for fast-down/slow-up smoothing | REJECT | dead code upstream (ADOPT-CANDIDATE DIFFS finding: unreferenced after introduction) |
| ba34eb7 | 2026-03-15 | feat: add link lifecycle state machine | DEFER-FOLLOWUP | `link_cc`/classifier subsystem we don't have |
| d53d8bc | 2026-03-15 | feat: add BDP hard-cap to EDPF scheduler | ADOPT-WITH-FORK-FIX | EDPF-introducing commit — redesigned as a ranking penalty (not hard exclusion) with finite-value guards per the frozen bug list above; fork's own EDPF fixes for the pipeline this touches are `b83f97b` (cold-start deadlock + BLEST starvation) |
| 57525c7 | 2026-03-15 | feat: add velocity penalty to EDPF predicted arrival | ADOPT-WITH-FORK-FIX | EDPF-introducing commit — redesigned as a ranking penalty with finite-value guards per the frozen bug list above; see `b83f97b` for the fork's own EDPF pipeline fixes |
| a8d8a37 | 2026-03-15 | feat: gate window recovery on RTT velocity | ADOPT-WITH-FORK-FIX | corrected units (ms/update not ms/s), recalibrated threshold, pinning test; TOML config key dropped (n/a) |
| de134ed | 2026-03-15 | style: apply cargo fmt formatting | REJECT | mechanical style-only |

## Row count check

138 data rows above, matching the verified commit count. All 138 rows carry a FINAL
verdict (no `PENDING-EVAL` remains) — `24b5f64` and `0cc0c6d` were resolved to REJECT
by todo 13's A/B measurement (see rows above).

## EXACT-SET VALIDATION (todo 14, post-merge)

Every SHA in the table above was resolved to its full 40-character commit id via
`git rev-parse`, sorted and de-duplicated, and diffed against the full,
sorted `git rev-list` output for the same range. Both endpoints are ordinary commits
in local history post-merge, so this is a real, reproducible check, not an estimate:

```
$ grep -oE '^\| [a-f0-9]{7} ' docs/notes/upstream-sync-2026-08-evaluation.md \
    | sed 's/| //;s/ //' | while read sha; do git rev-parse "$sha"; done \
    | sort -u | wc -l
138

$ git rev-list 80cd0c45851ef2b3d9ee48864a8a63e7fcd3bf74..c9f6bb2296f236d60802f2ec3b79d9da4dac6e28 \
    | sort -u | wc -l
138

$ diff <(table SHAs, resolved+sorted) <(git rev-list range, sorted)
(empty)
```

Both sets contain exactly 138 entries; the diff is empty. The 3 merge commits in the
range (`a608584`, `a844258`, `4eedb41`) are included naturally as ordinary table rows,
each individually triaged — not skipped or blanket-verdicted. Full command transcript
in `.omo/evidence/task-14-upstream-sync-irlserver.md`.
