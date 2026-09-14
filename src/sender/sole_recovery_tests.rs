use tokio::net::UdpSocket;

use crate::config::DynamicConfig;
use crate::connection::health::{HealthMachine, HealthState};
use crate::sender::ack::{AckContext, AckPolicy, apply_srtla_ack};
use crate::sender::housekeeping::tick_health;
use crate::sender::selection::adaptive::{AdaptiveState, select};
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

#[test]
fn original_recovery_evidence_cannot_cross_health_epochs_or_expiry() {
    use crate::connection::adaptive::{OriginalRecovery, RecoveryWindow};
    // Given two fully proved original trains.
    let mut evidence = OriginalRecovery::default();
    for seq in 0..20 {
        evidence.record_sent(
            seq,
            1000,
            RecoveryWindow {
                epoch_ms: 1000,
                timeout_ms: 2060,
            },
        );
    }
    for seq in 0..20 {
        evidence.acknowledge(seq, 1100);
    }
    assert_eq!(evidence.rounds(1100).count, 2);
    // When a new Stalled epoch begins, then old ACKs cannot qualify its new trains.
    for seq in 20..40 {
        evidence.record_sent(
            seq,
            2000,
            RecoveryWindow {
                epoch_ms: 2000,
                timeout_ms: 2060,
            },
        );
    }
    for seq in 0..20 {
        evidence.acknowledge(seq, 2100);
    }
    assert_eq!(evidence.rounds(2100).count, 0);
    for seq in 20..40 {
        evidence.acknowledge(seq, 2100);
    }
    assert_eq!(evidence.rounds(2100).count, 2);
    assert_eq!(evidence.rounds(7000).count, 0);
}

#[tokio::test]
async fn stalled_sole_carrier_recovers_from_qualified_original_data_rounds() {
    for acknowledgements in [0, 4, 5, 10] {
        // Given a genuinely selected Stalled sole carrier, excluded from duplicate probes.
        let clock = TestClock::new(1000);
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let mut conn = create_test_connection_to(peer.local_addr().unwrap()).await;
        conn.health = HealthMachine::new(HealthState::Stalled, 1000);
        conn.rtt.update_estimate(60);
        let mut conns = vec![conn];
        let mut state = AdaptiveState::default();
        assert_eq!(
            select(
                &mut conns,
                None,
                0,
                1000,
                &DynamicConfig::new().snapshot(),
                &mut state
            ),
            Some(0)
        );
        assert!(state.sole_carrier.is_some());
        assert!(!state.targets[0].eligible());
        let mut packet = [0; 16];
        for seq in 0_u32..20 {
            packet[..4].copy_from_slice(&seq.to_be_bytes());
            conns[0].queue_data_packet(&packet, Some(seq), 1000);
        }
        conns[0].flush_batch().await.unwrap();
        // When two ten-packet ORIGINAL trains receive their link-specific ACKs.
        clock.set(1100);
        for seq in (0..acknowledgements).chain(10..10 + acknowledgements) {
            apply_srtla_ack(
                &mut conns,
                seq,
                AckContext {
                    arrival_idx: 0,
                    reader_generation: 0,
                    policy: AckPolicy::Adaptive,
                },
            );
        }
        tick_health(&mut conns, &state.targets, 1100);
        // Then the same five-of-ten requirement works without self-probing or fake delivery.
        let expected = if acknowledgements >= 5 {
            HealthState::Rejoining
        } else {
            HealthState::Stalled
        };
        assert_eq!(
            conns[0].health.state(),
            expected,
            "ACKs per train={acknowledgements}"
        );
        assert_eq!(conns[0].probes.probes_sent, 0);
        assert_eq!(
            conns[0].delivery.delivered_bps(1100),
            f64::from(acknowledgements * 2 * 16 * 4)
        );
    }
}
