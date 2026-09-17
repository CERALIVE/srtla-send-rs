//! The writer half of the ADR-003 bind-map contract, as a test can drive it.
//!
//! Publication order is the contract, not a detail: the IP file is renamed
//! first, the sidecar second, and **the sidecar rename is the commit point**. A
//! reader landing between the two renames sees new IP bytes against an older
//! digest — the transient the sender's bounded retry exists to absorb.
//!
//! Every method here writes what a *correct* writer would, except the ones whose
//! names say otherwise; those exist so a failure branch has something real to
//! fail against.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// One sidecar row: which stable identity egresses through which interface.
#[derive(Debug, Clone)]
pub struct TwinRow {
    pub link_id: String,
    pub iface: String,
    pub priority: Option<f64>,
}

impl TwinRow {
    pub fn new(link_id: &str, iface: &str) -> Self {
        Self {
            link_id: link_id.to_string(),
            iface: iface.to_string(),
            priority: None,
        }
    }

    pub fn with_priority(link_id: &str, iface: &str, priority: f64) -> Self {
        Self {
            priority: Some(priority),
            ..Self::new(link_id, iface)
        }
    }
}

/// Publishes the `(ips_file, sidecar)` pair the sender reads.
pub struct BindMapPublisher {
    dir: tempfile::TempDir,
    generation: AtomicU64,
}

impl BindMapPublisher {
    pub fn new() -> Result<Self> {
        Ok(Self {
            dir: tempfile::tempdir().context("create the bind-map publication dir")?,
            generation: AtomicU64::new(0),
        })
    }

    #[must_use]
    pub fn ips_path(&self) -> PathBuf {
        self.dir.path().join("srtla_ips.txt")
    }

    #[must_use]
    pub fn sidecar_path(&self) -> PathBuf {
        self.dir.path().join("srtla_bind_map.json")
    }

    /// The generation of the most recent coherent publication.
    #[must_use]
    pub fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Publish a coherent pair at the next generation.
    pub fn publish(&self, ip: &str, rows: &[TwinRow]) -> Result<u64> {
        let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
        self.publish_at(ip, rows, generation)?;
        Ok(generation)
    }

    /// Publish a coherent pair at an explicit generation.
    ///
    /// Re-publishing *different* rows under an unchanged generation and digest
    /// is the `stale-generation` rejection, so a test needs to be able to pin it.
    pub fn publish_at(&self, ip: &str, rows: &[TwinRow], generation: u64) -> Result<()> {
        self.publish_rows(
            &rows.iter().map(|row| (ip, row)).collect::<Vec<_>>(),
            generation,
        )
    }

    pub(crate) fn publish_bond_at(&self, rows: &[(&str, TwinRow)], generation: u64) -> Result<()> {
        self.publish_rows(
            &rows.iter().map(|(ip, row)| (*ip, row)).collect::<Vec<_>>(),
            generation,
        )
    }

    fn publish_rows(&self, rows: &[(&str, &TwinRow)], generation: u64) -> Result<()> {
        let ips = rows
            .iter()
            .map(|(ip, _)| *ip)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        write_atomic(&self.ips_path(), ips.as_bytes())?;
        let digest = hex_digest(ips.as_bytes());
        write_atomic(
            &self.sidecar_path(),
            bond_sidecar_json(generation, &digest, rows).as_bytes(),
        )
    }

    /// Publish only the IP file — the legacy channel, byte-unchanged.
    pub fn publish_ips_only(&self, ip: &str, count: usize) -> Result<()> {
        self.publish_ips(&std::iter::repeat_n(ip, count).collect::<Vec<_>>())
    }

    pub(crate) fn publish_ips(&self, ips: &[&str]) -> Result<()> {
        let ips = ips.join("\n") + "\n";
        write_atomic(&self.ips_path(), ips.as_bytes())
    }

    /// Replace the sidecar with unparseable bytes, leaving the IP file coherent.
    ///
    /// This is the `malformed` degradation a live reload has to survive without
    /// un-binding a running bond.
    pub fn corrupt_sidecar(&self) -> Result<()> {
        write_atomic(&self.sidecar_path(), b"{ this is not a sidecar")
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

#[cfg(test)]
fn sidecar_json(generation: u64, digest: &str, ip: &str, rows: &[TwinRow]) -> String {
    bond_sidecar_json(
        generation,
        digest,
        &rows.iter().map(|row| (ip, row)).collect::<Vec<_>>(),
    )
}

fn bond_sidecar_json(generation: u64, digest: &str, rows: &[(&str, &TwinRow)]) -> String {
    let links = rows
        .iter()
        .map(|(ip, row)| {
            let priority = match row.priority {
                Some(value) => format!(r#","priority":{value}"#),
                None => String::new(),
            };
            format!(
                r#"{{"link_id":{:?},"ip":{ip:?},"iface":{:?}{priority}}}"#,
                row.link_id, row.iface,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"schema_version":1,"generation":{generation},"ips_file_sha256":"{digest}","links":[{links}]}}"#
    )
}

/// Write via a uniquely-named temp sibling, then `rename(2)`.
///
/// The unique name matters: the reader ignores a `.tmp` sibling, and two
/// publications racing on one fixed temp name would corrupt each other.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("publication path has no parent")?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path)
        .with_context(|| format!("commit {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_sidecar_bytes_unchanged_without_priority() {
        // Given: the legacy two-row publisher input.
        let rows = [TwinRow::new("twin-a", "ts0"), TwinRow::new("twin-b", "ts1")];
        // When: serializing without priorities.
        let actual = sidecar_json(7, "digest", "10.30.9.1", &rows);
        // Then: the exact pre-extension bytes, not just equivalent JSON.
        assert_eq!(actual.as_bytes(), br#"{"schema_version":1,"generation":7,"ips_file_sha256":"digest","links":[{"link_id":"twin-a","ip":"10.30.9.1","iface":"ts0"},{"link_id":"twin-b","ip":"10.30.9.1","iface":"ts1"}]}"#);
    }

    #[test]
    fn priority_is_emitted_only_for_the_row_that_supplies_it() {
        // Given: one prioritized row and one legacy row.
        let rows = [
            TwinRow::with_priority("a", "ts0", 0.5),
            TwinRow::new("b", "ts1"),
        ];
        // When: serializing the mixed pair.
        let actual = sidecar_json(1, "digest", "10.30.9.1", &rows);
        // Then: priority is additive and absent stays absent.
        assert_eq!(
            actual,
            r#"{"schema_version":1,"generation":1,"ips_file_sha256":"digest","links":[{"link_id":"a","ip":"10.30.9.1","iface":"ts0","priority":0.5},{"link_id":"b","ip":"10.30.9.1","iface":"ts1"}]}"#
        );
    }
}
