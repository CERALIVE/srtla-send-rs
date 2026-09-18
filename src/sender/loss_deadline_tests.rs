use super::*;
use crate::config::{ConfigSnapshot, DynamicConfig};
use crate::connection::health::{HealthMachine, HealthState};
use crate::mode::SchedulingMode;
use crate::sender::housekeeping::tick_health;
use crate::sender::selection::adaptive::AdaptiveState;
use crate::stats::SharedStats;

#[tokio::test]
async fn negotiated_loss_deadline_qualifies_settlement_without_refreshing_cohort_age() {
    // Given real admission reading a negotiated two-second budget, not the fallback.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    let stats = SharedStats::new();
    stats.set_negotiated_latency_ms(2000);
    let mut adaptive = AdaptiveState::new(stats);
    adaptive.update_stats(
        &mut conns,
        &ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            ..DynamicConfig::new().snapshot()
        },
    );
    clock.set(1900);
    send(&mut conns[0], 0..100, false).await;
    clock.set(1950);
    for seq in 0..20 {
        assert!(conns[0].handle_nak(seq));
    }
    conns[0].loss.advance(3899);
    assert_eq!(conns[0].loss.last_value(), None);
    conns[0].connected = true;
    conns[0].health = HealthMachine::new(HealthState::Rejoining, 3800);
    conns[0].delivery.record_probe_proof(3800);
    // When the two-second deadline settles an already 1900ms-old closed cohort.
    clock.set(3900);
    tick_health(&mut conns, &[], 3900);
    // Then it still authorizes loss demotion, without moving the ten-second age anchor.
    assert_eq!(conns[0].health.state(), HealthState::Degraded);
    assert_eq!(conns[0].loss.last_value(), Some(0.2));
    assert_eq!(conns[0].loss.last_cohort_ms(), Some(2000));
    assert!(conns[0].loss.loss_cohort_ok(3900, 10_000));
    assert!(!conns[0].loss.is_stale(11999, 10_000));
    assert!(conns[0].loss.is_stale(12000, 10_000));
}

#[tokio::test]
async fn retry_debits_keep_distinct_deadlines_across_closed_cohorts() {
    // Given one sequence NAKed in two accepted-send cohorts with different deadlines.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    clock.set(1900);
    send(&mut conns[0], 0..100, false).await;
    assert!(conns[0].handle_nak(0));
    clock.set(2100);
    send(&mut conns[0], 0..1, true).await;
    send(&mut conns[0], 100..199, false).await;
    assert!(conns[0].handle_nak(0));
    // When ACK arrives after the first deadline but before the retry deadline.
    clock.set(2500);
    ack(&mut conns, 0, (0, 0));
    ack(&mut conns, 0, (0, 0));
    conns[0].loss.advance(3000);
    // Then only the retry is forgiven: 1/100 followed by 0/100 with alpha 0.2.
    assert_eq!(conns[0].loss.last_value(), Some(0.008));
    assert_eq!(conns[0].congestion.nak_count, 2);
}

#[tokio::test]
async fn nak_first_seen_after_deadline_is_final_despite_later_ack() {
    // Given a send whose deadline is 1500, and no prior NAK observation.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    send(&mut conns[0], 0..100, false).await;
    // When the first NAK and a valid ACK arrive after that deadline.
    clock.set(1600);
    assert!(conns[0].handle_nak(0));
    ack(&mut conns, 0, (0, 0));
    conns[0].loss.advance(2000);
    // Then loss is final even though delivery can still earn its ordinary byte credit.
    assert_eq!(conns[0].loss.last_value(), Some(0.01));
}
