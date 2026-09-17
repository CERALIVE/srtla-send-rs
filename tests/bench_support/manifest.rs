//! Campaign boundary and paired work ordering.
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result, ensure};
use network_sim::harness::SrtProfile;
use network_sim::profile::Action;
use network_sim::scenarios::Profile;
use rand::SeedableRng;
use rand::seq::SliceRandom;

#[path = "model.rs"]
mod model;
pub use model::*;
#[path = "diagnostic_profiles.rs"]
mod diagnostic_profiles;
#[path = "priorities.rs"]
mod priorities;
#[cfg(test)]
#[path = "priority_tests.rs"]
mod priority_tests;
#[path = "receivers.rs"]
mod receivers;
#[path = "settle.rs"]
mod settle;
pub use settle::Settle;

#[derive(Debug, PartialEq, Eq)]
pub enum ManifestError {
    UnknownScenario(String),
    Invalid(String),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownScenario(id) => write!(f, "unknown scenario: {id}"),
            Self::Invalid(reason) => write!(f, "invalid manifest: {reason}"),
        }
    }
}
impl std::error::Error for ManifestError {}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && !value.contains("--")
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}

pub fn parse(json: &str) -> Result<Manifest> {
    parse_with_lock(
        json,
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/bench/receivers.lock.json"),
    )
}

pub fn parse_with_lock(json: &str, lock: &std::path::Path) -> Result<Manifest> {
    let mut manifest: Manifest = serde_json::from_str(json)?;
    receivers::normalize(&mut manifest, lock)?;
    manifest.validate()?;
    Ok(manifest)
}

impl Manifest {
    pub fn validate(&self) -> Result<()> {
        for id in self
            .cells
            .iter()
            .map(|c| &c.scenario)
            .chain(&self.scenarios)
        {
            self.scenario(id)?;
        }
        let valid = identifier(&self.campaign)
            && !self.cells.is_empty()
            && !self.candidates.is_empty()
            && !self.receivers.is_empty();
        ensure!(
            valid,
            ManifestError::Invalid("empty matrix or unsafe campaign name".into())
        );
        let mut labels = BTreeSet::new();
        for c in &self.candidates {
            ensure!(
                identifier(&c.label) && labels.insert(&c.label),
                ManifestError::Invalid("duplicate/unsafe candidate label".into())
            );
            ensure!(
                !c.args
                    .iter()
                    .any(|a| ["--control-socket", "--stats-file", "--bind-map"]
                        .iter()
                        .any(|flag| a == flag || a.starts_with(&format!("{flag}=")))),
                ManifestError::Invalid("runner owns socket/stats/bind-map paths".into())
            );
            ensure!(
                !c.args.iter().any(|a| a.contains("adaptive"))
                    && !c.env.keys().any(|k| k.contains("ADAPTIVE"))
                    || c.effective_config.is_some(),
                ManifestError::Invalid("adaptive overrides require effective_config".into())
            );
        }
        let mut receivers = BTreeSet::new();
        for r in &self.receivers {
            r.validate()?;
            ensure!(
                identifier(&r.name) && receivers.insert(&r.name),
                ManifestError::Invalid("duplicate/unsafe receiver name".into())
            );
        }
        let mut ids = BTreeSet::new();
        let mut covering = BTreeMap::new();
        for cell in &self.cells {
            self.cell_profile(cell)?;
            ensure!(
                covering
                    .insert(&cell.cell_id, cell.covering)
                    .is_none_or(|previous| previous == cell.covering),
                "paired candidates must agree on covering"
            );
            let receiver = self
                .receivers
                .iter()
                .find(|r| r.name == cell.receiver)
                .context("receiver")?;
            ensure!(
                cell.cell_id == receivers::cell_identity(cell, receiver),
                "incoherent cell_id"
            );
            match cell.sink.as_str() {
                "slt" => ensure!(
                    cell.metrics == "full" && !cell.fec && cell.port > 0,
                    "slt requires full metrics, a port and FEC off"
                ),
                "sls" => ensure!(
                    cell.metrics == "none"
                        && !cell.covering
                        && !cell.fec
                        && [4002, 4003].contains(&cell.port)
                        && receiver.listener_uri_extra.is_empty()
                        && cell.srt_profile != "legacy-default",
                    "sls requires metrics:none, covering:false, port 4002/4003, explicit latency \
                     and no URI/FEC overrides"
                ),
                _ => return Err(ManifestError::Invalid("unknown sink".into()).into()),
            }
            ensure!(
                cell.variant.is_empty() || !cell.covering,
                "variants must explicitly set covering:false"
            );
            ensure!(
                cell.runs > 0
                    && labels.contains(&cell.candidate)
                    && receivers.contains(&cell.receiver)
                    && ids.insert(cell.id()),
                ManifestError::Invalid("invalid, duplicate or unresolved cell".into())
            );
            preset(&cell.srt_profile)?;
        }
        for name in &self.srt_profiles {
            preset(name)?;
        }
        priorities::validate(self)?;
        Ok(())
    }

    pub fn order(&self) -> Vec<Work> {
        let mut groups = BTreeMap::new();
        for (i, cell) in self.cells.iter().enumerate() {
            groups.entry(&cell.cell_id).or_insert_with(Vec::new).push(i);
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(self.seed);
        let mut work = Vec::new();
        for group in groups.values() {
            let runs = group.iter().map(|i| self.cells[*i].runs).max().unwrap_or(0);
            for run in 0..runs {
                let mut arms: Vec<_> = group
                    .iter()
                    .copied()
                    .filter(|i| run < self.cells[*i].runs)
                    .collect();
                arms.shuffle(&mut rng);
                work.extend(arms.into_iter().map(|cell| Work { cell, run }));
            }
        }
        work
    }

    pub fn smoke(&mut self) {
        let combinations: BTreeMap<_, _> = self
            .cells
            .iter()
            .map(|c| {
                (
                    (
                        c.receiver.clone(),
                        c.srt_profile.clone(),
                        c.variant.clone(),
                        c.sink.clone(),
                        c.port,
                        c.fec,
                        c.covering,
                    ),
                    c.clone(),
                )
            })
            .collect();
        self.cells = combinations
            .into_values()
            .flat_map(|template| {
                self.candidates.iter().flat_map(move |candidate| {
                    ["A", "D"].map(|scenario| Cell {
                        cell_id: String::new(),
                        candidate: candidate.label.clone(),
                        scenario: scenario.into(),
                        runs: 2,
                        ..template.clone()
                    })
                })
            })
            .collect();
        for cell in &mut self.cells {
            if let Some(receiver) = self.receivers.iter().find(|r| r.name == cell.receiver) {
                cell.cell_id = receivers::cell_identity(cell, receiver);
            }
        }
        self.scenarios = vec!["A".into(), "D".into()];
        self.runs = Some(2);
        self.window_secs_override = None;
    }
}

pub fn preset(name: &str) -> Result<SrtProfile> {
    match name {
        "production" => Ok(SrtProfile::PRODUCTION),
        "strict" => Ok(SrtProfile::STRICT),
        "legacy-default" => Ok(SrtProfile::LEGACY_DEFAULT),
        _ => Err(ManifestError::Invalid(format!("unknown SRT profile: {name}")).into()),
    }
}

pub fn measurement_rate(profile: &Profile) -> Result<u64> {
    profile
        .timeline
        .events
        .iter()
        .find_map(|event| match event.action {
            Action::OfferedRate { bps } if event.at.is_zero() => Some(bps),
            _ => None,
        })
        .context("missing measurement-start OfferedRate")
}
