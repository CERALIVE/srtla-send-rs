# M4 — frozen first-pass protocol (provisional)

Declared before the first M4 measurement. No scheduler changes or policy tuning
belong to this campaign. TTL*=200, N=5 and coverage threshold0.95 come from the
committed M1/CIWIDTH outputs. K remains3 from M2.

## Scope and arithmetic

`m4a-ours-new.json`: five shipped CLI modes ×20 scenarios =100 primary metric
cells; upstream df0b393 classic/enhanced ×A/G/M1/M4 =8 reference cells; enhanced
M4 FEC off/on ×N3 =2 further reference cells. Total110 metric cells,
**546 planned metric indices** (500+40+6), not558: two N3 cells contribute6,
not18. Two separate600s M8 N1 soaks are non-metric. No m4b or4003 block is run;
Todo34 owns the post-defect, post-verdict lineage gate and reserved block.

Every CLI mode uses the same release+test-internals binary with no feature or
tuning environment override. All default shared feature bits are enabled.
Baselines use the locked real upstream binary, not an emulation, and are tagged
`candidate_class: baseline`, noncovering, with a distinct variant identity.
M6 carries the catalog's explicit +0.2/-0.2 positional priority sidecar for all
five modes. Scenario definitions, loads, durations, settling and recovery horizons
are unchanged. Primary receiver: SLT, production2000ms preset, TTL200, NAK-on,
periodic NAK gate-on, reorder freeze-on; source retains the ordinary caller preset.

The validator requires the exact full matrix and configuration before launch:
20 unique primary comparison groups, each with all five CLI candidates at N5;
all baseline/FEC variants noncovering. M8 is separately guarded by BENCH_SOAK=1.

## Frozen lineage-d1

`decide.py --rule lineage-d1` delegates to `lineage_rule.py`; the old `d1` and
ablation rules are unchanged. Only candidate-complete covering ours-new primary
groups enter the covering set. Missing/duplicate primary evidence fails, rather
than being converted into a negative performance result. Baselines, variants,
FEC, SLS and later lineage-gate rows cannot choose the winner, base or ship set.

Winner: maximum median useful goodput among the five CLI modes. Exact winner
ties prefer enhanced, then lexical order. Coverage requires complete same-index
pairing, CI-lower ratio≥0.95, loss difference≤0.1 percentage points, recovery
ratio≤1.10 for graded episodes, and no worse non-recovery rate. Existing paired
recovery semantics are retained: both-infinite recovery has unavailable ratio,
not fabricated1; genuinely zero/zero durations produce1. Required missing or
incoherent evidence cannot cover. Every planned observation must have settled;
metrics-v2 gates must pass on every post-settle run: zero packet-drop and belated
deltas and msRcvBuf minimum≥20% of the negotiated positive TSBPD delay.

The runner preserves full-window measurements after settling fails, marks them
failed/settle_timeout, and takes exactly one attempt per index. Reporting retains
those negative observations at N, exposes settled_rate, and never treats their
window as successfully post-settle. Infrastructure errors or absent full-window
measurements block reporting. This prevents selecting only successful attempts.
Ordinary non-M4 campaigns retain their existing successful-only semantics.

Scenario D is not pre-exempted. Only if **all five CLI candidates fail coverage**
on D does the mode-agnostic all-fail exemption remove that D obligation. Its
complete measurements, winner, per-candidate failing gates and exemption reason
remain in `cells`. No analogous exemption exists for adaptive/A or any other
scenario. No reusable exemption function was found in the referenced scenario
library; this explicitly implements the task's all-candidate predicate.

Ship set: smallest set covering all nonexempt primary obligations; ties prefer
fewer total tunables (the existing fixed TUNABLES table), then inclusion of
enhanced, then lexical tuple order. Base mode is independently the mode covering
the most obligations, with enhanced then lexical tie-break. Every obligation not
covered by base is a sacrificed cell even if another shipped mode covers it.
If no complete covering set exists, ship only base and mark all its sacrificed
cells `reason: uncovered`. Output includes per-candidate ratios and failing gates,
`refold:false`, `gate:null`. Byte-identical double execution is required.
The result is **provisional**, not authority to ship or retire any mode. Todo34
reruns this unchanged rule after the defect round and allowed fold-in.

## Supplements and process lifetime

FEC uses `packetfilter=fec,cols:10,rows:5` on both caller and listener versus
neither, enhanced/M4 only. Compare median useful goodput; an on/off ratio≤0.95
fails the no-regression≥5% rule. Its default `arq:onreq` means filter-confirmed
losses bypass the periodic-NAK gate; this pair does not test that gate.

M8: require complete600s observations, zero exited stack processes and zero
unrecovered links (all three links Healthy in the final recorded telemetry),
and no monotonically nonincreasing sequence of ten60s goodput buckets with a
strict overall decrease. Report the raw buckets and final health rather than
replacing unknowns with a pass. M7/G stderr rate uses actual lifetime-drained
line-counter deltas divided by observed run seconds, with RUST_LOG=info; bounded
log tails are diagnostic only and cannot supply this count.

First live launch uses `systemd-run --user`, never a tool-owned nohup background
job. P=1, entire runner taskset4-27, existing host measurement lock retained.
No exclusive-host claim: unrelated host load is recorded as before. Raw metric
records and CSV/telemetry retained; optional packet captures are off to avoid
exhausting the available artifact filesystem during this20h-scale campaign.
No required M4 metric uses a pcap. Immutable copied sender and benchmark-worker
binaries prevent later Cargo builds from changing a live experiment.

Smoke is a separate3-index A/FEC plumbing campaign followed by an identical
checkpoint replay. Freeze hashes are recorded before that launch. Main M4 uses
a separate fresh result root; a session loss resumes by waiting for the original
systemd process, not launching another. If the service itself dies, inspect owned
workers first and reuse identical artifacts/fingerprints/checkpoints.
