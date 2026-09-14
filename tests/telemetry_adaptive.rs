#![cfg(unix)]

use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use serde::Deserialize;
use srtla_send::protocol::{
    SRTLA_TYPE_KEEPALIVE, SRTLA_TYPE_REG1, SRTLA_TYPE_REG2, SRTLA_TYPE_REG3, create_ack_packet,
    get_packet_type,
};
use tokio::net::UdpSocket;

#[derive(Debug, Deserialize)]
struct Link {
    health: Option<String>,
    priority: Option<f64>,
    weight_percent: u8,
    bytes_sent_total: u64,
}

#[derive(Debug, Deserialize)]
struct Snapshot {
    schema_version: u32,
    connections: Vec<Link>,
}

struct Sender(Child);

impl Drop for Sender {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn live_snapshot() -> Snapshot {
    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips");
    let stats = dir.path().join("stats.json");
    std::fs::write(&ips, "127.0.0.1\n127.0.0.2\n").unwrap();
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let reservation = UdpSocket::bind("[::]:0").await.unwrap();
    let port = reservation.local_addr().unwrap().port();
    drop(reservation);
    let mut sender = Sender(
        Command::new(env!("CARGO_BIN_EXE_srtla_send"))
            .args([
                port.to_string(),
                "127.0.0.1".into(),
                receiver.local_addr().unwrap().port().to_string(),
            ])
            .arg(&ips)
            .args(["--mode", "adaptive", "--stats-file-interval", "100"])
            .arg("--stats-file")
            .arg(&stats)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let source = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut send_tick = tokio::time::interval(Duration::from_millis(2));
    send_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut observe_tick = tokio::time::interval(Duration::from_millis(100));
    let mut registered: HashSet<SocketAddr> = HashSet::new();
    let mut buf = [0_u8; 1500];
    let mut packet = [0_u8; 1316];
    let mut seq = 0_u32;
    loop {
        tokio::select! {
            result = receiver.recv_from(&mut buf) => {
                let (n, peer) = result.unwrap();
                match get_packet_type(&buf[..n]) {
                    Some(SRTLA_TYPE_REG1) => {
                        buf[..2].copy_from_slice(&SRTLA_TYPE_REG2.to_be_bytes());
                        buf[n - 1] ^= 1;
                        receiver.send_to(&buf[..n], peer).await.unwrap();
                    }
                    Some(SRTLA_TYPE_REG2) => {
                        receiver.send_to(&SRTLA_TYPE_REG3.to_be_bytes(), peer).await.unwrap();
                        registered.insert(peer);
                    }
                    Some(SRTLA_TYPE_KEEPALIVE) => {
                        receiver.send_to(&buf[..n], peer).await.unwrap();
                    }
                    _ if n == 1316 && peer.ip() == Ipv4Addr::new(127, 0, 0, 2) => {
                        let ack = create_ack_packet(&[u32::from_be_bytes(buf[..4].try_into().unwrap())]);
                        receiver.send_to(&ack, peer).await.unwrap();
                    }
                    _ => {}
                }
            }
            _ = send_tick.tick(), if registered.len() == 2 => {
                packet[..4].copy_from_slice(&seq.to_be_bytes());
                seq += 1;
                source.send_to(&packet, (Ipv4Addr::LOCALHOST, port)).await.unwrap();
            }
            _ = observe_tick.tick() => {
                assert!(sender.0.try_wait().unwrap().is_none(), "sender exited during live telemetry scenario");
                match std::fs::read_to_string(&stats) {
                    Ok(json) => {
                        let snapshot: Snapshot = serde_json::from_str(&json).unwrap();
                        assert!(snapshot.connections.iter().all(|link| link.health.is_some()), "adaptive snapshot omits health: {json}");
                        if let [held, carrier] = snapshot.connections.as_slice()
                            && held.health.as_deref() == Some("stalled")
                            && held.weight_percent == 0 && carrier.weight_percent == 100 {
                            return snapshot;
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => panic!("stats file: {error}"),
                }
            }
        }
    }
}

#[tokio::test]
async fn live_binary_publishes_zero_weight_for_data_blackhole_with_keepalives() {
    // Given two registered loopback links; one echoes keepalives but never ACKs DATA.
    // When real traffic drives the sender's housekeeping and stats-file publication.
    let snapshot = tokio::time::timeout(Duration::from_secs(15), live_snapshot())
        .await
        .unwrap();
    // Then the live wire document reports the actual hold, without inventing priority.
    assert_eq!(snapshot.schema_version, 1);
    assert_eq!(snapshot.connections[0].health.as_deref(), Some("stalled"));
    assert_eq!(snapshot.connections[0].weight_percent, 0);
    assert_eq!(snapshot.connections[1].weight_percent, 100);
    assert!(
        snapshot
            .connections
            .iter()
            .all(|link| link.bytes_sent_total > 0 && link.priority.is_none())
    );
}
