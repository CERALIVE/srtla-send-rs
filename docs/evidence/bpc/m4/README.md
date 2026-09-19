# M4 final — enhanced fallback, zero lineage passes, not shipping acceptance

## Todo 34 authoritative verdict and lineage gate — COMPLETE

The authoritative primary decider consumed ONLY `a/summary.json` and the empty
defect rerun summary. Two executions were byte-identical to the provisional
SHA `b47ee5d6…f4a0`: enhanced only, base enhanced, zero genuine coverage.
[foldin.json](foldin.json) records the requested single-member skip; no donor
term, ablation, scheduler source, default, or frozen primary rule was changed.

The real lineage campaign measured **18/18 cells, 66/66 original indices**,
including all nine required DISTINCT SLS4003 indices. **65 settled; one
irlserver-prod/A index timed out and retained its full-window measurement.**
All15 SLT cells fail the strict zero-drop/belated gate. All9 SLS runs settle
and pass registration/carry/latency conformance, but the publisher's missing
received-packet denominator cannot prove retransmission≤10%; it is unknown,
never inferred from player/NAK counters. SLS belated/occupancy remain not applicable.

```json
{"ship_set":["enhanced"],"base_mode":"enhanced","covered_by_base_pct":0.0,"passes_per_member":{"enhanced":0},"lineage_gate_swap":null}
```

**All18 lineage cells are sacrificed**, each with `reason:"lineage gate"` and
its actual lineage. The final verdict has37 sacrifices (19primary +18lineage).
One member means no alternative base exists; zero passing cells is not an
argument to add another mode. The final set is fixed for this round, not a
claim that enhanced passed the performance gates.

| Lineage / sink | Cells | Runs | Settled | Passing cells |
|---|---|---:|---:|---:|
| irlserver-prod / SLT | A,B1,G,M1,M4,M6 |30|29|0/6|
| ours-old / SLT | M1,M4,M6 |9|9|0/3|
| irlserver-next / SLT | M1,M4,M6 |9|9|0/3|
| belabox alias / SLT | M1,M4,M6 |9|9|0/3|
| ours-new / SLS4003 | M1,M4,M6 |9|9|0/3|

Full per-cell quotas/failures: [b/report.md](b/report.md). Every run's received
retransmission fraction and gate observations are in [verdict.json](verdict.json)
and [b/summary.json](b/summary.json), which is lineage-gate-only evidence and
must never be fed into the primary decider. Two final reductions match exactly:
SHA256 `5884fe8ac54d9271a08bb664a3418b04d436945d4674d734e5fe2ff61dc38029`.

### Durable execution and retained corrections

Both launches used **systemd-run --user from the start**, P1/taskset4–27, the
host measurement lock, immutable M4 sender and copied runner, six-hour bound,
and one attempt per index. No builds or competing campaign were launched during
measurement. First start07:31:07−05 to final end09:04:06−05 on2026-09-18:
**1h32m59s elapsed; 1h30m17s active service time** (3833s +1584s).

- First service `bpc-m4b-lineages`, invocation0a01ab8ef88c4e2ca31af873d891ecdc,
  PID3592329, measured48indices then stopped before SLS preparation because the
  launcher omitted `SLS_BIN`. The controller now resolves and hash-verifies it
  from the existing lock before starting any measurement.
- Checkpoint continuation `bpc-m4b-lineages-resume`, invocation
  6311c434771240d6b742869f8e7de795, PID441561, measured the remaining18 first
  attempts. The49 pre-resume receipt files (48outcomes +one exhaustion receipt)
  remained byte-identical. No measured index was retried or replaced.
- The reducer initially copied G's60s privileged-test duration instead of its
  **45s reference-catalog duration**. Corrected checker metadata to the unchanged
  source and raw windows, with a failing-first regression; no scenario shortening.
- SLS windows report actual elapsed milliseconds. Generic exact-window equality
  rejected normal sub-second differences after all runs completed. The SLS-only
  integrity path now retains those actual windows, already bounded by the catalog
  duration check, and checks configuration/provenance independently. SLT exact
  window checks remain; regression tests cover both. No pass threshold changed.
- Original runner exits101 and controller exits1 remain visible in both archived
  receipts. Final offline reduction exits0 twice. Do not confuse corrected
  reporting with a rerun or claim that the original controllers exited cleanly.

Portable hashes/receipts: [b/provenance.json](b/provenance.json),1259raw files;
raw root `/home/andres/.cache/opencode/tmp/opencode/m4b-measurement`.
The premeasurement [lineage method](lineage-method.md) remains byte-unchanged;
original and continuation frozen hash lists plus final reducer hashes retain
the correction history rather than rewriting it.

### Todo 34 full gate

Release build, fmt, Clippy,941library tests,56bench tests, report25/decide15
self-tests and changed-code LSP diagnostics pass. Before measurement, selected
Python52tests+3subtests passed. After the two checker fixes,43tests+3subtests
passed (18lineage tests plus25existing reporter tests); Ruff and shell syntax pass.
Both literal bounded feature gates exit101 at **known adaptive G/twin failures**:
G38/61 and43/61demoted, twins fail sustainedHealthy; both feature libraries966PASS.
No assertion, policy, threshold, or full-green acceptance waiver was changed.

## Todo 20 historical measurement record

The real campaign ran sequentially in the original detached user service
`bpc-m4.service` (invocation `f651750ad19945fd882fb0b7580345ba`). No session
restart, duplicate campaign, retry-to-success or cell-count reduction occurred.

| Item | Measured result |
|---|---|
| Start | 2026-09-17 17:34:38 −05:00 |
| End, including reduction | 2026-09-18 06:18:06 −05:00 |
| Total | **45,808 seconds / 12h 43m 28s** |
| Main110-cell matrix | 44,551 seconds / 12h 22m 31s |
| Two600s M8 soaks, including setup/teardown | 1,247 seconds / 20m 47s |
| Main indices | **546/546**:501 settled,45 measured settle timeouts |
| Main runner / reporter | exit101 (unsuccessful settling retained) / **exit0** |
| Soak runner | exit0 (both full windows completed; acceptance below fails) |

The task's16–20h estimate was not a minimum duration. Actual completion used every
declared cell and full catalog window. The arithmetic is500+40+6=546 metric runs,
not558; two FEC cells atN3 cannot contribute18. Two M8 runs bring the total to548.
The separate three-index resume smoke is not measurement evidence.

## Matrix and frozen-rule proof

Preflight output:

```
metric_cells=110 (cap 150; lineage gate + reserved block ≤ 36 reserved for todo 34)
primary_groups=20 cli_cells=100 baseline_cells=8 fec_cells=2 metric_runs=546 soak_runs=2
```

All20 primary groups are candidate-complete across the five CLI modes atN5.
Baseline/FEC cells are explicitly noncovering; all baseline candidates stay outside
winner/base/ship-set calculations. The summary contains26 groups/110 metric cells,
546 indices and **zero integrity errors**. Neither m4b nor4003 was run in Todo20;
both are now measured by Todo34 above.

The [frozen rule/binary ledger](frozen.sha256) was written before the first live
smoke and passed again after the full campaign. Two independent decision CLI
executions produced identical bytes and SHA-256:

`b47ee5d6165f369df4859be6c33b6d5d923203c55ca3f6c0dc2da8cce943f4a0`.

The requested CLI-only ship/base and sacrificed-array jq assertion passed.
Launch and double-run receipts: [measurement](measure-service.txt),
[checkpoint smoke](smoke-service.txt). The repeated comparison output is not an
additional verdict; only [verdict-provisional.json](verdict-provisional.json) is
the retained first-pass decision.

## Provisional verdict — read the terminal reason

```json
{"ship_set":["enhanced"],"base_mode":"enhanced","covered_by_base_pct":0.0,"refold":false,"gate":null}
```

**No mode covers any primary scenario under all frozen gates.** Enhanced is the
base only because every mode ties at zero coverage and the frozen tie-break picks
enhanced. No complete covering set exists, so the terminal rule returns that base.
This is not evidence that enhanced passed, and not authority to ship it or retire
any other mode. Todo34's authoritative unchanged-rule rerun is recorded above.

All100 primary mode/scenario cells fail the requirement that every run have zero
receiver drops and belated packets. Goodput alone cannot override that gate.
Other failures include settling, paired goodput uncertainty, loss, recovery and
the buffer floor. Missing/failed predicates were not removed after measurement.

**Scenario D is exempt, explicitly:** all five measured CLI modes fail coverage.
This does NOT mean all25 runs failed to settle: classic/adaptive/RTT settled5/5,
enhanced4/5 and EDPF0/5. The frozen exemption concerns complete coverage, not only
startup settling. D's measurements and every failing gate remain in `cells`.

All other19 scenarios are listed in `sacrificed_cells` with `reason: uncovered`:
A, B1, B2, C, E, F, G, H, I, J, K, L, M1–M7. Selected examples:

| Scenario | Winner by median goodput | Enhanced/winner median ratio | Enhanced CI-lower | Important additional gates |
|---|---|---:|---:|---|
| A | rtt-threshold |0.912385|0.515046|settling, goodput, loss|
| B1/B2 | enhanced |1.000000|1.000000|zero-drop/belated alone still blocks|
| C | classic |0.976030|0.757954|goodput, loss, recovery|
| J | enhanced |1.000000|1.000000|buffer floor|
| L | adaptive |0.897581|0.734953|buffer floor, goodput, loss|
| M7 | rtt-threshold |0.975364|0.966959|loss, recovery|

The receiver gate also blocks all these rows; the table does not claim its
additional-gate column is exhaustive. Full per-candidate details are in the JSON.

## Supplements

### FEC: PASS, narrowly

Enhanced/M4, N3 each, same filter on caller and listener:

- no FEC median: **8,000,578.133bps**;
- `fec,cols:10,rows:5` median: **7,614,902.400bps**;
- ratio **0.951794017**, regression **4.820598%**, below the frozen≥5% failure edge.

This is the declared median rule, not a claim of statistical equivalence or a
goodput improvement. Default `arq:onreq` allows filter-confirmed losses to bypass
the periodic-NAK gate, so this pair is not a validation of that gate.

### M8: FAIL for both modes

| Mode | Crashes | Unrecovered links | Final health | Monotonic decline | Result |
|---|---:|---:|---|---|---|
| enhanced |0|3|all three Degraded|false|FAIL|
| adaptive |0|3|all three Degraded|false|FAIL|

Both ran the full600s and kept the four observed stack processes alive. Under
the predeclared final-Healthy criterion, Degraded counts as unrecovered; this
does not claim zero DATA carriage. Ten60s delivered-goodput buckets are retained
verbatim in [supplements.json](supplements.json). The required
`crashes=0 unrecovered_links=0` acceptance is **unmet**, not waived.

### G/M7 stderr: PASS on the literal requested stream

All60 records (50 CLI plus10 upstream G references) measured0 stderr lines/s
after startup atRUST_LOG=info, below2. The lifetime counter survives64KiB-tail
truncation. This is a **stderr-only** assertion; it is not a bound on combined
stdout/stderr logging or proof that the sender emits no status logs.

## Gates and limitations

Full gates ran before measurement; no build/test contention was added while M4
held the measurement lock. Existing unrelated host load remains reflected in
report warnings; taskset4–27 is not an exclusive-core or idle-host certification.
No real bonded-hardware improvement is established by this netns campaign.

| Gate | Result |
|---|---|
| `cargo build --release` |PASS|
| `cargo fmt --all -- --check` |PASS|
| `cargo clippy -- -D warnings` |PASS|
| `cargo test --lib` |941 PASS|
| bounded `cargo test --all-features` |exit101: known adaptive G42/61 demoted and twin sustainedHealthy failure; library966 PASS|
| bounded `cargo test --features test-internals` |exit101: known adaptive G38/61 and same twin failure; library966 PASS|
| bench harness |54 PASS/4 intentionally ignored live entry points|
| network-sim library |111 PASS/2 existing ignored|
| report / existing decide self-tests |25 /15 PASS|
| selected Python tests |48 PASS,3 subtests|
| changed code LSP error diagnostics |21 files clean|

No test assertion was weakened and no full-green claim is made. The campaign is
complete; scheduler acceptance and the full green gate are not.

## Artifacts and reproducibility

- [a/report.md](a/report.md), [a/summary.json](a/summary.json): sole metric summary.
- [verdict-provisional.json](verdict-provisional.json): generated, never hand-edited.
- [supplements.json](supplements.json): FEC, M8 health/buckets and stderr rates.
- [provenance.json](provenance.json): phase timestamps, exit codes and SHA-256s of
  **9,919 retained raw files**, including checkpoints, CSVs and observation receipts.
- [method.md](method.md): premeasurement contract.

Machine-local raw root: `/home/andres/.cache/opencode/tmp/opencode/m4-measurement`.
Local execution receipt: `.omo/evidence/task-20-bonded-path-convergence.md`.
No receiver policy, scheduler defaults, tunables or reference scenario was changed.
