# M1 receiver TTL campaign

65 cells; 195 planned one-attempt outcomes. Failed settling remains failed.

Retransmission fraction = received retransmissions / received packets. Bootstrap median CI: 10,000 resamples, seed 20260913.

Old lineage has stock periodic NAK and lacks the freeze URI; its default freeze-off behavior is explicitly recorded. No emulation or patch of that baseline.

Freeze timing scope: Entire receiver capture, including warmup; gap detection to first NAK / first repair; unrepaired gaps retained as right-censored lower bounds.

| Receiver | Sender | Scenario | Offered override | Settled | Joint passing | Goodput Mbit/s (95% CI) | Retransmissions % by run |
|---|---|---|---|---|---|---|---|
| ours-old | classic | A | catalog | 0/3 | 0/3 | 9.743/9.691/9.815 | 39.192, 40.782, 29.668 |
| ours-old | enhanced | A | catalog | 0/3 | 0/3 | 9.696/9.638/9.699 | 40.680, 38.481, 36.791 |
| ours-old | classic | B1 | catalog | 0/3 | 0/3 | 4.367/4.362/4.469 | 23.615, 25.230, 23.261 |
| ours-old | enhanced | B1 | catalog | 0/3 | 0/3 | 4.365/4.327/4.412 | 25.011, 25.767, 25.415 |
| ours-old | classic | C | catalog | 0/3 | 0/3 | 10.606/10.485/11.602 | 40.908, 31.032, 32.666 |
| ours-old | enhanced | C | catalog | 0/3 | 0/3 | 11.457/10.957/12.007 | 37.629, 32.569, 29.941 |
| ours-old | classic | G | catalog | 3/3 | 0/3 | 9.008/8.921/9.281 | 15.985, 17.333, 14.680 |
| ours-old | enhanced | G | catalog | 0/3 | 0/3 | 4.574/4.344/4.840 | 28.089, 32.579, 32.774 |
| new-40 | classic | A | catalog | 0/3 | 0/3 | 9.777/9.741/9.832 | 30.878, 31.870, 32.929 |
| new-40 | enhanced | A | catalog | 0/3 | 0/3 | 9.730/9.728/9.779 | 40.176, 29.780, 38.116 |
| new-40 | classic | B1 | catalog | 0/3 | 0/3 | 4.390/4.375/4.404 | 23.833, 25.006, 24.248 |
| new-40 | enhanced | B1 | catalog | 0/3 | 0/3 | 4.388/4.377/4.553 | 27.468, 19.837, 21.019 |
| new-40 | classic | C | catalog | 0/3 | 0/3 | 11.036/10.864/11.862 | 30.987, 28.654, 29.333 |
| new-40 | enhanced | C | catalog | 0/3 | 0/3 | 11.197/10.880/11.501 | 26.564, 29.197, 29.535 |
| new-40 | classic | G | catalog | 3/3 | 2/3 | 12.802/10.204/12.804 | 11.633, 0.614, 0.334 |
| new-40 | enhanced | G | catalog | 1/3 | 0/3 | 4.461/4.262/4.538 | 25.593, 28.544, 31.134 |
| new-200 | classic | A | catalog | 3/3 | 0/3 | 18.090/14.235/22.233 | 7.776, 17.318, 5.614 |
| new-200 | enhanced | A | catalog | 2/3 | 0/3 | 10.280/10.060/20.031 | 24.830, 25.019, 7.189 |
| new-200 | classic | B1 | catalog | 0/3 | 0/3 | 4.590/4.533/4.814 | 28.636, 31.121, 27.829 |
| new-200 | enhanced | B1 | catalog | 3/3 | 3/3 | 9.597/9.595/9.597 | 0.360, 0.460, 0.430 |
| new-200 | classic | C | catalog | 3/3 | 0/3 | 14.757/10.867/15.332 | 35.332, 19.822, 22.527 |
| new-200 | enhanced | C | catalog | 3/3 | 0/3 | 11.723/10.622/22.399 | 29.169, 6.627, 37.132 |
| new-200 | classic | G | catalog | 3/3 | 3/3 | 12.804/12.802/12.806 | 0.281, 0.235, 0.254 |
| new-200 | enhanced | G | catalog | 3/3 | 3/3 | 12.804/12.802/12.804 | 0.642, 0.547, 0.752 |
| new-500 | classic | A | catalog | 3/3 | 0/3 | 12.837/12.073/16.504 | 35.502, 19.461, 28.480 |
| new-500 | enhanced | A | catalog | 3/3 | 0/3 | 14.038/13.600/16.793 | 15.563, 23.565, 30.639 |
| new-500 | classic | B1 | catalog | 0/3 | 0/3 | 4.590/4.415/4.754 | 24.267, 25.161, 23.511 |
| new-500 | enhanced | B1 | catalog | 3/3 | 3/3 | 9.600/9.599/9.601 | 0.445, 0.392, 0.475 |
| new-500 | classic | C | catalog | 3/3 | 0/3 | 16.802/16.181/18.214 | 13.180, 17.092, 8.431 |
| new-500 | enhanced | C | catalog | 3/3 | 0/3 | 15.860/15.817/16.497 | 17.873, 19.494, 15.603 |
| new-500 | classic | G | catalog | 3/3 | 2/3 | 12.804/10.942/12.805 | 0.267, 0.247, 6.045 |
| new-500 | enhanced | G | catalog | 3/3 | 3/3 | 12.804/12.803/12.806 | 0.366, 0.373, 0.451 |
| new-40 | enhanced | B1 | 24 | 0/3 | 0/3 | 4.495/4.399/4.512 | 24.374, 25.376, 24.139 |
| new-200 | enhanced | B1 | 24 | 3/3 | 0/3 | 5.432/5.381/5.507 | 20.335, 18.258, 18.403 |
| new-500 | enhanced | B1 | 24 | 3/3 | 0/3 | 5.263/5.251/5.357 | 18.993, 15.548, 14.746 |
| new-40 | enhanced | S-FREEZE-NORDR | 6 | 3/3 | 3/3 | 6.001/6.000/6.002 | 1.243, 1.217, 1.132 |
| new-200 | enhanced | S-FREEZE-NORDR | 6 | 3/3 | 3/3 | 6.000/6.000/6.000 | 1.258, 1.327, 1.162 |
| new-500 | enhanced | S-FREEZE-NORDR | 6 | 3/3 | 3/3 | 6.000/5.999/6.000 | 1.319, 1.327, 1.200 |
| thaw-40 | enhanced | S-FREEZE-NORDR | 6 | 3/3 | 3/3 | 6.002/6.000/6.002 | 1.205, 1.217, 1.151 |
| thaw-200 | enhanced | S-FREEZE-NORDR | 6 | 3/3 | 3/3 | 6.001/6.001/6.001 | 1.277, 1.365, 1.208 |
| thaw-500 | enhanced | S-FREEZE-NORDR | 6 | 3/3 | 3/3 | 6.001/6.000/6.002 | 1.205, 1.312, 1.204 |
| new-40 | enhanced | S-FREEZE-NORDR | 2 | 3/3 | 3/3 | 2.001/2.000/2.001 | 1.384, 1.297, 1.319 |
| new-200 | enhanced | S-FREEZE-NORDR | 2 | 3/3 | 3/3 | 2.000/2.000/2.000 | 1.286, 1.164, 1.209 |
| new-500 | enhanced | S-FREEZE-NORDR | 2 | 3/3 | 3/3 | 1.980/1.979/1.980 | 0.000, 0.000, 0.000 |
| thaw-40 | enhanced | S-FREEZE-NORDR | 2 | 3/3 | 3/3 | 2.000/2.000/2.001 | 1.603, 1.230, 1.317 |
| thaw-200 | enhanced | S-FREEZE-NORDR | 2 | 3/3 | 3/3 | 2.000/2.000/2.001 | 1.473, 1.362, 1.285 |
| thaw-500 | enhanced | S-FREEZE-NORDR | 2 | 3/3 | 3/3 | 1.990/1.987/1.992 | 0.466, 0.533, 0.499 |
| new-40 | belabox-c | B1 | catalog | 3/3 | 2/3 | 9.611/7.736/9.615 | 1.000, 45.200, 0.964 |
| new-40 | belabox-c | C | catalog | 0/3 | 0/3 | 14.294/14.102/14.772 | 57.367, 57.194, 57.730 |
| new-40 | belabox-c | G | catalog | 3/3 | 3/3 | 12.800/12.799/12.802 | 1.099, 1.103, 1.067 |
| new-200 | belabox-c | B1 | catalog | 3/3 | 3/3 | 9.616/9.615/9.617 | 0.354, 0.339, 0.405 |
| new-200 | belabox-c | C | catalog | 3/3 | 0/3 | 17.684/16.087/18.830 | 26.589, 28.562, 41.210 |
| new-200 | belabox-c | G | catalog | 3/3 | 3/3 | 12.800/12.799/12.802 | 0.912, 0.914, 0.902 |
| new-500 | belabox-c | B1 | catalog | 3/3 | 3/3 | 9.601/9.599/9.601 | 0.394, 0.375, 0.419 |
| new-500 | belabox-c | C | catalog | 3/3 | 3/3 | 22.406/22.405/22.407 | 0.255, 0.264, 0.201 |
| new-500 | belabox-c | G | catalog | 3/3 | 3/3 | 12.800/12.798/12.801 | 0.316, 0.256, 0.285 |
| new-40 | irlserver-rust | B1 | catalog | 1/3 | 0/3 | 8.099/8.061/8.122 | 29.997, 30.388, 29.752 |
| new-40 | irlserver-rust | C | catalog | 0/3 | 0/3 | 12.340/12.232/12.383 | 56.785, 57.444, 56.992 |
| new-40 | irlserver-rust | G | catalog | 3/3 | 3/3 | 12.803/12.803/12.804 | 1.721, 0.980, 0.962 |
| new-200 | irlserver-rust | B1 | catalog | 3/3 | 3/3 | 9.595/9.595/9.598 | 0.263, 0.326, 0.317 |
| new-200 | irlserver-rust | C | catalog | 3/3 | 3/3 | 22.401/22.401/22.402 | 1.006, 1.130, 0.879 |
| new-200 | irlserver-rust | G | catalog | 3/3 | 3/3 | 12.800/12.800/12.801 | 0.991, 1.774, 1.674 |
| new-500 | irlserver-rust | B1 | catalog | 3/3 | 3/3 | 9.598/9.596/9.598 | 0.309, 0.275, 0.224 |
| new-500 | irlserver-rust | C | catalog | 3/3 | 3/3 | 22.403/22.397/22.405 | 0.258, 0.254, 0.239 |
| new-500 | irlserver-rust | G | catalog | 3/3 | 3/3 | 12.803/12.802/12.804 | 0.823, 0.684, 0.787 |

## Freeze timing (per-run medians, milliseconds)

| Receiver | Mbit/s | Index | Gaps | First NAK | Repair | Repair lower bound incl. censored | Censored |
|---|---|---|---|---|---|---|---|
| new-40 | 6 | 0 | 346 | 74.83349609375 | 149.5791015625 | 149.576 | 1 |
| new-40 | 6 | 1 | 358 | 74.912841796875 | 149.787841796875 | 149.783 | 1 |
| new-40 | 6 | 2 | 321 | 74.9644775390625 | 149.55078125 | 149.551 | 0 |
| new-200 | 6 | 0 | 346 | 345.627197265625 | 420.2955322265625 | 420.287 | 4 |
| new-200 | 6 | 1 | 340 | 346.0455322265625 | 420.85302734375 | 420.830 | 7 |
| new-200 | 6 | 2 | 323 | 345.9405517578125 | 420.595947265625 | 420.561 | 2 |
| new-500 | 6 | 0 | 350 | 871.138671875 | 945.464111328125 | 945.446 | 3 |
| new-500 | 6 | 1 | 346 | 884.387939453125 | 959.08203125 | 959.051 | 5 |
| new-500 | 6 | 2 | 328 | 870.865966796875 | 945.64501953125 | 945.631 | 1 |
| thaw-40 | 6 | 0 | 331 | 0.1328125 | 75.001953125 | 75.002 | 0 |
| thaw-40 | 6 | 1 | 329 | 0.132080078125 | 74.90283203125 | 74.903 | 0 |
| thaw-40 | 6 | 2 | 334 | 0.134033203125 | 74.907470703125 | 74.907 | 0 |
| thaw-200 | 6 | 0 | 357 | 0.14990234375 | 75.0380859375 | 75.038 | 0 |
| thaw-200 | 6 | 1 | 355 | 0.146240234375 | 75.14404296875 | 75.143 | 1 |
| thaw-200 | 6 | 2 | 337 | 0.157958984375 | 75.2186279296875 | 75.218 | 1 |
| thaw-500 | 6 | 0 | 317 | 315.0013427734375 | 390.044921875 | 390.045 | 0 |
| thaw-500 | 6 | 1 | 336 | 314.493896484375 | 360.3408203125 | 360.341 | 0 |
| thaw-500 | 6 | 2 | 336 | 344.5400390625 | 390.5029296875 | 390.406 | 1 |
| new-40 | 2 | 0 | 110 | 210.496337890625 | 285.4794921875 | 285.458 | 2 |
| new-40 | 2 | 1 | 117 | 210.16796875 | 285.0999755859375 | 285.095 | 1 |
| new-40 | 2 | 2 | 118 | 210.462158203125 | 285.5533447265625 | 285.553 | 0 |
| new-200 | 2 | 0 | 119 | 1050.416015625 | 1125.202880859375 | 1125.185 | 2 |
| new-200 | 2 | 1 | 103 | 1050.343994140625 | 1125.18310546875 | 1125.114 | 2 |
| new-200 | 2 | 2 | 115 | 1050.364013671875 | 1125.328125 | 1125.316 | 1 |
| new-500 | 2 | 0 | 114 | None | None | 26482.704 | 114 |
| new-500 | 2 | 1 | 103 | None | None | 25980.519 | 103 |
| new-500 | 2 | 2 | 115 | None | None | 29415.546 | 115 |
| thaw-40 | 2 | 0 | 112 | 0.1546630859375 | 75.204345703125 | 75.204 | 0 |
| thaw-40 | 2 | 1 | 101 | 0.156005859375 | 75.238037109375 | 75.238 | 0 |
| thaw-40 | 2 | 2 | 102 | 0.141845703125 | 75.398193359375 | 75.384 | 1 |
| thaw-200 | 2 | 0 | 114 | 367.595458984375 | 442.808349609375 | 442.808 | 0 |
| thaw-200 | 2 | 1 | 107 | 405.76416015625 | 480.5361328125 | 480.536 | 0 |
| thaw-200 | 2 | 2 | 111 | 510.73583984375 | 600.671875 | 600.672 | 0 |
| thaw-500 | 2 | 0 | 112 | 1725.35302734375 | 1799.56787109375 | 25207.255 | 71 |
| thaw-500 | 2 | 1 | 91 | 1762.6168212890625 | 1815.1104736328125 | 2009.727 | 45 |
| thaw-500 | 2 | 2 | 114 | 1755.00390625 | 1799.972412109375 | 29632.531 | 74 |

## Warnings

- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 0: loadavg>2 (7.79)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 1: loadavg>2 (8.99)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 2: loadavg>2 (10.75)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (8.11)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (8.82)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (12.02)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (8.92)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (11.21)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (10.59)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 0: loadavg>2 (10.88)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 1: loadavg>2 (5.6)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 2: loadavg>2 (9.54)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (10.56)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (8.97)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.14)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (13.25)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (8.77)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (5.75)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (9.63)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (9.75)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (11.58)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (11.16)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (5.42)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (5.08)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 0: loadavg>2 (12.04)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 1: loadavg>2 (9.7)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 2: loadavg>2 (6.12)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (11.67)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (6.44)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (5.14)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (9.93)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (8.66)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (5.84)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (4.57)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (3.48)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (5.11)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 0: loadavg>2 (4.95)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 1: loadavg>2 (3.98)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 2: loadavg>2 (5.07)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.27)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (3.29)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.44)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (5.03)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (3.05)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (4.16)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.56)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (2.91)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (3.68)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.71)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (3.12)
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (3.65)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 0: loadavg>2 (6.47)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 1: loadavg>2 (6.11)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 2: loadavg>2 (5.0)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.05)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.47)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.35)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (4.63)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (4.4)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (5.71)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 0: loadavg>2 (4.25)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 1: loadavg>2 (4.35)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 2: loadavg>2 (3.71)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (6.09)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.08)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (7.13)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (4.24)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (3.75)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (5.42)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off run 0: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off run 1: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off run 2: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (10.56)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (10.3)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (9.38)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (10.78)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (10.06)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (9.2)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 0: loadavg>2 (9.7)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 1: loadavg>2 (8.39)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 2: loadavg>2 (10.81)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (11.52)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (9.91)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (11.34)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (8.88)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (8.74)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (9.34)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (10.65)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (5.38)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (4.66)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 0: loadavg>2 (9.55)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 1: loadavg>2 (3.67)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 2: loadavg>2 (4.73)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (12.52)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (3.83)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (5.6)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (10.26)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (4.03)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (4.02)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.21)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (5.34)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.13)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (2.93)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (3.21)
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (3.24)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 0: loadavg>2 (3.86)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 1: loadavg>2 (5.96)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--classic run 2: loadavg>2 (4.08)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (2.93)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (7.15)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--A@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (5.32)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (5.69)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (6.39)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (12.31)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 0: loadavg>2 (4.01)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 1: loadavg>2 (5.19)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--classic run 2: loadavg>2 (9.23)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.13)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (6.54)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.6)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (3.97)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (4.09)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (7.1)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.92)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.78)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--B1@offered-24--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.47)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (5.4)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (3.34)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (4.42)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 0: loadavg>2 (4.12)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 1: loadavg>2 (4.62)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--classic run 2: loadavg>2 (5.23)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.5)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.59)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (3.96)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (5.59)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (4.75)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--C@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (3.23)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 0: loadavg>2 (5.32)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 1: loadavg>2 (4.83)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--belabox-c run 2: loadavg>2 (5.7)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 0: loadavg>2 (4.79)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 1: loadavg>2 (4.76)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--classic run 2: loadavg>2 (5.58)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.89)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (5.52)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (3.93)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 0: loadavg>2 (3.86)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 1: loadavg>2 (5.75)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--G@--production--slt:4001--fec:off--irlserver-rust run 2: loadavg>2 (5.23)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (5.22)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (5.26)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (5.04)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.9)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.39)
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- new-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D1--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (3.64)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--classic run 0: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--classic run 0: loadavg>2 (3.35)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--classic run 1: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--classic run 1: loadavg>2 (5.02)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--classic run 2: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--classic run 2: loadavg>2 (5.2)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.31)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.74)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--A@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (7.14)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--classic run 0: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--classic run 0: loadavg>2 (7.07)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--classic run 1: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--classic run 1: loadavg>2 (6.0)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--classic run 2: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--classic run 2: loadavg>2 (4.02)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.78)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (3.84)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--B1@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.39)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--classic run 0: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--classic run 0: loadavg>2 (3.71)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--classic run 1: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--classic run 1: loadavg>2 (3.99)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--classic run 2: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--classic run 2: loadavg>2 (4.16)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.43)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.02)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--C@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.65)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 0: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 1: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off run 2: failed (settle_timeout)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--classic run 0: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--classic run 0: loadavg>2 (5.69)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--classic run 1: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--classic run 1: loadavg>2 (4.74)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--classic run 2: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--classic run 2: loadavg>2 (4.5)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.85)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (3.89)
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- ours-old@%26nakreport%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--G@--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (5.4)
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (6.2)
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (5.32)
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (5.2)
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.69)
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.12)
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- thaw-200@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D200%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (3.59)
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.02)
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.13)
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.99)
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (3.57)
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (3.46)
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- thaw-40@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D40%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (4.88)
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.53)
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.07)
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-2--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (5.4)
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 0: loadavg>2 (4.97)
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 1: loadavg>2 (4.41)
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2
- thaw-500@%26nakreport%3D1%26periodicnakgate%3D1%26lossmaxttl%3D500%26reorderfreeze%3D0--S-FREEZE-NORDR@offered-6--production--slt:4001--fec:off--enhanced run 2: loadavg>2 (6.37)
