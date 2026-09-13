use smallvec::SmallVec;

use super::AdaptiveFeatures;
use crate::connection::probe::{ProbeScheduler, ProbeTarget};
use crate::stats::SharedStats;

/// Bond-wide state owned by one send loop. Per-link timers live on the connections.
pub struct AdaptiveState {
    pub features: AdaptiveFeatures,
    pub sole_carrier: Option<(usize, u64)>,
    pub probe: ProbeScheduler,
    /// Reused admission snapshot for the post-selection DATA probe opportunity.
    pub targets: SmallVec<ProbeTarget, 4>,
    pub(super) stats: SharedStats,
    pub(super) sole_identity: Option<(u64, u32)>,
}

impl AdaptiveState {
    pub fn new(stats: SharedStats) -> Self {
        Self {
            features: AdaptiveFeatures::ALL,
            sole_carrier: None,
            probe: ProbeScheduler::default(),
            targets: SmallVec::new(),
            stats,
            sole_identity: None,
        }
    }
}

impl Default for AdaptiveState {
    fn default() -> Self {
        Self::new(SharedStats::new())
    }
}
