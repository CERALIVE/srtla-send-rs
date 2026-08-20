//! Sidecar document parsing: JSON in, validated header + raw rows out.
//!
//! Rows stay raw here on purpose. Row validation must run *after* the hash
//! coherence check (ADR-003 §4.1), and this function has never seen the IP file,
//! so it cannot perform that check. Validating rows early would report a row bug
//! for content that is already known to be stale.

use serde::Deserialize;

use super::error::BindMapError;

/// The sidecar `schema_version` this build reads and writes.
pub const BIND_MAP_SCHEMA_VERSION: u32 = 1;

/// Length of a lowercase-hex SHA-256 digest.
const SHA256_HEX_LEN: usize = 64;

/// A parsed sidecar with a validated header and still-raw rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindMapDoc {
    pub generation: u64,
    pub ips_file_sha256: String,
    pub(super) links: Vec<RawRow>,
}

impl BindMapDoc {
    /// Rows exactly as the writer published them, unvalidated.
    pub(super) fn raw_links(&self) -> &[RawRow] {
        &self.links
    }
}

/// One row before its fields have been parsed into typed values.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct RawRow {
    pub(super) link_id: String,
    pub(super) ip: String,
    pub(super) iface: String,
    #[serde(default)]
    pub(super) id_path: Option<String>,
}

/// The wire shape. Unknown keys are ignored so an additive field within the same
/// `schema_version` cannot break an older reader (ADR-003 §1.1).
#[derive(Deserialize)]
struct WireDoc {
    schema_version: u32,
    generation: u64,
    ips_file_sha256: String,
    #[serde(default)]
    links: Vec<RawRow>,
}

/// Parse a sidecar document and validate its header.
///
/// Checks run in the ADR's declared order — version, then generation, then
/// digest form — so the reported reason is reproducible.
pub fn parse_sidecar(bytes: &[u8]) -> Result<BindMapDoc, BindMapError> {
    let wire: WireDoc =
        serde_json::from_slice(bytes).map_err(|e| BindMapError::MalformedJson(e.to_string()))?;

    if wire.schema_version != BIND_MAP_SCHEMA_VERSION {
        return Err(BindMapError::UnsupportedSchemaVersion {
            found: wire.schema_version,
        });
    }

    if wire.generation == 0 {
        return Err(BindMapError::InvalidHeader {
            field: "generation",
            detail: "0 is the reserved unset sentinel; the floor is 1".to_string(),
        });
    }

    if wire.ips_file_sha256.len() != SHA256_HEX_LEN
        || !wire
            .ips_file_sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(BindMapError::InvalidHeader {
            field: "ips_file_sha256",
            detail: format!(
                "must be {SHA256_HEX_LEN} lowercase hex characters, got `{}`",
                wire.ips_file_sha256
            ),
        });
    }

    Ok(BindMapDoc {
        generation: wire.generation,
        ips_file_sha256: wire.ips_file_sha256,
        links: wire.links,
    })
}
