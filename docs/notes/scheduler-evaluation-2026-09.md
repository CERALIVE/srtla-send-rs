# Scheduler evaluation — September 2026

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
