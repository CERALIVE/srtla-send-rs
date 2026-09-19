//! Shared signal admission followed by mode-specific ranking.

pub mod adaptive;
pub(crate) mod admission;
pub mod blest;
pub mod edpf;
mod enhanced;
mod exploration;
mod features;
pub mod iods;
mod quality;
mod shared;
mod signals;

use adaptive::ranking::Selection;
pub use features::SchedulerFeatures;
pub use quality::calculate_quality_multiplier;
#[cfg(test)]
pub use select_connection_idx_with_state as select_connection_idx;
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
    ranking: Selection,
) -> Option<usize> {
    match ranking {
        Selection::Carrier(index) => return index,
        Selection::Ranked => {}
    }
    match config.mode {
        SchedulingMode::Enhanced => enhanced::select_connection(
            conns,
            last_idx,
            last_switch_time_ms,
            current_time_ms,
            config.effective_exploration_enabled(),
        ),
    }
}

pub fn edpf_pipeline_select(
    conns: &mut [SrtlaConnection],
    config: &ConfigSnapshot,
    edpf_state: &mut EdpfSchedulerState,
) -> Option<usize> {
    let mut shared = SchedulerShared::default();
    let mut admission = admission::admit(conns, 0, config, &mut shared);
    match adaptive::ranking::refresh(conns, &mut shared, 0, &mut admission) {
        Selection::Carrier(index) => return index,
        Selection::Ranked => {}
    }
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
