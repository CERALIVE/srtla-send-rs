# Upstream sync, 2026-11 (8db546c): triage ledger

Record of the second November 2026 merge of `irlserver/srtla_send` into the CERALIVE
fork, taken in two steps: `8db546c` first, then `b1cf1f2` on top (see "Second range"
below). The living contract is [`AGENTS.md`](../../AGENTS.md) and the docs it routes to;
this note records what crossed the boundary, why, and what is still open. The previous
sync is [`upstream-sync-2026-11.md`](upstream-sync-2026-11.md).

## Range

| Item | Value |
|---|---|
| Previous last-merged upstream SHA | `9e63d8fab313d876917102b0ae26823a45016d9e` |
| New last-merged upstream SHA | `8db546cb06f2d28a336e0a422f706ce79f515c95` |
| Commits in `9e63d8f..8db546c` | 81 (`git rev-list --count`), of which 2 are merges (`--merges --count`) |
| Fork merge commit | `3f887da` `chore: merge upstream irlserver/srtla_send 8db546c`, parents `4edec0d` (fork `main`) and `8db546c` |
| Second range | `8db546c..b1cf1f2`, 6 commits, of which 2 are merges |
| Second fork merge commit | `bfb0bbb` `chore: merge upstream irlserver/srtla_send b1cf1f2`, parents `84aa819` (this sync's branch) and `b1cf1f2` |

## Verdicts

Same vocabulary as the previous ledger: **ADOPT**, **ADOPT-WITH-FORK-FIX**,
**ALREADY-CONVERGENT**, **REJECT**, **MERGE**.

## Upstream commits

Oldest first (`git log --reverse --topo-order 9e63d8f..8db546c`).

| SHA | Subject | Author | Verdict | Evidence |
|---|---|---|---|---|
| `4ba1d31` | fix(stall): don't credit a link with delivery proof from an ACK that arrived on another link | Alex Lopez | ADOPT | Same fix as open fork PR #38 `a8f8dff`; taken as upstream wrote it. |
| `87c0304` | fix(stall): count a same-millisecond keepalive echo as delivery proof | Alex Lopez | ADOPT | Adopted as merged. |
| `c262c37` | fix(srtla-core): leave probes pending on foreign srtla acks | datagutt | ADOPT | Adopted as merged. |
| `dff19ee` | Merge pull request #28 from 036labs/fix/stall-latch-delivery-proof | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `f1fd6b3` | fix(srtla_send): bump tokio for RUSTSEC-2026-0386 and drop AsyncFd | datagutt | ADOPT-WITH-FORK-FIX | tokio 1.54.1 was already on the fork (`3c0381c`); upstream's `UdpSocket`-based `BatchUdpSocket` replaces the fork's `AsyncFd::register_with_interest` call site. |
| `06eb0a1` | chore(srtla_send): drop stale rand advisory ignore | datagutt | ADOPT | `deny.toml` ignore removed. `cargo deny` was not available on the merge host; CI verifies it. |
| `4d73538` | Merge pull request #29 from irlserver/fix/tokio-asyncfd-advisory | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `0d534fe` | fix(srtla-core): skip rtt samples from same-link resends | datagutt | ADOPT | Same `Option` packet-log design as fork `61f0d44`; upstream code and tests taken, the fork test `retransmitted_sequence_ack_preserves_rtt_floor_and_delivery_accounting` kept. |
| `5a9ae03` | fix(srtla_send): dial the receiver address in each uplink's family | datagutt | ADOPT-WITH-FORK-FIX | Family selection ported onto the spec-based pool: `create_connections_from_links` takes the answer set. A failed startup lookup dials nothing and waits for SIGHUP instead of exiting. The `AGENTS.md` edit rejected. |
| `f6dfa7f` | test(network-sim): wait for registered uplinks, not connected sockets | datagutt | ADOPT-WITH-FORK-FIX | Same intent as fork `88cca14`; the fork counter (REG3 lines plus `active=N`, over stdout and stderr) kept, upstream's unit test passes against it. |
| `883876b` | test(srtla_send): read sender logs from stdout in netns tests | datagutt | ALREADY-CONVERGENT | The fork already asserts on stdout (`bd04719`). |
| `9396e6b` | test(network-sim): run the sender binary cargo built for the test | datagutt | ADOPT-WITH-FORK-FIX | `find_srtla_send_binary` removed; the fork's twin stack and `tests/netns_twin.rs` now take `CARGO_BIN_EXE_srtla_send` too. |
| `352178a` | fix(srtla-core): skip rtt samples from retransmit-flagged copies | datagutt | ADOPT | Adopted as merged. |
| `46d3c99` | fix(srtla_send): don't re-home to an answer no uplink can reach | datagutt | ADOPT | `AGENTS.md` edit rejected. |
| `00c53cc` | Update .gitignore | datagutt | ADOPT | `/plans` added next to the fork entries. |
| `46f6b96` | docs(srtla_send): describe the current tree in agents.md | datagutt | REJECT | `AGENTS.md` only; the fork keeps its own. |
| `86c70a1` | docs(srtla_send): correct stall, timeout and control docs | datagutt | ADOPT | Adopted as merged. |
| `7277cf7` | docs(srtla-core): fix the conn timeout in the stall doc comment | datagutt | ADOPT | Adopted as merged. |
| `e42a142` | docs(srtla_send): drop the removed keyframe heuristic from docs | datagutt | ADOPT | Adopted as merged. |
| `6ebaa87` | docs(srtla_send): match the extended keepalive doc to the code | datagutt | ADOPT | Adopted as merged. |
| `78df3c1` | docs(srtla-protocol): drop the stale re-export note | datagutt | ADOPT | Adopted as merged. |
| `71b2755` | chore(srtla_send): drop the unused [test] table from cargo config | datagutt | ADOPT | Adopted as merged. |
| `80aff75` | fix(srtla-core): skip the best-path override while quality scoring is off | datagutt | ADOPT | Adopted as merged. |
| `8c6b934` | fix(srtla_send): cap the priority sidecar window | datagutt | ADOPT | Adopted as merged. |
| `302f4b9` | fix(srtla-core): keep a backed-off link_cc target at the floor | datagutt | ADOPT | Adopted as merged. |
| `208fb98` | fix(srtla_send): dial reloaded uplinks at the bond's pinned receiver address | datagutt | ADOPT-WITH-FORK-FIX | Pinned-address reload ported into the identity-based reload: `pinned_remotes` is taken before survivors are claimed, `reload_answers` feeds `create_missing`, and `forget_connections` runs in `Survivors::retire`. Upstream tests adapted to `specs_from_ips`. `AGENTS.md` edit rejected. |
| `4734f01` | fix(srtla_send): read control requests with a cancel-safe bounded reader | datagutt | ADOPT | Adopted as merged. |
| `aab1f0c` | fix(srtla_send): cap control socket subscriptions | datagutt | ADOPT-WITH-FORK-FIX | `subscribe` now returns `Option`; `tests/subscription_loom.rs` adapted. |
| `225c857` | perf(srtla_send): serialize each published event once | datagutt | ADOPT | The fork's Loom `cfg` imports kept; `json` import dropped. |
| `915b309` | fix(srtla_send): only unsubscribe ids the connection owns | datagutt | ADOPT | Adopted as merged; `get_capabilities` kept beside the upstream dispatch. |
| `5ab397a` | fix(srtla_send): create the control socket owner-only and never replace non-sockets | datagutt | ADOPT | Adopted as merged. |
| `ff224ef` | fix(srtla_send): time out and cap prometheus scrape connections | datagutt | ADOPT | Adopted as merged. |
| `5399d5d` | test(srtla_send): cover the unix control socket transport | datagutt | ADOPT | Adopted as merged. |
| `b69c511` | docs(srtla_send): document control socket limits and a private socket path | datagutt | ADOPT | Adopted as merged. |
| `ec90ef6` | docs(srtla_send): reword the subscribe doc comment | datagutt | ADOPT | Adopted as merged. |
| `c43a2d7` | test(srtla_send): fail netns tests on missing deps when required | datagutt | ADOPT | Adopted as merged. |
| `52ac081` | ci(srtla_send): run the netns suite against irlserver srtla_rec | datagutt | REJECT | CI only; the fork keeps its `ci.yml`. A netns lane against `srtla_rec` is a follow-up. |
| `8dbdb3a` | ci(srtla_send): check srtla-core visibility without test-internals | datagutt | REJECT | CI only; the fork keeps its `ci.yml`. |
| `68169d2` | ci(srtla_send): point the miri comment at src/net/batch_recv.rs | datagutt | REJECT | CI only; the fork keeps its `ci.yml`. |
| `9411b7c` | ci(srtla_send): check both receiver binaries and run every netns target | datagutt | REJECT | CI only; the fork keeps its `ci.yml`. |
| `2e950c9` | fix(srtla_send): log a total outage once instead of per dropped packet | datagutt | ADOPT | Adopted as merged. |
| `cb5fcef` | docs(srtla-core): say what has_connected selects | datagutt | ADOPT | Adopted as merged. |
| `201e948` | fix(srtla_send): rebuild recvmmsg pointers on every call | datagutt | ADOPT | Adopted as merged. Its three new pointer tests are not in the fork Miri lane yet. |
| `b60fb09` | test(srtla_send): correct the has_connected assertion message | datagutt | ADOPT | Adopted as merged. |
| `d52350b` | fix(srtla-core): reject a reg2 that does not echo our id half | datagutt | ADOPT | Adopted as merged. |
| `95df9f9` | fix(srtla-core): skip rtt samples from repeated probes on one link | datagutt | ADOPT | Replaces fork `61f0d44`'s rule that probe ACKs never sample RTT: a probe samples RTT unless its link probed the sequence twice. The fork's edit of `a_probe_survives_the_cumulative_ack_sweep_and_is_still_measurable` reverts to upstream's RTT assertion. |
| `54075fd` | fix(srtla_send): take srt-ack rtt samples only from the owning link's copy | datagutt | ADOPT | Adopted as merged. |
| `6a0112d` | feat(srtla-protocol): detect an srt caller starting a new session | datagutt | ADOPT | Adopted as merged. |
| `fdded82` | fix(srtla_send): reset ack state when the encoder starts a new srt session | datagutt | ADOPT | Adopted as merged. |
| `87b7f2b` | test(network-sim): inject every test stream from one source port | datagutt | ADOPT | Adopted as merged. |
| `b1879d8` | feat(srtla_send): add --listen-addr for the local srt listener | datagutt | ADOPT | `--listen-addr` defaults to `::`, the fork's previous bind, and the listener is still bound before the IP list is read. |
| `c1327dc` | fix(srtla_send): pin the srt client to its session | datagutt | ADOPT-WITH-FORK-FIX | `tests/session_bytes_integration.rs` and the twin traffic now inject from one encoder socket, since the pin drops other sources. |
| `291b727` | test(network-sim): send every netns injection from one port below the ephemeral range | datagutt | ADOPT-WITH-FORK-FIX | The fork stall-gate injector replaced by upstream's, which sends from `ENCODER_SOURCE_PORT`. |
| `b0c7644` | test(srtla_send): pin the registration grant drift on reload | datagutt | ADOPT | Adopted as merged. |
| `e45d8bd` | fix(srtla-core): key registration handshake state by connection id | datagutt | ADOPT-WITH-FORK-FIX | Probe loops in the fork's empty-pool startup and reload bootstrap keyed by `conn_id`; padded control sends kept. |
| `6b7eccc` | test(srtla-core): document how a late link is scored while the healthy link is capped | datagutt | ADOPT | Adopted as merged. |
| `4f19cd7` | chore(srtla_send): remove unused dependencies | datagutt | ADOPT-WITH-FORK-FIX | `assert_matches` stays a dev-dependency: the fork's bind-map tests use it. |
| `e73e12f` | chore(srtla-protocol): drop lint allows on public items | datagutt | ADOPT | Adopted as merged. |
| `dfc5af8` | chore(srtla_send): gate test-only re-exports on cfg(test) | datagutt | ADOPT | Fork re-exports for the bind-map and padding tests kept. |
| `280f587` | chore(srtla-core): drop dead_code allow on local_ip | datagutt | ADOPT | Adopted as merged. |
| `e3fb38e` | ci(srtla_send): pin third-party actions to commit shas | datagutt | REJECT | CI only; `build.yml` is not on the fork and `ci.yml` is the fork's. |
| `a88dd2e` | refactor(srtla-core): pass now into check_probing_complete | datagutt | ADOPT | Adopted as merged. |
| `066e6b4` | test(srtla_send): drive nak burst tests with explicit time | datagutt | ADOPT | Adopted as merged. |
| `81b6eac` | test(srtla_send): remove the wall-clock margin from the srt ack field test | datagutt | ADOPT | Adopted as merged. |
| `e8cb2f0` | docs(srtla_send): drop the probing clock exception from agents.md | datagutt | REJECT | `AGENTS.md` only. |
| `0ed81f1` | test(srtla_send): document send backpressure on a tiny loopback send buffer | datagutt | ADOPT | Adopted as merged. |
| `edb1907` | test(network-sim): send injected streams as srt data packets | datagutt | ADOPT | Adopted as merged. |
| `7928a76` | test(srtla_send): document how a repeated nak after a rerouted retransmit is charged | datagutt | ADOPT | Adopted as merged. |
| `ed49db6` | fix(srtla-core): skip the congestion charge for naks on retransmit copies | datagutt | ADOPT | `AGENTS.md` edit rejected. |
| `2dc11ac` | fix(srtla-core): judge stall rejoin on the recent rtt floor | datagutt | ADOPT | Adopted as merged. |
| `384ab3f` | test(network-sim): black out a stall-gate link in both directions | datagutt | ADOPT-WITH-FORK-FIX | Two-way blackhole kept together with the fork's 2000 packets/s load and log filter. |
| `12a89ae` | test(srtla_send): cover the latched rejoin with an inflated smoothed rtt | datagutt | ADOPT | Adopted as merged. |
| `1809321` | test(srtla_send): log nak age per link in the wire-loss test | datagutt | ADOPT | Adopted as merged. |
| `1dcb6b9` | test(srtla_send): arm the stall-gate netns test on live links | datagutt | ADOPT-WITH-FORK-FIX | `--stall-min-in-flight 8` and the Warming wait kept with the fork's load and log filter; `start_with_log_filter` takes the binary path. |
| `a866ff2` | ci(srtla_send): show netns test output on passing runs | datagutt | REJECT | CI only. |
| `f3a655d` | fix(srtla_send): flush older queued packets on other links before a full batch | datagutt | ADOPT-WITH-FORK-FIX | The ordering flush goes through the fork's `flush_connection(conn, &mut ConnIo, ..)`, so a device-bound send error still reaches the egress lifecycle. |
| `f3090b4` | test(network-sim): capture per-link loss and reorder in the wire-loss test | datagutt | ADOPT | Test and harness parts adopted; the `ci.yml` part rejected. |
| `83dfb05` | test(network-sim): classify naked packets by arrival in the wire-loss summary | datagutt | ADOPT | Adopted as merged. |
| `f898b92` | fix(srtla-core): charge naks only once the link's srtla acks show the loss | datagutt | ADOPT | The fork's flush timer arm keeps `&mut conn_io` and gains the pending-NAK expiry. |
| `6bde374` | docs(srtla_send): describe deferred nak charging | datagutt | ADOPT | `AGENTS.md` edit rejected. |
| `8db546c` | test(srtla_send): judge each wire-loss link by its share of the bond | datagutt | ADOPT-WITH-FORK-FIX | The share floor is added; the fork's absolute 0.5 Mbps per-link floor stays, so neither check is weakened. |

## Conflict resolutions

| File | Resolution |
|---|---|
| `.github/workflows/build.yml` | Deleted on the fork, modified upstream: stays deleted. |
| `.github/workflows/ci.yml` | Fork file retained byte-for-byte. |
| `.gitignore` | Union: fork `__pycache__/` and `/dist/` plus upstream `/plans`. |
| `AGENTS.md` | Fork file retained byte-for-byte. |
| `Cargo.toml` (auto-merged) | Upstream's unused-dependency removal taken except `assert_matches`, which the fork's bind-map tests use. |
| `Cargo.lock` | Fork lock (tokio 1.54.1, sha2, loom), pruned by cargo of the packages the removed dependencies pulled in. |
| `README.md` | Fork version line (4.2.0) and CERALIVE section retained; upstream flag and control documentation adopted. |
| `crates/network-sim/src/harness.rs` | Fork bounded output tail and `log_snapshot` kept, with upstream's `&self` accessors. Fork registration counter kept. Upstream's live-link wait, two-way impairment, pcap capture and binary-path parameter adopted. `start_with_log_filter` takes the binary path. `wait_for_connected_uplinks` and `find_srtla_send_binary` removed as upstream did. |
| `crates/network-sim/src/lib.rs` | Upstream export list. |
| `crates/network-sim/src/twin/stack.rs` (fork-only) | `TwinStack::start` takes the binary path; its traffic uses the shared SRT DATA injector. |
| `crates/srtla-core/src/connection/ack_nak.rs` | Upstream code and comments; the fork test `retransmitted_sequence_ack_preserves_rtt_floor_and_delivery_accounting` kept beside upstream's two tests. |
| `crates/srtla-core/src/connection/mod.rs` | Upstream comments. The fork's probe and keepalive doc edits from `61f0d44` revert with the behavior they described. |
| `src/control.rs` | Fork `get_capabilities` arm and ADR-003 `get_stats` tests kept; upstream subscription comment and tests adopted. |
| `src/net/batch_recv.rs` | Upstream `UdpSocket`-based socket. |
| `src/sender/connections.rs` | Fork spec-based, identity-claiming pool retained with upstream's family-aware dialing and reload pin: `apply_connection_changes` takes the resolver and registration manager, and `create_connections_from_ips`/`_from_links` take the answer set. Upstream `weight_for`/`apply_link_weights` stay rejected. Upstream tests ported. |
| `src/sender/housekeeping.rs` | Fork egress tick kept in the upstream `conn_id` loop; padded control sends kept. |
| `src/sender/mod.rs` | Fork `links` module, empty startup, padded probes and `&mut conn_io` flush kept. One receiver lookup for the bond, as upstream does, except that a failed lookup dials no uplink and waits for SIGHUP rather than exiting. Probe loops keyed by `conn_id`; pending-NAK expiry on the flush timer. |
| `src/sender/packet_handler.rs` | Upstream ordering flush routed through the fork's `flush_connection(conn, &mut ConnIo, ..)`. Upstream tests pass `&mut conn_io`. |
| `src/sender/rehome.rs` | Upstream `conn_id` probe loop with the fork's padded send. |
| `src/subscriptions.rs` | Fork Loom `cfg` imports kept; `json` import dropped as upstream did. |
| `src/tests/connection_tests.rs` | Union: fork padding tests and `conn_io_to` helper plus every upstream test. `a_probe_survives_the_cumulative_ack_sweep_and_is_still_measurable` is upstream's version again. |
| `src/tests/sender_tests.rs` | Upstream imports; `apply_connection_changes_forgets_removed_handshakes` uses `specs_from_ips`. |
| `tests/common/mod.rs` | Upstream `skip_or_fail`, `start_stack` and `inject_stream`; fork `start_stack_with_log_filter` added. The fork's sequenced injector is replaced by upstream's. |
| `tests/netns_stall_gate.rs` | Upstream two-way blackhole, `--stall-min-in-flight 8` and Warming wait, with the fork's 2000 packets/s load, log filter and stdout/stderr panic check. |
| `tests/netns_wire_loss.rs` | Fork fixed 4 Mbps load and absolute floor kept with upstream's share floor and capture analysis. |

## Adapted tests

Upstream tests that differ from `8db546c` after the merge, beyond the ones the
previous ledger already lists (`kill_returns_when_wrapper_and_inner_process_have_mismatched_groups`,
`periodic_flush_failure_recovers_the_link`, `successful_flush_registers_the_batch_and_stamps_last_sent`,
`test_shared_stats_empty_update`, `test_to_json_contains_expected_fields`,
`reload_reweights_existing_links_and_unnamed_links_fall_back_to_one`,
`weight_is_exported_per_link_in_metrics`, `test_apply_connection_changes_remove_stale`,
`test_pending_connection_changes`, and the classifier comment in
`a_link_that_is_late_never_earns_a_probation_window`). No assertion was removed.

| File | Test | Change |
|---|---|---|
| `src/sender/packet_handler.rs` | `retransmit_link_switch_moves_tracker_ownership`, `full_batch_flushes_older_packets_on_other_links_first`, `a_link_switch_below_the_threshold_does_not_flush` | Pass `&mut conn_io`. |
| `src/tests/connection_tests.rs` | `test_packet_tracking` | The fork's extra `contains_key` assertion kept. |
| `src/tests/sender_tests.rs` | `apply_connection_changes_forgets_removed_handshakes` | Reload through `specs_from_ips`; the fork keeps 192.168.1.10 by socket key, so the comment names the two links that leave. Both assertions unchanged. |
| `src/sender/connections.rs` | `reload_pin_*` (4) | The `reload` helper passes `specs_from_ips(new_ips, &[])`. Assertions unchanged. |
| `tests/netns_stall_gate.rs` | `test_stall_gate_latches_and_rejoins_after_dwell` | See the conflict table. |
| `tests/netns_wire_loss.rs` | `wire_loss_link_is_not_ratcheted_out_of_the_bond` | See the conflict table. |

Fork tests changed by the merge:

- `tests/session_bytes_integration.rs`: injects every burst from one encoder socket.
  With upstream's client pin (`c1327dc`), the old per-burst socket was a new source
  each time and its data was dropped. Assertions unchanged.
- `tests/subscription_loom.rs`: unwraps the `Option` that `subscribe` now returns.
- `src/tests/link_weight_tests.rs`: reloads pass a resolver and registration manager,
  and initial dials pass the receiver answer. Assertions unchanged.
- `tests/netns_twin.rs`: passes the cargo-built binary to `TwinStack::start`.
- `src/tests/connection_tests.rs`, `a_probe_survives_the_cumulative_ack_sweep_and_is_still_measurable`:
  the fork asserted that a probe ACK leaves RTT uninitialised. Upstream `95df9f9`
  makes a once-probed sequence a valid RTT sample, so the test asserts upstream's
  roughly 1000 ms RTT again, as it did before fork `61f0d44`. The delivery and
  in-flight assertions are unchanged.
- Fork tests whose bodies changed only through upstream's own API edits (`handle_nak`
  mode argument, `conn_id`-keyed registration, `SmallVec` pools) took upstream's
  edits as they were.

## Gate on the merge

On the pinned toolchain: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings` and
`cargo test --workspace --all-features` pass, except the two Linux `device_binding`
flush tests, which need a send from `[::1]` to be refused and fail the same way on
unchanged `main` on the merge host. The Loom lane builds and passes. `cargo audit` and
`cargo deny` were not installed on the merge host.

The privileged topology ran separately against a real irlserver `srtla_rec` built at
`877dc16` (the commit upstream's CI pins), with `NETNS_REQUIRED=1` so nothing could
skip: `netns_basic` 2, `netns_failure` 2, `netns_impairment` 3, `netns_scenario` 2,
`netns_stall_gate` 1, `netns_twin` 8 and `netns_wire_loss` 4, 22 passed and none failed.
That covers the stall-gate, wire-loss and twin-modem fixtures this merge changed.

Further checks on the merge plus the compatibility fix below:

- Miri, `cargo miri test --lib --no-default-features --features test-internals <filter>`
  on the pinned nightly: the lane's five filters and upstream `201e948`'s three new
  pointer tests (`pointers_follow_the_buffer_after_a_move_out_of_the_box`,
  `pointers_follow_both_buffers_after_a_swap`, `forged_packet_reads_correctly_after_a_move`)
  all pass. The CI lane still runs exactly the five filters its contract test pins;
  adding the three is a separate CI change.
- `cargo deny check advisories sources bans`: ok, including the advisory ignore upstream
  dropped from `deny.toml`. `licenses` fails the same way on unchanged `main` and is not
  part of this repository's gate. `cargo audit` was not run.
- Not run: the release and platform build matrix.

### Gate on the second step

On the branch with `b1cf1f2` merged and the docs for it, on the pinned toolchain:
`cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`
pass, and `cargo test --workspace --all-features` and `--features test-internals` pass
895 tests each (the two `device_binding` flush tests left out as above). The Loom lane
passes, as do the workflow, cache, authority, release-version, Debian-ordering and
doc-reference scripts.

The privileged topology against `srtla_rec` `877dc16`, with `NETNS_REQUIRED=1`:
`netns_arrival_reports` 1, `netns_basic` 2, `netns_failure` 2, `netns_impairment` 3,
`netns_scenario` 2, `netns_stall_gate` 1, `netns_twin` 8 and `netns_wire_loss` 4, 23
passed and none failed.

Miri ran again because `c831bf7` adds the socket generation to `src/net/batch_recv.rs`:
the lane's five filters and the three pointer tests from `201e948` pass. `Cargo.lock` and
`deny.toml` are unchanged by this step.

## Upstream co-author trailers

Upstream commits `4ba1d31` and `87c0304` carry co-author trailers that upstream added.
The merge keeps them verbatim, as the previous sync did for its range, because
rewriting them would change upstream SHAs and break the merge base. This was decided
for this sync before it was opened. Every commit the fork authors carries none, and
the second range adds no trailers.

## Compatibility fix on top of the merge

Before this sync an IPv4-mapped IPv6 source (`::ffff:a.b.c.d`) dialed the first
receiver answer and worked, because a dual-stack socket bound to it can send to an
IPv4 receiver. Upstream's per-family selection matched the written family, so such a
source was skipped against an A-only receiver and dialed an unreachable AAAA record
against a dual-stack one. The follow-up commit canonicalises the source for the socket
family, the answer match and the bind address in both the source-IP and the device
binder. Separate tests check: picking the IPv4 answer from an AAAA-first list and a
rebuild; a pinned reload that adds the mapped source with no lookup; the bound family and
delivery to an IPv4-only receiver; the device binder on `lo` where the kernel allows it.
Upstream has since merged the source-IP part (`722fe46`, `f263036`, in the second
range below); the device binder is the fork's.

## Second range: `8db546c..b1cf1f2`

Oldest first (`git log --reverse --topo-order 8db546c..b1cf1f2`).

| SHA | Subject | Author | Verdict | Evidence |
|---|---|---|---|---|
| `722fe46` | fix(srtla_send): treat an ipv4-mapped source as ipv4 | Paul Golmann | ALREADY-CONVERGENT | The source-IP half of the fork's compatibility fix `c4e7eec`, proposed upstream from it. The merge leaves the fork's `src/net/socket.rs` as it was and adds only a test to `src/sender/connections.rs`; the fork's device binder keeps its own canonicalisation. |
| `f263036` | test(srtla_send): cover ipv4-mapped sources on the wire and on reload | Paul Golmann | ADOPT | `reload_looks_up_ipv4_for_a_mapped_source_new_to_an_ipv6_bond` added beside the fork's tests. The fork's label assertion, its device-binder test and its longer doc comment on `reload_pin_dials_a_mapped_ipv4_source_at_the_bond_address` kept. |
| `783aab4` | Merge pull request #31 from openstreambox/osb/pr/mapped-ipv4-sources-irlserver | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |
| `c831bf7` | fix(srtla_send): ignore datagrams queued by an uplink's previous socket | Paul Golmann | ADOPT | Adopted as merged. |
| `a7c3f17` | feat(srtla_send): report per link the SRTLA ACK entries that arrived on it | Paul Golmann | ADOPT-WITH-FORK-FIX | The fork's `LinkStats` `Default` gains the four fields, and `link_arrival_tests` passes the fork's identities argument to `SharedStats::update`. The `AGENTS.md` edit rejected; the fields are recorded in the parity contract, the README telemetry paragraph and an `AGENTS.md` rule. |
| `b1cf1f2` | Merge pull request #30 from openstreambox/osb/pr/link-arrival-reports-irlserver | Thomas Lekanger | MERGE | `git show --cc` adds no content of its own. |

Conflicts: `AGENTS.md` kept the fork file byte for byte (upstream's only change is the
mapped-source sentence, which the fork's rules and parity contract already state).
`src/sender/connections.rs` took both sides' tests as above. Everything else merged
cleanly.

## Follow-ups

- Upstream `201e948` adds three pure pointer tests in `src/net/batch_recv.rs`
  (`pointers_follow_the_buffer_after_a_move_out_of_the_box`,
  `pointers_follow_both_buffers_after_a_swap`, `forged_packet_reads_correctly_after_a_move`);
  the fork Miri lane still runs the original five.
- Upstream's CI netns lane against `srtla_rec` (`52ac081`, `9411b7c`) is not adopted;
  the fork has no privileged CI lane.
- Add upstream `201e948`'s three pointer tests to the Miri lane and its contract test.
- Run `cargo audit` and the release/platform matrix in CI.
- A re-home whose answer lacks a family drops that family's uplinks from the bond
  (upstream `46d3c99` and `5a9ae03`); they return on the next IP-list reload.
