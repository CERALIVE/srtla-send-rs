#![cfg(loom)]

use loom::thread;
use srtla_send::subscription::SubscriptionManager;

const SNAPSHOT_ONE: &str = r#"{"schema_version":1,"last_updated_ms":1,"connections":[]}"#;
const SNAPSHOT_TWO: &str = r#"{"schema_version":1,"last_updated_ms":2,"connections":[]}"#;
const EVENT_ONE: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"schema_version":1,"last_updated_ms":1,"connections":[]}}"#;
const EVENT_TWO: &str = r#"{"jsonrpc":"2.0","method":"event","params":{"schema_version":1,"last_updated_ms":2,"connections":[]}}"#;

#[test]
fn broadcast_concurrent_with_subscribe_keeps_delivery_and_replay_invariants() {
    loom::model(|| {
        // Given one live subscriber and a fresh production manager.
        let manager = SubscriptionManager::new();
        let pre_registered = manager.subscribe();
        let subscriber = {
            let manager = manager.clone();
            thread::spawn(move || manager.subscribe())
        };
        let broadcaster = {
            let manager = manager.clone();
            thread::spawn(move || manager.broadcast(SNAPSHOT_ONE))
        };

        // When subscribe and broadcast race under every Loom schedule.
        let concurrent = subscriber.join().expect("subscriber thread completes");
        broadcaster.join().expect("broadcaster thread completes");

        // Then live, concurrent, and post-broadcast subscribers all observe F1.
        assert_eq!(pre_registered.try_recv(), Ok(EVENT_ONE.to_owned()));
        assert_eq!(concurrent.try_recv(), Ok(EVENT_ONE.to_owned()));
        assert_eq!(manager.subscribe().try_recv(), Ok(EVENT_ONE.to_owned()));
    });
}

#[test]
fn disconnected_subscriber_is_pruned_on_next_broadcast() {
    loom::model(|| {
        // Given a subscriber hang-up racing the first production broadcast.
        let manager = SubscriptionManager::new();
        let subscriber = {
            let manager = manager.clone();
            thread::spawn(move || drop(manager.subscribe()))
        };
        let broadcaster = {
            let manager = manager.clone();
            thread::spawn(move || manager.broadcast(SNAPSHOT_ONE))
        };

        // When both operations finish and the next telemetry tick broadcasts.
        subscriber.join().expect("subscriber thread completes");
        broadcaster.join().expect("broadcaster thread completes");
        manager.broadcast(SNAPSHOT_TWO);

        // Then the dead sender is pruned and the newest frame remains replayable.
        assert_eq!(manager.subscriber_count(), 0);
        assert_eq!(manager.subscribe().try_recv(), Ok(EVENT_TWO.to_owned()));
    });
}
