use super::*;

fn cohort(tracker: &mut LossTracker, start: u64, (sends, naks): (u32, u32)) {
    for _ in 0..sends {
        tracker.record_send(start);
    }
    for _ in 0..naks {
        tracker.record_data_nak(start + 500);
    }
    tracker.advance(start + 1000);
}

#[test]
fn loss_cohort_floor_discards_fifty_sends_and_fifty_naks() {
    // Given an unmeasured link.
    let mut tracker = LossTracker::new(0);
    // When a fully lost but sub-floor cohort completes.
    cohort(&mut tracker, 0, (50, 50));
    // Then no loss evidence is manufactured.
    assert_eq!(tracker.last_value(), None);
    assert_eq!(tracker.last_cohort_ms(), None);
    assert!(!tracker.loss_cohort_ok(1000, 10_000));
    assert_eq!(tracker.probe_loss(), None);
}

#[test]
fn loss_sub_floor_cohort_preserves_existing_ewma_and_timestamp() {
    // Given qualified loss evidence of 20%.
    let mut tracker = LossTracker::new(0);
    cohort(&mut tracker, 0, (100, 20));
    // When fifty sends and fifty NAKs occur in the next cohort.
    cohort(&mut tracker, 1000, (50, 50));
    // Then the guard discards rather than smooths the 100% observation.
    assert_eq!(tracker.last_value(), Some(0.2));
    assert_eq!(tracker.last_cohort_ms(), Some(1000));
    assert!(!tracker.loss_cohort_ok(2000, 10_000));
}

#[test]
fn loss_cohort_floor_and_one_second_boundary_are_inclusive() {
    // Given exactly the load floor, but an unfinished cohort.
    let mut tracker = LossTracker::new(0);
    assert_eq!(LOSS_COHORT_MIN_SENDS, 100);
    for _ in 0..LOSS_COHORT_MIN_SENDS {
        tracker.record_send(0);
    }
    tracker.advance(999);
    assert_eq!(tracker.last_value(), None);
    // When its first full second elapses.
    tracker.advance(1000);
    // Then zero loss is measured (not missing), at the cohort end.
    assert_eq!(tracker.last_value(), Some(0.0));
    assert_eq!(tracker.last_cohort_ms(), Some(1000));
    assert!(tracker.loss_cohort_ok(1000, 10_000));
}

#[test]
fn loss_ewma_uses_point_two_alpha_and_converges() {
    // Given a seed at 100% loss.
    let mut tracker = LossTracker::new(0);
    cohort(&mut tracker, 0, (100, 100));
    cohort(&mut tracker, 1000, (100, 0));
    assert_eq!(tracker.last_value(), Some(0.8));
    // When repeated qualifying cohorts measure 20% loss.
    for second in 2..102 {
        cohort(&mut tracker, second * 1000, (100, 20));
    }
    // Then EWMA converges to that fraction, not a percentage or NAK count.
    assert!((tracker.last_value().unwrap() - 0.2).abs() < 1e-8);
}

#[test]
fn loss_stale_detection_retains_the_original_evidence_date() {
    // Given evidence published at t=1000.
    let mut tracker = LossTracker::new(0);
    cohort(&mut tracker, 0, (100, 10));
    assert!(!tracker.is_stale(10_999, 10_000));
    // When the exact stale threshold passes without new load.
    tracker.advance(11_000);
    // Then stale evidence remains available, but cannot qualify normal loss.
    assert!(tracker.is_stale(11_000, 10_000));
    assert!(!tracker.loss_cohort_ok(11_000, 10_000));
    assert_eq!(tracker.last_cohort_ms(), Some(1000));
    assert_eq!(tracker.last_value(), Some(0.1));
}

#[test]
fn loss_long_idle_gap_neither_combines_cohorts_nor_refreshes_evidence() {
    // Given a loaded first cohort whose completion was not polled.
    let mut tracker = LossTracker::new(0);
    for _ in 0..100 {
        tracker.record_send(0);
    }
    // When the next event arrives after ten empty cohorts.
    tracker.record_send(11_500);
    // Then the first cohort is dated by its end, not the late observation.
    assert_eq!(tracker.last_cohort_ms(), Some(1000));
    assert!(tracker.is_stale(11_500, 10_000));
    assert!(!tracker.loss_cohort_ok(11_500, 10_000));
}

#[test]
fn loss_sub_floor_cohorts_never_accumulate_towards_the_floor() {
    // Given several individually sub-floor cohorts.
    let mut tracker = LossTracker::new(0);
    // When their combined send count exceeds 100.
    for second in 0..5 {
        cohort(&mut tracker, second * 1000, (99, 99));
    }
    // Then all are still discarded independently.
    assert_eq!(tracker.last_value(), None);
    assert!(tracker.is_stale(5000, 10_000));
}

#[test]
fn loss_boundary_send_belongs_to_the_next_cohort() {
    // Given 99 sends before the boundary.
    let mut tracker = LossTracker::new(0);
    for _ in 0..99 {
        tracker.record_send(0);
    }
    // When the hundredth send arrives at t=1000.
    tracker.record_send(1000);
    tracker.advance(2000);
    // Then neither 99 nor 1 satisfies the floor.
    assert_eq!(tracker.last_value(), None);
}

#[test]
fn delayed_feedback_preserves_evidence_age_and_history_is_bounded() {
    // Given one closed, loss-free cohort.
    let mut tracker = LossTracker::new(0);
    cohort(&mut tracker, 0, (100, 0));
    // When feedback arrives eight seconds after closure, then only its original date remains.
    tracker.record_data_nak_for_send(0, 9000);
    assert_eq!(tracker.last_value(), Some(0.01));
    assert_eq!(tracker.last_cohort_ms(), Some(1000));
    assert!(!tracker.loss_cohort_ok(9000, 10_000));
    tracker.record_data_nak_for_send(0, 11_000);
    assert_eq!(tracker.last_value(), Some(0.01));
    assert!(tracker.is_stale(11_000, 10_000));
    for second in 12..1000 {
        cohort(&mut tracker, second * 1000, (100, 1));
        assert!(tracker.closed.len() <= 10);
    }
}
