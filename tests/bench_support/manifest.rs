//! Campaign boundary and paired work ordering.
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use anyhow::{Context, Result, ensure};
use network_sim::harness::SrtProfile;
use network_sim::profile::Action;
use network_sim::scenarios::{self, Profile};
use rand::SeedableRng;
use rand::seq::SliceRandom;

#[path = "model.rs"]
mod model;
pub use model::*;

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
    let manifest: Manifest = serde_json::from_str(json)?;
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
            ensure!(
                identifier(&r.name) && receivers.insert(&r.name),
                ManifestError::Invalid("duplicate/unsafe receiver name".into())
            );
        }
        let mut ids = BTreeSet::new();
        for cell in &self.cells {
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
        Ok(())
    }

    pub fn scenario(&self, id: &str) -> Result<Profile> {
        let mut profile = scenarios::all()
            .into_iter()
            .find(|(name, _)| *name == id)
            .map(|(_, profile)| profile)
            .ok_or_else(|| ManifestError::UnknownScenario(id.into()))?;
        if let Some(seconds) = self.window_secs_override {
            profile.timeline.duration = Duration::from_secs(seconds);
        }
        profile.validate()?;
        Ok(profile)
    }

    pub fn order(&self) -> Vec<Work> {
        let mut groups = BTreeMap::new();
        for (i, cell) in self.cells.iter().enumerate() {
            groups
                .entry((&cell.scenario, &cell.receiver, &cell.srt_profile))
                .or_insert_with(Vec::new)
                .push(i);
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
        let combinations: BTreeSet<_> = self
            .cells
            .iter()
            .map(|c| (c.receiver.clone(), c.srt_profile.clone()))
            .collect();
        self.cells = combinations
            .into_iter()
            .flat_map(|(receiver, srt_profile)| {
                self.candidates.iter().flat_map(move |candidate| {
                    let receiver = receiver.clone();
                    let srt_profile = srt_profile.clone();
                    ["A", "D"].map(|scenario| Cell {
                        candidate: candidate.label.clone(),
                        scenario: scenario.into(),
                        receiver: receiver.clone(),
                        srt_profile: srt_profile.clone(),
                        runs: 2,
                    })
                })
            })
            .collect();
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

pub struct Settle {
    target: f64,
    previous_second: u64,
    consecutive: u32,
}

impl Settle {
    pub fn for_profile(profile: &Profile) -> Self {
        Self::new(profile.warmup_offered_bps)
    }

    pub fn new(warmup_bps: u64) -> Self {
        Self {
            target: f64::from(u32::try_from(warmup_bps).expect("scenario rates fit u32"))
                * scenarios::WARMUP_SETTLE_RATIO,
            previous_second: 0,
            consecutive: 0,
        }
    }

    pub fn observe(&mut self, second: u64, bps: f64) -> bool {
        if second != self.previous_second + 1 {
            self.consecutive = 0;
        }
        self.previous_second = second;
        self.consecutive = if bps.is_finite() && bps >= self.target {
            self.consecutive + 1
        } else {
            0
        };
        self.consecutive >= scenarios::WARMUP_SETTLE_SECONDS
    }
}
