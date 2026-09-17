#![cfg(unix)]

use std::net::Ipv4Addr;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use srtla_send::protocol::{
    SRT_TYPE_NAK, SRTLA_TYPE_KEEPALIVE, SRTLA_TYPE_REG1, SRTLA_TYPE_REG2, SRTLA_TYPE_REG3,
    extract_keepalive_conn_info, get_packet_type,
};
use tokio::net::UdpSocket;

struct Sender(Child);

impl Drop for Sender {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn run_reports(measured: bool, bypass: bool) {
    // Given a real sender process and one loopback receiver speaking registration.
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_srtla_send"));
    let dir = tempfile::tempdir_in(binary.parent().unwrap()).unwrap();
    let executable = dir.path().join("nak-test");
    std::fs::hard_link(binary, &executable).unwrap();
    let ips = dir.path().join("ips");
    std::fs::write(&ips, "127.0.0.1\n").unwrap();
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let reservation = UdpSocket::bind("[::]:0").await.unwrap();
    let port = reservation.local_addr().unwrap().port();
    drop(reservation);
    let _sender = Sender(
        Command::new(&executable)
            .env(
                "SRTLA_DISABLE_PREMATURE_NAK_RULE",
                if bypass { "1" } else { "0" },
            )
            .args([
                port.to_string(),
                "127.0.0.1".into(),
                receiver.local_addr().unwrap().port().to_string(),
            ])
            .arg(ips)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    let mut buf = [0; 1500];
    let peer = loop {
        let (len, peer) = receiver.recv_from(&mut buf).await.unwrap();
        match get_packet_type(&buf[..len]) {
            Some(SRTLA_TYPE_REG1) => {
                buf[..2].copy_from_slice(&SRTLA_TYPE_REG2.to_be_bytes());
                buf[len - 1] ^= 1;
                receiver.send_to(&buf[..len], peer).await.unwrap();
            }
            Some(SRTLA_TYPE_REG2) => {
                receiver
                    .send_to(&SRTLA_TYPE_REG3.to_be_bytes(), peer)
                    .await
                    .unwrap();
            }
            Some(SRTLA_TYPE_KEEPALIVE) => {
                if measured {
                    let timestamp = u64::from_be_bytes(buf[2..10].try_into().unwrap());
                    buf[2..10].copy_from_slice(&(timestamp - 1000).to_be_bytes());
                    receiver.send_to(&buf[..len], peer).await.unwrap();
                }
                break peer;
            }
            _ => {}
        }
    };
    let encoder = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let handshake = include_bytes!("fixtures/srt-hsrsp-latency2000.bin");
    encoder
        .send_to(handshake, (Ipv4Addr::LOCALHOST, port))
        .await
        .unwrap();
    loop {
        let (len, _) = receiver.recv_from(&mut buf).await.unwrap();
        if &buf[..len] == handshake {
            break;
        }
    }
    // The forwarded marker fences processing of the preceding RTT echo on this uplink.
    receiver.send_to(handshake, peer).await.unwrap();
    let len = encoder.recv(&mut buf).await.unwrap();
    assert_eq!(&buf[..len], handshake);
    let mut data = [0; 1316];
    data[..4].copy_from_slice(&42_u32.to_be_bytes());
    encoder
        .send_to(&data, (Ipv4Addr::LOCALHOST, port))
        .await
        .unwrap();
    loop {
        let (len, _) = receiver.recv_from(&mut buf).await.unwrap();
        if &buf[..len] == data {
            break;
        }
    }
    let mut nak = [0x5a; 20];
    nak[..2].copy_from_slice(&SRT_TYPE_NAK.to_be_bytes());
    nak[16..].copy_from_slice(&42_u32.to_be_bytes());
    // When three immediate reports name accepted DATA, without an intervening ACK.
    let started = std::time::Instant::now();
    for _ in 0..3 {
        receiver.send_to(&nak, peer).await.unwrap();
        let len = encoder.recv(&mut buf).await.unwrap();
        assert_eq!(&buf[..len], &nak);
    }
    assert!(
        started.elapsed() < Duration::from_millis(400),
        "host failed the 500ms protection timing budget"
    );
    let first = loop {
        let (len, _) = receiver.recv_from(&mut buf).await.unwrap();
        if let Some(info) = extract_keepalive_conn_info(&buf[..len]) {
            break info;
        }
    };
    // Then wire status proves retained flight without RTT-free blanket forgiveness.
    let protected = measured && !(bypass && cfg!(feature = "test-internals"));
    assert_eq!(first.in_flight, i32::from(protected));
    assert_eq!(first.nak_count, u32::from(!protected));
    // When the same NAK is mature, Then it is forwarded and ordinary loss is recorded.
    receiver.send_to(&nak, peer).await.unwrap();
    let len = encoder.recv(&mut buf).await.unwrap();
    assert_eq!(&buf[..len], &nak);
    let final_info = loop {
        let (len, _) = receiver.recv_from(&mut buf).await.unwrap();
        if let Some(info) = extract_keepalive_conn_info(&buf[..len]) {
            break info;
        }
    };
    assert_eq!(final_info.in_flight, 0);
    assert_eq!(final_info.nak_count, 1);
}

#[tokio::test]
async fn premature_nak_binary_preserves_encoder_reports_and_in_flight_data() {
    tokio::time::timeout(Duration::from_secs(10), run_reports(true, false))
        .await
        .unwrap();
}

#[tokio::test]
async fn premature_nak_binary_without_rtt_keeps_normal_penalty() {
    tokio::time::timeout(Duration::from_secs(10), run_reports(false, false))
        .await
        .unwrap();
}

#[tokio::test]
async fn premature_nak_bypass_is_effective_only_in_test_internals_builds() {
    // Given a measured link and the process-local M2 escape hatch, When immediate
    // NAKs arrive, Then only a test-internals binary takes the mature penalty path.
    tokio::time::timeout(Duration::from_secs(10), run_reports(true, true))
        .await
        .unwrap();
}
