use super::*;
use crate::connection::health::{HealthMachine, HealthState};
use crate::sender::housekeeping::tick_health;

async fn pending(count: u32) -> (TestClock, Vec<SrtlaConnection>, UdpSocket) {
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    clock.set(1900);
    send(&mut conns[0], 0..count, false).await;
    clock.set(1950);
    for seq in 0..20 {
        assert!(conns[0].handle_nak(seq));
    }
    conns[0].loss.advance(2000);
    (clock, conns, peer)
}

#[tokio::test]
async fn closed_unsettled_credit_succeeds_before_send_deadline() {
    // Given a closed denominator with retransmissions still inside the 500ms budget.
    let (clock, mut conns, _peer) = pending(100).await;
    assert_eq!(conns[0].loss.last_value(), None);
    clock.set(2100);
    send(&mut conns[0], 0..20, true).await;
    // When arrival-specific ACKs precede the original accepted-send deadline.
    clock.set(2200);
    for seq in 0..20 {
        ack(&mut conns, seq, (0, 0));
        ack(&mut conns, seq, (0, 0));
    }
    conns[0].loss.advance(2400);
    // Then closure has not made those debits irreversible, nor moved their denominator.
    assert_eq!(conns[0].loss.last_value(), Some(0.0));
    assert_eq!(conns[0].loss.last_cohort_ms(), Some(2000));
    assert_eq!(conns[0].congestion.nak_count, 20);
}

#[tokio::test]
async fn unresolved_at_deadline_becomes_final_loss() {
    // Given a closed cohort that has not reached its send deadline.
    let (clock, mut conns, _peer) = pending(100).await;
    conns[0].loss.advance(2399);
    assert_eq!(conns[0].loss.last_value(), None);
    // When the deadline is reached without delivery proof.
    conns[0].loss.advance(2400);
    // Then precisely the unresolved debits settle, dated by the original cohort end.
    assert_eq!(conns[0].loss.last_value(), Some(0.2));
    assert_eq!(conns[0].loss.last_cohort_ms(), Some(2000));
    assert!(conns[0].loss.loss_cohort_ok(2400, 10_000));
    clock.set(2401);
    for seq in 0..20 {
        ack(&mut conns, seq, (0, 0));
        ack(&mut conns, seq, (0, 0));
    }
    assert_eq!(conns[0].loss.last_value(), Some(0.2));
}

#[tokio::test]
async fn post_deadline_ack_cannot_forgive_even_an_open_cohort() {
    // Given sends early enough that the delivery deadline precedes cohort close.
    let clock = TestClock::new(1000);
    let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = vec![create_test_connection_to(peer.local_addr().unwrap()).await];
    send(&mut conns[0], 0..100, false).await;
    clock.set(1100);
    assert!(conns[0].handle_nak(0));
    // When the ACK arrives at the deadline, and is then replayed before close.
    clock.set(1500);
    ack(&mut conns, 0, (0, 0));
    clock.set(1600);
    ack(&mut conns, 0, (0, 0));
    conns[0].loss.advance(2000);
    // Then neither late delivery nor its duplicate cancels or doubles the debit.
    assert_eq!(conns[0].loss.last_value(), Some(0.01));
    assert_eq!(conns[0].delivery.delivered_bps(1600), 16.0 * 4.0);
}

#[tokio::test]
async fn closed_settlement_is_fenced_by_generation_and_recovery_epoch() {
    // Given closed-but-unsettled evidence and an old reader's ACK.
    let (clock, mut conns, _peer) = pending(100).await;
    assert_eq!(conns[0].loss.last_value(), None);
    clock.set(2200);
    for seq in 0..20 {
        ack(&mut conns, seq, (0, 1));
    }
    conns[0].loss.advance(2400);
    assert_eq!(conns[0].loss.last_value(), Some(0.2));
    // Given a fresh epoch with distinct new loss; old tokens must not cancel it.
    clock.set(2500);
    conns[0].loss.begin_recovered_epoch(2500);
    send(&mut conns[0], 100..200, false).await;
    assert!(conns[0].handle_nak(100));
    // When a valid reader finally confirms the old sequence.
    ack(&mut conns, 0, (0, 0));
    conns[0].loss.advance(3500);
    // Then the replacement epoch's loss survives.
    assert_eq!(conns[0].loss.last_value(), Some(0.01));
}

#[tokio::test]
async fn sub_floor_cohort_remains_ineligible_after_settlement() {
    // Given 99 accepted sends, with pending NAKs.
    let (clock, mut conns, _peer) = pending(99).await;
    // When some are recovered and the remainder pass the deadline.
    clock.set(2200);
    ack(&mut conns, 0, (0, 0));
    conns[0].loss.advance(2400);
    // Then settlement cannot promote a below-floor denominator.
    assert_eq!(conns[0].loss.last_value(), None);
    assert_eq!(conns[0].loss.last_cohort_ms(), None);
    assert!(!conns[0].loss.loss_cohort_ok(2400, 10_000));
}

#[tokio::test]
async fn genuinely_settled_loss_still_demotes_rejoining() {
    // Given a Rejoining link and a 20% debit not yet eligible as final loss.
    let (clock, mut conns, _peer) = pending(100).await;
    assert_eq!(conns[0].loss.last_value(), None);
    conns[0].connected = true;
    conns[0].health = HealthMachine::new(HealthState::Rejoining, 2200);
    // When housekeeping observes settled loss at the normal threshold.
    clock.set(2400);
    tick_health(&mut conns, &[], 2400);
    // Then Rejoining has no grace period or weaker degrade policy.
    assert_eq!(conns[0].health.state(), HealthState::Degraded);
    assert!(conns[0].health.loss_latched());
}
