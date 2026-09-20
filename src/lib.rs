//! SRTLA Sender Library
//!
//! This library provides functionality for SRTLA (SRT transport proxy with link
//! aggregation) sender implementation. It includes protocol handling,
//! connection management, and dynamic configuration.

// Use mimalloc as the global allocator for tests (non-Windows only). Excluded
// under miri: the batch_recv miri CI lane interprets the test binary, and miri
// cannot execute mimalloc's C FFI, so those runs fall back to miri's own
// allocator instead. Gated on the default-on `mimalloc` feature so
// --no-default-features also builds the test binary against the system
// allocator (docs/notes/mimalloc-decision.md).
#[cfg(all(not(windows), test, not(miri), feature = "mimalloc"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub mod bind_map;
pub mod config;
pub mod control;
pub mod control_socket;
pub mod metrics;
// Uplink socket I/O (shell): batched UDP socket + egress binders.
pub mod net;
pub mod priority_listener;
pub mod sender;
pub mod stats;
pub mod subscriptions;
// ADR-001 telemetry: the document model + units, and the opt-in `--stats-file`
// atomic publish mechanics that carry it.
pub mod telemetry_doc;
pub mod telemetry_file;
pub mod toml_config;
pub mod version;

// Test helpers module - available when test-internals feature is enabled
#[cfg(any(test, feature = "test-internals"))]
pub mod test_helpers;

#[cfg(test)]
pub mod tests;

pub use config::{ConfigSnapshot, DynamicConfig};
