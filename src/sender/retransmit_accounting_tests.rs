use std::time::Duration;

use tokio::net::UdpSocket;

use super::{AckContext, AckPolicy, apply_srtla_ack};
use crate::connection::SrtlaConnection;
use crate::connection::rate_cap::RateSignals;
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

fn packet(seq: u32, retransmitted: bool) -> [u8; 1332] {
    let mut bytes = [0_u8; 1332];
    bytes[..4].copy_from_slice(&seq.to_be_bytes());
    bytes[4] = if retransmitted { 0x04 } else { 0 };
    bytes
}

async fn receive(peer: &UdpSocket) -> [u8; 1332] {
    let mut bytes = [0_u8; 1332];
    let len = tokio::time::timeout(Duration::from_secs(1), peer.recv(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(len, bytes.len());
    bytes
}

fn ack(conn: &mut SrtlaConnection, seq: i32) {
    let reader_generation = conn.delivery.socket_generation;
    apply_srtla_ack(
        std::slice::from_mut(conn),
        seq,
        AckContext {
            arrival_idx: 0,
            reader_generation,
            policy: AckPolicy::Adaptive,
        },
    );
}

#[tokio::test]
async fn retransmit_only_load_counts_in_queue_flight_wire_and_delivered_rate() {
    // Given identical links carrying either normal DATA or rerouted R-marked DATA.
    for retransmitted in [false, true] {
        let clock = TestClock::new(1000);
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let mut conn = create_test_connection_to(peer.local_addr().unwrap()).await;
        conn.window = 20_000;
        let empty_score = conn.get_score();
        // When256 distinct packets pass through the real queued/accepted-prefix path.
        for batch in 0..16_u32 {
            for offset in 0..16 {
                let seq = batch * 16 + offset;
                conn.queue_data_packet(&packet(seq, retransmitted), Some(seq), 1000);
            }
            assert_eq!(
                conn.get_score(),
                20_000 / (i32::try_from((batch + 1) * 16).unwrap() + 1)
            );
            conn.flush_batch().await.unwrap();
            for offset in 0..16 {
                assert_eq!(
                    receive(&peer).await,
                    packet(batch * 16 + offset, retransmitted)
                );
            }
        }
        // Then both forms consume wire bytes and lower the same ranking/BDP inputs.
        assert_eq!(conn.bitrate.bytes_sent_total, 340_992);
        assert_eq!(conn.in_flight_packets, 256);
        assert_eq!(conn.delivery.attempts_since_proof, 256);
        assert_eq!(conn.get_score(), 77);
        assert!(conn.get_score() < empty_score);
        assert_eq!(conn.rate_cap.soft_cap_multiplier(256), 0.125);
        clock.set(1100);
        for seq in 0..256 {
            ack(&mut conn, seq);
        }
        conn.rate_cap.tick(
            &conn.delivery,
            &RateSignals {
                now_ms: 2000,
                srtt_ms: 60.0,
                rtt_min_ms: 60.0,
                loss_ewma: Some(0.0),
                queue_delay_ms: 0.0,
                velocity_ms_per_update: 0.0,
                jitter_ms: 0.0,
            },
        );
        assert_eq!(conn.in_flight_packets, 0);
        assert_eq!(conn.rate_cap.delivered_bps(), 1_363_968.0);
        assert_eq!(conn.rate_cap.target_bps(), 1_363_968.0);
        clock.set(3000);
        conn.bitrate.calculate();
        assert_eq!(conn.bitrate.current_bitrate_bps, 1_363_968.0);
        eprintln!(
            "R={retransmitted}: bytes=340992 flight=256 score=77 cap=.125 \
             ACK-rate/target=1363968bps"
        );
    }
}

#[tokio::test]
async fn repeated_sequence_counts_each_wire_send_but_not_multiple_ambiguous_deliveries() {
    // Given an original and retry of one sequence before either ACK arrives.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(peer.local_addr().unwrap()).await;
    for retransmitted in [false, true] {
        conn.queue_data_packet(&packet(7, retransmitted), Some(7), 1000);
        conn.flush_batch().await.unwrap();
        assert_eq!(receive(&peer).await, packet(7, retransmitted));
    }
    // When one sequence ACK arrives, its wire format cannot identify which copy won.
    assert_eq!(conn.bitrate.bytes_sent_total, 2664);
    assert_eq!(conn.delivery.attempts_since_proof, 2);
    assert_eq!(conn.in_flight_packets, 1);
    clock.set(1100);
    ack(&mut conn, 7);
    ack(&mut conn, 7);
    // Then wire expenditure counts twice, delivery proof once, with no replay credit.
    assert_eq!(conn.delivery.delivered_bps(1100), 5328.0);
    assert_eq!(conn.in_flight_packets, 0);
    eprintln!("same-seq: bytes=2664 attempts=2 unique-flight=1; repeated ACK credits1332B once");
}

#[tokio::test]
async fn nak_removes_flight_and_a_later_retry_reenters_the_same_accounting() {
    // Given one accepted original, genuinely observed on the peer socket.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(peer.local_addr().unwrap()).await;
    conn.queue_data_packet(&packet(9, false), Some(9), 1000);
    conn.flush_batch().await.unwrap();
    assert_eq!(receive(&peer).await, packet(9, false));
    // When NAK accounting runs, it does not itself emit a retransmission.
    clock.set(1100);
    assert!(conn.handle_nak(9));
    assert_eq!(conn.in_flight_packets, 0);
    let error = peer.try_recv(&mut [0; 1332]).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);
    conn.queue_data_packet(&packet(9, true), Some(9), 1100);
    conn.flush_batch().await.unwrap();
    // Then a subsequently supplied R-marked packet is counted as a fresh attempt.
    assert_eq!(receive(&peer).await, packet(9, true));
    assert_eq!(conn.bitrate.bytes_sent_total, 2664);
    assert_eq!(conn.in_flight_packets, 1);
    assert_eq!(conn.delivery.attempts_since_proof, 2);
    clock.set(1200);
    ack(&mut conn, 9);
    assert_eq!(conn.delivery.delivered_bps(1200), 5328.0);
    eprintln!("NAK: no wire retransmit; later R input bytes=2664 flight=1 delivered=5328bps");
}
