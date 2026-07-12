# SRTLA Sender (Rust)

[![CI](https://github.com/CERALIVE/srtla-send-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/CERALIVE/srtla-send-rs/actions/workflows/ci.yml)
[![Release](https://github.com/CERALIVE/srtla-send-rs/actions/workflows/release.yml/badge.svg)](https://github.com/CERALIVE/srtla-send-rs/actions/workflows/release.yml)

A Rust implementation of the SRTLA bonding sender. SRTLA is a SRT transport proxy with link aggregation for connection bonding that can transport [SRT](https://github.com/Haivision/srt/) traffic over multiple network links for capacity aggregation and redundancy. Traffic is balanced dynamically, depending on the network conditions. The intended application is bonding mobile modems for live streaming.

This application is experimental. Be prepared to troubleshoot it and experiment with various settings for your needs.

## Credits & Acknowledgments

This Rust implementation builds upon several open source projects and ideas:

- **[irlserver/srtla_send](https://github.com/irlserver/srtla_send)** - This project is a fork of irlserver/srtla_send (fork point `80cd0c4`). Thank you to the irlserver team for the original implementation.
- **[Moblin](https://github.com/eerimoq/moblin)** - Inspired by ideas and algorithms
- **[Original SRTLA](https://github.com/BELABOX/srtla)** - The foundational SRTLA protocol and reference implementation by Belabox

## Features

### Core SRTLA Functionality

- Multi-uplink bonding using a list of local source IPs
- Registration flow (REG1/REG2/REG3) with ID propagation
- SRT ACK and NAK handling (with correct NAK attribution to sending uplink)
- Dynamic path selection with automatic load distribution across all connections
- Keepalives with RTT measurement and time-based window recovery
- Live IP list reload on Unix via SIGHUP
- Runtime configuration via stdin or Unix socket (no restart required)

### Scheduling Modes

The sender supports four mutually exclusive scheduling modes:

#### Enhanced Mode (Default)

- **Exponential NAK Decay**: Smooth recovery from packet loss over ~8 seconds
- **NAK Burst Detection**: Extra penalties for connections experiencing severe packet loss (≥5 NAKs)
- **RTT-Aware Selection**: Small bonus (3% max) for lower-latency connections
- **Quality Scoring**: Automatic preference for higher-quality connections
- **Score Hysteresis**: 10% threshold prevents noise-driven flip-flopping while maintaining natural load distribution

#### Classic Mode

- Exact match to original `srtla_send.c` implementation
- Pure capacity-based selection without quality awareness
- Enable via `--mode classic`

#### RTT-Threshold Mode

- **Reduces Packet Reordering**: Groups links by RTT and strongly prefers low-RTT ("fast") links
- **Threshold-Based Selection**: Links within `min_rtt + delta` are considered "fast"
- **Quality-Aware Within Fast Links**: Applies NAK penalties when choosing among fast links
- **Automatic Fallback**: Uses slow links only when fast links are saturated
- **Enable via**: `--mode rtt-threshold`
- **Configure delta**: `--rtt-delta-ms N` (default 30ms) or runtime `rtt-delta N`
- **Use Case**: Heterogeneous networks where some links have significantly higher latency (e.g., satellite + cellular)

#### EDPF Mode

Earliest Delivery Path First. Instead of scoring links by capacity or RTT group, EDPF predicts when a packet would actually *arrive* over each link and picks the lowest. Selection runs through a three-stage pipeline:

- **BLEST head-of-line-blocking guard**: a static one-way-delay (OWD) filter (50ms threshold, no penalty term) drops links whose OWD would stall the in-order SRT byte stream behind a slower link.
- **IoDS in-order-delivery constraint**: bounds the candidate set to links that keep delivery monotonic. When the admitted set is empty it resets, so no link is permanently starved.
- **EDPF argmin**: among admitted links, selects the lowest predicted arrival `(in_flight_bytes + packet) / effective_capacity + owd`.

The scheduler state (BLEST + IoDS) is owned per send-loop (no thread-local), so selection is deterministic and allocation-free on the hot path.

- **Enable via**: `--mode edpf`
- **Tradeoffs**: minimizes end-to-end reordering and latency on heterogeneous links by modeling delivery time directly, at the cost of more per-packet computation than capacity-only Classic mode. Quality scoring and exploration do not apply.
- **Use Case**: Bonding links with differing bandwidth *and* latency where keeping the SRT stream in order with minimal added delay matters more than raw capacity packing.

### Optional Smart Exploration (Enhanced Mode Only)

- **Context-Aware Discovery**: Tests alternative connections when current best is degrading and alternatives have recovered
- **Periodic Fallback**: Every 30 seconds for 300ms as a safety net
- **Smart Switching**: Tries second-best connections instead of always sticking to current best
- **Enable via**: `--exploration` flag or runtime command `explore on`
- **Use Case**: More aggressive connection testing in unstable network conditions

## Assumptions and Prerequisites

This tool assumes that data is streamed from a SRT _sender_ in _caller_ mode to a SRT _receiver_ in _listener_ mode. To get any benefit over using SRT directly, the _sender_ should have 2 or more network links to the SRT listener (in the typical application, these would be internet-connected 4G modems). The sender needs to have [source routing](https://tldp.org/HOWTO/Adv-Routing-HOWTO/lartc.rpdb.simple.html) configured, as srtla uses `bind()` to map UDP sockets to a given connection.

## Requirements

- **Rust nightly toolchain** and Cargo
- Unix (Linux/macOS) or Windows
  - Note: SIGHUP-based IP reload is Unix-only; Windows runs without that arm

**Important:** This project requires Rust nightly due to advanced rustfmt configuration options used in the codebase.

## Build

```bash
cd srtla_send
rustup install nightly
rustup default nightly  # Set nightly as default for this project
cargo build --release
# binary at target/release/srtla_send
```

Alternatively, you can use nightly for individual commands:

```bash
cargo +nightly build --release
cargo +nightly fmt
cargo +nightly test
```

## Testing

The project includes comprehensive test suites covering unit tests, integration tests, and end-to-end tests.

### Run Tests Locally

```bash
# Run all tests (requires nightly)
cargo test

# Run with verbose output
cargo test --verbose

# Run specific test
cargo test test_connection_score

# Check formatting (requires nightly)
cargo fmt --all -- --check
```

### CI/CD

GitHub Actions runs on the **pinned nightly** from `rust-toolchain.toml` on every push
and pull request (`.github/workflows/ci.yml`):

- The gate: `fmt`, `clippy -D warnings`, the full test fan-out, and `cargo audit`
- Cross-builds for `aarch64-unknown-linux-gnu` (device) and `x86_64-unknown-linux-gnu`,
  each packaged into a `.deb`
- Cross-platform/cross-channel coverage (Linux/Windows/macOS, stable/beta)
- A `v*` release runs the full Rust gate plus the parallel loom and miri lanes before
  either architecture can be packaged or attached to the GitHub release
- `python3 scripts/release_workflow_contract_test.py` checks both publication graphs and
  simulates a failed gate to verify every publish job is skipped
- `bash scripts/release_version_contract_test.sh` proves `v3.2.0` selects 3.2.0 package
  metadata/artifact names and rejects a tag that differs from `Cargo.toml`

### Debian packaging

`ci/build-deb.sh` is the single source of truth for the `.deb`. It installs the binary at
`/usr/bin/srtla_send`, names the artifact `srtla-send-rs_<ver>_<arch>.deb` (Architecture
`arm64`/`amd64`), and declares `Conflicts: srtla (<< <cutover>)` because the `srtla`
package still ships the C `srtla_send`. Pushing a `v*` tag runs
`.github/workflows/release.yml`, which rebuilds both architectures and attaches the
`.deb`s to the GitHub release. The current source package version is `3.2.0`, producing
`srtla-send-rs_3.2.0_arm64.deb` and `srtla-send-rs_3.2.0_amd64.deb`; a tag build is
accepted only when the tag is `v3.2.0`. See `AGENTS.md` → CI / PACKAGING for the full
contract.

### TypeScript binding package

The `bindings/typescript/` helper publishes to the **public npm registry** as
`@ceralive/srtla-send` (`@ceralive` scope) via `.github/workflows/publish-bindings.yml`,
using npm **OIDC trusted publishing** (no `NPM_TOKEN`) — the same flow as
`@ceralive/cerastream`. It is a **separate** release track from the Rust `.deb`s:
pushing a `bindings-vYYYY.M.P` tag runs the typecheck + test gate, builds `dist/`, and
publishes the package. The gate also runs `bun run lint`; a separate publish job can
publish only after the validated `dist/` artifact is available. The published version is the committed
`bindings/typescript/package.json` `version` (CalVer, matching `@ceralive/cerastream`;
the workflow refuses to publish if the tag's version doesn't match it). To cut a
binding release: bump `package.json` `version`, commit, then
`git tag bindings-vYYYY.M.P && git push --tags`. See `AGENTS.md` → CI / PACKAGING.

### Test hardening (Tasks 7-8)

The telemetry layer has hardened integration tests:

- **`tests/telemetry_edge_cases.rs`** (9 tests): zero connections, zero-traffic active
  links, very-high RTT, `schema_version` pinned as a number, `bitrate_bps` x8 invariant
  across a range of wire byte rates.
- **`tests/telemetry_fixture_parity.rs`** (3 tests): Rust golden fixture vs TS-binding
  golden fixture asserted byte-identical, confirming the two consumers stay in sync.
- **`bindings/typescript/tests/telemetry-reader.test.ts`** (24 tests, 52 total): valid
  ADR-001 shape, `bitrate_bps` x8 invariant, malformed input returns `null` (non-JSON,
  truncated, empty, non-object, absent file, missing required fields, wrong types,
  out-of-domain numerics, schema version mismatch).

The binding's `tsconfig.json` was updated to include `tests/**/*` so `bun tsc --noEmit`
typechecks test files. `rootDir: "src"` moved to `tsconfig.build.json` only, keeping
the published `dist/` free of compiled test output.

## Usage

```bash
srtla_send [OPTIONS] SRT_LISTEN_PORT SRTLA_HOST SRTLA_PORT BIND_IPS_FILE
```

### Required Arguments

- `SRT_LISTEN_PORT`: UDP port on which to receive SRT packets locally
- `SRTLA_HOST`: hostname or IP of the SRTLA receiver (e.g., srtla_rec)
- `SRTLA_PORT`: UDP port of the SRTLA receiver
- `BIND_IPS_FILE`: path to a file with newline-separated local source IPs (uplinks)

### Options

- `--verbose`: Enable verbose (debug-level) logging
- `--dry-run`: Validate the IP list and resolve the receiver, print them, then exit without binding any socket (non-zero exit if the IP list is unusable)
- `--mode <MODE>`: Scheduling mode: `classic`, `enhanced` (default), `rtt-threshold`, `edpf`
- `--no-quality`: Disable quality scoring (enhanced/rtt-threshold only)
- `--exploration`: Enable connection exploration (enhanced only)
- `--rtt-delta-ms <N>`: RTT delta threshold in ms (default: 30, rtt-threshold only)
- `--control-socket <PATH>`: Unix domain socket path for remote control (e.g., `/tmp/srtla.sock`)
- `--stats-file <PATH>`: Write per-uplink telemetry JSON to `<PATH>` (opt-in; see [Telemetry](#telemetry))
- `--stats-file-interval <MS>`: Telemetry write cadence in milliseconds (default: 1000)
- `--earned-ack-window`: `[EXPERIMENTAL]` gate broadcast-ACK window growth to the earning link, with rate-limited probe growth for the rest. Default OFF (see [Experimental scheduler-hardening flags](#experimental-scheduler-hardening-flags))
- `--stall-deselect`: `[EXPERIMENTAL]` deselect a stalled link (high in-flight with no earned ACK/RTT sample) so healthy links carry traffic, re-probing so a recovered link re-enters. Default OFF (see [Experimental scheduler-hardening flags](#experimental-scheduler-hardening-flags))
- `--stall-min-in-flight <N>`: `[EXPERIMENTAL]` in-flight threshold that marks a link stall-eligible for `--stall-deselect` (default: 32)
- `--stall-ack-stale-ms <MS>`: `[EXPERIMENTAL]` earned-ACK/RTT staleness window in ms for `--stall-deselect` (default: 3000)
- `--stall-reprobe-ms <MS>`: `[EXPERIMENTAL]` re-probe interval in ms for `--stall-deselect` (default: 1000)
- `-v, --version`: Print version and exit

### Configuration check

Validate the receiver address and IP list without starting the stream or binding any socket:

```bash
./target/release/srtla_send 6000 rec.example.com 5000 ./uplinks.txt --dry-run
```

This prints the resolved receiver address(es) and source uplink IPs and exits `0`. If the IP list is missing, empty, or has no valid IPs, it prints a specific error and exits non-zero.

## Example Usage

Let's assume that the receiver has IP address 10.0.0.1 and the sender has 2 (unreliable) modems with IP addresses 192.168.0.2 and 192.168.1.2 respectively, which can reach the receiver. We'll set up the srtla sender to forward SRT traffic from port 6000 to the receiver's srtla service on port 5000.

### Sender Setup

```bash
echo 192.168.0.2 > /tmp/srtla_ips
echo 192.168.1.2 >> /tmp/srtla_ips
./target/release/srtla_send 6000 10.0.0.1 5000 /tmp/srtla_ips
```

With `srtla_send` running on the sender, SRT-enabled applications should stream to port `6000` on the sender and this data will be forwarded through srtla to the receiver.

### Additional Examples

**With logging and Unix socket control:**

```bash
RUST_LOG=info ./target/release/srtla_send --control-socket /tmp/srtla.sock 6000 rec.example.com 5000 ./uplinks.txt
```

**With classic mode:**

```bash
./target/release/srtla_send --mode classic 6000 rec.example.com 5000 ./uplinks.txt
```

**With RTT-threshold mode:**

```bash
./target/release/srtla_send --mode rtt-threshold --rtt-delta-ms 50 6000 rec.example.com 5000 ./uplinks.txt
```

**With quality scoring disabled:**

```bash
./target/release/srtla_send --no-quality 6000 rec.example.com 5000 ./uplinks.txt
```

Sample `uplinks.txt`:

```text
192.0.2.10
198.51.100.23
203.0.113.5
```

## Logging

This tool uses `tracing` with `EnvFilter`.

- Control verbosity with `RUST_LOG` (e.g., `RUST_LOG=info`, `RUST_LOG=debug`).
- Example:

```bash
RUST_LOG=info,hyper=off ./target/release/srtla_send 6000 host 5000 ./uplinks.txt
```

## Runtime Configuration

The sender supports dynamic runtime configuration changes through two methods:

### Method 1: Standard Input (stdin)

Type commands directly into the running process and press Enter.

### Method 2: Unix Domain Socket (Unix only)

Use the `--control-socket` option to enable remote control via Unix socket:

```bash
# Start with Unix socket control
./target/release/srtla_send --control-socket /tmp/srtla.sock 6000 10.0.0.1 5000 /tmp/srtla_ips

# Send commands remotely
echo 'mode classic' | socat - UNIX-CONNECT:/tmp/srtla.sock
echo 'status' | socat - UNIX-CONNECT:/tmp/srtla.sock
```

### Available Commands

- `mode classic` - Switch to classic mode
- `mode enhanced` - Switch to enhanced mode (default)
- `mode rtt-threshold` - Switch to RTT-threshold mode
- `mode edpf` - Switch to EDPF (Earliest Delivery Path First) mode
- `quality on|off` - Enable/disable quality scoring
- `explore on|off` - Enable/disable connection exploration
- `rtt-delta <ms>` - Set RTT delta threshold in milliseconds
- `status` - Display current configuration

### Connection Selection Algorithm Details

**Classic Mode**: Matches the original srtla_send logic without any enhancements.

**Enhanced Mode** (default): Quality-based scoring that punishes connections with recent NAKs. More recent NAKs = more punishment. Additional 30% penalty (0.7x multiplier) for NAK bursts (≥5 NAKs in short time). Optional connection exploration for testing alternative connections.

**RTT-Threshold Mode**: Groups links into "fast" and "slow" based on RTT measurements. Links within `min_rtt + delta` (default 30ms) are "fast" and strongly preferred. When quality scoring is also enabled, NAK penalties are applied within the fast link group. Falls back to slow links only when all fast links are saturated. Useful for reducing packet reordering in networks with heterogeneous latencies.

**EDPF Mode**: Earliest Delivery Path First. Runs a BLEST → IoDS → EDPF pipeline: a static-OWD head-of-line-blocking guard (50ms) excludes links that would stall the in-order stream, an in-order-delivery constraint bounds the candidate set (resetting when empty so no link starves), and the link with the lowest predicted arrival time `(in_flight_bytes + packet) / effective_capacity + owd` is selected. Scheduler state is owned per send-loop (no thread-local). Quality scoring and exploration do not apply.

## Experimental Scheduler-Hardening Flags

Two flags, gated behind their own CLI switches, harden the default `enhanced` mode against a specific satellite/LAN failure signature (a link that keeps a high scheduling weight while it silently degrades). Both are **default OFF** and mode-agnostic (they apply on top of whichever `--mode` is active). Neither has been validated against real bond hardware yet; treat every behavior claim below as a hypothesis pending that validation.

### `--earned-ack-window`

Without the flag, every broadcast SRTLA ACK grows ALL connected links' congestion window by one step, including links that are not actually carrying traffic. That growth is a deliberate "probing" mechanism (it keeps under-selected healthy links off the floor so the scheduler can re-pick them), not a bug, but it can let an unearned window climb on a link that has stopped delivering.

With the flag on, only the link that actually earned the ACK (the one whose sent sequence was acknowledged) gets the full window step. Every other connected link still grows, but at most once per `PROBE_GROWTH_INTERVAL_MS` (1000ms) — the same "probing" role, just rate-limited instead of unconditional.

### `--stall-deselect`

The 15s `CONN_TIMEOUT` liveness check only reads inbound bytes (including keepalive echoes), so a link that keeps echoing keepalives while it silently stops carrying data still reads "connected" for a long time. `--stall-deselect` adds a selection-time penalty for that case: a link with a high in-flight packet count (`--stall-min-in-flight`, default 32) and no earned ACK/RTT sample within `--stall-ack-stale-ms` (default 3000ms) is excluded from selection for one tick, letting healthy links carry the traffic instead. A link is re-probed every `--stall-reprobe-ms` (default 1000ms) so a recovered link re-enters selection. This is a selection-time penalty only — it never re-registers, resets, or touches `CONN_TIMEOUT`/housekeeping. If every connected link is stalled, selection falls back to the normal (non-deselecting) path so a link is always returned.

### Hardware-validation gate

Both flags ship with unit and golden-trace tests proving flag-off behavior is byte-identical to the pre-flag code path, but neither has been exercised against a real bonded link (e.g. Starlink + cellular) outside this repo's test harness. Do not turn either flag on in production, and do not cite either flag as a proven improvement, until that hardware validation has run. See `docs/notes/sendmmsg-deferred.md`-style deferred-item tracking conventions for how this repo records unrun hardware gates, and the [workspace diagnosis](https://github.com/CERALIVE/ceralive/blob/master/docs/notes/srtla-starlink-lan-diagnosis.md) for the mechanism analysis both flags address.

## IP List Reload (Unix only)

Send SIGHUP to trigger an IP list reload without restarting:

```bash
kill -HUP <pid_of_srtla_send>
```

Surviving uplinks keep streaming across the reload (no re-handshake, no
disconnect); newly listed IPs join and dropped IPs are torn down. The connection
pool is rebuilt in ips-file order, so each uplink's telemetry `conn_id` follows
the file.

A reload that would resolve to **zero valid source IPs** — a missing/unreadable,
empty, or all-garbage file — is **refused**: the sender logs a specific reason
(`ips file not found/unreadable`, `ips file is empty`, `invalid IP on line N`,
or `no valid source IPs … keeping existing connections`) and keeps streaming on
the existing links rather than tearing the stream down. A file that mixes valid
and invalid lines still applies, skipping the bad lines with a warning.

On Windows this arm is disabled; restart the process after editing the IP list.

## Telemetry (`--stats-file`, ADR-001)

`srtla_send` can publish a per-uplink JSON snapshot to a file for consumers such as the
CeraUI backend (`@ceralive/srtla` telemetry reader). It is **opt-in**: without
`--stats-file` no file is ever created.

```bash
./target/release/srtla_send 6000 10.0.0.1 5000 /tmp/srtla_ips \
  --stats-file /tmp/srtla-send-stats-6000.json --stats-file-interval 1000
```

The document is rewritten atomically (`<path>.tmp` → `fsync` → `rename(2)`) every
`--stats-file-interval` ms (default 1000), so a concurrent reader never observes a torn
write. It is a single newline-free object:

```json
{"schema_version":1,"last_updated_ms":1749556546000,"connections":[{"conn_id":"0","rtt_ms":42,"nak_count":3,"weight_percent":85,"window":8192,"in_flight":100,"bitrate_bps":2500000}]}
```

- `conn_id` — the uplink's index in `BIND_IPS_FILE` order, as a string.
- `rtt_ms` — Kalman-smoothed RTT.
- `weight_percent` — the link's normalized share of selection weight (0–100).
- `bitrate_bps` — send rate in **bits per second** (wire bytes/s × 8).
- `window` / `in_flight` — congestion-window and in-flight packet counts.

With no active links the file still exists with `"connections": []` ("running but idle",
distinct from "absent"). The live file is removed on clean shutdown (SIGTERM/SIGINT).

## Startup Without an IP List (Unix)

A missing, empty, or all-invalid `BIND_IPS_FILE` at startup is not fatal. The
sender binds its local SRT listener, starts with an empty uplink pool, and waits
for a `SIGHUP` reload — convenient when a supervisor (e.g. CeraUI) writes the IP
file and signals the process only once network interfaces appear.

## Clean Shutdown (Unix)

`SIGTERM` and `SIGINT` trigger a graceful shutdown: the process exits `0`
promptly, and the `--stats-file` telemetry file (with its `.tmp` sibling) is
removed so no stale snapshot outlives the process.

## How It Works

The core idea is that srtla keeps track of the number of packets in flight (sent but unacknowledged) for each link, together with a dynamic window size that tracks the capacity of each link - similarly to TCP congestion control. These are used together to balance the traffic through each link proportionally to its capacity. However, note that no congestion control is applied.

### srtla v2 Improvements

The main improvement in srtla v2 is that it supports multiple _srtla senders_ connecting to a single _srtla receiver_ by establishing _connection groups_. To support this feature, a 2-phase connection registration process is used:

Normal registration:

- Sender (conn 0): `SRTLA_REG1(sender_id = SRTLA_ID_LEN bytes sender-generated random id)`
- Receiver: `SRTLA_REG2(full_id = sender_id with the last SRTLA_ID_LEN/2 bytes replaced with receiver-generated values)`
- Sender (conn 0): `SRTLA_REG2(full_id)`
- Receiver: `SRTLA_REG3`
- [...]
- Sender (conn n): `SRTLA_REG2(full_id)`
- Receiver: `SRTLA_REG3`

### Implementation Details

- For each IP in `BIND_IPS_FILE`, the sender binds a UDP socket and connects to `SRTLA_HOST:SRTLA_PORT`.
- Incoming SRT UDP packets are read on `SRT_LISTEN_PORT` and forwarded over the currently selected uplink based on the score `window / (in_flight + 1)`.
- ACKs are applied to all uplinks to reduce in-flight counts; NAKs are attributed to the uplink that originally sent the sequence (tracked), falling back to the receiver uplink if unknown.
- **Burst NAK Detection**: The system tracks NAK bursts (multiple NAKs within 1 second) per connection. When quality scoring is enabled, connections with recent NAK bursts (≥5 NAKs in burst, within last 3 seconds) receive an additional 0.7x multiplier (30% reduction) to their quality score, helping avoid connections experiencing packet loss issues.
- Keepalives are sent when idle, and periodically for RTT measurement; the RTT is smoothed via a Kalman filter. The Kalman output is clamped to ≥0 before use. Keepalive RTT samples of exactly 0 are rejected (parity with the ACK path), so a reply that arrives within the same scheduler tick doesn't bias the filter downward. Window recovery is conservative and time-based when there are no recent NAKs.
- Small control packets (keepalive, REG1/REG2) are zero-padded to a 32-byte minimum on the wire (`MIN_CONTROL_PKT_LEN`), matching the C `pad_sendto` behavior, so cellular/carrier NAT keepalive thresholds don't silently drop tiny control frames. DATA packets are never padded.
- Each uplink's reader task is monitored on every housekeeping tick. If a reader exits unexpectedly (e.g. due to a socket error), it is restarted within one tick rather than waiting for the 15 s liveness timeout.
- The all-uplinks-failed global timeout measures time elapsed since the failure, not process uptime. A transient all-down blip on a long-running session no longer triggers an immediate fatal exit.

## Notes

- Ensure your system has the specified local source IPs configured and routable.
- The local SRT producer (e.g., `srt-live-transmit`) should send to `udp://127.0.0.1:SRT_LISTEN_PORT`.
- The SRTLA receiver must understand the SRTLA protocol (REG1/2/3, ACK, NAK, KEEPALIVE).
- The sender **should** implement congestion control using adaptive bitrate based on the SRT `SRTO_SNDDATA` size or on the measured `RTT`. Due to reordering, these values may be slightly higher during uncongested operation over srtla compared to direct SRT operation over one of the same network links.

## License

This Rust implementation is licensed under the MIT License. See the [LICENSE](./LICENSE) file for full details.

## Expected Behavior

### Load Distribution

With properly configured connections, you should observe:

**All connections active**: Traffic should appear on all uplinks (e.g., if you have 4 uplinks, all 4 should show active bitrate)

**Proportional distribution**:

- With equal connections: roughly equal traffic distribution (e.g., 25% each with 4 uplinks)
- With varying quality (enhanced mode): better connections get more traffic, degraded connections get less
- With varying capacity: connections with larger windows get proportionally more traffic

**Dynamic adaptation (enhanced mode)**:

- Connections experiencing NAKs automatically receive less traffic
- Connections recover to full capacity within ~8 seconds after issues resolve
- System continuously rebalances based on current conditions

### Monitoring

**Status logs** (every 30 seconds) show:

- Total bitrate across all connections
- Individual connection status (active/timed out)
- Window sizes and in-flight packet counts
- RTT measurements and connection quality metrics
- Current mode and configuration

**Debug logs** (when `RUST_LOG=debug`) show:

- Per-packet connection selection decisions
- Quality multiplier calculations
- NAK burst detections and recovery
- Exploration attempts
- Hysteresis decisions

### Troubleshooting

**If only some connections are used**:

1. Check for NAKs in logs - degraded connections naturally get less traffic in enhanced mode
2. Try classic mode: `mode classic` - disables quality awareness for pure capacity-based distribution
3. Temporarily disable quality scoring: `quality off`
4. Verify all uplinks can reach the receiver (check for timeout messages)
5. Check RTT differences - high-RTT connections get slightly less traffic in enhanced mode (3% max difference)

**If throughput is lower than expected**:

1. Verify SRT is not limiting the bitrate (check encoder settings)
2. Check for high packet loss (NAKs) on connections - indicates network issues
3. Ensure sender has sufficient CPU and network capacity
4. Monitor SRT `SRTO_SNDDATA` buffer - if full, increase bitrate or improve connections
5. Check connection windows in status logs - low windows indicate capacity limits

**If connections are flip-flopping**:

1. This should be minimal with 10% hysteresis in enhanced mode
2. Check if scores are truly identical (look for hysteresis messages in debug logs)
3. Verify connections have stable quality (no intermittent NAKs)
4. Consider using classic mode for perfectly equal connections

## Performance Tuning

### Constants (Advanced)

If needed, these can be adjusted in `src/sender/selection/`:

**Enhanced Mode (`enhanced.rs`):**

- `SWITCH_THRESHOLD`: 1.10 (10% hysteresis) - increase for more stability, decrease for faster response

**Quality Scoring (`quality.rs`):**

- `STARTUP_GRACE_PERIOD_MS`: 30000ms (30 seconds) - grace period before quality penalties apply
- `PERFECT_CONNECTION_BONUS`: 1.1 (10% bonus) - bonus for connections with no NAKs
- `STARTUP_NAK_PENALTY`: 0.98 (2% penalty) - light penalty during grace period
- `HALF_LIFE_MS`: 2000ms (2 seconds) - NAK penalty decay speed
- `MAX_PENALTY`: 0.5 (50% penalty) - maximum initial penalty after NAK
- `NAK_BURST_THRESHOLD`: 5 NAKs - minimum burst size to trigger extra penalty
- `NAK_BURST_MAX_AGE_MS`: 3000ms (3 seconds) - max age for burst penalty
- `NAK_BURST_PENALTY`: 0.7 (30% reduction) - multiplier applied for bursts
- `RTT_BONUS_THRESHOLD_MS`: 200ms - RTT threshold for bonus calculation
- `MIN_RTT_MS`: 50ms - minimum RTT for calculation (prevents division issues)
- `MAX_RTT_BONUS`: 1.03 (3% max bonus) - maximum RTT bonus multiplier

**Exploration (`enhanced.rs`):**

- Exploration period: `should_explore_now()` function, currently 30s - adjust exploration interval

### Runtime Optimization

For maximum throughput:

- Use enhanced mode (default) to automatically avoid degraded connections
- Ensure adequate SRT buffer size (`SRTO_SNDDATA`)
- Monitor for connection timeouts - these interrupt traffic flow
- Use `RUST_LOG=info` for minimal logging overhead (avoid debug in production)

For maximum stability:

- Use classic mode (`--mode classic`) for predictable, simple behavior
- Disable exploration (`explore off`) if not needed
- Increase hysteresis threshold if experiencing unnecessary switching
