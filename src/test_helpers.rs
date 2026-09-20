#![cfg(any(test, feature = "test-internals"))]
// Allow unused helpers/re-exports: they're used by library tests but not binary
// tests (the re-exports below export nothing in the binary crate).
#![allow(dead_code, unused_imports)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use socket2::{Domain, Protocol, Socket, Type};
use srtla_core::connection::SrtlaConnection;
// The socket-free connection builders (create_test_connection[s]) and the tokio
// clock seam (advance_test_clock) live in srtla-core's test-internals surface,
// re-exported here so `crate::test_helpers::*` keeps resolving for shell tests.
pub use srtla_core::test_helpers::{
    advance_test_clock, create_test_connection, create_test_connections,
};

use crate::net::{BatchUdpSocket, SourceIpBinder};
use crate::sender::{ConnIo, ConnIoMap};

/// Build the I/O half for a test connection: a real localhost UDP socket
/// wrapped in a `BatchUdpSocket`. Used by tests that drive reader/reconnect I/O.
pub fn create_test_conn_io() -> ConnIo {
    create_test_conn_io_for(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

/// Same, but with the uplink spec reporting `source_ip`.
///
/// The socket still binds loopback — the spec's IP is the pool builder's
/// *socket key*, not an address anything dials here. A map built with one
/// shared IP would collapse every test connection into a single key, so the
/// spec has to track the connection it belongs to.
pub fn create_test_conn_io_for(source_ip: IpAddr) -> ConnIo {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket
        .bind(&"127.0.0.1:0".parse::<SocketAddr>().unwrap().into())
        .unwrap();
    socket.set_nonblocking(true).unwrap();
    let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    ConnIo::unmapped(
        Arc::new(BatchUdpSocket::new(socket, remote).unwrap()),
        Arc::new(SourceIpBinder),
        remote,
        source_ip,
    )
}

/// Build a `ConnIoMap` giving each connection its own localhost socket, keyed by
/// `conn_id` (matching the shell's real lifecycle).
pub fn create_test_conn_io_map(connections: &[SrtlaConnection]) -> ConnIoMap {
    connections
        .iter()
        .map(|c| (c.conn_id, create_test_conn_io_for(c.local_ip)))
        .collect()
}
