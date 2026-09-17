# M3 four-quadrant interop

BLOCKER: Todo 24 must rerun M1's frozen rule with failing ours-3.3.0 scenarios added before TTL* is applied: C

120 one-attempt performance outcomes; three separate foreign scenario-I conformance runs.
New receiver: NAK-on, gate-on, freeze-on, TTL200. Old receiver: stock NAK, TTL40, freeze-off.
Current/released own senders use enhanced. Settling failures retain full-window measurements, not successful status.
Rule: at least2/3 jointly settled and retransmit≤10%, AND same-sender median goodput ratio≥0.95.

| Sender | Scenario | Settled new | Joint pass new | New median Mbit/s | Old median Mbit/s | Ratio | Disposition |
|---|---|---:|---:|---:|---:|---:|---|
| belabox-c | B1 | 3/3 | 3/3 | 9.615105 | 7.829557 | 1.228052 | pass |
| belabox-c | G | 3/3 | 3/3 | 12.800644 | 12.800410 | 1.000018 | pass |
| belabox-c | C | 3/3 | 0/3 | 16.379462 | 14.146123 | 1.157876 | known_limitation |
| belabox-c | M1 | 3/3 | 3/3 | 9.597091 | 9.006119 | 1.065619 | pass |
| irlserver-rust-classic | B1 | 3/3 | 3/3 | 9.596623 | 7.970164 | 1.204068 | pass |
| irlserver-rust-classic | G | 3/3 | 3/3 | 12.800176 | 12.776547 | 1.001849 | pass |
| irlserver-rust-classic | C | 3/3 | 3/3 | 22.399899 | 12.708875 | 1.762540 | pass |
| irlserver-rust-classic | M1 | 3/3 | 3/3 | 9.579427 | 9.202057 | 1.041009 | pass |
| irlserver-rust-enhanced | B1 | 3/3 | 2/3 | 9.594283 | 6.009148 | 1.596613 | pass |
| irlserver-rust-enhanced | G | 3/3 | 3/3 | 12.800410 | 9.838065 | 1.301111 | pass |
| irlserver-rust-enhanced | C | 3/3 | 1/3 | 21.535375 | 14.761133 | 1.458924 | known_limitation |
| irlserver-rust-enhanced | M1 | 3/3 | 3/3 | 9.573578 | 9.381618 | 1.020461 | pass |
| ours-3.3.0 | B1 | 3/3 | 3/3 | 9.596857 | 7.344567 | 1.306661 | pass |
| ours-3.3.0 | G | 3/3 | 3/3 | 12.803218 | 10.310421 | 1.241774 | pass |
| ours-3.3.0 | C | 3/3 | 0/3 | 21.854724 | 12.478663 | 1.751367 | receiver_pr_blocker |
| ours-3.3.0 | M1 | 3/3 | 3/3 | 9.597793 | 9.234577 | 1.039332 | pass |
| ours-new | B1 | 3/3 | 3/3 | 9.595921 | 4.478377 | 2.142723 | pass |
| ours-new | G | 3/3 | 3/3 | 12.803452 | 4.825099 | 2.653510 | pass |
| ours-new | C | 3/3 | 0/3 | 12.159489 | 11.963317 | 1.016398 | new_sender_failure |
| ours-new | M1 | 3/3 | 0/3 | 8.865395 | 7.699126 | 1.151481 | new_sender_failure |

## All per-run observations

| Sender | Receiver | Scenario | Settled | Retransmit % by index | Goodput Mbit/s by index |
|---|---|---|---:|---|---|
| belabox-c | ours-old | B1 | 0/3 | 41.246395, 43.431625, 41.557868 | 7.837277, 7.609170, 7.829557 |
| belabox-c | ours-old | G | 3/3 | 1.198033, 1.172004, 1.110370 | 12.800410, 12.800410, 12.800410 |
| belabox-c | ours-old | C | 0/3 | 57.384350, 55.590547, 53.542189 | 13.686926, 14.146123, 16.321734 |
| belabox-c | ours-old | M1 | 3/3 | 45.867543, 46.304082, 46.162817 | 9.006119, 8.942834, 9.058291 |
| belabox-c | ours-new-200 | B1 | 3/3 | 0.404858, 0.297692, 0.300139 | 9.615105, 9.615105, 9.616275 |
| belabox-c | ours-new-200 | G | 3/3 | 0.890574, 0.886698, 0.912063 | 12.800176, 12.800644, 12.801112 |
| belabox-c | ours-new-200 | C | 3/3 | 39.957648, 39.613453, 26.378674 | 16.080116, 16.379462, 18.747736 |
| belabox-c | ours-new-200 | M1 | 3/3 | 2.568044, 2.592050, 2.538119 | 9.597325, 9.597091, 9.596506 |
| irlserver-rust-classic | ours-old | B1 | 1/3 | 32.265207, 30.080298, 31.117613 | 7.886174, 8.118492, 7.970164 |
| irlserver-rust-classic | ours-old | G | 3/3 | 5.956515, 6.971576, 6.997869 | 12.783098, 12.775611, 12.776547 |
| irlserver-rust-classic | ours-old | C | 0/3 | 55.772044, 56.473008, 56.395081 | 13.008221, 12.441289, 12.708875 |
| irlserver-rust-classic | ours-old | M1 | 3/3 | 37.894232, 37.310495, 37.805797 | 9.217966, 9.159477, 9.202057 |
| irlserver-rust-classic | ours-new-200 | B1 | 3/3 | 0.309206, 0.326328, 0.209536 | 9.594517, 9.597325, 9.596623 |
| irlserver-rust-classic | ours-new-200 | G | 3/3 | 1.509062, 1.636081, 1.756818 | 12.800176, 12.799241, 12.801814 |
| irlserver-rust-classic | ours-new-200 | C | 3/3 | 1.064170, 1.149416, 1.102867 | 22.400777, 22.396916, 22.399899 |
| irlserver-rust-classic | ours-new-200 | M1 | 3/3 | 5.280443, 7.049479, 7.457779 | 9.584925, 9.579427, 9.563869 |
| irlserver-rust-enhanced | ours-old | B1 | 2/3 | 38.806582, 36.886341, 39.346918 | 6.009148, 6.388390, 5.788996 |
| irlserver-rust-enhanced | ours-old | G | 3/3 | 33.912914, 34.740367, 35.809334 | 10.180576, 9.838065, 9.411564 |
| irlserver-rust-enhanced | ours-old | C | 0/3 | 52.629029, 49.947286, 52.906827 | 15.392813, 14.761133, 13.662010 |
| irlserver-rust-enhanced | ours-old | M1 | 3/3 | 31.847717, 30.321520, 34.264573 | 9.381618, 9.488886, 8.632024 |
| irlserver-rust-enhanced | ours-new-200 | B1 | 3/3 | 4.813170, 28.415190, 1.622848 | 9.596623, 8.450475, 9.594283 |
| irlserver-rust-enhanced | ours-new-200 | G | 3/3 | 1.900357, 4.475153, 3.595064 | 12.803452, 12.799942, 12.800410 |
| irlserver-rust-enhanced | ours-new-200 | C | 3/3 | 12.615540, 1.877045, 33.688785 | 21.535375, 22.406216, 17.908128 |
| irlserver-rust-enhanced | ours-new-200 | M1 | 3/3 | 6.497276, 7.395026, 4.167989 | 9.567145, 9.573578, 9.589721 |
| ours-3.3.0 | ours-old | B1 | 0/3 | 43.826383, 44.314195, 44.630122 | 7.212148, 7.515588, 7.344567 |
| ours-3.3.0 | ours-old | G | 1/3 | 42.517033, 46.318145, 43.275950 | 10.634450, 10.240937, 10.310421 |
| ours-3.3.0 | ours-old | C | 0/3 | 54.560160, 57.917547, 55.655460 | 13.162807, 12.174930, 12.478663 |
| ours-3.3.0 | ours-old | M1 | 3/3 | 39.407219, 40.337843, 37.249552 | 9.272711, 9.234577, 9.224400 |
| ours-3.3.0 | ours-new-200 | B1 | 3/3 | 0.314121, 0.418400, 0.408660 | 9.596857, 9.597559, 9.595687 |
| ours-3.3.0 | ours-new-200 | G | 3/3 | 2.065887, 2.164417, 2.082850 | 12.804388, 12.802048, 12.803218 |
| ours-3.3.0 | ours-new-200 | C | 3/3 | 13.568125, 14.301265, 51.453458 | 21.889291, 21.854724, 14.530044 |
| ours-3.3.0 | ours-new-200 | M1 | 3/3 | 5.084944, 4.437479, 4.836751 | 9.597793, 9.595687, 9.601887 |
| ours-new | ours-old | B1 | 0/3 | 19.826243, 20.127264, 20.353982 | 4.507856, 4.338238, 4.478377 |
| ours-new | ours-old | G | 0/3 | 28.133151, 28.608017, 24.750386 | 4.418953, 4.861362, 4.825099 |
| ours-new | ours-old | C | 0/3 | 26.729530, 29.017190, 28.353629 | 11.035099, 11.963317, 12.078950 |
| ours-new | ours-old | M1 | 3/3 | 16.917397, 22.919650, 22.508416 | 7.590337, 7.699126, 7.807097 |
| ours-new | ours-new-200 | B1 | 3/3 | 0.348042, 0.394180, 7.548151 | 9.596155, 9.595921, 8.545695 |
| ours-new | ours-new-200 | G | 3/3 | 0.662767, 0.621444, 0.576285 | 12.804855, 12.802984, 12.803452 |
| ours-new | ours-new-200 | C | 3/3 | 33.907990, 24.750434, 28.501287 | 10.197245, 12.991903, 12.159489 |
| ours-new | ours-new-200 | M1 | 3/3 | 11.900728, 10.534844, 13.075927 | 8.865395, 9.210245, 8.843286 |

## Foreign-sender limitations

the receiver cannot detect sender lineage; the only lever is TTL*/gate, which M1 chose
Foreign-only failures do not revert receiver policy. These are namespace/netem observations, not hardware certification.

## Warnings

- loadavg>2
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c/0: loadavg=3.28
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c/1: loadavg=4.91
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c/2: loadavg=3.61
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=3.99
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust-classic/1: loadavg=4.83
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust-classic/2: loadavg=3.49
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=3.72
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust-enhanced/1: loadavg=4.8
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust-enhanced/2: loadavg=3.16
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--ours-3.3.0/0: loadavg=4.87
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--ours-3.3.0/1: loadavg=4.07
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--ours-3.3.0/2: loadavg=3.25
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--ours-new/0: loadavg=4.87
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--ours-new/1: loadavg=4.9
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--ours-new/2: loadavg=3.82
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c/0: loadavg=5.21
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c/1: loadavg=4.15
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c/2: loadavg=5.05
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=5.01
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust-classic/1: loadavg=5.71
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust-classic/2: loadavg=5.61
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=4.42
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust-enhanced/1: loadavg=3.39
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust-enhanced/2: loadavg=4.68
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--ours-3.3.0/0: loadavg=4.78
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--ours-3.3.0/1: loadavg=4.17
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--ours-3.3.0/2: loadavg=5.06
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--ours-new/0: loadavg=5.39
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--ours-new/1: loadavg=3.79
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--ours-new/2: loadavg=5.85
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c/0: loadavg=3.74
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c/1: loadavg=4.4
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c/2: loadavg=4.53
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=5.43
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust-classic/1: loadavg=4.44
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust-classic/2: loadavg=5.29
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=6.55
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust-enhanced/1: loadavg=4.11
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust-enhanced/2: loadavg=5.82
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--ours-3.3.0/0: loadavg=4.8
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--ours-3.3.0/1: loadavg=3.96
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--ours-3.3.0/2: loadavg=5.99
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--ours-new/0: loadavg=4.43
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--ours-new/1: loadavg=4.47
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--ours-new/2: loadavg=3.67
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--I@interop-conformance--production--slt:4001--fec:off--belabox-c/0: loadavg=5.02
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--I@interop-conformance--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=4.5
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--I@interop-conformance--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=4.5
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--belabox-c/0: loadavg=4.88
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--belabox-c/1: loadavg=4.94
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--belabox-c/2: loadavg=5.87
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=4.44
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--irlserver-rust-classic/1: loadavg=4.49
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--irlserver-rust-classic/2: loadavg=12.03
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=4.06
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--irlserver-rust-enhanced/1: loadavg=5.01
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--irlserver-rust-enhanced/2: loadavg=6.38
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--ours-3.3.0/0: loadavg=4.41
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--ours-3.3.0/1: loadavg=5.29
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--ours-3.3.0/2: loadavg=8.84
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--ours-new/0: loadavg=4.81
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--ours-new/1: loadavg=5.76
- ours-new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--M1@--production--slt:4001--fec:off--ours-new/2: loadavg=9.45
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--belabox-c/0: loadavg=6.09
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--belabox-c/1: loadavg=5.48
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--belabox-c/2: loadavg=4.76
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=4.98
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--irlserver-rust-classic/1: loadavg=5.7
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--irlserver-rust-classic/2: loadavg=3.79
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=5.8
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--irlserver-rust-enhanced/1: loadavg=6.37
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--irlserver-rust-enhanced/2: loadavg=5.25
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--ours-3.3.0/0: loadavg=5.59
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--ours-3.3.0/1: loadavg=5.05
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--ours-3.3.0/2: loadavg=4.07
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--ours-new/0: loadavg=5.37
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--ours-new/1: loadavg=6.3
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--ours-new/2: loadavg=6.48
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--belabox-c/0: loadavg=3.78
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--belabox-c/1: loadavg=5.29
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--belabox-c/2: loadavg=4.48
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=4.73
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--irlserver-rust-classic/1: loadavg=5.06
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--irlserver-rust-classic/2: loadavg=4.98
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=4.37
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--irlserver-rust-enhanced/1: loadavg=4.58
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--irlserver-rust-enhanced/2: loadavg=3.72
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--ours-3.3.0/0: loadavg=4.2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--ours-3.3.0/1: loadavg=3.74
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--ours-3.3.0/2: loadavg=4.6
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--ours-new/0: loadavg=7.15
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--ours-new/1: loadavg=5.1
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--ours-new/2: loadavg=6.25
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--belabox-c/0: loadavg=6.58
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--belabox-c/1: loadavg=4.57
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--belabox-c/2: loadavg=6.12
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=6.82
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--irlserver-rust-classic/1: loadavg=5.69
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--irlserver-rust-classic/2: loadavg=4.74
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=5.82
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--irlserver-rust-enhanced/1: loadavg=6.46
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--irlserver-rust-enhanced/2: loadavg=5.23
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--ours-3.3.0/0: loadavg=5.03
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--ours-3.3.0/1: loadavg=7.41
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--ours-3.3.0/2: loadavg=6.12
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--ours-new/0: loadavg=8.64
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--ours-new/1: loadavg=5.53
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--ours-new/2: loadavg=4.63
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--belabox-c/0: loadavg=4.92
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--belabox-c/1: loadavg=4.06
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--belabox-c/2: loadavg=5.04
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--irlserver-rust-classic/0: loadavg=5.11
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--irlserver-rust-classic/1: loadavg=7.03
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--irlserver-rust-classic/2: loadavg=4.66
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--irlserver-rust-enhanced/0: loadavg=4.67
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--irlserver-rust-enhanced/1: loadavg=10.33
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--irlserver-rust-enhanced/2: loadavg=4.51
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--ours-3.3.0/0: loadavg=4.55
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--ours-3.3.0/1: loadavg=8.43
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--ours-3.3.0/2: loadavg=4.5
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--ours-new/0: loadavg=6.21
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--ours-new/1: loadavg=6.99
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--M1@--production--slt:4001--fec:off--ours-new/2: loadavg=4.9
