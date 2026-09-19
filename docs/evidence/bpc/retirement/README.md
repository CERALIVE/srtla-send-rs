# Todo 21 — 4.0.0 retirement verification, 2026-09-18

Implementation and ordinary serial Bookworm gate verified. **Native privileged
acceptance is NOT green:** the Enhanced version of the existing mixed-bond
route-removal test fails its per-survivor 100 kB/two-second assertion. No ranking
change, threshold reduction, ignore, or manufactured native-green claim was used.
This receipt is not a hardware-canary pass or permission to publish a release.

## Applied decision

```
uv run scripts/bench/apply_verdict_precondition.py
{"default":"enhanced","branch":"fallback","final_set":["enhanced"]}
```

The frozen zero-coverage empty-set outcome is accepted as specified. M5 is skipped
because the base is Enhanced; canary is pending, not pass. Neither artifact nor
the statistical rules were changed. Six real-CLI precondition tests passed after
their initial missing-script failures.

Retained enum: `Enhanced`. Deleted variants: `Classic`, `RttThreshold`, `Edpf`,
`Adaptive`. Classic/RTT selector files and `legacy.rs` are deleted. Adaptive's
separate selector is removed; only shared support remains under its original
paths. Shared tests execute the Enhanced dispatcher through direct test aliases.
Adaptive-only CLI/netns suites and mode-comparison equality tests are retired,
not weakened or reported as repaired G/twin behavior.

EDPF's module, BLEST/IoDS and regression files remain by the explicit owner
carve-out: Todo34 skipped fold-in and did not port E1/E2. Its unit tests call the
retained pipeline directly. The native namespace test uses the hash-verified
locked pre-deletion binary, not a hidden mode in 4.0.0:

```
timeout --foreground --kill-after=10s 90s cargo test --features test-internals --test netns_edpf -- --nocapture
[edpf] window=16s link0(30ms)=3841 link1(150ms)=4671 total=8512 first_half=4053 second_half=4459
test result: ok. 1 passed; 0 failed; 0 ignored
exit_code=0
```

The five pre-deletion 80-decision traces were frozen and verified before removal.
Enhanced's remaining dispatcher trace compares against its original fixture row.
No pre-shared-layer fixture, scheduler constant or Enhanced ranking formula changed.

## CLI, control and capabilities

Real executable tests prove all four deleted values exit2 through clap, naming
4.0.0 and `docs/release-notes-4.0.0.md`. Default enum, CLI, DynamicConfig and
status agree on Enhanced. All seven retired CLI flags warn once when explicitly
supplied, even when numeric values equal their defaults. Earned-ACK flag behavior
and its tests are unchanged.

RPC errors have code−32602 and `data.kind: retired_mode`. Retired quality,
exploration and RTT-delta RPC/text writes return the additive deprecation result;
snapshots/status and test-build effective_config remain byte-identical.
The exact hello and methods arrays remain frozen. Probe/live parity passes.

Released3.3.0 versus4.0.0 probe comparison was machine-checked: after removing the
explicitly exempt version value and the two new capability members, objects are
identical. The complete semantic diff is:

```diff
- "version": "3.3.0"
+ "version": "4.0.0"
+ "adaptive_scheduler": true
+ "link_priority": true
```

The two capabilities were added earlier on this branch. `adaptive_scheduler`
continues to describe shared health adaptation, not a supported mode spelling.

## Gate results and boundaries

All six required commands passed together, exit0, in a disposable **Bookworm
container**, pinned `nightly-2026-06-12`, `RUST_TEST_THREADS=1`,
`CARGO_INCREMENTAL=0`. Required build/test utilities were gcc, libc6-dev, binutils,
python3 and procps. The source and cached registry were mounted read-only, with
an isolated Cargo target directory. No host package or service was changed.

| Command | Result |
|---|---|
| `cargo build --release` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy -- -D warnings` | PASS |
| `cargo test --lib` | PASS:931 |
| `timeout --foreground --kill-after=10s 300s cargo test --all-features` | PASS:949 library tests plus all integration targets |
| `timeout --foreground --kill-after=10s 300s cargo test --features test-internals` | PASS:949 library tests plus all integration targets |
| `bash scripts/release_version_contract_test.sh` | PASS:4.0.0, both architectures, mismatched tag refused |
| Binding install/lint/typecheck/test/build under Bun1.4.2 | PASS:146 tests/413 assertions |
| Changed-code LSP error diagnostics | PASS |

Namespace tests in that ordinary container use their existing prerequisite
self-skip boundary: this is NOT a privileged pass. Native all-features and
test-internals both exited101 at
`removing_nat_default_stops_its_data_while_other_links_continue` in `netns_bond`.
The fixture previously invoked Classic; after migration it invokes Enhanced.
Example observation: link0 bytes611472→611552 while another survivor carries
480812 bytes; the assertion requires100000 on EACH survivor in two seconds.
Neither pre-existing status under Enhanced nor a new scheduler defect is claimed
without a paired baseline. The assertion remains intact. Serial container execution
also avoids an observed parallel loopback `premature_nak` ten-second timeout;
it does not prove that parallel-test flake repaired.

Earlier container attempts retained failures from disk exhaustion, absent Python,
absent external `kill`, and an obsolete equal-share telemetry expectation. Only
the last involved a test correction: the unchanged Todo32 producer deliberately
reports0 weight without capacity evidence, so the stale100% assertion now pins0.
The test helper also now sets internal snapshot quality directly rather than
using a newly retired public setter; otherwise it prematurely warmed the cache.
No producer/scheduler behavior was changed to satisfy either test.

## Real Bookworm install and downgrade

Built on Bookworm with the pinned nightly and packaged through:

```
cargo build --release --offline --locked
bash ci/build-deb.sh amd64 <bookworm-target>/release/srtla_send <bookworm-target>/debs
```

The earlier cached3.3.0 package came from the released M3 interoperability input,
not a relabeled new binary. In a fresh `debian:bookworm-slim` container:

```
+ getconf GNU_LIBC_VERSION
glibc 2.36
+ apt install -y ./srtla-send-rs_4.0.0_amd64.deb
Setting up srtla-send-rs (4.0.0) ...
+ srtla_send -v
4.0.0 [srtla_send]
+ apt install -y ./srtla-send-rs_3.3.0_amd64.deb --allow-downgrades
Unpacking srtla-send-rs (3.3.0) over (4.0.0) ...
Setting up srtla-send-rs (3.3.0) ...
+ srtla_send -v
3.3.0 [srtla_send]
```

Both apt steps and both version executions succeeded. A superfluous trailing
dpkg-query diagnostic had an unescaped shell variable and failed AFTER this
proof; its error is retained, not represented as a successful command.
Both package control records have `Conflicts` and `Replaces` equal to
`srtla (<< 2026.6.2)`. Plain SemVer, no epoch. External versions.yaml remains
pinned atv3.3.0; no release asset, tag or retrieval pin was changed.

Verified archive SHA-256:

-4.0.0 amd64: `1d71c2a722b0a95108e0706f0e319aae65e3e9455a6bcab132d640f37180db0d`
-3.3.0 amd64: `88074aca7ec959504b0b0d028183758e62156c53120ba4edd40c22a3508fffa6`

## Historical execution lock

Added historical:true to smoke, M1, M2, M3, M4a and M4-soak manifests. The
owner-protected C1 manifest is unchanged (including its8/8 path-replacement WIP),
but requires_history detects its retired-mode arguments and fails closed too.
The real wrapper invocation without opt-in emitted:

```
Error: historical campaign requires --historical-from-lock; current binaries are forbidden
exit_code=101
```

Five lock tests cover explicit opt-in, omitted-marker bypass, selection of the
LOCKED path rather than manifest path, changed-byte rejection, and absent-lock
rejection. The benchmark target passed62tests with its4 existing live entry
points ignored. Historical locks are read-only and verified again by workers.
No statistical rule, receiver lock or measured campaign payload was rewritten.

## Review scope

New modules are small, typed boundary adapters; errors are explicit and owned
variants exhaustively matched. No new unsafe, dependency, protocol or routing
policy. Existing oversized dispatcher/test modules remain; this retirement
shrinks them rather than attempting an unrelated file-renaming/splitting campaign.
The EDPF evidence files are explicitly retained intact apart from their invocation
seams. The control client's existing253-pure-line module is a size follow-up;
its added status schema lives separately. This is not a claim of repo-wide size
or legacy-style compliance. Publication and hardware acceptance remain separate.
