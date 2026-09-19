use tokio::net::UdpSocket;

use super::SrtlaConnection;
use crate::protocol::{WINDOW_DECR, WINDOW_MIN, WINDOW_MULT};
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

async fn accepted_link(clock: &TestClock, rtt: Option<u64>) -> (SrtlaConnection, UdpSocket) {
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    if let Some(rtt) = rtt {
        conn.rtt.update_estimate(rtt);
    }
    for seq in 0..100 {
        conn.queue_data_packet(&[0; 16], Some(seq), 10_000);
    }
    clock.set(10_015);
    while conn.has_queued_packets() {
        conn.flush_batch().await.unwrap();
    }
    (conn, receiver)
}

#[tokio::test]
async fn premature_nak_uses_kernel_acceptance_not_queue_time() {
    // Given a 15ms batch delay and a measured 2ms RTT (5ms protection floor).
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted_link(&clock, Some(2)).await;
    assert_eq!(conn.packet_log.get(&0), Some(&10_000));
    assert_eq!(conn.delivery.sent_ms(0), Some(10_015));
    let window = conn.window;
    // When the NAK arrives 18ms after queueing, but only 3ms after acceptance.
    clock.set(10_018);
    assert!(conn.handle_nak(0));
    // Then no congestion or loss evidence changes and the packet stays in flight.
    assert_eq!(conn.window, window);
    assert_eq!(conn.congestion.nak_count, 0);
    assert_eq!(conn.packet_log.get(&0), Some(&10_000));
    assert_eq!(conn.in_flight_packets, 100);
    assert_eq!(conn.premature_nak_count, 1);
    assert_eq!(conn.premature_streak, 1);
    conn.loss.advance(11_000);
    assert_eq!(conn.loss.last_value(), Some(0.0));
}

#[tokio::test]
async fn premature_nak_boundary_is_penalised_normally() {
    // Given a measured link with accepted DATA.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted_link(&clock, Some(2)).await;
    let window = conn.window;
    // When age equals the protection threshold exactly.
    clock.set(10_020);
    assert!(conn.handle_nak(0));
    // Then the normal window, packet-log and cohort penalty applies.
    assert_eq!(
        conn.window,
        (window - WINDOW_DECR).max(WINDOW_MIN * WINDOW_MULT)
    );
    assert_eq!(conn.congestion.nak_count, 1);
    assert!(!conn.packet_log.contains_key(&0));
    assert_eq!(conn.in_flight_packets, 99);
    assert_eq!(conn.premature_nak_count, 0);
    conn.loss.advance(11_000);
    assert_eq!(conn.loss.last_value(), Some(0.01));
}

#[tokio::test]
async fn premature_nak_without_rtt_sample_is_inactive() {
    // Given accepted DATA but no measured RTT, despite the default 200ms baseline.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted_link(&clock, None).await;
    assert!(!conn.has_rtt_sample());
    assert_eq!(conn.premature_nak_threshold_ms(), 0.0);
    // When fresh NAKs arrive at the acceptance instant.
    for seq in 0..4 {
        assert!(conn.handle_nak(seq));
    }
    // Then every owned NAK takes the penalty path.
    assert_eq!(conn.congestion.nak_count, 4);
    assert_eq!(conn.in_flight_packets, 96);
    assert_eq!(conn.premature_nak_count, 0);
}

#[tokio::test]
async fn premature_nak_without_accept_timestamp_is_inactive() {
    // Given only the congestion log's queue timestamp, never accepted-send evidence.
    let _clock = TestClock::new(10_000);
    let mut conn = crate::test_helpers::create_test_connection().await;
    conn.rtt.update_estimate(100);
    conn.register_packet(7, 10_000);
    // When a NAK names this sequence immediately.
    assert!(conn.handle_nak(7));
    // Then missing acceptance cannot invent protection.
    assert_eq!(conn.congestion.nak_count, 1);
    assert_eq!(conn.in_flight_packets, 0);
    assert_eq!(conn.premature_nak_count, 0);
}

#[tokio::test]
async fn premature_nak_threshold_clamps_measured_minimum() {
    for (rtt, expected) in [
        (1, 5.0),
        (10, 5.0),
        (11, 5.5),
        (200, 100.0),
        (1000, 500.0),
        (10_000, 500.0),
    ] {
        // Given an independently measured baseline, including both clamp boundaries.
        let mut conn = crate::test_helpers::create_test_connection().await;
        conn.rtt.update_estimate(rtt);
        // When the threshold is derived, Then it retains half-ms precision.
        assert_eq!(conn.premature_nak_threshold_ms(), expected);
    }
}

#[tokio::test]
async fn premature_nak_fourth_consecutive_report_is_penalised() {
    // Given four different outstanding sequences on one measured link.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted_link(&clock, Some(100)).await;
    let window = conn.window;
    // When four premature reports arrive without intervening RTT evidence.
    for seq in 0..4 {
        assert!(conn.handle_nak(seq));
        // Then exactly the first three are suppressed, and the fourth resets the cap.
        assert_eq!(conn.premature_streak, u8::try_from((seq + 1) % 4).unwrap());
        assert_eq!(conn.congestion.nak_count, i32::from(seq == 3));
        assert_eq!(conn.window, window - i32::from(seq == 3) * WINDOW_DECR);
    }
    assert_eq!(conn.premature_nak_count, 3);
    assert_eq!(conn.in_flight_packets, 99);
    assert!((0..3).all(|seq| conn.packet_log.contains_key(&seq)));
    assert!(!conn.packet_log.contains_key(&3));
    conn.loss.advance(11_000);
    assert_eq!(conn.loss.last_value(), Some(0.01));
}

#[tokio::test]
async fn premature_nak_repeated_sequence_obeys_cap() {
    // Given a repeated report for the same outstanding sequence.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted_link(&clock, Some(100)).await;
    // When four reports name it, Then only the fourth removes and penalises it.
    for _ in 0..4 {
        assert!(conn.handle_nak(0));
    }
    assert_eq!(conn.premature_nak_count, 3);
    assert_eq!(conn.congestion.nak_count, 1);
    assert_eq!(conn.premature_streak, 0);
    assert!(!conn.handle_nak(0));
}

#[tokio::test]
async fn premature_nak_mature_penalty_resets_streak() {
    // Given two suppressions followed by an aging packet.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted_link(&clock, Some(100)).await;
    assert!(conn.handle_nak(0));
    assert!(conn.handle_nak(1));
    clock.set(10_065);
    // When the next NAK is mature, Then its normal penalty clears the streak.
    assert!(conn.handle_nak(2));
    assert_eq!(conn.premature_streak, 0);
    assert_eq!(conn.premature_nak_count, 2);
    assert_eq!(conn.congestion.nak_count, 1);
}

#[tokio::test]
async fn premature_nak_unknown_sequence_preserves_streak() {
    // Given one suppressed report on a measured link.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted_link(&clock, Some(100)).await;
    assert!(conn.handle_nak(0));
    // When a foreign NAK arrives, Then it neither claims ownership nor clears the cap.
    assert!(!conn.handle_nak(500));
    assert_eq!(conn.premature_streak, 1);
    assert_eq!(conn.premature_nak_count, 1);
}

// Linux `sendmmsg` reports the oversized datagram as a hard error after the accepted prefix.
#[cfg(target_os = "linux")]
#[tokio::test]
async fn premature_nak_partial_send_protects_only_accepted_prefix() {
    // Given a valid packet followed by an oversized packet in one batch.
    let clock = TestClock::new(10_000);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    conn.rtt.update_estimate(2);
    conn.queue_data_packet(&[0; 16], Some(0), 10_000);
    conn.queue_data_packet(&[0; 70_000], Some(1), 10_000);
    clock.set(10_015);
    assert!(conn.flush_batch().await.is_err());
    assert_eq!(conn.delivery.sent_ms(0), Some(10_015));
    assert_eq!(conn.delivery.sent_ms(1), None);
    // When both sequences are reported, Then only the accepted prefix is handled/protected.
    clock.set(10_018);
    assert!(conn.handle_nak(0));
    assert!(!conn.handle_nak(1));
    assert_eq!(conn.premature_nak_count, 1);
    assert_eq!(conn.in_flight_packets, 1);
    assert_eq!(conn.congestion.nak_count, 0);
}

// The sequential fallback swallows an error once a non-empty prefix is committed.
#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn premature_nak_partial_send_protects_only_accepted_prefix_sequential_fallback() {
    // Given a valid packet followed by an oversized packet in one batch.
    let clock = TestClock::new(10_000);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    conn.rtt.update_estimate(2);
    conn.queue_data_packet(&[0; 16], Some(0), 10_000);
    conn.queue_data_packet(&[0; 70_000], Some(1), 10_000);
    clock.set(10_015);
    assert!(conn.flush_batch().await.is_ok());
    assert_eq!(conn.delivery.sent_ms(0), Some(10_015));
    assert_eq!(conn.delivery.sent_ms(1), None);
    // When both sequences are reported, Then only the accepted prefix is handled/protected.
    clock.set(10_018);
    assert!(conn.handle_nak(0));
    assert!(!conn.handle_nak(1));
    assert_eq!(conn.premature_nak_count, 1);
    assert_eq!(conn.in_flight_packets, 1);
    assert_eq!(conn.congestion.nak_count, 0);
}
