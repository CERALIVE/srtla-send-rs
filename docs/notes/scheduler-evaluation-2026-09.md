2026-09-18 — classic/rtt-threshold/edpf/adaptive modes removed in 4.0.0 — read as history.
# Scheduler evaluation — September 2026

## Final Todo 30 disposition — scoped completion with documented findings

The owner ended the single-run investigation on 2026-09-15 and authorized scoped
completion, following Todo 29's distinction between delivered evaluation work and
performance acceptance. **The literal "every target passes" bar is not met.** Nine
non-adaptive targets have historical privileged-pass evidence; `netns_adaptive`
retains four separately characterized findings: D coupling (explicit ignore),
Twins preferred-share ceiling (informational), Twins full-rate health (baseline
health passes, earlier current failures unresolved), and G's wide demotion variance
(deferred to Wave 6 / Todo 32's N=5+ campaign, not more single-run diagnostics).

I's restored blocking precondition passes and is closed as a non-issue for Todo 30;
that scoped decision does not invent a cause for the earlier un-reproduced failures.
The genuine priority-snapshot fix `361d644` stays. No scheduler code or additional
assertion behavior was changed during closure. The unauthorized weakening in
`58e7207` is reversed, not endorsed. No fresh ten-target gate pass is claimed.
See the final Todo 30 subsection under KNOWN LIMITATION below for exact numbers,
the Twins baseline comparison, gate provenance and the limits of each conclusion.

## Intermediate correction record — 2026-09-15, before final owner disposition

This status supersedes the completion claim in `b8e809c`; earlier sections below
retain their historical run context. `361d644` (immediate shared snapshot refresh
after pool-control RPC application) is retained. `58e7207` is reversed in the
working tree, without a new commit: I's `before_healthy` requirement is blocking
again and retains the legitimate `17 <= t < 20` settling window. No assertion
threshold, production code, or frozen `src/connection/wire_rate*` file was changed.

### I: required wire-budget capture performed; earlier failure not reproduced

One fresh isolated exact-filter run on the pinned nightly passed with the restored
assertion. The harness already launches the sender with housekeeping at `debug`;
the outer command's `RUST_LOG` is not relied upon because the sender uses `env -i`.
The receiver was the explicit CeraLive `srtla/build/srtla_rec` build, not the
PATH-selected binary. The test used a 180s timeout and one test thread.

- Receiver process restart: **221.092875ms** (readiness-inclusive operation:
  2.276443712s; these are distinct measurements).
- REG3 recovery: **17.455938485s** after process respawn, within 18s.
- Sink recovery: **19.773828501s**, within 25s.
- `t=15..20s`: **26 samples**, sink range **12.707296–12.907328 Mbit/s**.
- Blocking `t=17..20s`: **15 samples**, all above **11.52 Mbit/s**;
  range **12.738880–12.886272 Mbit/s**, `pre_healthy=true`.

The final-five-second housekeeping events are below. Both links report the same
**11,313,708.49898476 bps** budget and **`DemandLimited`** phase in every row,
with socket generation 0. UTC is retained verbatim rather than presenting a
sub-millisecond event/measurement-clock alignment the harness does not record;
these fall at approximately t=15.8, 16.8, 18.8 and 19.8s. The logger's rate-tick
guard can omit an adjacent one-second tick; no missing rows are interpolated.

| UTC (2026-09-15) | `lte-a` attempted_wire_bytes | `lte-b` attempted_wire_bytes |
| --- | ---: | ---: |
| 05:00:21.712 | 21,011,144 | 20,992,680 |
| 05:00:22.712 | 21,831,852 | 21,809,632 |
| 05:00:24.712 | 23,469,040 | 23,448,624 |
| 05:00:25.712 | 24,279,312 | 24,278,656 |

`attempted_wire_bytes` is the cumulative kernel-accepted byte counter exposed by
`BatchSender::wire_sample` / `WireBudget::totals`, **not bytes in that single
housekeeping interval**. `target_bps` is the separate soft RateCap target, not the
hard wire budget; substituting it would produce the wrong attribution.

The sink measurement was also traced: `tests/netns_adaptive/observe.rs` sums the
latest ten complete 100ms UDP sink buckets and multiplies bytes by eight, producing
a trailing one-second useful-receiver rate. It is not telemetry wire bitrate or an
average over the whole three-second settling window. The assertion requires every
sampled trailing rate in that window to meet the target. Overlapping sampling does
not itself prove brittleness or authorize relaxing the requirement.

**Attribution limit:** neither low sink nor low/Draining wire budgets reproduced
in this run. It does not explain the historical failing runs, prove permanent
reliability, or establish that their root cause matches scenario A. Consequently
the KNOWN LIMITATION section's confirmed scope is **not extended to I**.

### G: high demotion reproduces twice in fresh isolation — review required

Six empty leftover test namespaces were removed before the I run. No test
namespaces/processes remained after I or between the two G runs; noninteractive
sudo worked, with no test sudo-heartbeat process. An unrelated systemd-user receiver
in the host network namespace was preserved, not mistaken for a leaked test process.
Each G invocation was an independent exact-filter run with the shared host
measurement lock, explicit CeraLive receiver, a 180s bound and one test thread.

| Run | Marginal netdev bytes | Demoted snapshots | Result |
| --- | ---: | ---: | --- |
| G1 | 3,778,380 | 29/61 (47.54%) | FAIL: unchanged ≤10% demotion requirement |
| G2 | 3,535,562 | 44/61 (72.13%) | FAIL: unchanged ≤10% demotion requirement |

Both satisfy the 3MB carriage requirement and 55-snapshot minimum. Neither resembles
the previously reported 0/61 isolated band. High demotion is therefore reproducible
in fresh test isolation, not adequately explained by accumulated test namespaces or
processes. This is a real unresolved failure; its cause, and whether a particular
code change induced it, are **not established**. No scheduler fix was attempted.

**At this intermediate point the owner's Step 3 stop boundary was reached.** Twins' reported failure of "both
links must remain Healthy after full-rate restoration" has not been compared
against `d168aa6` in this correction round. Its pre-existing/regression status is
unproven, and even a confirmed pre-existing failure still needs consultation rather
than an automatic waiver. D's existing documented ignore and Twins' accepted
priority-share ceiling disposition remain distinct from this unresolved health
requirement. The full adaptive target and ten-target gate were not run; there is
no new full-gate result or completion claim, and no new commit.

Retained local evidence bundle: `todo30-attribution.9xPKZn`, containing `run.log`,
`g1.log`, `g2.log` and harness directories `adpt_s_eb6b_0` (I), `adpt_s_89f9_0`
(G1), `adpt_s_fcd4_0` (G2), each with observations, sink buckets and sender logs.
The following endpoint comparison and final disposition supersede this intermediate
record's pending-work/commit status, not its measurements.

## Recovery-load scope (Todo 28, round 8)

Adaptive mode has no aggregate admission or source-backpressure mechanism; an
offered-rate jump above currently admitted/ramped capacity can cause real loss
and repeated health demotion during bond recovery. Recovery to Healthy is only
guaranteed under offered load the currently-admitted link set can sustain — the
bounded verification is in `tests/netns_adaptive.rs`.
Full-rate-restoration-during-recovery is a documented residual risk, not a passing
assertion. The revised tests must pass before this necessary load condition is
described as a verified recovery guarantee.

This scope correction follows four oracle consults and real kernel observations,
not detector-threshold tuning. The round-6 twin capture measured survivor TBF
backlog 49,082 B / 51 packets / 0 drops at t=28.268907560s, then 607,908 B /
494 packets / **929 drops** at t=29.186322215s after restoring 12.8 Mbit/s onto
one 8 Mbit/s carrier. Parent and child qdisc counters represent the same nested
backlog and must not be summed. Round 7 logged survivor Healthy→Degraded at
08:09:48.697749Z and recovering twin Rejoining→Degraded at 08:09:52.698742Z,
both attributed to normal DATA loss rather than queue entry.

The blocking D test holds 6.4 Mbit/s from t=20 through the t=28 restoration and
17-second recovery deadline, then restores its original **22.4 Mbit/s** at t=46.
Twins hold 6.4 Mbit/s through their 9-second deadline, then restore 12.8 Mbit/s
at t=38. Both preserve bounded Stalled→Rejoining→Healthy, reject relapse, and
separately check health and the original steady-state shares after full load.
`immediate_full_rate_restoration_stress` retains the old schedules as an explicitly
ignored diagnostic recorder, with no passing health/share assertion.

Scenario G is independent: its marginal-link demotion criterion is not excused by
this recovery-load scope finding. Todo 28 remains open pending real verification.

## Round-8 verification: scope correction is not sufficient

Final real-sudo runs, with temporary instrumentation removed, no scheduler overrides,
and each scenario individually bounded at 150s:

| Scenario | Result | Measurements |
| --- | --- | --- |
| D | FAIL, 92.58s | Rejoining +7.500220s; Healthy +15.591553s ≤17s; no Stalled/0 in the detection window; loses Healthy after full load; final Starlink share 45.6006% |
| G | FAIL, 79.85s | 3,937,450B ≥3MB; 45/61 demoted snapshots (73.7705%), 18 Degraded /27 Stalled /16 Rejoining /0 Healthy |
| I | PASS, 77.54s | Process restart 188.885090ms; REG3 +17.208207s; sink90 +19.621732s |
| Twins | FAIL, 118.00s | Stall +1.452654s; Rejoining +15.284371s >9s; no Healthy; final preferred share 55.6029% passes but health does not; first priority-set snapshot omits the applied key |

The twin diagnostic identifies a separate freshness-timing rejection during the
**feasible-load** period. At 08:36:45.267593Z it reports two successful probe
rounds, oldest start 1789375002570ms, current time 1789375005267ms (age **2697ms**),
sRTT 60.236422ms, one held link, and zero queue delay. The existing policy allows
only `max(2τ, 2×train_period+sRTT)` = **2060.236422ms**. Thus the oldest-start
freshness predicate rejects the completed evidence. Repeated successful pairs
remain too old at subsequent housekeeping observations. Kernel band-one qdisc
drops are zero through t=28..37s, with 0–1 queued packets: this interval is not
the earlier full-rate overload. No recovery-policy change was made under the
test-only D/twin scope; this newly exposed gap needs a regression and a separately
verified timing correction, not an unbounded wait or a relaxed deadline.

## G: real overflow, not a GE-loss threshold exemption

The actual installed kernel GE probabilities are `p=0.05, r=0.001,
1-h=0.008, 1-k=0.0002`. All four builder values are percentage points. The
stationary bad-state probability is 50/51, giving mean model loss
`(50×0.008 + 0.0002)/51 = 0.0078470588`, or **0.78470588%**.
See [tc-netem's GE parameter definitions](https://github.com/iproute2/iproute2/blob/main/man/man8/tc-netem.8).
This is a model calculation, not a measured normal-DATA loss EWMA.

The first instrumented run carries 4,229,512B but is demoted in 44/61 snapshots.
From t=0.045845007 to 59.258063697s, the marginal netem child accumulates **61,113
total drops**, versus **530 model-loss drops**. Its peak observed backlog is
669,900B /499 packets; multiple samples hit the 500-packet limit. Model and total
drops must not be confused, nor nested parent/child counters summed.

A falsification control removes only the marginal link's GE model and sets its
random loss to zero, leaving capacity, delay, source rate, siblings and all
production mechanisms unchanged. It still carries 3,659,996B with **36/61 demoted
snapshots**, and incurs **43,690 total qdisc drops with zero model drops** over
t=0.023634538..60.043909353s. Peak queue: 637,162B /500 packets. Normal-DATA-loss
demotions persist (08:40:06.361547Z, 08:40:25.361162Z, 08:40:46.360390Z).
The control was reverted before the final gate; it is not substituted for G.

The premise that G has three already-Healthy companions is also falsified by
telemetry: all four links are Degraded in the first instrumented measurement
snapshot. In the final original-profile run, lte-a is Degraded in all 61 snapshots;
lte-b has 51 Degraded /1 Stalled /9 Rejoining, and lte-c has 54 Degraded /7 Rejoining.
None has a Healthy sample. The marginal's final transitions are loss-labelled
demotions and DATA-proof stalls, not queue-entry-labelled demotions.

**Conclusion:** removing GE loss does not remove the failure. Real overflow and
bond-wide demotion occur without any scheduled blackhole/restore event. This does
not prove the settlement ledger has no other edge case, nor isolate the first
scheduler decision that starts the collapse. It does disprove the proposed
“genuinely marginal GE loss makes ≤10% unreasonable” explanation. Do not relax
that assertion. Per-link allocation, startup/settling and recovery feedback need
further causal isolation; no new production fix or integration-success claim is made.

Default library: 826 passed /0 failed /1 existing ignored. Full all-features suite
passes with privilege escalation disabled (library 851 /0 /1); its netns self-skips
are not privileged evidence. Release build, production and target Clippy, formatting,
and six touched/restored Rust-file LSP checks pass. Frozen earned-ACK,
stall-deselect and EDPF traces and the protected twin test remain byte-unchanged
against origin/main. No commit condition was met; Todo 28 stays unchecked.

## Round 9 — sampled freshness fixed; G startup allocation isolated further

### Recovery acquisition versus observation

Twins contains two uplinks, but only **one held/probed link** during feasible-load
recovery. `m×10/10×1000` therefore gives a1000ms train period, and
`max(2τ,2×period+sRTT)` gives2060.236422ms. The units and held count were correct.
What was missing was the **housekeeping observation phase**: a pair can complete
within its acquisition budget and expire before the next1Hz evaluation sees it.

The failing-first regression drives the actual ProbeScheduler and ProbeLog:
20opportunities105ms apart, every ACK60ms later; two trains complete at2055ms,
then housekeeping observes at2697ms—642ms later. Before the fix it remains
Stalled; after the fix it enters Rejoining. `HealthSignals.observation_interval_ms`
supplies the actual scheduled cadence (1000ms at runtime,0 for immediate policy
evaluation), added **only to recovery evidence freshness**. The acquisition formula,
probe ACK deadlines, epoch/future fencing, qualification counts, degradation timers
and conservative ramp are unchanged. No delayed-execution/unbounded grace is used.
Negative tests reject stale evidence at3061ms, evidence from the previous Stalled
epoch, and a second train with only four ACKs.

Live twins now reach Healthy before the unchanged9s deadline: diagnostic
Rejoining+2.673421s/Healthy+4.597722s; final uninstrumented
Rejoining+3.071572s/Healthy+5.086991s. Both preserve the feasible-load no-relapse
assertion. **The later full-rate health/share failure remains separate.**

### G: exact load arithmetic and the first queue growth

G offers12.8Mbit/s of1316-byte application payloads. The accepted-prefix capture
confirms1332-byte original SRT DATA packets: baseline12.9556Mbit/s, still less
than the companions'15Mbit/s even if the marginal contributes nothing. Network
framing and the small control/probe load do not by themselves consume that gap.
Thus the proposed “80% aggregate assumes the marginal is always available”
explanation does **not** establish overload.

Temporary cumulative counters distinguished accepted original DATA, R-marked
retransmits, probes and control; separate queued-intent counters identified ranked,
sole, fallback and pre-registration routing. Counters were read as deltas, never
summed across successive cumulative snapshots. Early per-link sample windows are
not perfectly aligned; their exact endpoints are retained in the task evidence.

| Link/cap | First ~1s original | First ~1s retry | Subsequent ~2s original | Subsequent ~2s retry |
| --- | ---: | ---: | ---: | ---: |
| marginal/1Mbit |1.862035|0.414810|1.178493|0.837060|
| LTE-A/5Mbit |3.369846|1.102031|4.426082|1.575574|
| LTE-B/5Mbit |3.739653|1.065600|3.642667|2.602667|
| LTE-C/5Mbit |3.883205|0.425834|3.769248|2.145321|

Rates are Mbit/s of kernel-accepted SRT DATA, not successful egress or useful
goodput. All early routed bytes are ranked admission; no pre-registration,
sole or fallback bytes occur in these intervals. The first marginal interval
already assigns more than twice its capacity while companions remain below5Mbit.
Later original+retry companion rates exceed5Mbit each **before health demotion**.

The synchronized warmup kernel capture records marginal queue growth:
09:15:25.290Z219,032B/168packets,6totaldrops—all6model loss;
09:15:26.185Z503,616B/399packets,8totaldrops—all8model loss. This particular
initial marginal episode is therefore **queue delay**, not proven buffer overflow.
By09:15:26.772219Z marginal sRTT is1552.821621ms and queue delay665ms while
loss EWMA is0. At09:15:28.772145Z it demotes on finalized normal DATA loss
(EWMA0.2654544212); the companions demote on queue delay afterward.
At the end of warmup, LTE-A's child qdisc reports12,891totaldrops versus40model
drops, LTE-B582/9, and LTE-C93/4. These are nested qdisc counters, not independent
root/TBF/netem drop populations to sum.

**Verdict:** the declared source is feasible, but initial per-link allocation and
subsequent retransmission feedback are not. It is not evidence that healthy
companions drop while each is below its actual wire capacity. Nor is it a proof
of held-link routing leakage. The first small source-code defect, if any, is not
yet isolated by a behavioral toggle; startup capacity discovery, per-link pacing
and ranking versus hard admission remain a control-design question. No G policy,
scenario definition or≤10%assertion was changed. Do not describe this as a passing
scope exception or deploy an unverified controller adjustment.

### Final clean-code verification

| Scenario | Result | Measurements |
| --- | --- | --- |
| D | FAIL,89.80s | No timely Stalled/0; Rejoining+1.867734s,Healthy+10.866317s≤17s; laterfull-ratehealthfails;keepalive6→6 |
| G | FAIL,75.54s |4,364,622B;35/61demoted (15Degraded/20Stalled/26Rejoining/0Healthy);companionshavezeroHealthy samples |
| I | PASS,74.35s |restart134.637883ms;REG3+17.547182443s;sink90+19.776561765s |
| Twins | FAIL,112.75s |boundedrecoveryPASS;bothprioritysnapshotsPASS;preferred47.5879%andlaterHealthyfail;legacysecond140B |

All four ran under real sudo with150s bounds after temporary diagnostics were
removed. Default library829passed and all-features library854passed (+3guards,
zero failures); the full all-features suite passed with privilege escalation
disabled and netns self-skips explicitly distinguished from the live runs.
Release,Clippy,formatting,LSP and unchanged frozen traces passed. Todo28 remains
unchecked; the partial freshness fix is retained, not represented as full success.

## Round 10 — retransmission accounting hypothesis not confirmed

### What retransmissions actually do

NAK reception parses loss sequences and forwards the original NAK to the local
SRT caller. `handle_nak` changes congestion/loss bookkeeping but sends no DATA.
The local SRT implementation retransmits; its R-marked packets re-enter
`handle_srt_packet` → adaptive selection → `forward_via_connection` → normal
queue → `flush_batch`. There is no separate NAK-side uplink retransmit path.

| Accounting surface | Treatment of R-marked DATA |
| --- | --- |
| BitrateTracker/session bytes | Every queued retry contributes its complete packet length |
| Queued score denominator | Every queued normal-path datagram contributes, including retries |
| Flight after acceptance | The retry is registered; entries are keyed by sequence, not physical copy |
| DeliveryLedger | Every accepted retry increments attempts and inserts/refreshes its sequence entry |
| Delivered rate | A valid arrival/generation-exact ACK credits that entry's wire length, including R-marked DATA |
| Karn eligibility | Disables ambiguous RTT sampling, not delivered-byte credit |
| RateCap | Reads the same ACK-derived delivered rate; retries are not filtered out |

Four executable checks pass against the existing production implementation:

1. Normal-only and R-only loads of256distinct1332-byte packets each produce
   **340,992B,256flight entries,256attempts,score77,soft-cap multiplier0.125**.
   ACKing them yields **1,363,968bit/s** delivered rate and bootstrap target for
   both forms; the transmitted-rate calculation agrees on the fixed2s window.
2. An original and retry of one still-outstanding sequence physically send
   **2664B** and count **2attempts**, but one flight entry. One sequence ACK plus
   its replay credits only1332B, yielding5328bit/s over2s.
3. NAK bookkeeping emits no payload to the peer. A subsequently supplied R-marked
   packet re-enters byte/flight/delivery accounting normally.
4. The production packet handler selects the admitted link for both original and
   R-marked input, despite a higher-scoring held link with an active cooldown;
   the chosen peer receives the original R flag unchanged.

These tests add evidence and regression coverage, **not a new production fix**.

### Important limitation, not a hidden counter fix

`DeliveryLedger` measures proven delivery, not total attempted wire expenditure.
Failed attempts and multiple unresolved copies cannot each earn credit from one
ambiguous sequence ACK. Flight is likewise a logical unresolved-sequence census,
not an exact count of every copy still buffered in the network. `RateCap` does not
read BitrateTracker's sent rate; that separation was deliberate in Todo20, and its
positive ranking multiplier is not a pacer for original or retransmitted traffic.

Thus **no R-specific bypass was demonstrated**, but this is not a claim that the
current proxies constrain total wire usage or accurately model physical queue
occupancy under retry amplification. Adding sent-byte pressure/pacing or changing
copy accounting would be a controller decision requiring its own specification
and causal proof—not filling in an accidentally omitted retry counter.
G's≥3MB/≤10%assertions and scenario definition remain unchanged.

### Final round-10 privileged results

| Scenario | Result | Measurements |
| --- | --- | --- |
| D | FAIL,93.02s | Stall+1.406478s passes; Rejoining+1.502634s; Healthy+17.655895s misses17s; keepalive-log count6→6; laterfull-ratehealthfails |
| G | FAIL,79.38s |3,576,202B passes;41/61demoted=67.2131%,all41Degraded;20Rejoining,0Healthy |
| I | PASS,76.80s |process150.748816ms;REG3+17.340518518s;sink90+19.648879754s |
| Twins | FAIL,117.30s |Stall+2.582163s;Rejoining+2.404688s,Healthy+4.554603s≤9s withoutlow-loadrelapse;bothprioritysnapshotsPASS;laterhealthand25.8724%preferredshareFAIL |

All four ran once under real sudo,150sboundeach,zero skips/timeouts. The full
all-features suite passed with escalation disabled (netns self-skips explicitly
not treated as live evidence); library833/858passed, four new coverage tests and
zero regressions from829/854. Release,Clippy,fmt,LSP and frozen traces passed.

**Gate clarification:** round8 explicitly required a separate post-restoration
Healthy/share assertion and those gates remain blocking. Only the old
immediate-restoration stress recorder is non-blocking. No retrospective change was
made to that contract. Even removing those later gates would leave D's bounded
recovery/keepalive proof and G failing in this run. Todo28 stays unchecked pending
the user's final decision; no commit or residual-risk acceptance was inferred.

### Retained fixes and mechanisms across all ten rounds

| Area | Retained work | Verification boundary |
| --- | --- | --- |
| Adaptive RTT attribution | Exclude cumulative SRT ACKs from per-link RTT; Karn exclusion for R-marked/repeated outstanding copies; use accepted-send ledger timestamps even after congestion-log pruning | Failing-first dispatch/wire regressions; legacy traces unchanged |
| Probe recovery bookkeeping | Pending new trains no longer erase preceding qualified rounds | Regression-tested; actual failed/expired trains still break qualification |
| Recovery loss epoch | Qualified Degraded/Stalled→Rejoining starts a new normal-loss epoch | Old outage loss cannot immediately contaminate fresh traffic |
| Receiver registration | A validated changed receiver full ID invalidates old-ID awaiting-REG3 grants | Regression plus repeated real receiver-restart success; REG_NGP acceptance unchanged |
| Sole-carrier recovery | Passive original-DATA witness can qualify Stalled recovery without forbidden self-probes | Two ten-packet groups/fiveACKs; generation/epoch/freshness and negative guards |
| Temporal NAK attribution | Charge accepted-send cohorts, bounded late correction/history, not feedback-arrival denominators | Late/subfloor/cross-epoch regressions; no loss-ratio clamp |
| Resolved loss debits | Exact pending debits survive retries and receive one arrival/generation/epoch-exact ACK credit | Duplicate/cross-link/stale/expired credits rejected; raw congestion penalties preserved |
| Deadline settlement | Separate cohort closure from settlement; accepted-send latency deadlines freeze per attempt; closed-unsettled credit allowed strictly before deadline | Negotiated-latency/retry/deadline-boundary regressions; not blanket eventual-delivery forgiveness |
| Queue-entry persistence | Require continuous threshold crossing for latchedτ before Healthy/Rejoining queue demotion | Precedence/RTT-latching/continuity tests; clearance and actual-relapse backoff unchanged |
| Sampled freshness | Add actual housekeeping observation interval to recovery freshness, not ACK deadlines or ramp | Failing-first2697ms case, stale/pre-epoch/fourACK guards, repeated live bounded twin recovery |
| Harness correctness | Measurement timestamp after netdev collection; aggregate registration counts; sink recovery after observed impact; process-only restart timing; default telemetry cadence; identity-keyed priority snapshots | Prevents false timing/registration/sink claims; intermittent first-priority-publication gap remains unresolved |
| Test scope/observability | D/Twins feasible load through deadlines; old schedules retained as nonblocking stress; suppress per-NAK congestion flood in test launcher | Complete health/share gates still red; not an engine-convergence fix |
| Round10 | Four wire/dispatch accounting checks documenting unchanged retry semantics | No new runtime fix or weakened assertion |

All nine earlier rounds' work is preserved. Correct component mechanisms and
passing unit gates do **not** constitute full bonded integration or hardware
validation. G startup allocation/retry feedback, D's remaining timing/proof
failures, and later full-rate health/share stability remain open.

## Round 11 — hard attempted-wire enforcement; acceptance still fails

Oracle consult5 rejects applying the D/twin load-scope correction to G. A feasible
steady-state allocation exists at unchanged12.8Mbit/s; retry amplification is an
endogenous control failure, not a legitimate overhead calibration. A per-link
send/dequeue budget is required; another ranking multiplier or bond-only cap is
insufficient. G's≥3MB and≤10%assertions remain unchanged.

### Implemented mechanism and its explicit assumption

The sender has no configured physical-capacity input. This experiment uses each
unchanged `RateCap.target_bps()` as the **estimated rate to enforce**, without
modifying RateCap or reading test topology/qdisc limits in production.
`WireBudget` is an independent monotonic byte-credit bucket with a3000-byte
(two-MTU) maximum burst. With constant rate r, its admission bound is
`bytes ≤ r×elapsed/8 + 3000`, not a promise of exactly r in every short interval.
Credit accrues under the old rate before a rate change; repeated configuration
never replenishes spent credit. Idle accumulation is bounded.

`BatchSender::flush` submits only the funded prefix. Every kernel-accepted queued
datagram spends its full UDP-payload length—original DATA, R-marked DATA, probes
and queued SRT control alike. Kernel-rejected/unaccepted suffixes do not spend
credit. An unfunded prefix produces no socket error and stays queued; existing
accepted-prefix registration/delivery/loss semantics remain unchanged. SRTLA
registration and keepalive control stays on its original separate control path.

Before enqueue, credit includes already queued reservations. A depleted preferred
link yields to another funded link among the existing admitted weights, overriding
cooldown but not health/deadline exclusion or sole-carrier retention. With no
funded admitted carrier, the handler returns a distinct Backpressured outcome.
The event loop retains one datagram, pauses local reads, and uses a1ms wakeup to
drain queues/retry while ACKs, timers, control and signals remain serviced. This
preserves the logical pool; no budget-specific “no available connection” drop is
introduced. It bounds user-space buffering, **not** producer-side UDP loss or
queue residence time. Kernel receive buffers can still fill.

Legacy modes disable the gate. Adaptive enforcement is always on, with no new
feature flag; the existing eight `AdaptiveFeatures` bits/defaults are unchanged.
Consequently their ablations do not disable this new wire gate. Diagnostic DEBUG
records expose configured budget and cumulative accepted UDP-payload bytes within
the current enabled budget epoch; neither is added to frozen stats-file JSON.

### Failing-first and invariant evidence

Before dequeue enforcement, the real UDP test delivered **812,520B**, including
13,320B probes, against a5Mbit/s one-second bound of **628,000B including burst**.
After enforcement, its deliberately coarse10ms service loop delivers266,400B and
retains the unfunded suffix. A separate continuously backlogged1ms test delivers
**627,372B** within that bound, proving the gate is not passing by near-zero output.

Two production-handler tests first failed: the exhausted preferred link retained
selection, and an all-exhausted packet was marked Consumed. They now pass: another
funded admitted link is selected, or the packet is deferred without queueing,
ownership insertion or byte-accounting duplication, then accepted after refill.
Additional guards cover rate-change credit preservation, bounded idle credit,
and a failed kernel send retaining credit for the unchanged queue. Six new tests.

### One real privileged run per unchanged scenario

| Scenario | Result | Measurements |
| --- | --- | --- |
| D | **PASS**,93.19s |τ1s;Stalled+1.433390s;Rejoining+2.466875s;Healthy+4.504822s;keepalive1→2;finalStarlink28.2318% |
| G | **FAIL**,78.83s |4,369,830B passes;31/61Degraded=50.8197%;30Rejoining,0Healthy on marginal |
| I | **FAIL — throughput regression**,74.99s |restart156.365719ms andREG3+17.619489262s pass; pre-restart sink2,684,640bps; no90% sink recovery |
| Twins | **FAIL**,116.06s |Stalled+2.542299s;Rejoining+2.463377s;Healthy+4.415677s;low-load recovery and finalhealth pass; preferred52.5257% fails; bothfirstRPCsnapshots stale |

No skips/timeouts;150s bound each. D's τ is smaller because observed RTT is now
lower; its unchanged formula therefore checks a9s recovery limit, not the former
17s limit. D passing does not prove full-rate throughput: its final15s combined
netdev carriage is only7,177,968B against the declared22.4Mbit/s offered load.

G's three companions are **Healthy in all61 snapshots**. Budget DEBUG counters
give these peak sampled UDP-payload attempted rates and maximum estimates:

| Link | Peak sampled attempted rate | Maximum configured budget |
| --- | ---: | ---: |
| LTE-A |2.908096Mbit/s|2.913461Mbit/s|
| LTE-B |2.222816Mbit/s|2.224517Mbit/s|
| LTE-C |2.855872Mbit/s|2.856335Mbit/s|
| marginal |1.119520Mbit/s|1.104081Mbit/s|

Marginal's one-second sample exceeds its rate estimate by15,439bit/s, within the
24,000bit burst allowance, but both exceed its physical1Mbit/s capacity. The
mechanism enforces the estimate; the estimate is not an adequate physical bound.
All its demotions are queue-labelled. Final per-link qdisc total/model drops are
**23/23,30/30,24/24,33/33**: zero non-model queue drops on all four links in this
run. This is not sufficient grounds to relax G: only one seed was tested, its
budget still exceeded physical capacity, and demotions were queue-related, not
attributed to inherent GE loss.

I's two links are Healthy before restart, but sink goodput is only2.684640Mbit/s
at t19.849291541s. The maximum observed sink rate after t23s is2.768864Mbit/s,
well below11.52Mbit/s. The unchanged soft controller's bootstrap/target estimate
is too conservative when promoted to a hard sending limit; its code and the
receiver registration fixes were not altered. **This is a real new acceptance
regression, not an allowed receiver-recovery exception.** No further tuning was
performed after this live pass.

### D clarification and protected-file caveat

Round10's Healthy+17.655895s failure was the **bounded feasible-load recovery
assertion**, not the post-restoration assertion: restore28.075s, deadline45.075s,
first observedHealthy≈45.731s, full-rate step46.027s. It cannot be folded into the
old immediate-restoration stress exception. The6→6 keepalive counter failed a
separate in-blackhole proof obligation; it does not prove complete network silence.
Its checked subwindow was25.222..27.825s (2.603s), shorter than the approximately
4s keepalive spacing visible in the same log. A zero counter crossing in that
subwindow is therefore not sufficient to diagnose failed keepalive transport.
The later Healthy/share gate is separate and remains blocking in the actual code.
The exact split of the old655.895ms miss between FSM timing and observation delay
is not proven by an uncalibrated UTC/relative-clock alignment. The round10 logs do
show a16.000327s Rejoining→Healthy interval. Round11 now passes all D assertions
without changing them; do not retroactively relabel the old bounded failure.

All specified golden traces and protected twin topology/stack/mod/test files are
unchanged against origin/main. The **entire** twin directory is not empty against
origin/main: pre-existing commit `e222542` added optional priority publication in
`twin/publish.rs` for Todo9. This round changes none of that directory; reverting
that inherited feature merely to make the broad diff empty would discard prior work.

Pinned gate: default library839passed, all-features864passed, test-internals864passed,
zero failures (+6tests); bounded full suites pass with escalation disabled and
netns self-skips explicitly distinguished from the live failures. Release,fmt,
Clippy and12changed-Rust-file LSP checks pass (only pre-existing inactive-cfg hints).
G's complete test file is blob-diff identical to the pre-live source; scenario
definition, RateCap, BitrateTracker and feature defaults are unchanged.

**Stop verdict:** the hard gate is implemented and regression-tested, but its rate
source is not acceptance-ready. G remains red and I regresses. No assertions were
weakened, no second tuning iteration was attempted, no commit was made, and Todo28
remains unchecked pending independent review.

## Round 12 — active wire-rate search remains acceptance-red

A queue-bound active-search estimator now supplies the attempted-wire budget instead
of treating `RateCap.target_bps()` as physical capacity. Its deterministic tests cover
1/5Mbit capacity search, an8Mbit demand-limited path, original-DATA proof, queue
persistence, generation/counter rollover and30-second reprobe. Those tests pass, but
the first real G run exposed an absorbing under-rate failure: zero demoted snapshots,
yet only125,944B on the marginal link after the estimator fell from2Mbit/s to its
12kbit/s floor during one startup queue episode.

Temporary fields on the existing DEBUG wire-budget event proved the installed budget
matched the estimator; this was not a stale `BatchSender` configuration. RTT rose
887→1492ms against a60ms floor while the queue estimate rose78.5→716ms, and repeated
200–500ms epochs charged the same draining tail as multiple overload decisions.

A failing-first causal experiment attached the accepted-send timestamp to the latest
RTT sample and required each subsequent drain cut to use three post-cut, non-falling
queue observations. It made unchanged G pass with6,331,456B and6/61 demoted samples.
The same behavior failed D: the periodic15-second Starlink waveform drove its estimate
from2Mbit/s through250/125/62.5/31.25/15.625kbit/s before the t20 obstruction. D then
carried only7,042B on Starlink during the obstruction and39,524B in the final phase;
it never produced the required Stalled/0 or bounded recovery snapshots. The causal
experiment was therefore reverted rather than retained as a G-only success.

The other post-experiment real runs remain useful boundary evidence: I passes with a
157.252325ms process restart, REG3+17.32348342s and sink90+19.636725224s. Twins reach
Stalled+1.461837107s, Rejoining+3.573487249s and Healthy+6.602219795s under feasible
load, but fail the first priority-set snapshot and finish with49.0417% preferred share
instead of[55%,70%). Legacy control carries only70B on the second twin.

This is not a reason to relax G's≤10% assertion or the D/twin scope-corrected gates.
It is evidence that queue-driven capacity discovery needs a causal policy that handles
both sustained bottlenecks and short temporal waveform events without collapsing an
otherwise high-capacity path. Todo28 stays unchecked and no round-12 production policy
is accepted from the partial result.

## Rounds 13–18 — repeat-cut causality and remaining controller limitation

Oracle consult9 retained fast200–500ms exploration only before a generation's first
congestion cut and required1000ms evidence afterward. Round16 corrected the delayed-
window fixture by driving a real first cut rather than synthesizing private state.
Consult10 then ruled out the30-second CapacityHeld reprobe from retained runtime logs:
Starlink repeatedly descended2M→1M→500k→250k→125k while remaining Draining, including
without D's periodic waveform. Transition-only diagnostics captured the final
250k→125k cut with current original-delivery proof but only173,676.998bit/s accepted
against the250k candidate, while queue evidence fell10.5→6ms. Round17 therefore added
current-epoch proof,90% utilization, and non-falling queue requirements.

Consult11 found that round17 applied those requirements only while the phase was
literally Draining. `resume_search()` preserves`last_cut_ms`, so a resumed Searching
candidate could invoke the shared congestion arm and bypass the intended repeat policy.
Round18 centralizes authorization on`last_cut_ms`: the first cut remains immediate;
every later cut, in every phase, requires a due, proven, fully-used epoch and the same
continuous non-falling queue sequence. A failing-first Draining→Searching underuse test
observed500k→250k before centralization and remains500k afterward. Its positive counterpart
proves fully-used, proven, non-falling evidence still permits500k→250k. No threshold,
decrease factor, SEARCH_GAIN, MIN_RATE_BPS, CAPACITY_REPROBE_MS, RateCap, WireBudget,
health policy, admission policy, or scenario definition changed.

The centralized gate is a genuine correctness improvement, but it does **not** make D
deterministic. The complete bounded-recovery evidence from rounds13–18 is:

| Evidence | Stalled | Rejoining | Healthy | Bounded verdict |
| --- | ---: | ---: | ---: | --- |
| round13 | +1.836s | +2.568s | +4.656s | pass |
| round14 | absent | +1.803s | +4.790s | fail: missing Stalled |
| round15 | +1.593s | +1.710s | +4.678s | pass |
| round16 initial | absent | +3.732s | +24.738s | fail |
| round16 repeat1 (`tau=1.864s`) | +3.689s | +1.585s | +5.605s | pass |
| round16 repeat2 | +1.863s | +1.851s | +3.902s | fail: feasible-load relapse |
| round17 run1 | +1.771s | +1.770s | +4.797s | pass |
| round17 run2 | absent | +2.688s | +8.699s | fail: missing Stalled |
| round18 run1 | +1.938s | +1.825s | +3.833s | pass |
| round18 run2 | +1.600s | +3.485s | +5.529s | pass |
| round18 run3 (`tau=1.520s`) | absent | +1.766s | +3.858s | fail: missing Stalled |
| round18 run4 | +2.526s | +2.298s | +4.417s | pass |
| round18 run5 | +2.415s | +2.455s | +4.159s | pass |

Round18 run3 still reached125k, but this is no longer an authorization bypass. Its
logged cumulative accepted-byte deltas show the500k and250k candidates consumed their
self-imposed budgets; the centralized helper is the only path to every post-first
`congestion()` call, and its deterministic negative/positive controls prove a cut cannot
occur without due/proven/fully-used/non-falling evidence. This exposes the separate
controller limitation consult11 anticipated: consuming a tiny self-imposed budget does
not prove physical congestion on D's20Mbit Starlink. At125k a1332-byte packet yields only
about11.7 attempts/s, so32 attempts plus1Hz observation do not reliably fit the bounded
Stalled window. Changing the50% decrease, queue threshold, or search gain would be a new
controller-policy decision with diminishing causal confidence and is deliberately not
attempted here.

Round18 G remains clean at6,904,330B and0/61 demoted; its established rounds14–18
demotion series is0,5,8,0,0,0 out of61. I also remains clean: process restart214.666ms,
readiness2.219s, REG3+17.601s, sink90+19.715s. D's later full-rate health/share failure
remains the separately documented overload boundary; round18 final Starlink shares were
12.070%,7.031%,3.090%,7.285%,8.615%. The centralized fix is retained uncommitted as a
proven local correctness improvement, but Todo28 acceptance and commit remain a human
decision because the mandatory D5/5 criterion achieved only4/5.

**Orchestrator acceptance (2026-09-14):** D's4/5 bounded-recovery pass rate is
accepted as a documented residual risk, parallel to the round-8 post-restoration
boundary. The centralized repeat-cut gate from round18 is committed as a genuine
correctness improvement. Future work that could close this residual is either
(a) Todo32's pre-declared threshold sweep revisiting `stall_attempts` sensitivity for
links operating at low self-imposed rates, or (b) a dedicated, separately scoped
redesign of the wire-rate controller's decrease factor or minimum exploration depth;
neither is in scope for Todo28.

## KNOWN LIMITATION: Adaptive mode baseline-topology throughput instability (scenario A, discovered post-Todo-28)

### Todo 30 final findings: wider reliability scope, not a green adaptive gate

**Owner disposition (2026-09-15): scoped completion with documented findings.**
Evaluation, assertion restoration and the legitimate snapshot fix are delivered;
stable adaptive behavior is not certified. The investigation stops here. Wave 6 /
Todo 32's dedicated **N=5+ statistical C2 campaign** owns the next reliability work.
No new ignore, tolerance change or scheduler fix is introduced to obtain closure.

#### G — 0–72% demotion, wider than the earlier 0–13% band

The exact unchanged requirement is at least 3,000,000 marginal netdev bytes,
at least 55 fresh snapshots, and no more than 10% Stalled/Degraded snapshots.
Fresh isolated observations are:

| Revision / run | Marginal bytes | Demoted | Outcome |
| --- | ---: | ---: | --- |
| Current, correction G1 | 3,778,380 | 29/61 (47.54%) | FAIL |
| Current, correction G2 | 3,535,562 | 44/61 (72.13%) | FAIL |
| Exact `d168aa6`, endpoint comparison | 5,764,114 | 4/61 (6.56%) | PASS |
| Current, endpoint comparison | 7,106,018 | 0/61 (0%) | PASS |

"Current" is `b8e809c` with I's staged assertion restoration; G is unchanged.
The only post-`d168aa6` production-source edit is `361d644`'s snapshot refresh in
the pool-control receive arm. G's supporting harness, network-sim, dependencies and
toolchain are unchanged; G sends no priority RPCs. The 29/61, 44/61 and 0/61 results
occurred on identical current source. Both endpoint builds passed this comparison.
There is **no identified causal commit**; the owner classifies this as a deeper
scheduler-reliability characteristic, not a test-assertion defect or a regression
assigned to today's changes. The comparison did not instrument RPC-branch reachability.

Test namespaces/processes were absent before and after runs, noninteractive sudo
worked, and the same CeraLive receiver and SRT tool were used. No accumulated test
contamination was observed; other time-dependent host variance is not thereby
mathematically excluded. The earlier 0/61 ×3 and reported 0–13% band were based on
too few runs to capture the observed tail. Neither these counts nor the clean
endpoint pair estimate a reliable pass probability. The 44/61 observation is
**72.13%**, conventionally summarized as 72%, not an upper bound on future behavior.
More single-run diagnostic cycles cannot resolve this reliability question. G's
assertion stays blocking; deferral of statistical characterization is not a CI waiver.

#### Twins — baseline health passes; two different baseline failures remain distinct

The final authorized diagnostic ran only `twins_on_one_ip_bond_under_adaptive` at
**exact `d168aa6ea2a5ba5e1b3771a64c5015a3c4574a03`**, in a clean detached worktree,
using the pinned nightly/default test profile, explicit CeraLive receiver, one test
thread and a 180s bound. No test namespaces/processes remained before or afterward.
The full test **failed (exit 101, 112.67s)**, but not on health:

- Stalled at obstruction+1.835312746s; Rejoining at restore+2.863482797s;
  Healthy at restore+4.854316076s, within the 9s deadline.
- Full rate restored at t=38.071s. The shared "both links must remain Healthy
  after full-rate restoration" check **passed**, as did the final-15-second check.
  All **93** samples at t=45.047296826..60.080275532 had both links Healthy.
- LegacyControl bytes were `[10,464,756, 140]`: the unmapped second twin remained
  dead weight, as required.
- Failure 1: the first snapshot after the t=25.050s clear RPC still reported
  `modem-a.priority = Some(0.2)`, observed at t=25.911s instead of `None`.
- Failure 2: final preferred share **0.5010552614224213 (50.105526%)** failed the
  baseline's old `[55%,70%)` requirement; final netdev bytes were
  `[12,798,190, 12,852,326]`.

`Checks::require` accumulates failures and `finish` reports them together. The
old assertion failures did **not** prevent evaluation of the health assertions;
the earlier "masked because never reached" explanation was wrong.

**Baseline full-rate health passes; earlier current-state health failures remain
unresolved.** This does not confirm a pre-existing health failure at `d168aa6`.
A single baseline pass versus historical current failures also does not identify
a causal commit or exclude intermittency. No additional comparison or mechanism
investigation was authorized. Scoped closure records this limit rather than calling
Twins-health fixed, proven pre-existing, or non-blocking. Its assertions stay intact.

The **separate Twins-share disposition** remains informational: the preference is a
bounded ranking multiplier, not a share allocator. Even ideal proportional use of
the 1.2x ceiling gives `1.2/(1+1.2) = 54.545%`, below the old 55% floor; this arithmetic
is not a universal traffic-share bound or a guarantee. The owner-accepted removal
of that blocking share range does not waive mapping, RPC snapshots, recovery or health.

#### I, D and the genuine priority-snapshot fix

- **I: closed as resolved/non-issue for Todo 30.** With the blocking `17 <= t < 20`
  precondition restored, all 15 samples passed (12.738880–12.886272 Mbit/s versus
  11.52 Mbit/s required). REG3 recovered in 17.455938485s, sink in 19.773828501s;
  both links had 11,313,708.49898476 bps / `DemandLimited` budgets. The earlier
  reported 2/3 failure pattern did not reproduce. This is a scoped owner disposition,
  not proof of permanent reliability or a retroactive cause for those failures.
  The blocking assertion remains; no low/Draining attribution to scenario A was found.
- **D: existing explicit ignore is unchanged.** Reason:
  `wire-rate/stall-detector coupling: 4/5 historical pass rate; N-run statistical evaluation deferred to Todo 32`.
  This is a documented non-blocking default-gate disposition, not a fresh pass.
- **Priority snapshots: genuine fix retained in `361d644`.** Pool-control mutation
  previously left cached per-link stats stale until housekeeping; immediate
  `adaptive_state.update_stats()` after application refreshes the inputs used by
  subsequent telemetry snapshots. Earlier before/after evidence established the
  fix; the new baseline clear-RPC mismatch independently reproduces the old defect.
  This fixes RPC-to-telemetry staleness, not scheduler health or measured wire share.

#### Gate accounting and evidence boundary

The nine non-adaptive targets have historical clean privileged-pass evidence:
`netns_basic` (2), `netns_bond` (5), `netns_edpf` (1), `netns_failure` (2),
`netns_impairment` (3), `netns_pr19_parity` (2), `netns_scenario` (2),
`netns_twin` (8), `netns_unconnected` (2): **27 tests**, no self-skips in that
recorded nine-target gate. That evidence predates the current ten-target gate;
it is not a fresh all-target run at final HEAD. The owner ended testing after the
single Twins baseline comparison. The current script fails fast on adaptive,
so its complete ten-target success line must not be claimed.

`netns_adaptive` has the four findings above; I is not a fifth outstanding finding.
**Todo 30's literal every-target-passes bar remains unmet.** Scoped completion
delivers an honest evaluation/decision record, not scheduler acceptance. The owner
may mark complete-with-documented-findings or retain an open dependency on Todo 32.

Local evidence bundles: `todo30-attribution.9xPKZn` (I, G1, G2) and
`g-bisection.AGlV6w` (G endpoint pair, `twins-baseline.log`, mapped
`adpt_s_59df_0`, legacy `adpt_s_59df_12`, raw observations and host checks).
No future campaign result, full-gate pass, or hardware validation is implied.

### Decision and discovery

The owner has ended the incremental investigation and selected **`d168aa6` as the
retained sender baseline**. The subsequent uncommitted estimator experiments are
reverted, not shipped or represented as an accepted performance improvement. Their
individually verified mechanisms and reusable tests are preserved below for future work.

Todo 29's smoke campaign exercised scenario A for the first time in this integration
effort: three 10 Mbit/s links, 60 ms delay, 15 ms stationary Normal jitter, 0.2% loss,
24 Mbit/s offered application traffic, and **no intentional impairment events**.
Baseline does not mean zero jitter/loss. The committed adaptive sender collapsed to
250 kbit/s per link and remained Draining. A loopback/three-uplink capture plus local
socket sampling showed receive-buffer saturation and socket drops rising from 213 to
159,077; 40,151 NAK-requested sequences had arrived locally before their NAK but never
appeared on an uplink. Paused local UDP reads are a confirmed loss amplifier, not a
lossless upstream backpressure mechanism.

Later experimental candidates removed that permanent low-rate state, briefly delivered
above the 21.6 Mbit/s settling target, then fell to roughly 7–11 Mbit/s in low phases.
Do not conflate those later peak/decline traces with the original `d168aa6` low-rate
collapse: these are different manifestations of an unresolved control-loop limitation.
The smoke harness correctly refused to call either a stable, successful measurement.

### Three confirmed mechanisms; none closed the live acceptance gap

1. **Asymmetric instantaneous queue evidence.** The first-cut detector accepted three
   high instantaneous RTT samples with a 2 ms endpoint rise. Ordinary jitter could
   enter Draining, while clearance also demanded instantaneous samples near the long
   minimum. It was not a monotonic, time-normalized growth detector. Unequal 1-second
   and 30-second minimum populations also bias the window signal under stationary
   jitter. Consult 14 separated immediate protection from durable inference: preserve
   the fast cut, save a provisional ceiling instead of mutating `blocked`, and use
   `max(0, queue_delay_ms - 0.5 * EWMA(abs(delta RTT)))`, alpha 1/16, for window evidence.
   A causal, due, proven, fully-used post-cut epoch could confirm the ceiling without
   another cut; clearance or 1500 ms expiry discarded it, retaining the reduced rate
   until normal clean search recovered. A full live trace recorded exactly one early
   fast-only cut per link, candidate clearance in 241–255 ms, a climb to 16 Mbit/s /
   DemandLimited, zero later cuts, zero confirmations, and zero expiry events. That
   particular permanent-collapse trap disappeared; stable useful delivery did not.
2. **Unvalidated DemandLimited budget versus physical capacity.** A concrete epoch
   accepted 11.164608 Mbit/s on a 10 Mbit/s link, yet reported `fully_used=false`
   because its unvalidated budget was 16 Mbit/s: 90% would require 14.4 Mbit/s. Corrected
   high queue and original delivery proof were present. Consult 15 waived utilization
   only in DemandLimited, preserving due/proven/causal/high/settle/non-falling/persistence
   requirements and direct `blocked` commitments for strong window evidence. A real
   epoch-transition test proved the new exact-boundary cut, with Searching/Draining/
   CapacityHeld negative controls intact. Live A still failed and never made the
   expected subsequent cut. The utilization veto was real, but not a complete cause.
3. **Missing repeat-confirmation reset on causal low samples.** Successive live causal
   low samples, with neither fast nor window-high evidence, retained the same old
   `repeat_high_since_ms`; those active-phase paths did not enter the authorization
   helper. A failing-first regression reproduced the missing reset. The final change
   cleared all three repeat fields immediately after every new causal non-high sample,
   before phase dispatch. Interrupted-high and noncausal-low controls passed. No
   premature bridged cut had been observed live, so that consequence remained a risk,
   not a measured cause of the throughput failure. Fixing the confirmed bookkeeping
   defect still did not close A acceptance.

The exact final two policy changes, applied after the provisional-learning experiment,
were:

```rust,ignore
// In post-first-cut authorization; other phase requirements remain unchanged.
let utilization_qualified =
    result.fully_used || matches!(self.phase, WireRatePhase::DemandLimited);
if !result.proven || !utilization_qualified {
    self.clear_repeat_confirmation();
    return false;
}

// Immediately after queue observation, before phase dispatch.
if queue.new_causal_sample_ms.is_some() && !queue.fast && !queue.window_high {
    self.clear_repeat_confirmation();
}
```

### What remains unresolved

With all three mechanisms in place, **44/44 focused wire-rate tests passed**, including
the original tight 1 Mbit/s and delayed-window convergence guards; the final default
library had 887 passes, zero failures, one existing ignore. The decisive live A run
nevertheless failed settling after 37.55 s. All three rates reached 16 Mbit/s /
DemandLimited and stayed there through shutdown, without a subsequent direct ceiling
commitment or congestion cut. The full sender log had 20,656 lines, startup to SIGTERM.

| Final experimental A observation | Value |
|---|---:|
| Peak complete sink-relative second | 23.256352 Mbit/s |
| Following second 14 | 11.159680 Mbit/s |
| Second 17 | 7.359072 Mbit/s |
| Final complete second 34 | 17.802848 Mbit/s |
| UDP5555 drops by second 13 | 110,161 |
| Final sampled UDP5555 drops | **546,981** |
| Warm-up receiver unique / dropped / retransmitted packets | 45,031 / 20,609 / 32,614 |

Receive memory repeatedly reached 212,928 of 212,992 bytes. These socket drops prove
severe local queue overflow, but are **not themselves the path-RTT congestion signal**
used by the estimator. We have not shown every authorization predicate simultaneously
true on an actual dispatch. A further gate, evidence-timing interaction, or broader
startup/retry/health-admission coupling remains a hypothesis—not a confirmed fourth
bug. The remaining causal conjunction needs observation, not another guessed threshold.
The receiver figures are warm-up report totals, not a completed 45-second campaign;
failed RunRecord zeroes are placeholders. No million-NAK count is inferred from them.

### Baseline guarantees and boundaries retained

Todo 28 did not exercise A. Restoring `d168aa6` preserves its accepted D/G/I/twin work
and avoids shipping any of these unaccepted experiments. Its acceptance is **scoped**:
D achieved 4/5 bounded recoveries by explicit owner acceptance, with later full-rate
health/share residuals documented above; G's final run carried 6,904,330 bytes with
0/61 demotions; I recovered registration in 17.601 s and sink throughput in 19.715 s.
Twin recovery/identity controls and known later share/publication limitations remain
as previously recorded. This is not a claim that every protected/adaptive test always
passes, or bonded-hardware validation. The smoke's three consecutive 90%-throughput
settling seconds are stronger than Todo 28's initial positive-traffic warm-up, and its
D workload does not inherit Todo 28's feasible-recovery source-rate adjustment.

### Future work: use the planned campaign, not another ad-hoc iteration

Wave 6's Campaign C1 covers all **13 concrete profiles** (A, B1, B2, C–L). Campaign C2,
Todo 32, is the designated adaptive ablation and pre-declared constant-sweep effort:
N=5 for each declared feature/target combination, with A as a mandatory per-candidate
and per-value regression guard. C2 itself is a **targeted matrix**, not every ablation
crossed with all 13 profiles. It must use C1's full-scenario context and preserve its
declared statistical criteria. No C2 value, missing A evidence, or architecture change
is silently authorized by this note. A controller redesign beyond the declared sweep
needs its own explicit scope and acceptance decision.

Start from this fix sequence and the preserved test source below; do not repeat the
same failed investigations from scratch. Consults 10–11 concern causality, utilization
and phase-bypass fixes; consults 12–13 and rounds 19–20 cover the rejected window-only
attempts; round 21 restored the baseline; consult 14 introduced provisional learning;
consult 15 and the final reset round isolated the two additional mechanisms. One prior
round-22 trial used a PATH receiver and is **not** a controlled comparison. Corrected
trials used the explicitly pinned CeraLive receiver SHA-256
`38b7b76c6e8f8e888aaecf1baf784c7b30895c6cc17e01e03857474e987ac854`.
Final experimental sender SHA-256 was
`a110e9c9a91a6dceef0700635d57c2b251372f6bdfb2469fde1c295273675968`.
The local investigation ledger retains the full round 19–22 history, consult 10–15
records, source diffs, immutable binaries, raw logs and socket/receiver/sink analyses.
That machine-local ledger is not a runtime/build dependency of this standalone repo.

### Todo 29 scope decision

Keep the literal classic/enhanced/adaptive × A/D manifest and execute every cell.
**Only adaptive/A is a documented non-blocking finding for smoke-path acceptance.**
Retain its failed artifacts and report them separately; never relabel them `ok` or
fabricate a complete 12-run summary. The other five cells still require two genuine
successful runs each, all original configuration, packet-count, full-window and
adaptive-D recovery assertions. Any failure there remains blocking. This exception
does not waive adaptive/A from C1/C2, prove adaptive performance, or automatically
complete Todo 29. The original classic/enhanced A successes prove measurement plumbing,
not missing D successes; a new scoped campaign must pass before closure.

### Preserved provisional-learning regression source (not active tests)

The following is the complete pre-revert `src/connection/wire_rate/provisional_tests.rs`.
It targets the **rejected experimental API**, including `candidate_blocked_bps`, absent
from `d168aa6`; restoring it alone does not compile. Its `super::tests::input` helper
and the real RttTracker/TestClock remain repo-local. RTT values are captured; the
100 ms repeating cadence is explicitly synthetic. The source is preserved as a
reference, not proof of an accepted live fix.

<!-- provisional-tests-reference-start -->
```rust,ignore
use super::tests::input;
use super::{WireRateEstimator, WireRateInput, WireRatePhase, WireRttSample};
use crate::connection::RttTracker;
use crate::utils::test_clock::TestClock;

fn fast_cut() -> WireRateEstimator {
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    for (now, bytes, raw) in [(1010, 1000, 8.0), (1040, 2000, 9.0), (1070, 3000, 10.0)] {
        estimator.update(&WireRateInput {
            queue_delay_ms: 0.0,
            ..input(now, bytes, raw)
        });
    }
    estimator
}

#[test]
fn fast_only_cut_protects_immediately_without_learning_a_ceiling() {
    // Given the unchanged three-sample fast edge, when protection fires.
    let estimator = fast_cut();
    // Then pacing halves immediately, but the noisy trigger establishes no bracket.
    assert_eq!(estimator.rate_bps(), 500_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
    assert_eq!(estimator.blocked, None);
}

#[test]
fn confirmed_candidate_records_the_pre_cut_ceiling_without_cutting_again() {
    // Given a fast-only cut with no durable ceiling yet.
    let mut estimator = fast_cut();
    assert_eq!(estimator.candidate_blocked_bps, Some(1_000_000.0));
    assert_eq!(estimator.blocked, None);
    // When a due, fully-used epoch has causal RTT and a corrected high window.
    estimator.update(&input(2070, 65_500, 10.0));
    // Then learning commits the old rate, without applying another reduction.
    assert_eq!(estimator.blocked, Some(1_000_000.0));
    assert_eq!(estimator.candidate_blocked_bps, None);
    assert_eq!(estimator.rate_bps(), 500_000.0);
    assert_eq!(estimator.last_cut_ms, Some(1070));
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
}

#[test]
fn candidate_confirmation_rejects_incomplete_post_cut_evidence() {
    // Given independent fast cuts, each missing one required confirmation dimension.
    for unqualified in [
        input(2069, 65_500, 10.0),
        input(2070, 3000, 10.0),
        WireRateInput {
            latest_original_delivery_ms: Some(1069),
            ..input(2070, 65_500, 10.0)
        },
        WireRateInput {
            latest_rtt: Some(WireRttSample {
                observed_ms: 2070,
                rtt_ms: 1001.0,
            }),
            queue_delay_ms: 100.0,
            ..input(2070, 65_500, 0.0)
        },
    ] {
        let mut estimator = fast_cut();
        // When the incomplete evidence arrives.
        estimator.update(&unqualified);
        // Then neither proof, utilization, timing nor causality may be bypassed.
        assert_eq!(estimator.blocked, None, "{unqualified:?}");
        assert_eq!(estimator.candidate_blocked_bps, Some(1_000_000.0));
        assert_eq!(estimator.rate_bps(), 500_000.0);
    }
}

#[test]
fn discarded_candidate_recovers_only_on_a_due_clean_search_epoch() {
    // Given a fast-only cut cleared by window evidence at1160ms.
    let mut estimator = fast_cut();
    for now in [1100, 1130, 1160] {
        estimator.update(&input(now, 3000, 0.0));
    }
    estimator.update(&input(2159, 65_438, 0.0));
    assert_eq!(estimator.rate_bps(), 500_000.0);
    // When the full one-second clean, proven, fully-used epoch becomes due.
    estimator.update(&input(2160, 65_500, 0.0));
    // Then normal unbracketed doubling, not an immediate restoration, recovers the rate.
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    assert_eq!(estimator.blocked, None);
    assert_eq!(estimator.candidate_blocked_bps, None);
}

#[test]
fn window_drain_resumes_search_without_restoring_the_pre_cut_rate() {
    // Given a fast-only cut with raw RTT still above the long-window floor.
    let mut estimator = fast_cut();
    // When three window-clear observations arrive despite high instantaneous RTT.
    for now in [1100, 1130, 1160] {
        estimator.update(&WireRateInput {
            queue_delay_ms: 0.0,
            ..input(now, 3000, 10.0)
        });
    }
    // Then clearance releases inference, not the protective reduction.
    assert_eq!(estimator.phase(), WireRatePhase::Searching);
    assert_eq!(estimator.rate_bps(), 500_000.0);
    assert_eq!(estimator.blocked, None);
}

#[test]
fn ambiguous_fast_cut_expires_without_a_sample_or_a_rate_restore() {
    // Given an unconfirmed fast cut and a signal in the hysteresis gap.
    let mut estimator = fast_cut();
    let ambiguous = WireRateInput {
        queue_delay_ms: 4.0,
        latest_original_delivery_ms: None,
        ..input(1100, 3000, 10.0)
    };
    estimator.update(&ambiguous);
    estimator.update(&WireRateInput {
        now_ms: 2569,
        ..ambiguous
    });
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
    // When the provisional lifetime reaches exactly1500ms without new RTT.
    estimator.update(&WireRateInput {
        now_ms: 2570,
        ..ambiguous
    });
    // Then missing evidence cannot trap the rate indefinitely or commit a ceiling.
    assert_eq!(estimator.phase(), WireRatePhase::Searching);
    assert_eq!(estimator.rate_bps(), 500_000.0);
    assert_eq!(estimator.blocked, None);
}

#[test]
fn captured_a_rtt_values_do_not_poison_unbracketed_search() {
    // Given actual keepalive RTT values from a stationary scenario-A collapse capture.
    // Arrival-minus-embedded-send milliseconds, link0/run-VdcL3K, in wire order.
    // Only cadence is synthetic: repeat at100ms to model continuing DATA RTT evidence.
    const RTT_MS: [u64; 20] = [
        29, 60, 64, 44, 73, 85, 67, 60, 73, 71, 46, 59, 73, 59, 59, 80, 63, 84, 56, 64,
    ];
    let clock = TestClock::new(1000);
    let mut tracker = RttTracker::default();
    let mut estimator = WireRateEstimator::default();
    let mut bytes = 0;
    let mut longest_drain_ms = 0;
    let mut draining_since = None;
    estimator.update(&input(1000, 0, 0.0));
    // When the real time-window minima and estimator consume sixty seconds of jitter.
    for (step, rtt) in (1_u64..=600).zip(RTT_MS.into_iter().cycle()) {
        let now = 1000 + step * 100;
        clock.set(now);
        tracker.update_estimate(rtt);
        bytes += (estimator.rate_bps().min(8_000_000.0) / 80.0) as u64;
        estimator.update(&WireRateInput {
            latest_rtt: Some(WireRttSample {
                observed_ms: now,
                rtt_ms: rtt as f64,
            }),
            slow_min_rtt_ms: tracker.slow_min_rtt_ms(),
            queue_delay_ms: tracker.queue_delay_ms(),
            ..input(now, bytes, 0.0)
        });
        match estimator.phase() {
            WireRatePhase::Draining => {
                let start = *draining_since.get_or_insert(now);
                longest_drain_ms = longest_drain_ms.max(now - start);
            }
            WireRatePhase::Searching
            | WireRatePhase::CapacityHeld
            | WireRatePhase::DemandLimited => {
                draining_since = None;
            }
        }
    }
    // Then transient protection never becomes a permanent low-rate/false-bracket trap.
    assert!(
        longest_drain_ms <= 1500,
        "draining={longest_drain_ms}ms, {estimator:?}"
    );
    assert!(estimator.rate_bps() >= 8_000_000.0, "{estimator:?}");
    assert_eq!(estimator.blocked, None);
}
```
<!-- provisional-tests-reference-end -->

### Scoped smoke rerun: exception implemented, Todo 29 still blocked

The owner-approved adaptive/A-only exception was implemented and the full six-cell
campaign rerun to completion on the restored baseline. It took **925.15 seconds**
(926.12 seconds including command overhead), within the one-hour bound. There were
**22 attempts: two successes and twenty settling failures**. Ten indices exhausted
their two-attempt budget. This is not a successful scoped campaign:

| Cell | Successful indices | Required result |
|---|---|---|
| classic/A | 1 only (1/2) | Blocking: index 0 missing |
| classic/D | none (0/2) | Blocking: both missing |
| enhanced/A | none (0/2) | Blocking: both missing |
| enhanced/D | 1 only (1/2) | Blocking: index 0 missing |
| adaptive/D | none (0/2) | Blocking: both missing |
| adaptive/A | none (0/2) | Sole non-blocking finding; both exhausted with settling failures |

Every exhausted record reports `settle_timeout`. The two successful measurements were
classic/A at **21,049,683.2 bit/s** useful goodput and enhanced/D at
**11,013,270.613333331 bit/s**. A successful measurement record is not a claim of good
viewer quality; their recorded viewer-loss ratios were 0.1224883 and 0.5100603.
Eight missing **required** indices remain after excluding adaptive/A. The checker
correctly returned nonzero and published no new success summary. The earlier A
successes remain valid historical observations, not replacements for this run's
missing indices or evidence that all D cells pass.

Sender SHA-256 was `b8763976a36134ecd115d9666a8b30791b24ccd6cb077fbf110ade6177e94af8`
(the original immutable clean `d168aa6` artifact); receiver SHA-256 was the pinned
CeraLive `38b7b76c…` above. Both were copied byte-identically into ignored repo-local
artifact paths for the portable manifest. No source rebuild was mislabelled as a
clean-revision artifact, no timeout was extended, and no raw record was edited.

The exception's synthetic tests reject a failed record in each of the five required
cells and reject optional-cell provenance/setup corruption; the original assertion
mutation controls also pass on synthetic fixtures. The real-success report path and
real-evidence mutation gate remain unvalidated until ten required successes exist.
The generic runner and reporter are unchanged. No broader exemption, scheduler fix,
or completion commit is justified by this rerun. **Todo 29 remains open.**

### Isolation and sequencing audit: no demonstrated cleanup fix

Subsequent true single-work-item tests bypassed the campaign loop and retries, while
using the same immutable `d168aa6` sender, pinned CeraLive receiver and settling logic.
Classic/A passed at warm-up16.018s on three windows of23.645888/23.561664/23.593248Mbit/s.
Adaptive/A failed with a maximum4.379648Mbit/s against21.6Mbit/s required; adaptive/D
failed with a maximum9.664704Mbit/s against20.16Mbit/s required. Both adaptive runs
reached CSV readiness and failed before measurement events began. This establishes a
standalone adaptive cold-start throughput limitation under the tested setup, not merely
interference from preceding campaign cells. It does not contradict Todo28's different
obstruction-recovery predicate or prove that the behavior is only benign slow discovery.

The isolated classic pass does **not** prove a campaign-sequencing bug. In the retained
fresh campaign, classic/A failed at **order index0**, before any previous cell in that
campaign could leave resources behind; classic/A later passed at index6 and enhanced/D
passed at index18. The lifecycle audit found synchronous bounded-worker completion and
cleanup before the next item, fresh per-worker topology and processes, PID/counter-based
names and unique files. Immutable executable files are reused; running senders and
receivers are not. No concrete overlap, reused runtime state or cleanup leak was
demonstrated. Stochastic network/source/host effects and measurement calibration remain
unresolved possibilities. No arbitrary cooldown or speculative cleanup patch was added.

The owner subsequently authorized **both adaptive/A and adaptive/D** as non-blocking
smoke-throughput findings, retaining all four classic/enhanced A/D cells as required
(eight successful runs). That policy is separate from a proven harness fix. The attempt
stopped at the explicit “no clear, fixable harness bug” boundary: the broader exception
has not yet replaced the earlier A-only checker, no new full campaign was run, and no
completion commit was made. Under the newly authorized policy, the retained campaign
still has only2/8 required successes. A/D exemptions do not apply to C1/C2 without a
separate decision. No passing campaign, reliable legacy-mode gate, or fixed harness is
claimed by this audit.

### Final owner calibration: one-of-two measurement-path coverage

The owner subsequently accepted the measured variance as grounds to calibrate the
smoke acceptance policy, rather than continue searching for an unproven sequencing
fix. Classic/A failed at the first campaign item, passed at a later item, and passed
true isolation; adaptive A and D failed true isolated cold-start settling. These
observations distinguish path coverage from performance certification. They do not
prove every remaining source of variance understood or make every attempted run good.

The unchanged manifest still schedules all six cells and two run indices per cell.
The 30-second settling budget, 90% target, three-consecutive-second requirement,
minimum prehistory and retry budget are unchanged. The new smoke-only acceptance is:

- At least **one successful run out of the two planned indices in each** classic/A,
  classic/D, enhanced/A and enhanced/D cell. One-of-two is a coverage criterion, not
  a strict statistical majority. No cell may borrow another cell's successes.
- Every successful required record must still satisfy the original goodput, in-window
  receiver-packet, configuration, full-window and D-event assertions. A missing second
  success is tolerated only with a recorded settling exhaustion, not arbitrary required
  setup/configuration failure.
- Adaptive A and D execution outcomes are informational, with provenance still checked.
  Their data and failures remain visible and are not treated as performance passes.
- All twelve planned index outcomes must exist. The published required summary contains
  the actual four-to-eight successes with original indices; an index1-only success
  stays index1, and no failed record is rewritten or counted as success.

The generic runner retains its original completeness exit status. A separate calibrated
checker/reporting layer supplies the smoke-path verdict; raw campaign failures remain
recorded. The reporter's opt-in `--smoke-coverage` is restricted to the four canonical
classic/enhanced A/D cells, runs2, seed1 and campaignsmoke; its default is unchanged.
C1/C2 retain complete evidence requirements and their own statistical rules. No scheduler
or harness-sequencing fix is claimed by this policy change. A final fresh campaign is
required before Todo29 can be marked complete under this calibration.

### Final calibrated campaign outcome: partial, zero D coverage

The single final rerun completed in **859.93s** (860.87s including command overhead),
with the same immutable `d168aa6` sender, pinned CeraLive receiver and unchanged six-cell
manifest. It produced **21 attempts: three successes and eighteen `settle_timeout`
failures**; nine indices exhausted their existing two-attempt budget.

| Cell | Successful planned indices | Calibrated verdict |
|---|---|---|
| classic/A | 0 and 1 (2/2) | Coverage passes |
| classic/D | none (0/2) | **Fails: zero coverage** |
| enhanced/A | 0 (1/2) | Coverage passes |
| enhanced/D | none (0/2) | **Fails: zero coverage** |
| adaptive/A | none (0/2) | Informational |
| adaptive/D | none (0/2) | Informational |

Completed useful-goodput measurements were classic/A **21,458,169.6** and
**21,010,612.622222222 bit/s**, and enhanced/A **21,251,820.8 bit/s**. Their viewer-loss
ratios were 0.1056554100, 0.1238761724 and 0.1148714708 respectively; these are measured
path successes, not low-loss performance certifications. Both required D cells had
four failed attempts each and no successful run. The calibrated checker explicitly
rejected those two cells. No complete calibrated success summary was published and no
failure was relabelled or merged with a historical success.

Calibration tooling is tested: eighteen reporter tests (including strict-default and
nonzero-index CLI behavior), synthetic end-to-end smoke reporting, zero-coverage and
provenance controls, and required-record mutation checks passed. Rust sender/harness
source remains unchanged from `d168aa6`, with875 library passes. This is the final
honestly partial outcome, not a passing Todo29 campaign. No additional iteration,
broader acceptance change or conditional completion commit followed the failed run.

### Final retrospective acceptance: classic/A and enhanced/A coverage only

**This owner decision supersedes the earlier Todo29 scope decisions above, not their
measurements.** After the final campaign, the owner required one successful planned
index out of two in **classic/A and enhanced/A only**, with every D cell and adaptive/A
informational. The instruction was to rescore the **existing final campaign**, make
no new live runs or design changes, and commit the tooling and evidence.

The read-only rescore passes. The [portable report](smoke-final-report-2026-09.md) contains
the actual statistical tables and warnings; the [receipt](smoke-final-receipt-2026-09.json)
records all six outcomes, the three successful run IDs, original indices, measured
goodput/loss, and exact source/binary/receiver/manifest and generated-artifact hashes.
Full raw host/process artifacts remain in the retained local campaign archive; no
machine-local path is required to build or test this standalone repository.

| Cell | Successful planned indices | Final owner-scoped verdict |
|---|---|---|
| classic/A | 0 and 1 (2/2) | Required coverage passes |
| enhanced/A | 0 (1/2) | Required coverage passes; index1 settling exhaustion retained |
| classic/D | none (0/2) | Informational; four settling failures |
| enhanced/D | none (0/2) | Informational; four settling failures |
| adaptive/A | none (0/2) | Informational; four settling failures |
| adaptive/D | none (0/2) | Informational; four settling failures |

The raw campaign **still failed with exit101**, in859.93s, with21 attempts,
three successes, eighteen settling failures and nine exhausted indices. Rescoring
does not rewrite those results, renumber an index, pool a historical success, or
claim twelve successes. The scoped summary has classic n2 and enhanced n1, both on A.
Their median useful goodput is21,234,391.111111112 and21,251,820.8bit/s respectively;
their observed loss and busy-host warnings remain visible. N=1/2 confidence intervals
are not performance certification, and one-of-two is not a statistical majority.

D's lack of coverage affected **all three candidates in this final campaign**. The
earlier enhanced/D success remains contrary evidence to a universal-D-failure claim.
Likewise, the first-item classic/A failure followed by later and isolated success did
not establish a cleanup bug. Adaptive A/D failures in true isolation remain a known
cold-start throughput limitation, distinct from Todo28's bounded obstruction recovery.
No scheduler fix, arbitrary cooldown, relaxed settling predicate or new measurement
is claimed. Runtime and harness source stay at the restored `d168aa6` baseline.

The explicit reporter option accepts only the two canonical required smoke cells,
runs2/seed1. All twelve terminal outcomes still undergo identity, binary/receiver hash
and profile/window validation. Required successes retain goodput, receiver-packet,
configuration and full-window checks; failed required indices must be settling
exhaustions. Synthetic controls exercise zero coverage, nonzero indices and corruption;
the actual final campaign passes the real-data mutation gate. Default manifest-complete
reporting and every C1/C2 requirement remain unchanged. **Todo29 is accepted under this
explicit final measurement-path scope only; adaptive performance remains unresolved.**
