# Scheduler bench tooling

Campaign support for the adaptive-scheduler evaluation. The runner itself is
`tests/bench_scheduler.rs`; everything here prepares inputs for it or reduces its
outputs.

| File | Purpose |
| --- | --- |
| `build_externals.sh` | Builds the external receiver/tooling binaries a campaign dials. |
| `build_candidate.sh` | Builds a clean current revision into a content-addressed, read-only sender artifact. |
| `assert_smoke.sh` | Requires at least one of two successes in classic/A and enhanced/A; validates all twelve outcomes and provenance. |
| `report_smoke.sh` | Reports actual successful indices for two required cells, with all D and adaptive/A outcomes separately informational. |
| `preflight.sh` | Checks host prerequisites before a privileged campaign. |
| `probe_srt_stats.sh` | Inspects a listener's CSV statistics capture. |
| `report.py` | Fail-closed reduction of a results tree into `report.md` + `summary.json`. |
| `decide.py` | Statistics and retention decisions over a reduced campaign. |

## Smoke campaign

Run `BENCH_ARTIFACT_DIR=/absolute/artifact/root bash scripts/bench/build_candidate.sh`
from the checkout **before changing any files**. It builds release + `test-internals`
in `target-ti`, rejects tracked or untracked changes, and prints one tab-separated
`path SHA-256` row. Cargo output goes to stderr. Published `bins/srtla_send-<full-sha>-<16-hex-digest>`
files are copies, mode 0555, never Cargo-output hard links or overwritten artifacts.
Identical rebuilds reuse the same name; a changed revision gets a new name. Rebuild
for each later campaign revision, then update all candidate paths in the manifest.

`manifests/smoke.json` records one concrete local artifact, not a portable auto-build
instruction. Replace its binary/receiver paths with your built artifacts on another
host. The checked-in paths are repo-local ignored copies of the immutable d168aa6
sender and pinned receiver; no sibling checkout or machine-specific absolute path is
required by the manifest. It declares classic/enhanced/adaptive, CeraLive, production SRT, A/D, two runs,
seed 1. All modes declare the test build’s complete effective configuration, because
the metrics response exposes it even when the selected mode is not adaptive.

After preflight and a scoped sudo-refresh heartbeat, run:

```bash
BENCH_SMOKE=1 BENCH_MANIFEST=scripts/bench/manifests/smoke.json \
  timeout --foreground --kill-after=30s 3600s \
  cargo test --features test-internals --test bench_scheduler -- --ignored --nocapture campaign
bash scripts/bench/report_smoke.sh scripts/bench/manifests/smoke.json "$RESULTS"
bash scripts/bench/assert_smoke.sh "$RESULTS/summary.json" scripts/bench/manifests/smoke.json "$RESULTS"
bash scripts/bench/assert_smoke_test.sh "$RESULTS/summary.json" scripts/bench/manifests/smoke.json "$RESULTS"
bash scripts/bench/build_candidate_test.sh
bash scripts/bench/assert_smoke_scope_test.sh
```

Set `RESULTS` to the runner’s output directory and `BENCH_ARTIFACT_DIR` to the build
artifact root. Stop the heartbeat on exit. A/D retain their full 45s/75s windows.
The assertion script defaults to the standard smoke evidence paths. Summary v1
contains distributions, not individual records: the checker also reads current
`<cell>/run-<index>.json` files, excluding failure attempts and stale archives. It
checks positive **in-window receiver packet deltas** from normalized CSV rows,
not a nonexistent top-level `srt` field. Configuration and binary hashes must match,
and summary fingerprints/counts/goodput must agree with the underlying records.
Mutation tests operate only on disposable copies of real evidence.

### Owner-calibrated coverage (smoke only)

The [documented investigation](../../docs/notes/scheduler-evaluation-2026-09.md#known-limitation-adaptive-mode-baseline-topology-throughput-instability-scenario-a-discovered-post-todo-28)
recorded classic/A failing at the first campaign item, passing later in that campaign,
and passing a true isolated item. No sequencing/cleanup defect was demonstrated. The
owner therefore calibrated **measurement-path coverage**, not scheduler performance:

- Execute the unchanged six-cell matrix and retain both planned index outcomes in every
  cell. The existing attempt budget and the 30s/90%/three-consecutive-seconds settling
  predicate are unchanged.
- Require **at least one successful run out of two planned indices in EACH** of
  classic/A and enhanced/A. Zero success in either required cell fails,
  regardless of totals elsewhere. This is one-of-two coverage, **not a strict majority**.
- A second required index may exhaust on `settle_timeout`; other required-cell failures
  remain fatal. Every successful required record retains the goodput, receiver-packet,
  configuration and full-window checks.
- All D cells and adaptive/A outcomes are informational. Adaptive A/D cold-start settling
  failures reproduced in true isolation; smoke D is not Todo28's bounded recovery test.
  Missing outcomes, malformed identity, wrong binary/receiver hashes and invalid
  manifest/profile metadata still fail validation across all cells.

The raw campaign may exit nonzero because it still requires twelve successes; retain
that exit/log. `report_smoke.sh` supplies the separately calibrated verdict. It validates
the original tree first, then derives a temporary two-cell manifest and read-only
file-symlink view. `report.py --smoke-coverage` is accepted only for those two canonical
cells with runs2/seed1/campaignsmoke. It requires one or two actual successes and keeps
their **original indices**, including a lone index1. Without that explicit flag, generic
manifest-complete reporting is unchanged. C1/C2 remain strict; no waiver transfers.

Summaries contain the actual two-to-four required successes, never twelve invented
successes. Failed attempts remain in raw evidence and report warnings; `known-findings.json`
retains all four informational cells separately. Counts and medians are matched against raw
records by the checker. A pass proves measured path coverage, not reliable settling
on every attempt, low viewer loss, or calibrated performance confidence intervals.

`assert_smoke_scope_test.sh` uses disposable **synthetic checker fixtures**, never
campaign evidence, to exercise one-of-two coverage, zero-success failures, nonzero
indices, informational D/adaptive outcomes, provenance corruption, and the remaining
required-record mutation suite. Reporter tests prove default completeness is unchanged
and the smoke CLI preserves original indices. The real-data
mutation gate still requires a genuinely complete scoped campaign.

The final live run failed the earlier four-required-cell policy: classic/A2/2 and
enhanced/A1/2, but classic/D0/2 and enhanced/D0/2. Both adaptive cells were0/2. The owner
then authorized the final two-A-cell scope above and rescoring **that existing data**,
without another live run. It passes: the original exit101, failed records and indices
remain unchanged. D failed across all three candidates in this final campaign, but an
earlier enhanced/D success prevents generalizing that observation to every run. See the
[portable report](../../docs/notes/smoke-final-report-2026-09.md) and
[receipt](../../docs/notes/smoke-final-receipt-2026-09.json) for the accepted scope,
actual measurements and provenance. No C1/C2 criterion is waived.

`BENCH_MAX_RETRIES` means total attempts, not additional retries. A missing manifest
binary fails canonicalization before any attempt; `SRTLA_REC_BIN` is not a campaign
override. Use an existing but failing executable in a separate negative-QA manifest
to test retry/exhaustion. Exhaustion files retain `status: failed`; the filename and
campaign exit signal exhaustion. Never change these contracts merely to match an
obsolete example invocation.

## Ablation: the bench passes the env per candidate

A candidate is a binary plus the environment it runs under. The two ablation
variables are read **once at startup**, so they cannot be changed mid-run and
there is no CLI flag for them — the manifest candidate's `env` object is the only
way to set them:

```json
{
  "label": "adaptive-no-stall",
  "bin": "<BENCH_ARTIFACT_DIR>/bins/srtla_send-<git-sha>-<sha256-prefix>",
  "args": ["--mode", "adaptive"],
  "env": {
    "SRTLA_ADAPTIVE_FEATURES": "loss,queue,deadline,rejoin,sole,pref,ratecap",
    "SRTLA_ADAPTIVE_TUNING": "stall_attempts=16"
  },
  "effective_config": {
    "adaptive_features": ["loss", "queue", "deadline", "rejoin", "sole", "pref", "ratecap"],
    "adaptive_tuning": {
      "stall_attempts": 16,
      "loss_enter": 0.1,
      "deadline_hold_fraction": 0.5,
      "ratecap_loss_backoff": 0.85
    }
  }
}
```

The candidate binary **must** be built with `--features test-internals`. Without
that feature neither variable is read at all — the gate is compile-time, so a
release binary handed an ablation env silently runs the shipped defaults instead
of failing. `build_candidate.sh` builds with the feature for exactly this reason.

### `SRTLA_ADAPTIVE_FEATURES`

Which adaptive mechanisms the selector runs.

- A comma list drawn from
  `stall,loss,queue,deadline,rejoin,sole,pref,ratecap` — listed bits on, the rest
  off.
- Or one whole-value word: `all`, `none`, or `default`. These are **not** list
  members: `stall,all` is a startup error.
- **Absent means the shipped default**, never the all-bits set. A
  `test-internals` binary with no variable set runs exactly what the release
  binary runs, which is what lets a campaign claim it measured what ships.
- An unknown token, an empty value, or the wrong case is a **startup error**
  (non-zero exit). The run fails instead of quietly measuring a different
  configuration than the manifest recorded.

### `SRTLA_ADAPTIVE_TUNING`

`key=value` pairs overriding exactly four sweepable constants. Unlisted keys keep
their shipped value.

| Key | Shipped | Domain |
| --- | --- | --- |
| `stall_attempts` | `32` | integer `1..=10000` |
| `loss_enter` | `0.10` | `(0.0, 1.0]` |
| `deadline_hold_fraction` | `0.5` | `(0.0, 1.0]` |
| `ratecap_loss_backoff` | `0.85` | `(0.0, 1.0)` |

An unknown key, a malformed pair, or an out-of-domain value is a **startup
error**. `deadline_hold_fraction` also moves the gate's release threshold, which
stays at `0.8 ×` the hold fraction so a swept value keeps a hysteresis band
instead of inverting against a fixed second literal.

### Verifying what actually ran

Both resolved values are published as `effective_config` on the control socket:

```
$ printf 'metrics\n' | socat - UNIX-CONNECT:/run/srtla-send.sock
{"adaptive_features":["loss","queue"],"adaptive_tuning":{"deadline_hold_fraction":0.5,
 "loss_enter":0.1,"ratecap_loss_backoff":0.85,"stall_attempts":16},
 "cooldown_hold_count":0,"nak_count":0,"switch_count":0}
```

The same two members are logged by the `status` command. The runner records the
`metrics` reply into each `RunRecord`'s `sender.effective_config` and
`report.py` compares it to the manifest candidate's declared `effective_config`
by exact equality; a mismatch fails the run with `config_mismatch` rather than
publishing a measurement attributed to the wrong configuration.

The per-feature-bit selector behaviour these variables switch is pinned
independently, without any environment, by
`src/tests/adaptive_ablation_tests.rs`.
