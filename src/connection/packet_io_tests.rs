use tokio::net::UdpSocket;

use crate::registration::SrtlaRegistrationManager;
use crate::stats::SharedStats;
use crate::test_helpers::create_test_connection_to;

const HSRSP: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/srt-hsrsp-latency2000.bin"
));

#[tokio::test]
async fn packet_io_hsrsp_publishes_latency_and_forwards_identical_bytes() {
    // Given a real captured conclusion and a clone held by an independent consumer.
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(listener.local_addr().unwrap()).await;
    let mut reg = SrtlaRegistrationManager::new();
    let (forwarder, _rx) = tokio::sync::mpsc::unbounded_channel();
    let stats = SharedStats::new();
    let consumer = stats.clone();
    // When the production receive path processes the handshake.
    let incoming = conn
        .process_packet(0, &mut reg, &listener, &forwarder, None, HSRSP, &stats)
        .await
        .unwrap();
    // Then forwarding bytes are unchanged and latency is visible without housekeeping.
    assert_eq!(incoming.forward_to_client.len(), 1);
    assert_eq!(incoming.forward_to_client[0].as_slice(), HSRSP);
    assert_eq!(consumer.negotiated_latency_ms(), Some(2000));
}

#[tokio::test]
async fn packet_io_invalid_hsrsp_still_forwards_and_preserves_known_latency() {
    // Given a known observation and the captured packet with one extension-type byte flipped.
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(listener.local_addr().unwrap()).await;
    let mut reg = SrtlaRegistrationManager::new();
    let (forwarder, _rx) = tokio::sync::mpsc::unbounded_channel();
    let stats = SharedStats::new();
    stats.set_negotiated_latency_ms(500);
    let mut packet = HSRSP.to_vec();
    packet[65] ^= 1;
    // When parsing fails inside receive processing.
    let incoming = conn
        .process_packet(0, &mut reg, &listener, &forwarder, None, &packet, &stats)
        .await
        .unwrap();
    // Then parse failure neither consumes the packet nor overwrites a known delay.
    assert_eq!(incoming.forward_to_client.len(), 1);
    assert_eq!(incoming.forward_to_client[0].as_slice(), packet);
    assert_eq!(stats.negotiated_latency_ms(), Some(500));
}

#[tokio::test]
async fn packet_io_drain_sniffs_the_real_udp_handshake() {
    // Given the captured handshake sent through an actual loopback uplink socket.
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    receiver
        .send_to(HSRSP, conn.socket.local_addr().unwrap())
        .await
        .unwrap();
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut reg = SrtlaRegistrationManager::new();
    let (forwarder, _rx) = tokio::sync::mpsc::unbounded_channel();
    let stats = SharedStats::new();
    // When the alternate receive entry point drains the socket.
    let incoming = tokio::time::timeout(std::time::Duration::from_secs(1), async {
        loop {
            let incoming = conn
                .drain_incoming(0, &mut reg, &listener, &forwarder, None, &stats)
                .await
                .unwrap();
            if incoming.read_any {
                break incoming;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    // Then it shares the same atomic observation and byte-preserving forwarding.
    assert_eq!(incoming.forward_to_client.len(), 1);
    assert_eq!(incoming.forward_to_client[0].as_slice(), HSRSP);
    assert_eq!(stats.negotiated_latency_ms(), Some(2000));
}
