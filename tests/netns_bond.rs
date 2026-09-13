//! Real kernel coverage for mixed and shared-IP bonds. Wire carriage is not SRT goodput.

#[path = "netns_bond/controls.rs"]
mod controls;
#[path = "netns_bond/profile.rs"]
mod profile;
#[path = "netns_bond/stack.rs"]
mod stack;

use std::thread;
use std::time::Duration;

use network_sim::bond::{BondRow, BondTopology, CarrierMode, LinkSpec, MappingMode, RECEIVER_IP};

const DIRECT: LinkSpec = LinkSpec {
    carrier: CarrierMode::Direct,
    shared_ip_with: None,
};
const NAT: LinkSpec = LinkSpec {
    carrier: CarrierMode::Nat,
    shared_ip_with: None,
};
const MIXED: [LinkSpec; 4] = [DIRECT, DIRECT, DIRECT, NAT];
const TWINS: [LinkSpec; 2] = [
    NAT,
    LinkSpec {
        shared_ip_with: Some(0),
        ..NAT
    },
];

fn mapping(count: usize) -> MappingMode {
    MappingMode::BindMap {
        rows: (0..count)
            .map(|iface_index| BondRow {
                link_id: format!("link-{iface_index}"),
                iface_index,
                priority: None,
            })
            .collect(),
    }
}

#[test]
fn four_distinct_links_register_and_carry_without_a_sidecar() {
    if !stack::available() {
        return;
    }
    // Given: three Direct links and one NAT carrier, explicit legacy-IP mode.
    let topo = BondTopology::new("bond4", &MIXED, MappingMode::None).unwrap();
    assert!(topo.sidecar_path().is_none());
    let mut stack = stack::Stack::start(topo).unwrap();
    stack.registered(4).unwrap();
    let before = stack.counters().unwrap();
    // When: sending sustained DATA through the real sender and receiver.
    stack.start_traffic().unwrap();
    // Then: every link carries at least 100kB, not just registration packets.
    let carried = stack.wait_for_carriage(&before).unwrap();
    eprintln!("four-link distinct carriage: {carried:?}");
}

#[test]
fn mapped_shared_ip_pair_registers_and_carries_on_both_interfaces() {
    if !stack::available() {
        return;
    }
    // Given: two independent NAT carriers serving exactly the same source IP.
    let topo = BondTopology::new("bondtw", &TWINS, mapping(2)).unwrap();
    assert_eq!(topo.sender_ip(0), "10.30.9.1");
    assert_eq!(topo.sender_ip(0), topo.sender_ip(1));
    let mut stack = stack::Stack::start(topo).unwrap();
    stack.registered(2).unwrap();
    assert!(
        stack
            .sender
            .log_snapshot()
            .join("\n")
            .contains("bind-map active (mapped)")
    );
    let before = stack.counters().unwrap();
    // When: classic mode forwards DATA with the published interface mapping.
    stack.start_traffic().unwrap();
    // Then: both twins carry at least 100kB.
    eprintln!(
        "shared-IP mapped carriage: {:?}",
        stack.wait_for_carriage(&before).unwrap()
    );
}

#[test]
fn explicit_legacy_control_leaves_the_second_twin_unused() {
    if !stack::available() {
        return;
    }
    // Given: the same twins, intentionally launched without the sidecar.
    let topo = BondTopology::new("bondctl", &TWINS, MappingMode::LegacyControl).unwrap();
    assert!(topo.sidecar_path().is_none());
    let mut stack = stack::Stack::start(topo).unwrap();
    stack.registered(1).unwrap();
    let before = stack.counters().unwrap();
    // When: sustained traffic is offered to the legacy sender.
    stack.start_traffic().unwrap();
    thread::sleep(Duration::from_secs(3));
    // Then: only the deterministic first representative carries DATA.
    let after = stack.counters().unwrap();
    assert!(after[0] - before[0] >= 100_000);
    assert!(
        after[1] - before[1] < 2048,
        "legacy twin unexpectedly carried DATA: {before:?} -> {after:?}"
    );
    eprintln!("shared-IP legacy control: {before:?} -> {after:?}");
}

#[test]
fn removing_nat_default_stops_its_data_while_other_links_continue() {
    if !stack::available() {
        return;
    }
    // Given: a live mapped mixed bond, with every path proven to carry DATA.
    let topo = BondTopology::new("bondfault", &MIXED, mapping(4)).unwrap();
    let mut stack = stack::Stack::start(topo).unwrap();
    stack.registered(4).unwrap();
    let initial = stack.counters().unwrap();
    stack.start_traffic().unwrap();
    stack.wait_for_carriage(&initial).unwrap();
    // When: only link 3's defaults disappear; the interface stays up.
    stack.topo.delete_default_route(3).unwrap();
    thread::sleep(Duration::from_secs(3));
    let before = stack.counters().unwrap();
    thread::sleep(Duration::from_secs(2));
    let after = stack.counters().unwrap();
    // Then: DATA carriage stops on link 3, while all other paths keep carrying.
    // tx_bytes includes a few ARP probes for the now-on-link receiver, not DATA.
    assert!(
        after[3] - before[3] < 2048,
        "route-less link still carries DATA: {before:?} -> {after:?}"
    );
    for i in 0..3 {
        assert!(
            after[i] - before[i] >= 100_000,
            "healthy link {i} stopped: {before:?} -> {after:?}"
        );
    }
    eprintln!("route-deletion observation: {before:?} -> {after:?}");
    stack.topo.restore_default_route(3).unwrap();
    eprintln!(
        "restored carriage: {:?}",
        stack.wait_for_carriage(&after).unwrap()
    );
}

#[test]
fn replug_and_link_state_preserve_each_path_to_the_receiver() {
    if !stack::available() {
        return;
    }
    // Given: one Direct and one NAT path with valid source routing.
    let topo = BondTopology::new("bondplug", &[DIRECT, NAT], MappingMode::None).unwrap();
    for i in 0..topo.link_count() {
        let iface = topo.sender_iface(i);
        let path = format!("/sys/class/net/{iface}/ifindex");
        let before = topo.sender_ns.exec_checked("cat", &[&path]).unwrap().stdout;
        topo.set_link_up(i, false).unwrap();
        topo.set_link_up(i, true).unwrap();
        // When: replacing the access veth, not merely toggling its administrative state.
        topo.replug(i).unwrap();
        // Then: a new kernel interface can still reach the receiver using that source.
        let after = topo.sender_ns.exec_checked("cat", &[&path]).unwrap().stdout;
        assert_ne!(before, after);
        topo.sender_ns
            .exec_checked(
                "ping",
                &["-c", "1", "-W", "2", "-I", topo.sender_ip(i), RECEIVER_IP],
            )
            .unwrap();
    }
}
