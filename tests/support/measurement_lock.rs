use std::fs::{File, OpenOptions};
use std::sync::{Mutex, MutexGuard, OnceLock};

pub struct MeasurementGuard {
    _host: File,
    _local: MutexGuard<'static, ()>,
}

// This test harness uses the pinned nightly; std file locking needs Rust 1.89, not new FFI.
#[clippy::msrv = "1.89"]
pub fn measurement_lock() -> MeasurementGuard {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let local = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // The fixed inode is intentional: separate integration binaries/worktrees share the host.
    let host = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(std::env::temp_dir().join("srtla-measurement.lock"))
        .expect("open measurement lock");
    host.lock().expect("lock measurement host");
    MeasurementGuard {
        _host: host,
        _local: local,
    }
}

#[test]
#[clippy::msrv = "1.89"]
fn measurement_lock_excludes_an_independent_open_file() {
    // Given the existing host measurement guard.
    let _guard = measurement_lock();
    let other = File::open(std::env::temp_dir().join("srtla-measurement.lock")).unwrap();
    // When a separate file description tries to lock, then the kernel excludes it.
    assert!(matches!(
        other.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
}
