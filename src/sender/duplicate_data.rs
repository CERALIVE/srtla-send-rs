//! Duplicate DATA wire form, outside normal sequence ownership.

#[cfg(feature = "test-internals")]
use std::sync::OnceLock;
#[cfg(any(test, feature = "test-internals"))]
use std::sync::atomic::{AtomicU64, Ordering};

use smallvec::SmallVec;

use crate::connection::SrtlaConnection;

#[cfg(any(test, feature = "test-internals"))]
struct DuplicateHook {
    every: u64,
    retransmit: bool,
    count: AtomicU64,
}

#[cfg(any(test, feature = "test-internals"))]
impl DuplicateHook {
    const fn new(every: u64, retransmit: bool) -> Self {
        Self {
            every,
            retransmit,
            count: AtomicU64::new(0),
        }
    }

    fn due(&self, packet: &[u8]) -> bool {
        packet.len() >= 16
            && packet[0] & 0x80 == 0
            && (self.count.fetch_add(1, Ordering::Relaxed) + 1).is_multiple_of(self.every)
    }

    fn target(
        &self,
        connections: &[SrtlaConnection],
        primary: usize,
        packet: &[u8],
    ) -> Option<(usize, bool)> {
        if !connections[primary].connected || !self.due(packet) {
            return None;
        }
        connections
            .iter()
            .enumerate()
            .find(|(idx, conn)| *idx != primary && conn.connected && !conn.is_timed_out())
            .map(|(idx, _)| (idx, self.retransmit))
    }
}

#[cfg(feature = "test-internals")]
pub(super) fn target(
    connections: &[SrtlaConnection],
    primary: usize,
    packet: &[u8],
) -> Option<(usize, bool)> {
    // The test driver launches one sender process per run; no environment mutation
    // or per-packet parsing, and no state exists in non-test-internals builds.
    static HOOK: OnceLock<Option<DuplicateHook>> = OnceLock::new();
    let hook = HOOK
        .get_or_init(|| {
            let every = std::env::var("SRTLA_TEST_DUP_EVERY").ok()?;
            let every = every.parse::<std::num::NonZeroU64>().ok()?;
            let retransmit = match std::env::var("SRTLA_TEST_DUP_RETX_BIT").as_deref() {
                Ok("0") => false,
                Ok("1") => true,
                _ => return None,
            };
            Some(DuplicateHook::new(every.get(), retransmit))
        })
        .as_ref()?;
    hook.target(connections, primary, packet)
}

/// Bypass queue, packet log, in-flight, bitrate and sequence ownership. An immutable
/// connection borrow makes sender-side bookkeeping mutations impossible here.
#[cfg(any(test, feature = "test-internals"))]
pub(super) async fn send_copy(
    conn: &SrtlaConnection,
    packet: &[u8],
    retransmit: bool,
) -> std::io::Result<()> {
    let copy = wire_copy(packet, retransmit);
    let sent = conn.socket.send(&copy).await?;
    if sent != copy.len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            "short duplicate DATA send",
        ));
    }
    Ok(())
}

fn wire_copy(packet: &[u8], retransmit: bool) -> SmallVec<u8, 1500> {
    let mut copy = SmallVec::from_slice_copy(packet);
    // draft-sharabayko-srt-01 §3.1: |PP|O|KK|R|message(26)|.
    // R = word 1 bit 26 = SRT byte offset 4, LSB bit 2 (mask 0x04).
    copy[4] = (copy[4] & !0x04) | if retransmit { 0x04 } else { 0 };
    copy
}

impl SrtlaConnection {
    /// Queue an already-forwarded DATA copy with probe-only accepted-prefix metadata.
    /// The caller owns pacing/eligibility; a stale dispatch or local sequence collision
    /// is refused rather than making ACK attribution ambiguous on this socket.
    pub fn queue_probe_packet(
        &mut self,
        packet: &[u8],
        dispatch: crate::connection::probe::ProbeDispatch,
    ) -> bool {
        if packet.len() < 16
            || packet[0] & 0x80 != 0
            || dispatch.conn_id != self.conn_id
            || dispatch.socket_generation != self.delivery.socket_generation
        {
            return false;
        }
        let Some(seq) = crate::protocol::get_srt_sequence_number(packet)
            .and_then(|seq| i32::try_from(seq).ok())
        else {
            return false;
        };
        if self.delivery.contains(seq)
            || self.packet_log.contains_key(&seq)
            || self.probes.has_seen(seq)
        {
            return false;
        }
        let copy = wire_copy(packet, false);
        self.batch_sender.queue_probe(&copy, seq, dispatch.train);
        true
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use tokio::net::UdpSocket;
    use tokio::time::{Duration, timeout};

    use super::*;
    use crate::connection::SrtlaConnection;

    #[test]
    fn cadence_counts_only_complete_data_headers() {
        // Given a hook with a two-DATA cadence.
        let hook = DuplicateHook::new(2, true);
        let data = [0u8; 16];
        // When controls and short frames interrupt the DATA stream.
        assert!(!hook.due(&[0x80; 16]));
        assert!(!hook.due(&[0; 15]));
        assert!(!hook.due(&data));
        // Then only the second complete DATA is selected.
        assert!(hook.due(&data));
        assert!(!hook.due(&data));
        assert!(hook.due(&data));
    }

    #[tokio::test]
    async fn copy_uses_alternate_socket_without_tracking_or_payload_mutation() {
        // Given two registered-state uplinks.
        let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = receiver.local_addr().unwrap().port();
        let mut connections = Vec::new();
        for _ in 0..2 {
            let mut conn = SrtlaConnection::connect_from_ip(
                IpAddr::V4(Ipv4Addr::LOCALHOST),
                "127.0.0.1",
                port,
            )
            .await
            .unwrap();
            conn.connected = true;
            connections.push(conn);
        }
        connections[1].register_packet(7, 100);
        connections[1].queue_data_packet(&[0; 32], Some(9), 120);
        for retx in [false, true] {
            let mut packet = [0x55; 32];
            packet[0] = 0;
            packet[4] = if retx { 0xf8 } else { 0xfc };
            let before = &connections[1];
            let state = (
                before.in_flight_packets,
                before.packet_log.len(),
                before.bitrate.bytes_sent_total,
            );
            // When the emission primitive sends the copy.
            send_copy(&connections[1], &packet, retx).await.unwrap();
            let mut buf = [0u8; 64];
            let (len, source) = timeout(Duration::from_secs(1), receiver.recv_from(&mut buf))
                .await
                .unwrap()
                .unwrap();
            // Then only byte 4 bit 2 changes; the registered socket and all tracking survive.
            assert_eq!(source, connections[1].socket.local_addr().unwrap());
            assert_eq!(len, packet.len());
            packet[4] = (packet[4] & !0x04) | if retx { 0x04 } else { 0 };
            assert_eq!(&buf[..len], &packet);
            let after = &connections[1];
            assert_eq!(
                (
                    after.in_flight_packets,
                    after.packet_log.len(),
                    after.bitrate.bytes_sent_total
                ),
                state
            );
            assert!(after.has_queued_packets());
        }
    }

    #[tokio::test]
    async fn target_requires_a_different_registered_link() {
        // Given a two-link pool, initially with only the primary registered.
        let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = receiver.local_addr().unwrap().port();
        let mut connections = Vec::new();
        for _ in 0..2 {
            connections.push(
                SrtlaConnection::connect_from_ip(
                    IpAddr::V4(Ipv4Addr::LOCALHOST),
                    "127.0.0.1",
                    port,
                )
                .await
                .unwrap(),
            );
        }
        connections[0].connected = true;
        let hook = DuplicateHook::new(1, false);
        // When registration changes, then only the other registered socket is eligible.
        assert_eq!(hook.target(&connections, 0, &[0; 16]), None);
        connections[1].connected = true;
        assert_eq!(hook.target(&connections, 0, &[0; 16]), Some((1, false)));
        assert_eq!(hook.target(&connections, 1, &[0; 16]), Some((0, false)));
        assert_eq!(hook.target(&connections, 0, &[0x80; 16]), None);
        assert_eq!(hook.target(&connections[..1], 0, &[0; 16]), None);
    }
}
