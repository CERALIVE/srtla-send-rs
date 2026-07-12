//! SRTLA Sender Library
//!
//! This library provides functionality for SRTLA (SRT transport proxy with link
//! aggregation) sender implementation. It includes protocol handling,
//! connection management, and dynamic configuration.

// Use mimalloc as the global allocator for tests (non-Windows only).
// Gated on the `mimalloc-allocator` feature so --no-default-features builds the
// test binary against the system allocator (docs/notes/mimalloc-decision.md).
#[cfg(all(not(windows), test, feature = "mimalloc-allocator"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(not(loom))]
pub mod config;
#[cfg(not(loom))]
pub mod connection;
#[cfg(not(loom))]
pub mod ewma;
#[cfg(all(unix, not(loom)))]
pub mod jsonrpc;
#[cfg(not(loom))]
pub mod kalman;
#[cfg(not(loom))]
pub mod mode;
#[cfg(not(loom))]
pub mod protocol;
#[cfg(not(loom))]
pub mod registration;
#[cfg(not(loom))]
pub mod sender;
#[cfg(not(loom))]
pub mod stats;
pub mod subscription;
#[cfg(not(loom))]
pub mod telemetry_file;
#[cfg(not(loom))]
pub mod utils;

// Test helpers module - available when test-internals feature is enabled
#[cfg(all(any(test, feature = "test-internals"), not(loom)))]
pub mod test_helpers;

#[cfg(all(test, not(loom)))]
pub mod tests;

// Re-export commonly used items
#[cfg(not(loom))]
pub use config::{ConfigSnapshot, DynamicConfig};
#[cfg(not(loom))]
pub use connection::SrtlaConnection;
#[cfg(not(loom))]
pub use mode::SchedulingMode;
#[cfg(not(loom))]
pub use protocol::*;
#[cfg(not(loom))]
pub use registration::SrtlaRegistrationManager;
#[cfg(not(loom))]
pub use utils::now_ms;
