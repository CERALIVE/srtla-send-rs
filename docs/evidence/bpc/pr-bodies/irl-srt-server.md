# feat: converge bonded listeners on one NAK-on gated policy (libsrt 1.5.7+ceralive.1)

> Prepared PR body for `CERALIVE/irl-srt-server`, branch `feat/bonded-path-convergence` → `master`.
> Status: **NOT OPENED**. Blocked on the owner action for `CERALIVE/srt` (PR #24 merge, tag
> `srt-v1.5.7+ceralive.1`, completed `publish-release.yml` dispatch). The single remaining
> engineering step before opening is the re-pin of `ARG SRT_COMMIT` + the three `ci.yml`
> `SRT_COMMIT` values + `scripts/check-srt-pin.sh` `EXPECTED_PIN` to the SHA the tag resolves
> to, followed by a green `docker build .` on that pin. Substitute the release SHA into the
> "Build pin" line below at that point.

---

## Summary

Both bonded publisher listeners (`listen_publisher_srtla` on 4002 and the deprecated
`listen_publisher_srtla_classic` alias on 4003) now carry ONE policy: reorder freeze,
NAK on, the periodic NAK gate from `CERALIVE/srt`, static `LOSSMAXTTL=200`, a 100 ms
receive-latency floor, and FEC accept. L3 (direct / player / fallback) is byte-unchanged.
A single environment override rolls both bonded ports back to either pre-cutover policy.

Build pin: `CERALIVE/srt` `<release SHA of srt-v1.5.7+ceralive.1>` (currently the branch
tip `ca14c8bd06c89d2fd7b69bb3d8eea48dd47c2e3e` until the tag exists).

## Policy: before / after

| Listener | Directive | Before | After |
|---|---|---|---|
| L1 | `listen_publisher_srtla` (4002) | freeze, NAK on, TTL40, floor100, FEC, no gate | freeze, NAK on, **gate on**, **TTL200**, floor100, FEC |
| L2 | `listen_publisher_srtla_classic` (4003) | freeze, NAK **off**, TTL40, floor100, no FEC, no gate | **identical to L1** (deprecated alias, one WARN per process) |
| L3 | `listen_publisher` / player / fallback | no freeze, default NAK, TTL200, no floor, no FEC | unchanged |

Startup proof per listener:

```
profile=L1-bonded freeze=1 nakreport=1 periodic_nak_gate=1 lossmaxttl=200 floor=100 fec_accept=1
profile=L2-bonded-alias freeze=1 nakreport=1 periodic_nak_gate=1 lossmaxttl=200 floor=100 fec_accept=1
profile=L3-direct freeze=0 nakreport=default periodic_nak_gate=0 lossmaxttl=200 floor=0 fec_accept=0
```

A build against stock or BELABOX libsrt refuses bonded startup with a named ERROR
(`bonded profile requires SRTO_PERIODICNAKGATE (libsrt >= 1.5.7+ceralive.1); refusing to start listener`)
and exits nonzero. There is no silent downgrade path.

Per-streamid policy is structurally impossible (`srtla_rec` is libsrt-free; the SRT handshake
terminates at the encoder), so TTL and the gate are the receiver's only levers and both are
fixed pre-accept.

## Rollback

`SLS_BONDED_PROFILE_OVERRIDE=legacy-l1|legacy-l2` + process restart. Read once by
`libsrt_init`, logged at INFO, switches both bonded ports together, never touches L3,
unknown values warn and keep `converged`. Each device reconnects once because NAKREPORT
and the gate are pre-connect options. No rebuild needed.

## Runbook

`docs/bonded-policy-cutover.md` (this branch): owner preconditions (tag + completed
`publish-release.yml` + literal `.deb` asset check), then srt image → receiver rollout on
BOTH ports → ≥24 h `/stats` soak (reorder hold via `msRcvBuf`/`latency`, drops via
`pktRcvDrop`/`viewerPktSndDrop`, NAK pressure via `pktSentNAKTotal` vs `pktRcvRetrans`,
reconnects via `uptime`/`ingestDiscontinuities`) → platform routing flip as the last,
independently revertible step. Port 4003 is removed one release after the platform flip.

## Evidence summary (measured, N=3 per cell unless stated)

**M1 (TTL* selection).** Frozen rule branch `core_failure_upstream_parity` → TTL* = 200.
Owned cell passes per TTL: 40 → 0/8, 200 → 3/8, 500 → 2/8. Maximum identifiable freeze
penalty 644.64892578125 ms (>250 ms) caps TTL at 200 independently. 24-Mbit diagnostic
median useful Mbit/s: TTL40 4.495222, TTL200 5.432214, TTL500 5.262830; best = 200,
gap 0%, no strict CI separation → controller = false.

**Todo 24 re-evaluation (released `ours-3.3.0` / scenario C added, 68 cells / 204 outcomes).**
Combined TTL* = 200, controller = false. The added cell at TTL200: 3/3 settled, 0/3 owned
joint pass, retransmissions 40.591255 / 26.554053 / 51.584450 %. At TTL500: 3/3 pass,
0.505766 / 0.192057 / 0.313309 %. TTL500 fixes only that cell and is forbidden by the
freeze cap. **Residual: released 3.3.0 on C still fails the retransmission criterion at
the shipped TTL200.** Not relabelled; accepted by the owner-gated rollout.

**M3 (foreign-sender interop, converged receiver vs pre-cutover).** Existing senders:
converged receiver settled 48/48, passed 39/48; pre-cutover settled 25/48, passed 6/48.
Per-cell scenario C:

| Sender | Passing | Goodput ratio | Disposition |
|---|---:|---:|---|
| `belabox-c` | 0/3 | 1.158 | `known_limitation` |
| `irlserver-rust-enhanced` | 1/3 | 1.459 | `known_limitation` |
| `irlserver-rust-classic` | 3/3 | 1.763 | pass (all four scenarios pass) |
| `ours-3.3.0` | 0/3 | 1.751 | `receiver_pr_blocker` → re-evaluated by Todo 24 above |

`ours-new` failed C (0/3, 1.016) and M1 (0/3, 1.151) in M3; superseded by the M4 lineage
campaign (SLS 4003, M1/M4/M6: 9/9 settled, conformance passing, retransmission unknown
because SLS publisher stats carry no received-packet denominator; zero lineage passes
overall). Treated as unproven, not passing.

**S-TWINPORT.** Outcome `distinct`: too few valid paired attached-player indices to
identify `d_goodput_pct` / `d_loss_pp`. 4002/4003 equivalence rests on the policy tables
and startup lines being literally identical, checked on every attempt, not on goodput.

## CI

- `Build Check` (`docker-build` amd64 + arm64 against the pinned CERALIVE/srt, asserts
  `SRT compat mode: reorderfreeze+periodicnakgate`, Trivy + Syft) — required.
- `CI`: `repository-contracts`, `build-and-test` (canonical debug / ASan-UBSan / TSan,
  stock full-test asserting bonded refusal + L3 relay, BELABOX compile-only),
  `clang-tidy` diff gate, `clang-format`, `fuzz` (4 × 60 s), `coverage`.
- `scripts/check-srt-pin.sh` enforces that Dockerfile and all three `ci.yml` pins agree.
- `tests/test_srt_profiles.cpp`: real ephemeral sockets, literal policy comparison across
  all overrides in fresh CTest processes, gate-syscall failure injection, accepted-socket
  inherited TTL / reorder tolerance (200 converged, 40 legacy).

## Out of scope

- The platform routing flip (todo 28) and the 4003 removal (one release later).
- Any change to L3, the freeze/TTL coupling, or a bitrate-bucket controller (not built;
  the static branch was selected by measurement).
