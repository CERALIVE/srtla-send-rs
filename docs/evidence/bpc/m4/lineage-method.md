# Todo 34 — frozen lineage-gate method

The authoritative `lineage-d1` invocation uses only `a/summary.json` and
`../defects/reruns/summary.json`. Before lineage measurement, two invocations
produce the same bytes as `verdict-provisional.json`:
`b47ee5d6165f369df4859be6c33b6d5d923203c55ca3f6c0dc2da8cce943f4a0`.
No defect reruns exist. Enhanced is the empty-set fallback, not genuine coverage.
`foldin.json` records the single-member skip; steps 3–5 do not execute.

## Scope and execution

Generate `scripts/bench/manifests/m4b-lineages.json` from that authoritative
verdict with `scripts/bench/m4b_manifest.py`. One member means 18 candidate cells,
66 one-attempt runs: irlserver-prod A/B1/G/M1/M4/M6 at N5 (30 runs), ours-old,
irlserver-next and belabox M1/M4/M6 at N3 (27 runs), plus the DISTINCT-reserved
ours-new SLS port4003 M1/M4/M6 at N3 (9 runs). Total M4a+M4b = 110+18 = 128 ≤150.
All cells are noncovering; M6 receives the same +0.2/−0.2 sidecar on every lineage.
There is no M4a rerun, ablation, shortened window or second fold-in round.

The candidate is the immutable Todo20 `target/m4-srtla_send`, SHA-256
`b88042354e9973c8ad4418a4c6da93a93eacf77e67e3e168651afbe41e7c2107`.
Its embedded source metadata remains dirty; no invented clean build SHA is claimed.
Receiver policies reuse the recorded M2/M3 lock: old TTL40/freeze-off,
onsmith SRTLAPATCHES=1/TTL200. BELABOX uses its own SRTLA receiver with the
locked irlserver-next SRT sink alias. Ordinary SLT cells use port4001; the
reserved SLS cells use actual server port4003 and its default converged policy.

The FIRST campaign launch is a transient `systemd-run --user` service, with
P=1, taskset CPUs4–27, the existing whole-campaign measurement lock, no retries,
and a six-hour outer timeout. Controller and runner are distinct from Cargo
build outputs. Exact command, start/end, exit, unit invocation and frozen hashes
are retained. Wait on the original PID; never launch a duplicate after session loss.
No builds or competing campaign are intentionally run during measurement.

## One-shot rule

Joint passing runs must settle AND have received retransmissions / received
packets ≤0.10: at least2/3 or4/5. Additionally, every SLT outcome must satisfy
the frozen metrics-v2 zero-drop/belated and buffer-floor gates. Missing required
measurements cannot pass. Full-window settle timeouts remain negative observations,
not successful settling. Infrastructure failures or missing indices refuse reduction.

SLS stays `metrics:none`, outside primary coverage. Reuse the established player
settling and raw start/end captures, but NOT TWINPORT's special retry budget or
98%-player-rate equivalence predicate. Its three conformance assertions remain
visible and blocking. Belated and occupancy remain explicitly not applicable,
as declared by the SLS stats-map contract. The publisher API has no received-packet
denominator and does not establish windowed counters; **received retransmission
fraction is unknown**, not zero and not a player-leg/NAK proxy. Therefore the
literal retransmission condition cannot be proven for this block. Still run all
nine indices and record `retransmit_unknown` failures honestly. No server or SRT
source change to manufacture a measurement is part of this round.

Select the member with most passing lineage cells, then most primary coverage,
then retain the existing base. Never expand the ship set. Here only enhanced is
eligible: the swap decision is trivial, but its measured passes and sacrifices are
not. Every cell with no passing member is appended to `sacrificed_cells` with
`reason:"lineage gate"` and its lineage. Existing primary sacrifices are retained.
The separate `m4b_gate.py` reducer leaves `decide.py`, `lineage_rule.py` and
`report.py` byte-unchanged. Its final document includes detailed observations,
`lineage_gate:{cells,passes_per_member,swap,failures}` and `lineage_gate_swap`.
Run both the primary decider and final reducer twice and compare bytes; never
feed a lineage summary into the primary covering-set decision.

Full gate results and measured outcomes are recorded separately. Completion of
this method does not waive the existing G/twin failures or imply shipping fitness.
