use super::{DataSend, DeliveryAck, DeliveryLedger};

#[test]
fn probe_proof_never_sets_original_delivery_timestamp() {
    // Given a fresh ledger with no normal DATA ACK.
    let mut ledger = DeliveryLedger::default();
    // When a duplicate probe proves liveness.
    ledger.record_probe_proof(1000);
    // Then general proof is fresh but original-only proof remains unknown.
    assert_eq!(ledger.latest_data_proof_ms(), Some(1000));
    assert_eq!(ledger.latest_original_delivery_ms(), None);
}

#[test]
fn original_delivery_at_zero_is_real_proof() {
    // Given accepted normal DATA at the monotonic origin.
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        7,
        DataSend {
            sent_ms: 0,
            len: 1332,
        },
    );
    // When the exact normal sequence is acknowledged at zero.
    assert!(ledger.acknowledge(
        DeliveryAck {
            seq: 7,
            socket_generation: 0
        },
        0
    ));
    // Then zero is preserved rather than treated as absent.
    assert_eq!(ledger.latest_original_delivery_ms(), Some(0));
}

#[test]
fn later_probe_cannot_refresh_an_original_delivery() {
    // Given a valid normal delivery at1040ms.
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        7,
        DataSend {
            sent_ms: 1000,
            len: 1332,
        },
    );
    assert!(ledger.acknowledge(
        DeliveryAck {
            seq: 7,
            socket_generation: 0
        },
        1040
    ));
    // When only a probe is acknowledged later.
    ledger.record_probe_proof(2000);
    // Then the original-delivery epoch does not move.
    assert_eq!(ledger.latest_original_delivery_ms(), Some(1040));
}

#[test]
fn generation_reset_clears_original_delivery() {
    // Given proven normal DATA on the previous socket.
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        7,
        DataSend {
            sent_ms: 1000,
            len: 1332,
        },
    );
    assert!(ledger.acknowledge(
        DeliveryAck {
            seq: 7,
            socket_generation: 0
        },
        1040
    ));
    // When the delivery generation rolls over.
    ledger.reset();
    // Then old original proof cannot validate the new path.
    assert_eq!(ledger.latest_original_delivery_ms(), None);
}
