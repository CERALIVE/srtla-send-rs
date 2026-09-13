use tokio::net::UdpSocket;
use tokio::time::{Duration, timeout};

use super::*;
use crate::sender::SequenceTracker;
use crate::sender::packet_handler::handle_srt_packet;
use crate::test_helpers::create_test_connection_to;

struct Forwarding {
    conns: SmallVec<SrtlaConnection, 4>,
    peers: [UdpSocket; 2],
    state: AdaptiveState,
    edpf: EdpfSchedulerState,
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
            edpf: EdpfSchedulerState::default(),
            sequences: SequenceTracker::new(),
            last: None,
            switched: 0,
        }
    }

    async fn send(&mut self, seq: u32) {
        let mut packet = [0; 1316];
        packet[..4].copy_from_slice(&seq.to_be_bytes());
        let cfg = ConfigSnapshot {
            mode: "adaptive".parse().unwrap(),
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
            &mut self.edpf,
            &mut self.state,
        )
        .await;
    }
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
