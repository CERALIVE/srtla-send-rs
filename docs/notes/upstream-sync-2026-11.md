# Upstream sync, 2026-11: triage ledger

Record of the November 2026 merge of `irlserver/srtla_send` into the CERALIVE fork.
The living contract is [`AGENTS.md`](../../AGENTS.md) and the docs it routes to; this
note records what crossed the boundary, why, and what is still open. The 2026-09 fork
record is [`upstream-hardfork-2026-09.md`](upstream-hardfork-2026-09.md).

## Range

| Item | Value |
|---|---|
| Previous last-merged upstream SHA | `df0b3938791ff24eced4aed8b29e3d49d0efb639` (v4.0.1, the fork base) |
| New last-merged upstream SHA | `9e63d8fab313d876917102b0ae26823a45016d9e` |
| Commits in `df0b393..9e63d8f` | 33 (`git rev-list --count`), of which 8 are merges (`--merges --count`) |
| Fork merge commit | `38cd628` `chore: merge upstream irlserver/srtla_send 9e63d8f`, parents `e4b6c3d` (fork `main`) and `9e63d8f` |

## Verdicts

- **ADOPT**: taken as upstream wrote it.
- **ADOPT-WITH-FORK-FIX**: taken, with a fork-side resolution so the parity contract holds.
- **ALREADY-CONVERGENT**: the fork already carries the same change under its own SHA.
- **REJECT**: not taken; the fork has its own equivalent or no use for it.
- **MERGE**: an upstream merge commit; it carries no content of its own.

## Upstream commits

Oldest first (`git log --reverse --topo-order df0b393..9e63d8f`).

| SHA | Subject | Author | Verdict | Evidence |
|---|---|---|---|---|
| `334793f` | recovery: don't let receiver traffic refresh a link awaiting REG3 | Simon | ADOPT | Adopted as merged. |
| `4802637` | Merge pull request #19 from ziggy6792/recovery-limbo-standalone | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `2063034` | fix(srtla_send): apply the --config toml file | datagutt | ADOPT-WITH-FORK-FIX | `--config` applied after `-v` and before `--dry-run`, so `--dry-run` validates it; the probe-before-logging order is kept. Pinned by `8f04681`. |
| `9ef77bb` | Merge pull request #21 from irlserver/fix/toml-config | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `a70fd4a` | fix(ci): move off yanked chacha20 0.10.1 | datagutt | ALREADY-CONVERGENT | Same change as fork `9c6e35c`. |
| `a93b274` | fix(network-sim): bound namespace-process teardown to the namespace's own PIDs | datagutt | ALREADY-CONVERGENT | Same change as fork `4f258c7`; the fork side of `crates/network-sim/src/harness.rs` is kept. |
| `4a1ba9f` | build: gate mimalloc behind a default-on feature | datagutt | ALREADY-CONVERGENT | Same change as fork `f0e33ab`. |
| `bb10171` | feat(srtla_send): exit cleanly on SIGTERM/SIGINT | datagutt | ADOPT-WITH-FORK-FIX | Fork shutdown hooks (stats-file unlink) kept; upstream `tests/signal_shutdown.rs` adopted unchanged. |
| `f731164` | Merge pull request #20 from irlserver/port/ceralive-2026-09 | Thomas Lekanger | MERGE | Upstream PR #20, its port of CERALIVE's 2026-09 work. `git show --cc` adds no content of its own. |
| `d75c271` | fix(srtla-core): retry a timed-out link every second before slowing down | datagutt | ADOPT | Adopted as merged. |
| `8828370` | Merge pull request #22 from irlserver/fix/fast-reconnect-retry | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `b235610` | chore(srtla_send): remove stale changelog | datagutt | ADOPT | `CHANGELOG.md` deletion adopted. |
| `e17e59f` | chore(srtla_send): enforce lf line endings | datagutt | ADOPT | `.gitattributes` adopted. |
| `b252821` | build(srtla_send): pin stable toolchain, keep nightly for rustfmt | datagutt | ADOPT-WITH-FORK-FIX | `xlint`/`xtest` aliases adopted; the stable toolchain pin and the `AGENTS.md` edits rejected (fork keeps `nightly-2026-06-12`). |
| `cdaeac4` | chore(srtla_send): fix spelling flagged by codespell | datagutt | ADOPT | Adopted as merged. |
| `3ea74ce` | ci(srtla_send): lint and test the whole workspace | datagutt | ADOPT-WITH-FORK-FIX | Intent ported as fork `230fc81` (whole-workspace lint and test); upstream's `ci.yml` restructure and codespell lane not adopted. |
| `f708c34` | ci(srtla_send): build debs against glibc 2.27 with cargo-zigbuild | datagutt | REJECT | zigbuild glibc-2.27 debs; the fork ships the Bookworm `GLIBC_2.36` package `srtla` through `ci/build-deb.sh`. |
| `76681f9` | ci(srtla_send): create draft releases from version tags | datagutt | REJECT | Draft releases; the fork publishes its release and dispatches APT. |
| `617df3f` | chore(srtla_send): add issue templates | datagutt | REJECT | Issue templates; issues are not used on the fork. |
| `c3f5291` | Merge pull request #23 from irlserver/ci/release-pipeline | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `3299825` | chore(srtla_send): bump version to 4.1.0 | datagutt | ADOPT-WITH-FORK-FIX | Fork keeps 4.1.0 through the merge; the version moves to 4.2.0 in its own release commit. |
| `c86a3e1` | fix(srtla_send): forward each srt ack and nak to the client once | datagutt | ADOPT | Adds `src/sender/client_dedup.rs` and its tests unchanged. |
| `ca2d033` | feat(classic): per-link weights from the IPs file (Moblin connection priorities) | Simon | ADOPT-WITH-FORK-FIX | Weight travels on `UplinkSpec`; IP-keyed `weight_for`/`apply_link_weights` rejected; bind-map parser parity in `ae407fe`; pinned by `c7a46c9`. |
| `5f2e081` | Merge pull request #24 from ziggy6792/link-weights | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `e40e1fa` | fix(srtla-core): window the rtt floor by time, not sample count | datagutt | ADOPT | Adds `crates/srtla-core/src/windowed_min.rs` unchanged. |
| `a158667` | fix(srtla-core): let a stall-latched link rejoin on keepalive echoes | datagutt | ADOPT | Adopted as merged. |
| `6076104` | docs(srtla-core): state that the windowed min expires only on update | datagutt | ADOPT | Adopted as merged. |
| `e7583d9` | fix(srtla_send): read the srt ack field as the first sequence still owed | datagutt | ADOPT | Adopted as merged. |
| `f6417e5` | Merge pull request #26 from irlserver/fix/rtt-min-window-and-stall-rejoin | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `59a7d8b` | Fix Debian build badge link in README | Thomas Lekanger | REJECT | Badge points at upstream `build.yml`; the fork keeps its CI and Release badges. |
| `4bd3f3f` | fix(priority): don't let the best-path override pick the lowest index on a tie | Simon | ADOPT | Adopted as merged. |
| `02dd232` | fix(priority): apply the best-path override only in enhanced mode | Simon | ADOPT | Adopted as merged. |
| `9e63d8f` | Merge pull request #27 from ziggy6792/classic-mode-retransmit-scheduling | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |

## Fork commits on this branch

On top of fork `main` `e4b6c3d`:

- `38cd628` `chore: merge upstream irlserver/srtla_send 9e63d8f`: the `--no-ff` merge, with the conflict resolutions below.
- `c7a46c9` `test(sender): pin link weights on the spec-based uplink pool`: tests that upstream's IPs-file weights reach the fork's spec-based pool, survive a weight-only reload without re-registration, and stay out of socket identity.
- `8f04681` `test(cli): validate --config under --dry-run`: tests that `--dry-run` rejects an unknown `--config` key and accepts a valid one without binding.
- `88cca14` `test(netns): wait for registered uplinks, not connected peers`: fixes a harness defect that predates the merge. `tests/common/mod.rs` waited for connected UDP peers, but uplink sockets are deliberately unconnected, so every netns test using `wait_until_ready` failed whenever it really ran. Readiness now waits, with the same 15 s bound, for every uplink to finish REG3 registration.
- `ae407fe` `fix(bind-map): parse weighted IPs-file lines like the sender does`: the bind-map sidecar parser and coherence check now share the sender's `parse_ip_line`, so a weighted line means the same thing to both.
- `3c0381c` `fix(deps): move tokio past RUSTSEC-2026-0386`: tokio 1.53.1 to 1.54.1, with the `AsyncFd::register_with_interest` call site in `src/net/batch_recv.rs`.
- `230fc81` `ci: lint and test the whole workspace`: clippy and the test suite now cover every workspace crate, the intent of upstream `3ea74ce`.

## Conflict resolutions

| File | Resolution |
|---|---|
| `.github/workflows/ci.yml` | Fork file retained byte-for-byte; the whole-workspace CI change landed separately as `230fc81`. |
| `.github/workflows/release.yml` | Fork file retained byte-for-byte. |
| `AGENTS.md` | Fork file retained byte-for-byte. |
| `Cargo.toml` | Fork sha2 0.11 and mimalloc decision comment retained; version remains 4.1.0; pins unchanged. |
| `README.md` | Fork CI and Release badges retained; remaining auto-merged upstream text adopted, then audited in the commit that adds this ledger. |
| `crates/network-sim/src/harness.rs` | Fork file retained byte-for-byte, including bounded teardown and output-tail handling. |
| `rust-toolchain.toml` | Fork nightly-2026-06-12, components, and target retained byte-for-byte. |
| `src/lib.rs` | Fork allocator decision comment retained. |
| `src/main.rs` | Fork Loom cfg, allocator cfg, probe-before-logging, dry-run and telemetry cleanup retained. Upstream explicit ArgMatches/config precedence applied at the existing parse position. Config validation precedes dry-run. Upstream Cli impl and CLI unit tests gated out under Loom. |
| `src/sender/connections.rs` | Fork spec-based pool, socket-key dedup and link-id claims retained. Weights travel positionally on UplinkSpec; claimed survivors update both core weight and stored I/O spec in place. Creation sets each spec's weight. IP-keyed weight_for/apply_link_weights rejected. Upstream removal of premature reconnect-success reset preserved. |
| `src/sender/mod.rs` | Fork LinkSource, empty startup, off-loop mapped reload, shutdown hooks and stats identity retained. Upstream client dedup and weighted legacy startup/reload adopted. Weight descriptions use line positions, never first IP match. |
| `src/tests/connection_tests.rs` | Union retained: fork control_packet_padded_to_32b and data_not_padded plus every upstream test. |
| `src/tests/sender_tests.rs` | Spec-based reload arguments retained and adapted to specs_from_ips(ips, weights); legacy creation passes empty weights. |

## Rejected files

Removed from the merge result; the fork keeps its own CI, release and packaging:

- `.github/workflows/build.yml`
- `scripts/package-deb.sh`
- `scripts/version.sh`
- `scripts/changelog.sh`
- `RELEASING.md`
- `.github/release-notes-header.md`
- `.github/ISSUE_TEMPLATE/bug_report.yml`
- `.github/ISSUE_TEMPLATE/config.yml`
- `.github/ISSUE_TEMPLATE/feature_request.yml`
- `.github/ISSUE_TEMPLATE/question.yml`
- `.codespellrc`

## Adapted upstream tests

At the original merge review, upstream tests that met the fork's contract were
adapted, never dropped; names were unchanged and no assertion was removed. All 570
`#[test]`/`#[tokio::test]` functions under `src/`, `crates/` and `tests/` at `9e63d8f`
were compared body by body with `38cd628`; the nine differences are listed below.
Later fork commits changed existing test bodies: the classifier comment, packet-log
timestamp assertion, probe RTT expectation, and stall/wire-loss fixtures. The released
tree therefore does not match `38cd628` for every test body; the follow-ups below
describe the changed expectations and fixture limits.

### Adapted in the merge

Before is upstream `9e63d8f`; after is the merge `38cd628`.

| File | Test | Before | After |
|---|---|---|---|
| `src/tests/link_weight_tests.rs` | `reload_reweights_existing_links_and_unnamed_links_fall_back_to_one` | Calls apply_link_weights on connections for IPs a,b with weights [10,1], [1,1], [1,7], and []. | Calls apply_connection_changes through specs_from_ips with exactly those IPs and weights and a real ConnIoMap. Expected pairs remain exactly (10,1), (1,1), (1,7), (1,1). |
| `src/tests/link_weight_tests.rs` | `weight_is_exported_per_link_in_metrics` | SharedStats::update(conns, classic, None, None). | SharedStats::update(conns, classic, None, None, None), preserving the fork identity parameter. Both metrics assertions unchanged. |
| `src/tests/sender_tests.rs` | `test_apply_connection_changes_remove_stale` | apply_connection_changes(&mut connections, &mut conn_io, &new_ips, &[], "127.0.0.1", 8080, ...). | apply_connection_changes(&mut connections, &mut conn_io, &crate::sender::specs_from_ips(&new_ips, &[]), "127.0.0.1", 8080, ...). Assertions unchanged. |
| `src/tests/sender_tests.rs` | `test_pending_connection_changes` | PendingConnectionChanges { new_ips: Some(SmallVec [192.168.1.100]), new_weights: SmallVec::new(), .. }; asserts changes.new_ips.is_some(). | PendingConnectionChanges { new_links: Some(crate::sender::specs_from_ips(&[192.168.1.100], &[])), .. }; asserts changes.new_links.is_some(). The receiver_host and receiver_port assertions are unchanged. |

### Carried unchanged from fork `main`

These upstream tests already had a fork form at `e4b6c3d`, and the merge kept it byte for
byte. Before is upstream `9e63d8f`; after is `38cd628`, identical to `e4b6c3d`.

| File | Test | Before | After |
|---|---|---|---|
| `crates/network-sim/src/harness.rs` | `kill_returns_when_wrapper_and_inner_process_have_mismatched_groups` | NamespaceProcess { .., stdout: Arc<Mutex<Vec>>, stderr: Arc<Mutex<Vec>> }. | NamespaceProcess { .., stdout_tail: new_output_tail(), stderr_tail: new_output_tail(), drain_threads: vec![] }, the fork's bounded output-tail harness. Assertions unchanged. |
| `src/sender/packet_handler.rs` | `periodic_flush_failure_recovers_the_link` | flush_all_batches(&mut connections, &conn_io, &mut seq_tracker). | flush_all_batches(&mut connections, &mut conn_io, &mut seq_tracker). Assertions unchanged. |
| `src/sender/packet_handler.rs` | `successful_flush_registers_the_batch_and_stamps_last_sent` | let conn_io = ..; flush_all_batches(&mut connections, &conn_io, &mut seq_tracker). | let mut conn_io = ..; flush_all_batches(&mut connections, &mut conn_io, &mut seq_tracker). Assertions unchanged. |
| `src/stats.rs` | `test_shared_stats_empty_update` | stats.update(&[], &config, None, None). | stats.update(&[], &config, None, None, None), the fork identity parameter. Assertions unchanged. |
| `src/stats.rs` | `test_to_json_contains_expected_fields` | Asserts the JSON contains "mode", "active_links", "total_window" and "links". | Same assertions plus one more: the JSON contains "session_bytes_sent". |

### Fork tests replaced by upstream's version

Before is fork `e4b6c3d`; after is `38cd628`, which takes upstream's `--config` parser.

| File | Test | Before | After |
|---|---|---|---|
| `src/toml_config.rs` | `test_defaults` | Concrete parser defaults: enhanced mode, false no_quality, and rtt_velocity_gate 2.0. | Retained baseline name. Asserts None for all six supported config keys; defaults now belong to CLI precedence. The retired rtt_velocity_gate key is explicitly rejected in test_full_toml. |
| `src/toml_config.rs` | `test_partial_toml` | Parses mode classic and rtt_velocity_gate 3.5; asserts no_quality defaults false. | Retained baseline name. Parses mode classic and supported conn_timeout_ms 3500; asserts their exact Some values and all four omitted keys remain None. Retired key rejection is tested. |
| `src/toml_config.rs` | `test_full_toml` | Parses mode classic, no_quality true and eight retired tuning keys; checks mode, no_quality and warming_rtt_probes 3. | Retained baseline name. Checks all six supported keys exactly and individually rejects each of the eight retired keys by name. All four new upstream TOML tests also remain. |

## Supply chain

Upstream is still red on RUSTSEC-2026-0386 at `9e63d8f` (`Cargo.lock` pins tokio 1.53.1).
The fork fixed it in `3c0381c` (tokio 1.54.1) without a new `[[advisories.ignore]]` entry;
`cargo audit` and `cargo deny check advisories sources` pass.

## Board bench

The November 10 S2 run used a single-uplink Rock board. Both 4.1.0 and 4.2.0
reported `timed_out_during_drop=false` through the 30 s blackhole. After fault
removal, polls showed `connected=false`, `timed_out=true` and `active_links=0`
at about 0.5 s, then `connected=true` again at about 2.5 s after removal.
Sender logs confirm socket reconnection and re-registration; receiver flow resumed
at about 4.0 s in both arms. The during-drop timeout condition was not met, but
single-uplink timeout/reconnect recovery was exercised after the outage.
Multi-link, cellular and handover cases remain unqualified on hardware.

## Recorded exception: upstream co-author trailers

Approved by the owner on 2026-11-08, against the authorship rule in the workspace
[`workspace-reference.md`](https://github.com/CERALIVE/ceralive/blob/master/docs/governance/workspace-reference.md)
(line 448). Upstream commits `ca2d033`, `4bd3f3f` and `02dd232` carry co-author
trailers that upstream added. They're imported verbatim, because rewriting them would change
upstream SHAs and break the merge base. Every commit the fork authors carries none.

Upstream `d75c271` also carries Simon's human co-author trailer, preserved as upstream history.

## Tags and delivery

The fetch auto-followed upstream's tags `v4.0.0` and `v4.0.1` into the local repo.
At that fetch, `origin` carried only `v4.1.0`. PR #34 subsequently merged as
`1b5febb`, and `v4.2.0` was published. Never push tags from a sync worktree.

Root pin PR #206 merged as `947e383`. Image pipeline PR #247 merged as `36aef71`,
selecting srtla 4.2.0 and moving CeraUI to `v2026.11.2`.

## Follow-ups

The Karn RTT correction rejects samples from retransmitted sequences
and duplicate probes while retaining all ACK accounting. It fixes a deterministic
RTT-attribution bug: a late ACK for an original copy was measured against the
retransmit's send time. One such sample pinned the link's RTT floor down for the
30 s minimum window introduced by upstream `e40e1fa`, potentially excluding a
healthy link. The regression is
`retransmitted_sequence_ack_preserves_rtt_floor_and_delivery_accounting` in
`crates/srtla-core/src/connection/ack_nak.rs`. `packet_log` retains ambiguous
entries without an RTT timestamp; existing ACK/NAK/sweep/recovery removal still
owns their lifetime. Probe ACKs now prove delivery only; held-out links get RTT
from keepalive echoes, roughly once per 4 s. Readmission with only that RTT source
is untested.

The wire-loss caller now uses equal 4000 kbps bounds, below either 6 Mbps path.
Its former adaptive 500–16000 kbps bounds stopped near 1.7 Mbps total because the
low-water mark counts unacknowledged packets. Fixed load alone also passed on old
main, so this fixture does not by itself guard Karn or exercise adaptive-demand
recovery. The 45-second run, 10-second warm-up and strict >500000/>300000 bps
assertions are unchanged.

Two campaigns each interleaved 40 runs across five states, eight runs per state,
with the same fixed 4000/4000 kbps caller. With the harness's stock receiver
(Arch `srt-live-transmit`, `lossmaxttl=0`, no reorder freeze), the clean 0%-loss
link logged several times more NAKs than the 2%-loss link: that receiver NAKs every
cross-link reordering, so these counts do not meaningfully measure per-link loss.
With a production-like receiver (CeraLive libsrt `1.5.6+ceralive.1`,
`lossmaxttl=200`, `reorderfreeze=1`), the lossy link had more NAKs in 40/40 runs.
Per-state medians ranged from 12–92 on the clean link and 112–270 on the lossy
link. The pre-Karn starved-lossy-link failure (43 kbps) still occurred in 1/8 runs
without Karn and 0/8 with it. These are small samples: eight per state on one host.

Changed test expectations:
- `test_packet_tracking`: a positive timestamp remains required; it is now
  explicitly `Some(positive)` in the packet log.
- `a_probe_survives_the_cumulative_ack_sweep_and_is_still_measurable`: the probe
  must still match and stamp delivery proof after the sweep, but its former
  approximately 1000 ms RTT expectation becomes an uninitialized RTT estimator.
  The ACK cannot distinguish duplicate copies. In-flight and delivery assertions
  are retained; the baseline test name is retained.
- **Tracked known risk: rare stall-gate fast-engagement flake.**
  The fixture fix in `bd04719` reduced the flake but did not make
  `netns_stall_gate` reliable. It can still rarely fail at `silence pull engaged`
  because the dead link's backlog may stay below `--stall-min-in-flight` (32).
  Pinning and explicit precondition work remain follow-ups.
- **Wire-loss regression: `tests/netns_wire_loss.rs::wire_loss_link_is_not_ratcheted_out_of_the_bond`.**
  It failed 1 run in 3 both before the bind-map fix (at `88cca14`) and after it.
  The RTT correction above addresses retransmission-copy ambiguity; the original
  topology, measurement window and assertions remain the acceptance gate, with
  the fixed offered load described above.
- **Harness pitfall: binary lookup.** `find_srtla_send_binary` in
  `crates/network-sim/src/harness.rs` falls back to the `srtla_send` on `PATH`, without a
  warning, when `<repo>/target` has no binary. With an external `CARGO_TARGET_DIR`, the
  tests can exercise an installed older sender.
- **Harness pitfall: silent self-skip.** The netns tests skip themselves and report as
  passed in about 0 s when `sudo` is unavailable, for example under systemd
  `NoNewPrivileges=yes`. Sandboxed and CI runs therefore don't really exercise them.
- **Codespell lane.** Upstream's `.codespellrc` and spell-check job were not adopted.
- **MSRV lane.** `Cargo.toml` declares `rust-version = "1.88"`, but no fork lane builds on it.
- **Newer upstream commits.** Anything on upstream after `9e63d8f` is for the next sync.
- **Completed CeraUI CI capture bump.** CeraUI PR #512 merged as `3e1ff5e`,
  moving the sender binary and captures to 4.2.0 in `.github/workflows/build-check.yml`,
  `scripts/ci/build-check-contract.mjs` and `docs/BUILD_PIPELINE.md` (all in `CERALIVE/CeraUI`).

### Known follow-ups

- Make the fallback scan in `src/sender/packet_handler.rs` prune only: no delivery
  proof, window updates or RTT samples on links the ACK did not arrive on.
- Exclude duplicate-probed sequences from cumulative RTT ownership.
- Base stall detection on data sent since the last delivery proof rather than
  instantaneous in-flight counts; pin the stall fixture's backlog precondition.
- Use production-like receiver options (`lossmaxttl=200`, `reorderfreeze=1`) in
  the wire-loss test and harness.
- Repair adaptive caller pacing and cover adaptive-demand recovery.
- Test held-out link readmission with keepalive-only RTT.
