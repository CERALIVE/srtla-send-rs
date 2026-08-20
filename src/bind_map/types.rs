//! Validated bind-map values (ADR-003 §1).
//!
//! Nothing in here can hold a value the ADR forbids: the constructors are the
//! only way in, and they parse rather than validate. Downstream code therefore
//! never re-checks a `LinkId` or an `IfaceName`.

use std::net::IpAddr;
use std::str::FromStr;

use sha2::{Digest, Sha256};

use super::error::BindMapError;

/// Maximum `link_id` length in bytes.
const LINK_ID_MAX_LEN: usize = 64;
/// Maximum interface-name length in bytes (`IFNAMSIZ - 1`).
const IFACE_MAX_LEN: usize = 15;
/// Maximum `id_path` length in bytes.
const ID_PATH_MAX_LEN: usize = 4096;

/// True for printable, non-space ASCII (`0x21`–`0x7E`).
fn is_printable_ascii(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| (0x21..=0x7e).contains(&b))
}

/// An opaque, writer-assigned link identity.
///
/// Stable across reloads, reconnects, IP changes, and interface renames. The
/// sender never parses meaning out of it and never invents one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LinkId(String);

impl LinkId {
    /// Parse a writer-supplied identity, rejecting anything outside the domain.
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() > LINK_ID_MAX_LEN {
            return Err(format!(
                "{} bytes exceeds the {LINK_ID_MAX_LEN}-byte ceiling",
                raw.len()
            ));
        }
        if !is_printable_ascii(raw) {
            return Err("must be 1-64 printable ASCII bytes with no spaces".to_string());
        }
        Ok(Self(raw.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A validated network-interface name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfaceName(String);

impl IfaceName {
    /// Parse an interface name against the kernel's own naming rules.
    pub fn parse(raw: &str) -> Result<Self, String> {
        if raw.len() > IFACE_MAX_LEN {
            return Err(format!(
                "{} bytes exceeds the {IFACE_MAX_LEN}-byte kernel ceiling",
                raw.len()
            ));
        }
        if !is_printable_ascii(raw) {
            return Err("must be 1-15 printable ASCII bytes with no spaces".to_string());
        }
        if raw.contains('/') || raw == "." || raw == ".." {
            return Err("must not be a path component".to_string());
        }
        Ok(Self(raw.to_string()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One validated sidecar row: an identity, its current socket key, and optional
/// writer provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindMapRow {
    pub link_id: LinkId,
    pub ip: IpAddr,
    pub iface: IfaceName,
    /// Opaque writer provenance. Never opened, stat-ed, or resolved.
    pub id_path: Option<String>,
}

/// Parse the optional `id_path` provenance field.
pub(super) fn parse_id_path(raw: &str) -> Result<String, String> {
    if raw.len() > ID_PATH_MAX_LEN {
        return Err(format!(
            "{} bytes exceeds the {ID_PATH_MAX_LEN}-byte ceiling",
            raw.len()
        ));
    }
    if !raw.starts_with('/') {
        return Err("must be an absolute path".to_string());
    }
    Ok(raw.to_string())
}

/// A coherent, fully validated mapping — the thing the sender may act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedPool {
    pub generation: u64,
    pub ips_file_sha256: String,
    pub rows: Vec<BindMapRow>,
}

/// The `BIND_IPS_FILE`, read once: its raw-byte digest and the IPs the legacy
/// rules accept from it.
///
/// Hashing and parsing share one read on purpose — reading twice would let the
/// file change in between and produce a digest describing content the sender did
/// not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpsFile {
    sha256: String,
    accepted: Vec<IpAddr>,
}

impl IpsFile {
    /// Parse and hash one read of the IP list.
    ///
    /// The accepted-IP rules are **identical** to `sender::read_ip_list`: trim,
    /// skip blank lines, skip unparseable lines. Duplicates are preserved,
    /// because position is what disambiguates them (ADR-003 §2).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BindMapError> {
        let text = std::str::from_utf8(bytes).map_err(|e| BindMapError::Unreadable {
            path: "<ips file>".to_string(),
            detail: format!("not valid UTF-8: {e}"),
        })?;

        let accepted = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| IpAddr::from_str(line).ok())
            .collect();

        Ok(Self {
            sha256: hex_digest(bytes),
            accepted,
        })
    }

    /// Lowercase hex SHA-256 over the raw bytes that were read.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// The accepted IPs, in file order, duplicates preserved.
    #[must_use]
    pub fn accepted(&self) -> &[IpAddr] {
        &self.accepted
    }
}

/// Lowercase hex SHA-256 of `bytes`.
fn hex_digest(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(out, "{byte:02x}");
    }
    out
}
