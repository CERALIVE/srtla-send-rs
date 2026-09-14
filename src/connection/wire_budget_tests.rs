use std::time::Duration;

use tokio::net::UdpSocket;

use super::WireBudget;
use crate::connection::probe::ProbeTrain;
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn attempted_wire_limit_counts_originals_retries_and_probes() {
    // Given a5Mbit companion with original+retry load already above its capacity.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let conn = create_test_connection_to(peer.local_addr().unwrap()).await;
    let mut batch = crate::connection::BatchSender::new();
    batch.configure_wire_rate(Some(5_000_000.0), 1000);
    let mut accepted_bytes = 0;
    let mut probe_bytes = 0;
    let mut received_bytes = 0;
    // When every10ms brings six DATA packets (half retries) plus periodic probes.
    for tick in 0..100_u32 {
        let now = 1000 + u64::from(tick) * 10;
        clock.set(now);
        for offset in 0..6 {
            let seq = tick * 6 + offset;
            let mut packet = [0; 1332];
            packet[..4].copy_from_slice(&seq.to_be_bytes());
            packet[4] = if offset % 2 == 0 { 0 } else { 0x04 };
            batch.queue_packet(&packet, Some(seq), now);
        }
        if tick % 10 == 0 {
            batch.queue_probe(
                &[0; 1332],
                i32::try_from(1000 + tick).unwrap(),
                ProbeTrain {
                    id: u64::from(tick),
                    started_ms: now,
                    deadline_ms: now + 3000,
                },
            );
        }
        let outcome = batch.flush(&conn.socket).await;
        assert!(outcome.error.is_none());
        accepted_bytes += outcome.accepted.iter().map(|p| p.2).sum::<usize>();
        probe_bytes += outcome.probes.iter().map(|p| p.len).sum::<usize>();
        for _ in 0..outcome.accepted.len() + outcome.probes.len() {
            received_bytes +=
                tokio::time::timeout(Duration::from_secs(1), peer.recv(&mut [0; 1500]))
                    .await
                    .unwrap()
                    .unwrap();
        }
    }
    // Then all wire categories share one hard bound, independent of any ACKs.
    assert_eq!(received_bytes, accepted_bytes + probe_bytes);
    assert!(probe_bytes > 0);
    eprintln!(
        "attempted wire={received_bytes}B, probes={probe_bytes}B, bound={}B",
        625_000 + WireBudget::BURST_BYTES
    );
    assert!(
        received_bytes <= 625_000 + WireBudget::BURST_BYTES,
        "5Mbit companion exceeded its1s attempted-wire budget: {received_bytes}B"
    );
    assert!(
        batch.has_queued_packets(),
        "unfunded suffix must remain queued"
    );
}

#[test]
fn reconfiguration_preserves_spent_credit_and_idle_burst_is_bounded() {
    // Given an exhausted1Mbit budget, changing the estimate cannot mint fresh credit.
    let mut budget = WireBudget::new(1_000_000.0, 1000);
    budget.debit(WireBudget::BURST_BYTES, 1000);
    // When it becomes5Mbit one millisecond later, only the OLD rate funded that millisecond.
    budget.configure(5_000_000.0, 1001);
    assert_eq!(budget.available(1001), 125.0);
    budget.configure(5_000_000.0, 1001);
    // Then repeated configuration and long idle cannot evade the bounded bucket.
    assert_eq!(budget.available(1001), 125.0);
    assert_eq!(budget.available(1005), 2625.0);
    assert_eq!(budget.available(100_000), WireBudget::BURST_BYTES as f64);
}

#[tokio::test]
async fn millisecond_dequeue_uses_the_budget_without_an_underutilization_escape() {
    // Given5Mbit of hard credit and a continuously backlogged real UDP queue.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let conn = create_test_connection_to(peer.local_addr().unwrap()).await;
    let mut batch = crate::connection::BatchSender::new();
    batch.configure_wire_rate(Some(5_000_000.0), 1000);
    let mut bytes = 0;
    // When serviced at the production1ms retry cadence for one second.
    for tick in 0..1000_u32 {
        let now = 1000 + u64::from(tick);
        clock.set(now);
        batch.queue_packet(&[0; 1332], Some(tick), now);
        let result = batch.flush(&conn.socket).await;
        assert!(result.error.is_none());
        for _ in &result.accepted {
            bytes += tokio::time::timeout(Duration::from_secs(1), peer.recv(&mut [0; 1500]))
                .await
                .unwrap()
                .unwrap();
        }
    }
    // Then the gate uses at least96% of capacity, yet never exceeds rate plus burst.
    assert!((600_000..=628_000).contains(&bytes), "accepted {bytes}B");
    eprintln!("1ms pacing accepted={bytes}B against625000B/s plus3000B burst");
}

#[tokio::test]
async fn failed_kernel_send_does_not_spend_wire_credit() {
    // Given a funded batch and a kernel-send failure with no accepted prefix.
    let _clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let bad = crate::test_helpers::create_test_connection_with_failing_send().await;
    let good = create_test_connection_to(peer.local_addr().unwrap()).await;
    let mut batch = crate::connection::BatchSender::new();
    batch.configure_wire_rate(Some(1_000_000.0), 1000);
    batch.queue_packet(&[0; 1500], Some(0), 1000);
    batch.queue_packet(&[0; 1500], Some(1), 1000);
    let failed = batch.flush(&bad.socket).await;
    assert!(failed.error.is_some());
    assert!(failed.accepted.is_empty());
    // When the unchanged queue is submitted on a reachable socket at the SAME time.
    let sent = batch.flush(&good.socket).await;
    // Then both packets retain their credit and are committed once.
    assert!(sent.error.is_none());
    assert_eq!(sent.accepted.len(), 2);
    assert!(!batch.has_queued_packets());
    assert!(!batch.can_queue_wire(1, 1000));
}
