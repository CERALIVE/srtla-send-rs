2026-09-18 — classic/rtt-threshold/edpf/adaptive modes removed in 4.0.0 — read as history.
# Scheduler benchmark report

Bootstrap: 10000 resamples; seed 20260913; 95% median CI.
Censored failure durations are +inf; null means unavailable, not zero.

## A — smoke / ceralive / production

| Candidate | Metric | n | Missing | Mean | Median | SD | 95% CI | Nonfinite rate |
|---|---|---:|---:|---:|---:|---:|---|---:|
| classic | useful_goodput_bps | 2 | 0 | 21234391.111111112 | 21234391.111111112 | 316470.5739540246 | [21010612.622222222, 21458169.6] | 0.0 |
| classic | viewer_loss_ratio | 2 | 0 | 0.11476579123710437 | 0.11476579123710437 | 0.012884024635051233 | [0.10565541004868512, 0.12387617242552364] | 0.0 |
| classic | per_link_share_gini | 2 | 0 | 2.9471340417549136e-05 | 2.9471340417549136e-05 | 2.127476049194905e-05 | [1.4427813005572313e-05, 4.451486782952596e-05] | 0.0 |
| classic | cpu_ms_per_mb | 2 | 0 | 35.46137093015924 | 35.46137093015924 | 0.7061053222263909 | [34.96207906858105, 35.960662791737434] | 0.0 |
| classic | switch_count | 2 | 0 | 4737.0 | 4737.0 | 21.213203435596427 | [4722.0, 4752.0] | 0.0 |
| classic | diagnostics.loss_ratio | 2 | 0 | 0.45074943105385334 | 0.45074943105385334 | 0.0008700465261316855 | [0.45013421525527786, 0.4513646468524289] | 0.0 |
| classic | diagnostics.mbps_recv_rate_mean | 2 | 0 | 29.621572089371977 | 29.621572089371977 | 0.0014705942262574882 | [29.620532222222216, 29.622611956521737] | 0.0 |
| classic | diagnostics.ms_rtt_median | 2 | 0 | 433.5085 | 433.5085 | 10.271433103515797 | [426.2455, 440.7715] | 0.0 |
| classic | diagnostics.pkt_belated_sum | 2 | 0 | 13590.0 | 13590.0 | 1385.9292911256332 | [12610.0, 14570.0] | 0.0 |
| classic | diagnostics.reorder_distance_max | 0 | 2 | None | None | None | [None, None] | None |
| classic | diagnostics.retrans_ratio | 2 | 0 | 0.5767915348310242 | 0.5767915348310242 | 0.0048923258800590945 | [0.57333213802546, 0.5802509316365885] | 0.0 |
| classic | load[0].reached_ms | 2 | 0 | 1500.0 | 1500.0 | 707.1067811865476 | [1000.0, 2000.0] | 0.0 |
| enhanced | useful_goodput_bps | 1 | 0 | 21251820.8 | 21251820.8 | None | [21251820.8, 21251820.8] | 0.0 |
| enhanced | viewer_loss_ratio | 1 | 0 | 0.11487147075379464 | 0.11487147075379464 | None | [0.11487147075379464, 0.11487147075379464] | 0.0 |
| enhanced | per_link_share_gini | 1 | 0 | 0.0014854325497010157 | 0.0014854325497010157 | None | [0.0014854325497010157, 0.0014854325497010157] | 0.0 |
| enhanced | cpu_ms_per_mb | 1 | 0 | 85.82793997585374 | 85.82793997585374 | None | [85.82793997585374, 85.82793997585374] | 0.0 |
| enhanced | switch_count | 1 | 0 | 864.0 | 864.0 | None | [864.0, 864.0] | 0.0 |
| enhanced | diagnostics.loss_ratio | 1 | 0 | 0.4517968654755294 | 0.4517968654755294 | None | [0.4517968654755294, 0.4517968654755294] | 0.0 |
| enhanced | diagnostics.mbps_recv_rate_mean | 1 | 0 | 29.631277777777782 | 29.631277777777782 | None | [29.631277777777782, 29.631277777777782] | 0.0 |
| enhanced | diagnostics.ms_rtt_median | 1 | 0 | 428.811 | 428.811 | None | [428.811, 428.811] | 0.0 |
| enhanced | diagnostics.pkt_belated_sum | 1 | 0 | 9619.0 | 9619.0 | None | [9619.0, 9619.0] | 0.0 |
| enhanced | diagnostics.reorder_distance_max | 0 | 1 | None | None | None | [None, None] | None |
| enhanced | diagnostics.retrans_ratio | 1 | 0 | 0.5873074705543201 | 0.5873074705543201 | None | [0.5873074705543201, 0.5873074705543201] | 0.0 |
| enhanced | load[0].reached_ms | 1 | 0 | 1000.0 | 1000.0 | None | [1000.0, 1000.0] | 0.0 |

| Candidate | Interval/check | No-collapse rate | Recovered rate | Failure rate |
|---|---|---:|---:|---:|
| classic | load[0] graded=True | 1.0 | 1.0 | 0.0 |
| enhanced | load[0] graded=True | 1.0 | 1.0 | 0.0 |

| Candidate | Reference | Paired n | Ratio 95% CI | Loss delta pp | Recovery ratio | Dropped |
|---|---|---:|---|---:|---:|---|
| classic | classic | 2 | [1.0, 1.0] | 0.0 | None | () |
| classic | enhanced | 1 | [1.009709699791935, 1.009709699791935] | -0.010567951669027131 | None | (1,) |
| enhanced | classic | 1 | [0.9903836718673339, 0.9903836718673339] | 0.010567951669027131 | None | (1,) |
| enhanced | enhanced | 1 | [1.0, 1.0] | 0.0 | None | () |

## Warnings

- Smoke coverage calibration: at least one of two successes per required cell; original indices retained; not a performance or C1/C2 acceptance verdict
- enhanced--A--ceralive--production run 1: failed (settle_timeout)
- smoke/A/ceralive/production/classic run 0: loadavg>2
- smoke/A/ceralive/production/classic run 0: loadavg>2 (5.23)
- smoke/A/ceralive/production/classic run 1: loadavg>2
- smoke/A/ceralive/production/classic run 1: loadavg>2 (5.0)
- smoke/A/ceralive/production/classic vs enhanced: dropped pairs (1,)
- smoke/A/ceralive/production/enhanced run 0: loadavg>2
- smoke/A/ceralive/production/enhanced run 0: loadavg>2 (4.94)
- smoke/A/ceralive/production/enhanced vs classic: dropped pairs (1,)

## Final owner-approved smoke scope

Acceptance requires at least one successful run out of the two planned indices in classic/A and enhanced/A. Counts and indices are actual successes, not planned or renumbered observations. Failed attempts remain in the unchanged original result tree and report warnings. Classic/D, enhanced/D, adaptive/A and adaptive/D were executed and are informational in known-findings.json. The original full-matrix campaign result remains a failure; this is a retrospective, explicitly scoped path-coverage acceptance, not a statistical majority, a C1/C2 waiver, or a performance pass. See docs/notes/scheduler-evaluation-2026-09.md for the measured variance and owner decision.

## Portable evidence receipt

The generated report above is preserved without changing its statistical tables.
The [receipt](smoke-final-receipt-2026-09.json) records all six cell outcomes, successful
run identities, source/receiver/manifest hashes, and generated-artifact digests.
`known-findings.json` and the full raw host/process artifacts named above remain in
the local retained campaign archive, not in the distributable source tree. The
[investigation note](scheduler-evaluation-2026-09.md#known-limitation-adaptive-mode-baseline-topology-throughput-instability-scenario-a-discovered-post-todo-28)
provides their scope and interpretation. N=1/2 intervals are not performance evidence.
