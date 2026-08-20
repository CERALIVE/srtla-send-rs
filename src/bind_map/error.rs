//! Bind-map rejection classes and the seven telemetry-facing reasons they map to.
//!
//! Two levels on purpose. [`BindMapError`] is fine-grained so a log line tells a
//! writer exactly which row and field to fix; [`DegradedReason`] is the small,
//! frozen set that leaves the process as typed telemetry. The mapping between
//! them is total and exhaustive — see [`BindMapError::reason`].

use std::fmt;
use std::net::IpAddr;

/// The operator-visible reason a configured bind-map is not in force.
///
/// These seven tokens are the wire contract (ADR-003 §6.4): they are echoed
/// verbatim into telemetry so a UI renders the sender's actual operating mode
/// from typed data instead of inferring it from log text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DegradedReason {
    /// The sidecar's digest did not name the `BIND_IPS_FILE` content on disk.
    HashMismatch,
    /// Bad JSON, bad header, bad row, bad correspondence, or a stale generation.
    Malformed,
    /// A row names an interface that does not exist on this host.
    UnknownIface,
    /// A hash mismatch persisted past the bounded retry budget.
    RetryExhausted,
    /// The sidecar path does not exist.
    MissingFile,
    /// I/O error, symlink, non-regular file, or unsafe permissions.
    Unreadable,
    /// A `schema_version` this build does not implement.
    Unsupported,
}

impl DegradedReason {
    /// The exact snake_case token published in telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HashMismatch => "hash_mismatch",
            Self::Malformed => "malformed",
            Self::UnknownIface => "unknown_iface",
            Self::RetryExhausted => "retry_exhausted",
            Self::MissingFile => "missing_file",
            Self::Unreadable => "unreadable",
            Self::Unsupported => "unsupported",
        }
    }
}

impl fmt::Display for DegradedReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Every way a bind-map pair can be refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindMapError {
    /// The sidecar path does not exist.
    MissingFile { path: String },
    /// The sidecar could not be read safely (I/O, symlink, mode, non-regular).
    Unreadable { path: String, detail: String },
    /// The sidecar is not parseable JSON, or is missing a required field.
    MalformedJson(String),
    /// A `schema_version` this build does not implement.
    UnsupportedSchemaVersion { found: u32 },
    /// A header field is present but outside its defined domain.
    InvalidHeader { field: &'static str, detail: String },
    /// A row field is present but outside its defined domain.
    InvalidRow {
        index: usize,
        field: &'static str,
        detail: String,
    },
    /// A row names an interface that does not exist on this host.
    UnknownIface { index: usize, iface: String },
    /// Two rows name the same `(ip, iface)` socket key.
    DuplicatePair {
        index: usize,
        ip: IpAddr,
        iface: String,
    },
    /// Two rows claim the same identity.
    DuplicateLinkId { index: usize, link_id: String },
    /// The sidecar does not describe the same number of uplinks as the IP list.
    RowCountMismatch { sidecar: usize, ips_file: usize },
    /// The Nth row does not describe the Nth accepted IP-list entry.
    RowIpMismatch {
        index: usize,
        sidecar: IpAddr,
        ips_file: IpAddr,
    },
    /// The digest does not name the IP-list content that was read alongside it.
    HashMismatch { sidecar: String, computed: String },
    /// The mapping changed without the generation moving, so it cannot be ordered.
    StaleGeneration { generation: u64 },
    /// A hash mismatch survived the whole retry budget.
    RetryExhausted { attempts: u32 },
}

impl BindMapError {
    /// The single telemetry reason this rejection reports as.
    ///
    /// Total by construction: adding a variant without classifying it fails to
    /// compile.
    #[must_use]
    pub const fn reason(&self) -> DegradedReason {
        match self {
            Self::MissingFile { .. } => DegradedReason::MissingFile,
            Self::Unreadable { .. } => DegradedReason::Unreadable,
            Self::UnsupportedSchemaVersion { .. } => DegradedReason::Unsupported,
            Self::HashMismatch { .. } => DegradedReason::HashMismatch,
            Self::UnknownIface { .. } => DegradedReason::UnknownIface,
            Self::RetryExhausted { .. } => DegradedReason::RetryExhausted,
            Self::MalformedJson(_)
            | Self::InvalidHeader { .. }
            | Self::InvalidRow { .. }
            | Self::DuplicatePair { .. }
            | Self::DuplicateLinkId { .. }
            | Self::RowCountMismatch { .. }
            | Self::RowIpMismatch { .. }
            | Self::StaleGeneration { .. } => DegradedReason::Malformed,
        }
    }
}

impl fmt::Display for BindMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFile { path } => write!(f, "bind-map sidecar not found: {path}"),
            Self::Unreadable { path, detail } => {
                write!(f, "bind-map sidecar {path} is unreadable: {detail}")
            }
            Self::MalformedJson(detail) => {
                write!(f, "bind-map sidecar is not valid JSON: {detail}")
            }
            Self::UnsupportedSchemaVersion { found } => write!(
                f,
                "bind-map schema_version {found} is not supported by this build"
            ),
            Self::InvalidHeader { field, detail } => {
                write!(f, "bind-map header field `{field}` is invalid: {detail}")
            }
            Self::InvalidRow {
                index,
                field,
                detail,
            } => write!(
                f,
                "bind-map row {index} field `{field}` is invalid: {detail}"
            ),
            Self::UnknownIface { index, iface } => write!(
                f,
                "bind-map row {index} names interface `{iface}`, which does not exist on this host"
            ),
            Self::DuplicatePair { index, ip, iface } => write!(
                f,
                "bind-map row {index} repeats the socket key ({ip}, {iface})"
            ),
            Self::DuplicateLinkId { index, link_id } => {
                write!(f, "bind-map row {index} repeats link_id `{link_id}`")
            }
            Self::RowCountMismatch { sidecar, ips_file } => write!(
                f,
                "bind-map describes {sidecar} link(s) but the ips file has {ips_file}"
            ),
            Self::RowIpMismatch {
                index,
                sidecar,
                ips_file,
            } => write!(
                f,
                "bind-map row {index} says {sidecar} but ips file line {index} says {ips_file}"
            ),
            Self::HashMismatch { sidecar, computed } => write!(
                f,
                "bind-map names ips_file_sha256 {sidecar} but the file on disk hashes to \
                 {computed}"
            ),
            Self::StaleGeneration { generation } => write!(
                f,
                "bind-map generation {generation} was reused for a changed mapping"
            ),
            Self::RetryExhausted { attempts } => write!(
                f,
                "bind-map and ips file never agreed across {attempts} attempt(s)"
            ),
        }
    }
}

impl std::error::Error for BindMapError {}
