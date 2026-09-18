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

Upstream history is merged through
`df0b3938791ff24eced4aed8b29e3d49d0efb639`. The seven commits after the previous sync
were compatibility-reviewed: existing fork-native protocol/recovery/registration fixes
remain authoritative, three narrow diagnostic improvements were adapted, and upstream's
default-on receiver re-home plus automatic `4.0.1` bump were not imported. See
[`docs/notes/upstream-sync-2026-09-evaluation.md`](docs/notes/upstream-sync-2026-09-evaluation.md).

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

All five modes now share health/deadline admission, quality × rejoin ramp ×
Healthy-only priority × soft rate-cap weighting, and paced duplicate probes.
Modes choose the ranking formula, not which health signals exist. This is the
deliberate pre-campaign Todo-32 behavior change; it is not a hardware-performance
acceptance. Historical experiment reports below describe their then-current code.

The shared `SchedulerFeatures` defaults are all ON; `--no-quality` gates QUALITY
in every mode. Internal bits are never serialized on the public control/telemetry
surfaces. Test-build `SRTLA_ADAPTIVE_FEATURES` and `SRTLA_ADAPTIVE_TUNING` keep their
names, with `quality` added to the feature token list.

**Todo-32 verification boundary:** clean pre-lift `3772598` reproduces the two
privileged adaptive failures: marginal demotion42/61 snapshots versus51/61 and40/61
after the lift, and twins losing sustained Healthy after full-rate restoration in
both versions. The owner accepted the refactor with these pre-existing issues
documented; neither assertion was relaxed. Build/Clippy/formatting and library
gates pass; the full feature suites remain red at those tests. The independent
privileged EDPF test passes. See AGENTS.md, “Todo32 gate disposition”; this is not
a throughput acceptance or evidence that the existing adaptive issues are fixed.

With all normal candidates held, the unchanged sole-carrier election runs before
the connected-only base-score fallback. Down-but-connected links remain usable in
that final escape; no connected pool is stranded. Held links publish zero weight,
and a zero-total snapshot never invents equal shares. Probe pacing and target state
are owned by the send loop's `SchedulerShared`, independent of mode.

**Unified ACK policy:** Classic, Enhanced, RTT-threshold and EDPF now use the same
arrival-link/generation-fenced delivery accounting as Adaptive. They no longer
scan other links for an SRTLA ACK or grow every link's window on a broadcast ACK.
Only the last unambiguous original in an ACK frame supplies RTT; cumulative SRT ACKs
still prune but never measure RTT. Probe ACKs supply health evidence without original
rate/window credit, and probe-only NAKs are excluded. Classic now receives the existing
time-based window recovery too.

**In-flight-aware NAK penalties:** every mode protects a recently accepted DATA packet
until `clamp(minimum measured RTT / 2, 5, 500)` ms after kernel acceptance, not queueing.
Without a real RTT sample or retained acceptance timestamp, the ordinary penalty applies.
At most three consecutive premature reports per link are suppressed; the fourth takes
the normal window/loss penalty and resets the count. A mature penalty or accepted
ACK/keepalive RTT sample also resets that streak. Suppression retains the in-flight
packet and does not feed congestion or loss cohorts. The encoder still receives every
NAK frame unchanged. This is automatic, with no new tuning flag; per-link status logs
show lifetime `premature_naks`, while telemetry JSON is unchanged. Unit/property and
loopback coverage do not establish a real-bond throughput improvement.

The M2 sender-mechanism campaign kept the production cap at three: B1/C settling
was 0/3 in both ON and test-bypass arms, while neither A nor G crossed the frozen
>5% goodput-regression rule with non-overlapping confidence intervals. Its bypass
exists only in `test-internals` builds and is not a CLI or deployable tuning option.
The retained capture also validated the retransmission bit for this caller build;
NAK-off blindness remains present on G. See
[`docs/evidence/bpc/m2-sender/`](docs/evidence/bpc/m2-sender/).

#### Enhanced Mode (Default)

**Cooldown candidate (2026-09-16):** See [evaluation](docs/notes/enhanced-cooldown-evaluation-2026-09.md) — unaccepted, reverted.

 - **Exponential NAK Decay**: Smooth recovery from packet loss over ~8 seconds
- **NAK Burst Detection**: Extra penalties for connections experiencing severe packet loss (≥5 NAKs)
- **RTT-Aware Selection**: Small bonus (3% max) for lower-latency connections
- **Quality Scoring**: Automatic preference for higher-quality connections
- **Score Hysteresis**: 10% threshold prevents noise-driven flip-flopping while maintaining natural load distribution

#### Classic Mode

- Capacity argmax after shared admission and weighting; no cooldown or exploration
- No longer an exact C-sender behavior mode; quality/health/preferences apply
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

- **BLEST head-of-line-blocking guard**: a static one-way-delay (OWD) filter (50ms threshold, no penalty term) drops links whose OWD would stall the in-order SRT byte stream behind a slower link. A dropped link is re-admitted while its predicted arrival is earlier than every admitted link's — a saturated fast link cannot permanently starve a high-latency uplink, because a packet that lands first blocks nothing.
- **IoDS in-order-delivery constraint**: bounds the candidate set to links that keep delivery monotonic. When the admitted set is empty it resets, so no link is permanently starved.
- **EDPF argmin**: among admitted links, selects the lowest predicted arrival `(in_flight_bytes + packet) / effective_capacity + owd`. A link with no measured send rate yet (freshly registered, or idle past the 2s bitrate window) uses a flat 1 Mbps bootstrap capacity, so it stays comparable instead of dropping out of the pipeline — without it the scheduler at startup selects nothing, sends nothing, and therefore never measures anything.

The scheduler state (BLEST + IoDS) is owned per send-loop (no thread-local), so selection is deterministic and allocation-free on the hot path.

- **Enable via**: `--mode edpf`
- **Tradeoffs**: models delivery time at greater computation cost than Classic. The shared multiplier replaces the old quality-cache input inside effective capacity, preserving its clamp, bootstrap and velocity/BDP terms. Exploration does not apply.
- **Use Case**: Bonding links with differing bandwidth *and* latency where keeping the SRT stream in order with minimal added delay matters more than raw capacity packing.

#### Adaptive Mode [PARTIAL]

**Final Todo 30 status (2026-09-15): scoped completion with documented findings,
not scheduler acceptance.** Nine non-adaptive targets have historical privileged
passes; the literal ten-target all-pass bar remains unmet. I passes with its blocking
three-second pre-restart assertion restored and is closed as a non-issue for this
task. The genuine RPC-to-telemetry priority-snapshot fix (`361d644`) is retained.

G spans **0/61, 4/61, 29/61 and 44/61** demoted snapshots (0–72.13%) across clean
isolated observations; identical current code both passes and fails. This exceeds
the earlier small-sample 0–13% band and needs Wave 6 / Todo 32's **N=5+ statistical
C2 campaign**, not more single-run tuning. Exact `d168aa6` Twins **passes full-rate
health**, while failing its old share floor and stale clear-RPC snapshot. Thus earlier
current-state Twins health failures are unresolved, not proven pre-existing or fixed.
D's documented ignore and Twins' informational preference-share ceiling remain
separate dispositions; G and Twins-health checks are still blocking. No scheduler
change or additional health waiver was made to close this investigation. See the
[final scoped findings](docs/notes/scheduler-evaluation-2026-09.md#final-todo-30-disposition--scoped-completion-with-documented-findings);
older run narratives below remain historical, not the current acceptance verdict.

Adaptive has a [known baseline-topology throughput limitation](docs/notes/scheduler-evaluation-2026-09.md#known-limitation-adaptive-mode-baseline-topology-throughput-instability-scenario-a-discovered-post-todo-28),
discovered by the post-Todo-28 smoke campaign on three equal10Mbit/s links with normal
jitter. The owner retained the `d168aa6` wire-rate baseline and reverted subsequent
experiments that passed unit tests but did not stabilize live delivery. The evaluation
note preserves those mechanisms and reusable test scaffolding for the planned C1/C2
campaigns. Prior bounded-recovery evidence does not establish stable throughput on A.

**Hard-admission correction (2026-09-15), not throughput acceptance:** adaptive no
longer installs a `WireBudget` in `wire_admission::configure`. The separate
`RateCap` BDP ranking multiplier is unchanged; ranking does not consume
`WireRateEstimator::rate_bps`. The estimator implementation and update call are
retained, but without a budget its accepted-byte sample is absent, so it cannot
perform active rate search. A fixed-clock regression proves that a queue-induced
500 kbit/s estimate cannot deny admission or limit the kernel-accepted batch.
Three fresh Scenario-A indices still exhausted the unchanged two-attempt budget
with `settle_timeout`. G and Twins sustained-health checks also fail in both
feature-suite runs; I passes. This narrow correction is therefore **insufficient**
to clear the live gate. No thresholds, scenarios, or campaign criteria were relaxed.

**ACK-frame follow-up (2026-09-15):** adaptive now takes at most one RTT sample
per SRTLA ACK frame, from its final entry in receiver arrival order. Earlier
entries include receiver batching delay. If the final entry is a duplicate probe,
retransmission-ambiguous original, replay, or missing original, the frame supplies
no RTT sample. Every entry still receives its existing delivery/loss/window
accounting; the four legacy modes retain per-sequence RTT behavior. Frame boundaries
survive batched reads. Seven regression/control tests include a staggered ten-packet
frame that formerly sampled 870–60 ms on a 60 ms path and now samples 60 ms only.

This does **not** fix live Scenario A: all six new attempts still fail
`settle_timeout`. Both admission-only and combined candidates briefly reach ~24 Mbps
one-second sink peaks, then fail to sustain the settling target; the combined
candidate's final one-second sink buckets are 7.02–11.07 Mbps. Its truncated sender
log tails cannot establish stable health or live RTT improvement. Separately,
stash-isolated HEAD reproduces both G demotion (30/61) and Twins sustained-health
failures, establishing pre-existing failure signatures, not equal failure rates
or a waiver. Scheduler acceptance and Campaign C1 remain blocked.
The final build/Clippy/library/changed-file-formatting gates pass, but both broader
feature suites still fail G and Twins health; later targets are not reached.

**Round-8 keepalive-detector experiment (2026-09-15): NOT an A/D solution.**
Separate socket-scoped keepalive reply tracking adds a three-interval silence
condition without requiring DATA attempts; the existing DATA-only stall condition
is retained. Real-UDP failing-first tests cover idle silence and a late DATA ACK
resetting the attempt counter. However, D deliberately passes small keepalives
while dropping DATA: ten detector-only, sole-ON release trials yield only **3/10**
timely Stalled/zero-weight observations, versus the nearest default baseline's
**2/10** (historical **4/5** is not a paired control). Complete D passes: **0/10**.
The conditional admission change was not attempted. G fails all three dedicated
trials; release Twins passes three, but broader test-profile gates still fail G
and Twins health. This experiment remains uncommitted and unaccepted; no C1 restart
or hardware improvement is claimed. See the [round-8 findings](docs/notes/adaptive-keepalive-round8.md).

`--mode adaptive` selects the health/deadline-gated capacity pipeline. It ranks
admitted links by the existing queued-load score and cached quality, multiplied by
rejoin ramp, Healthy-only preference, and the delivered-rate controller's soft cap.
Its 15ms switch cooldown and 10% hysteresis cannot retain a link outside admission.
`--stall-deselect` is a **no-op in every mode**: shared admission owns stall handling.

The deadline budget is the last observed negotiated receiver latency, or 500ms when
unknown. A link is held when `sRTT/2 + queue_delay > 0.5 × budget`; release needs
continuous prediction below `0.4 × budget` for τ (clamp(4×sRTT,1000,3000) ms).
With no admitted link, one eligible sole carrier is held for at least two seconds.
A carrier without recent DATA proof rotates after max(2000,τ) ms and stays behind
other candidates until it earns proof; keepalive RTT does not rehabilitate it.
If no eligible link remains, the highest-base-score connected link is the fallback.

Held-out links receive globally paced duplicate DATA, never the elected sole carrier.
Only a due probe forces its primary batch to flush; no copy precedes primary
acceptance, and neither probe selection nor emission changes switch history.

Housekeeping health/rate ticks, registration/recovery resets, lifecycle status and
adaptive control are integrated. Health starts Down and enters Rejoining after
registration. Do not deploy it as a proven improvement: policy and packet-handler
UDP tests are not bonded-hardware evidence.
Run `cargo test --lib adaptive` and `cargo test --test adaptive_cli`.

`HealthSignals.route_health` reuses the existing route classification, telemetry
emits optional `health` and `priority` after `link_id`, and
`SharedStats::pool_control()` exposes a bounded sender-owned
request/reply handle. Its consumer mutates real priority layers before replying.
The JSON-RPC priority method is integrated. Adaptive telemetry weights now come from
the same admission/ranking pass used by packet selection, refreshed after health
ticks before the initial and periodic snapshots, even with no DATA arriving.
Held-out links report zero; the sole carrier retains its ranking factors. The
last-resort connected fallback keeps its existing base-only rank. A zero-total
adaptive snapshot never substitutes legacy equal shares. These are normalized
ranking weights, not measured traffic shares or one-hot cooldown decisions.

Adaptive path RTT uses specific SRTLA ACKs and keepalive echoes, not cumulative SRT
ACKs: the latter name the next expected sequence and can return on another uplink.
Wire-marked retransmissions and repeated outstanding sequences still prove DATA
delivery, but are excluded from adaptive RTT sampling because the ACK cannot identify
which copy it acknowledges (Karn ambiguity). Eligible samples use kernel-acceptance
timestamps even when cumulative pruning already removed congestion-log entries.
Todo 32 applies this RTT policy to every mode. Pending probe trains no longer erase preceding
successful rounds, and qualified soft recovery starts a fresh normal-loss epoch
instead of blending the previous outage into the first recovered cohort.
Stalled links can also qualify recovery from original DATA: two ten-packet groups
with at least five link-specific ACKs each, using the same deadline and freshness
rules. This is passive observation, not self-probing: the elected sole carrier is
still never sent duplicate probes and remains Stalled without sufficient proof.
Evidence is bounded and scoped to the socket and current Stalled epoch.
These correctness fixes are regression-tested; the privileged D/G/twin integration
scenarios still expose unresolved recovery/selection behaviour and are not green.

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
# Run the default test command (privileged netns targets may self-skip)
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

- The gate: `fmt`, `clippy -D warnings`, bounded Rust tests, and `cargo audit`; privileged
  `netns_*` targets self-skip unless their root/tool dependencies are present
- Cross-builds for `aarch64-unknown-linux-gnu` (device) and `x86_64-unknown-linux-gnu`,
  each packaged into a `.deb`
- Cross-platform/cross-channel coverage (Linux/Windows/macOS, stable/beta)
- A required `bindings` job that runs the TypeScript binding gate (`bun install
  --frozen-lockfile`, lint, typecheck, tests, build) on **Bun 1.4.2** — a red
  binding blocks the PR, so a break no longer waits for a `bindings-v*` tag. The two
  binding contract scripts (`bindings_release_ref_contract_test.sh`,
  `bindings_package_manager_contract_test.sh`) run here too, right after Bun is
  installed: both evaluate JavaScript, and this is the only CI job with a JS runtime
- A `v*` release runs the full Rust gate plus the parallel `loom` contract job
  (a production subscription-concurrency invariant) and Miri lane before either
  architecture can be packaged or attached to the GitHub release. Both `.deb` builds
  execute inside Debian 12 (`debian:bookworm-slim`) and reject a final binary importing
  any GLIBC symbol newer than the device image's `GLIBC_2.36` ceiling
- Every Rust CI/release lane uses `Swatinem/rust-cache@v2` for Cargo's registry,
  git, and bounded dependency-target cache. Keys separate OS, runner architecture,
  toolchain, source/lockfile state, and `.deb` target architecture; Miri keeps
  target artifacts disabled. The toolchain action's implicit cache is disabled so
  there is one explicit cache owner per lane.
- `uv run scripts/release_workflow_contract_test.py` derives publication capability
  structurally from write permissions and secret references, then simulates a failed
  gate to verify every publication-capable job is skipped
- `uv run scripts/rust_cache_contract_test.py` verifies the cache action, key dimensions,
  bounded-target settings, and failure-propagation shape across both Rust workflows
- `bash scripts/release_version_contract_test.sh` proves `v3.3.0` selects 3.3.0 package
  metadata/artifact names and rejects a tag that differs from `Cargo.toml`
- `bash scripts/deb_version_ordering_test.sh` derives the package version from `Cargo.toml`,
  proves a patch bump sorts newer under Debian ordering, and reports whether the known stale
  CalVer artifact `2026.6.1` still outranks the current SemVer stream

### Debian packaging

`ci/build-deb.sh` is the single source of truth for the `.deb`. It installs the binary at
`/usr/bin/srtla_send`, names the artifact `srtla-send-rs_<ver>_<arch>.deb` (Architecture
`arm64`/`amd64`), and declares `Conflicts: srtla (<< <cutover>)` because the `srtla`
package still ships the C `srtla_send`. Pushing a `v*` tag runs
`.github/workflows/release.yml`, which rebuilds both architectures and attaches the
`.deb`s to the GitHub release. The current source package version is `3.3.0`, producing
`srtla-send-rs_3.3.0_arm64.deb` and `srtla-send-rs_3.3.0_amd64.deb`; a tag build is
accepted only when the tag is `v3.3.0`. See `AGENTS.md` → CI / PACKAGING for the full
contract. Release binaries are built against Debian 12 rather than the moving GitHub
runner userspace, keeping their GLIBC requirements compatible with the Bookworm device
image on both architectures.

### TypeScript binding package

The `bindings/typescript/` helper publishes to the **public npm registry** as
`@ceralive/srtla-send` (`@ceralive` scope) via `.github/workflows/publish-bindings.yml`,
using npm **OIDC trusted publishing** (no `NPM_TOKEN`) — the same flow as
`@ceralive/cerastream`. It is a **separate** release track from the Rust `.deb`s:
pushing a `bindings-vYYYY.M.P` tag runs the typecheck + test gate, builds `dist/`, and
publishes the package with the npm CLI pinned to `11.18.0`. The Bun gate runs lint,
typecheck, Bun-native tests, and build — Bun is both the package manager and the test
runtime, so npm appears only in the tarball guard and the publish itself; a
separate publish job needs both validated `dist/` and exact tag/ref/version/SHA
provenance. Manual workflow dispatch is dry-run-only and has no path to the OIDC publish
job. The published version is the committed
`bindings/typescript/package.json` `version` (CalVer, matching `@ceralive/cerastream`;
the workflow refuses to publish if the tag's version doesn't match it). To cut a
binding release: bump `package.json` `version`, commit, then
`git tag bindings-vYYYY.M.P && git push origin bindings-vYYYY.M.P`. See `AGENTS.md` → CI / PACKAGING.

### Test hardening (Tasks 7-8)

The telemetry layer has hardened integration tests:

- **`tests/telemetry_edge_cases.rs`** (9 tests): zero connections, zero-traffic active
  links, very-high RTT, `schema_version` pinned as a number, `bitrate_bps` x8 invariant
  across a range of wire byte rates.
- **`tests/telemetry_fixture_parity.rs`** (3 tests): Rust golden fixture vs TS-binding
  golden fixture asserted byte-identical, confirming the two consumers stay in sync.
- **`bindings/typescript/tests/telemetry-reader.test.ts`** (24 tests, 68 total): valid
  ADR-001 shape, `bitrate_bps` x8 invariant, malformed input returns `null` (non-JSON,
  truncated, empty, non-object, absent file, missing required fields, wrong types,
  out-of-domain numerics, schema version mismatch).
- **`bindings/typescript/src/telemetry/watch.test.ts`** (6 tests): event-driven watcher
  checks for absent, stale-boundary, stop, file-appears, invalid-schema, and payload
  behavior without fixed sleep windows.
- **`bindings/typescript/tests/telemetry-roundtrip.test.ts`** (14 tests): re-serializing a
  parsed snapshot reproduces the producer's bytes exactly, for every producer-ordered
  fixture — so no field the sender emits, `iface` and `link_id` included, is silently
  dropped by the reader. A Zod schema strips undeclared keys *without erroring*, so a
  parse-succeeds assertion cannot catch that; comparing bytes can. Includes a
  falsifiability control that deletes the two identity fields and requires the comparison
  to fail, plus the old-shape half: a pre-identity payload parses, round-trips byte-stably,
  and reports both fields `undefined` with no key materialized.
- **`tests/subscription_loom.rs`** (2 tests): Loom schedule exploration against the
  production manager under `cfg(loom)`, covering concurrent live-or-replay delivery
  and disconnected-subscriber pruning without copying the manager algorithm.
- **`tests/startup_bind_ordering.rs`** (3 tests): the local SRT listener is bound
  before the first uplink connect, before every uplink of a multi-link bond
  (including a failing attempt), and the port is genuinely held once the listener is
  logged. Unprivileged; needs no reachable receiver.

The binding's `tsconfig.json` was updated to include `tests/**/*` so `bun run typecheck`
typechecks test files. `rootDir: "src"` moved to `tsconfig.build.json` only, keeping
the published `dist/` free of compiled test output.

### Privileged network-namespace tests

Most Rust tests need no privileges. The `tests/netns_*.rs` supplements require Linux
network namespaces, passwordless `sudo`/`CAP_NET_ADMIN`, `srtla_rec`,
`srt-live-transmit`, and scenario-specific netem/tcpdump tools; otherwise they self-skip.
The harness tears down the exact PIDs reported for each ephemeral namespace with bounded
TERM-then-KILL polling; it does not assume the tracked `sudo` PID is a process-group leader
and never blocks on an unbounded child wait. Namespace and veth names both include the
PID+atomic-counter uniqueness suffix, so parallel scenarios in one test binary cannot
collide. CI/release test commands remain capped at 300 seconds, and manual privileged runs
use `./scripts/netns_test_gate.sh` (90 seconds per target by default; `netns_bond` gets
120 seconds via `NETNS_BOND_TEST_TIMEOUT_SECONDS`, and `netns_twin` gets
420 because its scenarios wait out the sender's own 15-second liveness timeout and
30-second status-log interval). One separate real-Starlink stall reproduction is
intentionally `#[ignore]` and runs only on hardware.

`tests/netns_adaptive.rs` adds four strict adaptive integration scenarios: the full
D obstruction waveform, G's marginal link over 60 seconds, I's CeraLive receiver
restart, and duplicate-IP twins with in-obstruction priority set/clear. It reuses
the FIFO-controlled real SRT source and host measurement lock (their two existing
unprivileged tests are also included). Assertions use telemetry identities and
per-interface netdev byte deltas, not IP identity or telemetry weights as traffic
shares. The twin scenario includes a second, unmapped legacy-control run.
The target has its own 360-second budget, `NETNS_ADAPTIVE_TEST_TIMEOUT_SECONDS`.
Receiver recovery timing starts at process respawn; the separate downstream-SRT
preflight/UDP-readiness wait is not charged to the two-second kill/respawn budget.
`BondRuntime::receiver_restart_elapsed()` reports that process-only duration.
Set `SRTLA_REC_BIN` to the intended CeraLive receiver build; optionally set
`NETNS_ADAPTIVE_ARTIFACT_DIR` to retain per-run observations, sink buckets and logs.
**Integration is not yet green:** D/G/twin health/rejoin and final-share assertions
remain red; the latest receiver-restart scenario passes. Deadline-settled loss is
regression-tested, but a diagnostic G run and mapped twins still demote on queue
delay. These assertions are blocking, not ignored or
relaxed; the adaptive engine remains experimental, not integration-validated.

A further instrumented G/twin investigation measured valid 60–61 ms RTT floors
with genuinely elevated fast-window minima. Kernel TBF queues filled after the
twin load returned to 12.8 Mbit/s while the survivor alone could carry 8 Mbit/s, then
filled on the recovering link. The rate controller supplies a soft ranking
penalty, not a paced send-rate limit; G also queued before target growth and
while Holding below its link capacity. No estimator filter or relaxed health
threshold was applied. Separate loss-labelled demotions occurred in the new G
run, so earlier queue-only attribution is specific to that earlier measurement.

The subsequent queue-entry persistence gate below passes its deterministic policy
tests but does not close this integration gap. In its first unchanged live run,
I passes; D still misses Healthy recovery and continuing keepalive-log evidence;
G carries 3,727,772 bytes but is Stalled/Degraded in 48/61 samples; twins miss
Healthy recovery and deliver 47.1518% final preferred share (required [55%,70%)).
Both twin priority snapshots pass in this run. No further persistence extension or
admission/sole-carrier change is included, and the adaptive gate remains blocking.

The subsequent recovery-load scope correction separates feasible recovery from a
full-rate step: D holds 6.4 Mbit/s through its 17-second recovery deadline, restoring
22.4 Mbit/s at t=46; twins hold 6.4 Mbit/s through their 9-second deadline, restoring
12.8 Mbit/s at t=38. Neither deadline nor the final health/share requirements was
relaxed. Adaptive has no aggregate admission or source backpressure, so immediate
full-rate restoration remains a documented risk and an ignored diagnostic stress
case, not a passing recovery assertion. The test's logging filter now retains
health/keepalive evidence by suppressing the per-NAK congestion flood.

**The revised tests still do not pass.** D recovers within its deadline but misses
stall detection and later loses Healthy; twins expose a successful-probe freshness
timing gap before full load, plus the recurring first-priority-snapshot failure.
G's chronic demotion also reproduces with zero configured loss on its 1 Mbit/s
link: real qdisc overflow remains, and its supposedly healthy companions demote
too. G's ≤10% criterion remains unchanged; this is not grounds for calling the
failure expected GE loss. Latest clean-code run: I passes; D/G/twins fail.
See [the evaluation findings](docs/notes/scheduler-evaluation-2026-09.md).

The sampled-freshness defect is now corrected: recovery evidence's acquisition
budget additionally accounts for the actual one-second housekeeping observation
interval. Immediate policy evaluation uses zero observation delay. Probe ACK
deadlines, epoch fencing, two successful trains, health thresholds and the rejoin
ramp remain unchanged. A real-scheduler regression and stale/insufficient-evidence
controls pin the distinction. In the latest live twins run, Rejoining occurs at
restore+3.0716s and Healthy at+5.0870s, within9s and without low-load relapse;
later full-rate health/share still fails, so the complete integration gate stays red.

G's12.8Mbit application load fits within the three companions'15Mbit capacity even
after ordinary framing. Startup accepted-prefix captures instead show excessive
initial allocation to the1Mbit link, followed by retransmission-inflated companion
rates above their individual5Mbit caps, before health demotion. No G assertion,
scenario, rate policy or ranking behavior was changed from that evidence alone.

The subsequent retransmission-accounting check found no special R-flag bypass:
SRT's caller supplies retransmissions through the same local listener and adaptive
selector as original DATA. Real-UDP tests confirm retry bytes, queued/in-flight
load, delivery attempts and ACK-derived rate are counted. Repeated outstanding
copies of the same sequence still share one flight/ledger entry and one ambiguous
ACK credit, while every wire send contributes to cumulative bytes. Consequently
the controller's delivered-rate/unique-flight proxies are not a total-wire pacing
budget. No new production fix was justified by this narrower hypothesis; G and
the complete adaptive integration gate remain unresolved.

The historical round-11 experiment below is superseded by the hard-admission
correction above; its measurements are not current acceptance evidence.
That experiment added **hard per-link attempted-wire admission**, separate from
RateCap's ranking multiplier. Queued originals, retransmits and probes share byte
credit derived from the current RateCap target, with a two-MTU burst limit.
Unfunded datagrams wait; only the kernel-accepted prefix spends credit. Admission
uses another funded, health-admitted link when available. Otherwise one pending
input datagram pauses local UDP reads while a 1ms pacing wakeup and the existing
ACK/control/signal handlers keep running. This is bounded user-space queueing,
not a lossless-backpressure guarantee across UDP. The four legacy modes are unchanged.

**This experiment is not acceptance-green.** In round11, G's three companions stay
Healthy and all four qdiscs show zero non-model drops, but the marginal link still
fails its unchanged≤10%demotion gate. The chosen rate estimate also creates an
I throughput regression: the pre-restart sink is only2.684640Mbit/s and recovery
never reaches90% of offered load. The gate enforces its supplied estimate; it does
not discover physical capacity. D passes, while twins still fail preference share
and first-post-RPC snapshots despite successful bounded recovery and final health.
Do not deploy or call the adaptive milestone complete. See the evaluation note
for the measured limits and explicit stop boundary.

`tests/netns_hsrsp_spike.rs` contains two explicitly ignored, one-link live-handshake
latency spikes. Point `SRTLA_REC_BIN` at an out-of-tree CeraLive receiver build, install
`tcpdump` and `tshark`, and run
`timeout --foreground --kill-after=10s 90s cargo test --test netns_hsrsp_spike -- --ignored --nocapture`.
It captures the first three seconds of caller negotiation at listener latencies
2000 and 500 ms, decodes HSRSP with a test-only clean-room helper, and requires
byte-identical forwarding from the uplink to sender loopback. Explicit runs fail
on missing prerequisites rather than silently passing. `UPDATE_GOLDEN=1` deliberately
regenerates `tests/fixtures/srt-hsrsp-latency2000.bin` (raw SRT UDP payload, 80 bytes);
`HSRSP_CAPTURE_DIR` optionally names an existing directory in which unique capture
subdirectories and process logs are retained. The captured fixture now also pins the
production HSRSP parser described below.
Its additional `hsrsp_reports_listener_nak_off` case self-skips without the tools,
privileges and `SRTLA_REC_BIN`. Run that filter without `--ignored` under the same
timeout; `UPDATE_GOLDEN=1` writes the separate `srt-hsrsp-nak-off.bin` capture.
The original NAK-on capture remains unchanged.

`tests/netns_twin.rs` covers the duplicate-IP twin case that a single-subnet veth
topology cannot express: two uplinks on ONE source address, each behind its own NAT
carrier namespace, exactly as two identical HiLink dongles present themselves. It proves
that both twins register and carry traffic at the same time — and, as the control, that
the *same* topology without `--bind-map` leaves the second twin carrying nothing. It also
covers reload remove/re-add under a stable `link_id`, a file-order swap that recreates no
socket, an unplug/replug recovering on a new ifindex, and a route-removal blackhole being
reported rather than read as healthy.

### Generic N-link bond fixtures

`network_sim::bond::BondTopology::new(test_name, links, mapping)` builds 1–253
links with an explicit `LinkSpec { carrier: CarrierMode::Direct | CarrierMode::Nat,
shared_ip_with: Option<usize> }`. Direct links use veth pairs and source routing;
NAT links each own a carrier namespace, IPv4 forwarding and MASQUERADE onto
`100.64.N.0/24`. The receiver holds `10.99.0.1` on loopback and uses source-hinted
return routes. Reverse-path filtering is disabled at both namespace and device
scope. All namespaces and interfaces use the existing PID+counter naming convention.

`shared_ip_with` is a zero-based reference to an earlier link. Every member of a
shared group must use NAT; the first shared group uses `10.30.9.1`. Mapping policy
has **no default**: distinct-IP callers explicitly pass `MappingMode::None`;
shared-IP callers must choose `BindMap { rows }` or `LegacyControl`. The latter
intentionally omits the sidecar for the falsifiability control. An ambiguous `None`
configuration returns a downcastable `BondConfigError` before creating namespaces.
Bind-map rows are `BondRow { link_id, iface_index, priority: Option<f64> }`, one per
interface, ordered as the IP file should be ordered. Priority is an optional,
forward-compatible writer field, not a new scheduler feature. `TwinRow::new` is
unchanged; `TwinRow::with_priority` adds it only when requested.

The topology publishes its launch files through a `BindMapPublisher` extension
kept under `bond/` (the original twin publisher only supports one repeated IP).
`sender_args((listen_port, receiver_port), extra)` and
`spawn_sender(binary, extra)` apply the chosen sidecar policy; the convenience
spawn uses ports 5555/5000. Receiver selection and `SrtProfile` remain the caller's
explicit responsibility. Drop all `NamespaceProcess` handles before the topology.

Fault/measurement APIs are zero-based: `sender_iface`, `sender_ip`, `tx_bytes`,
`set_link_up`, `delete_default_route`/`restore_default_route`, `replug`, and
`apply_impairment(i, &config)`. Replug destroys and recreates the access veth under
the same name, with a new ifindex and no inherited shaping. `tx_bytes` is the raw
kernel netdev count and resets on replug; it includes ARP/control traffic, so a
route blackhole can add a few ARP bytes while carrying no DATA.

`tests/netns_bond.rs` proves mixed 3-Direct/1-NAT carriage, mapped shared-IP
carriage (both links ≥100 kB in classic mode), the legacy control, route-loss
isolation/restoration, and Direct/NAT replug. These are kernel wire-carriage tests,
not useful-SRT-goodput or hardware-performance claims. The pre-existing A/B runner
now calls `network_sim::bond::configure_bond_routing` rather than owning that setup.

### Temporal network profiles

`network_sim::profile` supplies `Profile`, `LinkProfile`, `TimedEvent`, `Action`,
`Scheduler`, `EventLog`, and the `BondRuntime` adapter. Create the bond and stack,
construct `BondRuntime::new(&profile, &topology, ProcessEndpoints { sender,
receiver, offered_rate })`, then run `Scheduler::new(&profile)?.run(&mut |event|
runtime.apply(event))`. The offered-rate callback must update the actual source;
it is not a simulated throughput counter. The adapter uses stack receiver port 5000.

`TimedEvent::new` defaults observation tails to 30 seconds for impairment,
obstruction, restart, flap, route and reorder events, and 5 seconds for periodic
events. Offered-rate/cross-traffic edges default to zero. `graded` labels events
for later metrics; it does not bypass validation. Explicit short smoke tests can
set their own horizon. One-shot state changes have zero hold unless an explicit
later event restores their previous state on the same link/property; that restore
ends the hold. Irreversible restart/replug operations have zero one-shot hold.

`Periodic { every, action, hold, until }` emits onsets at `at + n*every <= until`,
not until the run's duration. State-changing actions restore their pre-onset value
after `hold`; restart/replug execute once per onset without an artificial inverse.
Validation requires each actual onset + hold + horizon to fit the duration.
Nested periodic declarations, zero intervals/holds, overlapping writes to a held
property, invalid link scopes/permutations and expansion above one million events
are rejected. Equal-time restores precede new onsets; other equal-time events keep
declaration order. `Profile::from_random_walk(ScenarioConfig)` preserves seeded
samples, clips the legacy generator's overshooting final frame, and marks updates
ungraded with zero horizons.

The synchronous runner anchors `std::time::Instant` on its first run, waits through
the full duration, and logs each successfully applied event with its actual elapsed
milliseconds. Running it again cannot repeat events. Application failure stops the
run, preserves prior successful log entries, and prevents a partial action retry.

Every generic bond owns a composed `LinkQdisc`: root `prio` handle `1:` with all
sixteen priomap entries zero; band `1:1` has netem `10:` or TBF `10:` → netem `11:`;
band `1:2` has `netem loss 100%` handle `20:`. `DataBlackhole` toggles only the
priority-10 IPv4 u32 filter `match u16 0x0400 0xfc00 at 2 flowid 1:2` (IP lengths
1024–2047, **not** all larger packets). Thus 1316B UDP DATA drops while 38B
keepalives pass. Impairment updates replace only band one; a qdisc-kind change
detaches only that band, never the root or classifier. The legacy free
`apply_impairment` still clears its root and must not be used on generic bonds.

`ImpairmentConfig` additionally supports `queue_limit` (netem packets),
`delay_distribution: Normal | Pareto` (requires positive jitter), and
`tbf_latency_ms` (default 1s). Composed netem defaults to 1000 packets. With a netem
child, that child's packet limit governs backlog; TBF's emitted latency parameter
is not an independent end-to-end latency guarantee. Link-up reinstalls source
routes removed by Linux on link-down. Runtime replug reapplies the configured
impairment/blackhole and preserves explicit route/link state.

Cross-traffic runs `iperf3 -u -b <mbit>M`, or a paced Python UDP fallback, bound to
both the selected source IP and device. Its server runs in the receiver namespace;
its handles use process-only teardown. `NamespaceProcess::pid()` reports the exact
inner PID discovered from namespace PID differences plus wrapper ancestry, never
the sudo PID. `restart_process_only()` rejects exited processes with typed
`ProcessControlError::AlreadyExited`, otherwise TERM→KILLs only that process and
respawns the same argv/environment. `SrtlaTestStack::restart_receiver()` exposes it
on the legacy stack. Ordinary handles retain namespace-wide final teardown.
`SighupReorder` publishes the requested topology-index order, advances mapped
sidecar generation coherently, then signals only the sender with HUP.

The `netns_bond` target includes real DATA/keepalive probes before, during and after
mid-blackhole updates, a ten-second live profile, receiver-only restart isolation,
both cross-traffic backends, and topology/control dispatch. Run bounded:

```bash
cargo test -p network-sim --lib profile
cargo clippy -p network-sim -- -D warnings
timeout --foreground --kill-after=10s 120s cargo test --test netns_bond -- --nocapture
```

### Receiver-side benchmark metrics

The provisional C1 production listener URI is
`mode=listener&latency=2000&lossmaxttl=40&reorderfreeze=1`: freeze on, NAK reports
**on** by default (L1's policy shape, retaining the bench's 2000 ms latency).
STRICT and LEGACY_DEFAULT retain their existing tuning, and caller URIs are unchanged.
This receiver-profile correction does not change scenario loads or settling thresholds.

`reorderfreeze` is a CeraLive-only libsrt option, so the bench needs an
`srt-live-transmit` built from the CeraLive SRT fork (`github.com/CERALIVE/srt`,
apps enabled, plus a local `reorderfreeze` row in `apps/socketoptions.hpp`). Point
`SRT_LIVE_TRANSMIT_BIN` at it for every privileged run; a vanilla PATH fallback
does not apply freeze. Enabling freeze alone does **not** fix jitter/reorder settling.

**Why drop `nakreport=0`: a real trade-off, not a pure win.** The fixed-build
four-condition `classic`/G factorial isolated NAK-off as the real Gilbert-Elliott
loss regression: NAK-off alone and freeze+NAK-off each settled **0/3**, with roughly
**44–46% received retransmissions** and thousands of non-model queue drops on every
link. Freeze-only settled **3/3**; revalidation measured **1.43–1.70%** retransmissions
and **zero non-model queue drops**. But removing NAK-off gives back B1/C's gains:
`classic`/B1 **2/3→0/3** settled, `classic`/C **3/3→0/3**, useful goodput **−21% / −46%**
versus both options. **B1/C are unresolved again; this is not full C1 acceptance.**

Keep three distinct concepts apart: **receiver presets** (`balanced`,
`low-latency`, `resilient`, `low-latency-fec`, `classic`) express latency/FEC intent;
**sender modes** (`classic`, `enhanced`, `rtt-threshold`, `edpf`, `adaptive`) choose
local per-packet paths across bonded links; **receiver policy** (freeze, NAK reports,
`lossmaxttl`) handles loss/reordering. The receiver cannot observe sender mode, and
G's catastrophic signature occurs with both classic and enhanced. Sender `classic`
does **not** imply receiver `classic`/L2. They are **not** independently choosable,
though: the sender's scheduler consumes the receiver's NAK stream as its link-quality
signal on every mode, so receiver NAK policy shapes sender scheduling (see the
[interop correction and coupling record](docs/notes/receiver-policy-evaluation-2026-09.md#7-interop-mechanics-no-negotiation-but-a-real-effect-on-the-peer),
sections 7–8). **Both L1 and L2 freeze; L1 keeps NAK on, L2 turns it off.** No
scheduler or deployed receiver preset is changed here.

The earlier upstream/BELABOX-pedigree justification for L2 is **retired**. Both
BELABOX and irlserver ship freeze+NAK-off as their single receiver mode, which matters
for interop expectations but proves usage, not correctness under our conditions;
the G factorial outranks that pedigree. Wire compatibility with third-party NAK-off
receivers remains, with a real-loss performance caveat. Full evidence, limitations
and fork-build prerequisites: `AGENTS.md` → BENCH RECEIVER-PROFILE DEPENDENCY.

The B1/C-versus-A/G split now has a source-verified root cause at the receiver (the
periodic NAK re-report bypasses the reorder tolerance; NAK-off trades it for FASTREXMIT):
see the [root-cause section](docs/notes/receiver-policy-evaluation-2026-09.md#11-root-cause-source-verified-supersedes-83-84),
which retires the earlier two-factor and mis-tuned-constants hypotheses.

`network_sim::metrics` supplies receiver CSV windowing, a pcap-free 100 ms UDP sink,
1 Hz link/telemetry collectors, optional control-command deltas, CPU/RSS readings,
impairment episodes, source-load intervals, and the serde `RunRecord` schema.
The [frozen definitions and collector contract](docs/notes/bench-scenarios.md#metrics)
include a [round-tripped example](crates/network-sim/fixtures/run-record-example.json).
Primary outcomes are useful sink goodput and viewer drop/unique-packet loss, not
sender wire bitrate. Buffered delivery before an impairment reaches the viewer is
not recovery evidence; planned idle is not a zero-baseline recovery success.

The CSV parser uses the real libsrt field names and the **second `Time` column**;
reorder distance is optional. `SrtStats::parse` implements the frozen cumulative
packet/interval-belated contract. For the current listener flags without fullstats,
explicitly select `CaptureSemantics::Interval`: later live evidence demonstrated
interval packet counters. Both modes preserve raw rows and map by time, including
gaps and zero-valued observations. Never infer capture semantics from counter values.

Run `cargo test -p network-sim --lib metrics` without privileges. No production
scheduler or telemetry-producer behavior changes; campaign execution/reporting remain
separate consumers of the schema.

### Reference-backed benchmark scenarios

`network_sim::scenarios::all()` returns thirteen concrete profiles, A–L with B1/B2.
The [scenario catalog](docs/notes/bench-scenarios.md#scenario-library-al) records
cited constants, explicit rates, 45–90s windows and metric obligations. A scenario
`Profile` owns the unchanged temporal profile in `timeline`, source/warm-up rates,
the production SRT preset, and optional ramp/restart settings for the campaign runner.
Its validation expands every periodic cycle and rejects late recovery horizons.

C/D preserve both the 215ms Starlink delay spike and 500ms capacity dip without
overlapping whole-impairment holds. D adds the NAT-link DATA-only 20–28s obstruction.
L warms up at feasible load, then overloads at measurement t=0; planned idle is
ungraded and its 400ms burst ramp is one source-load interval, not a zero-baseline
impairment recovery. The runner must start the source before settling (≥90% warm-up
rate for three consecutive seconds, 30s deadline), apply ramps through source control,
and enforce I's two-second kill/respawn budget. The library does not launch a campaign.

Run `cargo test -p network-sim --lib scenarios` and
`cargo clippy -p network-sim --all-targets -- -D warnings`; these are unprivileged
definition/metric tests, not evidence of a real bonded-hardware performance improvement.

### Paired checkpointed scheduler campaigns

C1 compares all five fork modes with upstream `df0b393` classic/enhanced: thirteen
profiles, production SRT, N=5, seed 20260913, 455 required successful runs. The
intentional fork 3.3.0/upstream 4.0.1 divergence is part of the comparison, not a
reason to remove upstream. External arms use `stats_file: false` and no
`effective_config` expectation: upstream does not implement the fork's stats-file
flag or adaptive metrics. Fork test-internals arms declare their actual complete
metrics configuration in every mode. Failed listener startup now retains captured
output in `sender-startup.log`, so an unsupported argument is not hidden behind an
empty UDP-listener poll. Startup and settling bounds remain unchanged; neither
startup success nor partial campaign progress establishes performance acceptance.

`tests/bench_scheduler.rs` consumes a required **explicit `cells` matrix**, not a
Cartesian product. It interleaves candidates per seeded pair, retries failed indices,
archives stale fingerprints, and atomically publishes the existing `RunRecord` schema.
The ignored `campaign` and `smoke` tests require `--features test-internals`, immutable
binaries and Linux netns privileges. Unprivileged manifest/order/resume tests run with
`cargo test --features test-internals --test bench_scheduler`.

See the [campaign contract and manifest example](docs/notes/bench-campaign.md).
Smoke runs A+D twice per candidate using full 45s/75s windows, never shortened D.
The runner uses interval CSV semantics, real warm-up settling, FIFO source-rate
changes, bounded worker processes, and the A/B runner's shared host measurement lock.
Live campaign validation remains separate from the unprivileged gate; no hardware
performance claim is implied.

**Failure interpretation and reconnect capture:** inspect an attempt's `reason` and
`detail` before interpreting its metrics. Failed records initialize goodput to zero,
`no_traffic` to true, and observations to empty; those are **unavailable measurement
placeholders**, not proof of a dead network. Source traffic and the 30-second settling
gate precede the first logged `OfferedRate` event, so `events: []` with
`settle_timeout` can coexist with substantial delivered warm-up traffic. Inspect the
raw sink CSV's numeric `bytes` column, not its file size. The settling criterion is
unchanged and failures never count toward N.

Scenario I can reconnect the downstream SRT socket after the SRTLA receiver restarts.
Its socket-relative CSV `Time` then resets. The campaign collector now brackets the
first flushed row of **each SocketID** against the monotonic clock, retaining the
offset and uncertainty in `clocks.json` (`csv_socket_clocks`). It preserves the real
outage gap and raw CSV rows, and sums explicitly interval-valued counters across
socket changes. Cumulative capture still rejects socket/counter resets; backwards
aligned times still fail. This repairs collection, not receiver recovery time or
scheduler performance. I's two-second process kill/respawn budget is unchanged.
Use a fresh output directory for a corrected campaign; do not rewrite old failures.

The [smoke tooling](scripts/bench/README.md#smoke-campaign) builds the clean current
revision into a SHA-addressed, read-only `test-internals` artifact and defines the
classic/enhanced/adaptive A+D matrix. The owner-calibrated smoke gate requires at least
one successful run out of two in each of classic/A and enhanced/A; all D and adaptive/A outcomes
are informational. `report_smoke.sh` validates all original outcomes and provenance,
then publishes actual successful counts and indices without relabelling failures.
`assert_smoke.sh` keeps the required-record goodput, receiver-packet, configuration,
and full-window checks. Zero coverage in either required cell remains blocking.
This is path-coverage calibration, not performance certification or a C1/C2 waiver. Negative tests
mutate disposable copies, never measured evidence. This proves measurement plumbing,
not scheduler superiority or permission to tune constants.

The [final smoke report](docs/notes/smoke-final-report-2026-09.md) retrospectively scores
the existing campaign: classic/A2/2 and enhanced/A1/2 pass this scope, while all four
informational cells remain0/2. The original campaign exit101 is retained as a failure;
no new live run or scheduler change was used to obtain the scoped pass.

### Statistical reports and retention decisions

#### Authoritative verdict and lineage gate (Todo 34)

The authoritative primary verdict is unchanged after the empty defect round:
enhanced alone, with zero genuine coverage. Fold-in is skipped, not a ranking
code change. The bounded lineage gate measures18 cells/66 runs, including the
distinct SLS4003 block, without re-running M4a or expanding the ship set.
It records real passing-cell counts and sacrifices even though a single member
makes the base swap impossible. SLS's missing received-packet denominator cannot
prove the retransmission limit; it remains unknown/failing, never zero-filled.
See the [frozen lineage method](docs/evidence/bpc/m4/lineage-method.md).
This evidence decision changes neither production defaults nor shipping acceptance.
Measured66/66indices (65settled) in1h30m17s active service time,1h32m59s elapsed.
The checkpoint continuation preserved all48 earlier outcomes; no measured retry.
Enhanced passes0/18lineage cells, swapnull; all18are explicitly sacrificed, in
addition to the19primary sacrifices. All15SLTcells fail zero-drop/belated; all9SLS
conformance runs pass but retransmission fractions remainunknown. The full feature
gates still fail the known adaptiveG/twin assertions. Complete results and retained
orchestration/checker-correction receipts: [M4 final evidence](docs/evidence/bpc/m4/README.md).

#### M4 provisional ours-new matrix

**Bounded defect round (Todo 33):** no scheduler repair met all five admission
predicates. Claim-specific probes passed; B1/B2 delivery deficits remain unresolved,
not dismissed as unavoidable loss. No measurement, threshold or sender behavior
changed. The [defect-round record](docs/evidence/bpc/defects/README.md) includes
the explicit no-admission decision and empty rerun summary for Todo34. Its checker
is `uv run scripts/bench/pr_description.py --check-defects docs/evidence/bpc/defects/`;
this validates evidence structure, not historical causality or shipping acceptance.

The M4 manifest declares110 metric cells/546 planned indices, plus two separate
600-second M8 soaks. Its20 primary groups each compare all five shipped CLI modes
atN5 under TTL200; upstream references and the enhanced M4 FEC off/on pair are
noncovering. FEC uses the same filter on both ends; its default on-request ARQ is
not a test of the periodic-NAK gate. The preflight validator checks exact scope.
`decide.py --rule lineage-d1 --summaries <summary.json> --out <verdict.json>`
applies the new frozen base-agnostic rule, including post-settle metrics-v2 gates
and explicit sacrificed cells. This is distinct from the historical `d1` below.
These Todo20 results were provisional; Todo34's authoritative rerun, lineage gate
and reserved4003 block are recorded above. All546 metric indices and two soaks completed in12h43m28s.
The frozen rule selected enhanced **only as its empty-set fallback**:0% coverage,
19 sacrificed scenarios, and D explicitly exempt after all five failed coverage.
No primary mode/scenario cell passed the strict zero-drop/belated all-runs gate.
FEC narrowly passed (−4.82% goodput); both crash-free M8 soaks failed final link
health. This is not a recommendation to ship or retire modes. See the
[M4 measured results](docs/evidence/bpc/m4/README.md) and
[frozen method](docs/evidence/bpc/m4/method.md).

#### M3 rollout interoperability

The M3 manifest covers existing/new sender × old/new receiver quadrants, including
the actual released `srtla-send-rs_3.3.0_amd64.deb`, BELABOX C, irlserver Rust in
classic/enhanced modes, and current enhanced. B1/G/C/M1 run three times against
each receiver; foreign scenario-I sub-runs additionally capture registration,
exact keepalive echoes and receiver-restart recovery. The new receiver uses M1's
TTL*=200; the genuine old build is TTL40/freeze-off, not a patched baseline.

Run the manifest as one detached transient user service with the locked lane/CPU
allocation and `BENCH_MAX_RETRIES=1`. Reduce with `report.py --m3-outcomes` and
the ordinary `--manifest`, `--results`, `--out`, `--json` arguments. It retains
complete measured settle failures, requires all123 outcomes, and additionally
publishes `spike.json` and `conformance.md` beside the report. Ordinary campaign
success requirements remain unchanged. See the
[predeclared method](docs/evidence/bpc/m3-interop/method.md).

A failing released3.3.0 arm blocks receiver rollout: Todo24 must rerun M1's rule
with the failing scenario added. A foreign-only failure is a documented known
limitation, not a receiver-policy revert. No receiver lineage detection is implied.

**Measured result:** released3.3.0 fails C's retransmission criterion despite
3/3 settling and1.751× median goodput. Receiver rollout is therefore blocked
pending Todo24. BELABOX/C and irlserver-enhanced/C are foreign limitations;
irlserver-classic passes all four scenarios. The literal BELABOX echo check
finds2-byte requests but32-byte padded replies; this is not a demonstrated
operational liveness failure. Full numbers, conformance and gate caveats:
[M3 evidence](docs/evidence/bpc/m3-interop/README.md).

#### M1 receiver TTL spike

`scripts/bench/manifests/m1-ttl.json` declares 65 cells and 195 one-attempt outcomes:
the four-scenario core sweep, overloaded B1 diagnostic, two-rate one-link freeze
control, and pinned BELABOX/irlserver sender interoperability arms. Catalog scenarios
are unchanged. Its 60ms/8Mbit, no-jitter diagnostic uses seeded 1% netem loss and
retains receiver-side packet captures for gap-to-NAK and gap-to-repair timing.
Configured netem delay is not silently halved; the decision uses the specified
60ms recovery-penalty threshold. The genuine old receiver build has stock periodic
NAK and no working freeze URI option; it is not mislabeled as freeze-on.

Use `BENCH_MAX_RETRIES=1` and the normal privileged campaign command. M1 retains
full-window measurements after a settle timeout without turning that failure into
a success. `report.py` requires all planned outcomes and produces a separate M1
summary, never D-1 covering evidence; missing/infrastructure failures still block.
`decide.py --summary <summary.json> --rule m1-ttl --out <spike.json>` applies the
frozen core/foreign thresholds, upstream-parity fallback, freeze cap, and controller
trigger. Re-running it on the same summary produces identical bytes. See
[`docs/evidence/bpc/m1-ttl/`](docs/evidence/bpc/m1-ttl/) for the measured decision;
the campaign does not itself change deployed receiver or scheduler defaults.

#### SLS conformance cells (not covering-set metrics)

**Twin-port fidelity check:** the two `twinport-{default,legacy-l2}` manifests
run the unchanged enhanced/M1 scenario three times on each bonded port and
separately check the four port/profile conformance configurations. SLS remains
the sink; its attached loopback player now requests200ms and captures receive
CSV. Explicit `SLS_BONDED_PROFILE_OVERRIDE=legacy-l2` reaches the SLS process;
unset uses the server default. The TWINPORT-only path settles before measuring
and requires zero player receive loss/drop plus at least98% of offered receive
rate. A failing player leg is discarded and rerun once, never counted as a valid
delivery sample. Delivered fraction uses player bytes / sender session-byte
delta, not publisher bitrate. See the
[predeclared method](docs/evidence/bpc/twinport/method.md).

The dedicated `twinport_report.py RAW_ROOT OUTPUT_DIRECTORY` reducer validates
both profile logs and the raw player stats, compares only complete N=3 paired
observations, and keeps SLS outside D-1. No publisher received-packet denominator
means conservative DISTINCT, never a NAK-count substitute. Unidentifiable deltas
are null rather than invented zero; DISTINCT requires Todo20's reserved4003
block. Synthetic conformance does not replace a failed M1 performance index.

**Measured disposition: DISTINCT.** The live publisher API has no received-packet
denominator, and strict valid M1coverage remains2/3 vs1/3 for default4002/4003
and0/3 on both rollback ports. Four synthetic conformance configurations pass;
legacy M1carry fails11/12attempts. No N=3equivalence estimate is claimed. Full
results and the non-green branch gate caveat:
[TWINPORT receipt](docs/evidence/bpc/twinport/README.md).

An explicit manifest cell may select `sink: "sls"`, `port: 4002` (bonded) or
`4003` (deprecated bonded alias), `metrics: "none"`, and `covering: false`.
Use an explicit SRT latency preset, FEC off, and no listener URI overrides.
The normal launcher runs real SLS instead of its SLT listener and attaches an
`srt-live-transmit` player. Only player output-file growth supplies goodput;
publisher `/stats` counters remain diagnostics. Registration, ≥90% of offered
bytes reaching the player, and latency `max(device preset, 100ms)` are blocking.
Unsupported belated/occupancy metrics remain null with `not_applicable` gates.

Build the locked server with
`SLS_WORKTREE=/absolute/server/checkout bash scripts/bench/build_receivers.sh --sls`,
then run `SLS_BIN=/absolute/server/build/bin/srt_server cargo test --test bench_scheduler conformance_smoke -- --nocapture`.
The test skips if `SLS_BIN` is unset or privileges are unavailable; when enabled it
uses the locked receiver/SLT tools and the exact Cargo-built sender, failing on
missing or mismatched artifacts. It runs one 20-second cell on each port using
the synthetic `SLS` fixture (two 10 Mbit/s links, 5ms delay, 1 Mbit/s offered),
not shortened performance scenarios. Raw artifacts and report-ready results are
retained in the printed temporary directory, under `artifacts/` and `results/`.

`RunRecord.sls_stats` preserves the captured publisher fields; `sls_conformance`
holds the assertions and byte/window evidence. SLS binary, linked libsrt and
template hashes enter the fingerprint. Early ring counters can be `-1` (unknown),
never interpreted as zero. `report.py` emits SLS records only in the separate
`conformance` collection/table, not metric groups or paired comparisons.
`decide.py` independently excludes SLS tags and canonical SLS identities from
D-1 and ablation. `--rule lineage-d1` now uses the separate frozen M4 rule above;
SLS cannot satisfy any of its primary coverage obligations.
No twin-port equivalence or reserved-block non-inferiority verdict is implied.

The standalone Python tools use uv inline dependencies (`numpy==2.*`, `pydantic==2.*`):

```bash
uv run scripts/bench/report.py --results results/shard-1 results/shard-2 \
  --manifest manifest.json --out report.md --json summary.json
uv run scripts/bench/decide.py --summary summary.json --rule d1 \
  --candidates adaptive,classic,enhanced,rtt-threshold,edpf \
  --scenarios A,B1,B2,C,D,E,F,G,H,I,K --n 10 --out verdict.json
uv run scripts/bench/report.py --self-test
uv run scripts/bench/decide.py --self-test
```

Result directories are shards of **one manifest campaign**, not pooled campaigns.
Only current `status: "ok"` records count. The manifest's explicit `cells[].runs`
and zero-based run indices are authoritative; optional top-level summary arrays/counts
do not generate cells. Missing cells/indices, insufficient counts, duplicate successes,
malformed records or identity mismatches fail reporting. Failed attempts remain warnings,
and `stale/`, `artifacts/`, `raw/` and `manifest.json` are excluded from result discovery.
Keep unrelated JSON outputs outside the result directories. Reporting invalidates the
previous `--json` output before loading; failure cannot leave an old successful summary.
Successful outputs are published by same-filesystem rename, with the summary last.

Statistics contain n, missing count, mean, median, sample SD and a percentile **median**
95% CI: **10,000** resamples, fixed seed **20260913**. Pairwise comparisons bootstrap
the **ratio of medians**, resampling matched `run_index` rows together. Viewer-loss
delta is the difference of medians ×100 (percentage points). Recovery compares matching
graded, positive-horizon episodes: `recovery_ms` after a restoration, otherwise
`failover_ms`. Failed/incomplete episodes contribute +∞, never disappear from the
denominator. A pair with an infinite candidate recovery and finite reference recovery
cannot cover, even if the overall median would conceal it. Both-infinite recovery ratios
remain +∞; 0/0 for two genuinely zero-duration recoveries is 1. JSON encodes infinite
statistics as `"+inf"`, undefined statistics as `null`; decision ratios with nonfinite
medians are `null`. CPU cost uses useful sink **decimal MB**, not wire bytes or megabits.

`summary.json` schema 1 contains bootstrap metadata, `groups` and `warnings`. Each group
is `{campaign, scenario, receiver, profile, cells: {candidate: evidence}}`. Evidence
contains scalar medians, n/run indices, metric distributions, per-event/per-load
distributions, non-recovery rates, fingerprints, integrity errors, checks and all-pairs
`comparisons`. Receiver names come from manifest cell IDs, not just receiver dialects.
No receiver/profile/campaign groups are averaged together.

D-1 chooses best by median useful goodput among the requested candidates with exactly
N records. Coverage requires paired goodput CI lower ≥0.95, viewer loss delta ≤0.1 pp,
and, where graded episodes exist, median recovery ratio ≤1.10 with no greater
non-recovery rate. Missing pairs, mismatched configuration/episodes and unavailable
required evidence cannot cover. Every requested candidate/scenario cell must have N
records. Every campaign/receiver/profile group must be covered independently. The
smallest covering set wins; ties prefer adaptive, fewer tunables, then fewer switches
(missing switch counters rank as unknown/infinite, never zero).

`verdict.json` contains per-group `scenarios` keyed by
`campaign/scenario/receiver/profile`, each with `best`, `covered_by`, and candidate maps
for `ci_lower`, `viewer_loss_delta_pp`, `recovery_ratio`, `nonrecovered_rate`.
A successful `verdict: "d1"` adds `shipped_modes`, `retired_modes`, `default_candidate`.
Failure exits nonzero with `verdict: null`, `reason: "insufficient_evidence"`, errors
and uncovered scenarios, **omitting all retirement/default fields**.
J/L are excluded from set coverage and retained in `reported_checks`: J's post-restore
recovered rate; L's overload no-collapse and post-idle burst recovered rates. Positive
but sub-target burst traffic remains a failure; idle creates no obligation.

For a homogeneous ablation/sweep target set, use `--rule ablation --baseline adaptive-all
--candidates <configuration-labels> --scenarios <targets> --n 5`. Every configuration
needs its **own** A control in each campaign/receiver/profile context. The predeclared
C2 A guard requires CI lower ≥0.98. Accepted configurations receive `objectives`:
the geometric mean of target median-goodput ratios to the baseline; failures appear in
`rejected_configurations`. No accepted configuration means a nonzero exit. Evaluate
different feature/sweep target sets separately. This scoring tool does not itself
change scheduler defaults, retire modes, or establish real-hardware performance.

### Per-link preference plumbing

Bind-map rows accept optional `priority` in the finite range **−0.20..=+0.20**.
An absent value means no preference. Out-of-range values invalidate the row through
the existing `malformed` path, with its zero-based row index: startup remains
duplicate-safe and degraded reload retains the last valid mapped pool. Hash coherence
still precedes row validation; unknown additive keys remain ignored. A priority-only
edit must advance the generation when the IP-file digest is unchanged.

Connections retain three independent layers: sidecar baseline, persistent `link_id`
override, and reload-volatile `conn_id` override, in that increasing precedence.
Applied reloads update the baseline, preserve the link override through reorder or
same-identity socket replacement, and clear the conn override. Clearing either
override affects only that layer, exposing the next remaining value.

The pure `preference_multiplier` returns `1 + p * clamp((window−10000)/10000, 0, 1)`
only for Healthy links, and `1.0` otherwise (including Rejoining). The adaptive
selector consumes it; priority control commands and runtime telemetry echoes are not yet added.
Internally, `sender::pool_control::PoolControlRequest::SetLinkPriority` addresses
either a typed `LinkId` or a telemetry-position `ConnId`. `submit` returns a oneshot
reply: enqueue success alone is not application. The sender's event loop resolves
the live pool and replies with the addressed value and actual effective priority,
or a typed unknown-link error. The queue holds at most 64 requests and reports Busy
on overflow; callers must bound their reply wait. A dropped reply cancels work not
yet applied, but cannot undo an application racing with cancellation.

Before every applied reload, the sender closes the old channel and rejects queued
requests as PoolReloaded, then publishes a new handle after rebuilding the pool.
Old handles remain closed: a positional request cannot silently target a different
modem after reorder. Requests during reload fail unavailable; fetch the current
handle for a new operation. Sender shutdown disconnects outstanding replies.
The four established scheduling modes and legacy invocation output remain unchanged.
Run `cargo test --lib bind_map`, `cargo test --lib preference`,
`cargo test --lib link_identity`, and `cargo test --test bind_map_contract`.

### Link-health policy (integrated into shared admission)

`src/connection/health.rs` provides `HealthState`, `HealthSignals`, `HealthConstants`,
`HealthMachine::step`, and `Transition { from, to, at_ms }`. It is an isolated,
allocation-free policy module: all evidence and timestamps are passed in; it does not
read clocks or perform I/O. Shared selection reads the connection-owned machine;
housekeeping transitions and lifecycle resets are integrated.

Hard failures enter Down; restored connections enter Rejoining rather than skipping
the ramp. The DATA-stall condition requires both 32 attempts without DATA proof and
proof age ≥τ, where τ = clamp(4×sRTT, 1000, 3000) ms (3000 ms without a sample).
The experimental round-8 alternative is keepalive silence ≥3×IDLE_TIME (3000ms),
with DATA proof either unknown or also ≥τ old. Fresh DATA proof prevents a
control-only loss from declaring an actively delivering link stalled. Silence
starts at the first accepted keepalive send until the first reply, then at each
accepted reply; further sends never renew it. Tracking is separate from RTT
sampling, accepts zero-ms and bare two-byte liveness without seeding RTT, rejects
unmatched/replayed timestamped echoes, and resets with socket recovery. A healthy
keepalive cannot veto the DATA-stall condition or substitute for DATA recovery
rounds. Thus an idle link stalled by control silence still needs DATA evidence to
rejoin; this is a limitation of the narrow detector experiment, not a new recovery
design. Degradation uses
qualifying normal-loss EWMA ≥10%, persistent queue delay ≥max(10, 0.25×slow-min-RTT) ms,
or an independently observed `RouteHealth::NoDefaultRoute`.
Queue entry from Healthy/Rejoining requires that predicate continuously for τ,
latched at the first qualifying evaluation. Falling below entry clears the pending
episode, as does entering Down/Stalled/Degraded; Rejoining→Healthy preserves it.
Hard failure, stall, and finalized loss/route degradation still take precedence
without waiting for the queue timer. Starting the timer never increases backoff:
only an actual Rejoining→Degraded transition does.

This is a deliberate queue-entry persistence policy, not threshold tuning or an
RTT-estimator filter. A real encoder restores its offered load without knowing the
bond's internal rejoin ramp; short restoration congestion should not itself trigger
a relapse. Sustained evidence still matures within 1–3 seconds of its first
qualifying evaluation, independent of ramp length and dwell multiplier. The design
budget including detector/housekeeping quantization is approximately 5 seconds from
physical onset, not a hard real-time guarantee. Admission/sole-carrier coordination
is separate; this gate does not promise that every overload scenario recovers.

Recovery requires continuous clearance for τ: loss ≤5% and queue delay
≤max(5, 0.125×slow-min-RTT) ms. A loss-triggered demotion retains its evidence
requirement; after 10 seconds without a valid normal cohort, only complete probe-train
loss evidence can clear it. A queue-only demotion needs no invented normal-loss sample.
Route-only degradation likewise creates no loss/queue evidence. Missing routes
block clearance even after those measurements recover. Unknown route observations
neither degrade a previously unobserved link nor clear a latched absence; a known
default route must return, followed by the same uninterrupted τ clearance dwell.
`HealthMachine::route_latched()` exposes that distinct cause. HealthConstants retain
all existing values; route recovery reuses τ instead of adding a new tuning knob.

Probe recovery assumes two 10-copy trains sharing a bond-wide 10-probes/s budget.
For `m` held links, train period is `m×1000` ms and the rejoin evidence window is
`max(2×τ, 2×train_period+sRTT)`; the caller supplies the oldest contributing train's
start timestamp so expired evidence cannot rejoin a link. Three held links at 40ms
RTT therefore get 6040ms, enough for trains at 3000/6000ms. Time spent stalled is not
a permanent recovery deadline. Rejoining ramps linearly from 5% to 100% over
`max(2×τ, train_period)×dwell_multiplier`; the exact same span gates Healthy entry.
The ramp freezes entry-time RTT/tuning but accepts the current held-link count.
Soft relapse doubles dwell up to 16; reaching Healthy resets it to 1.

Run `cargo test --lib health`. The table/property tests cover all edges, threshold
gaps, stale-loss probe recovery, feasible cadence, and repeated relapse. These are
pure policy tests, not evidence of a live obstruction fix or hardware performance.

### DATA delivery evidence (integrated)

Each connection owns an independent `DeliveryLedger` (`src/connection/delivery.rs`).
Only kernel-accepted DATA enters it: `SrtlaConnection::flush_batch`, implemented in
`src/connection/transmit.rs`, records the accepted prefix's sequence, acceptance
time and actual wire length, including a prefix sent before a later transmit error.
Queueing, failed sends, keepalive RTT, cumulative SRT ACKs and NAKs are not proof.
Existing telemetry byte accounting stays at queue time.

A link-specific SRTLA ACK consumes its ledger entry, resets attempts without proof,
stamps DATA proof time and credits the wire bytes. Cumulative ACKs and NAKs may have
already removed that sequence from the congestion packet log; they cannot erase
its independent delivery evidence. Legacy ACK return values, window growth and RTT
handling remain tied to the original packet log, independently of the ledger.

The ledger retains at most 4096 entries, expiring ages greater than 6000ms and
otherwise evicting the least recently used sequence (retransmission refreshes it).
`proof_age_ms(now_ms)` measures from first acceptance until initial proof, then from
last proof; idle/reset returns unknown. `delivered_bps(now_ms)` counts credited wire
bits over `(now−2000ms, now]`, divided by two seconds. It is not viewer goodput or the
existing send-rate telemetry. Equal-ms credits share a bucket, bounding the ring at
2000 buckets. Recovery and socket replacement clear evidence and advance its generation.
Explicit old-generation ACK tokens cannot consume even a reused sequence. Wire ACKs
themselves have no generation. Reader events now capture a generation token that
the new adaptive ACK policy checks; legacy modes retain their previous behavior.
This is local stale-reader fencing, not a wire authentication guarantee.

Run `cargo test --lib health_delivery`. The scenario-D fixture sends 40 DATA packets,
processes five keepalive RTT replies over four deterministic seconds, and drains the
packet log with three NAK frames. Its ledger-fed health step is Stalled while the
preserved legacy predicate control is **not stalled**. Adaptive tests additionally
drive exclusion, actual duplicate probes, and Rejoining selection on that fixture.
Housekeeping wiring remains separate; no real bonded-hardware improvement is claimed.

### Loss and queue evidence (integrated)

Each connection now tracks normal-DATA loss in 1000ms cohorts. Only kernel-accepted
sends count as load; only unique NAK hits in that link's normal packet log count as
loss. Cohorts with at least **100 sends** feed `NAKs / sends` into an alpha-0.2
EWMA. Smaller cohorts are discarded, so a starved link's 50 sends and 50 NAKs do
not create or change its loss estimate. Duplicate/foreign NAKs, unsent suffixes,
control frames and direct probe copies are excluded. A NAK is charged to its
accepted-send cohort, not the cohort open when feedback arrives. Ten seconds of
bounded closed-cohort history allows delayed feedback to correct the chronological
EWMA without refreshing the evidence's original end timestamp. Old sub-floor or
pre-recovery-epoch feedback cannot contaminate a new loaded cohort; feedback beyond
retention is not reassigned to the current denominator. No loss-ratio clamp is used.

Adaptive ACK handling distinguishes cohort **closure** from **settlement**. Closure
freezes the accepted-send count; EWMA waits until the cohort's accepted sends have
reached their delivery-budget deadlines. Each deadline is acceptance time plus the
same negotiated latency used by adaptive admission (500ms when unknown), frozen at
acceptance. A valid normal SRTLA ACK can clear its pending debit even after closure,
but strictly before that debit's deadline. At or after the deadline, unresolved or
newly observed NAKs are final; later ACKs cannot erase them. Retries keep distinct
cohort/deadline groups and never extend an older attempt's deadline.
Duplicate, cross-link, probe, stale-generation, expired-entry and old-epoch ACKs
cannot erase another loss. Pending evidence remains within the existing bounded
delivery ledger and ten-second cohort history. Raw congestion NAK/window penalties,
legacy ACK behaviour, and Rejoining/Healthy degradation thresholds are unchanged.
This is local deadline-budget accounting, not receiver playback-time measurement
or blanket forgiveness of eventual delivery.

`LossTracker::advance(now_ms)` closes elapsed cohorts, including on idle reads.
`last_value()` and `last_cohort_ms()` preserve the last qualifying estimate and its
original cohort-end timestamp; late polling cannot make stale evidence fresh.
A separate settlement timestamp allows newly settled high-latency cohorts to qualify
without moving the original ten-second staleness anchor. Late final NAK corrections
refresh neither timestamp. `loss_cohort_ok(now, stale_after_ms)` rejects sub-floor/stale cohorts, while retained
evidence remains readable for clearance. Recovery/socket replacement resets it.
`probe_loss()` remains unknown (`None`) until two completed probe trains provide
evidence; the probe mechanism below supplies it independently of normal cohorts.

The independent queue detector compares raw RTT minima over the last **1 second**
and **30 seconds**: `queue_delay_ms = max(0, (fast_min - slow_min) / 2)`.
`RttTracker::slow_min_rtt_ms()` supplies its time-based slow floor. Both reads
exclude expired observations even without new samples; absent evidence returns 0.
Monotonic minimum candidates and same-millisecond coalescing bound storage without
changing the time-window result. **Legacy sample-count RTT minima and BLEST/EDPF
selection are untouched.** These trackers do not yet drive HealthMachine or change
telemetry, CLI options, or scheduler decisions.

Run `cargo test --lib loss`, `cargo test --lib queue_delay`, and
`cargo test --lib rtt`. Coverage includes the load guard, stale evidence, EWMA
convergence, synthetic ramp/jitter, actual partial UDP sends, unique attribution,
and the unchanged legacy RTT behavior. No bonded-hardware improvement is claimed.

### Negotiated SRT latency (passive observation)

The receive path sniffs SRT v5 conclusion handshakes for the HSRSP extension and
records its **receiver TSBPD delay in milliseconds**. The parser starts extensions
at byte 64 and walks each descriptor's body-word count; HSRSP need not come first.
Non-handshakes, other handshake versions/phases, missing HSRSP and truncated frames
silently yield no observation. Every packet retains the same forwarding behavior
and bytes, whether decoding succeeds or fails.

JSON-RPC `get-status` includes optional `negotiated_latency_ms` once a nonzero
delay is observed; unknown is omitted, never reported as zero or null. The 30-second
status log also prints this observation (`None` when unknown). It is a bond-wide,
last-observed value: housekeeping, uplink reconnection and SIGHUP do not clear it;
a subsequent valid handshake replaces it, including zero meaning unknown. It is
not authenticated and has no stream/socket-generation freshness guarantee.

`SharedStats::negotiated_latency_ms() -> Option<u32>` reads a shared atomic directly,
without snapshot locks, configuration reads or awaiting housekeeping. This is
the observation consumed by the shared deadline gate in every mode.
The required stats-file telemetry fields remain unchanged; receiver flags are
an optional additive observation as described next.

The same parser also reads the receiver's **SRT version and handshake flags**.
`get-status` adds a `receiver` object with optional `nak_report`, `srt_version`
(for example `"1.5.6"`), and `rexmit_flag`. Before any HSRSP it is `{}`.
The 30-second log adds `receiver: nak_report=on|off|unknown srt=<version|unknown>`.
File/event telemetry adds top-level optional `receiver_nak_report` after
`disposition`, preserved by the TypeScript reader in producer order. Unknown is
omitted, never null; **schema_version remains 1**. The frozen older reader accepts
the additive field (and strips it until it knows that field).

Receiver observations are cached **once per bond**, not per uplink. A SIGHUP-added
or re-registered link inherits the same observation; an encoder's new valid HSRSP
updates it. Malformed handshakes never erase it. Policy consumers must interpret
unknown conservatively as NAK-on (`nak_report_enabled()` in Rust, `value ?? true`
in TS), while keeping unknown visibly distinct from on. This addition does not
change scheduling or NAK handling, nor authenticate these receiver claims.
The new `telemetry-receiver-flags` fixture pins explicit NAK-off across Rust and TS;
all older fixtures, including the frozen legacy producer, keep their exact bytes.

Run `cargo test --lib srt_handshake`, `cargo test --lib packet_io`, and
`cargo test --test parser_proptest`. `cargo test --test negotiated_latency` runs
the real binary against a loopback UDP test peer and queries its Unix control socket.
Tests use the committed real 2000ms capture,
including a one-byte extension-type failure control and unchanged forwarding bytes.

### Duplicate DATA probes

The production-ready mechanism in `src/connection/probe.rs` supplies an owned,
bond-wide token bucket capped at **10 probes/s**, with no accumulated idle burst.
Ten-copy trains rotate across Stalled/Degraded links or deadline-held
Healthy/Rejoining links, never Down or the elected sole carrier. Adaptive selection
supplies targets, and the packet handler calls `maybe_emit` after normal forwarding.
That method flushes the primary only for a due copy and requires full acceptance;
probes do not change switch/cooldown history. Housekeeping recovery remains pending.

Copies use the alternate's normal unpadded batch path, preserving every byte except
clearing SRT byte 4's retransmit mask `0x04`, as measured by the receiver spike.
Only kernel-accepted copies enter the separate, 256-entry LRU probe log. They never
enter the sequence tracker, normal packet log, original delivery ledger or in-flight
accounting. Five ACKs from ten distinct copies within `2*m seconds + sRTT` make a
train OK, where m is the number of eligible held links. Expired/incomplete trains
contribute losses; two finished trains provide probe-loss evidence. Idle history
expires, and the future health sampler must advance it before reading.

Adaptive ACK attribution checks the reader's captured socket generation, then looks
only in the **arrival link's** probe log and original ledger. An ACK arriving on A
cannot consume B's probe or vice versa, even with the same sequence and either
arrival order. Replayed/expired probe ACKs cannot fall back to another link's
original. Probe proof refreshes health only: no window growth or original delivered
bitrate credit. Todo 32 applies this attribution and growth policy to every mode.

Accepted probe bytes count in `bytes_sent_total` and `bitrate_bps` because duplicate
DATA costs wire capacity too; their unsent suffixes do not count. `probes_sent` is
status-log-only, not a new telemetry JSON field. The original feature-only spike
hook retains its direct-send/no-accounting behavior. Run `cargo test --lib probe`,
then the separate unchanged `cargo test --lib ack_rtt` and `cargo test --lib batch_io`
suites. Coverage uses real loopback UDP and deterministic clocks, not bonded-hardware
performance measurements.

### Delivered-rate controller (integrated)

`src/connection/rate_cap.rs` is a pure per-link controller designed against the
[audited congestion-controller defects](docs/notes/strata-port-evaluation.md).
Every mode reads its soft-cap multiplier from the connection-owned controller;
the one-second housekeeping `tick` and lifecycle resets are integrated.
Existing enhanced-mode time-based window recovery is unchanged and independent.

Each future one-second housekeeping tick reads the link's `DeliveryLedger` directly:
the input rate is SRTLA-ACK-credited DATA bits/s over two seconds, not transmitted
bitrate, probe traffic or keepalive proof. Bootstrap seeds `max(1 Mbps, first delivery)`.
Normal climb adds 2% per tick; stable RTT permits 6% (absolute Kalman velocity
≤0.1 ms/update, jitter ≤10% of sRTT, and zero queue delay). Recovery after loss
backoff or Drain gets five full 4% growth ticks; holding/idle does not spend them.

RTT above 1.5×baseline holds the target. At least 2×baseline with known zero loss
enters Drain, cutting once by 25%, with a ten-tick guard between cuts on different
episodes. Loss ≥1.5% can back off only while delivered rate is ≥30% of target:
`next = max(0.85 * previous, min(delivered, previous))`. After three completed
backoff ticks, loss that has not fallen below 80% of entry suppresses further cuts
for thirty ticks, including delay-only cuts. This avoids repeatedly cutting for
loss that the rate reduction does not improve.

Zero delivered rate holds the target indefinitely. Starting at the tenth consecutive
idle tick the BDP cap is suspended; renewed delivery restores it without reseeding.
Otherwise the cap is `max(32, floor(target_bps * rtt_min_ms / 1000 / 8 * 1.5 / 1316))`.
Above-cap load multiplies the ranking score by `cap / in_flight`, never by zero;
below-cap and suspended paths use 1.0. There are no share-based admission verdicts.

Run `cargo test --lib rate_cap`. Deterministic ledger-backed tests include the
120-idle-tick audit trace, five exact-named defect regressions, thirty-tick loss
latching and recovery boundaries. These are policy tests, not live-bond or
hardware-performance evidence; runtime/lifecycle integration remains later work.

### Duplicate-DATA receiver spike (test builds only)

`tests/netns_dup_spike.rs` is an ignored, privileged experiment using two registered
uplinks, the **CeraLive** receiver from `SRTLA_REPO/build-pkg/srtla_rec` (or
`build/srtla_rec`), real `srt-live-transmit`, and a Python UDP sink. Build that
receiver first; a PATH-only receiver is deliberately not accepted. Run with:

```bash
SRTLA_REPO=/absolute/path/to/ceralive-srtla \
  timeout --foreground --kill-after=10s 180s \
  cargo test --features test-internals --test netns_dup_spike -- --ignored --nocapture
```

Optional `DUP_SPIKE_OUTPUT` retains the pcap, full stats CSV, process logs, byte
streams, JSON results and decision in a fresh output directory. Each variant sends
3000 × 1316-byte messages over 15 seconds (200 pps), with receiver
`latency=2000&lossmaxttl=40`. A/B require at least 250 duplicate sequences at the
receiver's **loopback SRT input**, exact source/sink bytes, and full unique-packet
coverage. A fresh unregistered UDP port must put 50 replays on the receiver's
external interface and **zero** duplicates on loopback. Captures must have zero
kernel drops. The wire decision minimizes `belated + retransmitted` receiver events
(equal weights; ties choose clear), rather than prioritizing one counter lexically.

Only `test-internals` builds recognize `SRTLA_TEST_DUP_EVERY=<positive n>` and
`SRTLA_TEST_DUP_RETX_BIT=0|1`. Both must be valid to activate the hook. Every nth
complete DATA frame is selected for copying to another registered, non-timed-out
uplink. The primary batch is flushed first; an error or retained partial-send suffix
suppresses the copy. The alternate's own bound socket sends directly, bypassing
batch queue, packet log, in-flight, bitrate and sequence ownership. Only the copy's
SRT byte **4**, bit **2** (`0x04`, second header word bit 26) is changed. No hook code
or environment names are compiled into ordinary release builds. This is an isolated
receiver-dedup experiment, not a production scheduler or hardware-validation claim.

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
- `--mode <MODE>`: Ranking mode: `classic`, `enhanced` (default), `rtt-threshold`, `edpf`, `adaptive`; shared admission applies to all five
- `--no-quality`: Disable the shared quality multiplier in every mode
- `--exploration`: Enable connection exploration (enhanced only)
- `--rtt-delta-ms <N>`: RTT delta threshold in ms (default: 30, rtt-threshold only)
- `--control-socket <PATH>`: Unix domain socket path for remote control (e.g., `/tmp/srtla.sock`)
- `--stats-file <PATH>`: Write per-uplink telemetry JSON to `<PATH>` (opt-in; see [Telemetry](#telemetry))
- `--stats-file-interval <MS>`: Telemetry write cadence in milliseconds (default: 1000)
- `--earned-ack-window`: Retired compatibility flag; accepted and ignored under shared arrival-scoped ACK handling
- `--stall-deselect`: Retired compatibility flag; accepted and ignored with one WARN per process
- `--stall-min-in-flight <N>`: Ignored compatibility setting (parsed default: 32)
- `--stall-ack-stale-ms <MS>`: Ignored compatibility setting (parsed default: 3000)
- `--stall-reprobe-ms <MS>`: Ignored compatibility setting (parsed default: 1000)
- `--bind-map <PATH>`: Optional versioned bind-map sidecar describing `BIND_IPS_FILE` positionally (see [Bind-map sidecar](#bind-map-sidecar-optional)). Absent means byte-identical legacy behavior
- `--capabilities-json`: Print a machine-readable capability document and exit `0` (see [Capability probe](#capability-probe))
- `-v, --version`: Print version and exit (see [Version output](#version-output))

### Version output

`srtla_send -v` prints the crate version, an optional git build-metadata
parenthetical, and the package name:

```bash
$ ./target/release/srtla_send -v
3.3.0 (main@974c8b9) [srtla_send]
```

The parenthetical is emitted only when the build could resolve a commit. Building
outside a git checkout — an exported source tarball, a container that copies only
`src/`, a vendored crate — is a normal build with nothing to name, so the metadata
is omitted entirely rather than filled with a placeholder:

```bash
$ ./target/release/srtla_send -v
3.3.0 [srtla_send]
```

A tag build (detached HEAD) reports the bare hash, `3.3.0 (974c8b9) [srtla_send]`,
and a build from a modified working tree suffixes the hash with `-dirty`.

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

**Classic Mode**: Capacity argmax over shared admission weights, without cooldown.

**Enhanced Mode** (default): Quality-based scoring that punishes connections with recent NAKs. More recent NAKs = more punishment. Additional 30% penalty (0.7x multiplier) for NAK bursts (≥5 NAKs in short time). Optional connection exploration for testing alternative connections.

**RTT-Threshold Mode**: Groups links into "fast" and "slow" based on RTT measurements. Links within `min_rtt + delta` (default 30ms) are "fast" and strongly preferred. When quality scoring is also enabled, NAK penalties are applied within the fast link group. Falls back to slow links only when all fast links are saturated. Useful for reducing packet reordering in networks with heterogeneous latencies.

**EDPF Mode**: Earliest Delivery Path First over the shared admitted set. Its BLEST → IoDS → EDPF pipeline retains the static 50ms OWD guard, congestion escape, ordering reset, flat 1 Mbps bootstrap and velocity/BDP penalties. The shared multiplier supplies the existing loss-clamped effective-capacity input exactly once. State is owned per send-loop; exploration does not apply.

## Experimental Scheduler-Hardening Flags

**Retired compatibility inputs as of Todo 32:** both flags and the three stall
tunables remain accepted but are ignored. Enabling `--stall-deselect` emits one WARN
per process. Shared health admission and arrival-scoped ACK policy always apply.
The following descriptions record the superseded experimental mechanisms, not
current behavior or a way to disable shared admission.

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
{"schema_version":1,"last_updated_ms":1749556546000,"connections":[{"conn_id":"0","rtt_ms":42,"nak_count":3,"weight_percent":85,"window":8192,"in_flight":100,"bitrate_bps":2500000,"bytes_sent_total":812000000,"iface":"wwan0","link_id":"modem-a"}],"bytes_sent_total":1620000000,"bind_map_status":{"state":"active"},"disposition":{"state":"mapped"}}
```

- `conn_id` — the uplink's index in `BIND_IPS_FILE` order, as a string. **Transient** —
  see [Link identity](#link-identity-conn_id-is-transient-link_id-is-not) below.
- `rtt_ms` — Kalman-smoothed RTT.
- `weight_percent` — the link's normalized share of selection weight (0–100).
- `health` — optional in the schema, emitted by every mode: `healthy`, `degraded`, `stalled`, `rejoining`,
  or `down`. A held-out link has zero weight unless elected as the sole/fallback
  carrier; a Healthy link can still be deadline-held. The health enum alone does
  not determine admission.
- `priority` — optional effective configured preference (−0.2..=+0.2), echoed in
  any mode when set. This is not its multiplier: adaptive applies the bias only
  to Healthy links, with a window-dependent ramp. An absent priority is unknown,
  not a fabricated zero. Both fields follow `link_id`; absent keys are omitted.
- `bitrate_bps` — send rate in **bits per second** (wire bytes/s × 8).
- `window` / `in_flight` — congestion-window and in-flight packet counts.
- `bytes_sent_total` — cumulative **bytes** sent this session. Present at two scopes:
  per connection (that uplink) and at the top level (the whole bond).
- `iface` / `link_id` — **optional**, per connection. The interface the link's socket is
  bound to, and the bind-map sidecar's writer-assigned identity. Both are absent for an
  unmapped (legacy) link — the sender only ever *echoes* an identity and never invents one.
- `bind_map_status` / `disposition` — **optional**, top level. The sender's actual
  operating mode; see [Operating mode](#operating-mode-bind_map_status--disposition).

### Link identity: `conn_id` is transient, `link_id` is not

`conn_id` is a **position**, not an identity: it is the link's index in `BIND_IPS_FILE`
order, so a `SIGHUP` reload that reorders the file gives the same physical modem a
different `conn_id`. It is retained for compatibility and for correlating records *within
one snapshot*.

**A UI must key on `link_id`.** It is the sidecar's opaque, writer-assigned id, and it
survives reloads, reorders, reconnects, DHCP lease changes, and moves to a different
interface. Two twin modems that share one source IP are distinguishable *only* by it.
A link with no `link_id` is unmapped, and there is nothing stable to key on.

### Operating mode: `bind_map_status` + `disposition`

Two orthogonal fields, so a consumer renders what the sender is *actually* doing instead
of inferring it from log text (ADR-003 §6.4):

```json
"bind_map_status": {"state": "degraded", "reason": "hash_mismatch"},
"disposition": {"state": "retained_last_valid"}
```

- `bind_map_status.state` — `active` | `absent` | `degraded`. `reason` is present only
  when degraded, and is one of `hash_mismatch`, `malformed`, `unknown_iface`,
  `retry_exhausted`, `missing_file`, `unreadable`, `unsupported`.
- `disposition.state` — `mapped` | `retained_last_valid` | `legacy_unique_only` |
  `startup_collision_excluded`.

They are orthogonal because a degraded map does not imply a broken bond: a degraded
**reload** leaves the last valid mapped pool running (`retained_last_valid`), while a
degraded **startup** has nothing to retain and excludes the ambiguous rows
(`startup_collision_excluded`). The latter carries the group it broke up:

```json
"disposition": {"state": "startup_collision_excluded",
  "collisions": [{"ip": "192.168.8.100", "effective_index": 0, "excluded_indices": [1]}]}
```

`effective_index` / `excluded_indices` are **`BIND_IPS_FILE` line positions**, not
`conn_id`s — an excluded line never becomes a connection, so the two numberings diverge
exactly when this array is present. This is what lets an operator with two modems and one
visible link be told *why*, from typed data.

### `schema_version` handling

`schema_version` stays **`1`**. It names the shape of the **required** fields, not the set
of fields present:

- the schema grows **only by addition**, and every added field is **optional**;
- a consumer therefore keeps parsing a newer document (the Zod reader strips keys it does
  not know), and a producer that omits an added field — an older build — still validates;
- the version is reserved for a change no old consumer could survive: renaming, retyping,
  or **removing** a required field, or changing a unit.

None of `iface`, `link_id`, `bind_map_status`, or `disposition` does any of that, so none
of them bumps it. The proof is committed: `tests/fixtures/telemetry-golden.json` is
byte-for-byte `tests/fixtures/telemetry-legacy-producer.json` (the pre-ADR-003 producer's
own output) plus the additive tail, asserted by `tests/telemetry_fixture_parity.rs`.

Optional `health` and `priority` likewise keep schema 1. The ninth fixture,
`telemetry-adaptive`, covers a mapped Healthy carrier with priority +0.2 and a
Stalled neighbour with zero weight and no priority. Both copies are generated by
Rust and byte-roundtripped by the TS reader. All older fixtures remain unchanged;
old producers read as `undefined`, with neither key materialized. Deleting health
from a parsed adaptive document deliberately fails the byte comparison.

`cargo test --test telemetry_adaptive -- --nocapture` exercises the live binary on
two loopback uplinks, withholding DATA ACKs on one while echoing keepalives. The
harness runs the exact built executable through a private `atel-<test-pid>` hard
link so host-wide production-name cleanup cannot kill the test. Exit/deadline
failures include captured child logs; the scenario keeps its 15-second bound.

With no active links the file still exists with `"connections": []` ("running but idle",
distinct from "absent"). The live file is removed on clean shutdown (SIGTERM/SIGINT).

### Cumulative session bytes (`bytes_sent_total`)

This is the "how much data have I transferred?" figure, and it is deliberately **not**
the same kind of number as `bitrate_bps` sitting next to it:

| | `bitrate_bps` | `bytes_sent_total` |
|---|---|---|
| Unit | **bits** per second | **bytes** |
| Kind | instantaneous rate (2 s window) | cumulative count |
| Conversion | wire bytes/s **× 8** | none — passed through verbatim |

It counts SRT DATA at full wire length, so SRT-level retransmits are included (they
really do cost the data plan twice); SRTLA control frames — keepalives and registration
— are excluded, matching `bitrate_bps`.

The duplicate-probe mechanism also counts accepted DATA copies at full wire length
in both fields. It adds these bytes at accepted-prefix processing rather than
queueing, so an unsent probe suffix is excluded; normal DATA queue-time accounting
is unchanged. Every mode invokes the production probe scheduler.

**It resets only when the sender process does**, which is once per streaming session:

- a per-link reconnect (radio stall, socket replacement) does **not** reset it;
- a `SIGHUP` IP-list reload that drops an uplink does **not** make it go backwards —
  the bond figure is a session accumulator, not a sum of the currently-live links, so a
  departed link's bytes stay counted;
- a re-added uplink returns as a fresh connection and accrues on top;
- stopping the stream and starting a new one restarts it at 0.

Full rationale, the complete reset table, and the consumer contract are in
[`docs/adr/ADR-002-session-bytes-telemetry.md`](docs/adr/ADR-002-session-bytes-telemetry.md).

## Bind-map sidecar (optional)

`srtla_send` identifies an uplink by its local source IP. Two identical modems in
HiLink/RNDIS mode both present `192.168.8.100`, so the second one is silently collapsed
into the first and never carries traffic. `--bind-map` supplies the missing information —
which interface, and which stable identity, each row of the IP list refers to.

`BIND_IPS_FILE` is **not** changed. The mapping rides a separate JSON sidecar that
describes it **positionally**: the Nth row describes the Nth accepted IP line, which is
exactly what tells duplicate IPs apart.

```json
{"schema_version":1,"generation":7,"ips_file_sha256":"<64 lowercase hex>","links":[
  {"link_id":"modem-a","ip":"192.168.8.100","iface":"wwan0"},
  {"link_id":"modem-b","ip":"192.168.8.100","iface":"wwan1"}]}
```

```bash
./target/release/srtla_send 6000 rec.example.com 5000 /tmp/srtla_ips \
  --bind-map /tmp/srtla_bind_map.json
```

The writer publishes the IP file first and the sidecar second, each by atomic rename; the
**sidecar rename is the commit point**. A reader landing between the two renames sees new
IP bytes against an older sidecar — a detectable mismatch that a bounded retry (5 attempts
over at most 2 s) absorbs.

If the pair never agrees, the sender **fails open without guessing**:

- **at startup**, unique IPs run as usual, and each duplicate-IP group keeps one
  deterministic representative while the rest are excluded *and reported* — an operator
  with two modems and one visible link is told why;
- **on a reload** (SIGHUP) that degrades, the sender keeps the last valid mapping running
  rather than silently un-binding a live bond.

### What a mapped link does differently

A mapped uplink's socket is bound **to the interface and to the source address**:
`SO_BINDTODEVICE` decides which interface the packet physically leaves by (overriding the
routing table, so the host no longer needs source routing), and `bind(ip, 0)` pins the
source address the receiver sees. Both are needed — the device binding alone would let the
kernel choose a source address, which is exactly what makes two same-IP modems
indistinguishable on the wire.

Beyond binding, three things change for a mapped link:

- **Identity outlives the socket.** A link is its `link_id`, not its IP. Reordering the
  file, changing a modem's DHCP lease, or moving it to another interface does not make it
  a different link — but a socket key that moves gets a **new socket**, because the
  window, packet log, and in-flight counts all described the interface it left.
- **The interface is re-resolved by name every time a socket is created**, and re-checked
  every second. `SO_BINDTODEVICE` freezes the interface index at bind time, so a modem
  that is unplugged and replugged leaves a working-looking socket that can only fail. A
  re-enumeration rebinds; a disappearance marks the link `removed`, and it waits for the
  next reload rather than retrying against a name the kernel no longer knows.
- **Losing the default route is reported, not guessed at.** Traffic pinned to an interface
  with no default route is silently blackholed — IPv4 ARPs for the receiver's public
  address and `sendto` still succeeds. So default-route presence is read from the routing
  table and shown per link in the status log, separately from whether the link is still
  ACKing. Nothing is ever written to the routing table.

**Without `--bind-map` nothing above happens** — no hashing, no sidecar, no device
binding, no new failure mode. `--dry-run` validates both files and exits non-zero if the
sidecar is unusable.
Full contract: [`docs/adr/ADR-003-bind-map-contract.md`](docs/adr/ADR-003-bind-map-contract.md).

## Capability probe

`--capabilities-json` prints one line of JSON describing what this build supports, then
exits `0`. It binds no sockets, writes no files, and needs no positional arguments.

```bash
$ ./target/release/srtla_send --capabilities-json
{"schema_version":1,"binary":"srtla_send","version":"3.3.0","capabilities":{"bind_map":true,...}}
```

It exists so a supervisor can decide **before spawning a stream** whether to pass
`--bind-map`. Older binaries do not have the flag and exit non-zero with a usage error —
that is the intended "no support" answer. Treat **any** non-zero exit, unparseable output,
or timeout as no support and use the legacy spawn.

The **running** process answers the same question with the same document: the JSON-RPC
`get-capabilities` method on `--control-socket` returns every key of the probe document
verbatim, plus an additive `methods` array enumerating the control methods and event
topics. A supervisor that probed the binary and a consumer that asks the live socket can
never be told two different things (pinned by
`get_capabilities_matches_the_pre_spawn_probe_document`). `hello`'s `capabilities` field
is unchanged — it remains the frozen string array the TS control binding feature-detects
with.

`get-status` additionally reports the live operating mode and per-link identity:

```json
{"mode":"enhanced","quality_enabled":true,"exploration_enabled":false,"rtt_delta_ms":30,
 "bind_map_status":{"state":"active"},"disposition":{"state":"mapped"},
 "links":[{"conn_id":"0","iface":"wwan0","link_id":"modem-a"}]}
```

It also includes `negotiated_latency_ms` when a nonzero receiver delay has been
sniffed from HSRSP; see [Negotiated SRT latency](#negotiated-srt-latency-passive-observation).

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

- The local `SRT_LISTEN_PORT` listener is bound before the IP list is read and before any uplink is dialed, so a local SRT producer that connects the instant the process starts is never rejected while the bond is still coming up. Uplink setup is sequential (one resolve + bind + connect per link), so on a multi-modem bond this ordering is what keeps startup latency off the local listener.
- For each IP in `BIND_IPS_FILE`, the sender binds a UDP socket **without connecting it** to `SRTLA_HOST:SRTLA_PORT`; the resolved peer is named on every send instead. This is deliberate, not an oversight — it matches the C `srtla_send`/`_rec` reference pair and BELABOX, and tolerates a NAT/multi-homed receiver replying from a source address other than the one dialed. The tradeoff: any host that can reach an uplink's ephemeral port can inject traffic that reaches protocol state. The mitigation is defense in depth, not filtering — a `foreign_source_datagrams` counter plus a rate-limited (1/s) debug log on source mismatch, never a silent drop and never in the telemetry JSON.
- Incoming SRT UDP packets are read on `SRT_LISTEN_PORT` and forwarded over the currently selected uplink based on the score `window / (in_flight + 1)`. Outgoing DATA is flushed in batches of up to 32 datagrams via `sendmmsg(2)` on Linux (a sequential fallback on other platforms); a batch flush commits only the kernel-accepted prefix, in order, so a partial send can neither duplicate nor drop a packet, and any flush error is routed through the same connection-recovery path the rest of the send loop uses.
- A successful socket reconnect keeps using its cached receiver peer and never waits on diagnostic DNS. The blocking system resolver runs on one detached standard thread at a time; a Tokio task waits up to 3 seconds for the result, while the resolver keeps the process-wide permit until it actually returns so timeout or waiter cancellation cannot overlap another lookup. Detached resolver work does not join Tokio runtime shutdown. Empty answers are inconclusive, and drift warnings are limited to one per minute across the bond. The check never repoints one uplink independently; coordinated whole-bond receiver migration remains deferred.
- Internal timing (NAK decay, window recovery, liveness) reads a monotonic clock, so it survives a wall-clock step (NTP correction, manual clock change) without a spurious jump. The `--stats-file` telemetry's `last_updated_ms` deliberately stays wall-clock instead, because a telemetry reader compares it against its own `Date.now()`.
- The SRT NAK loss list is parsed starting at the correct wire offset (16 bytes into the control frame), with wrap-safe 31-bit sequence-number handling and a truncation warning if a single NAK frame names more loss entries than the per-packet cap.
- ACKs are applied to all uplinks to reduce in-flight counts; NAKs are attributed to the uplink that originally sent the sequence (tracked), falling back to the receiver uplink if unknown.
- RTT measured from an ACK is attributed the same way. An SRT cumulative ACK is broadcast to every uplink, but only the uplink the sequence tracker says carried the acknowledged sequence turns it into an RTT sample — the others would otherwise report a latency they never observed. If the sequence can no longer be attributed (the tracking entry expired), no uplink samples it. An SRTLA ACK names one specific sequence, so a packet-log hit is itself the attribution and it feeds the smoothed RTT directly.
- Sequence-number comparisons are 31-bit modular (RFC 1982), so ACK processing keeps advancing across the `0x7FFFFFFF → 0` wrap instead of stalling behind a numerically larger stale value. An ACK that is exactly half the sequence space away carries no ordering information and is ignored rather than guessed at.
- **Burst NAK Detection**: The system tracks NAK bursts (multiple NAKs within 1 second) per connection. When quality scoring is enabled, connections with recent NAK bursts (≥5 NAKs in burst, within last 3 seconds) receive an additional 0.7x multiplier (30% reduction) to their quality score, helping avoid connections experiencing packet loss issues.
- Keepalives are sent when idle, and periodically for RTT measurement; the RTT is smoothed via a Kalman filter, whose velocity diagnostics are milliseconds per sample (`ms/sample`), not per second. The Kalman output is clamped to ≥0 before use. Keepalive and ACK RTT samples share one plausibility gate: a sample of exactly 0 (a reply within the same millisecond, or a clock that moved backwards) and anything above 10 s are both discarded, so neither biases the filter. A genuine sub-millisecond round trip on a LAN or loopback link also measures 0 and is therefore not sampled. Window recovery is conservative and time-based when there are no recent NAKs.
- Small control packets (keepalive, REG1/REG2) are zero-padded to a 32-byte minimum on the wire (`MIN_CONTROL_PKT_LEN`), matching the C `pad_sendto` behavior, so cellular/carrier NAT keepalive thresholds don't silently drop tiny control frames. DATA packets are never padded.
- A REG3 only registers an uplink the sender actually sent a REG2 on, and the authorization is one-shot: a duplicate or replayed REG3 is counted and ignored instead of resetting a live uplink's window and in-flight state. A REG2 broadcast retry skips uplinks that are already registered or already awaiting their REG3. A SIGHUP reload that reorders the pool drops only *incomplete* registration attempts — established uplinks keep their socket, registration, and window.
- An accepted changed receiver group ID invalidates pending REG3 grants belonging to the previous ID. Otherwise those old grants would suppress the new group's REG2 broadcast until a retry timeout. Established sockets remain untouched, and REG_NGP acceptance is unchanged.
- A REG_ERR is honored only for an uplink that is actually mid-registration (awaiting its REG2, or awaiting its REG3); one arriving on an established link is counted and ignored rather than disconnecting it, and an in-phase REG_ERR clears only the handshake state that uplink owns, never another uplink's concurrent attempt. Every out-of-phase REG3/REG_ERR remains counted, but only the first of each kind logs at `WARN`; repeats are `DEBUG` so spoofed traffic cannot flood default-level logs.
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
- Last observed negotiated SRT receiver latency in milliseconds (unknown before HSRSP)

**Debug logs** (when `RUST_LOG=debug`) show:

- Per-packet connection selection decisions
- Quality multiplier calculations
- NAK burst detections and recovery
- Exploration attempts
- Hysteresis decisions

### Troubleshooting

**If only some connections are used**:

1. Check for NAKs in logs - degraded connections naturally get less traffic in enhanced mode
2. Try classic mode: `mode classic` - changes ranking, not shared health or quality admission
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

**Window Recovery (`connection/mod.rs`):**

- `RTT_VELOCITY_GATE_THRESHOLD`: 2.0 (ms per Kalman update, NOT ms/second) - halves the window-recovery increment while RTT is rising this fast or faster; sim-tested only, see [Experimental scheduler-hardening flags](#experimental-scheduler-hardening-flags)-style hardware-validation caveat in `AGENTS.md`

**EDPF Mode (`selection/edpf.rs`), `--mode edpf` only:**

- `VELOCITY_PENALTY_FACTOR`: 0.005 - scales the RTT-velocity ranking penalty added to predicted arrival time
- `BDP_OVERRUN_MULT`: 1.5 (with a 1ms propagation floor) - ranking penalty multiplier for links over their bandwidth-delay-product cap; a ranking term, not an exclusion, so an all-over-cap pool still selects the least-overrun link instead of emptying
- `BOOTSTRAP_CAPACITY_BPS`: 1 Mbps flat placeholder used for a link with no measured send rate yet (fresh registration, or idle past the 2s bitrate window)

Both EDPF constants and the RTT-velocity gate are heuristics carried from upstream, sim-tested (unit/golden tests plus the `netns_edpf` netem topology) but not exercised against real bonded hardware — see `AGENTS.md` → ROBUSTNESS FIXES (upstream sync, 2026-08) before citing either as a proven improvement.

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
