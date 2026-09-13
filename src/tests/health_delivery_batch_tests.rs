use tokio::net::UdpSocket;

use crate::test_helpers::create_test_connection_to;
use crate::utils::now_ms;
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn accepted_prefix_credits_actual_lengths_not_the_unsent_suffix() {
    // Given 40 differently sized DATA packets queued one second before acceptance.
    let _clock = TestClock::new(10_000);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    for seq in 0_u16..40 {
        conn.queue_data_packet(&vec![0; usize::from(seq) + 100], Some(u32::from(seq)), 9000);
    }
    // When only the bounded 32-packet prefix is flushed and ACKed.
    conn.flush_batch().await.unwrap();
    assert_eq!(conn.delivery.attempts_since_proof, 32);
    assert_eq!(conn.delivery.proof_age_ms(now_ms()), Some(0));
    for seq in 0..40 {
        conn.handle_srtla_ack_specific(seq, false);
    }
    // Then rate includes precisely sum(100..132), and the kernel saw those lengths.
    assert_eq!(conn.delivery.delivered_bps(now_ms()), 14_784.0);
    assert_eq!(conn.batch_sender.queued_count(), 8);
    let mut buffer = [0; 1500];
    for size in 100..132 {
        let (received, _) = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            receiver.recv_from(&mut buffer),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(received, size);
    }
}

#[tokio::test]
async fn hard_error_after_accepted_prefix_still_records_delivery_attempt() {
    // Given a valid datagram followed by a kernel-rejected oversized datagram.
    let _clock = TestClock::new(10_000);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    conn.queue_data_packet(&[0; 1316], Some(1), 9000);
    conn.queue_data_packet(&vec![0; 70_000], Some(2), 9000);
    // When the real batch send partially succeeds before returning EMSGSIZE.
    assert!(conn.flush_batch().await.is_err());
    // Then only the accepted DATA can prove delivery before caller recovery.
    assert_eq!(conn.delivery.attempts_since_proof, 1);
    assert_eq!(conn.batch_sender.queued_count(), 1);
    assert!(!conn.handle_srtla_ack_specific(2, false));
    assert_eq!(conn.delivery.last_data_proof_ms, 0);
    assert!(conn.handle_srtla_ack_specific(1, false));
    assert_eq!(conn.delivery.delivered_bps(now_ms()), 5264.0);
}
