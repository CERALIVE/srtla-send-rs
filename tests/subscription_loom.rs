//! Loom schedule exploration over the PRODUCTION subscription hub.
//!
//! Every assertion below is made against `SubscriptionHub`'s own
//! `subscribe`/`unsubscribe`/`publish` bodies (`src/subscriptions.rs`): the hub
//! is constructed, raced, and observed only through its public API. The
//! algorithm is deliberately NOT mirrored here — a model that re-implements the
//! hub stays green while the hub itself drifts, which is exactly the failure
//! this target exists to rule out. Under `--cfg loom` the hub swaps its
//! `tokio::sync::Mutex` and its capacity-one subscriber channel for Loom's
//! (`subscriptions::loom_sync`); nothing else about it changes.
//!
//! Two invariants:
//!
//! 1. A subscriber that registers concurrently with a publish is never lost:
//!    it observes that frame live, or it observes the next one. Upstream's hub
//!    keeps no last-broadcast frame, so "the next publish" is where the legacy
//!    manager's last-frame replay lands here.
//! 2. A subscriber whose receiver has hung up is pruned by the next publish.
//!
//! `subscribe`/`unsubscribe`/`len` are unix-gated along with the control socket
//! that owns them, so this target is too.
#![cfg(all(loom, unix))]

use loom::thread;
use serde_json::{Value, json};
use srtla_send::subscriptions::SubscriptionHub;
use srtla_send::subscriptions::loom_sync::{block_on, mpsc};

const TOPIC: &str = "stats";

/// Take whatever the hub pushed into a subscriber's capacity-one slot, parsed.
fn pushed(rx: &mpsc::Receiver<String>) -> Option<Value> {
    rx.try_recv()
        .map(|line| serde_json::from_str(&line).expect("the hub pushes JSON-RPC lines"))
}

/// The `frame` marker the test put in the published payload, as the subscriber
/// received it through the hub's `<topic>.update` envelope.
fn frame_marker(event: &Value, subscription_id: &str) -> u64 {
    assert_eq!(event["method"], format!("{TOPIC}.update"));
    assert_eq!(event["params"]["subscription_id"], subscription_id);
    event["params"]["data"]["frame"]
        .as_u64()
        .expect("frame marker survives the envelope")
}

#[test]
fn publish_concurrent_with_subscribe_loses_neither_the_subscriber_nor_the_event() {
    loom::model(|| {
        // Given a hub with one subscriber already registered.
        let hub = SubscriptionHub::new();
        let (settled_tx, settled_rx) = mpsc::channel::<String>(1);
        let settled_id = block_on(hub.subscribe(TOPIC, settled_tx));

        // When a second subscriber registers while a publish is in flight.
        let (racing_tx, racing_rx) = mpsc::channel::<String>(1);
        let subscriber = {
            let hub = hub.clone();
            thread::spawn(move || block_on(hub.subscribe(TOPIC, racing_tx)))
        };
        let publisher = {
            let hub = hub.clone();
            thread::spawn(move || block_on(hub.publish(TOPIC, json!({"frame": 1}))))
        };
        let racing_id = subscriber.join().expect("subscriber thread completes");
        publisher.join().expect("publisher thread completes");

        // Then the already-settled subscriber received the frame, on every
        // schedule -- its registration did not race anything.
        let settled = pushed(&settled_rx).expect("a settled subscriber never misses a publish");
        assert_eq!(frame_marker(&settled, &settled_id), 1);

        // And both subscriptions are registered, whichever order they landed in.
        assert_eq!(block_on(hub.len()), 2);

        // And the racing subscriber either saw that frame live, or sees the next
        // one -- it is never dropped on the floor. (Draining its capacity-one
        // slot above is what lets the next frame in; a full slot is a documented
        // drop, not a loss of the subscription.)
        let live = pushed(&racing_rx).map(|event| frame_marker(&event, &racing_id));
        assert!(
            matches!(live, None | Some(1)),
            "a racing subscriber may only ever observe the in-flight frame, got {live:?}"
        );

        block_on(hub.publish(TOPIC, json!({"frame": 2})));
        let next = pushed(&racing_rx).expect("a registered subscriber receives the next publish");
        assert_eq!(frame_marker(&next, &racing_id), 2);
    });
}

#[test]
fn a_hung_up_subscriber_is_pruned_by_the_next_publish() {
    loom::model(|| {
        // Given one subscriber that stays, and one that registers and hangs up
        // concurrently with a publish.
        let hub = SubscriptionHub::new();
        let (survivor_tx, survivor_rx) = mpsc::channel::<String>(1);
        let survivor_id = block_on(hub.subscribe(TOPIC, survivor_tx));

        let (hangup_tx, hangup_rx) = mpsc::channel::<String>(1);
        let hangup = {
            let hub = hub.clone();
            thread::spawn(move || {
                block_on(hub.subscribe(TOPIC, hangup_tx));
                drop(hangup_rx);
            })
        };
        let publisher = {
            let hub = hub.clone();
            thread::spawn(move || block_on(hub.publish(TOPIC, json!({"frame": 1}))))
        };
        hangup.join().expect("hang-up thread completes");
        publisher.join().expect("publisher thread completes");

        // When the next telemetry tick publishes (the survivor's capacity-one
        // slot drained first, so a full slot cannot stand in for a prune).
        let _ = pushed(&survivor_rx);
        block_on(hub.publish(TOPIC, json!({"frame": 2})));

        // Then the dead subscription is gone and the live one still receives.
        assert_eq!(block_on(hub.len()), 1);
        let delivered = pushed(&survivor_rx).expect("the surviving subscriber still receives");
        assert_eq!(frame_marker(&delivered, &survivor_id), 2);
    });
}
