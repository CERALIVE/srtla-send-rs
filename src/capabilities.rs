//! The `--capabilities-json` pre-spawn probe (ADR-003 §7).
//!
//! A caller must decide **before spawning a stream** whether the installed
//! binary understands `--bind-map`. Passing an unknown flag to an old binary
//! makes it exit non-zero with a usage error, which at spawn time is a failed
//! stream rather than a graceful downgrade.
//!
//! The load-bearing half of this contract lives in the caller: **non-zero exit,
//! unparseable output, or a timeout means NO SUPPORT** — fall back to the legacy
//! spawn and never pass `--bind-map`. The shipped 3.2.0 binary answers this flag
//! with `error: unexpected argument` and exit 2, which is exactly that signal.

use serde::Serialize;

use crate::bind_map::BIND_MAP_SCHEMA_VERSION;

/// Version of the **capability document** itself, independent of the sidecar's.
pub const CAPABILITIES_SCHEMA_VERSION: u32 = 1;

/// What this build can do. An open map by contract: consumers ignore unknown
/// keys, so a later build may add entries without breaking an older caller.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Capabilities {
    /// Whether `--bind-map` can actually steer egress on this platform.
    pub bind_map: bool,
    /// The sidecar `schema_version` this build reads.
    pub bind_map_schema_version: u32,
    /// Always true — self-describing, so a consumer that got a document knows it
    /// can ask again.
    pub capabilities_json: bool,
    pub dry_run: bool,
    pub stats_file: bool,
    pub control_socket: bool,
}

/// The full document written to stdout.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CapabilityDocument {
    pub schema_version: u32,
    pub binary: &'static str,
    pub version: &'static str,
    pub capabilities: Capabilities,
}

/// Build the capability document for this binary.
#[must_use]
pub fn capability_document() -> CapabilityDocument {
    CapabilityDocument {
        schema_version: CAPABILITIES_SCHEMA_VERSION,
        binary: "srtla_send",
        version: env!("CARGO_PKG_VERSION"),
        capabilities: Capabilities {
            // Egress pinning is `SO_BINDTODEVICE`, which is Linux-only. A build
            // that cannot honor the map must not claim it can.
            bind_map: cfg!(target_os = "linux"),
            bind_map_schema_version: BIND_MAP_SCHEMA_VERSION,
            capabilities_json: true,
            dry_run: true,
            stats_file: true,
            control_socket: cfg!(unix),
        },
    }
}

/// The document as the single line of JSON the probe prints.
///
/// Serialization of a fixed struct cannot fail, so this returns a plain `String`
/// rather than making every caller handle an impossible error.
#[must_use]
pub fn capability_json() -> String {
    serde_json::to_string(&capability_document())
        .unwrap_or_else(|e| unreachable!("capability document is always serializable: {e}"))
}
