use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};
use network_sim::harness::ReceiverSpec;
use serde::Deserialize;

use super::{Cell, Manifest, Receiver};

#[derive(Default, Deserialize)]
struct ReceiverLock {
    #[serde(default)]
    receivers: BTreeMap<String, LockedReceiver>,
}

#[derive(Deserialize)]
struct LockedReceiver {
    srtla_rec_bin: PathBuf,
    srt_live_transmit_bin: PathBuf,
}

pub fn normalize(manifest: &mut Manifest, lock_path: &Path) -> Result<()> {
    for receiver in &mut manifest.receivers {
        for (path, is_srt) in [
            (&mut receiver.bin, false),
            (&mut receiver.srt_live_transmit_bin, true),
        ] {
            if let Some(entry) = path
                .as_deref()
                .and_then(Path::to_str)
                .and_then(|s| s.strip_prefix("lock:"))
            {
                let lock: ReceiverLock =
                    serde_json::from_slice(&std::fs::read(lock_path).with_context(|| {
                        format!(
                            "receiver lock entry {entry:?}: read {}",
                            lock_path.display()
                        )
                    })?)
                    .with_context(|| format!("receiver lock entry {entry:?}"))?;
                let resolved = lock
                    .receivers
                    .get(entry)
                    .with_context(|| format!("missing receiver lock entry {entry:?}"))?;
                let binary = if is_srt {
                    &resolved.srt_live_transmit_bin
                } else {
                    &resolved.srtla_rec_bin
                };
                ensure!(
                    !binary.to_string_lossy().starts_with("lock:"),
                    "nested receiver lock entry {entry:?}"
                );
                *path = Some(binary.clone());
            }
        }
    }
    for cell in &mut manifest.cells {
        let receiver = manifest
            .receivers
            .iter()
            .find(|r| r.name == cell.receiver)
            .context("unresolved receiver")?;
        let canonical = cell_identity(cell, receiver);
        ensure!(
            cell.cell_id.is_empty() || cell.cell_id == canonical,
            "cell_id must encode the complete cell: {canonical}"
        );
        cell.cell_id = canonical;
    }
    Ok(())
}

fn escaped(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' => char::from(b).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub fn cell_identity(cell: &Cell, receiver: &Receiver) -> String {
    format!(
        "{}@{}--{}@{}--{}--{}:{}--fec:{}",
        receiver.name,
        escaped(&receiver.listener_uri_extra),
        cell.scenario,
        escaped(&cell.variant),
        cell.srt_profile,
        cell.sink,
        cell.port,
        if cell.fec { "on" } else { "off" }
    )
}

impl Receiver {
    pub fn resolve(&self, defaults: &ReceiverSpec) -> Result<ReceiverSpec> {
        let kind = match self.kind {
            Some(network_sim::metrics::record::ReceiverKind::Ceralive) => "ceralive",
            Some(network_sim::metrics::record::ReceiverKind::Irlserver) => "irlserver",
            Some(network_sim::metrics::record::ReceiverKind::Belabox) => "belabox",
            None => match self.name.as_str() {
                "ceralive" => "ceralive",
                "irlserver" => "irlserver",
                "belabox" => "belabox",
                _ => &defaults.srtla_rec_kind,
            },
        };
        let spec = ReceiverSpec {
            label: self.name.clone(),
            lineage: self.lineage.clone().unwrap_or_else(|| self.name.clone()),
            srtla_rec_kind: kind.into(),
            srtla_rec_bin: self
                .bin
                .clone()
                .unwrap_or_else(|| defaults.srtla_rec_bin.clone()),
            srt_live_transmit_bin: self
                .srt_live_transmit_bin
                .clone()
                .unwrap_or_else(|| defaults.srt_live_transmit_bin.clone()),
            listener_uri_extra: self.listener_uri_extra.clone(),
        };
        spec.resolved()
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(lineage) = &self.lineage {
            ensure!(
                [
                    "ours-new",
                    "ours-old",
                    "irlserver-prod",
                    "irlserver-next",
                    "belabox"
                ]
                .contains(&lineage.as_str()),
                "unknown receiver lineage {lineage}"
            );
        }
        ReceiverSpec::validate_options(&self.listener_uri_extra)
    }
}
