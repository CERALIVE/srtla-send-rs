# M2 sender-mechanism campaign

47 one-attempt outcomes; settle timeouts retain their complete measured windows.
Bootstrap median CI: 10,000 resamples, seed 20260913.
The raw runner exited 101 because 24 ours-old outcomes did not settle; this reduction follows the predeclared rule and retains those full-window measurements rather than relabeling them as successes.

## In-flight premature-NAK rule

| Scenario | ON settled | OFF settled | Delta | ON goodput Mbit/s median [95% CI] | OFF goodput Mbit/s median [95% CI] | Regression |
|---|---:|---:|---:|---|---|---|
| B1 | 0.000 | 0.000 | +0.000 | 4.480 [4.436, 4.488] | 4.424 [4.381, 4.469] | False |
| C | 0.000 | 0.000 | +0.000 | 11.967 [11.279, 12.381] | 11.561 [10.686, 11.607] | False |
| A | 0.000 | 0.000 | +0.000 | 9.914 [9.868, 9.986] | 10.000 [9.896, 10.050] | False |
| G | 0.000 | 0.000 | +0.000 | 4.341 [4.247, 4.376] | 4.401 [4.358, 4.707] | False |

K-cap decision: **K=3**; the rule remains enabled.

## NAK-off blindness

| Scenario | Enhanced Mbit/s median [95% CI] | Adaptive Mbit/s median [95% CI] | Rule passes |
|---|---|---|---|
| G | 12.803 [12.802, 12.805] | 12.803 [12.803, 12.803] | False |
| F | 12.803 [12.802, 12.805] | 12.803 [12.802, 12.803] | True |
| A | 12.664 [11.551, 15.355] | 14.507 [12.900, 20.024] | True |

Outcome: **present**.

## Retransmission-bit visibility

DATA=69278; originals=68639; retransmissions=639; R=1 originals=0; R=0 retransmissions=0.
Outcome: **counters trusted**. Capture SHA-256: `80abefedb4f3e8a90969f40b79a9eed23fe490da97d6ad3b9a471812ff26b166`.

## HSRSP lineage decode

| Receiver | Expected | Observed | Status line |
|---|---|---|---|
| ours-old | on | on | `receiver: nak_report=on srt=1.5.6` |
| ours-new | on | on | `receiver: nak_report=on srt=1.5.7` |
| irlserver-prod | off | null | `` |
| irlserver-next | on | on | `receiver: nak_report=on srt=1.5.6` |

The absent irlserver-prod observation is recorded as null (None), selecting fail-safe NAK-on policy without adding a parser heuristic.

## Warnings

- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 2: failed (settle_timeout)
