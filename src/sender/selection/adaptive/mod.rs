mod admission;
mod features;
mod preference;
mod ranking;
mod sole;
mod state;

pub use features::AdaptiveFeatures;
pub use preference::preference_multiplier;
pub use state::AdaptiveState;

use super::MIN_SWITCH_INTERVAL_MS;
use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;

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
    let (best_idx, best_score) = match ranking::refresh(conns, state, now_ms, cfg) {
        ranking::Selection::Ranked { index, score } => (index, score),
        ranking::Selection::Carrier(index) => return index,
    };
    let current_score = last_idx
        .and_then(|i| conns.get(i))
        .and_then(|conn| conn.adaptive.weight)
        .map(|weight| weight.score());
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
    Some(best_idx)
}
