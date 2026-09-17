#![cfg(any(test, feature = "test-internals"))]
#![allow(dead_code)] // Allow unused helpers - they're used by library tests but not binary tests

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use socket2::{Domain, Protocol, Socket, Type};
#[cfg(test)]
use tokio::time::Duration;
use tokio::time::Instant;

use crate::connection::{
    BatchSender, BatchUdpSocket, BitrateTracker, CachedQuality, CongestionControl, EgressLifecycle,
    ReconnectionState, RouteHealth, RttTracker, SrtlaConnection,
};
use crate::protocol::{PKT_LOG_SIZE, WINDOW_DEF, WINDOW_MULT};
use crate::utils::now_ms;

/// Shared test connection ID counter for all helper functions
static NEXT_TEST_CONN_ID: AtomicU64 = AtomicU64::new(1000);

/// Create a test UDP socket bound to localhost.
fn create_test_socket() -> Socket {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket
        .bind(&"127.0.0.1:0".parse::<SocketAddr>().unwrap().into())
        .unwrap();
    socket.set_nonblocking(true).unwrap();
    socket
}

/// Create a SrtlaConnection from a socket and connection parameters.
fn create_connection_from_socket(
    socket: Socket,
    remote: SocketAddr,
    local_ip: IpAddr,
    label: String,
) -> SrtlaConnection {
    let batch_socket = BatchUdpSocket::new(socket, remote).unwrap();

    SrtlaConnection {
        conn_id: NEXT_TEST_CONN_ID.fetch_add(1, Ordering::Relaxed),
        socket: Arc::new(batch_socket),
        remote,
        host: remote.ip().to_string(),
        port: remote.port(),
        local_ip,
        link_id: None,
        priority_baseline: None,
        priority_override_link: None,
        priority_override_conn: None,
        egress: EgressLifecycle::unmapped(),
        route_health: RouteHealth::Unknown,
        label,
        connected: true,
        window: WINDOW_DEF * WINDOW_MULT,
        in_flight_packets: 0,
        packet_log: FxHashMap::with_capacity_and_hasher(PKT_LOG_SIZE, Default::default()),
        delivery: crate::connection::delivery::DeliveryLedger::default(),
        premature_streak: 0,
        premature_nak_count: 0,
        rexmit_forwarded: 0,
        loss: crate::connection::loss::LossTracker::new(now_ms()),
        probes: crate::connection::probe::ProbeLog::default(),
        health: crate::connection::health::HealthMachine::new(
            crate::connection::health::HealthState::Down,
            now_ms(),
        ),
        rate_cap: crate::connection::rate_cap::RateCap::default(),
        wire_rate: crate::connection::wire_rate::WireRateEstimator::default(),
        adaptive: crate::connection::adaptive::AdaptiveLinkState::default(),
        highest_acked_seq: None,
        last_received: Some(Instant::now()),
        last_sent: None,
        last_keepalive_sent: None,
        keepalive_liveness: Default::default(),
        last_probe_growth_ms: 0,
        last_ack_or_rtt_sample_ms: 0,
        #[cfg(test)]
        last_stall_reprobe_ms: 0,
        last_trunc_warn_ms: 0,
        rtt: RttTracker::default(),
        congestion: CongestionControl::default(),
        bitrate: BitrateTracker::default(),
        reconnection: ReconnectionState {
            connection_established_ms: now_ms(),
            startup_grace_deadline_ms: now_ms(),
            ..Default::default()
        },
        quality_cache: CachedQuality::default(),
        batch_sender: BatchSender::new(),
    }
}

pub async fn create_test_connection() -> SrtlaConnection {
    let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    create_test_connection_to(remote).await
}

pub async fn create_test_connection_to(remote: SocketAddr) -> SrtlaConnection {
    let socket = create_test_socket();
    let local_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    create_connection_from_socket(socket, remote, local_ip, "test-connection".to_string())
}

/// A connection whose every send fails immediately, giving registration tests a
/// deterministic send-failure seam with no timing, network, or OS-semantics
/// dependency (see `BatchUdpSocket::fail_sends`).
pub async fn create_test_connection_with_failing_send() -> SrtlaConnection {
    let socket = create_test_socket();
    let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let local_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

    let conn = create_connection_from_socket(socket, remote, local_ip, "failing-send".to_string());
    conn.socket.fail_sends();
    conn
}

pub async fn create_test_connections(count: usize) -> SmallVec<SrtlaConnection, 4> {
    let mut connections = SmallVec::new();

    for i in 0..count {
        let socket = create_test_socket();
        let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080 + i as u16);
        let local_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10 + i as u8));
        let label = format!("test-connection-{}", i);

        connections.push(create_connection_from_socket(
            socket, remote, local_ip, label,
        ));
    }

    connections
}

/// Ranking fixtures represent healthy links after shared admission, not startup Down links.
#[cfg(test)]
pub async fn create_selection_test_connections(count: usize) -> SmallVec<SrtlaConnection, 4> {
    let mut conns = create_test_connections(count).await;
    for conn in &mut conns {
        conn.health = crate::connection::health::HealthMachine::new(
            crate::connection::health::HealthState::Healthy,
            now_ms(),
        );
    }
    let config = crate::config::DynamicConfig::new();
    config.set_quality_enabled(false);
    crate::sender::SchedulerShared::default().update_stats(&mut conns, &config.snapshot());
    conns
}

/// Advance the paused Tokio virtual clock by `by`.
///
/// Only meaningful inside `#[tokio::test(start_paused = true)]`. Connection timing
/// (`last_received`, `last_keepalive_sent`, the housekeeping `all_failed_at` timer)
/// reads `tokio::time::Instant`, so jumping the virtual clock makes timeout/keepalive
/// logic fire deterministically with no real sleep. This is the test seam: timing
/// tests advance the clock through here rather than calling `tokio::time::advance`
/// inline, keeping the dependency on the virtual clock explicit.
///
/// `cfg(test)` only: `tokio::time::advance` needs tokio's `test-util` feature,
/// which arrives via dev-dependencies. Gating it on `test-internals` too would
/// make `cargo build --features test-internals` (the A/B evaluation binary)
/// fail to compile, and enabling `tokio/test-util` in that binary would put a
/// clock indirection under the very measurements it exists to take.
#[cfg(test)]
pub async fn advance_test_clock(by: Duration) {
    tokio::time::advance(by).await;
}
