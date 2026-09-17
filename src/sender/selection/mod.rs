//! Shared signal admission followed by mode-specific ranking.

pub mod adaptive;
pub(crate) mod admission;
pub mod blest;
mod classic;
pub mod edpf;
mod enhanced;
mod exploration;
mod features;
pub mod iods;
#[cfg(test)]
mod legacy;
mod quality;
mod shared;
mod signals;
#[cfg(test)]
pub use legacy::{select_by_mode, select_connection_idx};
#[cfg(feature = "test-internals")]
pub mod rtt_threshold;
#[cfg(not(feature = "test-internals"))]
mod rtt_threshold;

use adaptive::ranking::Selection;
pub use features::SchedulerFeatures;
pub use quality::calculate_quality_multiplier;
pub use shared::SchedulerShared;

use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::mode::SchedulingMode;

pub const MIN_SWITCH_INTERVAL_MS: u64 = 15;

#[derive(Debug, Default)]
pub struct EdpfSchedulerState {
    pub(crate) blest: blest::BlestFilter,
    pub(crate) iods: iods::IodsFilter,
}

#[inline(always)]
pub fn select_connection_idx_with_state(
    conns: &mut [SrtlaConnection],
    last_idx: Option<usize>,
    last_switch_time_ms: u64,
    current_time_ms: u64,
    config: &ConfigSnapshot,
    edpf_state: &mut EdpfSchedulerState,
    shared: &mut SchedulerShared,
) -> Option<usize> {
    let mut admission = admission::admit(conns, current_time_ms, config, shared);
    let ranking = adaptive::ranking::refresh(conns, shared, current_time_ms, &mut admission);
    select_by_mode_with_state(
        conns,
        last_idx,
        last_switch_time_ms,
        current_time_ms,
        config,
        edpf_state,
        ranking,
    )
}

#[inline(always)]
fn select_by_mode_with_state(
    conns: &mut [SrtlaConnection],
    last_idx: Option<usize>,
    last_switch_time_ms: u64,
    current_time_ms: u64,
    config: &ConfigSnapshot,
    edpf_state: &mut EdpfSchedulerState,
    ranking: Selection,
) -> Option<usize> {
    let ranking = match ranking {
        Selection::Carrier(index) => return index,
        ranked @ Selection::Ranked { .. } => ranked,
    };
    match config.mode {
        SchedulingMode::Classic => classic::select_connection(conns),
        SchedulingMode::Enhanced => enhanced::select_connection(
            conns,
            last_idx,
            last_switch_time_ms,
            current_time_ms,
            config.effective_exploration_enabled(),
        ),
        SchedulingMode::RttThreshold => rtt_threshold::select_connection(
            conns,
            last_idx,
            last_switch_time_ms,
            current_time_ms,
            config.rtt_delta_ms,
        ),
        SchedulingMode::Edpf => edpf_pipeline_select(conns, config, edpf_state),
        SchedulingMode::Adaptive => adaptive::select_ranked(
            conns,
            last_idx,
            last_switch_time_ms,
            current_time_ms,
            ranking,
        ),
    }
}

fn edpf_pipeline_select(
    conns: &[SrtlaConnection],
    _config: &ConfigSnapshot,
    edpf_state: &mut EdpfSchedulerState,
) -> Option<usize> {
    use edpf::SRT_PKT_SIZE;
    let EdpfSchedulerState { blest, iods } = edpf_state;
    let candidates = with_congestion_escape(conns, blest.filter(conns));
    let ordered = iods.filter_valid(&candidates, |idx| {
        edpf::arrival_time(&conns[idx], SRT_PKT_SIZE)
    });
    if ordered.is_empty() {
        iods.reset();
        return edpf::select_from_indices(conns, &candidates, SRT_PKT_SIZE)
            .or_else(|| edpf::select_from(conns, SRT_PKT_SIZE));
    }
    if let Some(idx) = edpf::select_from_indices(conns, &ordered, SRT_PKT_SIZE) {
        if let Some(arrival) = edpf::arrival_time(&conns[idx], SRT_PKT_SIZE) {
            iods.record_scheduled(arrival);
        }
        return Some(idx);
    }
    edpf::select_from_indices(conns, &candidates, SRT_PKT_SIZE)
        .or_else(|| edpf::select_from(conns, SRT_PKT_SIZE))
}

// BLEST's static OWD guard must not starve a slower link that will arrive first.
fn with_congestion_escape(conns: &[SrtlaConnection], admitted: Vec<usize>) -> Vec<usize> {
    use edpf::SRT_PKT_SIZE;
    let Some(best_admitted) = admitted
        .iter()
        .filter_map(|&i| edpf::arrival_time(&conns[i], SRT_PKT_SIZE))
        .reduce(f64::min)
    else {
        return admitted;
    };
    let mut candidates = admitted;
    for (i, conn) in conns.iter().enumerate() {
        if candidates.contains(&i) {
            continue;
        }
        if edpf::arrival_time(conn, SRT_PKT_SIZE).is_some_and(|a| a < best_admitted) {
            candidates.push(i);
        }
    }
    candidates.sort_unstable();
    candidates
}

#[cfg(test)]
mod tests;
