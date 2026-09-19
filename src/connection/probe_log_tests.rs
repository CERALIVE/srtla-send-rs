use super::probe::{PROBE_LOG_CAPACITY, ProbeLog, ProbeTrain};

fn train(id: u64, start: u64) -> ProbeTrain {
    ProbeTrain {
        id,
        started_ms: start,
        deadline_ms: start + 4100,
    }
}

#[test]
fn probe_five_of_ten_is_ok_but_partial_train_is_not() {
    // Given five ACKed probes but only nine transmitted DATA copies.
    let mut log = ProbeLog::default();
    for seq in 0..9 {
        log.record_sent(seq, train(1, 0), 0);
    }
    for seq in 0..5 {
        assert!(log.acknowledge(seq, 100));
    }
    assert_eq!(log.rounds_ok(), (0, None));
    // When the tenth is accepted, then the train meets the 5/10 criterion.
    log.record_sent(9, train(1, 0), 100);
    assert_eq!(log.rounds_ok(), (1, Some(0)));
    assert_eq!(log.probe_loss(), None);
}

#[test]
fn probe_expired_trains_count_as_failed_in_loss() {
    // Given two ten-copy trains with only one ACK total.
    let mut log = ProbeLog::default();
    for seq in 0..20 {
        log.record_sent(seq, train(if seq < 10 { 1 } else { 2 }, 0), 0);
    }
    assert!(log.acknowledge(0, 100));
    // When both deadlines expire, then loss includes both failed trains.
    log.advance(4101);
    assert_eq!(log.probe_loss(), Some(0.95));
    assert_eq!(log.rounds_ok(), (0, None));
    assert!(log.probe_log.is_empty());
}

#[test]
fn probe_rounds_and_loss_expire_after_idle() {
    // Given two entirely successful trains.
    let mut log = ProbeLog::default();
    for seq in 0..20 {
        log.record_sent(seq, train(if seq < 10 { 1 } else { 2 }, 0), 0);
    }
    for seq in 0..20 {
        assert!(log.acknowledge(seq, 100));
    }
    assert_eq!(log.probe_loss(), Some(0.0));
    assert_eq!(log.rounds_ok(), (2, Some(0)));
    // When no fresh train exists after two train deadlines, then old success is unknown.
    log.advance(8201);
    assert_eq!(log.probe_loss(), None);
    assert_eq!(log.rounds_ok(), (0, None));
}

#[test]
fn probe_log_is_bounded_and_lru_refreshed() {
    // Given a full log, with sequence zero refreshed to newest.
    let mut log = ProbeLog::default();
    for seq in 0..256 {
        log.record_sent(seq, train(u64::try_from(seq / 10).unwrap(), 0), 0);
    }
    log.record_sent(0, train(0, 0), 1);
    // When an additional copy is accepted, then the oldest unrefreshed entry is evicted.
    log.record_sent(256, train(25, 0), 2);
    assert_eq!(log.probe_log.len(), PROBE_LOG_CAPACITY);
    assert!(log.probe_log.contains_key(&0));
    assert!(!log.probe_log.contains_key(&1));
    assert!(log.probe_log.contains_key(&256));
}

#[test]
fn probe_ack_deadline_is_inclusive_and_replays_are_inert() {
    // Given a pending copy at the exact train deadline.
    let mut log = ProbeLog::default();
    log.record_sent(1, train(1, 0), 0);
    // When acknowledged at the boundary, then exactly one proof is accepted.
    assert!(log.acknowledge(1, 4100));
    assert!(!log.acknowledge(1, 4100));
    assert!(!log.acknowledge(1, 4101));
}

#[test]
fn probe_reused_sequence_cannot_double_credit_a_train() {
    // Given an already consumed probe sequence.
    let mut log = ProbeLog::default();
    log.record_sent(1, train(1, 0), 0);
    assert!(log.acknowledge(1, 100));
    // When the same sequence is offered again, then its replay proves nothing.
    log.record_sent(1, train(1, 0), 200);
    assert!(!log.acknowledge(1, 300));
}

#[test]
fn probe_partial_expired_train_cannot_be_ok() {
    // Given five sent/ACKed probes, not a complete ten-sequence train.
    let mut log = ProbeLog::default();
    for seq in 0..5 {
        log.record_sent(seq, train(1, 0), 0);
    }
    for seq in 0..5 {
        assert!(log.acknowledge(seq, 100));
    }
    // When its deadline expires, then an incomplete train is failed.
    log.advance(4101);
    assert_eq!(log.rounds_ok(), (0, None));
}

#[test]
fn probe_loss_exactly_five_percent_meets_clear_threshold() {
    // Given twenty copies with nineteen ACKs across two trains.
    let mut log = ProbeLog::default();
    for seq in 0..20 {
        log.record_sent(seq, train(if seq < 10 { 1 } else { 2 }, 0), 0);
    }
    for seq in 0..19 {
        assert!(log.acknowledge(seq, 100));
    }
    // When the remaining copy expires, then one loss in twenty is exactly the threshold.
    log.advance(4101);
    assert_eq!(log.probe_loss(), Some(0.05));
}
