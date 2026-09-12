//! Batch transmit (`sendmmsg`) + unconnected-uplink behavior.
//!
//! Covers the prefix-commit flush contract, the three flush call sites routing a
//! hard error into recovery + sequence-tracker cleanup, the foreign-source
//! accounting that replaces `connect(2)`'s implicit source filter, and the
//! registration guard trio.

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;

    use smallvec::SmallVec;
    use tokio::net::UdpSocket;

    use crate::connection::SrtlaConnection;
    use crate::connection::batch_recv::BatchUdpSocket;
    use crate::connection::batch_send::{BATCH_SEND_SIZE, BatchSender};
    use crate::protocol::{SRTLA_ID_LEN, SRTLA_TYPE_REG_ERR, SRTLA_TYPE_REG3, create_reg2_packet};
    use crate::registration::SrtlaRegistrationManager;
    use crate::sender::SequenceTracker;
    use crate::sender::packet_handler::{flush_all_batches, forward_via_connection};
    use crate::sender::uplink::{create_uplink_channel, spawn_reader};
    use crate::test_helpers::{create_test_connection, create_test_connection_with_failing_send};
    use crate::utils::now_ms;

    /// An uplink whose every send fails immediately, with no timing,
    /// reachability, or OS-semantics dependency (see
    /// `BatchUdpSocket::fail_sends`).
    async fn unsendable_connection() -> SrtlaConnection {
        let conn =
            SrtlaConnection::connect_from_ip(IpAddr::V4(Ipv4Addr::LOCALHOST), "127.0.0.1", 9)
                .await
                .expect("bind must succeed even though the peer is unsendable");
        conn.socket.fail_sends();
        conn
    }

    async fn reachable_connection() -> (SrtlaConnection, UdpSocket) {
        let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = receiver.local_addr().unwrap().port();
        let conn =
            SrtlaConnection::connect_from_ip(IpAddr::V4(Ipv4Addr::LOCALHOST), "127.0.0.1", port)
                .await
                .unwrap();
        (conn, receiver)
    }

    // ------------------------------------------------------------------
    // Prefix-commit flush contract
    // ------------------------------------------------------------------

    /// One flush drains at most `BATCH_SEND_SIZE` datagrams and retains the
    /// unsent suffix in queue order, so a bounded batch can never drop packets.
    #[tokio::test]
    async fn flush_commits_only_the_bounded_accepted_prefix() {
        let (conn, receiver) = reachable_connection().await;
        let socket: Arc<BatchUdpSocket> = conn.socket.clone();
        let mut sender = BatchSender::new();

        let overflow = 8usize;
        let total = BATCH_SEND_SIZE + overflow;
        for i in 0..total {
            sender.queue_packet(&[i as u8; 40], Some(i as u32), 1_000 + i as u64);
        }

        let outcome = sender.flush(&socket).await;
        assert!(outcome.error.is_none(), "a reachable peer must not error");
        assert_eq!(
            outcome.accepted.len(),
            BATCH_SEND_SIZE,
            "one flush commits at most BATCH_SEND_SIZE datagrams"
        );
        assert_eq!(
            outcome.accepted.first().copied(),
            Some((Some(0), 1_000)),
            "the committed records are the queue-order prefix"
        );
        assert_eq!(
            sender.queued_count() as usize,
            overflow,
            "the unsent suffix stays queued"
        );

        let second = sender.flush(&socket).await;
        assert!(second.error.is_none());
        assert_eq!(second.accepted.len(), overflow, "the suffix drains next");
        assert_eq!(
            second.accepted.first().copied(),
            Some((
                Some(BATCH_SEND_SIZE as u32 as i32),
                1_000 + BATCH_SEND_SIZE as u64
            )),
            "the suffix resumes exactly where the prefix stopped"
        );
        assert_eq!(sender.queued_count(), 0);

        let mut buf = [0u8; 128];
        let (n, _) = receiver.recv_from(&mut buf).await.unwrap();
        assert_eq!(n, 40, "DATA is sent verbatim, never padded");
    }

    /// A transmit that accepts nothing commits nothing and reports the error;
    /// the queue is left intact for the caller's recovery to discard.
    #[tokio::test]
    async fn flush_with_no_accepted_datagrams_commits_nothing() {
        let conn = unsendable_connection().await;
        let socket: Arc<BatchUdpSocket> = conn.socket.clone();
        let mut sender = BatchSender::new();
        for i in 0..4 {
            sender.queue_packet(&[0xabu8; 24], Some(i), 500);
        }

        let outcome = sender.flush(&socket).await;
        assert!(
            outcome.accepted.is_empty(),
            "nothing may be committed when nothing was transmitted"
        );
        assert!(outcome.error.is_some(), "a hard send error must surface");
        assert_eq!(sender.queued_count(), 4, "the queue is retained verbatim");
    }

    /// ADR-002: byte accounting happens at queue time and must not move to
    /// transmit time, so a failed flush still counts the bytes.
    #[tokio::test]
    async fn telemetry_byte_accounting_stays_at_queue_time() {
        let mut conn = unsendable_connection().await;
        conn.queue_data_packet(&[0u8; 100], Some(1), now_ms());
        assert_eq!(conn.session_bytes_sent(), 100);
        assert!(conn.flush_batch().await.is_err());
        assert_eq!(
            conn.session_bytes_sent(),
            100,
            "bytes_sent_total is counted at queue_data_packet, not on transmit"
        );
    }

    // ------------------------------------------------------------------
    // Flush call sites route hard errors into recovery
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn periodic_flush_hard_error_recovers_and_cleans_the_tracker() {
        let mut connections = vec![unsendable_connection().await];
        connections[0].connected = true;
        let conn_id = connections[0].conn_id;
        let mut seq_tracker = SequenceTracker::new();
        let now = now_ms();

        connections[0].queue_data_packet(&[1u8; 32], Some(7), now);
        seq_tracker.insert(7, conn_id, now);

        flush_all_batches(&mut connections, &mut seq_tracker).await;

        assert!(
            !connections[0].connected,
            "a periodic-flush hard error must mark the link for recovery"
        );
        assert_eq!(
            seq_tracker.get(7, now),
            None,
            "recovery must drop the link's sequence entries"
        );
    }

    #[tokio::test]
    async fn switch_flush_hard_error_recovers_the_previous_connection() {
        let mut connections = vec![
            unsendable_connection().await,
            create_test_connection().await,
        ];
        connections[0].connected = true;
        let previous_id = connections[0].conn_id;
        let mut seq_tracker = SequenceTracker::new();
        let mut last_selected_idx = Some(0usize);
        let mut last_switch_time_ms = 0u64;
        let now = now_ms();

        connections[0].queue_data_packet(&[2u8; 48], Some(11), now);
        seq_tracker.insert(11, previous_id, now);

        // Switching to uplink 1 flushes uplink 0's queue; that flush hard-fails.
        forward_via_connection(
            1,
            &[3u8; 48],
            Some(12),
            &mut connections,
            &mut last_selected_idx,
            &mut last_switch_time_ms,
            &mut seq_tracker,
            now,
        )
        .await;

        assert!(
            !connections[0].connected,
            "the PREVIOUS connection is the one whose flush failed"
        );
        assert_eq!(
            seq_tracker.get(11, now),
            None,
            "the previous connection's sequence entries are dropped"
        );
        assert_eq!(
            seq_tracker.get(12, now),
            Some(connections[1].conn_id),
            "the newly selected link's tracking is untouched"
        );
    }

    /// After a link is recovered, a NAK naming a sequence it never actually
    /// transmitted must not be routed back to it or penalize it.
    #[tokio::test]
    async fn late_nak_after_recovery_does_not_penalize_the_link() {
        let mut connections = vec![unsendable_connection().await];
        connections[0].connected = true;
        let conn_id = connections[0].conn_id;
        let mut seq_tracker = SequenceTracker::new();
        let now = now_ms();

        connections[0].queue_data_packet(&[4u8; 64], Some(4242), now);
        seq_tracker.insert(4242, conn_id, now);
        flush_all_batches(&mut connections, &mut seq_tracker).await;

        assert_eq!(seq_tracker.get(4242, now), None);
        connections[0].handle_nak(4242);
        assert_eq!(
            connections[0].total_nak_count(),
            0,
            "a NAK for a never-transmitted sequence must not penalize the recovered link"
        );
    }

    // ------------------------------------------------------------------
    // Unconnected sockets: source policy
    // ------------------------------------------------------------------

    /// Uplink sockets are unconnected, so a datagram from a source other than
    /// the resolved receiver IS processed (multi-homed / NAT receivers), and is
    /// counted rather than dropped.
    #[tokio::test]
    async fn foreign_source_datagram_is_processed_and_counted() {
        let (conn, expected_peer) = reachable_connection().await;
        let uplink_addr = conn.socket.local_addr().expect("uplink must be bound");
        drop(expected_peer);

        let (packet_tx, mut packet_rx) = create_uplink_channel();
        let reader = spawn_reader(
            conn.conn_id,
            conn.label.clone(),
            conn.socket.clone(),
            packet_tx,
        );

        let stranger = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        stranger.send_to(&[0xeeu8; 16], uplink_addr).await.unwrap();

        let packet = tokio::time::timeout(std::time::Duration::from_secs(2), packet_rx.recv())
            .await
            .expect("foreign-source datagram must still be delivered")
            .expect("channel open");
        assert_eq!(packet.bytes.len(), 16);
        assert_eq!(
            conn.foreign_source_datagrams(),
            1,
            "a foreign source is counted, never dropped"
        );

        reader.handle.abort();
    }

    // ------------------------------------------------------------------
    // Registration guard trio
    // ------------------------------------------------------------------

    #[tokio::test]
    async fn failed_reg1_send_keeps_registration_immediately_retriable() {
        let mut reg = SrtlaRegistrationManager::new();
        let mut conn = create_test_connection_with_failing_send().await;

        reg.send_reg1_to(0, &mut conn).await;

        assert_eq!(
            reg.pending_reg2_idx(),
            None,
            "a failed REG1 send must not open a phantom pending window"
        );
        assert_eq!(reg.reg1_target_idx(), Some(0));
        assert!(
            reg.reg1_next_send_at_ms() <= now_ms(),
            "the handshake must stay immediately retriable"
        );
    }

    #[tokio::test]
    async fn failed_reg2_send_leaves_the_reg3_gate_closed() {
        let mut reg = SrtlaRegistrationManager::new();
        let mut conn = create_test_connection_with_failing_send().await;

        reg.send_reg2_to(0, &mut conn).await;

        assert!(
            !reg.is_awaiting_reg3(0),
            "a REG2 that never left the host must not arm the REG3 gate"
        );
    }

    /// A failed REG2 *resend* must also revoke the grant an earlier successful
    /// REG2 left on the same index: "armed only on a send that left the host"
    /// has to hold for the current socket generation, not a stale one.
    #[tokio::test]
    async fn failed_reg2_resend_revokes_a_stale_pre_existing_grant() {
        let mut reg = SrtlaRegistrationManager::new();
        let (mut good, _peer) = reachable_connection().await;
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (instant_tx, _instant_rx) = tokio::sync::mpsc::unbounded_channel();
        let reg3 = [(SRTLA_TYPE_REG3 >> 8) as u8, (SRTLA_TYPE_REG3 & 0xff) as u8];

        reg.send_reg2_to(0, &mut good).await;
        assert!(reg.is_awaiting_reg3(0), "the first REG2 arms the gate");

        let mut broken = unsendable_connection().await;
        reg.send_reg2_to(0, &mut broken).await;

        assert!(
            !reg.is_awaiting_reg3(0),
            "a failed REG2 resend must leave no stale grant behind"
        );

        broken
            .process_packet(0, &mut reg, &listener, &instant_tx, None, &reg3)
            .await
            .unwrap();
        assert!(
            !broken.connected,
            "a REG3 riding the revoked grant must not connect the uplink"
        );
        assert_eq!(reg.out_of_phase_reg3(), 1);
    }

    // ------------------------------------------------------------------
    // REG_ERR phase gate
    // ------------------------------------------------------------------

    /// A forged 2-byte REG_ERR is the cheapest possible remote DoS: without a
    /// phase gate it force-disconnects any established, forwarding uplink.
    #[tokio::test]
    async fn out_of_phase_reg_err_does_not_disconnect_a_live_uplink() {
        let (mut conn, _peer) = reachable_connection().await;
        let mut reg = SrtlaRegistrationManager::new();
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (instant_tx, _instant_rx) = tokio::sync::mpsc::unbounded_channel();
        let reg3 = [(SRTLA_TYPE_REG3 >> 8) as u8, (SRTLA_TYPE_REG3 & 0xff) as u8];
        let reg_err = [
            (SRTLA_TYPE_REG_ERR >> 8) as u8,
            (SRTLA_TYPE_REG_ERR & 0xff) as u8,
        ];

        reg.send_reg2_to(0, &mut conn).await;
        conn.process_packet(0, &mut reg, &listener, &instant_tx, None, &reg3)
            .await
            .unwrap();
        assert!(conn.connected, "the uplink is established");
        assert!(
            !reg.is_awaiting_reg3(0),
            "no registration is in flight any more"
        );

        conn.process_packet(0, &mut reg, &listener, &instant_tx, None, &reg_err)
            .await
            .unwrap();
        conn.process_packet(0, &mut reg, &listener, &instant_tx, None, &reg_err)
            .await
            .unwrap();

        assert!(
            conn.connected,
            "an out-of-phase REG_ERR must not disconnect an established uplink"
        );
        assert_eq!(
            reg.out_of_phase_reg_err(),
            2,
            "every rejected frame remains counted after WARN output is demoted"
        );
        assert_eq!(
            reg.pending_reg2_idx(),
            None,
            "no handshake state existed to clear"
        );
    }

    /// The REG1/REG2 fields are single-slot globals: a REG_ERR about uplink A
    /// must not abort uplink B's concurrent handshake.
    #[tokio::test]
    async fn out_of_phase_reg_err_does_not_damage_another_uplinks_handshake() {
        let mut reg = SrtlaRegistrationManager::new();
        let (mut conn_a, _peer_a) = reachable_connection().await;
        let (mut conn_b, _peer_b) = reachable_connection().await;
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (instant_tx, _instant_rx) = tokio::sync::mpsc::unbounded_channel();
        let reg_err = [
            (SRTLA_TYPE_REG_ERR >> 8) as u8,
            (SRTLA_TYPE_REG_ERR & 0xff) as u8,
        ];

        reg.send_reg1_to(1, &mut conn_b).await;
        assert_eq!(reg.pending_reg2_idx(), Some(1));
        let pending_timeout = reg.pending_timeout_at_ms();

        conn_a
            .process_packet(0, &mut reg, &listener, &instant_tx, None, &reg_err)
            .await
            .unwrap();

        assert_eq!(
            reg.pending_reg2_idx(),
            Some(1),
            "uplink B keeps its in-flight REG2 window"
        );
        assert_eq!(
            reg.reg1_target_idx(),
            Some(1),
            "uplink B keeps its REG1 target"
        );
        assert_eq!(reg.pending_timeout_at_ms(), pending_timeout);
        assert_eq!(reg.out_of_phase_reg_err(), 1);
    }

    /// A REG_ERR that answers a handshake actually in flight — in either the
    /// awaiting-REG2 or the awaiting-REG3 phase — still tears that attempt down.
    #[tokio::test]
    async fn in_phase_reg_err_still_aborts_the_registration() {
        let mut reg = SrtlaRegistrationManager::new();
        let (mut conn, _peer) = reachable_connection().await;
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (instant_tx, _instant_rx) = tokio::sync::mpsc::unbounded_channel();
        let reg_err = [
            (SRTLA_TYPE_REG_ERR >> 8) as u8,
            (SRTLA_TYPE_REG_ERR & 0xff) as u8,
        ];

        reg.send_reg1_to(0, &mut conn).await;
        assert_eq!(reg.pending_reg2_idx(), Some(0));
        conn.connected = true;

        conn.process_packet(0, &mut reg, &listener, &instant_tx, None, &reg_err)
            .await
            .unwrap();

        assert!(!conn.connected, "an in-phase REG_ERR tears the link down");
        assert_eq!(reg.pending_reg2_idx(), None);
        assert_eq!(reg.reg1_target_idx(), None);
        assert_eq!(reg.pending_timeout_at_ms(), 0);
        assert_eq!(reg.out_of_phase_reg_err(), 0);

        let mut awaiting = SrtlaRegistrationManager::new();
        let (mut conn2, _peer2) = reachable_connection().await;
        awaiting.send_reg2_to(0, &mut conn2).await;
        assert!(awaiting.is_awaiting_reg3(0));
        conn2.connected = true;

        conn2
            .process_packet(0, &mut awaiting, &listener, &instant_tx, None, &reg_err)
            .await
            .unwrap();

        assert!(
            !conn2.connected,
            "a REG_ERR while awaiting REG3 also aborts the attempt"
        );
        assert!(
            !awaiting.is_awaiting_reg3(0),
            "the grant is revoked by the in-phase REG_ERR"
        );
        assert_eq!(awaiting.out_of_phase_reg_err(), 0);
    }

    #[test]
    fn reg2_with_a_foreign_id_prefix_is_rejected() {
        let mut reg = SrtlaRegistrationManager::new();
        let ours = reg.srtla_id;
        reg.set_pending_reg2_idx(Some(0));

        let mut forged = ours;
        forged[0] ^= 0xff;
        forged[SRTLA_ID_LEN / 2..].fill(0xab);
        reg.process_registration_packet(0, &create_reg2_packet(&forged));

        assert_eq!(reg.srtla_id, ours, "a forged REG2 must not replace our id");
        assert!(!reg.broadcast_reg2_pending());
        assert_eq!(
            reg.pending_reg2_idx(),
            Some(0),
            "the handshake stays pending for the genuine reply"
        );

        let mut genuine = ours;
        genuine[SRTLA_ID_LEN / 2..].fill(0xcd);
        reg.process_registration_packet(0, &create_reg2_packet(&genuine));
        assert_eq!(
            reg.srtla_id, genuine,
            "the matching-prefix REG2 is accepted"
        );
        assert!(reg.broadcast_reg2_pending());
    }

    #[test]
    fn out_of_phase_reg3_is_counted_and_ignored() {
        let mut reg = SrtlaRegistrationManager::new();
        let reg3 = [(SRTLA_TYPE_REG3 >> 8) as u8, (SRTLA_TYPE_REG3 & 0xff) as u8];

        reg.process_registration_packet(0, &reg3);
        reg.process_registration_packet(0, &reg3);
        assert!(
            !reg.has_connected,
            "REG3 on an uplink that was never sent a REG2 must not connect it"
        );
        assert_eq!(
            reg.out_of_phase_reg3(),
            2,
            "every rejected frame remains counted after WARN output is demoted"
        );

        reg.arm_reg3_gate(0);
        reg.process_registration_packet(0, &reg3);
        assert!(
            reg.has_connected,
            "an in-phase REG3 still connects the link"
        );
        assert_eq!(reg.out_of_phase_reg3(), 2);
    }

    #[tokio::test]
    async fn replayed_reg3_does_not_wipe_a_live_connection() {
        let (mut conn, _receiver) = reachable_connection().await;
        let mut reg = SrtlaRegistrationManager::new();
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (instant_tx, _instant_rx) = tokio::sync::mpsc::unbounded_channel();
        let reg3 = [(SRTLA_TYPE_REG3 >> 8) as u8, (SRTLA_TYPE_REG3 & 0xff) as u8];

        reg.send_reg2_to(0, &mut conn).await;
        assert!(reg.is_awaiting_reg3(0));

        conn.process_packet(0, &mut reg, &listener, &instant_tx, None, &reg3)
            .await
            .unwrap();
        assert!(
            conn.connected,
            "the first in-phase REG3 registers the uplink"
        );
        assert!(
            !reg.is_awaiting_reg3(0),
            "the one-shot grant must be consumed by the REG3 it authorized"
        );

        let mut connections: SmallVec<SrtlaConnection, 4> = SmallVec::new();
        connections.push(conn);
        let mut last_selected_idx = None;
        let mut last_switch_time_ms = 0u64;
        let mut seq_tracker = SequenceTracker::new();
        let now = now_ms();
        forward_via_connection(
            0,
            &[7u8; 48],
            Some(9),
            &mut connections,
            &mut last_selected_idx,
            &mut last_switch_time_ms,
            &mut seq_tracker,
            now,
        )
        .await;
        flush_all_batches(&mut connections, &mut seq_tracker).await;

        let live_log = connections[0].packet_log.clone();
        let live_in_flight = connections[0].in_flight_packets;
        assert!(
            !live_log.is_empty() && live_in_flight > 0,
            "the uplink must carry real in-flight state before the replay"
        );

        connections[0]
            .process_packet(0, &mut reg, &listener, &instant_tx, None, &reg3)
            .await
            .unwrap();

        assert_eq!(
            reg.out_of_phase_reg3(),
            1,
            "the replayed REG3 must be rejected by the phase gate and counted"
        );
        assert_eq!(
            connections[0].packet_log, live_log,
            "a replayed REG3 must not clear the in-flight packet log of a live uplink"
        );
        assert_eq!(
            connections[0].in_flight_packets, live_in_flight,
            "a replayed REG3 must not reset the in-flight counter of a live uplink"
        );
        assert!(connections[0].connected, "the uplink stays connected");
    }

    #[tokio::test]
    async fn reg2_broadcast_arms_the_reg3_gate_for_every_uplink() {
        let mut reg = SrtlaRegistrationManager::new();
        let (conn_a, _peer_a) = reachable_connection().await;
        let (conn_b, _peer_b) = reachable_connection().await;
        let mut connections: SmallVec<SrtlaConnection, 4> = SmallVec::new();
        connections.push(conn_a);
        connections.push(conn_b);

        reg.set_broadcast_reg2_pending(true);
        reg.reg_driver_send_if_needed(&mut connections).await;

        assert!(reg.is_awaiting_reg3(0));
        assert!(reg.is_awaiting_reg3(1));
        assert!(
            !reg.broadcast_reg2_pending(),
            "a fully successful broadcast is not retried"
        );
    }

    #[tokio::test]
    async fn reg2_broadcast_retry_skips_already_connected_uplinks() {
        let mut reg = SrtlaRegistrationManager::new();
        let (conn_a, _peer_a) = reachable_connection().await;
        let conn_b = unsendable_connection().await;
        let mut connections: SmallVec<SrtlaConnection, 4> = SmallVec::new();
        connections.push(conn_a);
        connections.push(conn_b);
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (instant_tx, _instant_rx) = tokio::sync::mpsc::unbounded_channel();
        let reg3 = [(SRTLA_TYPE_REG3 >> 8) as u8, (SRTLA_TYPE_REG3 & 0xff) as u8];

        reg.set_broadcast_reg2_pending(true);
        reg.reg_driver_send_if_needed(&mut connections).await;
        assert!(reg.is_awaiting_reg3(0), "uplink 0's REG2 left the host");
        assert!(!reg.is_awaiting_reg3(1), "uplink 1's REG2 send failed");
        assert!(
            reg.broadcast_reg2_pending(),
            "a partially failed broadcast is retried on the next tick"
        );

        connections[0]
            .process_packet(0, &mut reg, &listener, &instant_tx, None, &reg3)
            .await
            .unwrap();
        assert!(connections[0].connected);
        assert!(
            !reg.is_awaiting_reg3(0),
            "the one-shot grant is consumed by the REG3 it authorized"
        );

        let mut last_selected_idx = None;
        let mut last_switch_time_ms = 0u64;
        let mut seq_tracker = SequenceTracker::new();
        let now = now_ms();
        forward_via_connection(
            0,
            &[3u8; 48],
            Some(11),
            &mut connections,
            &mut last_selected_idx,
            &mut last_switch_time_ms,
            &mut seq_tracker,
            now,
        )
        .await;
        flush_all_batches(&mut connections, &mut seq_tracker).await;
        let live_log = connections[0].packet_log.clone();
        let live_in_flight = connections[0].in_flight_packets;
        assert!(!live_log.is_empty() && live_in_flight > 0);

        reg.reg_driver_send_if_needed(&mut connections).await;
        assert!(
            !reg.is_awaiting_reg3(0),
            "the retry pass must not re-arm the consumed grant of a connected uplink"
        );

        connections[0]
            .process_packet(0, &mut reg, &listener, &instant_tx, None, &reg3)
            .await
            .unwrap();
        assert_eq!(
            reg.out_of_phase_reg3(),
            1,
            "a REG3 replayed after the retry tick stays out of phase"
        );
        assert_eq!(
            connections[0].packet_log, live_log,
            "the live uplink's packet log survives the retry + replay"
        );
        assert_eq!(
            connections[0].in_flight_packets, live_in_flight,
            "the live uplink's in-flight counter survives the retry + replay"
        );
    }
}
