# S-TWINPORT results

**Decision: DISTINCT** — run the reserved 4003 block in M4.

See [predeclared method](method.md). These are SLS/attached-player measurements, not bare SLT sink measurements.

## Paired post-settle inference

Profile | Valid 4002 indices | Valid 4003 indices | Δfraction ×100 | 95% CI | Δpublisher loss pp
---|---|---|---|---|---
default | (0, 1) | (0,) | None | [None, None] | None
legacy-l2 | () | () | None | [None, None] | None

None/null means unidentifiable, NOT zero. A true-alias conclusion is not authorized.

## Every attempt (including discarded player legs)

Profile | Scenario | Port | Index | Attempt | Settled | Player valid | Player Mbit/s | Loss/drop | Delivered fraction | registered/carry/latency | Disposition
---|---|---|---|---|---|---|---|---|---|---|---
default | M1 | 4002 | 0 | 1 | True | False | 9.265008768 | 0/0 | 0.579369859 | ok/ok/ok | player_leg_invalid
default | M1 | 4002 | 0 | 2 | True | True | 9.431857594 | 0/0 | 0.621001631 | ok/ok/ok | ok
default | M1 | 4002 | 1 | 1 | True | True | 9.450510573 | 0/0 | 0.629150900 | ok/ok/ok | ok
default | M1 | 4002 | 2 | 1 | True | False | 9.267145061 | 0/0 | 0.597842100 | ok/ok/ok | player_leg_invalid
default | M1 | 4002 | 2 | 2 | True | False | 9.402327761 | 0/0 | 0.625485561 | ok/ok/ok | player_leg_invalid
default | M1 | 4003 | 0 | 1 | True | True | 9.464590104 | 0/0 | 0.626963580 | ok/ok/ok | ok
default | M1 | 4003 | 1 | 1 | True | False | 9.200840589 | 0/0 | 0.527538655 | ok/ok/ok | player_leg_invalid
default | M1 | 4003 | 1 | 2 | True | False | 9.240457989 | 0/0 | 0.586109984 | ok/ok/ok | player_leg_invalid
default | M1 | 4003 | 2 | 1 | True | False | 9.317974615 | 0/0 | 0.588077141 | ok/ok/ok | player_leg_invalid
default | M1 | 4003 | 2 | 2 | True | False | 9.372205274 | 0/0 | 0.608959995 | ok/ok/ok | player_leg_invalid
default | SLS | 4002 | 0 | 1 | True | True | 1.033852550 | 0/0 | 0.950828242 | ok/ok/ok | ok
default | SLS | 4003 | 0 | 1 | True | True | 1.033810080 | 0/0 | 0.905715878 | ok/ok/ok | ok
legacy-l2 | M1 | 4002 | 0 | 1 | True | False | 8.536813503 | 0/0 | 0.291534293 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4002 | 0 | 2 | True | False | 8.946335013 | 0/0 | 0.300510831 | ok/ok/ok | player_leg_invalid
legacy-l2 | M1 | 4002 | 1 | 1 | True | False | 8.498806847 | 0/0 | 0.250085256 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4002 | 1 | 2 | True | False | 8.616024086 | 0/0 | 0.284399496 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4002 | 2 | 1 | True | False | 8.621558736 | 0/0 | 0.263883482 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4002 | 2 | 2 | True | False | 8.156918003 | 0/0 | 0.286652627 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4003 | 0 | 1 | True | False | 8.726179223 | 0/0 | 0.328885537 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4003 | 0 | 2 | True | False | 8.618169358 | 0/0 | 0.292162016 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4003 | 1 | 1 | True | False | 8.566617028 | 0/0 | 0.339495056 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4003 | 1 | 2 | True | False | 8.607012879 | 0/0 | 0.300975524 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4003 | 2 | 1 | True | False | 8.717627548 | 0/0 | 0.264948851 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | M1 | 4003 | 2 | 2 | True | False | 8.477531026 | 0/0 | 0.261379660 | ok/FAIL/ok | player_leg_invalid
legacy-l2 | SLS | 4002 | 0 | 1 | True | True | 1.033633093 | 0/0 | 0.907717403 | ok/ok/ok | ok
legacy-l2 | SLS | 4003 | 0 | 1 | True | True | 1.033656558 | 0/0 | 0.905479938 | ok/ok/ok | ok

## Diagnostic-only final-attempt comparisons

These retain the actual numbers from discarded runs for audit, not for the decision or a confidence claim about valid N=3 runs.
- default: Δfraction×100=-1.652556582; diagnostic bootstrap [-4.304091553, 0.596194855].
- legacy-l2: Δfraction×100=0.550938855; diagnostic bootstrap [-2.527296720, 1.657602868].

## Publisher field availability and override proof

All live raw API responses retain publisher keys and both edge counters. No publisher packet-received denominator exists; no normalized publisher-loss rate can be computed. The player's receive denominator is NOT substituted.

Both listener startup policy lines are checked on every attempt: default TTL200/freeze1/NAK1/gate1/FEC1; rollback TTL40/freeze1/NAK0/gate0/FEC0. Actual negotiated publisher latency is2000ms for M1,500ms for synthetic SLS. Actual player receive latency is200ms, verified from its CSV.

## Caveats

- Publisher /stats exposes pktRcvDrop but no received-packet denominator: d_loss_pp is null, never NAK-derived.
- Invalid player legs and unsettled outcomes do not count. Diagnostic final-attempt comparisons are NOT equivalence evidence.
- Four synthetic conformance configurations are separate from twelve planned M1 indices.
