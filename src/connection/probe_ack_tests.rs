use tokio::net::UdpSocket;

use super::SrtlaConnection;
use super::delivery::DataSend;
use super::probe::ProbeTrain;
use crate::sender::packet_handler::{AckContext, AckPolicy, apply_srtla_ack};
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

async fn collision() -> Vec<SrtlaConnection> {
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut a = create_test_connection_to(peer.local_addr().unwrap()).await;
    let mut b = create_test_connection_to(peer.local_addr().unwrap()).await;
    a.delivery.record_sent(
        7,
        DataSend {
            sent_ms: 100,
            len: 1316,
        },
    );
    a.register_packet(7, 100);
    b.probes.record_sent(
        7,
        ProbeTrain {
            id: 1,
            started_ms: 100,
            deadline_ms: 4200,
        },
        100,
    );
    vec![a, b]
}

fn ack(conns: &mut [SrtlaConnection], idx: usize) {
    let context = AckContext {
        arrival_idx: idx,
        reader_generation: conns[idx].delivery.socket_generation,
        policy: AckPolicy::Adaptive,
    };
    apply_srtla_ack(conns, 7, context);
}

#[tokio::test]
async fn probe_attribution_is_arrival_scoped() {
    // Given A owns an original and B owns a probe with the same sequence.
    let _clock = TestClock::new(200);
    for first in [0, 1] {
        let mut conns = collision().await;
        // When either link receives the ACK first.
        ack(&mut conns, first);
        // Then only that arrival link earns proof; the other entry survives.
        assert_eq!(conns[first].delivery.last_data_proof_ms, 200);
        assert_eq!(conns[1 - first].delivery.last_data_proof_ms, 0);
        assert_eq!(
            conns[0].delivery.delivered_bps(200),
            if first == 0 { 5264.0 } else { 0.0 }
        );
        assert_eq!(conns[1].probes.probe_log.contains_key(&7), first == 0);
    }
}

#[tokio::test]
async fn probe_both_ack_orders() {
    // Given identical link evidence in two pools.
    let _clock = TestClock::new(200);
    let mut states = Vec::new();
    for order in [[0, 1], [1, 0]] {
        let mut conns = collision().await;
        // When ACK order is reversed.
        for idx in order {
            ack(&mut conns, idx);
        }
        // Then capture only proof and accounting outcomes.
        states.push(
            conns
                .iter()
                .map(|c| {
                    (
                        c.delivery.last_data_proof_ms,
                        c.delivery.delivered_bps(200),
                        c.in_flight_packets,
                        c.window,
                        c.probes.probe_log.len(),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(states[0], states[1]);
    assert_eq!(states[0][0].1, 5264.0);
    assert_eq!(states[0][1].1, 0.0);
}

#[tokio::test]
async fn probe_replay_after_consumption_or_expiry_has_no_effect() {
    // Given a consumed or expired B probe while A still owns its original.
    for expired in [false, true] {
        let clock = TestClock::new(200);
        let mut conns = collision().await;
        if expired {
            clock.set(4201);
        } else {
            ack(&mut conns, 1);
            clock.set(300);
        }
        let proof = conns[1].delivery.last_data_proof_ms;
        // When B receives a replay, then neither B's proof nor A's ledger is consumed.
        ack(&mut conns, 1);
        assert_eq!(conns[1].delivery.last_data_proof_ms, proof);
        assert_eq!(conns[0].delivery.last_data_proof_ms, 0);
        ack(&mut conns, 0);
        assert!(conns[0].delivery.delivered_bps(crate::utils::now_ms()) > 0.0);
    }
}

#[tokio::test]
async fn probe_old_reader_generation_ack_ignored_after_reconnect() {
    // Given sequence reuse after socket invalidation.
    let _clock = TestClock::new(200);
    let mut conns = collision().await;
    conns[1].mark_for_recovery();
    conns[1].probes.record_sent(
        7,
        ProbeTrain {
            id: 2,
            started_ms: 200,
            deadline_ms: 4300,
        },
        200,
    );
    // When a queued old-reader ACK arrives.
    apply_srtla_ack(
        &mut conns,
        7,
        AckContext {
            arrival_idx: 1,
            reader_generation: 0,
            policy: AckPolicy::Adaptive,
        },
    );
    // Then the new probe survives, with no proof on either link.
    assert!(conns[1].probes.probe_log.contains_key(&7));
    assert_eq!(conns[1].delivery.last_data_proof_ms, 0);
    assert_eq!(conns[0].delivery.last_data_proof_ms, 0);
}

#[tokio::test]
async fn probe_ack_never_grows_window() {
    // Given unrelated originals in flight on the probed link.
    let _clock = TestClock::new(200);
    let mut conns = collision().await;
    conns[1].register_packet(8, 100);
    let before: Vec<_> = conns
        .iter()
        .map(|c| (c.window, c.in_flight_packets, c.packet_log.clone()))
        .collect();
    // When its probe is acknowledged, then all congestion accounting stays identical.
    ack(&mut conns, 1);
    assert_eq!(
        before,
        conns
            .iter()
            .map(|c| (c.window, c.in_flight_packets, c.packet_log.clone()))
            .collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn probe_nak_on_probe_ignored() {
    // Given a probe-only sequence and a normal-loss cohort at its minimum load.
    let clock = TestClock::new(200);
    let mut conns = collision().await;
    for _ in 0..100 {
        conns[1].loss.record_send(200);
    }
    // When its NAK is offered to normal attribution.
    assert!(!conns[1].handle_nak(7));
    clock.set(1200);
    conns[1].loss.advance(1200);
    // Then no normal loss was charged and probe evidence remains outstanding.
    assert_eq!(conns[1].loss.last_value(), Some(0.0));
    assert!(conns[1].probes.probe_log.contains_key(&7));
}
