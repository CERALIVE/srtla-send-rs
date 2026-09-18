# 4.0.0 — one selectable scheduler

## Breaking scheduling-mode change

`enhanced` is the only accepted scheduling mode and remains the default. Remove
`--mode classic`, `--mode rtt-threshold`, `--mode edpf`, and `--mode adaptive`
from invocations. Each now exits with a clap argument error (code 2) before
startup. Runtime `set-mode` rejects those values with JSON-RPC code `-32602` and
`error.data.kind: "retired_mode"`; `get-status.mode` reports `enhanced`.
The CeraUI spike found no mode argument or control write in its spawn path.

The final set is the frozen verdict's Enhanced-only **empty-covering-set
fallback**, not a claim of genuine performance coverage. M5 was skipped because
the base is already Enhanced; the hardware canary remains **pending**, not pass.
`apply_verdict_precondition.py` therefore reports the fallback branch with default
Enhanced. No benchmark rule, quality constant, scenario or acceptance threshold
was tuned to produce this outcome. Real Starlink/cellular validation is still open.

Enhanced retains shared health/deadline admission, quality × rejoin × preference
× soft-cap weighting, sole-carrier recovery, duplicate probes, 15ms switch hold
and 10% hysteresis. Its frozen 80-packet trace is unchanged. Removing Adaptive's
separate ranking algorithm does not remove the shared mechanisms Todo32 gave to
Enhanced; their internal names and paths are retained without renaming.

## Retired controls remain accepted

These CLI options have no effect and emit exactly one startup WARN each when
explicitly supplied (including a value equal to the old default):

- `--no-quality`
- `--exploration`
- `--rtt-delta-ms`
- `--stall-deselect`
- `--stall-min-in-flight`
- `--stall-ack-stale-ms`
- `--stall-reprobe-ms`

Omitted options do not warn. Discovery/version probes remain side-effect-free.
The JSON-RPC methods `set-quality`, `set-exploration`, `set-rtt-delta` and text
commands `quality on|off`, `explore on|off`, `rtt-delta N` still validate their
parameter types and succeed with `{"ok":true,"deprecated":true,"effect":"none"}`.
They do not change visible or effective configuration. `--earned-ack-window` is
unchanged: experimental/retired, parsed default OFF; the previously established
arrival-scoped ACK policy remains in force.

`hello.capabilities` and `get-capabilities.methods` retain their exact arrays.
Compared with the released 3.3.0 probe, `version: "4.0.0"` is the explicitly
exempted value change; `adaptive_scheduler: true` and `link_priority: true` are
additive capabilities introduced earlier on this branch. The former describes
the shared adaptive health machinery, NOT acceptance of `--mode adaptive`.
Probe and live capability documents continue to agree.

## Historical regression and campaign evidence

EDPF's algorithm, BLEST/IoDS support and both regression files are retained by
explicit exception: Todo34 skipped fold-in and did not port the E1/E2 pins.
The unit tests exercise `edpf_pipeline_select` directly, outside CLI dispatch.
`tests/netns_edpf.rs` verifies and executes the pre-deletion `m4a-ours-new/edpf`
binary from `receivers.lock.json`; an absent historical artifact is an explicit
prerequisite skip, and a hash mismatch is a failure. This is historical EDPF
evidence, not a selectable 4.0.0 mode or a 4.0.0 EDPF integration claim.

Adaptive-only CLI and namespace suites are retired with that mode. Their prior
G/twin failures remain historical findings, not fixes established by deletion.
Shared health/probe/ACK/weight regressions continue against Enhanced. The two
old adaptive-versus-enhanced equality tests are removed rather than turned into
self-comparisons. Both pre-shared-layer and five-arm pre-deletion trace fixtures
remain frozen. Mode-specific notes have a dated historical header only.

Manifests requesting retired modes require locked historical artifacts:

```sh
bash scripts/bench/run_campaign.sh --historical-from-lock scripts/bench/manifests/m4a-ours-new.json
```

The runner refuses before network setup without opt-in, resolves every candidate
from the campaign/label lock, verifies SHA-256, and never rewrites that lock on a
historical run. Workers repeat artifact verification before launch. Missing locks
fail closed. The owner-protected `c1-baselines.json` is not edited; its retired
mode arguments trigger the same mandatory historical gate even without a marker.
Frozen decision rules and historical matrix contents remain unchanged.

## Packaging and rollback

Package version is plain `4.0.0` (no epoch), binary `/usr/bin/srtla_send`.
`Conflicts` and `Replaces` remain `srtla (<< 2026.6.2)`. Published 3.3.0 artifacts
and the external `versions.yaml` rollback pin are not changed by this task.
Build release packages on Debian Bookworm; a host build is not an ABI-floor proof.

```sh
apt install ./srtla-send-rs_4.0.0_amd64.deb
srtla_send -v
apt install ./srtla-send-rs_3.3.0_amd64.deb --allow-downgrades
srtla_send -v
```

The TypeScript spawn union is narrowed in a separate **BREAKING binding change**;
control status mode remains a string. See `bindings/typescript/RELEASE-NOTES.md`.
No tag, push, npm publication or deployed-device change is part of this work.
