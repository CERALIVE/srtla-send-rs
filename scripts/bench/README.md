# Scheduler bench tooling

Campaign support for the adaptive-scheduler evaluation. The runner itself is
`tests/bench_scheduler.rs`; everything here prepares inputs for it or reduces its
outputs.

| File | Purpose |
| --- | --- |
| `build_externals.sh` | Builds the external receiver/tooling binaries a campaign dials. |
| `preflight.sh` | Checks host prerequisites before a privileged campaign. |
| `probe_srt_stats.sh` | Inspects a listener's CSV statistics capture. |
| `report.py` | Fail-closed reduction of a results tree into `report.md` + `summary.json`. |
| `decide.py` | Statistics and retention decisions over a reduced campaign. |

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
