# M3 predeclared measurement method

Five sender configurations (BELABOX C 37862da, irlserver Rust df0b393 classic
and enhanced, released CeraLive 3.3.0 enhanced, current HEAD enhanced), two
receiver policies, B1/G/C/M1, three indices each: 40 cells / 120 observations.
Three additional N=1 foreign-sender scenario-I cells supply conformance evidence.
These form four rollout quadrants: existing/new sender × existing/new receiver;
foreign senders belong to the existing-sender population, but have a distinct
known-limitation disposition. The current and released own senders use enhanced,
the shipped default; this is not a claim about every local scheduling mode.

Receiver new is the locked ours-new libsrt with NAK on, periodic gate on,
freeze on, TTL=200, from M1's core-failure upstream-parity fallback. Receiver old
is the genuinely unmodified b06fdb6b build, TTL40, NAK on and freeze off: its URI
parser lacks the freeze row. Both use the same locked CeraLive SRTLA receiver.
Caller libsrt is held constant across all cells. Full catalog windows, offered
rates, warmup threshold and events remain unchanged. No foreign source edits.

Exactly one attempt per index. A full-window settling timeout is a measured
negative result; no retry/replacement. Missing metrics/infrastructure failures
block reduction rather than being interpreted as bad network performance.
For each (sender, scenario), require at least two of three new-receiver runs
to jointly settle and have received retransmission fraction ≤0.10; require
new-receiver median useful goodput / same-sender old-receiver median ≥0.95.
Report every per-run value so settling and retransmission are independently
visible. This uses the prior M1 joint-run convention; no threshold is tuned.

Foreign failures are known limitations, never grounds to revert the policy:
the receiver cannot detect sender lineage; the only lever is TTL*/gate, which
M1 chose. Any released 3.3.0 failure blocks the receiver PR: Todo24 must rerun
M1's frozen rule with every failing scenario added before TTL* is applied.

Conformance uses scenario I's unmodified 20s receiver restart, ≤2s process
kill/respawn budget and 60s measurement window. Before starting either peer,
capture both directions of registration/keepalive on both sender interfaces,
512-byte snaplen, and retain pcaps plus capture-drop logs. Prove the initial
REG1 → receiver REG2 → sender REG2 → REG3 exchange with exact group identity,
and both links' grants; compare literal keepalive payloads and lengths in both
directions. Restart recovery requires a fresh group handshake after the event
and useful sink delivery ≥90% of offered rate for three consecutive seconds
within the remaining scenario window. Report timing separately; do not treat
the profile's ungraded restart event as an automatic pass.

Launch one transient `systemd-run --user` service, not nohup, pinned with taskset
to CPUs4–27 (lock.parallel_lanes=1), retaining the ordinary host measurement
lock. Command/start/exit/log metadata lives beside raw artifacts. Resume by
observing that service, never launching a duplicate after chat failure.

## Conformance stimulus correction (after the performance campaign)

The loaded BELABOX scenario-I run captured no keepalives: continuous DATA does
not exercise its idle-only keepalive path. This was reported as an evidence gap,
not an incompatible echo. A separate, explicitly labeled ten-second idle capture
was therefore added for each foreign configuration against the identical new
receiver. It uses the same stack with source rate left at zero after registration.
Its `identity.json` must match the original sender/receiver hashes and options.
These are control conformance observations only, not retries or additional
performance indices; all original123 outcomes and their decisions are unchanged.
Raw supplement: `conformance-idle/` alongside the original campaign `results/`.
Reproduce with `M3_IDLE_DIR=<raw-root>/conformance-idle timeout 120s taskset -c 4-27
cargo test --features test-internals --test bench_scheduler
m3_tests::m3_idle_conformance -- --ignored --exact --nocapture`.
