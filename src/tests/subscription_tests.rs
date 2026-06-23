//! Stats-subscription push tests (ADR-001 / T4).
//!
//! Covers the `subscribe-events` event stream on the Unix `--control-socket`:
//! a loopback roundtrip that the pushed frame is a JSON-RPC `"event"`
//! notification carrying the full ADR-001 snapshot, last-known-state replay,
//! the bounded channel never blocking the broadcast (hot path), dual-publish
//! parity with the `--stats-file` sink, and dead-subscriber cleanup.
//!
//! Gated to `unix` because the control socket is Unix-only.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::config::{DynamicConfig, spawn_config_listener};
use crate::stats::SharedStats;
use crate::subscription::SubscriptionManager;
use crate::telemetry_file::{TelemetryConn, build_telemetry_json, write_atomic};

const SUBSCRIBE_FRAME: &str = r#"{"jsonrpc":"2.0","method":"subscribe-events","id":1}"#;

fn sample_conn() -> TelemetryConn {
    TelemetryConn {
        conn_id: 0,
        rtt_ms: 42,
        nak_count: 3,
        weight_percent: 85,
        window: 8192,
        in_flight: 100,
        bitrate_bytes_per_sec: 312_500,
    }
}

fn spawn_listener_with(
    config: &DynamicConfig,
    subscriptions: &SubscriptionManager,
) -> (String, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let path = dir.path().join("control.sock");
    let path_str = path.to_str().expect("utf-8 socket path").to_string();
    spawn_config_listener(
        config.clone(),
        Some(path_str.clone()),
        SharedStats::new(),
        subscriptions.clone(),
    );
    (path_str, dir)
}

fn connect(path: &str) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match UnixStream::connect(path) {
            Ok(s) => return s,
            Err(e) => {
                assert!(Instant::now() < deadline, "connect {path}: {e}");
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

// ---- 1. subscribe-events pushes a JSON-RPC "event" notification ------------

#[test]
fn subscribe_events_receives_event_notification() {
    // Given a listener with one already-broadcast snapshot to replay
    let config = DynamicConfig::new();
    let subscriptions = SubscriptionManager::new();
    subscriptions.broadcast(&build_telemetry_json(1_749_556_546_000, &[sample_conn()]));
    let (path, _dir) = spawn_listener_with(&config, &subscriptions);

    // When a client subscribes
    let mut client = connect(&path);
    writeln!(client, "{SUBSCRIBE_FRAME}").expect("write subscribe frame");
    client.flush().expect("flush subscribe frame");
    let mut reader = BufReader::new(client.try_clone().expect("clone stream"));
    let mut line = String::new();
    reader.read_line(&mut line).expect("read event line");

    // Then the next line is a notification with method "event" and the full
    // ADR-001 snapshot as params (no `id` — notifications omit it).
    let event: Value = serde_json::from_str(line.trim_end()).expect("event is valid JSON");
    assert_eq!(event["jsonrpc"], Value::from("2.0"));
    assert_eq!(event["method"], Value::from("event"));
    assert!(event.get("id").is_none(), "a notification must omit id");
    let params = &event["params"];
    assert_eq!(params["schema_version"], Value::from(1));
    assert!(params.get("last_updated_ms").is_some(), "snapshot needs ms");
    assert!(
        params["connections"].is_array(),
        "snapshot needs connections[]"
    );
}

// ---- 2. the last-known snapshot is replayed immediately on subscribe -------

#[test]
fn subscribe_events_replays_last_known_snapshot() {
    // Given a manager with a previously broadcast snapshot
    let subscriptions = SubscriptionManager::new();
    let snapshot = build_telemetry_json(42, &[sample_conn()]);
    subscriptions.broadcast(&snapshot);

    // When a new subscriber joins
    let rx = subscriptions.subscribe();

    // Then it receives that snapshot immediately, verbatim (not blank)
    let frame = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("replayed frame within timeout");
    let event: Value = serde_json::from_str(&frame).expect("valid JSON");
    assert_eq!(event["method"], Value::from("event"));
    let expected: Value = serde_json::from_str(&snapshot).expect("valid snapshot JSON");
    assert_eq!(event["params"], expected);
    assert_ne!(event["params"], Value::Null, "replay must not be blank");
}

// ---- 3. a slow subscriber never blocks the broadcast (hot path) ------------

#[test]
fn slow_subscriber_does_not_block_broadcast() {
    // Given a subscriber that never drains its channel
    let subscriptions = SubscriptionManager::new();
    let _rx = subscriptions.subscribe();

    // When many snapshots are broadcast rapidly
    let start = Instant::now();
    for i in 0..10_000u64 {
        subscriptions.broadcast(&build_telemetry_json(i, &[]));
    }

    // Then broadcast returns promptly (capacity-1 channel drops, never blocks)
    // and the lagging subscriber stays registered.
    assert!(
        start.elapsed() < Duration::from_secs(5),
        "broadcast blocked on a slow subscriber"
    );
    assert_eq!(subscriptions.subscriber_count(), 1);
}

// ---- 4. the stats file keeps writing during a subscription (dual-publish) --

#[test]
fn file_sink_still_writes_during_subscription() {
    // Given an active subscriber and a stats-file path
    let dir = tempfile::tempdir().expect("create tempdir");
    let path = dir.path().join("stats.json");
    let subscriptions = SubscriptionManager::new();
    let rx = subscriptions.subscribe();

    // When one snapshot drives both sinks (the real tick builds it once)
    let snapshot = build_telemetry_json(7, &[sample_conn()]);
    write_atomic(&path, &snapshot).expect("file sink write");
    subscriptions.broadcast(&snapshot);

    // Then the file holds the snapshot verbatim and the subscriber's event
    // carries the same snapshot in params (file AND subscription).
    let file_content = std::fs::read_to_string(&path).expect("read stats file");
    assert_eq!(file_content, snapshot);
    let frame = rx
        .recv_timeout(Duration::from_secs(1))
        .expect("event frame within timeout");
    let event: Value = serde_json::from_str(&frame).expect("valid JSON");
    let file_value: Value = serde_json::from_str(&file_content).expect("valid file JSON");
    assert_eq!(event["params"], file_value);
}

// ---- 5. a disconnected subscriber is pruned on the next broadcast ----------

#[test]
fn subscriber_cleanup_on_disconnect() {
    // Given a registered subscriber
    let subscriptions = SubscriptionManager::new();
    let rx = subscriptions.subscribe();
    assert_eq!(subscriptions.subscriber_count(), 1);

    // When it disconnects (the receiver is dropped) and a broadcast runs
    drop(rx);
    subscriptions.broadcast(&build_telemetry_json(1, &[]));

    // Then the dead subscriber is removed (no channel accumulation)
    assert_eq!(subscriptions.subscriber_count(), 0);
}
