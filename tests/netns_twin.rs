//! Duplicate-IP twin-modem scenarios over real network namespaces.
//!
//! Everything the bind-map exists for only becomes observable when two uplinks
//! genuinely share one source address. Unit tests can drive the state machines
//! with an injected resolver, but they cannot show that the kernel actually
//! sends two same-address sockets out of two different interfaces, that a
//! replugged device hands back a different ifindex, or that a route-less
//! interface blackholes traffic while `sendto` keeps reporting success. These
//! do.
//!
//! Each scenario runs in its own set of namespaces (sender + one NAT carrier per
//! twin + receiver), so nothing here touches the host's addresses, routes, or
//! netfilter rules, and every namespace is torn down on drop even if a test
//! panics mid-way. Self-skips without netns privileges, `srtla_rec`,
//! `srt-live-transmit`, `iptables`, or `python3`.
//!
//! allow: SIZE_OK — one `cargo test --test` target is one file by construction,
//! and `scripts/netns_test_gate.sh` registers targets, not scenarios. Splitting
//! these eight independent scenarios across files would multiply the gate's
//! serial per-target budget without making any one of them easier to read. The
//! reusable machinery already lives in `network-sim`'s `twin` module; what is
//! left here is Given/When/Then assertions and nothing else.

mod common;

use std::thread::sleep;
use std::time::Duration;

use network_sim::check_binary;
use network_sim::twin::{Mapping, TwinRow, TwinStack};

/// Registration is a handful of round trips, but these targets run in parallel
/// on a loaded machine, so the gate is generous rather than tight.
const REGISTERED: Duration = Duration::from_secs(45);
/// One housekeeping tick detects an egress change; allow for scheduling noise.
const TICK_DETECTED: Duration = Duration::from_secs(20);
/// `CONN_TIMEOUT` is 15 s, and recovery needs a REG cycle on top.
const ACK_TIMEOUT: Duration = Duration::from_secs(60);
/// The periodic status log runs every 30 s; two intervals covers a bad landing.
const STATUS_LOGGED: Duration = Duration::from_secs(75);

/// Enough per-link headroom that neither twin can absorb the whole stream, and
/// enough delay that keepalive round trips produce a non-zero RTT sample.
const LINK_RATE_KBIT: u64 = 2000;
const LINK_DELAY_MS: u32 = 25;
const OFFERED_PACKETS_PER_SEC: u32 = 2000;

fn skip_without_twin_deps() -> bool {
    if common::skip_without_deps() {
        return true;
    }
    for tool in ["iptables", "python3"] {
        if check_binary(tool).is_none() {
            eprintln!("Skipping: system tool '{tool}' not found");
            return true;
        }
    }
    false
}

/// Start a bonded twin pair, capped so the bond has to use both links.
fn started(test_name: &str, mapping: Mapping) -> TwinStack {
    common::build_srtla_send();
    let stack = TwinStack::start(test_name, &["twin-a", "twin-b"], mapping).expect("start stack");
    stack
        .shape_all(LINK_RATE_KBIT, LINK_DELAY_MS)
        .expect("cap the twins");
    stack
}

fn carried_bytes(stack: &mut TwinStack, seconds: u64) -> Vec<u64> {
    stack
        .carried_bytes(OFFERED_PACKETS_PER_SEC, seconds)
        .expect("measure carried bytes")
}

fn count_lines_naming(log: &str, needle: &str) -> usize {
    log.lines().filter(|line| line.contains(needle)).count()
}

// ---------------------------------------------------------------------------
// Scenario 1 — two interfaces, one IP, both bonded
// ---------------------------------------------------------------------------

#[test]
fn twin_modems_on_one_ip_both_register_and_carry_traffic() {
    if skip_without_twin_deps() {
        return;
    }

    // Given: two uplinks that both present 10.30.9.1, each behind its own NAT
    // carrier — the shape two identical HiLink dongles take on a real board.
    let mut stack = started("tw_carry", Mapping::BindMap);

    // When: the sender is given the mapping that tells them apart.
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("both twins must register");
    let startup = stack.sender_log();

    // Then: the mapping is in force and each identity is pinned to its own
    // interface — not collapsed onto one socket by their shared address.
    assert!(
        startup.contains("bind-map active (mapped)"),
        "the sidecar must be applied:\n{startup}"
    );
    for idx in 0..2 {
        assert!(
            startup.contains(&format!("added uplink {}", stack.label(idx))),
            "twin {idx} must come up on its own interface:\n{startup}"
        );
    }

    // And: both twins actually carry the stream at the same time.
    let carried = carried_bytes(&mut stack, 8);
    for (idx, bytes) in carried.iter().enumerate() {
        assert!(
            *bytes > 300_000,
            "twin {idx} carried only {bytes} bytes; the bond is not using it (carried: \
             {carried:?})"
        );
    }
}

#[test]
fn without_a_bind_map_both_twins_egress_through_one_interface() {
    if skip_without_twin_deps() {
        return;
    }

    // Given: the identical topology, but the legacy IP-only launch.
    let mut stack = started("tw_legacy", Mapping::Legacy);

    // When: the sender identifies uplinks by source IP alone, so nothing tells
    // it which of the two interfaces each row meant.
    stack
        .wait_for_registered(1, REGISTERED)
        .expect("the bond must register");
    let startup = stack.sender_log();

    // Then: no socket is device-bound. Both rows dial out of whichever
    // interface the routing table prefers.
    assert!(
        !startup.contains(&format!("added uplink {}", stack.label(0))),
        "the legacy path must not device-bind anything:\n{startup}"
    );

    // And: the second modem carries nothing at all. That is the regression the
    // bind-map exists for, and asserting it here is what makes the twin-bonding
    // scenario above falsifiable rather than decorative — the same topology
    // yields one live link without the mapping and two with it.
    let carried = carried_bytes(&mut stack, 6);
    assert!(
        carried[0] > 100_000,
        "the routed interface must carry the stream (carried: {carried:?})"
    );
    assert!(
        carried[1] < 5_000,
        "the second twin must be dead weight without a mapping (carried: {carried:?})"
    );
}

// ---------------------------------------------------------------------------
// Scenario 2 — reload add / remove / re-add
// ---------------------------------------------------------------------------

#[test]
fn a_reload_removes_and_re_adds_a_twin_under_a_stable_link_id() {
    if skip_without_twin_deps() {
        return;
    }

    // Given: a bonded twin pair.
    let stack = started("tw_reload", Mapping::BindMap);
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("both twins must register");
    let rows: Vec<TwinRow> = stack.rows().to_vec();

    // When: the writer republishes a pair naming only the first twin.
    stack.reload(&rows[..1]).expect("publish the shrunk pair");

    // Then: the second uplink leaves the pool.
    let after_removal = stack
        .wait_for_log("removed 1 stale connection(s)", TICK_DETECTED)
        .expect("the dropped twin must be retired");
    let readds_before =
        count_lines_naming(&after_removal, &format!("added uplink {}", stack.label(1)));

    // When: it is republished under the SAME opaque identity.
    stack.reload(&rows).expect("publish the restored pair");

    // Then: it comes back as that identity, on its own interface, and the bond
    // returns to two registered uplinks.
    let after_readd = stack
        .wait_for_log("added 1 new connection(s)", TICK_DETECTED)
        .expect("the returning twin must be added");
    assert!(
        count_lines_naming(&after_readd, &format!("added uplink {}", stack.label(1)))
            > readds_before,
        "the twin must return under link_id twin-b on its own interface:\n{after_readd}"
    );
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("the bond must be whole again");
}

#[test]
fn a_degraded_reload_retains_the_mapped_twin_pool() {
    if skip_without_twin_deps() {
        return;
    }

    // Given: a bonded twin pair running on an applied mapping.
    let stack = started("tw_degrade", Mapping::BindMap);
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("both twins must register");
    let before: Vec<u64> = (0..2)
        .map(|idx| stack.topo.tx_bytes(idx).expect("read tx_bytes"))
        .collect();

    // When: the sidecar becomes unreadable and the sender is told to reload.
    stack
        .publisher
        .corrupt_sidecar()
        .expect("corrupt the sidecar");
    stack.sighup().expect("signal the reload");

    // Then: the live bond is retained rather than falling back to legacy, which
    // would strip per-interface egress pinning off both twins mid-stream.
    let log = stack
        .wait_for_log(
            "bind-map degraded: malformed (retained_last_valid)",
            TICK_DETECTED,
        )
        .expect("the degradation must be named");
    assert!(
        !log.contains("removed 1 stale connection(s)")
            && !log.contains("removed 2 stale connection(s)"),
        "a degraded reload must not tear a live bond down:\n{log}"
    );

    // And: both twins are still on the wire afterwards.
    sleep(Duration::from_secs(5));
    for idx in 0..2 {
        let carried = stack.topo.tx_bytes(idx).expect("read tx_bytes") - before[idx];
        assert!(
            carried > 100,
            "twin {idx} went silent across the degraded reload ({carried} bytes)"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 3 — file-order swap under a stable link_id
// ---------------------------------------------------------------------------

#[test]
fn a_file_order_swap_keeps_each_link_id_on_its_own_interface() {
    if skip_without_twin_deps() {
        return;
    }

    // Given: a bonded twin pair. Both IP lines are identical, so swapping the
    // sidecar rows changes each link's FILE POSITION and nothing else — which
    // is exactly the case legacy positional identity gets wrong.
    let stack = started("tw_swap", Mapping::BindMap);
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("both twins must register");
    let rows: Vec<TwinRow> = stack.rows().to_vec();
    let before = stack.sender_log();
    let spoken_before: Vec<usize> = (0..2)
        .map(|idx| count_lines_naming(&before, &stack.label(idx)))
        .collect();

    // When: the two rows trade places at a new generation.
    let swapped = vec![rows[1].clone(), rows[0].clone()];
    stack.reload(&swapped).expect("publish the swapped pair");
    let reloaded = stack
        .wait_for_log("applying queued connection changes", TICK_DETECTED)
        .expect("the reload must be applied");

    // Then: nothing is torn down or dialed — identity survived the move.
    assert!(
        !reloaded.contains("stale connection(s)") && !reloaded.contains("new connection(s)"),
        "a pure reorder must not recreate either socket:\n{reloaded}"
    );

    // And: each identity keeps speaking on the interface it was mapped to, and
    // never on the other twin's.
    sleep(Duration::from_secs(6));
    let after = stack.sender_log();
    for idx in 0..2 {
        assert!(
            count_lines_naming(&after, &stack.label(idx)) > spoken_before[idx],
            "twin {idx} stopped speaking after the swap:\n{after}"
        );
    }
    for (identity, foreign_iface) in [(0_usize, 1_usize), (1, 0)] {
        let crossed = format!(
            "on {} [{}]",
            rows[foreign_iface].iface, rows[identity].link_id
        );
        assert!(
            !after.contains(&crossed),
            "`{crossed}` means identity followed file position, not the mapping:\n{after}"
        );
    }
}

#[test]
fn a_reordered_sidecar_at_the_same_generation_is_refused() {
    if skip_without_twin_deps() {
        return;
    }

    // Given: a bonded twin pair on an applied mapping.
    let stack = started("tw_stalegen", Mapping::BindMap);
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("both twins must register");
    let rows: Vec<TwinRow> = stack.rows().to_vec();
    let applied_generation = stack.publisher.current_generation();

    // When: the identities trade INTERFACES — a mapping that really would move
    // traffic — but the generation and the IP file both stand still, so the two
    // mappings cannot be ordered against each other.
    let crossed = vec![
        TwinRow::new(&rows[0].link_id, &rows[1].iface),
        TwinRow::new(&rows[1].link_id, &rows[0].iface),
    ];
    stack
        .publisher
        .publish_at(network_sim::twin::TWIN_IP, &crossed, applied_generation)
        .expect("republish at the applied generation");
    stack.sighup().expect("signal the reload");

    // Then: it is refused and the applied mapping keeps running.
    let log = stack
        .wait_for_log("(retained_last_valid)", TICK_DETECTED)
        .expect("the stale republication must be refused");
    assert!(
        !log.contains("new connection(s)"),
        "an unorderable mapping must not be applied:\n{log}"
    );
    for idx in 0..2 {
        let moved = format!("on {} [{}]", crossed[idx].iface, crossed[idx].link_id);
        assert!(
            !log.contains(&moved),
            "`{moved}` means the refused mapping took effect anyway:\n{log}"
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario 4 — stale ifindex across an unplug/replug
// ---------------------------------------------------------------------------

#[test]
fn a_deleted_and_recreated_interface_recovers_on_the_new_ifindex() {
    if skip_without_twin_deps() {
        return;
    }

    // Given: a bonded twin pair, and the ifindex the second socket was bound
    // against. `SO_BINDTODEVICE` froze that index at setsockopt time.
    let mut stack = started("tw_ifindex", Mapping::BindMap);
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("both twins must register");
    let bound_ifindex = stack
        .topo
        .ifindex(1)
        .expect("the twin must have an ifindex");

    // When: the interface is unplugged.
    stack.topo.unplug(1).expect("unplug the twin");

    // Then: the link is marked removed instead of spinning the reconnect
    // backoff against a name the kernel no longer knows.
    stack
        .wait_for_log("egress interface is gone; link removed", TICK_DETECTED)
        .expect("the disappearance must be detected within a tick");
    assert!(
        stack.topo.ifindex(1).is_none(),
        "the interface must genuinely be gone"
    );

    // When: the same interface NAME comes back at a different index.
    stack.topo.replug(1).expect("replug the twin");
    let new_ifindex = stack.topo.ifindex(1).expect("the replug must enumerate");
    assert_ne!(
        bound_ifindex, new_ifindex,
        "the replug must produce a genuinely new ifindex, or this proves nothing"
    );

    // Then: the sender recovers onto it rather than reusing the stale socket.
    stack
        .wait_for_log(
            &format!(
                "{} timed out; attempting full socket reconnection",
                stack.label(1)
            ),
            ACK_TIMEOUT,
        )
        .expect("the stale binding must drive a reconnect");
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("the replugged twin must re-register");

    // And: the proof is on the new netdev's own counters, which started at zero
    // when it was created — traffic there cannot have come from the old index.
    let carried = carried_bytes(&mut stack, 8);
    assert!(
        carried[1] > 100_000,
        "the replugged twin must carry traffic on its new ifindex (carried: {carried:?})"
    );
}

// ---------------------------------------------------------------------------
// Scenario 5 — route-removal blackhole
// ---------------------------------------------------------------------------

#[test]
fn a_route_removal_blackhole_is_reported_and_never_reads_healthy() {
    if skip_without_twin_deps() {
        return;
    }

    // Given: a bonded twin pair carrying a stream.
    let mut stack = started("tw_blackhole", Mapping::BindMap);
    stack
        .wait_for_registered(2, REGISTERED)
        .expect("both twins must register");
    stack
        .start_traffic(OFFERED_PACKETS_PER_SEC, 90)
        .expect("start traffic");
    sleep(Duration::from_secs(3));
    let surviving_before = stack.topo.tx_bytes(0).expect("read tx_bytes");

    // When: the second twin loses its default route. Nothing errors: the socket
    // is device-bound, so IPv4 treats the receiver as on-link, ARPs for it, and
    // drops every datagram while `sendto` keeps returning success.
    stack.topo.blackhole(1).expect("remove the default route");

    // Then: the route invariant says so on its own axis, within one tick.
    let observed = stack
        .wait_for_log(
            &format!(
                "{}: egress interface lost its default route",
                stack.label(1)
            ),
            TICK_DETECTED,
        )
        .expect("the blackhole must be reported, not inferred");
    assert!(
        !observed.contains("regained its default route"),
        "the route must not be claimed healthy while it is absent:\n{observed}"
    );

    // And: ACK liveness independently fails, and the sender's own reported
    // egress health for that interface never reads as routed.
    stack
        .wait_for_log(
            &format!(
                "{} timed out; attempting full socket reconnection",
                stack.label(1)
            ),
            ACK_TIMEOUT,
        )
        .expect("a blackholed link must also fail ACK liveness");
    let status = stack
        .wait_for_log(
            &format!(
                "iface={} link=BOUND route=no_default_route",
                stack.rows()[1].iface
            ),
            STATUS_LOGGED,
        )
        .expect("the status log must report the blackholed link honestly");
    assert!(
        !status.contains(&format!(
            "iface={} link=BOUND route=no_default_route",
            stack.rows()[0].iface
        )),
        "the healthy twin must not be tarred with the blackholed one's state:\n{status}"
    );

    // And: the stream survives on the twin that still has a route.
    let surviving_carried = stack.topo.tx_bytes(0).expect("read tx_bytes") - surviving_before;
    assert!(
        surviving_carried > 100_000,
        "the bond must keep running on the healthy twin ({surviving_carried} bytes)"
    );

    // When: the route comes back — the namespace teardown would drop it anyway,
    // so no host state can be left modified by a failure above.
    stack
        .topo
        .restore_route(1)
        .expect("restore the default route");

    // Then: the recovery is reported on the same axis.
    stack
        .wait_for_log(
            &format!(
                "{}: egress interface regained its default route",
                stack.label(1)
            ),
            TICK_DETECTED,
        )
        .expect("the restored route must be reported");
}
