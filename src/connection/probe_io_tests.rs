use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::timeout;

use super::health::HealthState;
use super::probe::{ProbeOpportunity, ProbeScheduler, ProbeTarget};
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

fn data(seq: u32) -> [u8; 20] {
    let mut packet = [0x55; 20];
    packet[..4].copy_from_slice(&seq.to_be_bytes());
    packet[4] = 0xfc;
    packet
}

#[tokio::test]
async fn probe_bytes_sent_total_includes_probe_bytes_without_normal_ownership() {
    // Given one original sent normally and a held alternate, over real loopback UDP.
    let clock = TestClock::new(100);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![
        create_test_connection_to(receiver.local_addr().unwrap()).await,
        create_test_connection_to(receiver.local_addr().unwrap()).await,
    ];
    let packet = data(7);
    conns[0].queue_data_packet(&packet, Some(7), 100);
    conns[0].flush_batch().await.unwrap();
    let mut buffer = [0; 64];
    timeout(Duration::from_secs(1), receiver.recv_from(&mut buffer))
        .await
        .unwrap()
        .unwrap();
    let target = ProbeTarget {
        conn_id: conns[1].conn_id,
        socket_generation: 0,
        health: HealthState::Stalled,
        deadline_held: true,
        sole_carrier: false,
        srtt_ms: 100,
    };
    let offer = ProbeOpportunity {
        primary_conn_id: conns[0].conn_id,
        packet: &packet,
        targets: &[target],
    };
    // When the owned scheduler emits the copy through the unpadded batch path.
    let sent = ProbeScheduler::default()
        .emit(&mut conns, offer)
        .await
        .unwrap();
    sent.outcome.unwrap();
    let (len, addr) = timeout(Duration::from_secs(1), receiver.recv_from(&mut buffer))
        .await
        .unwrap()
        .unwrap();
    // Then the actual wire bytes differ only in byte4/mask04; accounting is probe-only.
    let mut expected = packet;
    expected[4] &= !0x04;
    assert_eq!(&buffer[..len], &expected);
    assert_eq!(addr, conns[1].socket.local_addr().unwrap());
    assert_eq!(conns[1].bitrate.bytes_sent_total, 20);
    assert_eq!(conns[1].probes.probes_sent, 1);
    assert!(conns[1].probes.probe_log.contains_key(&7));
    assert_eq!(conns[1].in_flight_packets, 0);
    assert!(conns[1].packet_log.is_empty());
    assert_eq!(conns[1].delivery.attempts_since_proof, 0);
    clock.set(2100);
    conns[1].bitrate.calculate();
    assert_eq!(conns[1].bitrate.current_bitrate_bps, 80.0);
}

#[tokio::test]
async fn probe_zero_rate_emits_no_datagrams_over_five_seconds() {
    // Given a zero-rate scheduler and a real alternate socket.
    let clock = TestClock::new(0);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    let target = ProbeTarget {
        conn_id: conn.conn_id,
        socket_generation: 0,
        health: HealthState::Degraded,
        deadline_held: true,
        sole_carrier: false,
        srtt_ms: 100,
    };
    let mut conns = vec![conn];
    let mut scheduler = ProbeScheduler::with_rate(0);
    // When DATA is offered every millisecond for five seconds.
    for now in 0..=5000 {
        clock.set(now);
        assert!(
            scheduler
                .emit(
                    &mut conns,
                    ProbeOpportunity {
                        primary_conn_id: 0,
                        packet: &data(7),
                        targets: &[target]
                    }
                )
                .await
                .is_none()
        );
    }
    // Then neither wire nor byte/probe accounting contains a send.
    assert_eq!(
        receiver.try_recv_from(&mut [0; 64]).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
    assert_eq!(conns[0].probes.probes_sent, 0);
    assert_eq!(conns[0].bitrate.bytes_sent_total, 0);
}

#[tokio::test]
async fn probe_configured_limit_bounds_real_emission() {
    // Given the production constant, an owned scheduler and a real held socket.
    let clock = TestClock::new(0);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    let target = ProbeTarget {
        conn_id: conn.conn_id,
        socket_generation: 0,
        health: HealthState::Stalled,
        deadline_held: true,
        sole_carrier: false,
        srtt_ms: 100,
    };
    let mut conns = vec![conn];
    let mut scheduler = ProbeScheduler::default();
    let mut received = 0;
    // When DATA is offered for five virtual seconds through the real emission API.
    for now in 0..5000u32 {
        clock.set(u64::from(now));
        if let Some(emission) = scheduler
            .emit(
                &mut conns,
                ProbeOpportunity {
                    primary_conn_id: 0,
                    packet: &data(now),
                    targets: &[target],
                },
            )
            .await
        {
            emission.outcome.unwrap();
            timeout(Duration::from_secs(1), receiver.recv_from(&mut [0; 64]))
                .await
                .unwrap()
                .unwrap();
            received += 1;
        }
    }
    // Then the bound holds on actual datagrams, including a scratch constant of zero.
    assert_eq!(received, super::probe::PROBE_MAX_PPS * 5);
    assert_eq!(conns[0].probes.probes_sent, u64::from(received));
    assert_eq!(
        receiver.try_recv_from(&mut [0; 64]).unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[tokio::test]
async fn probe_partial_batch_commits_only_accepted_copies() {
    // Given a mixed batch: valid probe, oversized original, unsent probe suffix.
    let _clock = TestClock::new(100);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    let target = ProbeTarget {
        conn_id: conn.conn_id,
        socket_generation: 0,
        health: HealthState::Stalled,
        deadline_held: true,
        sole_carrier: false,
        srtt_ms: 100,
    };
    let dispatch = ProbeScheduler::default().next(&[target], 100).unwrap();
    assert!(conn.queue_probe_packet(&data(1), dispatch));
    conn.queue_data_packet(&vec![0; 70_000], Some(99), 100);
    assert!(conn.queue_probe_packet(&data(2), dispatch));
    assert_eq!(conn.batch_sender.queued_count(), 1);
    let bytes_before = conn.bitrate.bytes_sent_total;
    // When the real kernel accepts the prefix and rejects the oversized middle.
    assert!(conn.flush_batch().await.is_err());
    // Then only the first probe is accounted; neither probe becomes a normal send.
    assert_eq!(conn.probes.probes_sent, 1);
    assert_eq!(conn.bitrate.bytes_sent_total - bytes_before, 20);
    assert!(conn.probes.probe_log.contains_key(&1));
    assert!(!conn.probes.probe_log.contains_key(&2));
    assert_eq!(conn.delivery.attempts_since_proof, 0);
    assert_eq!(conn.in_flight_packets, 0);
    assert!(conn.has_queued_packets());
}
