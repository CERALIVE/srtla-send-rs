2026-09-18 — classic/rtt-threshold/edpf/adaptive modes removed in 4.0.0 — read as history.
# Enhanced cooldown candidate — UNACCEPTED; behaviour reverted; tests retained (2026-09-16)

## Decision

The isolated candidate removes enhanced's unconditional 15 ms incumbent hold,
retaining quality scoring, 10% score hysteresis, ACK/NAK accounting, recovery,
flush-on-switch and batch sizing. It does **not** change the default mode or add
a flag. RTT-threshold and adaptive retain their cooldown. Enhanced's optional
exploration retains its separate 15 ms pacing: this is deliberately narrower
than removing every use of the constant.

Outright removal of the load-selection hold was the first candidate because
upstream `df0b393` independently made that choice and the earlier isolated G
diagnostic supported it. The August evaluation's `24b5f64` rejection was
dependency-only, **unmeasured**: the separate flush-removal step `0cc0c6d` failed
its goodput gate. That historical rejection does not measure this candidate.
No batch-flush implementation was changed here.

**Do not ship this candidate or restart Campaign C1 on this evidence.** The new
paired L2 trials do not reproduce the earlier G success. A load-aware cooldown
is now the alternative to evaluate, not an authorized arbitrary threshold: the
observations below do not identify a safe incumbent-overload bound. No such
condition was invented, and no commit was made.

## Deterministic evidence

Before the production edit, three new tests failed for the intended reason:
the incumbent won after its real queued DATA halved its score; all 16 packets
of a frozen-clock burst went to one of four equal links; and a 12% score
advantage could not overcome the timer. These pass with the candidate.

Tests additionally cover both quality settings, exploration on/off, switch ages
0/1/5/14/15/16 ms, below/above hysteresis, an invalid incumbent, an empty pool,
and real loopback packet forwarding. The latter verifies an immediate switch
flushes the previous partial DATA batch byte-exactly, including when the next
input carries the retransmission bit. Existing switch-flush failure/recovery
tests remain unchanged.

`src/tests/selection_mode_traces.rs` freezes 80-byte decision streams captured
**before** the fix for classic, RTT-threshold, EDPF and adaptive. All match after
the fix; they include queued load, switches, holds and a disconnected incumbent.
Existing adaptive/EDPF golden tests also pass unchanged. Four old assertions
explicitly requiring enhanced's removed hold were updated; exploration checks
still distinguish the best link from the exploratory runner-up.

## Paired corrected-receiver diagnostic

Twelve fresh trials, two per arm per scenario, interleaved by the existing seeded
manifest ordering (`20260913`). Each runs the real campaign Stack, source,
monotonic sink and unchanged settling predicate for **30 seconds**, continuing
after any early qualification. No retries, measurement timeline, full campaign,
or fabricated accepted records. F here is **enhanced/F**, not the earlier
receiver investigation's classic/F control.

Both arms use the committed receiver-baseline correction (`1c4679d`): CeraLive
SRT 1.5.6 apps with the local URI-option exposure, listener
`latency=2000&lossmaxttl=40&reorderfreeze=1&nakreport=0`. This is classic/L2,
**not** the production-default balanced/L1 profile. Actual shared-library
resolution was checked with `ldd`.

| Scenario/run | Baseline settled | Candidate settled | Baseline final goodput Mbps | Candidate final goodput Mbps | Baseline retransmit % | Candidate retransmit % |
|---|---|---|---:|---:|---:|---:|
| A/0 | no | no | 19.955 | 20.459 | 45.113 | 44.838 |
| A/1 | no | no | 19.791 | 20.575 | 45.049 | 44.660 |
| F/0 | no | no | 10.490 | 10.577 | 39.533 | 38.876 |
| F/1 | no | 10 s | 10.473 | 12.802 | 40.238 | 5.616 |
| G/0 | no | no | 10.989 | 10.806 | 36.691 | 44.935 |
| G/1 | no | no | 10.866 | 10.955 | 38.975 | 44.367 |

Goodput is the mean of sink seconds 20–29. Retransmit percentage is
`sum(pktRcvRetrans)/sum(pktRecv)` over retained interval CSV rows, not the exact
same ten-second window. It is a descriptive diagnostic, not an aligned loss
acceptance metric. N=2 cannot establish statistical non-regression.

G's final per-link **non-model netem drops** (`drops - dropped`, handle `11:`):

- Baseline/0: 7515 / 5536 / 6340 / 4560; candidate/0: 5900 / 9519 / 8218 / 10825.
- Baseline/1: 7393 / 6085 / 6092 / 6907; candidate/1: 4292 / 7842 / 6147 / 6039.

Neither G candidate sustains the offered 12.8 Mbps or eliminates companion
overflow. Both show a higher received-retransmission fraction. A improves
modestly but still has heavy overflow; candidate A/1 has more non-model drops
on all three links than its paired baseline. Only candidate F/1 has zero
non-model drops on every link. These are concrete reasons to withhold acceptance,
not to label every difference a proven population-level regression.

## Provenance and remaining gates

- Baseline sender revision `7e9a780a8e94b0c89dfb5b68b6448ca222faf12d`, SHA-256
  `4a11c169ec51c1343769e09dd753048a2d7f9cb33efc501e03e257c44313bde6`.
  `src/` is identical between that revision and pre-edit HEAD `1c4679d`.
- Candidate SHA-256
  `10327647b12415c4893d8f05a4843950bfe4bf0784756af843852d1e4df985bd`.
- SRT apps SHA-256
  `639c4a641ce84a1848d6527e7a0878fe9c7c73ebf6fff93bfbe16ce23dec7445`;
  loaded libsrt SHA-256
  `3f41e353927bb155e86903580995f24b5c5f2f2cb39bd78f024aee5a3ba71c0d`.
- CeraLive SRTLA receiver SHA-256
  `fa3524c844d1cd7f805fecc0b06fc68f1594c3a4b9e53cf27b7ea37a0c1e6ab7`.

`scripts/bench/build_candidate.sh` correctly refused the dirty worktree, which
already held an owner manifest edit. Its guard was not weakened, spoofed or
bypassed. The diagnostic instead used a fresh release + `test-internals` build
copied into a separate read-only artifact. That establishes diagnostic binary
identity, **not** the required clean, committed candidate-build provenance.

No CPU-per-useful-byte measurement, historical 20/60/120 ms paired rerun,
all-scenario non-regression, balanced/L1 validation or hardware validation was
completed. The old receiver versus corrected receiver was not factorially
retested here; the mismatch with historical G results is not sufficient to
attribute causality to one libsrt option. A bounded follow-up must resolve that
interaction and specify evidence for any load-aware bound before changing the
default scheduler's shipping policy.

## Final code checks

- Pinned `cargo build --release`: pass.
- `cargo clippy -- -D warnings`: pass.
- `cargo test --lib`: 898 passed, one existing hardware ignore.
- `cargo test --lib --features test-internals`: 923 passed, one hardware ignore.
- `sender::selection` suite: 30 passed before and after the edit.
- `cargo fmt --all -- --check`: only the two pre-existing, untouched
  `tests/netns_adaptive.rs` hunks (lines 24 and 135); no new formatting failures.
- Changed Rust files: no LSP errors or warnings; the selection module has only
  the normal inactive-code hint for `test-internals` being disabled in the LSP.
- Documentation reference checker and `git diff --check`: pass.

These checks do not override the live acceptance failure or the clean candidate
artifact limitation above. The candidate remains uncommitted.
