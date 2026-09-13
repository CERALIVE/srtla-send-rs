//! Real sender binary: loopback wire forwarding and Unix JSON-RPC observation.
#![cfg(unix)]

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use serde::Deserialize;
use srtla_send::protocol::{SRTLA_TYPE_REG1, SRTLA_TYPE_REG2, SRTLA_TYPE_REG3, get_packet_type};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UdpSocket, UnixStream};

const HSRSP: &[u8] = include_bytes!("fixtures/srt-hsrsp-latency2000.bin");

#[derive(Debug, Deserialize)]
struct Status {
    negotiated_latency_ms: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Response {
    result: Status,
}

struct Sender(Child);

impl Drop for Sender {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn forwarded_status(reply: &[u8]) -> Status {
    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips");
    std::fs::write(&ips, "127.0.0.1\n").unwrap();
    let control = dir.path().join("control.sock");
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let reservation = UdpSocket::bind("[::]:0").await.unwrap();
    let listen_port = reservation.local_addr().unwrap().port();
    drop(reservation);
    let sender = Sender(
        Command::new(env!("CARGO_BIN_EXE_srtla_send"))
            .args([
                listen_port.to_string(),
                "127.0.0.1".to_string(),
                receiver.local_addr().unwrap().port().to_string(),
            ])
            .arg(&ips)
            .arg("--control-socket")
            .arg(&control)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap(),
    );

    let mut buf = [0_u8; 1500];
    loop {
        let (n, peer) = receiver.recv_from(&mut buf).await.unwrap();
        match get_packet_type(&buf[..n]) {
            Some(SRTLA_TYPE_REG1) => {
                buf[..2].copy_from_slice(&SRTLA_TYPE_REG2.to_be_bytes());
                buf[n - 1] ^= 1;
                receiver.send_to(&buf[..n], peer).await.unwrap();
            }
            Some(SRTLA_TYPE_REG2) => {
                receiver
                    .send_to(&SRTLA_TYPE_REG3.to_be_bytes(), peer)
                    .await
                    .unwrap();
                break;
            }
            _ => {}
        }
    }

    let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    client
        .send_to(HSRSP, (std::net::Ipv4Addr::LOCALHOST, listen_port))
        .await
        .unwrap();
    let peer = loop {
        let (n, peer) = receiver.recv_from(&mut buf).await.unwrap();
        if &buf[..n] == HSRSP {
            break peer;
        }
    };
    receiver.send_to(reply, peer).await.unwrap();
    let (n, _) = client.recv_from(&mut buf).await.unwrap();
    assert_eq!(&buf[..n], reply, "sender must preserve the entire datagram");

    let stream = loop {
        match UnixStream::connect(&control).await {
            Ok(stream) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                tokio::task::yield_now().await;
            }
            Err(error) => panic!("control socket connect: {error}"),
        }
    };
    let mut stream = BufReader::new(stream);
    stream
        .get_mut()
        .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"get-status\"}\n")
        .await
        .unwrap();
    let mut response = String::new();
    stream.read_line(&mut response).await.unwrap();
    let response: Response = serde_json::from_str(&response).unwrap();
    drop(sender);
    response.result
}

#[tokio::test]
async fn captured_hsrsp_reaches_client_and_live_status() {
    // Given the real fixture, When forwarded by the binary, Then status exposes 2000 ms.
    let status = tokio::time::timeout(Duration::from_secs(10), forwarded_status(HSRSP))
        .await
        .unwrap();
    assert_eq!(status.negotiated_latency_ms, Some(2000));
}

#[tokio::test]
async fn malformed_hsrsp_reaches_client_without_inventing_latency() {
    // Given a flipped extension type, When forwarded by the binary, Then delay stays unknown.
    let mut reply = HSRSP.to_vec();
    reply[65] ^= 1;
    let status = tokio::time::timeout(Duration::from_secs(10), forwarded_status(&reply))
        .await
        .unwrap();
    assert_eq!(status.negotiated_latency_ms, None);
}
