//! Loom model test for `SubscriptionManager` (`src/subscription.rs`).
//!
//! `SubscriptionManager` fans telemetry frames out to control-socket
//! subscribers behind a `std::sync::Mutex<Inner>` guarding `{ last_frame,
//! subscribers }`, where each subscriber owns a capacity-1
//! `std::sync::mpsc::sync_channel`. The telemetry tick (hot path) calls
//! `broadcast` — non-blocking `try_send`, drop-newest-on-full, prune-on-
//! disconnect — while the Unix control thread concurrently calls `subscribe`
//! and later drops the receiver.
//!
//! loom swaps std's `Mutex`/`Arc`/channel for model-checked equivalents and
//! exhaustively explores every thread interleaving; std types are NOT
//! instrumented, so this file re-models the exact algorithm of
//! `src/subscription.rs` on `loom::sync` primitives (a faithful capacity-1
//! channel plus the same `retain`/replay logic) and asserts four invariants
//! under ALL interleavings:
//!   1. no deadlock (loom fails the model if any schedule deadlocks);
//!   2. `last_frame` after a broadcast equals that broadcast's frame;
//!   3. a subscriber registered before a broadcast and not full receives that
//!      frame or a strictly newer one (no lost wakeup to a live, non-full
//!      subscriber);
//!   4. a disconnected subscriber is pruned on the next broadcast (no panic, no
//!      unbounded growth).
//!
//! Not part of the default gate. Run with:
//!   RUSTFLAGS="--cfg loom" cargo test --features test-internals --test subscription_loom
#![cfg(loom)]

use loom::sync::{Arc, Mutex};
use loom::thread;

type Frame = u64;

struct Chan {
    state: Mutex<ChanState>,
}

struct ChanState {
    slot: Option<Frame>,
    receiver_alive: bool,
}

enum TrySendErr {
    Full,
    Disconnected,
}

struct Sender {
    chan: Arc<Chan>,
}

impl Sender {
    fn try_send(&self, frame: Frame) -> Result<(), TrySendErr> {
        let mut st = self.chan.state.lock().unwrap();
        if !st.receiver_alive {
            return Err(TrySendErr::Disconnected);
        }
        if st.slot.is_some() {
            return Err(TrySendErr::Full);
        }
        st.slot = Some(frame);
        Ok(())
    }
}

struct Receiver {
    chan: Arc<Chan>,
}

impl Receiver {
    fn try_recv(&self) -> Option<Frame> {
        self.chan.state.lock().unwrap().slot.take()
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        self.chan.state.lock().unwrap().receiver_alive = false;
    }
}

fn channel() -> (Sender, Receiver) {
    let chan = Arc::new(Chan {
        state: Mutex::new(ChanState {
            slot: None,
            receiver_alive: true,
        }),
    });
    (
        Sender {
            chan: Arc::clone(&chan),
        },
        Receiver { chan },
    )
}

#[derive(Clone)]
struct Manager {
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    last_frame: Option<Frame>,
    subscribers: Vec<Sender>,
}

impl Manager {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                last_frame: None,
                subscribers: Vec::new(),
            })),
        }
    }

    fn broadcast(&self, frame: Frame) {
        let mut inner = self.inner.lock().unwrap();
        inner.last_frame = Some(frame);
        inner
            .subscribers
            .retain(|tx| !matches!(tx.try_send(frame), Err(TrySendErr::Disconnected)));
    }

    fn subscribe(&self) -> Receiver {
        let (tx, rx) = channel();
        let mut inner = self.inner.lock().unwrap();
        if let Some(frame) = inner.last_frame {
            let _ = tx.try_send(frame);
        }
        inner.subscribers.push(tx);
        rx
    }

    fn last_frame(&self) -> Option<Frame> {
        self.inner.lock().unwrap().last_frame
    }

    fn subscriber_count(&self) -> usize {
        self.inner.lock().unwrap().subscribers.len()
    }
}

const F1: Frame = 1;
const F2: Frame = 2;

#[test]
fn broadcast_concurrent_with_subscribe_drop_keeps_invariants() {
    loom::model(|| {
        let mgr = Manager::new();

        // Given a subscriber registered before any broadcast and never drained
        // (the no-lost-wakeup target for invariant 3; its slot starts empty).
        let pre = mgr.subscribe();

        // When the telemetry tick broadcasts while a control client races a
        // subscribe + hang-up. The tick is the sole broadcaster, so it can
        // assert invariant 2 right after its own broadcast.
        let tele = {
            let mgr = mgr.clone();
            thread::spawn(move || {
                mgr.broadcast(F1);
                assert_eq!(mgr.last_frame(), Some(F1), "last_frame must equal F1");
            })
        };
        let ctrl = {
            let mgr = mgr.clone();
            thread::spawn(move || {
                let rx = mgr.subscribe();
                drop(rx);
            })
        };

        tele.join().unwrap();
        ctrl.join().unwrap();

        // Then `pre` (registered before F1, not full at F1) holds F1 or a
        // strictly newer frame — the broadcast was never lost to it.
        let got = pre
            .try_recv()
            .expect("pre-registered subscriber lost its frame");
        assert!(
            got >= F1,
            "subscriber received a stale/garbage frame: {got}"
        );
    });
}

#[test]
fn disconnected_subscriber_is_pruned_on_next_broadcast() {
    loom::model(|| {
        let mgr = Manager::new();

        // Given a control subscriber that connects then hangs up, racing a
        // telemetry broadcast (exercises prune-mid-broadcast and prune-after).
        let ctrl = {
            let mgr = mgr.clone();
            thread::spawn(move || {
                let rx = mgr.subscribe();
                drop(rx);
            })
        };
        let tele = {
            let mgr = mgr.clone();
            thread::spawn(move || {
                mgr.broadcast(F1);
            })
        };

        ctrl.join().unwrap();
        tele.join().unwrap();

        // When the next broadcast runs, the hung-up subscriber must be pruned
        // under every interleaving: no panic, and the live set never grows.
        mgr.broadcast(F2);
        assert_eq!(
            mgr.subscriber_count(),
            0,
            "disconnected subscriber not pruned"
        );
        assert_eq!(mgr.last_frame(), Some(F2));
    });
}
