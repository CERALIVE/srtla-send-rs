# S-TWINPORT predeclared method

Two sequential manifests, default (override UNSET) and `legacy-l2`. Each runs
enhanced/M1/ours-new N=3 per port, seed20260913, production latency2000ms,
unchanged90s/9.6Mbit scenario. Same-index port arms are seeded/interleaved.
Default server policy is TTL200 (M1's choice); rollback intentionally restores
TTL40/NAK-off/gate-off on BOTH listeners. Reuse the immutable M3 current sender
(production sender code has not changed), not a foreign or emulated binary.

SLS is the receiver; loopback player attaches to4000 at200ms. Player file byte
growth is the sole useful-goodput numerator. Post-settle delivered fraction uses
that growth divided by the top-level sender telemetry `bytes_sent_total` delta;
telemetry cadence100ms is recorded, with both raw edge snapshots retained.
Warmup uses90% offered for3consecutive seconds, at least10s, at most30s.
An un-settled run retains diagnostics but never supplies post-settle inference.

Player CSV is its OWN receive-side statistics, interval counters,100-packet
cadence. Window loss/drop sums must both be zero; time-weighted mbpsRecvRate
must be at least98% of offered (9.408Mbit forM1). Invalid runs are labelled
`player_leg_invalid`, discarded from inference, and rerun ONCE. Other failures
are not retried. No best-attempt selection or invalid result substitution.

The source/server/player path also runs the existing synthetic SLS20s/1Mbit
fixture once per port/profile: FOUR separate conformance configurations, not six.
These are labelled supplemental and cannot replace M1 performance observations.
All M1 attempts retain their registered/carry/latency assertions too.

Compare4003−4002 within each profile, paired by index, only with all3valid pairs.
Use the difference of arm medians of delivered fraction, jointly resampled
10,000times, fixed seed20260913, order-statistic95%bounds as in report.py.
The equality check is zero contained in that paired-difference CI (equivalently,
the observed difference is within its bootstrap error interval), not the vacuous
test that an estimate lies in its own confidence interval. `d_goodput_pct` is
100times the delivered-fraction difference, NOT a relative bitrate ratio.

Publisher loss uses ONLY a usable `/stats` publisher drop/received packet pair
with measurement-window semantics. Absolute rate difference must be≤0.05pp.
Raw authenticated responses at both window edges preserve unknown keys. If no
usable pair exists, classify DISTINCT; never substitute NAK counts, byte rates,
player-side packet counts, or inferred packet sizes. Unidentifiable deltas are
JSON null rather than fabricated numeric zero (the requested numeric shape has
no truthful encoding for this explicitly specified missing-pair branch).

Alias requires complete valid paired delivery evidence and a passing publisher
loss pair for both profiles. DISTINCT directs Todo20 to its reserved4003 block;
it does not assert the two source policies differ. No scheduler/server tuning,
source load reduction, reference-window shortening, or C1 manifest edit.

First launch is a transient `systemd-run --user` service with durable command,
start/end/exit logs, CPUs4–27, parallel_lanes1, and the normal measurement lock.
No campaign overlaps builds or another campaign. Gates run after measurement.
