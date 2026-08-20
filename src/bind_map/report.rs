//! The telemetry-facing projection of an ADR-003 resolution (ADR-003 §6.4).
//!
//! A [`Resolution`] is the sender's live decision object: it owns the effective
//! link set and the pool to remember. Telemetry needs only the *observable* part
//! of that decision — is the map in force, what is actually running, and which
//! same-IP group had to be broken up — in a shape a UI can render without
//! inferring anything from log text.
//!
//! Nothing here re-derives state. Every value is read straight off the
//! [`BindMapStatus`] / [`BindMapDisposition`] / [`CollisionGroup`] types the
//! contract already defines, through their own token accessors, so a UI and a
//! log line can never disagree about what the sender is doing.

use serde::Serialize;

use super::error::DegradedReason;
use super::resolve::{BindMapDisposition, BindMapStatus, CollisionGroup, Resolution};

/// Is the map in force, and if not, why not?
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BindMapStatusRecord {
    /// `active` | `absent` | `degraded`.
    pub state: &'static str,
    /// One of the seven degraded reasons; absent unless `state` is `degraded`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
}

impl Default for BindMapStatusRecord {
    /// No `--bind-map` was supplied, which is the shipped default.
    fn default() -> Self {
        Self {
            state: BindMapStatus::Absent.state_str(),
            reason: None,
        }
    }
}

impl From<&BindMapStatus> for BindMapStatusRecord {
    fn from(status: &BindMapStatus) -> Self {
        Self {
            state: status.state_str(),
            reason: status.reason().map(super::DegradedReason::as_str),
        }
    }
}

/// One same-IP group that a degraded startup could not disambiguate.
///
/// The indices are **`BIND_IPS_FILE` line positions**, not telemetry `conn_id`s:
/// an excluded line never becomes a connection, so the two numberings diverge
/// exactly when this record is non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CollisionRecord {
    pub ip: String,
    pub effective_index: usize,
    pub excluded_indices: Vec<usize>,
}

impl From<&CollisionGroup> for CollisionRecord {
    fn from(group: &CollisionGroup) -> Self {
        Self {
            ip: group.ip.to_string(),
            effective_index: group.effective_index,
            excluded_indices: group.excluded_indices.clone(),
        }
    }
}

/// What is the sender actually running, and at whose expense?
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DispositionRecord {
    /// `mapped` | `retained_last_valid` | `legacy_unique_only` | `startup_collision_excluded`.
    pub state: &'static str,
    /// The groups a `startup_collision_excluded` disposition cost. Empty for
    /// every other disposition, and omitted from the JSON when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collisions: Vec<CollisionRecord>,
}

impl Default for DispositionRecord {
    /// Legacy behavior with no ambiguity, which is the shipped default.
    fn default() -> Self {
        Self {
            state: BindMapDisposition::LegacyUniqueOnly.as_str(),
            collisions: Vec::new(),
        }
    }
}

/// The pair of typed fields telemetry publishes at the top level of a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct BindMapReport {
    pub bind_map_status: BindMapStatusRecord,
    pub disposition: DispositionRecord,
}

impl BindMapReport {
    /// A degradation that never reached [`Resolution`] — the outer `Err` arm of
    /// a pair read, where `BIND_IPS_FILE` itself was unusable so there was no
    /// link set to resolve against.
    #[must_use]
    pub fn degraded(reason: DegradedReason, disposition: BindMapDisposition) -> Self {
        Self {
            bind_map_status: BindMapStatusRecord::from(&BindMapStatus::Degraded(reason)),
            disposition: DispositionRecord {
                state: disposition.as_str(),
                collisions: Vec::new(),
            },
        }
    }
}

impl From<&Resolution> for BindMapReport {
    fn from(resolution: &Resolution) -> Self {
        Self {
            bind_map_status: BindMapStatusRecord::from(&resolution.status),
            disposition: DispositionRecord {
                state: resolution.disposition.as_str(),
                collisions: resolution
                    .excluded
                    .iter()
                    .map(CollisionRecord::from)
                    .collect(),
            },
        }
    }
}
