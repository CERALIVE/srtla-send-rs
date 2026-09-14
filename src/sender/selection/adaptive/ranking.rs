use super::{AdaptiveFeatures, AdaptiveState, admission, preference_multiplier, sole};
use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::connection::adaptive::SelectionWeight;
use crate::connection::health::HealthState;

pub(super) enum Selection {
    Ranked { index: usize, score: f64 },
    Carrier(Option<usize>),
}

struct Ranking<'a> {
    now_ms: u64,
    config: &'a ConfigSnapshot,
    features: AdaptiveFeatures,
    held_links: u32,
}

/// Shared by DATA selection and the post-health-tick publication. It advances the
/// real deadline/election state, but never queues packets or changes switch history.
/// Four inputs keep the two mutable policy stores separate from config and clock.
pub(super) fn refresh(
    conns: &mut [SrtlaConnection],
    state: &mut AdaptiveState,
    now: u64,
    cfg: &ConfigSnapshot,
) -> Selection {
    admission::snapshot(conns, state, now);
    let held =
        u32::try_from(state.targets.iter().filter(|t| t.eligible()).count()).unwrap_or(u32::MAX);
    let ranking = Ranking {
        now_ms: now,
        config: cfg,
        features: state.features,
        held_links: held,
    };
    let mut best: Option<(usize, f64)> = None;
    for (i, conn) in conns.iter_mut().enumerate() {
        conn.adaptive.weight = None;
        let target = &state.targets[i];
        if matches!(target.health, HealthState::Down) || target.eligible() {
            continue;
        }
        let weight = ranking.weight(conn);
        let score = weight.score();
        conn.adaptive.weight = Some(weight);
        if best.is_none_or(|(_, previous)| score > previous) {
            best = Some((i, score));
        }
    }
    if let Some((index, score)) = best {
        state.sole_carrier = None;
        state.sole_identity = None;
        return Selection::Ranked { index, score };
    }
    if state.features.contains(AdaptiveFeatures::SOLE) {
        if let Some(i) = sole::select(conns, state, now) {
            state.targets[i].sole_carrier = true;
            conns[i].adaptive.weight = Some(ranking.weight(&mut conns[i]));
            return Selection::Carrier(Some(i));
        }
    } else {
        state.sole_carrier = None;
        state.sole_identity = None;
    }
    // Preserve the connected-pool escape and its base-only ordering. Only its
    // chosen carrier is admitted; soft-held neighbours must still publish zero.
    let fallback = conns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.connected)
        .max_by(|(a, ca), (b, cb)| ca.get_score().cmp(&cb.get_score()).then(b.cmp(a)))
        .map(|(i, _)| i);
    if let Some(i) = fallback {
        conns[i].adaptive.weight = Some(SelectionWeight {
            base_score: conns[i].get_score(),
            quality_multiplier: 1.0,
            effective_multiplier: 1.0,
        });
    }
    Selection::Carrier(fallback)
}

impl Ranking<'_> {
    fn weight(&self, conn: &mut SrtlaConnection) -> SelectionWeight {
        let quality = if self.config.effective_quality_enabled() {
            conn.get_cached_quality_multiplier(self.now_ms)
        } else {
            1.0
        };
        let ramp = if self.features.contains(AdaptiveFeatures::REJOIN) {
            conn.health.ramp_multiplier(self.now_ms, self.held_links)
        } else {
            1.0
        };
        let preference = if self.features.contains(AdaptiveFeatures::PREF) {
            preference_multiplier(conn.effective_priority(), conn.window, conn.health.state())
        } else {
            1.0
        };
        let cap = if self.features.contains(AdaptiveFeatures::RATECAP) {
            conn.rate_cap
                .soft_cap_multiplier(u32::try_from(conn.in_flight_packets).unwrap_or(0))
        } else {
            1.0
        };
        SelectionWeight {
            base_score: conn.get_score(),
            quality_multiplier: quality,
            effective_multiplier: quality * ramp * preference * cap,
        }
    }
}
