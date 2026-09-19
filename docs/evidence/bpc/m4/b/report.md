# Todo 34 measured lineage results

Portable campaign-log copies omit the final empty line for repository whitespace checks;
the hash-indexed originals remain unchanged in the raw root.

All rows are enhanced. PASS requires the joint quota AND applicable metrics-v2 gates.

| Lineage | Scenario | Sink/port | Settled | Joint settle/retransmit | Final | Failure gates |
|---|---|---|---:|---:|---|---|
| belabox | M1 | slt/4001 | 3/3 | 0/3 | FAIL | joint_settle_retransmit_quota, post_settle_zero_drop_belated |
| belabox | M4 | slt/4001 | 3/3 | 3/3 | FAIL | post_settle_zero_drop_belated |
| belabox | M6 | slt/4001 | 3/3 | 3/3 | FAIL | post_settle_zero_drop_belated |
| irlserver-next | M1 | slt/4001 | 3/3 | 0/3 | FAIL | joint_settle_retransmit_quota, post_settle_zero_drop_belated |
| irlserver-next | M4 | slt/4001 | 3/3 | 2/3 | FAIL | post_settle_zero_drop_belated |
| irlserver-next | M6 | slt/4001 | 3/3 | 3/3 | FAIL | post_settle_zero_drop_belated |
| irlserver-prod | A | slt/4001 | 4/5 | 1/5 | FAIL | joint_settle_retransmit_quota, post_settle_zero_drop_belated |
| irlserver-prod | B1 | slt/4001 | 5/5 | 5/5 | FAIL | post_settle_zero_drop_belated |
| irlserver-prod | G | slt/4001 | 5/5 | 5/5 | FAIL | post_settle_zero_drop_belated |
| irlserver-prod | M1 | slt/4001 | 5/5 | 4/5 | FAIL | post_settle_zero_drop_belated |
| irlserver-prod | M4 | slt/4001 | 5/5 | 5/5 | FAIL | post_settle_zero_drop_belated |
| irlserver-prod | M6 | slt/4001 | 5/5 | 5/5 | FAIL | post_settle_zero_drop_belated |
| ours-new | M1 | sls/4003 | 3/3 | 0/3 | FAIL | retransmit_unknown, joint_settle_retransmit_quota |
| ours-new | M4 | sls/4003 | 3/3 | 0/3 | FAIL | retransmit_unknown, joint_settle_retransmit_quota |
| ours-new | M6 | sls/4003 | 3/3 | 0/3 | FAIL | retransmit_unknown, joint_settle_retransmit_quota |
| ours-old | M1 | slt/4001 | 3/3 | 0/3 | FAIL | joint_settle_retransmit_quota, post_settle_zero_drop_belated |
| ours-old | M4 | slt/4001 | 3/3 | 0/3 | FAIL | joint_settle_retransmit_quota, post_settle_zero_drop_belated |
| ours-old | M6 | slt/4001 | 3/3 | 3/3 | FAIL | post_settle_zero_drop_belated |

Active service duration: 5417s; first start through final end: 5579s.
Measured 66/66 original indices; 65 settled.

SLS retransmission fractions are unknown, not zero. All nine SLS conformance results passed.
The original 48 measured indices were preserved byte-for-byte during the checkpoint continuation.
Both controller exits remain 1 (checker errors retained); final offline reduction succeeds and is byte-deterministic.
See ../lineage-method.md and ../README.md for the fixed rule, corrections and non-green full-gate boundaries.
