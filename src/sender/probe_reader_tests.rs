use std::collections::HashMap;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::timeout;

use super::ack::{AckContext, AckPolicy, apply_srtla_ack};
use super::uplink::{create_uplink_channel, restart_reader_for, sync_readers};
use crate::connection::delivery::DataSend;
use crate::protocol::SRTLA_TYPE_ACK;
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn probe_reader_captures_generation_at_spawn_not_dispatch() {
    // Given a real reader task and a queued packet from generation zero.
    let _clock = TestClock::new(200);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    let mut conns = vec![conn];
    let mut readers = HashMap::new();
    let (tx, mut rx) = create_uplink_channel();
    sync_readers(&conns, &mut readers, &tx);
    let mut wire = SRTLA_TYPE_ACK.to_be_bytes().to_vec();
    wire.extend_from_slice(&7u32.to_be_bytes());
    receiver
        .send_to(&wire, conns[0].socket.local_addr().unwrap())
        .await
        .unwrap();
    let old = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    conns[0].mark_for_recovery();
    conns[0].delivery.record_sent(
        7,
        DataSend {
            sent_ms: 200,
            len: 1316,
        },
    );
    // When the queued old-reader event is applied after invalidation.
    apply_srtla_ack(
        &mut conns,
        7,
        AckContext {
            arrival_idx: 0,
            reader_generation: old.reader_generation,
            policy: AckPolicy::Adaptive,
        },
    );
    // Then same-sequence new evidence is untouched, while a restarted reader proves it.
    assert_eq!(old.reader_generation, 0);
    assert_eq!(&old.bytes[..], &wire);
    assert_eq!(conns[0].delivery.last_data_proof_ms, 0);
    restart_reader_for(&conns[0], &mut readers, &tx);
    receiver
        .send_to(&wire, conns[0].socket.local_addr().unwrap())
        .await
        .unwrap();
    let fresh = timeout(Duration::from_secs(1), rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fresh.reader_generation, conns[0].delivery.socket_generation);
    apply_srtla_ack(
        &mut conns,
        7,
        AckContext {
            arrival_idx: 0,
            reader_generation: fresh.reader_generation,
            policy: AckPolicy::Adaptive,
        },
    );
    assert_eq!(conns[0].delivery.last_data_proof_ms, 200);
    for reader in readers.into_values() {
        reader.handle.abort();
    }
}
