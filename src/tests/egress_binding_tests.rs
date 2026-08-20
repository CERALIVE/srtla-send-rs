//! Binder selection, the wire source address a mapped link puts on the wire,
//! and the connection-level reaction to a vanished interface.

#[cfg(unix)]
use std::io;
#[cfg(target_os = "linux")]
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};

use crate::bind_map::IfaceName;
#[cfg(unix)]
use crate::connection::egress::{EgressFault, classify_egress_fault};
use crate::connection::{RouteHealth, bind_for_link, bind_from_ip};

fn iface(name: &str) -> IfaceName {
    IfaceName::parse(name).expect("test interface name must be valid")
}

fn loopback() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}

#[test]
fn an_unmapped_link_binds_through_the_legacy_source_ip_path() {
    // Given: a link with no bind-map row.
    // When: its socket is created.
    let sock = bind_for_link(loopback(), 0, None).expect("legacy bind must succeed");

    // Then: it is bound to the source IP, exactly as `bind_from_ip` does.
    let legacy = bind_from_ip(loopback(), 0).expect("legacy bind must succeed");
    assert_eq!(
        sock.local_addr().unwrap().as_socket().unwrap().ip(),
        legacy.local_addr().unwrap().as_socket().unwrap().ip()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_mapped_link_binds_to_the_device_and_pins_its_source_address() {
    // Given: a bind-map row naming the loopback interface.
    // When: the socket is created for it.
    let sock = bind_for_link(loopback(), 0, Some("lo")).expect("device bind must succeed on lo");

    // Then: the source address is pinned, not left to the kernel's choice —
    // SO_BINDTODEVICE alone would leave it unspecified.
    let local = sock.local_addr().unwrap().as_socket().unwrap();
    assert_eq!(local.ip(), loopback());
    assert_ne!(local.port(), 0, "an ephemeral port must have been assigned");
}

#[cfg(target_os = "linux")]
#[test]
fn the_pinned_source_address_is_what_the_peer_actually_observes() {
    // Given: a receiver on loopback, and a device-bound uplink socket whose
    // source address the bind-map pinned.
    let receiver = std::net::UdpSocket::bind("127.0.0.1:0").expect("bind receiver");
    let receiver_addr = receiver.local_addr().unwrap();
    let uplink = bind_for_link(loopback(), 0, Some("lo")).expect("device bind must succeed on lo");
    let bound = uplink.local_addr().unwrap().as_socket().unwrap();

    // When: a datagram leaves that socket.
    let uplink: std::net::UdpSocket = uplink.into();
    uplink.set_nonblocking(false).unwrap();
    uplink.send_to(b"srtla", receiver_addr).expect("send");

    // Then: the source address ON THE WIRE is the one that was bound — this is
    // what lets a receiver tell two same-IP twin modems apart.
    let mut buf = [0u8; 16];
    receiver
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let (len, observed): (usize, SocketAddr) = receiver.recv_from(&mut buf).expect("recv");
    assert_eq!(&buf[..len], b"srtla");
    assert_eq!(
        observed, bound,
        "the peer must observe the exact source address the binder pinned"
    );
}

#[test]
fn binding_to_an_interface_that_does_not_exist_is_refused() {
    // Given: a bind-map row naming an interface the kernel does not know.
    // When/Then: the bind fails rather than silently falling back to a
    // kernel-chosen egress.
    assert!(bind_for_link(loopback(), 0, Some("nosuchif-zz9")).is_err());
}

#[cfg(unix)]
#[test]
fn enodev_and_enetunreach_are_the_two_faults_that_kill_a_binding() {
    // Given: the errnos a stale SO_BINDTODEVICE ifindex produces.
    // When/Then: each is classified, and an ordinary send error is not.
    assert_eq!(
        classify_egress_fault(&io::Error::from_raw_os_error(libc::ENODEV)),
        Some(EgressFault::InterfaceGone)
    );
    assert_eq!(
        classify_egress_fault(&io::Error::from_raw_os_error(libc::ENETUNREACH)),
        Some(EgressFault::NetworkUnreachable)
    );
    assert_eq!(
        classify_egress_fault(&io::Error::from_raw_os_error(libc::EAGAIN)),
        None
    );
}

#[tokio::test]
async fn reconnect_refuses_to_rebuild_a_socket_on_a_vanished_interface() {
    // Given: a live connection whose egress interface has disappeared (modelled
    // with a name the host genuinely does not have).
    let mut conn = crate::test_helpers::create_test_connection().await;
    conn.egress.adopt(Some(iface("nosuchif-zz9")));
    let stale = std::sync::Arc::as_ptr(&conn.socket) as *const ();

    // When: the reconnect path runs.
    let outcome = conn.reconnect().await;

    // Then: it refuses rather than binding something the kernel would steer by
    // its own routing table, the link is marked removed to await a reload, and
    // no new socket was installed.
    assert!(outcome.is_err());
    assert!(conn.is_removed());
    assert_eq!(std::sync::Arc::as_ptr(&conn.socket) as *const (), stale);
}

#[tokio::test]
async fn housekeeping_does_not_retry_a_removed_link() {
    // Given: a timed-out connection whose interface is gone — the state that
    // would otherwise drive an endless bind/backoff loop every tick.
    use std::collections::HashMap;

    use crate::registration::SrtlaRegistrationManager;
    use crate::sender::SequenceTracker;
    use crate::sender::uplink::{ReaderHandle, UplinkPacket, create_uplink_channel};

    let mut connections = vec![crate::test_helpers::create_test_connection().await];
    connections[0].egress.adopt(Some(iface("nosuchif-zz9")));
    connections[0].mark_for_recovery();
    let attempts_before = connections[0].reconnection.reconnect_failure_count;
    let (packet_tx, _rx): (_, tokio::sync::mpsc::UnboundedReceiver<UplinkPacket>) =
        create_uplink_channel();
    let mut handles: HashMap<u64, ReaderHandle> = HashMap::new();

    // When: a housekeeping tick runs.
    let _ = crate::sender::housekeeping::handle_housekeeping(
        &mut connections,
        &mut SrtlaRegistrationManager::new(),
        false,
        &mut None,
        &mut handles,
        &packet_tx,
        &mut SequenceTracker::new(),
    )
    .await;

    // Then: the link is reported removed and no reconnect was attempted — it
    // waits for a reload that names an interface that exists.
    assert!(connections[0].is_removed());
    assert_eq!(
        connections[0].reconnection.reconnect_failure_count,
        attempts_before
    );
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn route_health_is_reported_independently_of_ack_liveness() {
    // Given: a connection that is ACK-live (freshly received) and pinned to an
    // interface that genuinely exists on this host.
    let mut conn = crate::test_helpers::create_test_connection().await;
    conn.egress.adopt(Some(iface("lo")));
    assert!(!conn.is_timed_out(), "the link is ACK-live");

    // When: the egress poll refreshes the route invariant.
    conn.poll_egress(&crate::connection::egress::SystemIfaceResolver);

    // Then: route health is observed on its own axis. `lo` never carries a
    // default route, so an ACK-live link is correctly reported as one whose
    // pinned egress would blackhole — a state ACK liveness cannot express.
    assert_eq!(conn.route_health, RouteHealth::NoDefaultRoute);
    assert!(!conn.is_timed_out());
}

#[tokio::test]
async fn an_unmapped_link_has_no_route_invariant_to_observe() {
    // Given: a legacy link with no interface binding.
    let mut conn = crate::test_helpers::create_test_connection().await;

    // When: the egress poll runs.
    conn.poll_egress(&crate::connection::egress::SystemIfaceResolver);

    // Then: nothing is claimed about its route — the sender does not guess
    // which interface a source-IP-bound socket egresses through.
    assert_eq!(conn.route_health, RouteHealth::Unknown);
}
