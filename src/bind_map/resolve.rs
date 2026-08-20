//! Fail-open, duplicate-safe resolution (ADR-003 §6).
//!
//! Fail-open means "keep streaming". It does not mean "guess". The naive
//! fallback — no map, so collapse duplicate IPs and carry on — is exactly the
//! silent-collapse bug this contract exists to fix, so degradation never
//! blindly collapses an ambiguous group.
//!
//! Startup and reload get different answers because they have different
//! material: startup has nothing to retain, reload has a known-good pool.

use std::collections::HashMap;
use std::net::IpAddr;

use super::error::{BindMapError, DegradedReason};
use super::types::{IfaceName, IpsFile, LinkId, MappedPool};

/// Is a configured bind-map in force?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindMapStatus {
    /// A coherent, valid pair is applied.
    Active,
    /// `--bind-map` was not supplied.
    Absent,
    /// The map is configured but not in force.
    Degraded(DegradedReason),
}

impl BindMapStatus {
    /// The snake_case state token (ADR-003 §6.4), without the degraded reason.
    ///
    /// This is the one place the three state words are spelled; the log line and
    /// the telemetry projection both read it, so they can never drift apart.
    #[must_use]
    pub const fn state_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Absent => "absent",
            Self::Degraded(_) => "degraded",
        }
    }

    /// The degraded reason, or `None` for the two non-degraded states.
    #[must_use]
    pub const fn reason(&self) -> Option<DegradedReason> {
        match self {
            Self::Degraded(reason) => Some(*reason),
            Self::Active | Self::Absent => None,
        }
    }
}

/// What is the sender actually running?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindMapDisposition {
    /// The map is in force.
    Mapped,
    /// A degraded reload was rejected; the last valid mapped pool still runs.
    RetainedLastValid,
    /// Legacy behavior; no collision groups were present.
    LegacyUniqueOnly,
    /// Degraded at startup with at least one collision group.
    StartupCollisionExcluded,
}

impl BindMapDisposition {
    /// The exact snake_case token published in logs and telemetry (ADR-003 §6.4).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Mapped => "mapped",
            Self::RetainedLastValid => "retained_last_valid",
            Self::LegacyUniqueOnly => "legacy_unique_only",
            Self::StartupCollisionExcluded => "startup_collision_excluded",
        }
    }
}

/// One uplink the sender should actually run.
///
/// `iface` and `link_id` are `None` whenever the sender had to fall back — a
/// degraded read never invents a binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveLink {
    pub ip: IpAddr,
    pub iface: Option<IfaceName>,
    pub link_id: Option<LinkId>,
}

/// An ambiguous same-IP group that degraded startup could not disambiguate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollisionGroup {
    pub ip: IpAddr,
    /// Index of the row that runs — always the first occurrence in file order.
    pub effective_index: usize,
    /// Indices of the rows that do not run.
    pub excluded_indices: Vec<usize>,
}

/// Which side of the "have we ever applied a valid map?" line we are on.
pub enum ResolvePhase<'a> {
    /// No valid map has been applied yet.
    Startup,
    /// A valid map is already running.
    Reload { last_valid: &'a MappedPool },
}

/// The typed decision: what runs, under what status, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub status: BindMapStatus,
    pub disposition: BindMapDisposition,
    pub links: Vec<EffectiveLink>,
    pub excluded: Vec<CollisionGroup>,
    /// The pool the caller must remember. `Some` iff `status == Active`.
    pub applied: Option<MappedPool>,
}

/// `--bind-map` was not supplied: byte-identical legacy behavior.
///
/// Every accepted IP is handed on in file order, **duplicates included** —
/// deduplication is the legacy pool builder's job, and filtering here would
/// change shipped behavior.
#[must_use]
pub fn resolve_absent(ips: &IpsFile) -> Resolution {
    Resolution {
        status: BindMapStatus::Absent,
        disposition: BindMapDisposition::LegacyUniqueOnly,
        links: ips
            .accepted()
            .iter()
            .map(|ip| EffectiveLink {
                ip: *ip,
                iface: None,
                link_id: None,
            })
            .collect(),
        excluded: Vec::new(),
        applied: None,
    }
}

/// Turn a read outcome into the sender's operating decision.
#[must_use]
pub fn resolve(
    outcome: Result<MappedPool, BindMapError>,
    ips: &IpsFile,
    phase: ResolvePhase<'_>,
) -> Resolution {
    match outcome {
        Ok(pool) => Resolution {
            status: BindMapStatus::Active,
            disposition: BindMapDisposition::Mapped,
            links: pool
                .rows
                .iter()
                .map(|row| EffectiveLink {
                    ip: row.ip,
                    iface: Some(row.iface.clone()),
                    link_id: Some(row.link_id.clone()),
                })
                .collect(),
            excluded: Vec::new(),
            applied: Some(pool),
        },
        Err(err) => degrade(err.reason(), ips, phase),
    }
}

fn degrade(reason: DegradedReason, ips: &IpsFile, phase: ResolvePhase<'_>) -> Resolution {
    match phase {
        // A live bond already has interface bindings. Falling back to legacy
        // would silently un-bind every one of them, which is worse than running
        // the last known-good mapping until a coherent pair arrives.
        ResolvePhase::Reload { last_valid } => Resolution {
            status: BindMapStatus::Degraded(reason),
            disposition: BindMapDisposition::RetainedLastValid,
            links: last_valid
                .rows
                .iter()
                .map(|row| EffectiveLink {
                    ip: row.ip,
                    iface: Some(row.iface.clone()),
                    link_id: Some(row.link_id.clone()),
                })
                .collect(),
            excluded: Vec::new(),
            applied: None,
        },
        ResolvePhase::Startup => {
            let (links, excluded) = exclude_collisions(ips.accepted());
            let disposition = if excluded.is_empty() {
                BindMapDisposition::LegacyUniqueOnly
            } else {
                BindMapDisposition::StartupCollisionExcluded
            };
            Resolution {
                status: BindMapStatus::Degraded(reason),
                disposition,
                links,
                excluded,
                applied: None,
            }
        }
    }
}

/// Keep one deterministic representative per IP — the first occurrence in file
/// order — and report every row that therefore does not run.
///
/// The *effective* link set here equals what legacy would have produced. The
/// difference is that the ambiguity is named instead of silently swallowed: an
/// operator with two modems and one visible link gets told why.
fn exclude_collisions(accepted: &[IpAddr]) -> (Vec<EffectiveLink>, Vec<CollisionGroup>) {
    let mut first_seen: HashMap<IpAddr, usize> = HashMap::new();
    let mut links = Vec::new();
    let mut groups: Vec<CollisionGroup> = Vec::new();

    for (index, ip) in accepted.iter().enumerate() {
        match first_seen.get(ip) {
            None => {
                first_seen.insert(*ip, index);
                links.push(EffectiveLink {
                    ip: *ip,
                    iface: None,
                    link_id: None,
                });
            }
            Some(&effective_index) => match groups.iter_mut().find(|g| g.ip == *ip) {
                Some(group) => group.excluded_indices.push(index),
                None => groups.push(CollisionGroup {
                    ip: *ip,
                    effective_index,
                    excluded_indices: vec![index],
                }),
            },
        }
    }

    (links, groups)
}
