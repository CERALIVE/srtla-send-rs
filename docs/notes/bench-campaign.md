2026-09-18 — classic/rtt-threshold/edpf/adaptive modes removed in 4.0.0 — read as history.
# Paired scheduler campaigns

`tests/bench_scheduler.rs` is an explicitly ignored Linux measurement harness,
not a default CI performance gate. Its unprivileged tests exercise the manifest,
ordering, checkpoint, fingerprint, configuration and source-control boundaries.
Privileged campaign validation is a separate step; these tests do not establish
hardware performance or live receiver interoperability.

## Manifest

`BENCH_MANIFEST` accepts inline JSON or a JSON-file path. Example:

```json
{
  "campaign": "comparison",
  "seed": 42,
  "candidates": [
    {
      "label": "baseline",
      "bin": "/absolute/baseline/srtla_send",
      "args": ["--mode", "enhanced"],
      "env": {},
      "stats_file": true
    },
    {
      "label": "candidate",
      "bin": "/absolute/candidate/srtla_send",
      "args": ["--mode", "edpf"],
      "env": {},
      "stats_file": true
    }
  ],
  "receivers": [{"name": "ceralive", "bin": "/absolute/srtla_rec"}],
  "cells": [
    {"candidate": "baseline", "scenario": "A", "receiver": "ceralive", "srt_profile": "production", "runs": 3},
    {"candidate": "candidate", "scenario": "A", "receiver": "ceralive", "srt_profile": "production", "runs": 3}
  ]
}
```

**`cells` is required and authoritative.** Optional top-level `scenarios`,
`srt_profiles` and `runs` are descriptive metadata, never a Cartesian-product
instruction or a substitute for per-cell counts. Unknown fields, scenarios,
presets, duplicate cells and unresolved candidate/receiver references are rejected
before network setup. Labels use ASCII alphanumeric, `_` and `-`, up to 64 bytes;
`--` is reserved. Cell IDs are
`<candidate>--<scenario>--<receiver>--<srt_profile>`.

Candidate `args` and `env` default to empty. Arguments are extra sender options,
not the four topology-owned positionals. The runner owns `--control-socket`,
`--stats-file` and bind-map paths; supplying those options is an error. It always
adds a control socket. `stats_file` defaults to false because upstream binaries
may not implement that flag. Enable it for telemetry-capable fork binaries.
The candidate starts under `env -i`, with the manifest environment plus explicit
defaults `RUST_LOG=info` and `PATH=/usr/bin:/bin`, both recorded in its fingerprint.

Optional candidate `effective_config` has the existing collector shape:
`{"adaptive_features": <sender-owned JSON>, "adaptive_tuning": <sender-owned JSON>}`.
Either field may be omitted. This is the expected **complete** observed configuration,
not a partial matcher. Adaptive overrides require this explicit expectation; the
runner does not derive future feature/tuning schemas from CLI strings. Absence
means expect no adaptive configuration. Unsupported metrics cannot satisfy a
requested configuration; any mismatch produces `reason: "config_mismatch"`.
Observed configuration alone is stored under `sender.effective_config`.

Receiver `kind` optionally selects `ceralive`, `irlserver` or `belabox`. Otherwise
those canonical names select the dialect; a custom name uses `SRTLA_REC_KIND`
(default ceralive). An externals-build manifest can supply its resolved `kind`
and binary path directly. The selected dialect determines flags vs BELABOX
positionals. SRT presets are `production`, `strict`, and `legacy-default`.
`SRT_LIVE_TRANSMIT_BIN` selects the caller/listener binary.

`window_secs_override` optionally changes scenario duration, but full validation
still requires at least 45 seconds and every restore/horizon to fit. It cannot
silently clip D. Omit it for normal campaigns.

## Execution

```bash
cargo test --features test-internals --test bench_scheduler
cargo test --features test-internals --test bench_scheduler -- --list
cargo clippy --test bench_scheduler --features test-internals -- -D warnings

BENCH_MANIFEST=/absolute/campaign.json \
  cargo test --features test-internals --test bench_scheduler campaign -- --exact --ignored --nocapture

BENCH_MANIFEST=/absolute/campaign.json \
  cargo test --features test-internals --test bench_scheduler smoke -- --exact --ignored --nocapture
```

`smoke` (or `BENCH_SMOKE=1`) uses **A and D, every manifest candidate, two runs**
for each receiver/preset combination already represented in the matrix. It discards
window overrides: A is 45 seconds and D is 75 seconds, including restore/horizon.
The private `worker` test is invoked only by the runner.

Within each scenario/receiver/preset group, each run index executes every requested
candidate once in a seeded shuffled order. A retry pass follows all arms of that
pair before the next run index, rather than retrying one arm repeatedly first.
Runs are serialized with the A/B runner's extracted `measurement_lock`: its existing
local mutex plus a kernel file lock on a fixed host inode also exclude other test
binaries/worktrees. The test-only lock uses Rust 1.89 standard APIs under the pinned
nightly; the production package MSRV is unchanged.

Each attempt creates a fresh `BondTopology`, receiver, capturing SRT listener,
100ms UDP sink, sender, caller and FIFO-controlled Python CBR generator. No source
traffic is sent until all uplinks register and runtime shaping is initialized.
Warm-up traffic uses `warmup_offered_bps`; settling requires ≥90% for three consecutive
seconds, with a 30-second bound (`settle_timeout`). At least ten seconds of sink
prehistory are retained. The measurement clock starts with the t=0 `OfferedRate`
dispatch; L therefore settles at 12.8Mbit before switching to 20Mbit. Its later
400ms source ramp is one timeline event. I enforces the two-second receiver-only
kill/respawn budget. Link counters are sampled before replug and at 1Hz, serialized
against fault application. CPU/control edges exclude collector shutdown time.

The sink's absolute monotonic origin is calibrated to the run clock. CSV socket
time is aligned by bracketing the first flushed row during warm-up, not by assuming
packet-report cadence; `clocks.json` preserves its uncertainty in milliseconds.
Current listener flags explicitly select **`CaptureSemantics::Interval`**. Do not
add `-fullstats`: it changes belated-counter semantics.

Each attempt runs in a subprocess under GNU `timeout`, bounded by scenario duration
plus 60 seconds, then a ten-second kill-after. Normal teardown is process-scoped;
after worker exit the parent cleans any remaining namespaces identified by that
worker's PID+counter suffix, including partially created topology. Cleanup commands
are separately bounded. Control sockets use short temporary paths, not long artifact
paths. Use an otherwise idle Linux host with passwordless sudo and the required
network tools; missing prerequisites fail rather than silently skip a requested run.

## Checkpoint/result contract

`BENCH_OUT_DIR` selects results (default: the ignored evidence tree's
`bench/<campaign>` directory). `BENCH_ARTIFACT_DIR` selects raw artifacts (default:
`<out>/artifacts`). The normalized explicit matrix is saved as `<out>/manifest.json`.
Each attempt has a unique artifact directory with `request.json`, `result.json`,
`worker.log`, individual process logs, `receiver.csv`, `sink.csv`, and `clocks.json`
when calibration/measurement reached that point.

`sender-startup.log` is written immediately after the sender listener wait, including
on failure, before the partially built stack is dropped. Inspect it first for a
port-5555 timeout: candidate `stats_file: true` injects `--stats-file` independently
of `candidate.args`, and upstream `df0b393` rejects that flag before binding.
Its arms must set `stats_file: false` and omit unsupported `effective_config`
(an empty object is an explicit expectation, not absence). All five fork
test-internals modes expose the complete adaptive configuration even when the
selected scheduler is legacy; declare it verbatim. No candidate is excluded by
its version label, and no startup/settling timeout is extended for this correction.

Every result is the existing **RunRecord v1**, with additive `attempt` (1-based),
optional `reason`, and optional `detail` at top level. Run indices are 0-based;
`started_at` is UTC RFC3339; `order_index` is the invocation's execution ordinal.
Existing `episodes`, `load_intervals`, raw series, CPU edges, diagnostics and
optional sender counters retain their schema. Missing early measurements have
`no_traffic: true`, empty raw series and `status: "failed"`; never grade their
placeholder numerical fields. Host load is captured per attempt and adds exactly
`"loadavg>2"` to warnings when above two.

- Success: `<out>/<cell>/run-<i>.json`, `status: "ok"`.
- Failure: `run-<i>.failed-<attempt>.json`, `status: "failed"`.
- Exhaustion: `run-<i>.exhausted.json`, retaining the last failed document.
- `BENCH_MAX_RETRIES` is the **total attempt budget**, default 2 (not 2 extra retries).
- Only matching-fingerprint `ok` documents count toward each cell's N. Failures
  never skip an index; a later invocation retries it until exhausted. An exhausted
  campaign finishes the other cells and returns nonzero for missing successes.
- A stale record moves into a unique directory under the cell's `stale/`; it is
  neither counted nor overwritten. Stale failures/exhaustion do not spend the new
  configuration's retry budget. Corrupt checkpoints fail closed.
- Publication is sibling tempfile → fsync → atomic rename → directory fsync.

The fingerprint uses Todo 11's exact JSON-tuple hash of candidate, receiver,
scenario hash, SRT preset and requested/verified effective configuration. Candidate
and receiver binaries are content-hashed, not identified by pathname. The scenario
hash includes full base impairments/carriers, events, duration, offered/warm-up
rates, ramp/restart metadata, seed, telemetry selection and SRT tool content hash.
Thus changing binaries, environment, args, scenario policy, preset or configuration
cannot reuse a previous success. Candidate/receiver hashes are rechecked in the worker
before any network setup. Keep all binaries immutable for the duration of a campaign.

No pcap is produced by default. `BENCH_PCAP_ON_FAIL=1` captures each sender link,
then removes captures after a successful measured run; failure or missing required
assertion metrics retains them. Optional unavailable upstream control fields and
the unsupported reorder-distance column are not fabricated into assertion metrics.
