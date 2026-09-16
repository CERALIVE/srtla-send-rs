# Receiver-policy evaluation: `nakreport=0` and `reorderfreeze=1` on the C1 bench (2026-09)

## Scope and outcome

This note is the durable record of the receiver-policy investigation that changed
the C1 bench receiver baseline twice on 2026-09-16. It exists so that a fresh clone
carries the evidence behind the committed configuration, the trade-off that
configuration accepts, and one hypothesis that was tested and killed, so nobody
spends another dozen bench trials re-deriving the same dead end.

Two commits landed:

- `1c4679d` `fix(bench): use CeraLive SRT fork with reorderfreeze for C1 receiver
  baseline`: switched the bench listener onto a CeraLive SRT fork build and set
  `reorderfreeze=1&nakreport=0` (the shape of the production `classic`/L2 profile).
- `46170c6` `fix(bench): drop nakreport=0 from the C1 receiver baseline`: kept the
  fork build and freeze, removed NAK-off. `SrtProfile::PRODUCTION` now emits
  `mode=listener&latency=2000&lossmaxttl=40&reorderfreeze=1`
  (`crates/network-sim/src/harness.rs`).

The shipped baseline is therefore **freeze on, NAK reports on**. It is a
provisional C1 baseline, not C1 acceptance, and it carries a known, bounded
limitation: Scenarios B1 and C do not settle under it. Everything below is
`classic` sender unless stated otherwise; the failure signature that drove the
change was first seen on `enhanced` and then reproduced on `classic`, so it is not
scheduler-mode-specific.

All trials were 30 s warm-up diagnostics on fresh network namespaces, seed
`20260913`, CPU affinity 8–23, with the host measurement lock held, run through a
private copy of the campaign stack that changed only the listener treatment.
"Settled" means three consecutive complete 1 s useful-goodput windows at or above
90% of the offered rate, first accepted at second ≥ 10, evaluated over windows
1–29. "Final 10 s" is the mean of windows ending at seconds 20–29. Received
retransmission fraction is `100 × Σ pktRcvRetrans / Σ pktRecv` over the retained
interval CSV. Non-model queue drops are netem handle `11:` generic `drops` minus
extended `dropped`, per link, never summed with parent TBF/prio counters. A
collection exit of 0 is never a performance pass.

Every trial verified the listener's actual `Media path:` log and
`/proc/<pid>/cmdline` before traffic was enabled, failing closed on any mismatch.
The sender was the archived clean `7e9a780` classic build (SHA-256
`4a11c169…13bde6`); the SRT apps were the fork's `srt-live-transmit` (SHA-256
`639c4a64…dec7445`) resolving its build-local libsrt 1.5.6; the SRTLA receiver was
CeraLive's `srtla_rec` (SHA-256 `fa3524c8…1e6ab7`). None of those binaries was
rebuilt between rounds.

## Evidence provenance (machine-local, NOT in the repo)

The raw trial directories live only on the workstation that ran them, under
`/home/andres/.cache/opencode/tmp/opencode/`. They are not tracked, not reproducible
from a clone, and will not outlive that machine. They are cited here because they
are the honest provenance record, not because a reader can fetch them:

| Directory | Round | Trials |
|---|---|---|
| `c1-l2-repro/` | Original vanilla-vs-both-options B1/C arms; also hosts the unchanged `summarize.py` reducer | archived |
| `classic-g-l2/` | First `classic`/G observation under the `1c4679d` receiver (the reframing result) | 4 |
| `classic-g-factorial/` | 4-condition `classic`/G factorial on the fixed fork build | 12 |
| `upstream-classic-g/` | `irlserver/srtla_send@df0b393` vs fork under NAK-off + G | 6 |
| `classic-freeze-b1-c-g/` | Freeze-only revalidation on B1, C, G | 9 |
| `classic-b1-c-a-factorial/` | B1/C NAK-off-only arms plus the full 4-condition A factorial | 18 |
| `classic-a-spread-factorial/` | The falsification run: A with 20/60/120 delays, `first30/` and `full45/` | 12 |

Each directory retains the scratch `diagnostic.rs`/`stack.rs`, the `run.log` with
every URI check, per-trial `listener-cmdline.bin`, sink and receiver CSVs, 1 Hz
qdisc/telemetry samples, and the reducer's `summary.jsonl`/`csv-summary.jsonl`/
`queues.jsonl`. Roughly 57 bounded trials in total, on top of the 10.9 h campaign
that first exposed the problem.

## 1. The finding: `nakreport=0` is the isolated regression trigger under real loss

The trigger was a question from the owner: `nakreport` is a receiver option, so
`1c4679d` applied to every sender mode equally, yet it had only been verified on
`classic`/B1,C,F and `enhanced`/A,G. `enhanced`/G was already failing before the
change, so an unchanged 0/3 hid the severity. Running a **previously clean** cell,
`classic`/G, under the corrected receiver gave 0/4 settled against a historical
5/5, with 44.8–46.6% received retransmissions (historical 1.7–1.9%) and thousands
of non-model queue drops on every link, including the three companions that carry
only 0.2% random loss. That proved the regression was receiver-side and not the
enhanced cooldown candidate, but it could not say which part: the fork build, the
freeze option, or NAK-off had all changed together.

The factorial that isolated it held the SRT build fixed and varied only the
listener URI suffix over the base
`srt://:4001?mode=listener&latency=2000&lossmaxttl=40`. Three trials per
condition, three interleaved blocks (`1,2,3,4 / 4,3,2,1 / 2,4,1,3`), no retries.
Scenario G: link 0 is 60 ms one-way, 1 Mbit, Gilbert-Elliott 5%/0.1/0.8/0.02;
links 1–3 are 60 ms, 5 Mbit, 0.2% random loss; all queue limit 500; offered
12.8 Mbps (`crates/network-sim/src/scenarios.rs`, `scenario_g`).

| Condition | Suffix | Settled | Final 10 s Mbps (mean) | Received retx % | Non-model queue drops, links 0/1/2/3 |
|---|---|---:|---:|---|---|
| 1 bare | none | 3/3 | 12.796 | 1.860–2.119 | 0 / 0 / 0 / 0 |
| 2 freeze only | `&reorderfreeze=1` | 3/3 | 12.804 | 1.504–1.605 | 0 / 0 / 0 / 0 |
| 3 NAK-off only | `&nakreport=0` | 0/3 | 10.762 | 44.043–45.320 | 4983–6446 / 7586–13889 / 6731–9164 / 5852–10455 |
| 4 both (`1c4679d`) | `&reorderfreeze=1&nakreport=0` | 0/3 | 10.791 | 43.622–46.467 | 4777–6514 / 10102–11872 / 6129–14041 / 6476–11173 |

Conditions 1 and 2 also had zero SRT receive drops; conditions 3 and 4 had
5043–5713 and 5095–5666 respectively. Model (configured) drops stayed small in
every arm (64–90 on the GE link, 38–62 per companion), so the failing arms' drops
are real queue overflow, not misclassified configured loss.

Toggling NAK-off toggles the failure in both freeze strata. Three explanations are
falsified as sufficient causes under these conditions: the fork build alone
(condition 1 is clean), freeze alone (condition 2 is clean), and a
both-options-required interaction (condition 3 fails without freeze). This is a
2×2 option factorial on one fixed build, not a three-factor experiment across two
builds; it does not claim every libsrt build behaves the same way. The
packet-level NAK/retransmit sequence was not traced; the aggregate counters
identify the trigger, not the mechanism.

The freeze-only revalidation that followed (`classic-freeze-b1-c-g/`) reproduced
condition 2 on G in three interleaved positions: 3/3 settled, 12.802 Mbps,
1.432–1.699% received retransmissions, zero non-model drops and zero SRT receive
drops. That is the configuration `46170c6` ships.

## 2. The trade-off: B1 and C are UNRESOLVED under the shipped baseline

Dropping NAK-off is not a free fix. The same nine-trial revalidation ran B1 and C
under freeze-only, and both gave back everything `1c4679d` had gained:

| Scenario | Receiver | Settled | Mean final 10 s Mbps | Received retx % | Non-model drops by link (range) |
|---|---|---:|---:|---|---|
| B1 | vanilla, host app (archived) | 0/2 | 7.233 | 44.544–44.546 | 8396–10711 / 6915–6960 / 7243–8460 |
| B1 | freeze + NAK-off (`1c4679d`, archived) | 2/3 | 8.550 | 30.408–40.696 | 4419–6273 / 3129–5406 / 5356–6580 |
| B1 | freeze only (`46170c6`, fresh) | 0/3 | 6.758 | 44.454–49.051 | 10402–11338 / 7302–7561 / 5995–9176 |
| C | vanilla, host app (archived) | 0/2 | 9.051 | 51.816–52.490 | 44203–48721 / 50618–59426 |
| C | freeze + NAK-off (`1c4679d`, archived) | 3/3 | 18.925 | 40.023–42.212 | 4966–7820 / 40539–45810 |
| C | freeze only (`46170c6`, fresh) | 0/3 | 10.274 | 50.963–53.929 | 44174–49579 / 55536–65441 |

Relative to both-options, freeze-only loses about 21.0% of B1's final-window
goodput and 45.7% of C's. B1 lands roughly 6.6% below the archived vanilla mean;
C keeps a modest 13.5% improvement over vanilla, which is about 12.4% of the
absolute gain both-options had shown. No freeze-only B1 or C window reached its
settling threshold at all (maxima 8.307 and 16.255 Mbps against 8.64 and 20.16).

A later round (`classic-b1-c-a-factorial/`) added the missing NAK-off-only arm and
showed that **NAK-off alone delivers most of the B1/C gain**: B1 2/3 settled at
8.416 Mbps (both-options 8.550), C 2/3 at 19.796 Mbps (both-options 18.925).
Freeze is not required for those scenarios. That closes the "maybe freeze does
the work and NAK-off is dispensable" reading: it is exactly backwards. The option
that fixes B1/C is the option that destroys G.

**B1 and C are therefore unresolved under the shipped baseline.** They are
heterogeneous-delay scenarios that settle only under a receiver option proven
unsafe on scenarios with real configured loss. This is recorded as a known,
bounded limitation. It was not quietly fixed, and the bench campaign criteria,
scenario loads and 30 s settling gate were not relaxed to hide it. Do not hide it
with a scenario-specific NAK setting either; that would build the policy on a
discriminator this investigation could not find (section 4).

Two caveats on the B1/C numbers. The vanilla and both-options arms are archived
batches (`c1-l2-repro/`), not same-session pairs with the freeze-only run, and the
vanilla arm used the original host `srt-live-transmit`, so it carries a build
confound the fork-build arms do not. Report these as a large directional loss of
the gain, not as a statistically estimated success rate or as proof that freeze
itself harms B1. And even under NAK-off, B1/C keep large non-model queues and
substantial SRT receive drops; they were never clean production wins.

## 3. Scenario matrix: where NAK-off helps and where it hurts

Link parameters are from `crates/network-sim/src/scenarios.rs` and
`scenarios/satellite.rs`. Delays are one-way netem delays (section 5). Every LTE
link has 0.2% independent random loss and a 500-packet netem limit unless noted.

| Scenario | Links (one-way delay / rate / jitter) | Configured loss | Offered | NAK-on (best arm) | NAK-off (best arm) | Effect of NAK-off |
|---|---|---|---|---|---|---|
| A | 3 × 60 ms / 10 Mbit / 15 ms Normal | 0.2% each | 24 Mbps | 3/3, 21.496 Mbps (neither, fresh) | 0/3, 20.253 Mbps (both, fresh) | **hurts** |
| G | 60 ms / 1 Mbit / none + 3 × 60 ms / 5 Mbit / none | GE 5% on link 0, 0.2% others | 12.8 Mbps | 3/3, 12.804 Mbps (freeze, fresh) | 0/3, 10.791 Mbps (both, fresh) | **hurts, catastrophically** |
| B1 | 20 / 60 / 120 ms, 4 Mbit each, no jitter | 0.2% each | 9.6 Mbps | 0/3, 6.758 Mbps (freeze, fresh) | 2/3, 8.550 Mbps (both, archived) | **helps** |
| C | NAT Starlink 45 ms ± 5 / 20 Mbit / limit 4000 + LTE 65 ms / 8 Mbit | 0% Starlink, 0.2% LTE | 22.4 Mbps | 0/3, 10.274 Mbps (freeze, fresh) | 3/3, 18.925 Mbps (both, archived) | **helps** |

The full four-condition A factorial, all same-session and same-build (no batch or
host-app confound):

| A condition | Settled | Mean final 10 s Mbps | Received retx % | Non-model drops by link (range) |
|---|---:|---:|---|---|
| neither | 3/3 | 21.496 | 57.759–60.720 | 55298–60038 / 37366–43706 / 33102–34338 |
| freeze only | 2/3 | 20.920 | 56.614–61.089 | 55825–61179 / 35516–39016 / 30452–34221 |
| NAK-off only | 0/3 | 19.819 | 45.246–47.590 | 19914–26798 / 19868–23412 / 15153–22992 |
| both | 0/3 | 20.253 | 43.896–46.296 | 20348–27475 / 15426–24145 / 14862–17896 |

A's harm is not G's signature. On A, NAK-off *lowers* the received retransmission
fraction (44–48% against 57–61%) and *lowers* non-model queue drops, yet delivers
less useful goodput and loses settling 3/3→0/3; mean SRT receive drops rise from
about 7100 to about 10500. The NAK-on A arms are themselves unhealthy (heavy
queues and receiver drops) despite passing the short settling gate in 5/6 trials,
so there is no "clean A" claim here either. Received retransmission fraction never
counts an attempt the qdisc discarded before the receiver saw it; a lower fraction
is not evidence of fewer attempts.

C's arms are its warm-up base topology only. Its later periodic Starlink
delay-spike/capacity-dip waveform was not exercised in any of these rounds.

The "archived" versus "fresh" labels matter: archived arms come from earlier
batches on the same binaries (`c1-l2-repro/`), fresh arms were measured in the
round that produced the table. Cross-batch comparison is directional evidence,
not a paired estimate. Only the A factorial and the G factorial are fully
same-session 2×2 designs.

## 4. FALSIFIED: cross-link delay spread as the discriminator (do not re-run)

Once the A factorial landed, the matrix posed an obvious puzzle: A and B1 both
configure 0.2% loss on every link and go opposite directions. Loss level cannot
be the axis. An orchestrator-level hypothesis followed and it deserves a full
account, because it was persuasive.

**The hypothesis.** Cross-link delay spread discriminates. A (60/60/60) and G
(60/60/60/60) have zero spread and are hurt; B1 (20/60/120) and C (45/65 plus
satellite spikes) have large spread and are helped. Per-link jitter does not
discriminate (A has 15 ms and is hurt; B1 has none and is helped). The proposed
mechanism was coherent: heterogeneous delays produce systematic, persistent
cross-link reordering, so the receiver carries a standing loss list of gaps that
are not losses; periodic NAK re-reports flood the sender with spurious
retransmit requests, and turning NAK reports off is a large win. Homogeneous
links stay in rough sync, a gap is far more likely a genuine loss, and NAK-off
removes the only follow-up recovery path. It fit 4/4 scenarios, it would have
explained why BELABOX and irlserver ship NAK-off (real multi-carrier bonds are
the high-spread regime), and it would have implied a topology-keyed receiver
policy rather than one canonical policy.

**The predeclared test.** Take Scenario A exactly (three direct links, 10 Mbit
each, 15 ms Normal jitter, 0.2% loss, limit 500, 24 Mbps offered, 45 s profile)
and change only the three `link.base.delay_ms` fields to B1's 20/60/120. Run all
four receiver conditions, three trials each, same binaries, same summarizer. The
variant was built inside the diagnostic harness only; `scenarios.rs` was not
edited. Applied qdiscs on all three links were read back live (`tc -s -j qdisc
show`) and checked against the intended delay/jitter/loss/limit/TBF values before
traffic. Prediction: NAK-off must flip from harmful to helpful.

**Result: it did not flip.** (`classic-a-spread-factorial/`, 12 trials.)

| Receiver condition | Archived A 60/60/60, settled | Archived 30 s final 10 s Mbps | Fresh A 20/60/120, settled | Fresh 30 s final 10 s Mbps | Fresh 45 s final 10 s Mbps |
|---|---:|---:|---:|---:|---:|
| neither | 3/3 | 21.496 | 1/3 | 20.783 | 21.004 |
| freeze only | 2/3 | 20.920 | 0/3 | 19.900 | 20.905 |
| NAK-off only | 0/3 | 19.819 | 0/3 | 19.907 | 20.001 |
| both | 0/3 | 20.253 | 0/3 | 20.433 | 20.226 |

Both NAK-off arms stayed at 0/3. What the delay spread did was degrade the
NAK-on controls (neither 3/3→1/3, freeze 2/3→0/3). Spread makes everything
worse without changing NAK-off's sign. Without freeze, NAK-off-only still loses
about 4.2% first-30 goodput against neither (4.8% at 45 s). Within the freeze
stratum there is a small, window-sensitive wobble (both is 2.7% above freeze-only
at 30 s, 3.2% below at 45 s), which is not a confirmation of anything. NAK-off
still lowers received retransmissions (~44–45% vs ~55–62%) and queue drops, with
the same unhealthy delivery as homogeneous A. The hypothesis is dead under its
own predeclared test.

**The follow-on candidate fails too.** After the manipulation, A-spread and B1
differ in per-link capacity (10 vs 4 Mbit), offered load (24 vs 9.6 Mbps) and
jitter (15 ms Normal vs none). Jitter is the tempting next axis, and it fails
immediately on G: G has zero jitter and is still destroyed by NAK-off. Capacity
and offered rate remain candidates only as interactions (equal 0.2% loss
probabilities are not equal loss events per second at different packet rates,
and a 500-packet queue means a different time budget at 4 Mbit than at 10), but
nothing was tested that isolates them.

**No single-axis explanation survives.** Loss level, delay spread and jitter
have each been eliminated as sufficient discriminators. The most likely reading
is two distinct mechanisms sharing one symptom: G's collapse is plausibly a
genuine-loss-recovery failure (5% GE bursts with the NAK follow-up removed),
while A's degradation is something else that happens to also read as "worse
under NAK-off". That would explain why every single-axis hypothesis explained
some scenarios and failed others.

**Why this matters for policy.** The mechanism was worth finding because it
would have unlocked a topology-keyed receiver policy: serve NAK-off only to
bonds whose shape benefits. The falsification is evidence that such a policy is
not cleanly derivable from a bond's parameters, and a policy that cannot be
predicted from a bond's parameters is not one that can be auto-selected safely.
It also means the four-point correlation must not be used to explain upstream
and BELABOX's choice as "proven correct for heterogeneous links", nor to
invalidate C1's scenario mix. Do not re-run the spread manipulation on the same
premise; it has been done, with 45 s extension, and the answer is no.

The practical decision never needed the mechanism. NAK-off is unsafe as a
general default (it destroys A and G); freeze-only is safe on A and G and costs
B1/C; therefore the committed baseline stands and NAK-off is not a broad
production preset. An independent architecture review reached the same
conclusion before the last three diagnostics ran.

## 5. Two methodological corrections

**Delays are one-way, not RTT.** The catalog values (`60/60/60`, `20/60/120`)
are sender-egress one-way netem delays; `scenarios.rs` says so at its top and
tells the reader not to halve them. The orchestrator analysis that proposed the
spread hypothesis labelled them "RTTs" throughout, so its absolute numbers were
off by 2× (B1's spread is ~200 ms in RTT terms, not 100 ms). Relative ordering
across scenarios was unaffected, and the falsification used the verbatim catalog
values in the same shaping layer as every archived run, so the conclusion is not
touched. Loaded measured RTTs include queueing and are not expected to equal
twice the configured delay. "15 ms Normal jitter" is also a distribution
parameter, not a bounded ±15 ms interval.

**The diagnostics observed 30 s even where the catalog says 45 s.** Every
factorial in this note ran a 30 s warm-up window, including on A, whose catalog
profile is 45 s. The falsification run observed the full 45 s per trial while
saving an exact 30 s snapshot, so `first30/` is comparable to every archived arm
and `full45/` extends the same trials (not twelve extra replicates). The
settling gate stayed at the original 30 s bound in both views; the 45 s extension
did not grant extra settling time. The conclusion held in both windows.

## 6. The upstream/BELABOX tension, unresolved

Both reference lineages ship freeze-on plus NAK-off as their single receiver
mode: BELABOX's own SRT fork, and `irlserver/srt`'s `belabox` branch via
`SRTO_SRTLAPATCHES`, which `irlserver/irl-srt-server` sets in production and
which bundles both behaviours in one flag. CeraLive exposes them separately as
`SRTO_REORDERFREEZE` and the standard `SRTO_NAKREPORT`. Our controlled evidence
says that shared configuration collapses under real configured loss. This is
not explained.

One comparison was run to narrow it (`upstream-classic-g/`): the actual
`irlserver/srtla_send` binary at `df0b393` (`4.0.1 (df0b393) [srtla_send]`,
SHA-256 `11954307…3d51`) against our fork's `classic`, six same-session trials
under identical NAK-off (no freeze) plus Scenario G, both senders under the exact
manifest environment (`RUST_LOG=info`, `PATH=/usr/bin:/bin`, `env -i`).

| Sender | Settled (second) | Final 10 s Mbps | Received retx % | Non-model queue drops, links 0/1/2/3 | SRT receive drops |
|---|---|---:|---:|---|---:|
| upstream `df0b393` classic | 3/3 (12, 14, 15) | 11.801 / 12.747 / 12.284 | 17.369 / 15.086 / 15.184 | 158388 / 51736 / 4455 / 602; 203419 / 43205 / 262 / 127; 155348 / 66933 / 0 / 0 | 2519 / 923 / 1687 |
| fork `7e9a780` classic | 0/3 | 10.603 / 10.882 / 10.343 | 46.655 / 44.365 / 45.906 | 6207 / 12122 / 7690 / 9148; 5372 / 7565 / 9347 / 8323; 5869 / 8198 / 12303 / 11806 | 6222 / 5421 / 5699 |

Upstream settled where the fork did not, so there is a real behavioural
divergence between the two senders under degraded NAK feedback. But upstream
was not clean. It passed the settling gate by concentrating roughly
155,000–203,000 non-model drops onto the 1 Mbit link, whose entire 30 s
capacity is about 2,850 packets (roughly 55× oversubscribed), while the two
healthiest links carried near-clean traffic. It also recorded 923–2,519 SRT
receive drops per trial and dipped to 8.6–9.5 Mbps in individual final-window
seconds; only one of its three trials sustained the offered rate across the
final ten seconds. Our fork spread the load evenly and every link overflowed
moderately, so nothing carried clean. Upstream's lower received retransmission
fraction does not show fewer retransmission attempts, because qdisc-discarded
packets never enter that fraction. No pcaps were taken, so no NAK timing or
send-side retry count exists for either sender.

So "match upstream" is not automatically the correct target: upstream's result
is better for our gate, not obviously more correct. Two caveats keep this open.
First, `irlserver/srtla_send` at `df0b393` is itself Rust, not BELABOX's original
C `srtla_send.c`, so this comparison cannot resolve a Rust-vs-C divergence
shared by both Rust ports and says nothing about BELABOX field behaviour. Second,
candidate explanations for why the upstream ecosystem has not noticed (field
loss patterns differ from a 5% GE model; users tolerate degradation that our
binary gate calls failure; something in our setup does not generalise; a latent
problem nobody isolated) are all unconfirmed. This note does not invent a
resolution. The sender-side divergence is a separately scoped investigation
(section 9) and the receiver-baseline decision was deliberately kept independent
of it.

## 7. Interop mechanics (settled)

`SRTO_NAKREPORT` and `SRTO_REORDERFREEZE` are unilateral per-side receiver
options. Each end of an SRT connection sets them for itself; nothing about them
is negotiated in the handshake. A sender retransmits whatever a NAK requests and
neither knows nor cares whether that NAK came from initial loss detection or a
periodic follow-up. Changing our receiver's NAK policy therefore carries no
negotiation cost in either direction: our sender to a third-party receiver is
governed entirely by their configuration, and a third-party sender to our
receiver simply sees a different NAK cadence. This is unlike FEC, which is
genuinely negotiated (`SRTO_PACKETFILTER` plus a `receiver_supports_fec`
capability gate, a real two-sided handshake). A minor supporting point: SRT's
sender-side `SRTO_RETRANSMITALGO` "efficient" algorithm, default-on since 1.4.4,
is documented as working best when the receiver does send periodic NAK reports.

One caveat is worth keeping. "No negotiation cost" is not "no interop risk".
Enabling NAK reports changes feedback volume, so a third-party sender facing our
receiver must handle repeated NAKs and loss ranges without retransmission,
CPU, or logging amplification. That deserves one cross-pair test rather than an
assumption; it was not run as part of this investigation.

## 8. Retired reasoning: pedigree is not correctness

When `classic`/L2 was first chosen as the bench baseline, part of the
justification was that it "matches upstream's one-and-only mode, strongest
evidence pedigree". That reasoning is explicitly retired. Pedigree shows a
configuration is *used*; it does not show the configuration is *correct* under
our conditions. The G factorial outranks it. `46170c6` already made this
correction in `AGENTS.md` (BENCH RECEIVER-PROFILE DEPENDENCY, "Retired rationale")
and `README.md`; this note is consistent with both and adds the evidence behind
them.

A related confusion is also retired: the name collision between the sender
`classic` scheduling mode and the receiver `classic` preset implies a coupling
that does not exist. The receiver cannot observe the sender's scheduler, and G's
failure crossed `classic` and `enhanced` identically. Sender `--mode classic`
does not select receiver L2 or NAK-off, and no document should say it does.

## 9. What remains open

- **B1 and C are unresolved** under the shipped freeze-only baseline. They settle
  only under NAK-off, which is unsafe on A and G. No discriminator that would
  permit a scenario- or topology-keyed policy has survived testing.
- **The sender-side divergence versus upstream is uninvestigated.** Our fork and
  `irlserver/srtla_send@df0b393` distribute load very differently under degraded
  NAK feedback (even spread with moderate overflow everywhere versus
  concentration onto the marginal link). Which behaviour is actually right has
  not been determined, and the comparison cannot speak to BELABOX's C sender.
- **Campaign C1's scenario mix** pairs homogeneous-delay scenarios (A, G, H, F)
  with heterogeneous ones (B1, B2, C, D). Whether a verdict averaged across both
  regimes has one coherent real-world referent is an open question. The spread
  hypothesis would have answered it; its falsification means it stays open, and
  it is not grounds for changing the mix now.
- **The live-production receiver migration** implied by the reconciliation
  (collapsing L1/L2 into one canonical bonded policy, keeping port 9001 and the
  `classic` preset as a deprecated compatibility alias, dropping the
  "bandwidth-saver" claim) belongs to a different repository and owner. Nothing
  in this repo implements or schedules it.
- **One cross-pair interop test** for a third-party NAK-off sender against a
  NAK-on CeraLive receiver (section 7) has not been run.
- **Hardware validation.** Every number here is a small-N netns observation
  under host load above 2 (the existing load-warning limitation applies to all
  trials, clean and failing arms alike). None of it is bonded-hardware evidence.
