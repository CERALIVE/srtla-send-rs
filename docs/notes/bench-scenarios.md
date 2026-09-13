# Scheduler benchmark scenarios

`network_sim::scenarios` provides thirteen concrete profiles in twelve families (B1/B2).
The library and frozen receiver-side measurements below are benchmark inputs and
contracts, not scheduler performance results or hardware validation.

## Scenario library A–L

Call `scenario_a()` through `scenario_l()` (B uses `scenario_b1()` and `scenario_b2()`),
or `all()` for stable `(id, Profile)` pairs. `scenarios::Profile` contains the unchanged
temporal `profile::Profile` in `timeline`, plus `offered_bps`, `warmup_offered_bps`,
`srt_profile`, optional `source_ramp` and optional `receiver_restart_budget`.
Pass `&profile.timeline` to the scheduler and metric evaluators; pass
`profile.aggregate_capacity_bps()` explicitly to `load_intervals::evaluate`.
The wrapper's `validate()` calls temporal validation on the **full expansion**, then
checks source policy. It neither truncates a late cycle nor relaxes its horizon.

### Common execution contract

1. Wait until **all** links have registered; start the real SRT source at
   `warmup_offered_bps` **before** settling, not after it.
2. Settle on sink rate ≥0.9×`warmup_offered_bps` for **three consecutive seconds**,
   bounded by **30 seconds** from source start; fail the run as `settle_timeout`
   otherwise. The exported `WARMUP_SETTLE_RATIO`, `WARMUP_SETTLE_SECONDS` and
   `WARMUP_TIMEOUT` pin these settings. Retain ten seconds of warm-up sink history
   for impairment baselines before starting measurement.
3. Anchor event clocks at measurement start. Every profile includes an explicit
   t=0 `OfferedRate` boundary; L changes from feasible warm-up to overload there.
   The runner must apply the boundary to the source's live control pipe.
4. Use `SrtProfile::PRODUCTION` (2000ms latency, `lossmaxttl=40`) unless the campaign
   manifest explicitly overrides it. Preserve that effective profile in results.

This task supplies definitions, not the campaign runner. The runner must enforce
settling, source-control ramps, and restart wall-time budgets; executing `timeline`
alone cannot enforce metadata it does not own. In particular, L's `SourceRamp` means
linear 0→10Mbit over 400ms starting at t=30. Do not treat it as an immediate rate jump,
restart the source, or emit intermediate `OfferedRate` log boundaries: those would
incorrectly split the single burst `LoadInterval` and change its grading target.

Rates below are decimal Mbit/s; `rate_kbit` uses decimal kbit/s and **TBF enforcement**.
Delays are the exact **netem delay values**, not RTT values divided by two. Unless
specified otherwise, synthetic LTE links use 60ms, 0.2% random loss, limit 500,
no jitter, direct carriers and TBF's existing default latency (1s). Positive jitter
uses the normal distribution. GE overrides random loss. Supplemental link rates,
windows and walk bounds not pinned by the task are explicit synthetic choices below.
Warm-up equals offered rate except L. All events are graded unless explicitly noted.

| ID | Constants and timeline (seconds relative to measurement) | Offered / warm-up Mbit/s | Window | Citation and grading |
|---|---|---|---|---|
| A | 3× LTE: 60±15ms normal, 10Mbit TBF, 0.2% loss, limit 500 | 24 / 24 (80% aggregate) | 45s | [6], [7], [13]; viewer loss + useful goodput, no-collapse |
| B1 | Historical topology: 20/60/120ms at 4Mbit each | 9.6 / 9.6 | 45s | Historical synthetic control, [6–9] heterogeneity; viewer loss + useful goodput, no-collapse |
| B2 | 35±5/90±20ms normal at 8Mbit each | 12.8 / 12.8 | 45s | [6–9], [13]; viewer loss + useful goodput, no-collapse |
| C | NAT Starlink 45±5ms normal, 20Mbit, limit 4000, base loss 0%; +74ms for 215ms every 15s; capacity 50% for 500ms every 15s; LTE 65ms, 8Mbit, 0.2% | 22.4 / 22.4 | 60s | [1–3], [6–9]; viewer loss + useful goodput and 5s periodic episodes |
| D | C plus Starlink `DataBlackhole` on at 20, off at 28; keepalives pass; 30s fault recovery horizon | 22.4 / 22.4 | 75s | [1–3], [13], [24], workspace srtla-starlink-lan-diagnosis note; viewer loss + goodput, failover/outage/rejoin episodes |
| E | LTE 10Mbit TBF latency 2s, netem 60ms limit 1000; `CrossTraffic` 9Mbit at 15–45; second link healthy 8Mbit; 30s cross-load recovery horizon | 14.4 / 14.4 | 75s | [9], [13]; viewer loss + goodput, cross-load recovery episode |
| F | Two 8Mbit links; link 1 `gemodel 1% 0.2 0.5 0.01` | 12.8 / 12.8 | 45s | [6–9] inferred usable-loss stress, [13]; viewer loss + useful goodput; link shares diagnostic |
| G | Marginal 1Mbit link 0 `gemodel 5% 0.1 0.8 0.02`, beside 3×5Mbit | 12.8 / 12.8 | 45s | [6–9] inferred weak-link stress, [13]; viewer loss + useful goodput; link shares diagnostic |
| H | 3×5Mbit; t=15 link2 down, 25 up, 35 link1 replug, 50 link0 default route off, 60 on, 70 SIGHUP reorder `[2,0,1]`; 30s recovery tails except final reorder 20s | 12 / 12 | 90s | Synthetic lifecycle control [13], [24], workspace diagnosis note; viewer loss + goodput, lifecycle recovery episodes |
| I | Two 8Mbit links; receiver restart at 20; exact receiver kill+respawn budget 2s; actual restart gap **ungraded**, 30s recovery horizon | 12.8 / 12.8 | 60s | [20–23], receiver-restart control; no-collapse outside gap, post-restart recovery; loss/goodput recorded |
| J | Two 8Mbit links; **all links** at 100% loss from 20 to 23; loss onsets/restores **ungraded**, 30s recovery horizon | 12.8 / 12.8 | 60s | [6–9] inferred stalls, [13]; **only** no-collapse outside forced gap + episode recovery; whole-window loss/goodput diagnostic |
| K | C's Starlink capacity-only cycle (no delay spike); LTE walk seed 42, step 2s, rate 2–8Mbit ±1Mbit/step, base delay 65ms, jitter 20ms, delay step 5ms, loss ≤1% ±0.1 percentage point/step; samples ungraded with zero tails | 12 / 12 | 60s | [1–3], [6–9]; viewer loss + useful goodput, 5s periodic recovery; walk episodes diagnostic |
| L | Two 8Mbit links; t=0 offered 1.25×aggregate for 20s; t=20 idle 10s **ungraded**; t=30 ramp 0→10Mbit over 400ms, graded, horizon 10s | 20 / 12.8 (warm-up 0.8×aggregate) | 60s | Synthetic overload [13], SRT [20–23]; sustained overload **only no-collapse**; burst LoadInterval target acquisition + no-collapse; no idle obligation |

### Periodic expansion and bounded recovery

C/D compose simultaneous whole-impairment changes without overlapping holds:
215ms at delay 119ms/rate 10Mbit, then 285ms at delay 45ms/rate 10Mbit, then restore
45ms/20Mbit. Both phases recur every 15s with 5s horizons. At the shared 215ms
boundary the scheduler restores the first hold before starting the second; the
equal-timestamp base transition has no modeled dwell time. K uses one 500ms
half-capacity hold. All periodic declarations set **`until = duration − 6s`**.

Onsets start at t=15 (second C/D phase at 15.215). C/K `until=54s` gives final
restore 45.5s and horizon end 50.5s. D `until=69s` gives final restore 60.5s and
horizon end 65.5s. `until` is an **inclusive onset bound**, not an extra onset:
even a cycle beginning at 69s would have 69.5+5=74.5s inside D's 75s window.
D's blackhole restore at 28s plus 30s ends at 58s. H's route restoration plus tail
ends at 90s; its 70s reorder explicitly uses 20s to fit the same window. Restore
edges have zero tails rather than creating a second recovery obligation.

The failure control adds a 15s periodic event with first onset 14s, `until=59s`,
500ms hold and 5s horizon to a 60s profile. Its early cycles fit, but final onset
59s does not: `Profile::validate()` rejects it without silently clipping that cycle.

### Grading integration

Use Todo 11's [frozen metric definitions](#metrics), unchanged. Always record useful
sink goodput, viewer loss and diagnostics; the table controls which become gates.
No-collapse means ≥0.7×min(offered, aggregate) in ≥90% of eligible one-second buckets.
I/J's actual forced gaps and L's planned idle are excluded, never inferred from
observed low rate. Retain episode records for I/J to assess post-outage recovery;
their ungraded forced-loss interval must not erase that recovery evidence.

For L, pass the explicit **16,000,000bps** aggregate capacity to the evaluator:
the overload target is **14,400,000bps = 0.9×aggregate**, not the impossible
0.9×20Mbit offered rate. Sustained overload is gated only on no-collapse. The burst
target is explicitly **9,000,000bps = 0.9×min(10Mbit, aggregate)**. Its `reached_ms`
must be `Some` within ten seconds; a still-dead sink has `None` and `recovered=false`.
The idle prehistory is **not** an impairment baseline. Unit integration tests feed
the real scenario expansion into the frozen evaluator and pin both targets and the
dead-sink failure, rather than copying the metric formula into another evaluator.

### Citation key

Numbers retain the approved draft's “Reference-backed link profiles” numbering.
These references motivate the ranges; the concrete synthetic controls are not
claimed to be verbatim measurements from every cited paper.

- [1] Nature `s44459-026-00044-z`; [2] `OASIcs.NINeS.2026.7`;
  [3] `sigcomm26-dissect-starlink` — satellite delay, reconfiguration and capacity.
- [6] DOI `10.1145/3618257.3624814`; [7] Mahimahi traces;
  [8] DOI `10.1145/2342468.2342470`; [9] Alfredsson, WoWMoM 2013 — LTE ranges.
- [13] `tc-netem(8)`; [24] `tc-u32(8)` — impairment and DATA-selective classifier.
- [20] SRT `latency.md`; [21] `srt-live-transmit.md`; [22] BELABOX README;
  [23] SRT `statistics.md` — SRT settings and measurement context. The corrected
  real CSV names/cadence in the frozen definitions below supersede the draft's
  preliminary field-name and timer assumptions.

```bash
cargo test -p network-sim --lib scenarios
cargo clippy -p network-sim --all-targets -- -D warnings
bash scripts/check-doc-refs.sh
```

## Metrics

### Frozen receiver definitions

PRIMARY `viewer_loss_ratio = Δpkt_drop_total / (Δpkt_recv_unique + Δpkt_drop_total)`
(0 when the denominator is 0 and the sink received nothing ⇒ mark `no_traffic`);
`pkt_belated` and loss/retrans are diagnostics.

`useful_goodput_bps` = sink bytes in window ×8/window.

The window denominator for goodput is **seconds**, not milliseconds. Stored clocks are
signed milliseconds relative to measurement start, so negative times retain warm-up.
`Window` requires `end_ms > start_ms`. All window selection uses timestamps, never
row indices. CSV rows and 100 ms sink buckets use `(start_ms, end_ms]` membership.
Cumulative deltas use the last CSV row ≤ start and the last CSV row ≤ end. A missing
start baseline is an error, not a presumed zero. Counter decreases or a socket-ID
change across a cumulative window are typed errors, not saturating subtraction.

**Corrected libsrt 1.5.5 fields (case-sensitive):**

| CSV header | Metric name | Frozen window operation |
|---|---|---|
| `pktRecv` | `pkt_recv_total` | Cumulative delta |
| `pktRecvUnique` | `pkt_recv_unique` | Cumulative delta |
| `pktRcvLoss` | `pkt_loss_total` | Cumulative delta |
| `pktRcvDrop` | `pkt_drop_total` | Cumulative delta; REQUIRED |
| `pktRcvRetrans` | `pkt_retrans_total` | Cumulative delta |
| `byteRecv` | `byte_recv` | Cumulative delta; bytes, not bits |
| `pktRcvBelated` | `pkt_belated_sum` | Interval SUM, not a difference |
| `pktReorderDistance` | `reorder_distance_max` | Optional maximum; genuinely absent on this libsrt |
| `msRTT` | `ms_rtt_median` | Median of rows inside the window |
| `mbpsRecvRate` | `mbps_recv_rate_mean` | Arithmetic mean of rows inside the window |

The real header begins `Timepoint,Time,SocketID,Time,...`. The **second `Time`**,
selected by header position, is the numeric millisecond clock; neither the first
matching name nor `SocketID` is used. `clock_offset_ms` aligns this socket clock with
the sink/event clock. Missing `pktRcvDrop` returns `MetricError::MissingColumn` naming
that exact column. Missing `pktReorderDistance` produces `None` (JSON `null` in the
result), never a parse failure. No rows inside a window produces `None` for gauges.

Diagnostic denominators are explicit: `loss_ratio = Δpkt_loss_total /
(Δpkt_recv_total + Δpkt_loss_total)` and `retrans_ratio = Δpkt_retrans_total /
Δpkt_recv_total`; each is zero for a zero denominator. These are not the primary
viewer-loss gate. Buffered sink delivery can coexist with zero CSV deltas; in that
case `viewer_loss_ratio` is zero but `no_traffic` is **false**.

### Capture semantics and the corrected live evidence

The requested frozen calculation is exposed by `SrtStats::parse`, with recorded
`capture_semantics: "cumulative_packets_interval_belated"`. Do not silently apply
it to another capture mode. The later live CSV-smoke investigation demonstrated
that the existing listener's `-statsout ... -statspf:csv -stats 1000` (without
`-fullstats`) emits **interval** packet counters, including decreasing counts.
For those exact flags, use:

```rust,ignore
SrtStats::parse_with_semantics(csv, clock_offset_ms, CaptureSemantics::Interval)?
```

This explicit mode prefix-sums the six receive/byte counters before applying the
same delta-window algorithm, leaves interval belated values alone, and records
`capture_semantics: "interval"`. The original header and every raw CSV row remain
unchanged. Neither mode guesses semantics from whether a sequence happens to rise.
`-fullstats` also makes belated cumulative and is **not** one of these modes; do not
add that flag as a purported fix without a separate explicit normalization contract.

The stats count is packets, **not milliseconds**. The earlier pause observation
included zero-valued rows, but later evidence showed an actual reporting gap:
buffered application reads can still trigger reports during a source pause, while
indefinite silence need not. Both zero-valued rows and gaps are supported. Never
use an assumed timer, outage cadence, or row number to align CSV with viewer delivery.

### Frozen impairment-episode definition

Every IMPAIRMENT event (SetImpairment, DataBlackhole, LinkUp, DefaultRoute, Replug,
ReceiverRestart, SighupReorder, CrossTraffic) forms an:

> `Episode { onset_ms, restore_ms: Option, baseline_bps = mean sink rate over the 10 s before onset, horizon_ms = the originating TimedEvent.horizon (30 s one-shot / 5 s periodic micro-event — NEVER an independent default) after restore (or after onset when there is no restore), impacted: bool = sink rate fell below 0.5×baseline within the episode, failover_ms: Option = onset → first 1 s bucket ≥ 0.9×baseline AFTER the first sub-0.5 bucket (None if never recovered within horizon; 0 if !impacted), recovery_ms: Option = restore → first bucket ≥ 0.9×baseline (None if never), outage_ms = Σ buckets < 0.1×baseline inside the episode, recovered: bool }`.

Implementation details that prevent premature success:

- Consume the scheduler's **actual** `EventLog` timestamps and the original
  `TimedEvent.horizon`. Expanded `Periodic` actions must already be concrete.
- Infer a restoration from the previous property state on the same link, using
  the profile scheduler's shared initial states. A zero-tail restoration remains
  a diagnostic episode at its own onset, not a new hold extending to the next fault.
  Cross-traffic on/off holds still work with their default zero tail. Irreversible
  restart/replug events have no synthetic restoration.
- Detect impact and sum outage at the original sink resolution (normally 100 ms).
  Confirm recovery only at the **end of a complete one-second bucket whose start
  follows the first impact bucket**. Ignore buckets straddling an onset/restore.
  This is bounded detection, not an instantaneous recovery timestamp.
- A two-second SRT buffer can keep the first post-onset buckets at baseline.
  Those buckets are not recovery evidence. Before impact, an unfinished horizon
  has `failover_ms: null`; a fully observed, genuinely unimpacted horizon has `0`.
- `complete` requires both the ten-second baseline and the entire episode to be
  observed. Missing baseline is not evidence of success; the diagnostic baseline
  is zero and the episode cannot pass. `recovered` requires failover evidence and,
  when a restore exists, post-restore recovery evidence as well.
- `event_index`, `end_ms`, `graded`, and `complete` are additional audit fields.
  Do not replace an originating five-second horizon with a thirty-second default.

### Frozen source-load definition

`OfferedRate` events are SOURCE-LOAD BOUNDARIES, not impairment episodes: they never
use the 10 s-before baseline; a graded `OfferedRate{bps, graded: true}` interval is
graded as a:

> `LoadInterval { start_ms, end_ms, target_bps = 0.9 × min(bps, aggregate_capacity_bps), no_collapse = sink rate ≥ 0.7 × min(bps, aggregate) in ≥ 90 % of its buckets, reached_ms: Option = first bucket ≥ target_bps (None if never — a dead sink NEVER counts as reached), recovered = reached_ms.is_some() }`.

An `OfferedRate{0}` idle interval is `graded: false` and creates no obligation.
Forced total-outage intervals (scenario J's all-links loss, I's restart gap) are
`graded: false` by the scenario definition and excluded from no-collapse grading.

Intervals end at the next offered-rate boundary or profile duration. A positive
offered-rate horizon additionally bounds target acquisition (L's burst uses ten
seconds); a zero horizon uses the interval end. `reached_ms` is elapsed from the
interval start to the confirming one-second bucket's end. A positive sink rate is
required even if an explicitly supplied aggregate capacity is zero. Missing sink
coverage cannot pass no-collapse. `offered_bps` and `graded` are retained in each
serialized interval. Ungraded results must not become report obligations.

Ungraded reversible fault holds are excluded using their actual onset/restore
times; an ungraded synchronous action's scheduled-to-completed gap is also excluded
(the restart adapter completes after respawn). A one-second bucket overlapping an
excluded interval is excluded as a whole. Zero-tail restore edges must not exclude
the following healthy interval. Scenario definitions must explicitly label forced
outages; collectors never infer them from low throughput.

## Collector interfaces

- `sink::spawn(namespace, port, path)` starts a pcap-free Python UDP sink on receiver
  loopback, using process-only RAII teardown. It prints
  `ready,<bound-port>,<monotonic-origin-ns>` after binding and writes
  `t,bytes,pkts` every 100 ms, including zero-traffic buckets. `t` is bucket-end ms
  relative to that origin. Calibrate the ready origin against the event clock and
  supply the offset to `SinkSeries::parse`. Keep warm-up data. Do not run another
  sink on the same port. Python is a host harness dependency, not a sender dependency.
- `sampling::sample_for(duration, callback)` is a blocking, caller-owned 1 Hz lane
  with initial/final samples and actual elapsed timestamps. It skips missed ticks
  rather than synthesizing snapshots. Run it separately from the event scheduler,
  align its launch clock, and join before namespace/process teardown.
- `link_counters::sample(Some(ns), iface, t_ms)` reads the namespace's real sysfs
  `tx_bytes`, `tx_packets`, and ifindex. `None` reads the current namespace and
  supports unprivileged `lo` tests. `shares` banks newly observed generations;
  same-generation decreases fail. These are netdev bytes (including control/ARP),
  **not useful goodput**. Capture final counters before a planned replug when exact
  totals are needed: periodic sampling cannot recover unobserved bytes from a
  destroyed interface.
- `StatsFileSample::sample(path, t_ms)` returns absent for a missing opt-in file,
  preserves required signed `window`/`in_flight` and weight, and keeps optional
  health, priority, iface, link_id. `last_updated_ms` remains wall-clock producer
  time; the sample's `t_ms` is the observation clock. Unknown health strings are
  retained for future candidates. Missing optional fields are omitted, not invented.
- `control::query(path, timeout)` sends `metrics\n` with bounded I/O and a 64 KiB
  response limit. Missing/refused sockets, empty/unsupported replies, and a silent
  timeout skip gracefully. Malformed supported counters fail. The three monotonic
  counters are delta-calculated; decreases fail. Optional process-reported
  `adaptive_features` / `adaptive_tuning` remain opaque JSON under `effective_config`.
  They are never reconstructed from command-line arguments. Non-Unix builds return
  absent for this Unix-only capability.
- `CpuSample::sample(pid, t_ms)` reads stat fields 14+15 after correctly parsing the
  parenthesized command name, `/proc/loadavg`, and `/proc/<pid>/status` `VmHWM`.
  `delta` verifies PID/start-time identity and converts ticks with an explicit
  `NonZeroU64` clock rate (`clock_ticks_per_second()` uses `getconf CLK_TCK`, no FFI).
  Peak RSS is the process high-water mark in kB, not just endpoint RSS.

## RunRecord schema (version 1)

`network_sim::metrics::RunRecord` is serde-serializable JSON with:

- Identity: campaign, cell_id, UUID run_id, scenario `{id,family,hash}`, candidate
  `{label,bin_sha256,args,env}`, receiver `{kind,sha256}`, owned srt_profile,
  fingerprint, run_index, seed, order_index, started_at, status (`ok` / `failed`).
- Window results: window, useful_goodput_bps, offered_bps, viewer_loss_ratio,
  no_traffic, diagnostics, per_link, sender, events, **episodes and load_intervals**,
  loadavg_1m, warnings.
- Sender summary: optional switch/NAK deltas, CPU ms, peak RSS kB, optional
  effective_config, and timestamped stats-file samples.
- Raw evidence: stats_csv_path, sink_series_path, embedded optional stats_csv
  (capture semantics, original header/rows and aligned typed counters), sink_series,
  link_counters, control_metrics, and optional CPU edge snapshots/clock rate.

The fingerprint is SHA256 over the JSON tuple
`[candidate, receiver, scenario.hash, srt_profile, effective_config]`. Environment
maps and opaque JSON maps serialize in sorted-key order. Digests and UUIDs are
validated at deserialization; unsupported schema versions and invalid windows fail.
The fingerprint is computed with `compute_fingerprint()` after assigning the
process-observed configuration. Absence is different from an empty configuration.

The committed [example](../../crates/network-sim/fixtures/run-record-example.json)
has one episode and one load interval. Its synthetic raw series reproduces its
primary summaries; hashes identifying imaginary binaries/scenarios are explicitly
illustrative. A test pins both JSON roundtrip and its real computed fingerprint.

```bash
cargo test -p network-sim --lib metrics
cargo clippy -p network-sim -- -D warnings
```

These tests require no namespaces, packet capture, or privileges. They include an
actual ephemeral-port UDP sink and Unix-socket query, plus offline CSV/JSON and
unprivileged Linux proc/sys reads. C/D horizon fixtures expand the complete delay
spike and capacity-reset waveform (and D's 20–28 s obstruction); because the profile
model holds a whole impairment property, the 500 ms rate dip is expressed as a
215 ms combined spike followed by its remaining 285 ms capacity dip, not overlapping
writes. The scenario library preserves that composed waveform and bounds.
