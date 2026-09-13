//! Six-argument adapters keep frozen legacy traces on the production dispatch paths.

use super::{
    AdaptiveState, EdpfSchedulerState, select_by_mode_with_state, select_connection_idx_with_state,
};
use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;

pub fn select_connection_idx(
    conns: &mut [SrtlaConnection],
    last: Option<usize>,
    switched: u64,
    now: u64,
    config: &ConfigSnapshot,
    edpf: &mut EdpfSchedulerState,
) -> Option<usize> {
    select_connection_idx_with_state(
        conns,
        last,
        switched,
        now,
        config,
        edpf,
        &mut AdaptiveState::default(),
    )
}

pub fn select_by_mode(
    conns: &mut [SrtlaConnection],
    last: Option<usize>,
    switched: u64,
    now: u64,
    config: &ConfigSnapshot,
    edpf: &mut EdpfSchedulerState,
) -> Option<usize> {
    select_by_mode_with_state(
        conns,
        last,
        switched,
        now,
        config,
        edpf,
        &mut AdaptiveState::default(),
    )
}
