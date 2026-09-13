mod admission;
mod features;
mod preference;
mod sole;
mod state;

pub use features::AdaptiveFeatures;
pub use preference::preference_multiplier;
pub use state::AdaptiveState;

use super::MIN_SWITCH_INTERVAL_MS;
use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::connection::health::HealthState;

/// The six-argument selector contract mirrors the legacy history/clock/config seam.
/// Health/rate ticks are separate: this path only updates admission and election state.
pub fn select(
    conns: &mut [SrtlaConnection],
    last_idx: Option<usize>,
    last_switch_ms: u64,
    now_ms: u64,
    cfg: &ConfigSnapshot,
    state: &mut AdaptiveState,
) -> Option<usize> {
    admission::snapshot(conns, state, now_ms);
    let held =
        u32::try_from(state.targets.iter().filter(|t| t.eligible()).count()).unwrap_or(u32::MAX);
    let mut best: Option<(usize, f64)> = None;
    let mut current_score = None;
    for (i, conn) in conns.iter_mut().enumerate() {
        let target = &state.targets[i];
        if matches!(target.health, HealthState::Down) || target.eligible() {
            continue;
        }
        let base = f64::from(conn.get_score());
        let quality = if cfg.effective_quality_enabled() {
            conn.get_cached_quality_multiplier(now_ms)
        } else {
            1.0
        };
        let ramp = if state.features.contains(AdaptiveFeatures::REJOIN) {
            conn.health.ramp_multiplier(now_ms, held)
        } else {
            1.0
        };
        let preference = if state.features.contains(AdaptiveFeatures::PREF) {
            preference_multiplier(conn.effective_priority(), conn.window, conn.health.state())
        } else {
            1.0
        };
        let cap = if state.features.contains(AdaptiveFeatures::RATECAP) {
            conn.rate_cap
                .soft_cap_multiplier(u32::try_from(conn.in_flight_packets).unwrap_or(0))
        } else {
            1.0
        };
        let score = base * quality * ramp * preference * cap;
        if Some(i) == last_idx {
            current_score = Some(score);
        }
        if best.is_none_or(|(_, previous)| score > previous) {
            best = Some((i, score));
        }
    }
    if let Some((best_idx, best_score)) = best {
        state.sole_carrier = None;
        state.sole_identity = None;
        // A score exists ONLY for an admitted current link: cooldown cannot bypass gates.
        if let (Some(last), Some(current)) = (last_idx, current_score)
            && last != best_idx
        {
            if now_ms.saturating_sub(last_switch_ms) < MIN_SWITCH_INTERVAL_MS {
                crate::ab_metrics::record_cooldown_hold();
                return Some(last);
            }
            if best_score < current * 1.10 {
                return Some(last);
            }
        }
        return Some(best_idx);
    }
    if state.features.contains(AdaptiveFeatures::SOLE) {
        if let Some(i) = sole::select(conns, state, now_ms) {
            state.targets[i].sole_carrier = true;
            return Some(i);
        }
    } else {
        state.sole_carrier = None;
        state.sole_identity = None;
    }
    // Rank connected links even if every hard/soft gate vetoed them; never drop a pool.
    conns
        .iter()
        .enumerate()
        .filter(|(_, c)| c.connected)
        .max_by(|(a, ca), (b, cb)| ca.get_score().cmp(&cb.get_score()).then(b.cmp(a)))
        .map(|(i, _)| i)
}
