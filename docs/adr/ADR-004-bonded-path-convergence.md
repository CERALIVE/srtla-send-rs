# ADR-004: Bonded-Path Convergence — the receiver-adaptive sender and the single scheduler

## Status

Accepted

> **Numbering note.** This is the fourth ADR in `srtla-send-rs`. It records the
> outcome of the bonded-path-convergence programme (2026-09): why the sender now
> observes the receiver's NAK policy, why every scheduling mode except Enhanced was
> retired in `4.0.0`, and what evidence the verdict rests on. It does **not** change
> the telemetry schema owned by `srtla/docs/adr/ADR-001-telemetry-ipc.md` (carried
> here by `ADR-001-control-protocol.md`), the cumulative-bytes extension in
> `ADR-002-session-bytes-telemetry.md`, or the bind-map sidecar in
> `ADR-003-bind-map-contract.md`. Every telemetry and control addition it names is
> optional and additive; `schema_version` stays `1`.
>
> Numbers in this document are copied from the frozen evidence under
> `docs/evidence/bpc/` (verdict: `m4/verdict.json`; defect round:
> `defects/none.json`; fold-in: `m4/foldin.json`; ablation: `m4/ablation-skipped.json`;
> M5: `m5/skipped.json`; canary: `canary/canary.json`; sender spikes:
> `m2-sender/spike.json`). Where a number is quoted, that file is the source of truth.

## Context

### The failure the programme set out to explain

A bonded sender that used the receiver's SRT NAK stream as its per-link quality
signal behaved catastrophically in two opposite directions depending on one receiver
option, `SRTO_NAKREPORT`:

- **NAK-on** (periodic loss reports): on heterogeneous bonds (scenarios B1/C — one
  slow link) the slow link was starved and useful goodput fell by tens of percent.
- **NAK-off**: under real loss (scenarios A/G) the encoder retransmitted whole
  ACK-to-tip spans on every RTO, with 44–46% of received packets being
  retransmissions.

The root cause is in the receiver's libsrt, not the scheduler
(`docs/notes/receiver-policy-evaluation-2026-09.md` §11, source-verified):

1. Stock SRT inserts a sequence gap into `m_pRcvLossList` **immediately**;
   `SRTO_LOSSMAXTTL` (the reorder tolerance) lives on a separate `m_FreshLoss`
   record and only delays the **first** gap-triggered LOSSREPORT.
2. The periodic NAK timer (`checkNAKTimer`) re-serialises the **whole** loss list
   every `max((SRTT + 4·RTTVar)/2, 20 ms)` under LiveCC and consults neither
   `m_FreshLoss` nor the tolerance. A packet that is merely late on the slow link is
   NAKed within one timer tick, and each such NAK is a window decrement on that link.
3. Under `NAKREPORT=0` the encoder's SRT sender runs LiveCC FASTREXMIT: on each RTO
   it retransmits everything from `m_iSndLastAck` to `m_iSndCurrSeqNo`.

The fix for (2) is on the receiver side — a periodic-NAK gate keyed on fresh-loss
membership (`SRTO_PERIODICNAKGATE`, CeraLive SRT fork; the same shape BELABOX's SRT
branch adopted in mid-2026). This ADR is about what the **sender** must do to be
correct against **both** kinds of receiver, because the fleet will contain both for a
long time.

### Why the receiver cannot negotiate per sender lineage

It was proposed that the receiver detect which sender lineage (BELABOX C,
irlserver Rust, this fork, an old release of this fork) is registering and choose its
NAK policy accordingly. **This is structurally impossible**, and the programme
records it so it is not proposed again:

- `srtla_rec` is **libsrt-free**. It forwards SRTLA-encapsulated UDP datagrams
  between the bonded uplinks and one local SRT socket; it does not participate in
  the SRT handshake and has no place to express a per-sender option.
- The **SRT handshake terminates at the encoder**, not at the bonding sender.
  `srtla_send` forwards the encoder's SRT bytes unchanged; the HSRSP that carries
  `NAKREPORT` is negotiated between the encoder's libsrt caller and the receiver's
  libsrt listener. The bonding sender can *observe* that handshake on its way through,
  but it is not a party to it.

The receiver therefore has exactly one lever — its own policy (`TTL*`, gate, freeze)
— applied uniformly to every sender. The M1 campaign chose `TTL200` + gate + freeze
(`docs/evidence/bpc/m1-ttl/`). The sender must cope with whatever it is given.

### Why the scheduler had to converge

Five CLI scheduling modes (classic, enhanced, rtt-threshold, edpf, adaptive) had
accreted, each with its own private view of link health, ACK attribution and NAK
penalty. Each mode fixed one scenario and regressed another, and every fix had to be
re-derived five times. The programme's rule was: measure all five under one frozen
statistical rule, ship the smallest covering set, and delete the rest.

## Decision

### 1. The sender reads the receiver's NAK policy from the HSRSP

The receive path (`src/protocol/srt_handshake.rs`, `src/receiver_handshake.rs`)
sniffs SRT v5 conclusion handshakes passing from the receiver to the encoder, walks
the extension blocks, and reads the HSRSP: packed SRT version at `E+4`, flags at
`E+8`, receiver TSBPD delay at `E+12`. Bit 4 of the flags is `NAKREPORT`; bit 5 is
`REXMITFLG`. Forwarding stays byte-identical whether or not decoding succeeds.

- **Fail-safe: `None` ⇒ NAK-on behaviour.** `ReceiverHandshake::nak_report_enabled()`
  returns `true` when no HSRSP has been observed. A sender must assume the receiver
  *will* send periodic NAKs until told otherwise, because the NAK-on failure mode
  (slow-link starvation) is the one the sender can mitigate and the NAK-off failure
  mode (bulk retransmission) is the encoder's, not the sender's. The M2 spike
  decoded `ours-old=on`, `ours-new=on`, `irlserver-next=on`, and `irlserver-prod`
  produced **no** concrete observation, recorded as `null` under this rule with no
  parser heuristic (`m2-sender/spike.json`, `M2-HSRSP`).
- **Bond-scoped caching.** The observation is held once per bond under its own lock
  in `SharedStats` (`src/stats.rs`), never per uplink. A SIGHUP-added or
  re-registered link inherits it; housekeeping snapshot rebuilds never clear it; a
  later valid encoder HSRSP replaces it; a malformed handshake never erases it.
- **Unknown stays visibly distinct from on.** Telemetry and `get-status` omit the
  field when unknown (never `null`, never `false`); only the *policy accessor*
  collapses unknown to on.

### 2. The in-flight NAK rule

`SrtlaConnection::handle_nak` (`src/connection/ack_nak.rs`) protects a DATA packet
that the receiver could not physically have judged lost yet:

- **Threshold:** `clamp(rtt_min_ms / 2, 5, 500)` ms, measured from the packet's
  **kernel-acceptance** timestamp (`DeliveryLedger::sent_ms`, recorded by
  `transmit.rs` after the batch flush returns), not from its queue time.
- **K-cap:** at most **3** consecutive premature NAKs per link are suppressed
  (`premature_streak < 3`); the fourth takes the unchanged normal penalty and resets
  the streak. Every normal penalty and every accepted ACK/keepalive RTT sample also
  resets it.
- **Inactive without an RTT sample.** With `has_rtt_sample() == false` the threshold
  is `0.0` and no non-negative age can be below it, so the rule is a no-op until the
  link has earned a real RTT. Missing or evicted ledger evidence likewise disables it
  for that packet. There is no CLI flag, control key or telemetry field; the
  lifetime `premature_naks` counter is status-log-only.
- Suppression retains the packet-log entry and in-flight count and skips congestion
  and loss-cohort accounting. Every NAK frame still reaches the encoder unchanged.

The M2 spike compared K=3 against a test-only bypass: B1 and C settle-rate deltas
were both `0.000` (ON 0/3, OFF 0/3); A and G did not meet the frozen `>5%` disjoint-CI
regression predicate (A `9.914` vs `10.000` Mbit/s, G `4.341` vs `4.401` Mbit/s
medians), so **K=3 is kept** and no confirmation campaign was triggered
(`M2-INFLIGHT`). The bypass compiles only under `test-internals`.

### 3. Retransmit counters are observability only

`SrtlaConnection::rexmit_forwarded` counts SRT DATA packets forwarded on a link with
the SRT retransmission bit (`R`, byte 4 bit 2) set. It is printed per link in the
30-second status log and returned per link as `rexmit_forwarded` on `get-status`.
It is **not** in the ADR-001 telemetry file and it **never affects scheduling or
window arithmetic**.

**Why attribution was rejected.** It was proposed to use the R bit to attribute loss
to the link that carried the original and so avoid the NAK-penalty blindness a
NAK-off receiver causes. It was rejected because a NAK-off receiver **still sends the
first gap-triggered LOSSREPORT** (`SRTO_NAKREPORT=0` disables only the *periodic*
re-report; the immediate report on gap detection and the `LOSSMAXTTL`-deferred first
report remain). The sender therefore already receives one NAK per genuine loss on
either policy; what it lacks under NAK-off is the *repeated* signal, and a
retransmission arriving on link X says nothing about which link lost the original.
Attributing it would charge the wrong link. The counter stays a diagnostic. The M2
spike validated the counter itself: 69,278 DATA packets, 639 independently identified
retransmissions, zero flagged originals, zero unflagged retransmissions, zero capture
drops (`M2-REXMIT`). NAK-off blindness remains a **known limitation**: on G,
adaptive's CI lower bound was `12.803` Mbit/s against the required `13.442` Mbit/s
(`M2-NAK-OFF`).

### 4. The shared signal layer: every mode consumes every signal

Before the matrix was run, the per-mode private views were lifted into one layer
(`src/sender/selection/admission.rs`, `signals.rs`, `shared.rs`; see `AGENTS.md`
→ SHARED SCHEDULER ADMISSION):

- **Admission** runs once before any ranking: Down links are excluded; Degraded,
  Stalled and deadline-held links are probe targets; only zero connected links can
  yield no selection (sole-carrier election, then a connected-only base-score escape).
- **Weights** are lazy and candidate-scoped, in a fixed order:
  `quality × rejoin ramp × Healthy-only preference × soft rate cap`.
- **ACK policy** is one policy: arrival-link/socket-generation attribution, no
  cross-link first-match, no broadcast window growth, probe ACKs prove health only,
  one RTT sample per SRTLA ACK frame.
- **Health**, **delivery ledger**, **loss cohorts**, **queue-delay detector**,
  **duplicate probes**, **rate cap** and the **negotiated-latency deadline** are all
  connection-owned and mode-independent.

**Modes are ranking formulas, nothing more.** That is what made a fair matrix
possible — a mode could no longer win by *seeing* a signal another mode lacked — and
it is what made deletion cheap: retiring a mode removes a ranking function, not a
health subsystem. `SchedulerFeatures` (nine bits, all ON by default) gates the
mechanisms for test-build ablation only; the bits never appear on any public surface.

### 5. The lineage matrix and the `lineage-d1` rule

The M4 matrix (`docs/evidence/bpc/m4/method.md`) measured all five CLI modes on the
`ours-new` receiver lineage (`TTL200`, gate on, freeze on, NAK-on) across 20
scenarios (A–L with B1/B2, M1–M7) at N=5 — 100 primary cells, 546 metric indices,
plus 8 upstream reference cells, an FEC pair at N=3 and two 600 s M8 soaks —
12 h 43 m 28 s, one attempt per index, full-window measurement retained after a
settle timeout. N=5 and the 0.95 coverage threshold were frozen beforehand by the
CI-width spike (`ciwidth/spike.json`: median half-width 1.15% ≤ 2.5%).

**Metrics-v2 gates** (blocking, every post-settle run in a cell): zero
`pktRcvDrop` delta, zero `pktRcvBelated` delta, and `msRcvBuf` minimum ≥ 20% of the
negotiated TSBPD delay, on top of settled N and the paired goodput CI lower ≥ 0.95,
loss delta ≤ 0.1 pp and recovery ratio ≤ 1.10.

**The `lineage-d1` rule** (`scripts/bench/lineage_rule.py`, frozen SHA recorded in
`m4/frozen.sha256`): winner = maximum median useful goodput; a cell is *covered*
only if a candidate passes every gate; ship set = the smallest set covering all
non-exempt primary obligations; base mode = the mode covering the most obligations
(ties → enhanced); every obligation the base does not cover is a `sacrificed_cell`.
Scenario D is exempt only if **all five** candidates fail it. If no covering set
exists, the rule's pre-declared **empty-set terminal** ships the base alone and
lists every obligation as `reason: uncovered`. Two independent executions must be
byte-identical (`b47ee5d6…f4a0`).

### 6. The defect-fix round (five-predicate admission)

One bounded round investigated every cell a mode "should" have won
(`defects/none.json`). A fix was admissible only if **all five** predicates held:
(1) a real pre-existing design claim in the docs; (2) unit tests encoding it; (3) a
**failing** claim test at exact campaign source; (4) a demonstrated code-defect →
lost-cell causal chain; (5) the off-limits surfaces (thresholds, ablation constants,
premature-NAK path, adaptive performance, candidate lock, frozen measurements, C1
manifest) untouched. Three fresh claim probes were run on the embedded base
(`enhanced` sustained-loss ranking, `rtt-threshold` zero-fast-score fallback,
`edpf` bootstrap-through-IoDS-reset): **3 PASS / 0 FAIL**. Every mode assessed
`[true, true, false, false, true]`. **Zero defects admitted, zero fixes, zero reruns,
coverage delta 0.** One unadmitted backlog item is recorded (EDPF's "allocation-free
hot path" claim vs `Vec` allocations in `blest.rs`/`iods.rs`: no causal proof).

### 7. The fold-in round

`m4/foldin.json`: `{"skipped": true, "outcome": "single covering mode; no fold-in
needed"}`. With one member in the ship set there is no donor mode and no ranking
term to port. Steps 3–5 of the fold-in protocol did not execute; the primary
verdict was re-derived byte-identically after the defect round.

### 8. The ablation pins

`m4/ablation-skipped.json`: Enhanced's ranking is `quality multiplier × capacity
score` with switch-hold hysteresis and has **no switchable ingredient** in the
ablation sweep set (`stall, loss, queue, deadline, rejoin, sole, pref, ratecap`) —
those bits gate shared *mechanisms*, not Enhanced's formula. The pins are therefore
the existing defaults, inlined: all nine `SchedulerFeatures` bits **ON everywhere**,
`AdaptiveTuning::SHIPPED` constants, premature-NAK rule ON with K=3. No sweep was
run and no constant was changed.

### 9. M5 and the hardware canary

- **M5** (`m5/skipped.json`): `{"reason": "base mode is enhanced", "gate":
  "skipped"}` — the final gate exists to validate a *changed* default; the default
  did not change.
- **Hardware canary** (`canary/canary.json`): two 1800 s arms (survivor Enhanced vs
  the released `ours-3.3.0` binary, SHA `ebe34ee2…7d77c`) with pass criteria
  `unrecovered_links = 0`, `stream_drops = 0`, survivor goodput ≥ 0.95 × 3.3.0,
  `pktRcvDrop` not worse. **Status: `pending`.** The bench host has no Starlink +
  cellular bond. The runner kit (`scripts/bench/canary/`) dry-runs without binding
  sockets. `selected_alternative`: Enhanced stays default; the canary is the first
  item of the next release. The retired `stall_deselect_real_starlink_repro` test no
  longer exists (`cargo test … --ignored` selected zero tests), so it cannot stand in
  for the canary.

### 10. The verdict

`m4/verdict.json` (authoritative; byte-identical to the provisional verdict after the
empty defect round; lineage gate appended by M4b):

| Field | Value |
|---|---|
| `ship_set` | `["enhanced"]` |
| `base_mode` | `enhanced` |
| `covered_by_base_pct` | **0.0** |
| `refold` / `gate` | `false` / `null` |
| Primary cells | 100 (20 scenarios × 5 modes), **100 fail** `post_settle_zero_drop_belated_rate` |
| `sacrificed_cells` | **37** = 19 primary (`uncovered`) + 18 lineage (`lineage gate`) |
| Exempt | D (all five fail coverage) |
| Lineage gate | Enhanced **0/18** passes (belabox, irlserver-next, irlserver-prod, ours-old on SLT; ours-new on SLS 4003); `swap: null` |
| FEC pair | ratio `0.951794` (−4.82%), narrowly under the 5% failure edge |
| M8 soaks | both FAIL final health (3 Degraded each), 0 crashes |

**Final set: Enhanced only, by the empty-set terminal — NOT genuine coverage.** The
default is unchanged. **DELETED in 4.0.0: `classic`, `rtt-threshold`, `edpf`,
`adaptive`** (CLI, RPC `set-mode`, `SchedulingMode` enum). EDPF's pipeline and
its E1/E2 regression tests are retained *outside* CLI dispatch because fold-in never
ported those pins; nothing dispatches to it in production.

The strict gate failed everywhere for the same reason: every lossy scenario
registers at least one receiver-side drop or belated event in a 45–90 s window even
with a scheduler recovering perfectly at the application level (scenario A alone has
0.2% baseline loss). That is a working-as-specified finding, not a `decide.py` bug,
and the defect round did not find a mode causing *unnecessary* drops beyond that.

### 11. Per-mode pros and cons, derived from the measured cells

Goodput ratios below are `goodput_ratio` (median useful goodput ÷ the cell winner's)
from `m4/verdict.json`, `ours-new` lineage, N=5. Every cell failed
`post_settle_zero_drop_belated_rate`; only additional failing gates are noted.
"Won" = cell winner by median goodput.

| Mode | Won (of 20) | Where it was strong (ratio) | Where it was weak (ratio) | Verdict |
|---|---|---|---|---|
| **enhanced** | 3 — B1, B2, J | B1 `1.000`, B2 `1.000` (heterogeneous bonds — the NAK-on starvation case the programme targeted), J `1.000`, G `1.000`, K `1.000`, M4 `1.000`, M6 `1.000`, F `1.000`, I `1.000`; never below `0.898` (L) | A `0.912` (CI lower `0.515`, also fails `settled_rate`/`loss`), L `0.898`, C `0.976`, D `0.951`, M7 `0.975` | **Shipped default.** Most consistent floor of the five and the only mode at `1.000` on both B1 and B2; wins few cells outright because classic/rtt-threshold edge it on stable-link scenarios |
| **classic** | 8 — C, D, G, H, K, M1, M4, M5 | C `1.000`, D `1.000` (Starlink spike/dip), H `1.000`, M1 `1.000`, M5 `1.000`; F/G/K/I ties at `1.000` | **B1 `0.484`**, B2 `0.850`, A `0.749`, L `0.759` — collapses on heterogeneous bonds (fails `settled_rate` on both B cells) | Deleted. Highest win count but the worst tail: half of B1's goodput on the scenario a bonding sender exists for |
| **rtt-threshold** | 5 — A, E, F, M2, M7 | A `1.000` (CI lower `1.000`, the only mode passing all but the zero-drop gate on A), E `1.000`, M2 `1.000`, M7 `1.000`, F `1.000` | **B1 `0.480`**, B2 `0.866`, G `0.825`, L `0.824` — the fast/slow split starves the slow link exactly when it is needed | Deleted. Best on homogeneous loss (A) and RTT-ramp (M2, M7); same B1 collapse as classic |
| **edpf** | 0 | F `1.000`, I `1.000`, M6 `1.000`, M2 `0.999`; never a sole winner | **G `0.356`**, B1 `0.485`, L `0.730`, C `0.749` (also fails `nonrecovery_rate`), D `0.760`, H `0.810`, K `0.823` — the measured-send-rate predictor is not a capacity oracle | Deleted; pipeline retained as regression evidence only. Worst single cell in the matrix (G) |
| **adaptive** | 4 — I, L, M3, M6 | L `1.000` (overload/idle burst, the only mode above `0.898` there), I `1.000`, M3 `1.000`, B1 `1.000`, G `1.000`, K `1.000` | **A `0.666`** (CI lower `0.461`), C `0.828`, B2 `0.941`; the pre-existing G-demotion and twin-health privileged failures were never resolved | Deleted. Its health/admission/probe machinery survives as the shared layer Enhanced uses; its private ranking did not earn a place |

Reading the table honestly: no mode covered anything under the frozen rule, so the
"pros" are relative goodput medians, not passes. Enhanced was chosen as base because
the rule's tie-break prefers it and because its worst cell (`0.898`) is the least bad
worst cell of the five; classic and rtt-threshold win more cells but each halves B1.

### 12. `src/sender/selection/` is fork-owned from 4.0.0

From `4.0.0` the scheduler directory is **fork-owned**. Upstream `irlserver/srtla_send`
scheduler changes are **triaged** in the upstream-sync evaluation note of the merge that
brings them in and, where adopted, re-implemented against the shared layer as
individually gated commits — they are **never merged** into `selection/` by conflict
resolution. The upstream five-mode surface no longer exists here to merge into.

### 13. The CeraUI reader (SPIKE-CERAUI-READER) and CeraUI mode spike

- **Reader strictness: PASS.** The frozen `@ceralive/srtla` Zod `telemetrySchema`
  uses plain `z.object` (not `.strict()`); executed under Bun against
  `schema_version: 1` documents carrying `receiver_nak_report: true` and `false`,
  both parsed and the unknown key was **silently stripped, not rejected**. The
  optional field is therefore emitted unconditionally; no compile-time gate is
  needed. The consequence — a stripping reader loses the field with no error — is
  why the `@ceralive/srtla-send` reader declares it in producer order and the
  byte-parity round-trip suite exists.
- **Mode spike: PASS.** CeraUI's real spawn passes listen port, host, port, ips
  file, stats file, control socket, exec path and bind-map args — **no `--mode`**,
  and no `set-mode` control write anywhere in CeraUI or the image templates. Mode
  deletion is therefore invisible to the device integration.

### 14. The FEC `arq:onreq` limitation

The M4 FEC pair (`packetfilter=fec,cols:10,rows:5` on both ends, Enhanced/M4, N=3:
`8,000,578` vs `7,614,902` bps, ratio `0.951794`) is a no-regression check on the
filter, **not** a test of the periodic-NAK gate. libsrt's FEC filter defaults to
`arq:onreq`: a loss the filter itself confirms as unrecoverable is requested by the
filter's own path and **bypasses the periodic-NAK gate**. A gated receiver with FEC
enabled therefore still emits filter-confirmed loss reports outside the gate; the gate
governs only the timer-driven re-report. Nothing in this ADR claims FEC + gate was
validated together.

## Consequences

- **Public surfaces (all additive, `schema_version` stays 1):**
  - telemetry file/event: optional top-level `receiver_nak_report` after
    `disposition`; omitted when unknown;
  - `get-status`: `receiver: {nak_report?, srt_version?, rexmit_flag?}` (`{}` before
    any HSRSP), optional `negotiated_latency_ms`, and per-link `rexmit_forwarded`
    in `links[]`;
  - status log: `receiver: nak_report=on|off|unknown srt=<version|unknown>`,
    per-link `rexmit_fwd=` and `premature_naks=`.
- **Breaking (4.0.0):** `--mode` accepts only `enhanced`; `set-mode` rejects retired
  values with `-32602` / `error.data.kind = "retired_mode"`; `--no-quality`,
  `--exploration`, `--rtt-delta-ms` and the four stall options are accepted, ignored,
  and warn once each; `SCHEDULING_MODES` in the TS sender binding narrows to
  `['enhanced']`. `docs/release-notes-4.0.0.md` carries the migration.
- **Rollback** is reinstalling the released `3.3.0` `.deb`; the device integration
  passes no mode, so nothing else changes.
- **Open, tracked, not waived:** the hardware canary is pending; the privileged
  `netns` G-demotion and twin sustained-health assertions remain red as history; B1/B2
  delivery deficits under the strict gate are unresolved; NAK-off blindness on G is a
  known limitation; a receiver-side `arq:onreq`+gate interaction is untested.
- **Do not re-run** the receiver-lineage negotiation idea, per-mode private signal
  views, or the R-bit loss attribution without a new ADR that addresses the structural
  reasons above.
