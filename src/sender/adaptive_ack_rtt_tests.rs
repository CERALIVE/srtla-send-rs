use tokio::net::UdpSocket;

use crate::connection::SrtlaIncoming;
use crate::registration::SrtlaRegistrationManager;
use crate::sender::ack::{AckContext, AckPolicy, apply_srtla_ack};
use crate::sender::packet_handler::process_connection_events_at;
use crate::sender::sequence::SequenceTracker;
use crate::stats::SharedStats;
use crate::test_helpers::create_test_connection;
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn adaptive_retransmission_ack_does_not_lower_the_path_floor() {
    // Given a wire-marked retransmission; its ACK may belong to an earlier copy.
    let clock = TestClock::new(995);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn =
        crate::test_helpers::create_test_connection_to(receiver.local_addr().unwrap()).await;
    conn.rtt.update_estimate(60);
    let mut packet = [0_u8; 20];
    packet[..4].copy_from_slice(&42_u32.to_be_bytes());
    packet[4] = 0x04;
    conn.queue_data_packet(&packet, Some(42), 995);
    conn.flush_batch().await.unwrap();
    clock.set(1000);
    let mut conns = vec![conn];
    // When its specific ACK arrives only 5ms after the retransmission.
    apply_srtla_ack(
        &mut conns,
        42,
        AckContext {
            arrival_idx: 0,
            reader_generation: 0,
            policy: AckPolicy::Adaptive,
        },
    );
    // Then it proves delivery, but cannot manufacture a 5ms RTT (Karn ambiguity).
    assert_eq!(conns[0].delivery.proof_age_ms(1000), Some(0));
    assert_eq!(conns[0].rtt.slow_min_rtt_ms(), 60.0);
    assert_eq!(conns[0].in_flight_packets, 0);
}

#[tokio::test]
async fn adaptive_cumulative_ack_cannot_fabricate_a_path_rtt() {
    // Given a 60ms path and a cumulative ACK naming the next expected sequence,
    // that sequence's recent send is not a received-packet timestamp.
    let clock = TestClock::new(1000);
    let mut conn = create_test_connection().await;
    conn.rtt.update_estimate(60);
    conn.register_packet(100, 970);
    conn.register_packet(101, 980);
    let mut tracker = SequenceTracker::new();
    tracker.insert(100, conn.conn_id, 970);
    let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let (forward, _received) = tokio::sync::mpsc::unbounded_channel();
    let mut incoming = SrtlaIncoming::default();
    incoming.ack_numbers.push(100);
    let mut conns = vec![conn];
    // When the production adaptive dispatch receives the cumulative ACK.
    process_connection_events_at(
        AckContext {
            arrival_idx: 0,
            reader_generation: 0,
            policy: AckPolicy::Adaptive,
        },
        &mut conns,
        &mut SrtlaRegistrationManager::new(),
        &forward,
        None,
        &socket,
        &tracker,
        Some(incoming),
        &SharedStats::new(),
    )
    .await
    .unwrap();
    // Then congestion pruning still works, but neither RTT floor nor queue evidence changes.
    assert!(!conns[0].packet_log.contains_key(&100));
    assert!(conns[0].packet_log.contains_key(&101));
    assert_eq!(conns[0].rtt.slow_min_rtt_ms(), 60.0);
    clock.set(2100);
    conns[0].rtt.update_estimate(60);
    assert_eq!(conns[0].rtt.queue_delay_ms(), 0.0);
}
