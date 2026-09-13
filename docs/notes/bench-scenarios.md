# Scheduler benchmark scenarios

The scenario library will extend this document. This section freezes the receiver-side
measurement contract implemented by `network_sim::metrics`; it does not claim a scheduler
performance result or hardware validation.

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
writes. The future scenario library must preserve that composed waveform and bounds.
