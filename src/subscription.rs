//! Telemetry event-subscription fan-out for the JSON-RPC control socket.
//!
//! A client sends `{"jsonrpc":"2.0","method":"subscribe-events"}` on the Unix
//! `--control-socket`; the sender then streams JSON-RPC `"event"` notifications
//! carrying the full ADR-001 telemetry snapshot at the `--stats-file-interval`
//! cadence — the *same* snapshot the `--stats-file` sink writes (dual-publish).
//!
//! The manager owns two things behind one lock: the list of live subscriber
//! channels and the last broadcast frame (so a fresh subscriber gets current
//! state immediately, without waiting for the next tick). Each subscriber has a
//! capacity-1 bounded channel: a slow consumer drops stale frames rather than
//! ever back-pressuring the telemetry tick. The tick (the hot path) only ever
//! calls [`SubscriptionManager::broadcast`], which is non-blocking by
//! construction — `try_send` never waits.
//!
//! The module is cross-platform so the telemetry tick can call `broadcast`
//! unconditionally; subscribers can only ever be registered through the
//! Unix-only control socket, so [`SubscriptionManager::subscribe`] is dead on
//! non-Unix targets (where it is allow-listed) but still compiled so the
//! last-frame replay stays wired into the shared state.

use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Arc, Mutex};

use tracing::warn;

/// Per-subscriber channel capacity. Capacity 1 keeps only the freshest pending
/// frame: if the consumer has not drained the previous one, the next broadcast
/// is dropped (logged) instead of queuing unbounded or blocking the tick.
const SUBSCRIBER_CHANNEL_CAPACITY: usize = 1;

/// Shared telemetry-event fan-out: live subscriber channels plus the last
/// broadcast frame for immediate replay on subscribe.
#[derive(Clone, Default)]
pub struct SubscriptionManager {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    /// The most recent JSON-RPC `"event"` notification frame, replayed verbatim
    /// to any subscriber that joins before the next broadcast.
    last_frame: Option<String>,
    /// Send halves of every live subscriber's capacity-1 channel.
    subscribers: Vec<SyncSender<String>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Broadcast one ADR-001 snapshot to every live subscriber and record it for
    /// replay. `snapshot_json` is the pre-serialized telemetry document from
    /// `build_telemetry_json` — the byte-identical document the `--stats-file`
    /// sink publishes — wrapped here as the `params` of an `"event"`
    /// notification.
    ///
    /// Never blocks: a subscriber whose 1-slot channel is full drops this frame
    /// (logged at WARN) and stays registered; a disconnected subscriber is
    /// pruned. This is the only method the telemetry tick (hot path) calls.
    pub fn broadcast(&self, snapshot_json: &str) {
        let frame = build_event_frame(snapshot_json);
        let mut inner = self.lock();
        inner.last_frame = Some(frame.clone());
        inner
            .subscribers
            .retain(|tx| match tx.try_send(frame.clone()) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) => {
                    warn!("stats subscription: subscriber lagging, dropping event frame");
                    true
                }
                Err(TrySendError::Disconnected(_)) => false,
            });
    }

    /// Register a new subscriber, returning the receiver the socket handler
    /// drains. The last-known snapshot (if any) is enqueued immediately so a
    /// fresh subscriber sees current state without waiting for the next tick.
    ///
    /// Only ever called from the Unix control-socket handler; compiled (but
    /// unused) elsewhere so the last-frame replay path stays linked.
    #[cfg_attr(not(unix), allow(dead_code))]
    pub fn subscribe(&self) -> Receiver<String> {
        let (tx, rx) = sync_channel(SUBSCRIBER_CHANNEL_CAPACITY);
        let mut inner = self.lock();
        if let Some(frame) = inner.last_frame.as_ref() {
            // Replay last-known state; the channel is empty so this cannot fail
            // for a Full reason, and the receiver is live (we just made it).
            let _ = tx.try_send(frame.clone());
        }
        inner.subscribers.push(tx);
        rx
    }

    /// Lock the shared state, recovering the guard even if a previous holder
    /// panicked (the data is plain owned values, so a poisoned lock is safe to
    /// keep using).
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Number of live subscribers. Test-only assertion hook used by the lib
    /// test tree; `dead_code`-allowed for the binary's own test build, which
    /// recompiles this shared module without those tests.
    #[cfg(all(test, unix))]
    #[allow(dead_code)]
    pub fn subscriber_count(&self) -> usize {
        self.lock().subscribers.len()
    }
}

/// Wrap a pre-serialized ADR-001 snapshot document as a JSON-RPC `"event"`
/// notification line. `snapshot_json` is a complete JSON object (built by
/// `build_telemetry_json`), embedded as the `params` value without a re-parse
/// or re-serialize round-trip.
fn build_event_frame(snapshot_json: &str) -> String {
    format!(r#"{{"jsonrpc":"2.0","method":"event","params":{snapshot_json}}}"#)
}
