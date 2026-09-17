# M1: static TTL200 fallback; controller not triggered

**Spike complete; universal performance criterion failed.** No production receiver
or sender default was changed. This session resumed and preserved a prior session's
tested, uncommitted harness work rather than restarting implementation.

## Measurement

- **65 cells, 195 one-attempt indices**, no omitted foreign or low-rate freeze arm.
- **14,847.179 seconds (4h 7m 27s)** real namespace/netem campaign.
- **136 settled, 59 measured settle timeouts**; runner exit **101** retained.
- All 195 yielded full-window observations; M1 report exits **0** because outcome
  coverage is complete, not because every performance condition passed.
- Load average exceeded the reporter's informational threshold of 2 on every run
  (range **2.91–13.25**). Affinity and sequential execution are proven configuration;
  an independently certified idle host or a cause of variation is not claimed.
- Source head `0206dd3359bca79e2383fd8530437c50181a19cf`, with no production-source
  diff. Harness changes were uncommitted during measurement. Immutable sender SHA256:
  `fc0c210491c3634e30f42ee906651ac2880fcf7eb5501a445be3980ec8889257`.

## Frozen decision

| TTL | Core passing / 8 | Foreign passing / 6 | Total passing / 14 |
|---|---:|---:|---:|
| 40 | 0 | 3 | 3 |
| 200 | 3 | 5 | 8 |
| 500 | 2 | 6 | 8 |

**TTL\* = 200**, branch `core_failure_upstream_parity`: no TTL passes all four
core scenarios for both senders. The explicit upstream-parity fallback therefore
applies. Even the secondary count/tie rule would favor 200 over 500 (8 total each,
3 versus 2 core cells). The freeze cap independently prohibits selecting 500.

At TTL200 the passing core cells are enhanced/B1 and both G cells. Failing examples:

| Cell | Settled | Received retransmissions by index | Useful goodput by index (Mbit/s) |
|---|---:|---|---|
| classic/A | 3/3 | 7.776%, 17.318%, 5.614% | 18.090, 14.235, 22.233 |
| enhanced/A | 2/3 | 24.830%, 25.019%, 7.189% | 10.060, 10.280, 20.031 |
| classic/B1 | 0/3 | 28.636%, 31.121%, 27.829% | 4.814, 4.590, 4.533 |
| classic/C | 3/3 | 35.332%, 19.822%, 22.527% | 10.867, 15.332, 14.757 |
| enhanced/C | 3/3 | 29.169%, 6.627%, 37.132% | 11.723, 22.399, 10.622 |
| BELABOX/C | 3/3 | 26.589%, 28.562%, 41.210% | 17.684, 18.830, 16.087 |

`spike.json.failing_cells` contains **every** failing new-receiver core/foreign
cell, at every TTL, with all three numeric observations. The old baseline and all
diagnostics remain in `report.md` / `summary.json`. Todo 8's in-flight-aware NAK
rule is the compensating mechanism required by the fallback, not a claim that it
rescues these failures.

## Controller and freeze

Enhanced/B1 at 24Mbit offered:

| TTL | Median useful Mbit/s | 95% bootstrap median CI |
|---|---:|---|
| 40 | 4.495222 | [4.398832, 4.511599] |
| 200 | 5.432214 | [5.381446, 5.507314] |
| 500 | 5.262830 | [5.250665, 5.357114] |

**Controller = false:** the best diagnostic TTL is 200, identical to TTL\*; the
gap against that same choice is 0%, so the trigger's conjunction fails.

The worst identifiable freeze penalty is on the required **2Mbit arm at TTL200**:
enabled median repair lower bound **1125.185ms**, disabled upper bound **480.536ms**,
penalty **at least 644.649ms**. At 6Mbit the corresponding penalty is >=345.416ms.
Both exceed 250ms; TTL\* stays capped at 200 even though the fallback already chose
200. Record all three consequences:

1. L3 keeps freeze **OFF** — justified by the measured non-reordering-path penalty.
2. A one-link bond inherits that penalty — accepted, not hidden as bonding gain.
3. Freeze and TTL are coupled and must be decided jointly.

TTL500/2Mbit is worse than its low retransmission figure suggests: freeze-on has
**332 detected gaps and zero observed repairs/NAKs across three runs**. Freeze-off
still has 190 censored gaps. Their relative median penalty is **unidentifiable**
and stays null. It is not reported as a zero penalty or invented finite recovery.
The independent TTL200 result already proves the safety cap. See `method.md` for
full-capture timing scope, censor bounds and the reduction correction.

## Reproduce

Raw data is retained at:
`/home/andres/.cache/opencode/tmp/opencode/m1-ttl-measurement/`.
`provenance.json` hashes **1,050 input files** including 195 result records, receiver
and sink CSVs, clocks, and all freeze pcaps/capture logs. Large raw artifacts remain
outside Git; the portable reduced observations and their input hashes are committed.

```sh
uv run scripts/bench/report.py \
  --results /home/andres/.cache/opencode/tmp/opencode/m1-ttl-measurement/results \
  --manifest scripts/bench/manifests/m1-ttl.json \
  --out docs/evidence/bpc/m1-ttl/report.md \
  --json docs/evidence/bpc/m1-ttl/summary.json
uv run scripts/bench/decide.py --rule m1-ttl \
  --summary docs/evidence/bpc/m1-ttl/summary.json --out /tmp/m1-first.json
uv run scripts/bench/decide.py --rule m1-ttl \
  --summary docs/evidence/bpc/m1-ttl/summary.json --out /tmp/m1-second.json
cmp /tmp/m1-first.json /tmp/m1-second.json
```

Two actual invocations matched byte-for-byte; both SHA256:
`5abb87999b500d4d01314b7be08f74a240325c5a52dcfd99559fba6bc48fe7b9`.
Required `jq` spike-schema acceptance exits 0. Unit/CLI fixtures are synthetic and
kept separate from campaign results; no preflight or interrupted-launch observation
was pooled into these 195 indices.
