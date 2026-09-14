use tokio::net::UdpSocket;

use crate::connection::SrtlaConnection;
use crate::sender::ack::{AckContext, AckPolicy, apply_srtla_ack};
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

#[path = "loss_settlement_tests.rs"]
mod settlement_tests;

#[path = "loss_deadline_tests.rs"]
mod deadline_tests;

async fn send(conn: &mut SrtlaConnection, sequences: std::ops::Range<u32>, retransmitted: bool) {
    for seq in sequences {
        let mut packet = [0_u8; 16];
        packet[..4].copy_from_slice(&seq.to_be_bytes());
        packet[4] = if retransmitted { 0x04 } else { 0 };
        conn.queue_data_packet(&packet, Some(seq), crate::utils::now_ms());
    }
    while conn.has_queued_packets() {
        conn.flush_batch().await.unwrap();
    }
}

fn ack(conns: &mut [SrtlaConnection], seq: i32, arrival: (usize, u32)) {
    apply_srtla_ack(
        conns,
        seq,
        AckContext {
            arrival_idx: arrival.0,
            reader_generation: arrival.1,
            policy: AckPolicy::Adaptive,
        },
    );
}

async fn check_same_cohort_delivery(retransmitted: bool) {
    // Given100 accepted originals,20 NAKs, and optionally real wire-marked retries.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    send(&mut conns[0], 0..100, false).await;
    clock.set(1100);
    for seq in 0..20 {
        assert!(conns[0].handle_nak(seq));
    }
    if retransmitted {
        clock.set(1200);
        send(&mut conns[0], 0..20, true).await;
    }
    // When their own link confirms delivery before the send cohort closes.
    clock.set(1300);
    for seq in 0..20 {
        ack(&mut conns, seq, (0, 0));
    }
    conns[0].loss.advance(2000);
    // Then unresolved loss is zero but the raw network/congestion signal remains20.
    assert_eq!(
        conns[0].loss.last_value(),
        Some(0.0),
        "retransmitted={retransmitted}"
    );
    assert_eq!(conns[0].congestion.nak_count, 20);
    assert_eq!(conns[0].delivery.delivered_bps(1300), 20.0 * 16.0 * 4.0);
}

#[tokio::test]
async fn same_cohort_late_original_resolves_naks_without_erasing_raw_congestion() {
    check_same_cohort_delivery(false).await;
}

#[tokio::test]
async fn same_cohort_retransmission_resolves_naks_without_erasing_raw_congestion() {
    check_same_cohort_delivery(true).await;
}

#[tokio::test]
async fn repeated_retry_naks_resolve_once_and_cannot_credit_unrelated_loss() {
    // Given two failed attempts for20sequences and one other unresolved sequence.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    send(&mut conns[0], 0..100, false).await;
    clock.set(1100);
    for seq in 0..20 {
        assert!(conns[0].handle_nak(seq));
    }
    send(&mut conns[0], 0..20, true).await;
    clock.set(1200);
    for seq in 0..20 {
        assert!(conns[0].handle_nak(seq));
    }
    assert!(conns[0].handle_nak(99));
    send(&mut conns[0], 0..20, true).await;
    // When successful retries are ACKed, including duplicate ACKs.
    clock.set(1300);
    for seq in 0..20 {
        ack(&mut conns, seq, (0, 0));
        ack(&mut conns, seq, (0, 0));
    }
    conns[0].loss.advance(2000);
    // Then only sequence99 remains lost:1debit/140accepted sends, rawNAKs41unchanged.
    assert_eq!(conns[0].loss.last_value(), Some(1.0 / 140.0));
    assert_eq!(conns[0].congestion.nak_count, 41);
}

#[tokio::test]
async fn recovered_loss_credit_is_arrival_generation_and_cohort_exact() {
    for (arrival, generation, at) in [(1, 0, 1300), (0, 1, 1300), (0, 0, 2000)] {
        // Given a pending loss onA and the same sequences sent byB.
        let clock = TestClock::new(1000);
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let mut conns = vec![
            create_test_connection_to(peer.local_addr().unwrap()).await,
            create_test_connection_to(peer.local_addr().unwrap()).await,
        ];
        send(&mut conns[0], 0..100, false).await;
        send(&mut conns[1], 0..100, false).await;
        clock.set(1100);
        for seq in 0..20 {
            assert!(conns[0].handle_nak(seq));
        }
        // When ACKs arrive elsewhere, from an old reader, or at the closure boundary.
        clock.set(at);
        for seq in 0..20 {
            ack(&mut conns, seq, (arrival, generation));
        }
        conns[0].loss.advance(2000);
        // ThenA's closed/unconfirmed loss cannot be erased by those ACKs.
        assert_eq!(
            conns[0].loss.last_value(),
            Some(0.2),
            "arrival={arrival},generation={generation},at={at}"
        );
    }
}

#[tokio::test]
async fn same_millisecond_recovery_epoch_rejects_old_loss_credit() {
    // Given a loss epoch replaced at the SAME timestamp, with new unrelated loss.
    let _clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    send(&mut conns[0], 0..100, false).await;
    for seq in 0..20 {
        assert!(conns[0].handle_nak(seq));
    }
    conns[0].loss.begin_recovered_epoch(1000);
    send(&mut conns[0], 100..200, false).await;
    for seq in 100..120 {
        assert!(conns[0].handle_nak(seq));
    }
    // When old sequences earn real ACKs, their cohort timestamp alone still matches.
    for seq in 0..20 {
        ack(&mut conns, seq, (0, 0));
    }
    conns[0].loss.advance(2000);
    // Then the epoch identity prevents old credits cancelling the new cohort's loss.
    assert_eq!(conns[0].loss.last_value(), Some(0.2));
}
