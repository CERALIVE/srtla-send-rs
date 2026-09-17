# S-TWINPORT execution and gate receipt

**Decision: DISTINCT (conservative, not proof of different listener policies).**
Todo20 must execute its reserved 4003 block: `{enhanced,adaptive} × {M1,M4,M6}`.
[report.md](report.md), [spike.json](spike.json) and [summary.json](summary.json)
retain the verdict, every attempt, listener-policy proofs, byte edges and hashes.
The [method](method.md) was written before measurement.

## Execution

- Source HEAD1ca02bb, pinned nightly2026-06-12. Unchanged production sender:
  M3's immutable `target/m3-srtla_send`, SHA256
  `e01d301939d55cd05bc55bc8e2b68a15a4f7893bcadb36e992a405510682582d`.
- FIRST launch: `systemd-run --user`, unit `bpc-twinport.service`, invocation
  `35fd91e47c98497fa4c79518d84b2dc2`, launcher PID2239492. No duplicate launch,
  tool-owned campaign, or successful-attempt replacement.
- `pidwait -p 2239492` waited for the original process. Execution2026-09-17
  15:55:47–16:39:00−05:00; default1189.73s and rollback1402.33s. Both exits101
  are preserved:3 and6 missing successful indices, respectively.
- CPUs4–27, one lane, shared measurement lock. No builds during measurement.
- Raw root: `/home/andres/.cache/opencode/tmp/opencode/twinport-measurement`.
  `run.sh`, `campaign.log`, start/end/exit files and both profile trees retain
  exact commands, all results and complete artifacts.
-12 planned M1 indices +4 synthetic conformance indices +10 authorized reruns
  =26 attempts.19 fail the player gate, all for receive rate below9.408Mbit/s;
  player loss/drop is0/0 in all26. All26 settled.
- Default valid M1 coverage:4002 indices0,1;4003 index0. Rollback: neither port
  has a valid M1 index. One valid default pair, zero rollback pairs; no N=3 claim.

## Decision numbers and schema limitation

`d_goodput_pct=null`: no complete valid N=3 paired set. `d_loss_pp=null`: all26
live publisher responses expose the known23 keys including `pktRcvDrop`, but
NO received-packet denominator. There is no dimensionally valid loss rate.
The task's six-key shape is retained, but its numeric-value request is impossible
on this missing-data branch; null is truthful, not fabricated0. The supplied
jq acceptance predicate passes. No NAK-count, player-packet or bitrate-derived
publisher denominator was substituted.

For audit ONLY, final-attempt diagnostic Δdelivered_fraction×100:
- Default−1.6525565821202348; paired bootstrap95% interval
  [−4.304091552601818,0.5961948552858476].
- Legacy-l2+0.5509388551323047; interval
  [−2.527296720431532,1.6576028679501587].

These include discarded legs and **cannot be promoted into valid equivalence
estimates**, even though both diagnostic intervals include0.

## Conformance: FOUR configurations, not six

Four separately labelled20s/1Mbit synthetic SLS cells all passed:

| Override | Port | Registered | Carry | Latency | Publisher/player latency |
|---|---:|---|---|---|---|
| unset/default |4002|ok|ok|ok|500/200ms|
| unset/default |4003|ok|ok|ok|500/200ms|
| legacy-l2 |4002|ok|ok|ok|500/200ms|
| legacy-l2 |4003|ok|ok|ok|500/200ms|

All M1 attempts were also checked: default10/10 pass all three assertions;
rollback12/12 pass registration/latency, but **carry passes only1/12 attempts**.
That one rollback attempt still fails the stricter player-rate prerequisite.
Synthetic passes do NOT erase M1 failures or replace missing performance samples.
M1 publisher latency was2000ms; every player's actual receive CSV shows200ms.

Every server log proves BOTH listener profiles: default freeze1/NAK1/gate1/
TTL200/FEC1; legacy-l2 freeze1/NAK0/gate0/TTL40/FEC0. Both ports actually carried
connections in both modes. SLS, not bare SLT, was the sink.

## Full gate (one pass after measurement)

Exact commands/logs/statuses: raw-root `full-gate.sh`, `gate-summary.log`, and
`gate/{name.command,name.log,name.exit}`. The gate ran in transient user service
`bpc-twinport-gate`. Wrapper completion is NOT an all-green claim; individual
command statuses below are authoritative.

| Gate | Result |
|---|---|
| `cargo build --release` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy -- -D warnings` | PASS |
| `cargo test --lib` |941 passed |
| bounded `cargo test --all-features` | exit101; feature lib966 passed; netns_adaptive G32/61 demoted; twins fail sustained Healthy |
| bounded `cargo test --features test-internals` | exit101; feature lib966 passed; G40/61 demoted; same twin health failure |
| network-sim library |110 passed,2 existing ignored |
| bench target |52 passed,4 explicit live tests ignored |
| bench-target Clippy | PASS |
| report / decide self-tests |25 /15 passed |
| selected M1/M2/M3 + TWINPORT Python tests |39 passed, including9 new TWINPORT |
| Ruff / requested jq acceptance | PASS / PASS |
| changed Rust/Python LSP error diagnostics | all11 files clean |

Full feature failures match the documented branch failure classes; no new baseline
comparison or waiver is claimed. Twin final shares70.1587045029354%/49.245497389285614%,
but both fail the actual sustained-Healthy assertion. Receiver restart passes in
both. Neither suite timed out or self-skipped the failing privileged target.
No assertion was weakened. Binding/Loom/Miri not rerun: no corresponding changes.

The measurement and gate services finished; worker namespace teardown completed
and `sudo -n ip netns list` was empty. Raw evidence is intentionally retained.
C1's owner-edited manifest and unrelated srt-release.json remain excluded.
