use tokio::net::UdpSocket;
use tokio::time::{Duration, timeout};

use super::*;
use crate::sender::SequenceTracker;
use crate::sender::packet_handler::{SrtPacketOutcome, handle_srt_packet};
use crate::test_helpers::create_test_connection_to;

struct Forwarding {
    conns: SmallVec<SrtlaConnection, 4>,
    peers: [UdpSocket; 2],
    state: AdaptiveState,
    sequences: SequenceTracker,
    last: Option<usize>,
    switched: u64,
}

impl Forwarding {
    async fn new() -> Self {
        let peers = [
            UdpSocket::bind("127.0.0.1:0").await.unwrap(),
            UdpSocket::bind("127.0.0.1:0").await.unwrap(),
        ];
        let mut conns = SmallVec::new();
        for peer in &peers {
            let mut conn = create_test_connection_to(peer.local_addr().unwrap()).await;
            conn.health = HealthMachine::new(HealthState::Healthy, 0);
            conns.push(conn);
        }
        conns[1].health = HealthMachine::new(HealthState::Stalled, 0);
        Self {
            conns,
            peers,
            state: AdaptiveState::default(),
            sequences: SequenceTracker::new(),
            last: None,
            switched: 0,
        }
    }

    async fn send(&mut self, seq: u32) {
        self.send_data(seq, false).await;
    }

    async fn send_data(&mut self, seq: u32, retransmitted: bool) -> SrtPacketOutcome {
        let mut packet = [0; 1316];
        packet[..4].copy_from_slice(&seq.to_be_bytes());
        packet[4] = if retransmitted { 0x04 } else { 0 };
        let cfg = ConfigSnapshot {
            mode: "enhanced".parse().unwrap(),
            ..config()
        };
        handle_srt_packet(
            Ok((packet.len(), "127.0.0.1:12345".parse().unwrap())),
            &mut packet,
            &mut self.conns,
            &mut self.last,
            &mut self.switched,
            &mut self.sequences,
            &mut None,
            true,
            &cfg,
            &mut self.state,
        )
        .await
    }
}

#[tokio::test]
async fn adaptive_rate_estimate_does_not_override_ranked_link_cooldown() {
    // Given two healthy links with old hard budgets that adaptive must disable.
    let _clock = TestClock::new(10_000);
    let mut f = Forwarding::new().await;
    for conn in &mut f.conns {
        conn.health = HealthMachine::new(HealthState::Healthy, 0);
        conn.batch_sender
            .configure_wire_rate(Some(1_000_000.0), 10_000);
        assert!(conn.batch_sender.wire_limited());
        assert!(conn.batch_sender.can_queue_wire(1316, 10_000));
    }
    f.send(0).await;
    f.send(1).await;
    // When a third datagram exceeds the current link's two-MTU credit.
    assert_eq!(f.send_data(2, true).await, SrtPacketOutcome::Consumed);
    // Then rate-only funding cannot override the ranked link's cooldown.
    assert_eq!(f.last, Some(0));
    assert_eq!(f.conns.len(), 2);
}

#[tokio::test]
async fn adaptive_originals_and_retries_continue_beyond_old_wire_bursts() {
    // Given two healthy links and five packets exceeding their old combined bursts.
    let _clock = TestClock::new(10_000);
    let mut f = Forwarding::new().await;
    for conn in &mut f.conns {
        conn.health = HealthMachine::new(HealthState::Healthy, 0);
        conn.batch_sender
            .configure_wire_rate(Some(1_000_000.0), 10_000);
    }
    // When originals and retries enter at the same clock tick, none is backpressured.
    for seq in 0..5 {
        assert_eq!(
            f.send_data(seq, seq % 2 == 1).await,
            SrtPacketOutcome::Consumed
        );
    }
    for conn in &mut f.conns {
        conn.flush_batch().await.unwrap();
    }
    // Then every datagram is counted once and accepted without accruing rate credit.
    assert_eq!(
        f.conns
            .iter()
            .map(|c| c.bitrate.bytes_sent_total)
            .sum::<u64>(),
        5 * 1316
    );
    for seq in 0..5 {
        assert!(f.sequences.get(seq, 10_000).is_some());
    }
    assert_eq!(f.conns.len(), 2);
}

#[tokio::test]
async fn adaptive_forwarding_emits_probes_after_primary_acceptance() {
    // Given a real registered primary and a Stalled alternate on ephemeral UDP.
    let _clock = TestClock::new(10_000);
    let mut f = Forwarding::new().await;
    // When the production packet handler forwards one DATA packet.
    f.send(42).await;
    // Then the original and probe are accepted, with only the original owning load.
    assert_eq!(f.conns[1].probes.probes_sent, 1);
    assert_eq!(f.conns[0].delivery.attempts_since_proof, 1);
    assert_eq!(f.conns[1].delivery.attempts_since_proof, 0);
    assert_eq!(f.conns[1].in_flight_packets, 0);
    assert_eq!(f.sequences.get(42, 10_000), Some(f.conns[0].conn_id));
    assert_eq!((f.last, f.switched), (Some(0), 10_000));
    let mut original = [0; 1316];
    let mut copy = [0; 1316];
    let n = timeout(Duration::from_secs(1), f.peers[0].recv(&mut original))
        .await
        .unwrap()
        .unwrap();
    let m = timeout(Duration::from_secs(1), f.peers[1].recv(&mut copy))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&original[..n], &copy[..m]);
}

#[tokio::test]
async fn adaptive_failed_primary_suppresses_probes_and_removes_ownership() {
    // Given a primary whose first flush fails.
    let _clock = TestClock::new(10_000);
    let mut f = Forwarding::new().await;
    f.conns[0].socket.fail_sends();
    // When a due probe needs its original flushed, then failure recovers the primary.
    f.send(42).await;
    assert!(!f.conns[0].connected);
    assert_eq!(f.sequences.get(42, 10_000), None);
    assert_eq!(f.conns[1].probes.probes_sent, 0);
}

#[tokio::test]
async fn adaptive_probe_failure_recovers_only_the_alternate() {
    // Given a failing probe target, with old normal ownership still attached.
    let _clock = TestClock::new(10_000);
    let mut f = Forwarding::new().await;
    f.conns[1].socket.fail_sends();
    f.sequences.insert(41, f.conns[1].conn_id, 10_000);
    // When the primary succeeds and probe fails, then only the alternate is recovered.
    f.send(42).await;
    assert!(f.conns[0].connected);
    assert!(!f.conns[1].connected);
    assert_eq!(f.sequences.get(41, 10_000), None);
    assert_eq!(f.sequences.get(42, 10_000), Some(f.conns[0].conn_id));
}

#[tokio::test]
async fn adaptive_probes_do_not_flush_every_packet_or_probe_the_sole_carrier() {
    // Given two Stalled links and a fast sole carrier.
    let _clock = TestClock::new(10_000);
    let mut f = Forwarding::new().await;
    f.conns[0].health = HealthMachine::new(HealthState::Stalled, 0);
    rtt(&mut f.conns[0], 20.0);
    rtt(&mut f.conns[1], 100.0);
    f.send(41).await;
    // When another packet arrives at the same clock tick, then pacing keeps it batched.
    f.send(42).await;
    assert!(f.conns[0].has_queued_packets());
    assert_eq!(f.conns[0].probes.probes_sent, 0);
    assert_eq!(f.conns[1].probes.probes_sent, 1);
    assert!(f.state.targets[0].sole_carrier);
}

#[tokio::test]
async fn retransmitted_input_obeys_the_same_health_admission_as_original_data() {
    // Given a held link with a larger raw window and a just-established cooldown.
    for retransmitted in [false, true] {
        let _clock = TestClock::new(10_000);
        let mut f = Forwarding::new().await;
        f.conns[1].window = 1_000_000;
        f.last = Some(1);
        f.switched = 10_000;
        // When original or R-marked DATA enters the real production packet handler.
        f.send_data(42, retransmitted).await;
        // Then both select the admitted carrier, never the held link or its cooldown.
        assert_eq!(f.last, Some(0));
        assert_eq!(f.sequences.get(42, 10_000), Some(f.conns[0].conn_id));
        assert_eq!(f.conns[0].in_flight_packets, 1);
        assert_eq!(f.conns[0].bitrate.bytes_sent_total, 1316);
        assert_eq!(f.conns[1].in_flight_packets, 0);
        assert_eq!(f.conns[1].probes.probes_sent, 1);
        let mut received = [0; 1316];
        assert_eq!(
            timeout(Duration::from_secs(1), f.peers[0].recv(&mut received))
                .await
                .unwrap()
                .unwrap(),
            1316
        );
        assert_eq!(received[4] & 0x04, if retransmitted { 0x04 } else { 0 });
        eprintln!(
            "R={retransmitted}: production handler selected admitted0, held1 original-flight0, \
             wire flag preserved"
        );
    }
}
