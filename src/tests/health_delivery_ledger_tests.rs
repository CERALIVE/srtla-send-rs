use super::{DataSend, DeliveryAck, DeliveryLedger};

fn ack(seq: i32) -> DeliveryAck {
    DeliveryAck {
        seq,
        socket_generation: 0,
    }
}

#[test]
fn expiry_boundary_and_attempt_count_are_independent() {
    // Given two sends at the same time.
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        1,
        DataSend {
            sent_ms: 1000,
            len: 100,
        },
    );
    ledger.record_sent(
        2,
        DataSend {
            sent_ms: 1000,
            len: 200,
        },
    );
    // When one ACK arrives at 6000ms age and the other beyond the ceiling.
    assert!(ledger.acknowledge(ack(1), 7000));
    ledger.record_sent(
        3,
        DataSend {
            sent_ms: 7000,
            len: 300,
        },
    );
    assert!(!ledger.acknowledge(ack(2), 7001));
    // Then expiry is not proof and cannot zero the new attempt.
    assert_eq!(ledger.attempts_since_proof, 1);
    assert_eq!(ledger.delivered_bps(7001), 400.0);
}

#[test]
fn lru_refresh_keeps_retransmission_and_evicts_oldest() {
    // Given a full map, including a refreshed sequence at the same timestamp.
    let mut ledger = DeliveryLedger::default();
    for seq in 0..4096 {
        ledger.record_sent(
            seq,
            DataSend {
                sent_ms: 1000,
                len: 100,
            },
        );
    }
    ledger.record_sent(
        0,
        DataSend {
            sent_ms: 1000,
            len: 200,
        },
    );
    // When one more accepted send exceeds capacity.
    ledger.record_sent(
        4096,
        DataSend {
            sent_ms: 1000,
            len: 300,
        },
    );
    // Then the least recently used entry, not the refreshed entry, is evicted.
    assert_eq!(ledger.entries.len(), 4096);
    assert_eq!(ledger.attempts_since_proof, 4098);
    assert!(!ledger.acknowledge(ack(1), 1001));
    assert!(ledger.acknowledge(ack(0), 1001));
    assert_eq!(ledger.delivered_bps(1001), 800.0);
}

#[test]
fn old_token_cannot_consume_a_reused_sequence() {
    // Given the same sequence reused after a socket reset.
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        7,
        DataSend {
            sent_ms: 1000,
            len: 100,
        },
    );
    ledger.reset();
    ledger.record_sent(
        7,
        DataSend {
            sent_ms: 2000,
            len: 200,
        },
    );
    // When a queued old-generation token arrives before the current one.
    assert!(!ledger.acknowledge(ack(7), 2010));
    // Then it cannot remove or credit the new entry.
    assert_eq!(ledger.attempts_since_proof, 1);
    assert_eq!(ledger.last_data_proof_ms, 0);
    assert!(ledger.acknowledge(
        DeliveryAck {
            seq: 7,
            socket_generation: 1
        },
        2020
    ));
    assert_eq!(ledger.delivered_bps(2020), 800.0);
}

#[test]
fn duplicate_ack_is_one_shot_and_rate_expires_at_two_seconds() {
    // Given one credited send followed by another unproved attempt.
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        7,
        DataSend {
            sent_ms: 1000,
            len: 1316,
        },
    );
    assert!(ledger.acknowledge(ack(7), 1040));
    ledger.record_sent(
        8,
        DataSend {
            sent_ms: 1050,
            len: 100,
        },
    );
    // When the original ACK is replayed.
    assert!(!ledger.acknowledge(ack(7), 1060));
    // Then it neither credits bytes twice nor renews proof/attempts.
    assert_eq!(ledger.attempts_since_proof, 1);
    assert_eq!(ledger.last_data_proof_ms, 1040);
    assert_eq!(ledger.delivered_bps(3039), 5264.0);
    assert_eq!(ledger.delivered_bps(3040), 0.0);
}

#[test]
fn delivered_ring_is_bounded_by_millisecond_buckets() {
    // Given two ACKs per millisecond across three full seconds.
    let mut ledger = DeliveryLedger::default();
    // When crediting these distinct accepted attempts.
    for now in 0..3000 {
        for seq in [1, 2] {
            ledger.record_sent(
                seq,
                DataSend {
                    sent_ms: now,
                    len: 100,
                },
            );
            assert!(ledger.acknowledge(ack(seq), now));
        }
    }
    // Then the ring retains at most 2000 buckets and sums the whole 2s window.
    assert_eq!(ledger.delivered_bytes_window.len(), 2000);
    assert_eq!(ledger.delivered_bps(2999), 1_600_000.0);
}

#[test]
fn eviction_does_not_move_the_initial_proof_age_anchor() {
    // Given no proof yet, with the first send at monotonic zero.
    let mut ledger = DeliveryLedger::default();
    assert_eq!(ledger.proof_age_ms(0), None);
    ledger.record_sent(
        1,
        DataSend {
            sent_ms: 0,
            len: 100,
        },
    );
    // When a later send expires the first one.
    ledger.record_sent(
        2,
        DataSend {
            sent_ms: 6001,
            len: 100,
        },
    );
    // Then proof age remains time since first attempt, not time since eviction.
    assert_eq!(ledger.proof_age_ms(6001), Some(6001));
    assert_eq!(ledger.attempts_since_proof, 2);
    assert_eq!(ledger.entries.len(), 1);
}

#[test]
fn stale_entry_generation_is_rejected_even_with_current_token() {
    // Given an explicitly stale stored generation (reset normally removes it).
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        1,
        DataSend {
            sent_ms: 1000,
            len: 100,
        },
    );
    ledger.entries.get_mut(&1).unwrap().generation = u32::MAX;
    // When looking up that entry with a current token.
    let proved = ledger.acknowledge(ack(1), 1040);
    // Then the entry's own scope independently prevents proof.
    assert!(!proved);
    assert_eq!(ledger.last_data_proof_ms, 0);
}
