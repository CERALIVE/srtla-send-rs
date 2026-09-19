2026-09-18 — classic/rtt-threshold/edpf/adaptive modes removed in 4.0.0 — read as history.
# Round 8: independent keepalive silence — negative A/D result

Date: 2026-09-15. Base: `bb1df19f58add402b16075415c514f62257eaa48`.
Disposition: **NO COMMIT; no verified A/D solution.** The experiment is retained
uncommitted for inspection, not approved for production. No further policy variant
was attempted after the fixed live matrix.

## What the code actually measures

`needs_keepalive()` depends only on connected state and elapsed time since the
last keepalive send, not DATA routing. Housekeeping checks it every second and
sends an extended 38-byte keepalive on the unpaced control path. `IDLE_TIME=1s`
is a minimum gap checked on a sampled loop: scheduling/I/O can extend actual gaps.
RTT's waiting flag is separately armed only when the last RTT measurement is
absent or over three seconds old; frequent DATA RTT samples suppress that arm.
Therefore `handle_keepalive_response()` returning `Some(rtt)` is **not** a
continuous record of keepalive replies. Its zero-ms rejection also cannot define
liveness on fast links. Neither the RTT algorithm nor keepalive wire format changed.

The narrow additional detector uses independently matched keepalive replies:

```
old DATA stall = attempts_since_proof >= 32 AND proof_age >= tau
new control stall = keepalive_silence >= 3 * IDLE_TIME
                    AND (proof_age unknown OR proof_age >= tau)
Stalled = old DATA stall OR new control stall
```

Tau remains clamp(4×sRTT,1000,3000)ms. Control age starts at the first successful
send if no reply has arrived; later sends do not restart it. Replies refresh the
receive timestamp independently of DATA and RTT. A fixed 16-slot pending ring
rejects unmatched, replayed, reordered-old, future and >10s timestamped echoes.
Bare two-byte replies can consume an outstanding send without creating RTT proof.
This is not authentication: bare replies have no identity, and the existing
accept-any-source socket policy is unchanged. Resetting the connection clears the
control evidence and pending timestamps.

OR, rather than replacing DATA evidence, is essential: a small successful control
packet does not prove that a large DATA packet traverses the path. Fresh DATA
proof still prevents control-only loss from marking an actively delivering link
dead. Recovery remains the existing DATA/probe-train contract; the narrow change
does not let keepalives rehabilitate a DATA-stalled link. Consequently an idle
control-stalled link also waits for DATA recovery evidence; standalone idle
control-path recovery is not implemented or claimed.

## Failing-first proof and preservation

Three real-UDP regressions were run before implementation: two failed
`left: Healthy, right: Stalled`; the healthy-reply control passed.
The failures cover zero DATA attempts after keepalive replies cease, and a late
DATA ACK resetting attempts to zero before silence. Production send, receive and
housekeeping paths are exercised with a deterministic clock. The completed patch
adds nine tests, including policy precedence, fresh DATA, zero-ms/bare replies,
replay/ordering, bounded pending storage, and recovery invalidation.

All existing health assertions remain; three existing literal constructors gain
only the optional control-age input. The sole tests and `sole.rs` are unchanged.
No admission relaxation, rate-based exclusion, threshold/settling-budget change,
campaign/harness edit, prior committed ACK/admission-fix change, or C1 relaunch.

## Live provenance

All dedicated runs use one immutable mode-0555 release/test-internals candidate:

- Sender SHA256: `1b413dea4dffdb9402e2bc6a66e86a21c8f5a86d6392e8592d567105607b79f1`.
- Test executable SHA256: `3d7a60a64d8b8c937fc3c9520dc799b1f6fffd4e4d6c2c46444e5d95371ce68b`.
- CeraLive receiver SHA256: `cee2c4c1a681bb7558d8cf7b5e0437963fdc5aa5acff47205cab02fa90f698e6`.

The unchanged netns fixture clears its environment. A per-invocation private
mount namespace supplies a launcher at the fixture's expected sender path, which
execs that immutable candidate with the original logging environment and no
adaptive overrides. External read-only metrics capture verifies all eight
features (including sole) and unchanged tuning for all nineteen launched senders:
10 D, 3 G, and 6 mapped/legacy Twins. Host executables are not overwritten.
Each invocation has the existing 360s bound. None self-skipped or timed out.

## Scenario D — ten fixed trials, detector only, sole ON

The original full 75s waveform and all assertions remain unchanged. First
`Stalled` **and zero-weight** is searched across all post-onset observations.
This conjunction is not the exact FSM transition time: sole fallback can retain
weight on a Stalled link. Recovery columns are offsets from restoration.

|Run|Onset s|Restore s|tau+2 budget s|First Stalled/0 delay s|Rejoining s|Healthy s|
|---|---:|---:|---:|---:|---:|---:|
|1|20.114|28.054|5.000|~8.035817428, after restore|2.925914253|7.969946909|
|2|20.092|28.194|5.000|not observed|9.553515675|18.678626132|
|3|20.026|28.125|3.116|1.820352418, timely|0.824842630|16.870709286|
|4|20.114|28.115|5.000|~30.583106387, after restore|0.620081313|not observed|
|5|20.202|28.188|3.176|2.778151750, timely|0.694689790|16.786735667|
|6|20.166|28.105|5.000|not observed|0.856529596|17.757244858|
|7|20.081|28.142|5.000|3.600134456, timely|1.615638279|5.643778004|
|8|20.173|28.178|5.000|not observed|14.532371681|not observed|
|9|20.234|28.052|3.000|not observed|6.319510124|21.337530637|
|10|20.172|28.198|5.000|not observed|13.303721433|not observed|

Timely offsets and recovery offsets are direct test-calculated values from
unrounded events, shown to nine decimals. Approximate late offsets subtract the
1ms-rounded onset logs (±0.5ms); all observations retain polling quantization.
Missing cases remain in the denominator and are censored by the 75s scenario,
not silently discarded.

**Timely detection: 3/10 under both the original tau+2 and fixed-five-second
comparisons. Complete D: 0/10. Post-full-rate sustained health: 0/10.** Bounded
Healthy recovery also fails runs2–6 and8–10. Run4 additionally fails feasible-load
recovery. Runs7/10 fail the unchanged keepalive-RTT-log-increment assertion (which
does not count every actual reply). Survivor-DATA and final-share checks pass.

Comparison: nearest default baseline2/10, historical4/5. This is one extra timely
run, not compelling improvement or a new paired causal result. Earlier claims of
an established80% current baseline must not be revived. The conditional admission
experiment was therefore not entered; no Scenario A or combined-D run is claimed.

### Why the proposed signal does not solve this failure class

Scenario D's `DataBlackhole` classifier selects IP lengths1024–2047 and drops the
1316-byte DATA frames; it deliberately passes the small keepalives. Independent
control probing detects complete path silence, **not this size-selective failure**.
The BFD-style separation principle is useful, but does not make the existing
keepalive share the DATA packet's forwarding fate.

The nine retained Stalled-transition lines all meet the original DATA condition;
none is uniquely enabled by control expiry. At the timely transitions in runs3,
5 and7, keepalive silence is1955/1804/947ms, below the3000ms deadline, while DATA
attempts are32372/32071/17097. Thus those timely detections are not evidence that
the new control path fired. One later run4 transition has3812ms control silence,
but5041 DATA attempts and1150ms proof age already satisfy the original detector.
This is bounded transition-log evidence, not an invented per-tick activation
trace or proof that every nondetection had the same counter-starvation mechanism.

## G and Adaptive Twins — three dedicated trials each

|G run|Marginal netdev bytes|Demoted snapshots|Verdict|
|---|---:|---:|---|
|1|3,795,488|48/61|FAIL demotion|
|2|4,003,842|39/61|FAIL demotion|
|3|4,075,636|32/61|FAIL demotion|

All pass the3MB carriage floor and fail the unchanged≤10% demotion criterion.
This matches a pre-existing failure signature, not proof of equal probabilities.

|Twins run|Stalled/0 after onset s|Rejoining after restore s|Healthy after restore s|Preferred share|Verdict|
|---|---:|---:|---:|---:|---|
|1|1.401455674|2.441565010|5.461959844|55.3802785%|PASS|
|2|1.727083403|2.777194872|4.761999684|53.1856834%|PASS|
|3|1.348859216|4.301759007|5.249128457|50.7535128%|PASS|

All unchanged mapped/legacy, priority snapshot and health assertions pass here.
Share is informational. The broader test-profile gates below still fail Twins
sustained health; these three release-candidate passes are not a claimed repair.

## Gate — NOT GREEN

- Pinned release build and Clippy `-D warnings`: PASS.
- Library:892 passed,0 failed,1 ignored.
- Both bounded feature suites:917 library passes/1 ignored, then
  `netns_adaptive`3 passed/2 failed/2 ignored. G demotion35/61 and44/61; Twins
  post-full-rate/final-window health fails; I passes. Both exit101 and later
  integration targets are not reached. Target durations269.85/273.58s.
- `scripts/netns_test_gate.sh`: first target fails G31/61 and Twins health;
  I passes (REG3 at17.415959609s, sink90 at19.871568565s, pre_healthy=true).
  Target3 passed/2 failed/2 ignored,274.11s; later targets not run.
- Whole-tree format check: FAIL only the two pre-existing, untouched
  `tests/netns_adaptive.rs:24/135` spans. Changed-file formatting is clean.
- All eleven changed Rust files: LSP error diagnostics clean.
- No Miri/Loom/TS-binding/hardware gate is claimed.

All four frozen before/after SHA256s match:

|File under `src/connection/`|SHA256|
|---|---|
|`wire_rate.rs`|`85b278d6104af2951bd6a05492879b4572d00a839d3e769ccadd52445f122f79`|
|`wire_rate/queue.rs`|`3cbec592cb060e3bfd55f4e123bda7b6a5b527b2656700a7d5d15dfc3b88b77b`|
|`wire_rate/search.rs`|`f9f15bff130b4e9bb65769d4473f0c9430f3ad808eb60f2ccc24796885bc755d`|
|`wire_budget.rs`|`ea61939c8b41dca211d5b473009f76d6b283d0d4e6b7bd0be959adfa57b5e7a6`|

`sole.rs` remains `4888ef78997d27c99bb11dc9690598add6d40a4f68083842e2710645e220154b`.
No code variation followed the live outcomes. No commit, push, C1 relaunch,
Todo31 checkbox change, threshold padding, or further architectural attempt.
