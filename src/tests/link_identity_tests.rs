//! Link identity across reloads: `link_id` is the identity, `(ip, iface)` is
//! only the current socket key.

use std::net::{IpAddr, Ipv4Addr};

use smallvec::SmallVec;

use crate::bind_map::{IfaceName, LinkId};
use crate::connection::{SrtlaConnection, UplinkSpec};
use crate::registration::SrtlaRegistrationManager;
use crate::sender::{SequenceTracker, apply_link_changes};
use crate::test_helpers::create_test_connections;

const HOST: &str = "127.0.0.1";
const PORT: u16 = 9000;

fn ip(last: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(192, 168, 8, last))
}

fn mapped(link_id: &str, last: u8, iface: &str) -> UplinkSpec {
    UplinkSpec {
        ip: ip(last),
        iface: Some(IfaceName::parse(iface).unwrap()),
        link_id: Some(LinkId::parse(link_id).unwrap()),
    }
}

/// Re-stamp test connections as mapped uplinks, so the pool looks like one a
/// bind-map produced without needing a real interface to bind to.
fn adopt_specs(connections: &mut [SrtlaConnection], specs: &[UplinkSpec]) {
    for (conn, spec) in connections.iter_mut().zip(specs) {
        conn.local_ip = spec.ip;
        conn.link_id = spec.link_id.clone();
        conn.egress.adopt(spec.iface.clone());
        conn.label = spec.label(HOST, PORT);
    }
}

async fn apply(
    connections: &mut SmallVec<SrtlaConnection, 4>,
    links: &[UplinkSpec],
) -> Option<usize> {
    let mut last_selected_idx = Some(0);
    apply_link_changes(
        connections,
        links,
        HOST,
        PORT,
        &mut last_selected_idx,
        &mut SequenceTracker::new(),
        &mut SrtlaRegistrationManager::new(),
    )
    .await;
    last_selected_idx
}

#[tokio::test]
async fn twin_modems_sharing_one_ip_are_two_links_not_one() {
    // Given: two bind-map rows with the SAME source IP on different interfaces —
    // the HiLink twin case legacy identity silently collapses to one link.
    let mut connections = create_test_connections(2).await;
    let twins = [
        mapped("modem-a", 100, "wwan0"),
        mapped("modem-b", 100, "wwan1"),
    ];
    adopt_specs(&mut connections, &twins);
    let ids: Vec<u64> = connections.iter().map(|c| c.conn_id).collect();

    // When: that exact set is reapplied.
    apply(&mut connections, &twins).await;

    // Then: both survive. Dedup ran on (ip, iface), not on ip.
    assert_eq!(connections.len(), 2);
    assert_eq!(
        connections.iter().map(|c| c.conn_id).collect::<Vec<_>>(),
        ids
    );
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn a_source_ip_change_rebinds_the_link_under_its_unchanged_identity() {
    // Given: a link with a stable identity, bound to a loopback source address
    // this host can actually bind (so the rebind really happens).
    let loopback = |last: u8| UplinkSpec {
        ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, last)),
        iface: None,
        link_id: Some(LinkId::parse("modem-a").unwrap()),
    };
    let mut connections = create_test_connections(1).await;
    adopt_specs(&mut connections, &[loopback(2)]);
    let before = connections[0].conn_id;

    // When: a reload gives the SAME link_id a different source IP (a DHCP lease
    // change on the same modem).
    apply(&mut connections, &[loopback(3)]).await;

    // Then: the link is still that link, but on a fresh socket — the identity
    // is carried, the socket-scoped state is not.
    assert_eq!(connections.len(), 1);
    assert_eq!(
        connections[0].link_id.as_ref().map(LinkId::as_str),
        Some("modem-a")
    );
    assert_ne!(connections[0].conn_id, before);
    assert_eq!(
        connections[0].local_ip,
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3))
    );
}

#[tokio::test]
async fn an_interface_change_recreates_the_socket_instead_of_carrying_it_over() {
    // Given: a mapped link with accumulated, ifindex-scoped connection state.
    let mut connections = create_test_connections(1).await;
    adopt_specs(&mut connections, &[mapped("modem-a", 100, "wwan0")]);
    let before = connections[0].conn_id;
    connections[0].in_flight_packets = 17;

    // When: a reload moves that link_id onto a different interface.
    apply(&mut connections, &[mapped("modem-a", 100, "wwan3")]).await;

    // Then: the old connection did not survive. Its window, packet log, and
    // in-flight count all described an interface that no longer carries it, so
    // transferring them would credit the new link with the old one's history.
    // (The rebind itself fails here — wwan3 does not exist on the test host —
    // which is exactly the point: nothing was carried over.)
    assert!(
        connections.iter().all(|c| c.conn_id != before),
        "an interface change must not hand the old socket to the new binding"
    );
}

#[tokio::test]
async fn an_unchanged_mapped_reload_keeps_every_socket() {
    // Given: a two-link mapped bond.
    let mut connections = create_test_connections(2).await;
    let links = [
        mapped("modem-a", 100, "wwan0"),
        mapped("modem-b", 100, "wwan1"),
    ];
    adopt_specs(&mut connections, &links);
    let sockets: Vec<*const ()> = connections
        .iter()
        .map(|c| std::sync::Arc::as_ptr(&c.socket) as *const ())
        .collect();

    // When: SIGHUP re-publishes the identical mapping.
    let selection = apply(&mut connections, &links).await;

    // Then: no link re-handshakes — same sockets, and the cached selection
    // index is preserved because nothing moved.
    let after: Vec<*const ()> = connections
        .iter()
        .map(|c| std::sync::Arc::as_ptr(&c.socket) as *const ())
        .collect();
    assert_eq!(after, sockets);
    assert_eq!(selection, Some(0));
}

#[tokio::test]
async fn reordering_the_map_repositions_links_without_rebinding_them() {
    // Given: a mapped bond in file order [A, B].
    let mut connections = create_test_connections(2).await;
    let a = mapped("modem-a", 100, "wwan0");
    let b = mapped("modem-b", 100, "wwan1");
    adopt_specs(&mut connections, &[a.clone(), b.clone()]);
    let cid_a = connections[0].conn_id;
    let cid_b = connections[1].conn_id;

    // When: the map is republished as [B, A].
    let selection = apply(&mut connections, &[b, a]).await;

    // Then: both survive, repositioned — telemetry `conn_id` tracks file order —
    // and the reorder invalidates the cached selection index.
    assert_eq!(connections[0].conn_id, cid_b);
    assert_eq!(connections[1].conn_id, cid_a);
    assert_eq!(selection, None);
}

#[tokio::test]
async fn dropping_a_mapped_link_leaves_its_twin_running() {
    // Given: twin modems on one IP.
    let mut connections = create_test_connections(2).await;
    let a = mapped("modem-a", 100, "wwan0");
    let b = mapped("modem-b", 100, "wwan1");
    adopt_specs(&mut connections, &[a.clone(), b]);
    let cid_a = connections[0].conn_id;

    // When: one twin is unplugged and the map republished without it.
    apply(&mut connections, &[a]).await;

    // Then: the survivor keeps its identity and its socket. Under legacy
    // IP-only keying this pair was indistinguishable.
    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0].conn_id, cid_a);
}

#[tokio::test]
async fn a_duplicate_socket_key_is_deduplicated_the_way_a_duplicate_ip_was() {
    // Given: a pool and a reload naming the SAME (ip, iface) twice.
    let mut connections = create_test_connections(1).await;
    let a = mapped("modem-a", 100, "wwan0");
    adopt_specs(&mut connections, &[a.clone()]);

    // When: the duplicate row is applied.
    apply(&mut connections, &[a.clone(), a]).await;

    // Then: one socket, not two — two rows naming one socket key is still one
    // socket, exactly as two identical IPs were before.
    assert_eq!(connections.len(), 1);
}

#[tokio::test]
async fn an_unmapped_reload_still_matches_a_link_by_its_socket_key() {
    // Given: a mapped link.
    let mut connections = create_test_connections(1).await;
    adopt_specs(&mut connections, &[mapped("modem-a", 100, "wwan0")]);
    let before = connections[0].conn_id;

    // When: a reload arrives with no mapping but the same socket key.
    apply(
        &mut connections,
        &[UplinkSpec {
            ip: ip(100),
            iface: Some(IfaceName::parse("wwan0").unwrap()),
            link_id: None,
        }],
    )
    .await;

    // Then: the live link is matched on its socket key rather than torn down —
    // losing the map is not a reason to drop a bond that did not change.
    assert_eq!(connections.len(), 1);
    assert_eq!(connections[0].conn_id, before);
}
