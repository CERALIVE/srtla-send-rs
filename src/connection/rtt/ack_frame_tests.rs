use smallvec::SmallVec;
use tokio::net::UdpSocket;

use crate::config::{ConfigSnapshot, DynamicConfig};
use crate::connection::SrtlaConnection;
use crate::connection::probe::ProbeTrain;
use crate::mode::SchedulingMode;
use crate::protocol::SRTLA_TYPE_ACK;
use crate::registration::SrtlaRegistrationManager;
use crate::sender::SequenceTracker;
use crate::sender::packet_handler::handle_uplink_packet;
use crate::sender::uplink::UplinkPacket;
use crate::stats::SharedStats;
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

async fn staggered_originals(clock: &TestClock) -> SrtlaConnection {
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    for seq in 0..10_u32 {
        let sent = 1000 + u64::from(seq * seq * 10);
        clock.set(sent);
        let mut packet = [0_u8; 20];
        packet[..4].copy_from_slice(&seq.to_be_bytes());
        conn.queue_data_packet(&packet, Some(seq), sent);
        conn.flush_batch().await.unwrap();
    }
    clock.set(1870); // Last acceptance at 1810 + actual path RTT 60ms.
    conn
}

async fn dispatch(conns: &mut [SrtlaConnection], sequences: &[u32], mode: SchedulingMode) {
    let mut bytes = SmallVec::from_slice_copy(&SRTLA_TYPE_ACK.to_be_bytes());
    bytes.extend_from_slice(&[0, 0]);
    for seq in sequences {
        bytes.extend_from_slice(&seq.to_be_bytes());
    }
    let packet = UplinkPacket {
        conn_id: conns[0].conn_id,
        reader_generation: conns[0].delivery.socket_generation,
        bytes,
    };
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let (forward, _received) = tokio::sync::mpsc::unbounded_channel();
    handle_uplink_packet(
        packet,
        conns,
        &mut SrtlaRegistrationManager::new(),
        &forward,
        None,
        &listener,
        &SequenceTracker::new(),
        &ConfigSnapshot {
            mode,
            ..DynamicConfig::new().snapshot()
        },
        &SharedStats::new(),
    )
    .await;
}

#[tokio::test]
async fn coalesced_ack_samples_only_the_final_original() {
    // Given ten unevenly spaced kernel-accepted originals on a fresh RTT tracker.
    let clock = TestClock::new(1000);
    let mut conns = [staggered_originals(&clock).await];
    // When one receiver-order frame acknowledges all ten.
    dispatch(
        &mut conns,
        &(0..10).collect::<Vec<_>>(),
        SchedulingMode::Adaptive,
    )
    .await;
    // Then all delivery credit survives, but precisely one 60ms sample reaches RTT.
    let rtt = &conns[0].rtt;
    eprintln!(
        "ACK samples={:?}, Kalman={} jitter={}",
        rtt.rtt_sample_filter,
        rtt.kalman_rtt.value(),
        rtt.rtt_jitter_ms
    );
    assert_eq!(
        rtt.rtt_sample_filter.iter().copied().collect::<Vec<_>>(),
        [60.0]
    );
    assert_eq!(rtt.kalman_rtt.value(), 60.0);
    assert_eq!(rtt.rtt_jitter_ms, 0.0);
    assert_eq!(conns[0].delivery.delivered_bps(1870), 800.0);
    assert_eq!(conns[0].in_flight_packets, 0);
}

#[tokio::test]
async fn coalesced_ack_ending_in_probe_does_not_sample_an_earlier_original() {
    // Given nine originals followed by a duplicate-DATA probe in receiver order.
    let clock = TestClock::new(1000);
    let mut conns = [staggered_originals(&clock).await];
    conns[0].probes.record_sent(
        10,
        ProbeTrain {
            id: 1,
            started_ms: 1810,
            deadline_ms: 4000,
        },
        1810,
    );
    // When the last entry is a probe (never part of the original ledger).
    dispatch(
        &mut conns,
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 10],
        SchedulingMode::Adaptive,
    )
    .await;
    // Then original delivery and probe proof are credited without any path RTT sample.
    assert!(conns[0].rtt.rtt_sample_filter.is_empty());
    assert_eq!(conns[0].delivery.delivered_bps(1870), 720.0);
    assert!(!conns[0].probes.probe_log.contains_key(&10));
}

#[tokio::test]
async fn coalesced_ack_ending_in_retransmission_does_not_sample_earlier_entries() {
    // Given a final original whose retransmission makes its ACK Karn-ambiguous.
    let clock = TestClock::new(1000);
    let mut conns = [staggered_originals(&clock).await];
    conns[0].delivery.mark_retransmitted(9);
    // When the coalesced frame ends in that ambiguous entry.
    dispatch(
        &mut conns,
        &(0..10).collect::<Vec<_>>(),
        SchedulingMode::Adaptive,
    )
    .await;
    // Then all delivery is credited, but no earlier coalesced timing is substituted.
    assert!(conns[0].rtt.rtt_sample_filter.is_empty());
    assert_eq!(conns[0].delivery.delivered_bps(1870), 800.0);
}

#[tokio::test]
async fn coalesced_ack_uses_receiver_order_not_the_largest_sequence() {
    // Given originals reordered on the wire, with the earliest send arriving last.
    let clock = TestClock::new(1000);
    let mut conns = [staggered_originals(&clock).await];
    // When the receiver reports reverse sequence order.
    dispatch(
        &mut conns,
        &[9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
        SchedulingMode::Adaptive,
    )
    .await;
    // Then the final entry's real delay is preserved, not optimized away by sorting.
    assert_eq!(
        conns[0]
            .rtt
            .rtt_sample_filter
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [870.0]
    );
}

#[tokio::test]
async fn coalesced_ack_frames_sample_independently_but_replays_do_not() {
    // Given ten originals acknowledged in two distinct receiver frames.
    let clock = TestClock::new(1000);
    let mut conns = [staggered_originals(&clock).await];
    // When both frames and a replay of the second are dispatched.
    for sequences in [&[0, 1, 2, 3, 4][..], &[5, 6, 7, 8, 9], &[5, 6, 7, 8, 9]] {
        dispatch(&mut conns, sequences, SchedulingMode::Adaptive).await;
    }
    // Then each new frame contributes one sample and the replay contributes none.
    assert_eq!(
        conns[0]
            .rtt
            .rtt_sample_filter
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [710.0, 60.0]
    );
    assert_eq!(conns[0].delivery.delivered_bps(1870), 800.0);
}

#[tokio::test]
async fn coalesced_ack_probe_only_frame_never_samples_original_rtt() {
    // Given a complete duplicate-DATA train separate from the original ledger.
    let clock = TestClock::new(1000);
    let mut conns = [staggered_originals(&clock).await];
    let train = ProbeTrain {
        id: 1,
        started_ms: 1810,
        deadline_ms: 4000,
    };
    for seq in 10..20 {
        conns[0].probes.record_sent(seq, train, 1810);
    }
    // When a frame acknowledges only that train.
    dispatch(
        &mut conns,
        &(10..20).collect::<Vec<_>>(),
        SchedulingMode::Adaptive,
    )
    .await;
    // Then probe health proof is retained without original credit or RTT samples.
    assert!(conns[0].rtt.rtt_sample_filter.is_empty());
    assert_eq!(conns[0].delivery.proof_age_ms(1870), Some(0));
    assert_eq!(conns[0].delivery.delivered_bps(1870), 0.0);
    assert!(conns[0].probes.probe_log.is_empty());
}

#[tokio::test]
async fn coalesced_ack_uses_final_entry_rtt_in_enhanced_too() {
    // Given the same staggered traffic under the established enhanced mode.
    let clock = TestClock::new(1000);
    let mut conns = [staggered_originals(&clock).await];
    // When its coalesced ACK crosses the same parser and dispatcher.
    dispatch(
        &mut conns,
        &(0..10).collect::<Vec<_>>(),
        SchedulingMode::Enhanced,
    )
    .await;
    // Then shared ACK policy samples only the final receiver-order original.
    assert_eq!(conns[0].rtt.rtt_sample_filter.len(), 1);
    assert_eq!(conns[0].in_flight_packets, 0);
}
