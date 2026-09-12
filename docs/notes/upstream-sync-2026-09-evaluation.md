# Upstream sync evaluation: 2026-09

## Scope and outcome

This evaluation covers the complete upstream range after the previous recorded sync:

- Fork parent: `https://github.com/irlserver/srtla_send.git`
- Previous upstream parent: `c9f6bb2296f236d60802f2ec3b79d9da4dac6e28`
- Evaluated upstream tip: `df0b3938791ff24eced4aed8b29e3d49d0efb639`
- Merge base: `c9f6bb2296f236d60802f2ec3b79d9da4dac6e28`
- Exact commit count: 7

The sync advances the upstream parent with a normal two-parent merge commit. It does not
copy upstream's split-crate architecture into this fork. Existing fork-native fixes stay
authoritative. Three narrow improvements that were not yet complete here are adapted in
follow-up commits on the same integration branch:

1. correct the remaining Kalman RTT velocity log unit from `ms/s` to `ms/sample`;
2. move detect-only receiver DNS drift diagnostics off the housekeeping loop, bound the
   lookup, prevent concurrent checks, treat an empty answer as inconclusive, and share the
   warning throttle across the process;
3. keep counting every out-of-phase REG3/REG_ERR while emitting only the first rejection
   of each kind at `WARN` and subsequent rejections at `DEBUG`.

Whole-bond receiver re-home remains deferred. The upstream `4.0.1` version bump is not a
fork release decision and is not imported; this fork is already released and pinned at
`3.3.0`.

## Exact-set validation

The fetched reference was pin-verified before the merge:

```text
git rev-parse refs/remotes/irlserver/main
df0b3938791ff24eced4aed8b29e3d49d0efb639

git merge-base c9f6bb2296f236d60802f2ec3b79d9da4dac6e28 refs/remotes/irlserver/main
c9f6bb2296f236d60802f2ec3b79d9da4dac6e28

git rev-list --count c9f6bb2296f236d60802f2ec3b79d9da4dac6e28..refs/remotes/irlserver/main
7
```

The exact ordered set is:

| # | Commit | Subject | Verdict |
|---|---|---|---|
| 1 | `d3fe9df` | `fix(srtla-protocol): validate SRT NAK range endpoints and stop wrap-safely` | Already stronger in fork; no direct port |
| 2 | `83ca832` | `docs(srtla-core): label the kalman RTT velocity as ms/sample, not ms/s` | Adapt remaining log label |
| 3 | `21fc289` | `fix(srtla-core): compare ACK sequences with 31-bit serial arithmetic` | Already stronger in fork; no direct port |
| 4 | `8f8823b` | `fix(srtla_send): harden the uplink recovery path` | Recovery already stronger; adapt DNS diagnostics |
| 5 | `4fb3079` | `fix(srtla-core): harden registration against spoofed control packets` | Gates already stronger; adapt bounded logging |
| 6 | `171ddc1` | `feat(srtla_send): re-home the whole bond when the receiver moves` | Deferred; do not ship as-is |
| 7 | `df0b393` | `fix(srtla_send): update version to 4.0.1 in Cargo files` | Reject automatic version bump |

Every commit in `c9f6bb2..df0b393` appears exactly once in the table.

## Commit decisions

### `d3fe9df`: NAK range endpoint validation

No code is copied. The fork's `src/protocol/parsers.rs` and
`src/protocol/srt_seq.rs` already provide a stronger contract:

- parse the SRT NAK loss list at byte offset 16;
- reject a range end with the high bit set;
- use typed 31-bit serial arithmetic, including wrap-crossing ranges;
- bound the whole packet's expanded loss list, including single entries;
- report truncation through `NakList.truncated` and a rate-limited warning.

The relevant tests include `test_parse_srt_nak_range_at_domain_max_emits_one`,
`test_parse_srt_nak_range_crossing_wrap_is_expanded`,
`test_parse_srt_nak_range_end_with_high_bit_is_skipped`, and
`test_parse_srt_nak_truncation_stops_parsing`.

### `83ca832`: Kalman RTT velocity unit

The fork's thresholds and documentation already use milliseconds per Kalman update/sample,
not milliseconds per second. One keepalive diagnostic still printed `ms/s`; the integration
adapts that remaining label to `ms/sample` without changing the filter or thresholds.

### `21fc289`: wrap-safe cumulative ACK handling

No code is copied. The fork uses `Option<SrtSeq>` instead of an out-of-domain integer
sentinel and rejects malformed high-bit ACK words at the parser boundary rather than
normalizing them later. `src/connection/ack_nak.rs` performs bounded modular walking and
serial-order pruning. `src/tests/ack_rtt_tests.rs` covers first ACKs, duplicates, stale
ACKs, sequence wrap, wide gaps, and the exactly-half-space ambiguity.

### `8f8823b`: recovery and DNS drift diagnostics

The send/recovery portion is already stronger here:

- `sendmmsg(2)` commits only the kernel-accepted prefix;
- all hard flush errors recover the link and remove its sequence ownership;
- `last_sent` advances only for confirmed transmission;
- DNS drift is detect-only and never repoints one uplink independently.

The DNS diagnostic still needed adaptation. Before this sync, `SrtlaConnection::reconnect`
awaited `lookup_host` on the housekeeping loop, an empty answer counted as drift, and every
uplink owned an independent warning timer. The adapted shape is detached and bounded,
permits at most one process-wide lookup at a time, treats empty results as inconclusive,
and emits at most one drift warning per minute for the whole bond. Socket recreation keeps
using the cached peer and does not wait on diagnostic DNS.

### `4fb3079`: registration spoof hardening

The fork's registration gates remain authoritative and stronger than upstream's sans-IO
variant:

- a REG3 grant is armed only after a successful REG2 send and is consumed once;
- a failed REG2 resend revokes a stale grant;
- a broadcast retry skips connected and already-granted uplinks;
- REG_ERR is phase-gated and clears only state owned by the addressed uplink;
- a SIGHUP reorder clears incomplete index-scoped state;
- out-of-phase events remain explicit and counted.

This sync adopts only bounded log severity: the first rejected REG3 and the first rejected
REG_ERR warn, while later rejections of the same kind are debug-only. Acceptance behavior
does not change. The reverted REG_NGP gate and REG3 expiry remain absent so ordinary
receiver restart recovery is preserved.

### `171ddc1`: whole-bond receiver re-home

The architectural direction is reasonable, but the upstream implementation is not safe
to ship in this fork yet:

1. It updates connection metadata before fallible socket replacement. If replacement
   fails, a surviving old socket can still target the old receiver while registration is
   restarted, so metadata coherence is not wire-destination coherence.
2. A later SIGHUP addition resolves DNS independently. Reordered multi-address answers can
   attach the new link to a different receiver from the surviving bond. A safe design needs
   one authoritative bond endpoint used by every add, replace, and reconnect path.
3. Reader cancellation does not invalidate packets already queued in `UplinkPacket`, which
   currently carries only `conn_id` and bytes. Old-generation control traffic can therefore
   enter newly reset registration state. Migration needs a socket/bond generation check.
4. Upstream has resolver-seam unit tests but no live or network-namespace test that moves a
   receiver, completes a fresh handshake, and proves payload forwarding resumes.

This remains a separate, versioned follow-up. Its acceptance gate must cover failed socket
rebuilds, queued old controls, DNS reorder plus SIGHUP, duplicate-IP device-bound links,
removed interfaces, session-byte monotonicity, and real end-to-end payload recovery.

### `df0b393`: upstream `4.0.1` version

The version-only upstream commit is not imported. Fork versions name fork releases, tags,
and Debian artifacts; they do not automatically mirror upstream's package number. The
repository evidence at evaluation time is coherent on `3.3.0`:

- `Cargo.toml` and the `srtla_send` package entry in `Cargo.lock` are `3.3.0`;
- `scripts/release_version_contract_test.sh` selects tag `v3.3.0` and both
  `srtla-send-rs_3.3.0_{arm64,amd64}.deb` artifacts;
- GitHub release `v3.3.0` is published;
- the workspace `versions.yaml` pin is `v3.3.0`.

## Merge-resolution policy

The merge is a real `--no-ff` two-parent merge, not `-s ours`. The upstream parent remains
in history so the next sync starts after `df0b393`. Conflict resolution intentionally
retains this fork's integrated architecture and excludes cleanly added upstream-only files
as well as conflicted ones. In particular:

- no `crates/srtla-core`, `crates/srtla-protocol`, or replacement `src/net` tree is restored;
- no `src/sender/rehome.rs` or `--no-rehome` CLI behavior is introduced;
- no duplicate sequence helper is added beside `src/protocol/srt_seq.rs`;
- no upstream manifest, dependency, toolchain, workflow, or `4.0.1` version change is kept;
- the three narrow improvements above land as reviewable fork-native follow-up commits.

This records ancestry reconciliation and selective adaptation honestly; it does not claim
that all seven upstream patches were copied into the fork.
