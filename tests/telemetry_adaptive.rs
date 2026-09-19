#![cfg(unix)]

use std::collections::HashSet;
use std::fs::OpenOptions;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
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

struct ChildPaths<'a> {
    executable: &'a Path,
    ips: &'a Path,
    stats: &'a Path,
    log: &'a Path,
}

/// Bounded spawn attempts. Each attempt draws an independent ephemeral port, so
/// three is already far past the measured loss rate while still failing fast on
/// a real bind defect.
const SPAWN_ATTEMPTS: u32 = 3;
/// The sender binds its listener before awaiting anything else, so readiness is
/// milliseconds away; this budget only has to outlast process startup.
const LISTEN_READY_TIMEOUT: Duration = Duration::from_secs(2);
const READY_POLL: Duration = Duration::from_millis(10);

/// Spawns the sender and returns only once the child owns the SRT listen port.
///
/// Reserving an ephemeral port and releasing it before `exec` leaves a window in
/// which any other process on the host can take it; the child then dies roughly
/// a millisecond later with `bind local SRT UDP listener: Address already in
/// use`, which reads exactly like the sender crashing. The window cannot be
/// closed by holding the reservation open, because the child binds the port as a
/// plain unicast UDP socket with no `SO_REUSEPORT`, and teaching production to
/// share a listen port to suit a test would be the wrong direction entirely. So
/// the acquisition is confirmed instead of assumed, and a port lost in that
/// window — identified by the child's own bind error, never by a bare early exit
/// — is re-drawn. Every other failure stays fatal on the first occurrence.
async fn spawn_listening(paths: &ChildPaths<'_>, receiver_port: u16) -> (Sender, u16) {
    let mut attempts_left = SPAWN_ATTEMPTS;
    loop {
        attempts_left -= 1;
        let reservation = UdpSocket::bind("[::]:0").await.unwrap();
        let port = reservation.local_addr().unwrap().port();
        drop(reservation);
        // Append, so a re-drawn port keeps the losing attempt's log for diagnosis.
        let log = OpenOptions::new().append(true).open(paths.log).unwrap();
        let mut sender = Sender(
            Command::new(paths.executable)
                .env("RUST_LOG", "info")
                .args([
                    port.to_string(),
                    "127.0.0.1".into(),
                    receiver_port.to_string(),
                ])
                .arg(paths.ips)
                .args(["--mode", "enhanced", "--stats-file-interval", "100"])
                .arg("--stats-file")
                .arg(paths.stats)
                .stdin(Stdio::null())
                .stdout(Stdio::from(log.try_clone().unwrap()))
                .stderr(Stdio::from(log))
                .spawn()
                .unwrap(),
        );
        if await_listening(&mut sender, paths.log, port).await {
            return (sender, port);
        }
        drop(sender);
        assert!(
            attempts_left > 0,
            "SRT listen port taken in the reservation window on all {SPAWN_ATTEMPTS} attempts; \
             log:\n{}",
            std::fs::read_to_string(paths.log).unwrap(),
        );
    }
}

/// Bounded readiness poll for the child's own report that it bound `port`.
///
/// Returns `false` for exactly one retryable outcome — the reserved port was
/// taken between release and the child's bind. Any other early exit, and the
/// absence of readiness altogether, panics with the captured log.
async fn await_listening(sender: &mut Sender, log_path: &Path, port: u16) -> bool {
    let listening = format!("listening for SRT on [::]:{port}");
    let deadline = tokio::time::Instant::now() + LISTEN_READY_TIMEOUT;
    loop {
        let log = std::fs::read_to_string(log_path).unwrap();
        if log.contains(&listening) {
            return true;
        }
        if let Some(status) = sender.0.try_wait().unwrap() {
            // The listener bind precedes every await in the sender, so a child
            // that exited without that line never owned the port. Only the bind
            // error itself is treated as the lost-reservation race.
            if log.contains("bind local SRT UDP listener") && log.contains("Address already in use")
            {
                return false;
            }
            panic!("sender exited before binding the SRT listener: {status:?}; log:\n{log}");
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "sender never reported its SRT listener within {LISTEN_READY_TIMEOUT:?}; log:\n{log}",
        );
        tokio::time::sleep(READY_POLL).await;
    }
}

async fn live_snapshot() -> Snapshot {
    let binary = Path::new(env!("CARGO_BIN_EXE_srtla_send"));
    // A same-filesystem hard link executes the exact Cargo artifact under a private
    // kernel process name, outside host-wide `killall srtla_send` cleanup.
    let dir = tempfile::tempdir_in(binary.parent().unwrap()).unwrap();
    let executable = dir.path().join(format!("atel-{}", std::process::id()));
    std::fs::hard_link(binary, &executable).unwrap();
    let ips = dir.path().join("ips");
    let stats = dir.path().join("stats.json");
    let log_path = dir.path().join("sender.log");
    std::fs::File::create(&log_path).unwrap();
    std::fs::write(&ips, "127.0.0.1\n127.0.0.2\n").unwrap();
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let (mut sender, port) = spawn_listening(
        &ChildPaths {
            executable: &executable,
            ips: &ips,
            stats: &stats,
            log: &log_path,
        },
        receiver.local_addr().unwrap().port(),
    )
    .await;
    // Unchanged 15s telemetry budget, started where the scenario itself starts so
    // that confirming the listener cannot borrow from it.
    let deadline = tokio::time::sleep(Duration::from_secs(15));
    tokio::pin!(deadline);
    #[cfg(target_os = "linux")]
    assert_eq!(
        std::fs::read_to_string(format!("/proc/{}/comm", sender.0.id())).unwrap(),
        format!("atel-{}\n", std::process::id()),
        "test child must not share the production process name used by host-wide cleanup",
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
            _ = &mut deadline => panic!("live telemetry deadline exceeded; sender log:\n{}",
                std::fs::read_to_string(&log_path).unwrap()),
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
                if let Some(status) = sender.0.try_wait().unwrap() {
                    panic!("sender pid={} exited during live telemetry scenario: {status:?}; log:\n{}",
                        sender.0.id(), std::fs::read_to_string(&log_path).unwrap());
                }
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

/// Whether a second loopback source IP can be bound on this host. macOS only
/// configures 127.0.0.1 by default, so this two-link scenario self-skips there
/// while still running for real on the Linux device/CI target.
fn second_loopback_bindable() -> bool {
    std::net::UdpSocket::bind("127.0.0.2:0").is_ok()
}

#[tokio::test]
async fn live_binary_publishes_zero_weight_for_data_blackhole_with_keepalives() {
    if !second_loopback_bindable() {
        eprintln!("Skipping: 127.0.0.2 is not locally bindable on this host");
        return;
    }
    // Given two registered loopback links; one echoes keepalives but never ACKs DATA.
    // When real traffic drives the sender's housekeeping and stats-file publication.
    let snapshot = live_snapshot().await;
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
