# S-PARALLEL — conservative sequential terminal outcome

Date: 2026-09-17. Sender checkout HEAD at execution: `68a6c51`, following Todo 9's
`61b8710`. **Final `parallel_lanes = 1`. Parallel isolation is NOT proven.**

The owner explicitly permits this terminal outcome when a genuine multi-lane
measurement is impractical. This run has real privileged P=1 evidence, not a
permission failure. No P=2/4/6/8 data was produced. The mandated JSON outcome
`isolation failed at P=2; sequential` means **P=2 could not be certified**, not
that an observed P=2 goodput median failed a statistical test.

## Host topology and privilege

`lscpu -e`, `lscpu`, `nproc`, the processor entries in `/proc/cpuinfo`, and sysfs
topology report 28 online/available logical CPUs, one socket, 20 physical cores,
one NUMA node (node 0, CPUs 0–27), Intel Core i7-14700K. Current process affinity
was 0–27. CPU topology from `lscpu -e`:

| Logical CPUs | Core IDs | SMT arrangement | Maximum MHz |
|---|---|---|---:|
| 0–1, 2–3, 4–5, 6–7 | 0, 1, 2, 3 | two threads per core | 5500 |
| 8–9, 10–11 | 4, 5 | two threads per core | 5600 |
| 12–13, 14–15 | 6, 7 | two threads per core | 5500 |
| 16–27 | 8–19 | one thread per core | 4300 |

All rows are online, socket 0, NUMA node 0. The complete sibling partition is in
the lock's `host.smt_siblings`; sysfs confirms `cpu0/thread_siblings_list = 0-1`
and `cpu16/thread_siblings_list = 16`.

- `sudo -n true`: exit 0, passwordless sudo available.
- `sudo -n unshare --net -- ip link show`: exit 0, isolated loopback visible.
- Cargo, jq, ip, tc, taskset, srtla_rec and srt-live-transmit available.
- No host package installation, service shutdown, or host affinity reconfiguration.

Reserve CPUs **0–3** (two whole SMT cores) for shared/OS work; allocate **4–27**
to lane 0. This follows Todo 36/the task's explicit four-thread reservation,
rather than the verification section's older illustrative pool `8-23`. No SMT
pair is split. This is an exclusion from the bench's affinity mask, **not** a
cpuset reservation that evicts other programs from those CPUs.

The host was busy: preflight load average **9.65 / 11.10 / 9.55**, with multiple
browser/development processes consuming CPU. The existing reporter flags every
measured run with `loadavg>2` (8.69, 10.4, 10.63). Its warning threshold is not a
claim that 28 threads are saturated. The relevant failure is the plan's explicit
quiet-host prerequisite. Other users' tasks were neither killed nor repinned.

## Discovered runner boundary

Contrary to the premise that Todo 9 supplied a lane-aware runner, the current
`tests/bench_support/runner.rs` holds one measurement lock for the entire campaign
and executes each paired work item sequentially. `tests/support/measurement_lock.rs`
uses a shared host lock inode. `tests/bench_support/receivers.rs` consumes receiver
entries, **not `host.parallel_lanes` or the CPU/port map**. Candidate lock updates
preserve unknown metadata through serde flattening; this spike retains the
diagnostic's actual candidate hash without replacing the receiver map.

`tests/bench_support/stack.rs` hard-codes topology name `bench` and ports
4001 (SRT listener), 5000 (SRTLA receiver), 5555 (sender SRT), 6000 (encoder feed),
9999 (UDP sink). Control uses a unique temporary Unix socket, not TCP; stats HTTP
is absent. Namespace/veth names retain PID+atomic-counter uniqueness. Artifact
directories are unique per attempt inside `lane-0`.

Launching two campaign processes would **serialize**, not prove simultaneous
lanes. Bypassing the measurement lock would undermine its cross-worktree safety
contract. No such bypass was attempted. Implementing a dispatcher and consuming
the new map requires files under `tests/`, outside this task's scope fence.

Therefore the lock's `bpc0-` namespace prefix and `[20000,20099]` port allocation
are **reserved metadata, NOT wired launch configuration**. Future lane k has
`base + k*100` through `base + k*100 + 99`; the validator checks this contract.
It proves static allocation consistency only, not live isolation. The legacy
sequential diagnostic uses its original names/ports, as recorded in `spike.json`.

## Real P=1 diagnostic

The actual cell was **enhanced × A × ours-new**, production SRT, receiver extras
`&periodicnakgate=1&lossmaxttl=200`, seed 20260913, N=3, **one attempt per index**.
No smoke override, shortened window, retry cherry-picking, or threshold change.
All three completed the unchanged warm-up gate and full **45-second** measurement
window at **24 Mbit/s** offered load. The runner exited 0 in **189.62 seconds**;
`report.py` exited 0. `ok` means valid collection, not acceptable viewer loss.

Before launching:

```sh
cargo build --release --features test-internals
cargo test --features test-internals --test bench_scheduler --no-run
taskset -c 4-27 sudo -n sh -c 'taskset -pc $$'
```

Both builds passed. The sudo child reported affinity **4–27**. The entire
diagnostic was launched under taskset, so the runner and spawned descendants
inherit that mask, including the sender, receiver, SRT listener/caller, Python
encoder feed and sink. No every-process/thread live affinity census was captured;
this inheritance probe must not be described as one. tc/netem are per-netns kernel
constructs and require no separate userspace process pinning.

The manifest explicitly supplied the current `target/release/srtla_send`, the
installed CeraLive receiver (`/usr/local/bin/srtla_rec`, resolved by the runner),
and the SRT worktree's built tool with both receiver options. It declared
`stats_file: true`, all nine shared scheduler features including `quality`, and
the default tuning (32, 0.1, 0.5, 0.85). Actual binary hashes are frozen below;
the sender was a Cargo output checked by the runner, not an immutable release
artifact. `receivers.lock.json` remains otherwise unpopulated pending Todo 4.

Final inspection found another task editing this shared worktree, including
production source. HEAD identifies the checkout base, **not a proven clean source
tree for the builds**; no source snapshot was frozen. The measured binary hashes
remain the artifact identity of record. This is another reason not to promote
these diagnostic observations into campaign/isolation certification.

With `ROOT=/home/andres/.cache/opencode/tmp/opencode/bpc-task36-p1`, execution was:

```sh
BENCH_MANIFEST='<explicit one-cell JSON described above; normalized copy in results/manifest.json>' \
BENCH_OUT_DIR="$ROOT/results" BENCH_ARTIFACT_DIR="$ROOT/lane-0" \
BENCH_MAX_RETRIES=1 taskset -c 4-27 \
timeout --foreground --kill-after=30s 420s \
cargo test --features test-internals --test bench_scheduler -- \
  --ignored --nocapture --exact campaign

uv run scripts/bench/report.py --results "$ROOT/results" \
  --manifest "$ROOT/results/manifest.json" \
  --out "$ROOT/report.md" --json "$ROOT/summary.json"
```

Use the saved normalized manifest to reproduce the exact input; reusing these
result paths resumes the old checkpoints rather than making new observations.
The general netns gate script was not used: it ignores its positional target.

| Run index | UTC start | Goodput (bps) | Post-settle pktRcvDrop | Load average |
|---:|---|---:|---:|---:|
| 0 | 07:56:53.904 | 12,510,071.466666669 | 47,541 | 8.69 |
| 1 | 07:57:57.036 | 23,837,497.6 | 718 | 10.4 |
| 2 | 07:59:00.258 | 16,867,259.733333334 | 29,705 | 10.63 |

Drops use `diagnostics.pkt_drop_delta`, derived from interval receiver CSV in the
measurement window, not a cumulative whole-process count. The receiver CSV has
the genuine `pktRcvDrop` column. Raw run IDs, record hashes and artifact root are
in `spike.json`. Successful settling did not prevent subsequent throughput loss.
No causal attribution of this variability to host contention or scheduler defects
is established by this experiment.

## CI comparison and decision per P

The unchanged reporter uses **10,000 percentile-median bootstrap resamples**,
seed **20260913**. P=1 median: **16,867,259.733333334 bps**, 95% median CI
**[12,510,071.466666669, 23,837,497.6] bps**; median drops **29,705**.
This is a univariate baseline CI, not a cross-P paired confidence interval.
The reporter's self-comparison `[1,1]` is tautological and is not isolation evidence.

| P | Completed observations | Comparison against P=1 | pktRcvDrop not worse? | Decision |
|---:|---|---|---|---|
| 1 | lane 0: 3/3 | baseline CI above; busy-host diagnostic only | baseline only | conservative terminal allocation |
| 2 | not run | unavailable; no paired CI | unknown | not certified; sequential fallback |
| 4 | not run | unavailable | unknown | not authorized |
| 6 | not run | unavailable | unknown | not authorized |
| 8 | not run | unavailable | unknown | not authorized |

**Do not infer P=2 failed statistically.** No larger P qualifies, because the
multi-lane execution prerequisite and quiet-host condition were not met. Select
the owner's predeclared terminal alternative: **campaigns run sequentially on
one lane**. Never exceed 1 based on apparent host idleness. Campaigns themselves
remain sequential, pairing remains back-to-back, and no campaign-quality or
scheduler-performance acceptance follows from this diagnostic.

For downstream execution, retain the existing host measurement lock, explicitly
launch the whole runner with `taskset -c 4-27`, use a fresh campaign-specific
`lane-0` artifact directory, and arrange a quiet host. Merely adding this host
block does not make the legacy runner enforce affinity or read allocated ports.
Multi-lane dispatcher/prefix/port plumbing and live isolation proof remain
unimplemented/unrun, not silently satisfied by the terminal outcome.

## Provenance and allocation checks

| Artifact | SHA-256 |
|---|---|
| sender | `bd8bb7a25037b613026fc222612ffda2710fa2985355bbd82b201530e1c0fe57` |
| srtla_rec | `cee2c4c1a681bb7558d8cf7b5e0437963fdc5aa5acff47205cab02fa90f698e6` |
| srt-live-transmit | `e34c3571d7222d44deb45114f57912591e60caa65cfeac69c0a2b39fcfaf1e76` |
| generated summary.json | `a87f090ff99ad57a056c67cd555f84005d8a1015f4769700dc1d90d21a286953` |

```sh
bash scripts/bench/validate_parallel_host.sh
bash scripts/bench/validate_parallel_host_test.sh
```

The validator checks nonempty integer CPU sets, exact online/reserved coverage,
four reserved threads, SMT cohesion, contiguous lane IDs, pairwise-disjoint CPU
sets, inclusive non-overlapping port ranges, 100-port stride, unique prescribed
namespace prefixes and per-lane artifact directories. Tests exercise one- and
two-lane **synthetic** valid maps and sixteen independently broken maps. CPU and
port-overlap errors name both lanes. Synthetic validator tests are never counted
as throughput observations. The failing-first invocation exited 127 before the
validator existed. No production source or baseline manifest is changed.
