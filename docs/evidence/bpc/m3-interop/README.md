# M3 complete — released3.3.0 blocks receiver rollout on C

**Measurement complete, not universal interoperability PASS.** The receiver PR
is blocked by the frozen rule. **Todo24 must rerun M1's rule with scenario C
added before applying TTL*.** M1's current selection remains200; this task
neither reruns that selection nor changes any production policy.

## Launch and provenance

The FIRST campaign launch used `systemd-run --user`, transient service
`bpc-m3-interop.service`, invocation `7e06480251ec4478b452c974531620c9`,
with the sender worktree as WorkingDirectory and RemainAfterExit=yes.
The service ran independently of the tool session; a blocking `pidwait` waited
for the original launcher, without polling/relaunching. This session did not
die, so survival of an actual chat crash was not re-tested; tool-boundary
survival and natural completion are observed.

- Start: 2026-09-17 12:29:02−05:00; end: 15:20:17−05:00.
- Measured runner duration: **10,274.41s**, 43cells / 123indices.
- **91 settled,32 measured settle timeouts**; runner exit101 preserved.
- 120 performance observations plus3 scenario-I conformance observations.
- One lane, taskset CPUs4–27, ordinary whole-campaign host lock, one attempt/index.
- Source HEAD `6fb189b`; bench-only changes uncommitted during measurement.
- Current sender SHA256 `e01d301939d55cd05bc55bc8e2b68a15a4f7893bcadb36e992a405510682582d`.
- Released `.deb` SHA256 `88074aca7ec959504b0b0d028183758e62156c53120ba4edd40c22a3508fffa6`;
  extracted sender `ebe34ee2bb6c8d832801e20ec32adc7cf8aba9c544915de751efad14f8f1d77c`.
  Downloaded from v3.3.0; **built_from_source=false**.
- All binary/lineage identities are in `receivers.lock.json`; raw input hashes
  are in `summary.json.input_sha256`. No foreign source was modified.

Raw root: `/home/andres/.cache/opencode/tmp/opencode/m3-interop-measurement`.
It retains `run.sh`, start/end/exit records, campaign log, results, per-run raw
artifacts, `conformance-idle/`, and `gate/` logs. No failed attempt was replaced.

## Four quadrants

Existing senders include BELABOX, both irlserver configurations, and released3.3.0.
New sender is current enhanced. Counts below are individual runs, not final
sender/scenario decisions; a joint pass means settled AND retransmissions≤10%.

| Sender population | Receiver | Runs | Settled | Joint pass |
|---|---|---:|---:|---:|
| Existing | Old |48|25|6|
| Existing | New TTL200 |48|48|39|
| New | Old |12|3|0|
| New | New TTL200 |12|12|6|

All new-receiver performance arms settled3/3. Every same-sender median-goodput
ratio exceeds0.95. Failures below are therefore retransmission failures, not
goodput regressions or inability to connect. `report.md` includes every cell
and all three raw goodput/retransmission values.

| Sender | B1 | G | C | M1 |
|---|---|---|---|---|
| BELABOX C |PASS|PASS|KNOWN LIMITATION|PASS|
| irlserver Rust classic |PASS|PASS|PASS|PASS|
| irlserver Rust enhanced |PASS|PASS|KNOWN LIMITATION|PASS|
| Released3.3.0 enhanced |PASS|PASS|**RECEIVER PR BLOCKER**|PASS|
| Current enhanced |PASS|PASS|FAIL|FAIL|

### Blocker and known limitations, with numbers

| Sender/scenario | Retransmit % by index | Joint pass | Goodput new/old |
|---|---|---:|---:|
| Released3.3.0 / C |13.568125,14.301265,51.453458|0/3|1.751367×|
| BELABOX / C |39.957648,39.613453,26.378674|0/3|1.157876×|
| irlserver enhanced / C |12.615540,1.877045,33.688785|1/3|1.458924×|
| Current enhanced / C |33.907990,24.750434,28.501287|0/3|1.016398×|
| Current enhanced / M1 |11.900728,10.534844,13.075927|0/3|1.151481×|

**Foreign mitigation:** the receiver cannot detect sender lineage; the only
lever is TTL*/gate, which M1 chose. Do not revert policy for foreign-only
failures. Todo27 must carry these limitations into cutover documentation.
The separate owned rollout failure is a Todo24 blocker regardless of the
foreign results or the substantial median-goodput improvement.
The mandated `receiver_pr_blocker` action was performed by irl-srt-server todo 24 — the verbatim stated action, the evidence commit, and the residual carried into K13 are recorded in [`blocker-resolution.json`](./blocker-resolution.json).

## Conformance: eight PASS, one literal-echo FAIL

All three foreign configurations complete REG1/2/3 with both links and a fresh
receiver group after scenario-I restart. Three-second sustained sink recovery
after restart completion: BELABOX10.946s, irlserver classic11.865s,
irlserver enhanced11.941s. Restart completion lags are54/135/59ms, conservative
upper bounds on process kill/respawn within the2s budget.

BELABOX sends keepalives only when idle, so loaded I supplied no samples.
The explicitly separate ten-second idle supplement (56.01s total stack time
for all three configurations) preserved all original campaign observations.
Both Rust configurations pass literal38-byte echo equality on both paths.

**The task's literal two-byte-echo premise is false for this locked receiver.**
BELABOX emitted nine two-byte `9000` requests on capture0. Each reply was
**32bytes**, `9000` followed by30zero bytes; capture1 additionally saw one
32-byte keepalive. All captures have zero kernel drops. The exact-length/equality
check therefore remains **FAIL**, not silently relaxed to a PASS.
This does **not** establish an operational BELABOX liveness failure: its pinned
`srtla_send.c` updates `last_rcvd` before dispatch and returns on KEEPALIVE
without requiring `n==2` (lines381,419–421). Its loaded registration/restart
tests pass. Do not change the32-byte NAT-padding floor to manufacture exact
echo parity. A longer idle-liveness proof is outside this ten-second observation.

`conformance.md` has one PASS/FAIL row per sender/check. The optional commercial
BELABOX receiver and real cellular hardware were not tested.

## Verification

| Gate | Result |
|---|---|
| cargo build --release |PASS|
| cargo fmt --all -- --check |PASS|
| cargo clippy -- -D warnings |PASS|
| cargo test --lib |PASS,941 tests|
| bounded cargo test --all-features |**FAIL101**, existing adaptive G/twin signatures|
| bounded cargo test --features test-internals |**FAIL101**, same signatures|
| Bench target / bench Clippy |PASS,49 tests;4 explicitly ignored live entries|
| report.py / decide.py self-tests |PASS,25/15|
| M1/M2/M3 Python regression tests |PASS,59|
| Ruff on M3 modules/tests |PASS|
| report.py --m3-outcomes |PASS,exit0|
| Four-quadrant and released-pass-or-Todo24-blocker jq predicates |PASS|

Both feature gates reached real privileged tests and failed, not timed out or
self-skipped. G demoted46/61 and39/61 samples; twins failed sustained Healthy
after restoration (final preferred shares46.929% and51.300%). These are the
previously documented branch failure classes. No fresh paired baseline was
run here, no test assertion was weakened, and no full-green gate is claimed.
Loom/Miri and the unchanged TypeScript binding lane were not rerun for this
bench-only task. Host-load warnings remain in the summary; no idle-host or
real-bond-hardware performance claim follows from namespace measurements.
