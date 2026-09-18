# Todo 33 — one bounded round, no admitted scheduler repair

**Outcome: no candidate met all five admission predicates.** This does not prove
the schedulers optimal, all receiver drops unavoidable, or M4 acceptable for shipping.
No sender behavior, constant, threshold, premature-NAK path, campaign observation,
candidate lock, or reference manifest was changed. No fix/revert pair was made.

## What was investigated

Read the M4 verdict and the recorded empty-covering-set finding first. All 100
primary candidate cells fail the all-runs zero-drop/belated gate. Enhanced is the
fallback at 0% genuine coverage; D is exempt and the other 19 scenarios sacrificed.
The independent median-goodput winner is descriptive, not proof of gate coverage.

| Mode | Predeclared cells | Lost to median-goodput winner | Scope |
|---|---|---|---|
| classic | none | not assessed | Excluded by design |
| enhanced | F, G, M1 | F, G, M1 | Quality, decay, burst/cache, incumbent validity |
| rtt-threshold | B1, B2, M2 | B1, B2 | Grouping, sample validity, saturation fallback |
| edpf | B1, B2, M2, M3 | all four | Bootstrap, BLEST escape, IoDS reset, BDP ranking |
| adaptive | none for performance | not assessed | No new correctness exception established |

M2 is an RTT-threshold goodput winner even though it fails the hard gate. It was
not rerun. Both interpretations of “lost” are preserved in [none.json](none.json);
no winner-cell rerun or performance-based admission was smuggled through the gate.

## Frozen observations, not newly measured results

These medians are from `docs/evidence/bpc/m4/a/summary.json`; all rows are N=5.
Every row's zero-drop/belated pass rate is 0. The summary's gate values, not a
comparison of these medians to zero, decide all-runs coverage.

| Mode / scenario | Useful Mbit/s | Settled | Receiver drops | Belated |
|---|---:|---:|---:|---:|
| enhanced / F | 12.804154 | 5/5 | 0 | 64 |
| enhanced / G | 12.802750 | 5/5 | 0 | 289 |
| enhanced / M1 | 8.914759 | 5/5 | 5696 | 2150 |
| rtt-threshold / B1 | 4.606585 | 0/5 | 21364 | 584 |
| rtt-threshold / B2 | 7.776917 | 2/5 | 20877 | 342 |
| rtt-threshold / M2 | 7.934954 | 5/5 | 305 | 406 |
| edpf / B1 | 4.651504 | 0/5 | 20867 | 525 |
| edpf / B2 | 8.383329 | 4/5 | 18293 | 623 |
| edpf / M2 | 7.927935 | 5/5 | 357 | 365 |
| edpf / M3 | 7.840202 | 5/5 | 955 | 674 |

F/G enhanced have zero median viewer loss and approximately their 12.8 Mbit/s
offered delivery. Their belated counts are not an isolated ranking failure.
B1/B2 losses are substantial and **not dismissed as purely environmental**.
Baseline loss and receiver recovery alone cannot establish which fraction is
avoidable. The permitted evidence did not isolate an implementation violation.

Scenario source: `crates/network-sim/src/scenarios.rs:28-41,84-108` (B1/B2, F/G)
and `scenarios/dynamic.rs:27-97` (M1 capacity/RTT/loss ramps, M2 link outage,
M3 DATA blackhole). The inherited LTE baseline includes 0.2% loss. No scenario
definition was changed to make a mechanism or gate pass.

## Claim probes and five-predicate assessment

Actual pre-existing claims are in README's **Enhanced Mode (Default)**,
**RTT-Threshold Mode**, and **EDPF Mode** sections. Enhanced's more precise
contracts appear at `selection/enhanced.rs:3-10,18-34`: NAK-aware score and
deliberate cooldown/hysteresis, not guaranteed global-optimum traffic shares.
The shared product is computed once at `selection/admission.rs:63-94`, and the
quality formula/cache are `selection/quality.rs:66-113` and
`connection/mod.rs:656-665`. No sign reversal or duplicated multiplier was found.

Three new diagnostic unit probes were appended **only to the existing test file**
in a detached scratch checkout of embedded base `87328c6`:

1. `defect_probe_enhanced_sustained_loss_lowers_rank`: 100 deterministic 50ms
   refreshes with repeated loss past startup grace. A clean equal-capacity link
   wins; burst quality is 0.35 and enters the shared product once. **PASS**.
2. `defect_probe_rtt_split_falls_back_at_zero_fast_score`: measured 40/240ms split,
   fast path at zero score, cooldown expired. Production dispatch selects the
   usable slow path. **PASS**.
3. `defect_probe_edpf_bootstrap_survives_ordering_reset`: all-unmeasured 40/120/240ms
   paths, IoDS primed above every arrival. The pipeline resets ordering and selects
   the lowest-delay path rather than returning an empty pool. **PASS**.

`cargo test --lib defect_probe -- --nocapture`: **3 passed, 0 failed**, 0.01s
test execution. No production patch preceded or followed these probes. The test
patch and transcript are retained in the local task receipt; the scratch was
restored and removed. These passing probes cannot be relabelled failing-first TDD.
The evidence checker separately follows red-to-green CLI testing.

Existing tests additionally cover recent-versus-old NAK quality, burst penalties,
grace-period edges, invalid incumbent release, RTT overshoot, EDPF congestion
escape, queued backlog, deterministic mode traces, and all-over-BDP selection.
RTT saturation is a window score, not a physical-qdisc-capacity measurement;
EDPF's measured send rate is not an oracle for capacity. Changing either heuristic
to improve B1/B2 would need separate design/performance work, not this defect round.

For each investigated mode the predicate assessment is `[true,true,false,false,true]`:
a real old claim exists, unit tests encode it, no failing claim test at exact campaign
source was established, no code-defect causal chain was demonstrated, and the
off-limits surfaces remain untouched. **No record is admitted on partial proof.**

## Source provenance caveat

`receivers.lock.json.candidates["m4a-ours-new"]` records sender paths and SHA-256s,
but **none has `source_sha`**. The retained binary's verified SHA-256 is
`b88042354e9973c8ad4418a4c6da93a93eacf77e67e3e168651afbe41e7c2107`; its version is
`3.3.0 (feat/adaptive-scheduler-bench@87328c6-dirty) [srtla_send]`.

The initial Todo 20 receipt also names `87328c6`; `src/`, Cargo manifests/lock and
build.rs have no committed diff between that base and `41dcb50`. Harness sources
did change. This justifies source-base diagnostic probes, **not** manufacturing an
exact source commit for a dirty build. `artifacts.ours-new.source_sha` is libsrt's
commit, not the sender's. `pre_fix_source_sha` stays null in this **none** record.
An admitted defect would still require exact source provenance and captured failure.

## Backlog and handoff

- **EDPF allocation claim mismatch, unadmitted:** README says allocation-free,
  while `blest.rs:33` and `iods.rs:28-43` allocate vectors. No allocation failure
  test or causal link to these cells was established. Do not turn this into an
  unsolicited optimization, second round, or permission to tune constants.
- **Provenance gap:** future defect admission must resolve the dirty campaign
  source, not backfill an unsupported SHA into the historical lock.
- **Reporter `live:false` support is not established:** current
  `report.py:139-148,756-802` resolves `supersedes` but does not inspect `live`.
  With zero reruns there is nothing to exclude. Any future reverted-run campaign
  must test and implement that exclusion before treating it as supported.

No rerun was warranted, so [reruns/summary.json](reruns/summary.json) has
`cells: []`, `runs: 0` and the native reporter's empty `schema_version/groups`
envelope. The additive envelope is necessary because the actual Todo 34 decider
requires those two fields; the requested bare two-key sentinel alone is not a
valid decider input. No original record is superseded; every M4 number remains
frozen. No post-fix build or superseding candidate-lock entry exists.

No classic/enhanced fix landed, so M1's rule was **not** reinvoked. Its recorded
TTL*=200 is unchanged, not a new before/after measurement. Todo 24's separate
M3 released-sender blocker remains outstanding; no Todo 24 receipt exists here.

Validate the disposition with:

```sh
uv run scripts/bench/pr_description.py --check-defects docs/evidence/bpc/defects/
```

This new command validates shape and caps, not historical truth or statistical
regression. It is the minimal Todo 33 checker, not the future Todo 29 PR generator
or spike checker. Adaptive correctness exceptions require separate normal TDD
and matrix smoke evidence and are not admitted by this mode-repair schema.

## Verification

| Gate | Result |
|---|---|
| Three fresh source-base claim probes | 3 PASS; no failing scheduler claim |
| Checker CLI regression suite | 34 PASS, after captured failing-first CLI test |
| `--check-defects docs/evidence/bpc/defects/` | PASS, zero admitted records |
| Empty summary | Equals native `report.Summary(groups=(), warnings=())`; unchanged decision reader accepts it |
| Changed Python error diagnostics / Ruff | Clean / PASS |
| `cargo build --release` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy -- -D warnings` | PASS |
| `cargo test --lib` | 941 PASS |
| Bounded `cargo test --all-features` | exit 101: 966 library PASS, bench 54 PASS/4 ignored; adaptive G demoted 35/60 and twins sustained-Healthy FAIL |
| Bounded `cargo test --features test-internals` | exit 101: 966 library PASS, bench 54 PASS/4 ignored; adaptive G demoted 36/61 and twins sustained-Healthy FAIL |

Both broad runs reached the actual privileged tests, not self-skips; restart
passed, later integration targets were not reached. These are the already-recorded
G/twin failure classes, not a new waiver or a full-green claim. No scheduler source
changed in this task and no additional live measurement was authorized as a repair.

The first broad attempts failed to compile the bench harness against a stale
`network-sim` artifact after the scratch build shared the target directory. Current
source did contain `stderr_line_count`; `cargo clean -p network-sim` removed the
stale artifacts and the single retry of each command compiled and reached the
failures above. Both first-attempt logs are retained; no source workaround was made.
