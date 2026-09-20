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

// `--cfg loom` compiles this crate down to `subscriptions` alone. Loom replaces
// that module's synchronization primitives with its own, which changes the
// channel type `control` hands the hub, and Loom cannot model tokio's runtime
// anyway — so the rest of the tree is excluded from the lane rather than
// contorted to fit it. See `tests/subscription_loom.rs`.
#[cfg(not(loom))]
pub mod bind_map;
// The `--capabilities-json` pre-spawn probe document, also served at runtime by
// the JSON-RPC `get_capabilities` method.
#[cfg(not(loom))]
pub mod capabilities;
#[cfg(not(loom))]
pub mod config;
#[cfg(not(loom))]
pub mod control;
#[cfg(not(loom))]
pub mod control_socket;
#[cfg(not(loom))]
pub mod metrics;
// Uplink socket I/O (shell): batched UDP socket + egress binders.
#[cfg(not(loom))]
pub mod net;
#[cfg(not(loom))]
pub mod priority_listener;
#[cfg(not(loom))]
pub mod sender;
#[cfg(not(loom))]
pub mod stats;
pub mod subscriptions;
// ADR-001 telemetry: the document model + units, and the opt-in `--stats-file`
// atomic publish mechanics that carry it.
#[cfg(not(loom))]
pub mod telemetry_doc;
#[cfg(not(loom))]
pub mod telemetry_file;
#[cfg(not(loom))]
pub mod toml_config;
#[cfg(not(loom))]
pub mod version;

// Test helpers module - available when test-internals feature is enabled
#[cfg(all(any(test, feature = "test-internals"), not(loom)))]
pub mod test_helpers;

#[cfg(all(test, not(loom)))]
pub mod tests;

#[cfg(not(loom))]
pub use config::{ConfigSnapshot, DynamicConfig};
