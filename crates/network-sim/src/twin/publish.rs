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

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// One sidecar row: which stable identity egresses through which interface.
#[derive(Debug, Clone)]
pub struct TwinRow {
    pub link_id: String,
    pub iface: String,
}

impl TwinRow {
    pub fn new(link_id: &str, iface: &str) -> Self {
        Self {
            link_id: link_id.to_string(),
            iface: iface.to_string(),
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
        let ips = std::iter::repeat_n(ip, rows.len())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        write_atomic(&self.ips_path(), ips.as_bytes())?;
        let digest = hex_digest(ips.as_bytes());
        write_atomic(
            &self.sidecar_path(),
            sidecar_json(generation, &digest, ip, rows).as_bytes(),
        )
    }

    /// Publish only the IP file — the legacy channel, byte-unchanged.
    pub fn publish_ips_only(&self, ip: &str, count: usize) -> Result<()> {
        let ips = std::iter::repeat_n(ip, count)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
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

fn sidecar_json(generation: u64, digest: &str, ip: &str, rows: &[TwinRow]) -> String {
    let links = rows
        .iter()
        .map(|row| {
            format!(
                r#"{{"link_id":"{}","ip":"{}","iface":"{}"}}"#,
                row.link_id, ip, row.iface
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
    let file_name = path
        .file_name()
        .context("publication path has no file name")?;
    let temp = parent.join(format!(
        "{}.{}.publishing",
        file_name.to_string_lossy(),
        std::process::id()
    ));
    std::fs::write(&temp, bytes).with_context(|| format!("stage {}", temp.display()))?;
    // The sender refuses a group- or world-writable sidecar outright.
    #[cfg(unix)]
    std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o600))
        .with_context(|| format!("tighten {}", temp.display()))?;
    std::fs::rename(&temp, path).with_context(|| format!("commit {}", path.display()))?;
    Ok(())
}
