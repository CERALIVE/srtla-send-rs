//! Scoped monotonic-ms override for current-thread tests only.

use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

thread_local! {
    static NOW: Cell<Option<u64>> = const { Cell::new(None) };
}

pub(crate) fn current() -> Option<u64> {
    NOW.get()
}

/// !Send: an async test holding this guard must use Tokio's current-thread runtime.
pub(crate) struct TestClock(PhantomData<Rc<()>>);

impl TestClock {
    pub(crate) fn new(now: u64) -> Self {
        assert!(NOW.get().is_none(), "nested test clocks are unsupported");
        NOW.set(Some(now));
        Self(PhantomData)
    }

    pub(crate) fn set(&self, now: u64) {
        assert!(NOW.get().is_some_and(|previous| previous <= now));
        NOW.set(Some(now));
    }
}

impl Drop for TestClock {
    fn drop(&mut self) {
        NOW.set(None);
    }
}

#[test]
fn clock_is_scoped_and_thread_isolated() {
    // Given a scoped clock on this test thread.
    let clock = TestClock::new(10_000);
    // When advancing that clock.
    clock.set(14_000);
    // Then production clock reads use it, but another thread cannot see it.
    assert_eq!(crate::utils::now_ms(), 14_000);
    assert_eq!(std::thread::spawn(current).join().unwrap(), None);
}
