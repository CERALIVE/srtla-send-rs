use smallvec::SmallVec;

use super::adaptive::preference_multiplier;
use super::signals::snapshot;
use super::{SchedulerFeatures, SchedulerShared};
use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::connection::adaptive::SelectionWeight;
use crate::connection::health::HealthState;

pub(crate) struct Admission {
    pub eligible: SmallVec<usize, 4>,
    pub held: SmallVec<usize, 4>,
    pub held_links: u32,
    pub weight: SmallVec<Option<SelectionWeight>, 4>,
    pub fallback: bool,
    pub features: SchedulerFeatures,
    now_ms: u64,
}

// Snapshot before weighting: held links must not advance their 50ms quality cache.
pub(crate) fn admit(
    conns: &mut [SrtlaConnection],
    now_ms: u64,
    cfg: &ConfigSnapshot,
    shared: &mut SchedulerShared,
) -> Admission {
    let features = cfg.scheduler_features();
    snapshot(conns, shared, now_ms, features);
    let mut admission = Admission {
        eligible: SmallVec::new(),
        held: SmallVec::new(),
        held_links: 0,
        weight: smallvec::smallvec![None; conns.len()],
        fallback: false,
        features,
        now_ms,
    };
    for (i, conn) in conns.iter_mut().enumerate() {
        conn.adaptive.weight = None;
        let target = &shared.targets[i];
        if target.eligible() {
            admission.held.push(i);
        } else if !matches!(target.health, HealthState::Down) {
            admission.eligible.push(i);
        }
    }
    admission.held_links = u32::try_from(admission.held.len()).unwrap_or(u32::MAX);
    admission.fallback = admission.eligible.is_empty();
    if admission.fallback {
        admission.eligible.extend(
            conns
                .iter()
                .enumerate()
                .filter(|(_, c)| c.connected)
                .map(|(i, _)| i),
        );
    }
    admission
}

impl Admission {
    pub fn compute_weight(&mut self, index: usize, conn: &mut SrtlaConnection) -> SelectionWeight {
        if let Some(weight) = self.weight[index] {
            return weight;
        }
        let quality = if self.features.contains(SchedulerFeatures::QUALITY) {
            conn.get_cached_quality_multiplier(self.now_ms)
        } else {
            1.0
        };
        let ramp = if self.features.contains(SchedulerFeatures::REJOIN) {
            conn.health.ramp_multiplier(self.now_ms, self.held_links)
        } else {
            1.0
        };
        let preference = if self.features.contains(SchedulerFeatures::PREF) {
            preference_multiplier(conn.effective_priority(), conn.window, conn.health.state())
        } else {
            1.0
        };
        let cap = if self.features.contains(SchedulerFeatures::RATECAP) {
            conn.rate_cap
                .soft_cap_multiplier(u32::try_from(conn.in_flight_packets).unwrap_or(0))
        } else {
            1.0
        };
        let weight = SelectionWeight {
            base_score: conn.get_score(),
            quality_multiplier: quality,
            effective_multiplier: quality * ramp * preference * cap,
        };
        self.weight[index] = Some(weight);
        weight
    }
}

#[cfg(test)]
#[path = "admission_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "admission_feature_tests.rs"]
mod feature_tests;
