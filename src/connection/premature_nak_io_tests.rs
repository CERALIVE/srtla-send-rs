use smallvec::SmallVec;
use tokio::net::UdpSocket;

use super::SrtlaConnection;
use crate::connection::delivery::DataSend;
use crate::protocol::{SRT_TYPE_NAK, SRTLA_TYPE_KEEPALIVE};
use crate::registration::SrtlaRegistrationManager;
use crate::sender::SequenceTracker;
use crate::sender::ack::{AckContext, AckPolicy, apply_srtla_ack_frame};
use crate::sender::packet_handler::handle_uplink_packet;
use crate::sender::uplink::UplinkPacket;
use crate::stats::SharedStats;
use crate::test_helpers::create_test_connection;
use crate::utils::test_clock::TestClock;

async fn measured_link() -> SrtlaConnection {
    let mut conn = create_test_connection().await;
    conn.rtt.update_estimate(100);
    for seq in 0..4 {
        conn.register_packet(seq, 10_000);
        conn.delivery.record_sent(
            seq,
            DataSend {
                sent_ms: 10_000,
                len: 16,
            },
        );
    }
    conn
}

#[tokio::test]
async fn premature_nak_streak_resets_on_ack_frame_rtt() {
    // Given two suppressions on a link with eligible originals.
    let clock = TestClock::new(10_000);
    let mut conn = measured_link().await;
    assert!(conn.handle_nak(0));
    assert!(conn.handle_nak(1));
    clock.set(10_010);
    // When the production frame policy accepts an RTT sample.
    apply_srtla_ack_frame(
        std::slice::from_mut(&mut conn),
        &[2],
        AckContext {
            arrival_idx: 0,
            reader_generation: 0,
            policy: AckPolicy::Adaptive,
        },
    );
    // Then the next fresh NAK starts at one rather than continuing the old streak.
    assert_eq!(conn.premature_streak, 0);
    conn.register_packet(3, 10_010);
    conn.delivery.record_sent(
        3,
        DataSend {
            sent_ms: 10_010,
            len: 16,
        },
    );
    assert!(conn.handle_nak(3));
    assert_eq!(conn.premature_streak, 1);
    assert_eq!(conn.premature_nak_count, 3);
}

#[tokio::test]
async fn premature_nak_streak_resets_on_cumulative_ack_rtt() {
    // Given a suppressed NAK and an owned cumulative-ACK sample.
    let _clock = TestClock::new(10_000);
    let mut conn = measured_link().await;
    assert!(conn.handle_nak(0));
    // When the retained legacy API records a real RTT, Then clear the streak.
    conn.handle_srt_ack(1, 10_010, true);
    assert_eq!(conn.premature_streak, 0);
}

#[tokio::test]
async fn premature_nak_streak_resets_on_specific_ack_rtt() {
    // Given a suppressed NAK and a specific ACK for a different original.
    let clock = TestClock::new(10_000);
    let mut conn = measured_link().await;
    assert!(conn.handle_nak(0));
    clock.set(10_010);
    // When the specific-ACK API records RTT, Then clear the streak.
    assert!(conn.handle_srtla_ack_specific(1, false));
    assert_eq!(conn.premature_streak, 0);
}

#[tokio::test]
async fn premature_nak_rejected_ack_rtt_preserves_streak() {
    // Given a suppressed NAK and an ACK in the same millisecond as acceptance.
    let _clock = TestClock::new(10_000);
    let mut conn = measured_link().await;
    assert!(conn.handle_nak(0));
    // When zero RTT is rejected, Then ACK processing alone must not clear the cap.
    apply_srtla_ack_frame(
        std::slice::from_mut(&mut conn),
        &[1],
        AckContext {
            arrival_idx: 0,
            reader_generation: 0,
            policy: AckPolicy::Adaptive,
        },
    );
    assert_eq!(conn.premature_streak, 1);
}

#[tokio::test]
async fn premature_nak_streak_resets_only_on_valid_keepalive_rtt() {
    for elapsed in [0, 10, 10_001] {
        // Given suppression and a pending timestamped keepalive.
        let clock = TestClock::new(10_000);
        let mut conn = measured_link().await;
        assert!(conn.handle_nak(0));
        conn.rtt.record_keepalive_sent();
        let mut packet = [0; 10];
        packet[..2].copy_from_slice(&SRTLA_TYPE_KEEPALIVE.to_be_bytes());
        packet[2..].copy_from_slice(&10_000_u64.to_be_bytes());
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (forward, _rx) = tokio::sync::mpsc::unbounded_channel();
        clock.set(10_000 + elapsed);
        // When the actual receive path handles the echo.
        conn.process_packet(
            0,
            &mut SrtlaRegistrationManager::new(),
            &listener,
            &forward,
            None,
            &packet,
            &SharedStats::new(),
        )
        .await
        .unwrap();
        // Then only a plausible, nonzero RTT clears the streak.
        assert_eq!(conn.premature_streak, u8::from(elapsed != 10));
    }
}

#[tokio::test]
async fn premature_nak_forwarding_is_unchanged_and_fallback_stops() {
    for tracked in [false, true] {
        // Given two links holding the same seq; only the first owns this report.
        let _clock = TestClock::new(10_000);
        let mut conns = [measured_link().await, measured_link().await];
        let mut tracker = SequenceTracker::new();
        if tracked {
            tracker.insert(0, conns[0].conn_id, 10_000);
        }
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let encoder = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (forward, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut packet = [0x5a; 20];
        packet[..2].copy_from_slice(&SRT_TYPE_NAK.to_be_bytes());
        packet[16..].copy_from_slice(&0_u32.to_be_bytes());
        // When production uplink dispatch parses, applies and forwards a premature NAK.
        let uplink = UplinkPacket {
            conn_id: conns[0].conn_id,
            reader_generation: 0,
            bytes: SmallVec::from_slice_copy(&packet),
        };
        handle_uplink_packet(
            uplink,
            &mut conns,
            &mut SrtlaRegistrationManager::new(),
            &forward,
            Some(encoder.local_addr().unwrap()),
            &listener,
            &tracker,
            &crate::config::DynamicConfig::new().snapshot(),
            &SharedStats::new(),
        )
        .await;
        let mut received = [0; 64];
        let len = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            encoder.recv(&mut received),
        )
        .await
        .unwrap()
        .unwrap();
        // Then encoder bytes are identical and no second link is tried or penalised.
        assert_eq!(&received[..len], &packet);
        assert_eq!(conns[0].premature_nak_count, 1);
        assert_eq!(conns[1].premature_nak_count, 0);
        assert!(
            conns
                .iter()
                .all(|c| c.in_flight_packets == 4 && c.congestion.nak_count == 0)
        );
    }
}
