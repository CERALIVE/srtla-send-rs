use std::time::Duration;

use anyhow::{Result, ensure};

use crate::harness::SrtProfile;
use crate::profile::{Action, LinkProfile, Profile as Timeline};

pub const WARMUP_SETTLE_RATIO: f64 = 0.9;
pub const WARMUP_SETTLE_SECONDS: u32 = 3;
pub const WARMUP_TIMEOUT: Duration = Duration::from_secs(30);

/// Source/stack settings surrounding the unchanged temporal scheduler profile.
/// The runner starts source traffic AFTER all-link registration but BEFORE settling,
/// retains ten seconds of prehistory, then starts the measurement-relative timeline.
#[derive(Debug, Clone)]
pub struct Profile {
    pub timeline: Timeline,
    pub offered_bps: u64,
    pub warmup_offered_bps: u64,
    pub srt_profile: SrtProfile,
    pub source_ramp: Option<SourceRamp>,
    pub receiver_restart_budget: Option<Duration>,
}

/// A source-control ramp attached to ONE OfferedRate boundary. Intermediate source
/// control writes must not create extra LoadIntervals or restart the SRT source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRamp {
    pub at: Duration,
    pub duration: Duration,
    pub from_bps: u64,
    pub to_bps: u64,
}

impl Profile {
    pub(super) fn new(links: Vec<LinkProfile>, offered_bps: u64, seconds: u64) -> Self {
        Self {
            timeline: Timeline {
                links,
                events: vec![super::edge(
                    0,
                    None,
                    Action::OfferedRate { bps: offered_bps },
                )],
                duration: Duration::from_secs(seconds),
            },
            offered_bps,
            warmup_offered_bps: offered_bps,
            srt_profile: SrtProfile::PRODUCTION,
            source_ramp: None,
            receiver_restart_budget: None,
        }
    }

    /// Configured TBF capacity; pass this explicitly to the frozen LoadInterval evaluator.
    pub fn aggregate_capacity_bps(&self) -> u64 {
        self.timeline
            .links
            .iter()
            .filter_map(|link| link.base.rate_kbit)
            .sum::<u64>()
            * 1_000
    }

    /// Validate full periodic expansion, source readiness and source-ramp bounds.
    pub fn validate(&self) -> Result<()> {
        self.timeline.validate()?;
        ensure!(
            self.timeline.duration >= Duration::from_secs(45),
            "measurement window below 45s"
        );
        ensure!(
            self.offered_bps > 0 && self.warmup_offered_bps > 0,
            "source rates must be positive"
        );
        ensure!(
            self.warmup_offered_bps <= self.aggregate_capacity_bps(),
            "warm-up exceeds capacity"
        );
        ensure!(
            self.timeline.events.iter().any(|e| e.at.is_zero()
                && e.action
                    == Action::OfferedRate {
                        bps: self.offered_bps
                    }),
            "missing t=0 offered-rate boundary"
        );
        if let Some(ramp) = self.source_ramp {
            ensure!(
                !ramp.duration.is_zero()
                    && ramp
                        .at
                        .checked_add(ramp.duration)
                        .is_some_and(|end| end <= self.timeline.duration),
                "source ramp exceeds duration"
            );
            ensure!(
                self.timeline.events.iter().any(
                    |e| e.at == ramp.at && e.action == Action::OfferedRate { bps: ramp.to_bps }
                ),
                "ramp requires an offered-rate boundary"
            );
        }
        Ok(())
    }
}
