# Upstream hard fork, 2026-09: closure ledger

Closure record for the September 2026 hard fork of the CERALIVE bonding stack onto its
irlserver/Haivision upstreams. Each repository's own `AGENTS.md` is the living contract;
this note only records where the fork started, what crossed the boundary, what was
measured, what was erased, and how to get back.

## Bases

| Repo | Canonical | Upstream base (also the last-merged upstream SHA) |
|---|---|---|
| `CERALIVE/srtla-send-rs` (Rust sender) | `main` | `irlserver/srtla_send` `df0b3938791ff24eced4aed8b29e3d49d0efb639` (v4.0.1) |
| `CERALIVE/irl-srt-server` (SLS) | `main` | `irlserver/irl-srt-server` `a86dd8abd3659baea8ca8315bb4842410d4bc292` |
| `CERALIVE/srtla` (C receiver) | `main` | `irlserver/srtla` `b8359bc80ce99ac4fc8f603cfa455cdd0a0a4241` |
| `CERALIVE/srt` (libsrt) | `master` | Haivision `v1.5.7` `899348d8318eb9a3c5a5b6ec43c4a1114288773a`, true-merged as `7a6cc868408e35ef60520ca587d5e8b4373cc920` |

The first three are exact clones of their base plus individually committed CERALIVE
commits; the base SHA is the first commit's parent and stays an ancestor of `main`
forever. `srt` was advanced from its existing `master` by one upstream merge.

## Ported and dropped, per repo

- **Sender.** Ported as `port(*)` commits on top of `df0b393`: the CeraUI control-plane
  flags (`--verbose`, `--dry-run`), empty-start and clean-shutdown semantics, 32-byte
  control padding, the ADR-001 `--stats-file` sink, ADR-002 session bytes, the mimalloc
  feature gate, `--capabilities-json` plus `get_capabilities`, the ADR-003 bind-map, the
  Loom lane, and the `netns_twin` topology. Dropped: every legacy scheduler mode and
  scheduler-hardening flag, the legacy control dialect, and the TypeScript binding
  track. The full list is the commit range
  [`df0b393...v4.1.0`](https://github.com/CERALIVE/srtla-send-rs/compare/df0b3938791ff24eced4aed8b29e3d49d0efb639...v4.1.0),
  and the contract those ports satisfy is `AGENTS.md`, PARITY CONTRACT.
- **SLS.** Server source stays byte-identical to upstream except the ledgered `port`
  rows (security-class `fix(auth)`/`fix(core)`); CI/CD, the libsrt pin, and the contract
  scripts were carried. Everything else (profiles/modes, audio-gap concealment, the
  `ConnRateLimiter`, already-upstream remediation) was dropped. Ledger:
  [`docs/notes/upstream-hardfork-sls-ledger.md`](https://github.com/CERALIVE/irl-srt-server/blob/main/docs/notes/upstream-hardfork-sls-ledger.md).
- **Receiver.** `src/` is upstream verbatim, with one accepted exception: the
  pre-existing NDEBUG `assert()` clock-read defect in `src/sender.cpp` (commit
  `2dafdf2`). Carried: receiver-only install, the GTest harness, the compat/A-B harness,
  CI. Dropped: high-RTT tiers, relative jitter scoring, lifecycle hooks, structured log
  markers, the C sender, sender telemetry, TS bindings. The recovery-keepalive cadence
  change was measured instead of shipped (D21 below). Ledger: the receiver's
  [`AGENTS.md`](https://github.com/CERALIVE/srtla/blob/main/AGENTS.md) ("What CERALIVE
  adds, and nothing else") plus the D21 patch and verdict under
  [`tests/compat/patches/`](https://github.com/CERALIVE/srtla/tree/main/tests/compat/patches)
  and
  [`docs/evidence/ab-keepalive-cadence/`](https://github.com/CERALIVE/srtla/tree/main/docs/evidence/ab-keepalive-cadence).
- **srt.** The two socket-option commits on top of the v1.5.7 merge, `SRTO_PERIODICNAKGATE`
  (119, tri-state) and the `SRTO_SRTLAPATCHES` (118) compat enumerator, plus their
  declaration fix and release notes. Nothing else.

## Identities

| Artifact | Identity |
|---|---|
| Sender package and GitHub release | APT component `srtla`, package `srtla 4.1.0`, tag `v4.1.0` (`35b6df538cfe58dcea846da48f4ef46e48b21399`) |
| SLS image and tag | `ghcr.io/ceralive/irl-srt-server:3.1.0`, tag `v3.1.0` (`05a3ac2fedac037e062d0b870c001b174587f5fa`) |
| libsrt | tag `srt-v1.5.7+ceralive.2` (`d487b13365205b6cd5da9d9b50868c323e255b7c`), package `libsrt1.5-ceralive 1.5.7+ceralive.2` |
| C receiver | **unreleased** by decision D22: no tags, no releases, no `.deb`; consumers build `srtla_rec` from a git ref |

## A/B verdicts

Both campaigns ran on the receiver repo's compat harness against pre-registered rules;
the verdict files carry the rule's `sha256` so the decision is reproducible.

- **D10, periodic-NAK gate** (`ab-periodic-nak`, arm A `periodicnakgate=1` filter vs
  arm B `periodicnakgate=2` suppress-exact-upstream; N=3, 4 cells). **Winner: B (`2`)**.
  A won `loss_pp` on 1/4 cells with the goodput guard holding; the rule needed 3. So
  `SRTLA_PATCHES_DEFAULT_NAKGATE = 2` shipped in `srt-v1.5.7+ceralive.2`.
  Rule `sha256` `3afffaf6743175a065855d4216791d82afc5a0612c30db84f251a63932d49fea`.
- **D21, recovery-keepalive cadence** (`ab-keepalive-cadence`, arm A upstream vs arm B
  the cadence patch; N=3, 3 scenarios). **Winner: A (upstream)**. B won 0/3 (S1 time to
  recovery 1017 vs 1018 ms; S2 both censored at 38000 ms; S3 B slower, 22335 vs
  16167 ms). No port PR; the patch stays under `tests/compat/patches/` as a build-time
  artifact only. Rule `sha256`
  `001627231a69858063cec124e465c782cd99f4f1bfbb04bb319c749e20a61f58`.

## Erasure receipts

- **GitHub (todo 50).** Sender: 22 tags / 7 releases became 1 / 1, exactly `v4.1.0`;
  21 tags and 6 releases deleted (`v1.0.0`, `v1.0.1`, `v3.0.1`, `v3.1.0`, `v3.2.0`,
  `v3.3.0` releases; the `v1.*`, `v2.*`, `v3.*`, and `bindings-v*` tags).
  Receiver: 3 tags / 3 releases (`v2026.6.0`, `v2026.6.1`, `v2026.6.2`) became 0 / 0.
  `https://github.com/CERALIVE/srtla-send-rs/releases/download/v3.2.0/srtla-send-rs_3.2.0_amd64.deb`
  answers HTTP 404.
- **npm (todo 51).** `@ceralive/srtla-send` deleted by the owner through the npmjs.com
  package settings (the equivalent of `npm unpublish --force`); an independent
  `npm view @ceralive/srtla-send version` afterwards exits non-zero with `E404`.
  `@ceralive/srtla` was never published (`E404` before and after).
- **APT (todo 52).** Component `srtla-send-rs` retired from `apt.ceralive.tv` via
  `apt-worker` PRs #48 and #49: exactly six objects deleted
  (`srtla-send-rs_{3.1.0,3.2.0,3.3.0}_{amd64,arm64}.deb`), the allowlist entry removed,
  both `Packages` indexes re-signed. `grep -c '^Package: srtla-send-rs$'` is 0 on both
  architectures; `srtla 4.1.0` and `libsrt1.5-ceralive 1.5.7+ceralive.2` still resolve,
  and every other component's record count is unchanged before and after.

## Rollback

The pre-fork canonical branches survive verbatim as `legacy` on the three swapped repos:
sender `35b673d663314e64a4a1249ed3ce1969b5bf7fef`, SLS
`ae229f96e38ff23ab880913c3c8d9c8e933e0f77`, receiver
`182972cc23da6625a288108562663c97b59892c0`. Any old release can be rebuilt and
re-tagged from those commits; nothing in the erasure wave rewrote history.

The npm package is **not recoverable** and is not meant to be: the binding was absorbed
into CeraUI (`packages/srtla-send`) and is consumed there, so nothing republishes
`@ceralive/srtla-send`. npm never lets an unpublished version be published again, so the
old `2026.x` releases cannot come back; that was accepted when the erasure was
authorized.
