//! Pair validation: does this sidecar coherently describe this IP file?
//!
//! Check order is fixed by ADR-003 §4.1 so the reported reason is reproducible.
//! Coherence comes first: rows held during a hash mismatch may be about to be
//! replaced, and reporting a row bug for stale content sends a writer chasing
//! the wrong thing.

use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;

use super::error::BindMapError;
use super::iface::IfaceOracle;
use super::sidecar::BindMapDoc;
use super::types::{BindMapRow, IfaceName, IpsFile, LinkId, MappedPool, parse_id_path};

/// Everything validation needs beyond the two files themselves.
#[derive(Clone, Copy)]
pub struct ValidateCtx<'a> {
    /// Answers "does this interface exist right now?".
    pub ifaces: &'a dyn IfaceOracle,
    /// The last mapping actually applied, if any — only used to order generations.
    pub prior: Option<&'a MappedPool>,
}

/// Validate a candidate `(ips_file, sidecar)` pair into an applicable mapping.
pub fn validate_pair(
    ips: &IpsFile,
    doc: BindMapDoc,
    ctx: ValidateCtx<'_>,
) -> Result<MappedPool, BindMapError> {
    if doc.ips_file_sha256 != ips.sha256() {
        return Err(BindMapError::HashMismatch {
            sidecar: doc.ips_file_sha256,
            computed: ips.sha256().to_string(),
        });
    }

    let rows = parse_rows(&doc, ctx.ifaces)?;
    check_correspondence(&rows, ips.accepted())?;
    check_uniqueness(&rows)?;

    let pool = MappedPool {
        generation: doc.generation,
        ips_file_sha256: doc.ips_file_sha256,
        rows,
    };
    check_generation(&pool, ctx.prior)?;
    Ok(pool)
}

/// Row syntax, then interface existence, in index order.
fn parse_rows(doc: &BindMapDoc, ifaces: &dyn IfaceOracle) -> Result<Vec<BindMapRow>, BindMapError> {
    let mut rows = Vec::with_capacity(doc.raw_links().len());
    for (index, raw) in doc.raw_links().iter().enumerate() {
        let invalid = |field: &'static str, detail: String| BindMapError::InvalidRow {
            index,
            field,
            detail,
        };

        let link_id = LinkId::parse(&raw.link_id).map_err(|e| invalid("link_id", e))?;
        let ip = IpAddr::from_str(&raw.ip)
            .map_err(|e| invalid("ip", format!("`{}` is not an IP address: {e}", raw.ip)))?;
        let iface = IfaceName::parse(&raw.iface).map_err(|e| invalid("iface", e))?;
        let id_path = raw
            .id_path
            .as_deref()
            .map(parse_id_path)
            .transpose()
            .map_err(|e| invalid("id_path", e))?;

        if !ifaces.exists(iface.as_str()) {
            return Err(BindMapError::UnknownIface {
                index,
                iface: iface.as_str().to_string(),
            });
        }

        rows.push(BindMapRow {
            link_id,
            ip,
            iface,
            id_path,
        });
    }
    Ok(rows)
}

/// The Nth row must describe the Nth accepted IP-list entry — the whole basis of
/// duplicate-IP disambiguation. There is no partial acceptance.
fn check_correspondence(rows: &[BindMapRow], accepted: &[IpAddr]) -> Result<(), BindMapError> {
    if rows.len() != accepted.len() {
        return Err(BindMapError::RowCountMismatch {
            sidecar: rows.len(),
            ips_file: accepted.len(),
        });
    }
    for (index, (row, expected)) in rows.iter().zip(accepted).enumerate() {
        if row.ip != *expected {
            return Err(BindMapError::RowIpMismatch {
                index,
                sidecar: row.ip,
                ips_file: *expected,
            });
        }
    }
    Ok(())
}

/// `(ip, iface)` is the socket key and `link_id` is the identity; neither may repeat.
fn check_uniqueness(rows: &[BindMapRow]) -> Result<(), BindMapError> {
    let mut pairs = HashSet::new();
    let mut ids = HashSet::new();
    for (index, row) in rows.iter().enumerate() {
        if !pairs.insert((row.ip, row.iface.clone())) {
            return Err(BindMapError::DuplicatePair {
                index,
                ip: row.ip,
                iface: row.iface.as_str().to_string(),
            });
        }
        if !ids.insert(row.link_id.clone()) {
            return Err(BindMapError::DuplicateLinkId {
                index,
                link_id: row.link_id.as_str().to_string(),
            });
        }
    }
    Ok(())
}

/// A mapping that changed while its generation stood still cannot be ordered
/// against what is already applied, so it is refused rather than guessed at.
///
/// A generation that *decreased* is not checked here at all: that is
/// indistinguishable from a legitimate writer restart (ADR-003 §3).
fn check_generation(
    candidate: &MappedPool,
    prior: Option<&MappedPool>,
) -> Result<(), BindMapError> {
    let Some(prior) = prior else {
        return Ok(());
    };
    let same_epoch = candidate.generation == prior.generation
        && candidate.ips_file_sha256 == prior.ips_file_sha256;
    if same_epoch && candidate.rows != prior.rows {
        return Err(BindMapError::StaleGeneration {
            generation: candidate.generation,
        });
    }
    Ok(())
}
