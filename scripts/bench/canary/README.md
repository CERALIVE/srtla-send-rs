# Real-bond canary

`run-canary.sh` is **owner-run only**. It does not make a bench host into a
substitute for the Starlink + cellular rig. The script runs two sequential
30-minute enhanced-mode arms against the same production receiver (SRTLA port
`9000`), captures each sender's atomic `--stats-file` snapshots, polls the
receiver's `/stats`, and leaves raw material plus a machine-readable verdict.

The arms are deliberately both present even though Todo 34 retained
`enhanced` as the default through the empty-set fallback:

1. `survivor`: this worktree's post-ablation enhanced build.
2. `ours-3.3.0`: the immutable released binary from
   `scripts/bench/receivers.lock.json` at
   `.candidates["m3-interop"]["ours-3.3.0"]`.

This detects a shared-path regression between the changed tree and the released
3.3.0 sender; it is **not** evidence for a default-mode flip.

## Dry-run

This is safe on a development or bench host. It validates only the local lock
shape and command-line arguments; it does not bind a port, read a modem list,
start a sender, or contact a receiver.

```bash
bash scripts/bench/canary/run-canary.sh --dry-run
```

## Owner invocation on the real rig

Build the survivor from the exact worktree revision first, put it at the
read-only path supplied through `CANARY_SURVIVOR_BIN`, and use the same source,
receiver, bind-map/IP-list publication, and traffic offer for both arms. The
source command is run under a fixed 1800-second timeout and receives
`CANARY_LISTEN_PORT` plus `CANARY_SESSION_SECONDS` in its environment.

```bash
CANARY_SURVIVOR_BIN=/opt/canary/srtla_send-survivor \
CANARY_RECEIVER_HOST=receiver.example.net \
CANARY_BIND_IPS_FILE=/run/ceralive/modem-ips.txt \
CANARY_EXPECTED_LINKS=2 \
CANARY_STATS_URL=http://receiver.example.net:8080/stats \
CANARY_STREAM_COMMAND='exec /opt/rig/start-source --srt-port "$CANARY_LISTEN_PORT" --seconds "$CANARY_SESSION_SECONDS"' \
bash scripts/bench/canary/run-canary.sh --output /var/tmp/srtla-canary-$(date -u +%Y%m%dT%H%M%SZ)
```

The receiver must be production on SRTLA UDP port `9000`, or a staging receiver
with the same build. The `/stats` endpoint may be separate; its URL is explicit
to avoid guessing the receiver's HTTP listener.

The defaults expect `/stats` to expose a receive-rate field named
`mbpsRecvRate`, `mbps_recv_rate`, `goodput_mbps`, or `goodputMbps`, and a
cumulative drop counter named `pktRcvDrop`, `pkt_rcv_drop`, `pktRcvDropTotal`,
or `pkt_rcv_drop_total`. For another known-compatible receiver build, set
`CANARY_GOODPUT_JQ` and `CANARY_PKT_RCV_DROP_JQ` to jq expressions that return
one numeric value per snapshot. The script fails closed if it cannot obtain at
least two samples of either metric.

## Verdict

For each arm, `receiver-stats/` holds raw `/stats` snapshots, `telemetry/` holds
copied sender snapshots, `resources/sender.csv` records CPU/RSS samples, and
`summary.json` holds the reduced values. CPU and RSS are explicitly recorded
for release notes only and never gate the result.

`canary-result.json` passes only when **both** arms have zero unrecovered links
and zero stream drops, the survivor's mean receiver goodput is at least
`0.95 ×` the 3.3.0 arm, and the survivor's `pktRcvDrop` delta is not worse.
For a telemetry producer without the additive `health` field (such as 3.3.0),
the link check falls back to requiring exactly `CANARY_EXPECTED_LINKS` live
connections in the final snapshot; set that value to the number of accepted IP
list rows, not the number of physical modems hoped to be present.

Copy the run directory and its result into the release evidence before deciding
the release. If the rig is unavailable in the release window, do not run this on
`enhanced` as the default, and make the flip the first item of the next release.
