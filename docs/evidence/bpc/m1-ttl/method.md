# M1 measurement and decision method

The manifest declares 65 cells and 195 independent, one-attempt indices. The
runner retains its original exit status; report completion does not imply every
cell settled. Only `ok` and fully measured `settle_timeout` outcomes are eligible.
Infrastructure or metric-collection failures block reduction, not silently count
as evidence for the fallback. QA and interrupted-launch artifacts are excluded.

## Frozen rule interpretation

1. A core cell passes with 3/3 runs both settled and <=5% received retransmissions.
   A foreign cell passes with >=2/3 runs both settled and <=10% retransmissions.
2. Choose the smallest of 40/200/500 that passes all eight core and six foreign
   sender/scenario cells. If none does, rank total passing cells, core passing
   cells, then smaller TTL. The explicitly specified no-core-TTL fallback takes
   precedence: if no TTL passes all eight core cells, use upstream-parity 200 and
   retain failing cells with numbers. Todo 8's in-flight-aware NAK protection is
   the compensating mechanism, not a claim that it cures every failed scenario.
3. Freeze penalty bounds the difference between median-of-three per-run repair
   latency medians for enabled/disabled arms. Both offered rates and all TTLs are
   reported. The maximum identifiable penalty supplies the safety cap; the 2Mbit
   arm is never replaced by the higher-packet-rate result. >=60ms records all three
   required consequences; >250ms caps the selected TTL at 200.
4. Best 24Mbit TTL is greatest median useful goodput, ties smaller TTL. Controller
   requires that it differ from the final static choice, improve its goodput by
   >=5% relative to that choice, and have CI lower strictly above that choice's
   CI upper. Median bootstrap CIs use 10,000 resamples and seed 20260913.

These interpretations were recorded before full-campaign reduction. The rule is
implemented in `scripts/bench/m1_rule.py`, with adversarial synthetic tests kept
separate from measured evidence.

## Freeze timing

One 8Mbit link, 60ms netem delay verbatim, no jitter, 1% random loss with explicit
netem seed `42 + run_index`. The specified 60ms decision threshold is not a claim
that the actual round trip equals the configured one-way delay. Capture starts
before registration and covers warmup plus the full 45-second measurement window.
The source uses the arm's 2/6Mbit rate during both warmup and measurement.

At the receiver's loopback SRT input, a new higher sequence detects a gap. Time to
the first loss-list NAK naming that sequence and to its first eventual arrival are
recorded independently. Duplicate DATA does not create another gap. NAK ranges
and sequence wrap are decoded. Kernel capture drops must be zero.

Unrepaired gaps are retained as right-censored at the last captured packet. Both
observed-repair medians and medians of the capture-end lower bounds are reported.
For an upper bound, unrepaired entries are assigned infinity rather than dropped.
The safety comparison subtracts the disabled median's upper bound from the enabled
median's lower bound; this establishes a lower bound on the penalty. If the disabled
upper bound is unbounded, that comparison is null and cannot trigger the cap.
Censored counts remain visible, and these bounds are not claimed to estimate
unobserved eventual repair times.
The method measures wire repair, not playback delay or application recovery.

The explicit upper-bound/null handling was added during reduction after observing
TTL500/2Mbit's lack of repairs. The initial reducer refused such captures instead
of inventing zero latency. This correction does not change any threshold or select
a favored TTL: TTL200/2Mbit independently demonstrates a >=644.649ms penalty.

## Provenance and limits

- Ours: default-feature release build copied to read-only `target/m1-srtla_send`.
  The live harness is built with `test-internals`; no ablation overrides are set.
- BELABOX sender source: `37862da3d0c13b46956efd3f88877053293d97d6`.
- irlserver sender source: `df0b3938791ff24eced4aed8b29e3d49d0efb639`, classic.
- New receiver SRT source: `ca14c8bd06c89d2fd7b69bb3d8eea48dd47c2e3e`.
- Old receiver SRT source: `b06fdb6b85937f3f5cf5452b150a6bb7e35b0226`.
  This genuine binary lacks the freeze URI row; its default-off behavior and stock
  periodic NAK are preserved. It is not emulated or mislabeled freeze-on.
- Both receiver cells use the CeraLive SRTLA receiver from the lineage lock.
- Sequential campaign, host measurement lock, entire runner on CPUs 4–27.
- Real Linux namespace/netem measurements, not validation on radio hardware.
