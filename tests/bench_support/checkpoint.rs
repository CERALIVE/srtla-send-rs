//! Atomic fingerprint-aware checkpoints.
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use network_sim::metrics::identity::Hash256;
use network_sim::metrics::record::RunStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub fingerprint: Hash256,
    pub status: RunStatus,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Next {
    Complete,
    Attempt(u32),
    Exhausted,
}

pub struct Store {
    directory: PathBuf,
    fingerprint: Hash256,
    max_attempts: u32,
}

impl Store {
    pub fn new(directory: &Path, fingerprint: Hash256, max_attempts: u32) -> Self {
        Self {
            directory: directory.into(),
            fingerprint,
            max_attempts,
        }
    }

    fn current(&self, path: &Path) -> Result<Option<Receipt>> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let receipt: Receipt = serde_json::from_slice(&bytes)
            .with_context(|| format!("corrupt checkpoint {}", path.display()))?;
        if receipt.fingerprint == self.fingerprint {
            return Ok(Some(receipt));
        }
        let stale = self.directory.join("stale");
        fs::create_dir_all(&stale)?;
        let archive = tempfile::Builder::new()
            .prefix("checkpoint-")
            .tempdir_in(stale)?
            .keep();
        fs::rename(
            path,
            archive.join(path.file_name().context("checkpoint filename")?),
        )?;
        Ok(None)
    }

    pub fn next(&self, run: u32) -> Result<Next> {
        fs::create_dir_all(&self.directory)?;
        if let Some(receipt) = self.current(&self.directory.join(format!("run-{run}.json")))? {
            match receipt.status {
                RunStatus::Ok => return Ok(Next::Complete),
                RunStatus::Failed => anyhow::bail!("failed record at success path"),
            }
        }
        let exhausted = self.directory.join(format!("run-{run}.exhausted.json"));
        // Inspect all attempts, including those made with an older retry budget.
        let mut attempts = Vec::new();
        for entry in fs::read_dir(&self.directory)? {
            let path = entry?.path();
            let name = path
                .file_name()
                .context("checkpoint name")?
                .to_string_lossy();
            if let Some(number) = name
                .strip_prefix(&format!("run-{run}.failed-"))
                .and_then(|s| s.strip_suffix(".json"))
            {
                let attempt: u32 = number.parse()?;
                if self.current(&path)?.is_some() {
                    attempts.push((attempt, path));
                }
            }
        }
        self.current(&exhausted)?;
        attempts.sort_by_key(|(attempt, _)| *attempt);
        let last = attempts.last().map_or(0, |(attempt, _)| *attempt);
        if last >= self.max_attempts {
            let path = &attempts.last().context("positive retry budget required")?.1;
            atomic_bytes(&exhausted, &fs::read(path)?)?;
            return Ok(Next::Exhausted);
        }
        Ok(Next::Attempt(last + 1))
    }

    pub fn save(&self, run: u32, attempt: u32, record: &impl Serialize) -> Result<()> {
        let bytes = serde_json::to_vec(record)?;
        let receipt: Receipt = serde_json::from_slice(&bytes)?;
        ensure!(
            receipt.fingerprint == self.fingerprint,
            "checkpoint fingerprint mismatch"
        );
        let suffix = match receipt.status {
            RunStatus::Ok => String::new(),
            RunStatus::Failed => format!(".failed-{attempt}"),
        };
        atomic_bytes(
            &self.directory.join(format!("run-{run}{suffix}.json")),
            &bytes,
        )
    }

    pub fn count_ok(&self, runs: u32) -> Result<u32> {
        let mut count = 0;
        for run in 0..runs {
            if let Some(receipt) = self.current(&self.directory.join(format!("run-{run}.json")))? {
                match receipt.status {
                    RunStatus::Ok => count += 1,
                    RunStatus::Failed => {}
                }
            }
        }
        Ok(count)
    }
}

pub fn atomic(path: &Path, record: &impl Serialize) -> Result<()> {
    atomic_bytes(path, &serde_json::to_vec(record)?)
}

fn atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("checkpoint parent")?;
    fs::create_dir_all(parent)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path)?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
