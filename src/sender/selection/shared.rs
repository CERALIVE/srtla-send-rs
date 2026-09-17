use smallvec::SmallVec;

use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::connection::probe::{ProbeScheduler, ProbeTarget};
use crate::stats::SharedStats;
use crate::utils::now_ms;

/// Bond-wide state owned by one send loop. Per-link timers live on the connections.
pub struct SchedulerShared {
    pub sole_carrier: Option<(usize, u64)>,
    pub probe: ProbeScheduler,
    /// Reused admission snapshot for the post-selection DATA probe opportunity.
    pub targets: SmallVec<ProbeTarget, 4>,
    pub(super) stats: SharedStats,
    pub(super) sole_identity: Option<(u64, u32)>,
}

impl SchedulerShared {
    /// Refresh shared admission before publishing, including idle startup.
    pub(crate) fn update_stats(&mut self, conns: &mut [SrtlaConnection], cfg: &ConfigSnapshot) {
        let now = now_ms();
        let mut admission = super::admission::admit(conns, now, cfg, self);
        super::adaptive::ranking::refresh(conns, self, now, &mut admission);
        self.stats.update(conns, cfg);
    }

    pub fn new(stats: SharedStats) -> Self {
        Self {
            sole_carrier: None,
            probe: ProbeScheduler::default(),
            targets: SmallVec::new(),
            stats,
            sole_identity: None,
        }
    }
}

impl Default for SchedulerShared {
    fn default() -> Self {
        Self::new(SharedStats::new())
    }
}
