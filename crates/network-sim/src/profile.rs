//! Temporal impairment profiles and monotonic event execution.

mod action;
mod expansion;
pub mod qdisc;
mod runtime;
mod scheduler;
pub mod traffic;
use std::time::Duration;

use anyhow::{Result, ensure};
pub use runtime::{BondRuntime, ProcessEndpoints};
pub use scheduler::{EventLog, EventRecord, Scheduler};

use crate::bond::CarrierMode;
use crate::{ImpairmentConfig, Scenario, ScenarioConfig};

#[derive(Debug, Clone)]
pub struct Profile {
    pub links: Vec<LinkProfile>,
    pub events: Vec<TimedEvent>,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct LinkProfile {
    pub base: ImpairmentConfig,
    pub carrier: CarrierMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimedEvent {
    pub at: Duration,
    pub link: Option<usize>,
    pub action: Action,
    pub horizon: Duration,
    pub graded: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    SetImpairment(ImpairmentConfig),
    DataBlackhole {
        on: bool,
    },
    LinkUp(bool),
    DefaultRoute(bool),
    Replug,
    ReceiverRestart,
    SighupReorder(Vec<usize>),
    CrossTraffic {
        mbit: u32,
        on: bool,
    },
    OfferedRate {
        bps: u64,
    },
    /// `until` bounds onsets inclusively; `hold` restores the prior state.
    Periodic {
        every: Duration,
        action: Box<Action>,
        hold: Duration,
        until: Duration,
    },
}

impl TimedEvent {
    pub const fn new(at: Duration, link: Option<usize>, action: Action) -> Self {
        let horizon = match &action {
            Action::Periodic { .. } => Duration::from_secs(5),
            Action::OfferedRate { .. } | Action::CrossTraffic { .. } => Duration::ZERO,
            Action::SetImpairment(_)
            | Action::DataBlackhole { .. }
            | Action::LinkUp(_)
            | Action::DefaultRoute(_)
            | Action::Replug
            | Action::ReceiverRestart
            | Action::SighupReorder(_) => Duration::from_secs(30),
        };
        Self {
            at,
            link,
            action,
            horizon,
            graded: true,
        }
    }
}

impl Profile {
    pub fn validate(&self) -> Result<()> {
        self.expanded_events().map(|_| ())
    }

    /// Random-walk samples are ungraded instantaneous updates, with no recovery tail.
    /// The legacy generator's ceil-rounded final frame is clipped to the run duration.
    pub fn from_random_walk(config: ScenarioConfig) -> Result<Self> {
        ensure!(!config.step.is_zero(), "random-walk step must be positive");
        let duration = config.duration;
        let frames = Scenario::new(config).frames();
        let links = frames
            .first()
            .into_iter()
            .flat_map(|f| &f.configs)
            .map(|base| LinkProfile {
                base: base.clone(),
                carrier: CarrierMode::Direct,
            })
            .collect();
        let events = frames
            .into_iter()
            .filter(|f| f.t <= duration)
            .flat_map(|frame| {
                frame
                    .configs
                    .into_iter()
                    .enumerate()
                    .map(move |(link, config)| TimedEvent {
                        at: frame.t,
                        link: Some(link),
                        action: Action::SetImpairment(config),
                        horizon: Duration::ZERO,
                        graded: false,
                    })
            })
            .collect();
        let profile = Self {
            links,
            events,
            duration,
        };
        profile.validate()?;
        Ok(profile)
    }
}

#[cfg(test)]
mod edge_tests;
#[cfg(test)]
mod tests;
