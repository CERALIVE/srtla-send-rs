use smallvec::SmallVec;

use super::AdaptiveFeatures;
use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::connection::probe::{ProbeScheduler, ProbeTarget};
use crate::mode::SchedulingMode;
use crate::stats::SharedStats;
use crate::utils::now_ms;

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
    /// Publish immediately after health/rate ticks, including idle startup. Legacy
    /// modes bypass adaptive policy completely and retain their previous stats path.
    pub(crate) fn update_stats(&mut self, conns: &mut [SrtlaConnection], cfg: &ConfigSnapshot) {
        match cfg.mode {
            SchedulingMode::Adaptive => {
                super::ranking::refresh(conns, self, now_ms(), cfg);
            }
            SchedulingMode::Classic
            | SchedulingMode::Enhanced
            | SchedulingMode::RttThreshold
            | SchedulingMode::Edpf => {}
        }
        self.stats.update(conns, cfg);
    }

    /// Feature bits come from `adaptive_env`, which resolves to
    /// `AdaptiveFeatures::default()` unless a `test-internals` build was given
    /// `SRTLA_ADAPTIVE_FEATURES` — the release binary and an unset-env
    /// `test-internals` binary therefore run the same set.
    pub fn new(stats: SharedStats) -> Self {
        Self {
            features: crate::adaptive_env::features(),
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
