use tokio::net::UdpSocket;

use super::SrtlaConnection;
use crate::test_helpers::{create_test_connection_to, create_test_connection_with_failing_send};
use crate::utils::now_ms;
use crate::utils::test_clock::TestClock;

async fn accepted(count: u32) -> (SrtlaConnection, UdpSocket) {
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(peer.local_addr().unwrap()).await;
    for seq in 0..count {
        conn.queue_data_packet(&[0; 16], Some(seq), now_ms());
    }
    while conn.has_queued_packets() {
        conn.flush_batch().await.unwrap();
    }
    (conn, peer)
}

#[tokio::test]
async fn loss_counts_unique_attributed_normal_data_naks() {
    // Given 100 accepted normal DATA packets and a sequence-free control frame.
    let clock = TestClock::new(10_000);
    let (mut conn, _peer) = accepted(100).await;
    conn.queue_data_packet(&[0; 16], None, now_ms());
    conn.flush_batch().await.unwrap();
    // When 20 owned sequences are NAKed twice, plus one foreign sequence.
    for seq in 0..20 {
        assert!(conn.handle_nak(seq));
        assert!(!conn.handle_nak(seq));
    }
    assert!(!conn.handle_nak(500));
    clock.set(11_000);
    conn.loss.advance(now_ms());
    // Then only 20 unique normal-log hits / 100 accepted sends enter the EWMA.
    assert_eq!(conn.loss.last_value(), Some(0.2));
    assert!(conn.loss.loss_cohort_ok(now_ms(), 10_000));
    assert_eq!(conn.loss.probe_loss(), None);
}

// Linux `sendmmsg` surfaces the rejected oversized entry as a hard error.
#[cfg(target_os = "linux")]
#[tokio::test]
async fn loss_partial_send_counts_only_the_kernel_accepted_prefix() {
    // Given 99 accepted packets; the next batch has one valid then oversized DATA.
    let clock = TestClock::new(10_000);
    let (mut conn, _peer) = accepted(99).await;
    conn.queue_data_packet(&[0; 16], Some(99), 0);
    conn.queue_data_packet(&[0; 70_000], Some(100), 0);
    conn.queue_data_packet(&[0; 16], Some(101), 0);
    clock.set(10_500);
    // When the kernel accepts the prefix but rejects the rest of the batch.
    assert!(conn.flush_batch().await.is_err());
    assert!(conn.handle_nak(99));
    assert!(!conn.handle_nak(100));
    assert!(!conn.handle_nak(101));
    clock.set(11_000);
    conn.loss.advance(now_ms());
    // Then the load floor is exactly met, using acceptance time rather than queue time.
    assert_eq!(conn.loss.last_value(), Some(0.01));
    assert_eq!(conn.loss.last_cohort_ms(), Some(11_000));
}

// The sequential fallback commits the same prefix but reports success.
#[cfg(not(target_os = "linux"))]
#[tokio::test]
async fn loss_partial_send_counts_only_the_kernel_accepted_prefix_sequential_fallback() {
    // Given 99 accepted packets; the next batch has one valid then oversized DATA.
    let clock = TestClock::new(10_000);
    let (mut conn, _peer) = accepted(99).await;
    conn.queue_data_packet(&[0; 16], Some(99), 0);
    conn.queue_data_packet(&[0; 70_000], Some(100), 0);
    conn.queue_data_packet(&[0; 16], Some(101), 0);
    clock.set(10_500);
    // When the sequential path accepts the prefix and stops at the oversized entry.
    assert!(conn.flush_batch().await.is_ok());
    assert!(conn.handle_nak(99));
    assert!(!conn.handle_nak(100));
    assert!(!conn.handle_nak(101));
    clock.set(11_000);
    conn.loss.advance(now_ms());
    // Then the load floor is exactly met, using acceptance time rather than queue time.
    assert_eq!(conn.loss.last_value(), Some(0.01));
    assert_eq!(conn.loss.last_cohort_ms(), Some(11_000));
}

#[tokio::test]
async fn loss_queued_and_failed_packets_cannot_satisfy_the_floor() {
    // Given a loaded queue on a deterministic send-failing socket.
    let clock = TestClock::new(10_000);
    let mut conn = create_test_connection_with_failing_send().await;
    for seq in 0..100 {
        conn.queue_data_packet(&[0; 16], Some(seq), now_ms());
    }
    clock.set(11_000);
    conn.loss.advance(now_ms());
    assert_eq!(conn.loss.last_value(), None);
    // When a flush accepts nothing.
    assert!(conn.flush_batch().await.is_err());
    clock.set(12_000);
    conn.loss.advance(now_ms());
    // Then neither queueing nor failed I/O creates loss evidence.
    assert_eq!(conn.loss.last_value(), None);
    assert_eq!(conn.loss.last_cohort_ms(), None);
}

#[tokio::test]
async fn loss_fifty_accepted_sends_and_fifty_naks_remain_unknown() {
    // Given a real loopback link with only fifty accepted normal sends.
    let clock = TestClock::new(10_000);
    let (mut conn, _peer) = accepted(50).await;
    // When every one of those sequences is NAKed.
    for seq in 0..50 {
        assert!(conn.handle_nak(seq));
    }
    clock.set(11_000);
    conn.loss.advance(now_ms());
    // Then even 100% observed loss is discarded below the load floor.
    assert_eq!(conn.loss.last_value(), None);
    assert!(!conn.loss.loss_cohort_ok(now_ms(), 10_000));
}

#[tokio::test]
async fn loss_recovery_discards_previous_socket_evidence() {
    // Given one qualified cohort followed by an incomplete cohort.
    let clock = TestClock::new(10_000);
    let (mut conn, _peer) = accepted(100).await;
    clock.set(11_000);
    conn.loss.advance(now_ms());
    assert_eq!(conn.loss.last_value(), Some(0.0));
    for _ in 0..100 {
        conn.loss.record_send(now_ms());
    }
    // When recovery invalidates the connection's packet generation.
    conn.mark_for_recovery();
    clock.set(12_000);
    conn.loss.advance(now_ms());
    // Then neither completed nor incomplete old-path evidence survives.
    assert_eq!(conn.loss.last_value(), None);
    assert_eq!(conn.loss.last_cohort_ms(), None);
}

#[tokio::test]
async fn loss_socket_replacement_discards_previous_evidence() {
    // Given qualified evidence on a socket that has not been marked for recovery.
    let clock = TestClock::new(10_000);
    let (mut conn, _peer) = accepted(100).await;
    clock.set(11_000);
    conn.loss.advance(now_ms());
    assert_eq!(conn.loss.last_value(), Some(0.0));
    // When reconnect replaces the socket directly.
    conn.reconnect().await.unwrap();
    // Then evidence cannot transfer to the new path.
    assert_eq!(conn.loss.last_value(), None);
    assert_eq!(conn.loss.last_cohort_ms(), None);
}

#[tokio::test]
async fn delayed_naks_are_charged_to_the_accepted_send_cohort() {
    // Given1000 accepted sends followed by a different cohort with only100 sends.
    let clock = TestClock::new(10_000);
    let (mut conn, _peer) = accepted(1000).await;
    clock.set(11_000);
    for seq in 1000..1100 {
        conn.queue_data_packet(&[0; 16], Some(seq), 11_000);
    }
    while conn.has_queued_packets() {
        conn.flush_batch().await.unwrap();
    }
    // When all1000 old sends are NAKed during the small new cohort.
    clock.set(11_200);
    for seq in 0..1000 {
        assert!(conn.handle_nak(seq));
    }
    clock.set(12_000);
    conn.loss.advance(12_000);
    // Then loss1.0 in the OLD cohort followed by0 in the NEW gives EWMA0.8, not2.0.
    assert_eq!(conn.loss.last_value(), Some(0.8));
    assert_eq!(conn.loss.last_cohort_ms(), Some(12_000));
}

#[tokio::test]
async fn delayed_subfloor_and_pre_recovery_naks_cannot_poison_new_evidence() {
    for recovered in [false, true] {
        // Given either a sub-floor old cohort or one predating qualified recovery.
        let clock = TestClock::new(10_000);
        let (mut conn, _peer) = accepted(if recovered { 100 } else { 50 }).await;
        clock.set(11_000);
        if recovered {
            conn.loss.begin_recovered_epoch(11_000);
        }
        for seq in 100..200 {
            conn.queue_data_packet(&[0; 16], Some(seq), 11_000);
        }
        while conn.has_queued_packets() {
            conn.flush_batch().await.unwrap();
        }
        // When old NAKs arrive after the new epoch/cohort started.
        for seq in 0..50 {
            assert!(conn.handle_nak(seq));
        }
        conn.loss.advance(12_000);
        // Then the current cohort remains loss-free; old evidence stays in its own scope.
        assert_eq!(conn.loss.last_value(), Some(0.0), "recovered={recovered}");
    }
}
