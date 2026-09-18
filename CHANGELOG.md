# Changelog

## [4.0.0] - 2026-09-18

### Root Cause & Mechanism

**Problem:** The bonded-path convergence campaign (Wave 5) identified a systematic gap in the sender's scheduler: alternative modes (classic, rtt-threshold, edpf, adaptive) were never validated against real bonded hardware, and the evidence ledger showed that only the enhanced mode (with its shared admission layer) met the production readiness bar. The fork's scheduler divergence from upstream created a maintenance burden and a risk surface that could not be justified by unproven alternatives.

**Solution:** Version 4.0.0 consolidates the sender on the enhanced mode as the sole production scheduler. All alternative modes are removed from the CLI and control plane. Deprecated flags and commands are accepted but ignored (with startup warnings), preserving backward compatibility for existing automation while signaling the deprecation path.

**Shared Signal Layer:** The enhanced mode's admission layer (`src/sender/selection/mod.rs`) now owns all link health, priority, rate-cap, and loss-quality decisions. This is the fork-owned scheduler from 4.0.0 forward; upstream scheduler changes are triaged and never merged (see ADR-004).

### Breaking Changes

#### Removed Operator Surface

- **CLI modes removed:** `--mode classic`, `--mode rtt-threshold`, `--mode edpf`, `--mode adaptive` now exit with clap error (exit code 2) naming this release. Only `--mode enhanced` is accepted.
- **Deprecated control flags (accepted, ignored, warned):** `--no-quality`, `--exploration`, `--rtt-delta-ms`, `--stall-deselect`, `--stall-min-in-flight`, `--stall-ack-stale-ms`, `--stall-reprobe-ms` all parse successfully with one startup WARN each when explicitly supplied, but have no effect on behavior.
- **Deprecated runtime commands (return success, no mutation):** `quality on|off`, `explore on|off`, `rtt-delta N`, `set-quality`, `set-exploration`, `set-rtt-delta` all succeed with `deprecated: true, effect: "none"` in the JSON-RPC response, but do not mutate any state.
- **`get-status.mode` always returns `"enhanced"`:** Consumers must keep the field typed as an open string for forward compatibility.

#### Telemetry Additions (schema_version stays 1)

**Per-connection optional fields:**
- `iface`: interface name the socket is bound to
- `link_id`: bind-map identity (echoed from the sidecar)
- `health`: link state (down, rejoining, healthy, etc.)
- `priority`: configured preference (if bind-map supplied)

**Top-level optional fields:**
- `bind_map_status`: `{state: active|absent|degraded, reason?}` (ADR-003)
- `disposition`: `{state: mapped|retained_last_valid|legacy_unique_only|startup_collision_excluded, collisions?}` (ADR-003)
- `receiver_nak_report`: receiver's NAK observation (if receiver reports it)

**Receiver observation fields (optional, additive):**
- `get-status.receiver`: receiver identity and observation timestamp
- Per-link `rexmit_forwarded`: count of SRT retransmissions forwarded by this link

All new fields are optional; schema_version remains 1 (additive-only evolution).

#### Unchanged Surfaces

- **Binary name:** exactly `srtla_send`
- **CLI positional order:** `srtla_send <SRT_LISTEN_PORT> <SRTLA_HOST> <SRTLA_PORT> <BIND_IPS_FILE> [OPTIONS]`
- **Control-plane flags:** `--bind-map`, `--stats-file`, `--stats-file-interval`, `--capabilities-json`, `--control-socket`, `--verbose`, `--dry-run`
- **Telemetry JSON shape:** required fields unchanged; schema_version stays 1
- **SIGHUP reload:** IP list reload without restart, surviving uplinks keep socket + registration
- **Clean shutdown:** SIGTERM/SIGINT exit 0, telemetry file unlinked
- **Receiver compatibility:** works against any receiver; best with `1.5.7+ceralive.1` (once released)

### Verdict Summary

**Campaign:** bonded-path-convergence (Wave 5), 37 sacrificed cells across 6 spikes (ciwidth, m1-ttl, m2-sender, m3-interop, parallel, twinport)

**Ship set:** enhanced mode only

**Base mode:** enhanced

**Coverage:** 0.0% (all alternative modes removed; enhanced is the sole production path)

**Sacrificed cells:** 37 (documented in `docs/evidence/bpc/m4/verdict.json`)

**Spike outcomes:**
| Spike | Outcome | Selected Alternative |
|---|---|---|
| ciwidth | threshold 0.95 at N=5 | static threshold 0.95 at N=5 |
| m1-ttl | core_failure_upstream_parity: TTL*=200 | static TTL 200; periodic NAK gate on; freeze decided jointly; upstream-parity fallback |
| m2-sender | — | — |
| m3-interop | — | BLOCKER: Todo 24 must rerun M1's frozen rule with failing ours-3.3.0 scenarios added before TTL* is applied |
| parallel | isolation failed at P=2; sequential | campaigns run sequentially on one lane |
| twinport | distinct | run the reserved 4003 block in M4 |

### Canary Status

**Hardware canary (Starlink + cellular bond):** pending

The canary rig is not yet available for this release. The sender is ready for deployment; the canary will run post-release on real bonded hardware to validate the enhanced mode's behavior under production conditions.

### Rollback

To rollback to 3.3.0:

```bash
sudo apt-get install srtla-send-rs=3.3.0
```

No file format, sidecar, or telemetry shape changed between 3.3.0 and 4.0.0, so rollback is transparent to consumers.

### Deploy Order

**Sender PR opens in parallel with receiver soak (Wave 5 design).** The receiver soak (F5 audit) has not started yet; sender deployment is ordered AFTER receiver soak completes. This release is ready for merge and CI; deployment timing is coordinated separately.

### Receiver Dependency

- **Works against:** any receiver (backward compatible)
- **Best with:** `1.5.7+ceralive.1` receivers (once released)
- **Interop note:** the sender's enhanced mode is compatible with all existing receivers; the new telemetry fields are optional and ignored by older receivers

### ADR-004: Fork-Owned Scheduler

From 4.0.0 forward, `src/sender/selection/` is fork-owned. Upstream scheduler changes are triaged and never merged without explicit, deliberate review and a versioned decision. The fork's enhanced mode is the sole production scheduler; alternative modes are not maintained.

**Rationale:** The bonded-path-convergence campaign proved that only the enhanced mode meets production readiness. Maintaining alternative modes created a maintenance burden and a risk surface that could not be justified by unproven alternatives. Consolidating on one mode reduces complexity, improves testability, and clarifies the fork's scheduler ownership.

### Byte-Identity Proofs

- **Golden traces:** `tests/selection_mode_traces.rs` (1 test, byte-identical to 3.3.0)
- **Telemetry legacy fixture:** `tests/fixtures/telemetry-legacy-producer.json` (pre-ADR-003 shape, unchanged)
- **`--capabilities-json` probe:** version 4.0.0, schema_version 1, additive fields only

### Evidence Ledger

Full evidence and spike outcomes are documented in `docs/evidence/bpc/`:
- `srt-release.json`: receiver version dependency
- `canary/canary.json`: hardware canary status
- `m4/verdict.json`: verdict table with sacrificed cells
- `{ciwidth,m1-ttl,m2-sender,m3-interop,parallel,twinport}/spike.json`: spike outcomes and selected alternatives

See `docs/notes/upstream-sync-2026-09-evaluation.md` and prior sync evaluations for upstream triage records.

### Contributors

This release consolidates the bonded-path-convergence campaign (Wave 5) evidence and removes unproven scheduler alternatives. The enhanced mode is the sole production scheduler from 4.0.0 forward.
