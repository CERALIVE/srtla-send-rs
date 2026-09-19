use super::*;
use crate::config::DynamicConfig;
use crate::test_helpers::create_selection_test_connections as create_test_connections;

#[test]
fn test_select_connection_idx_empty() {
    let mut conns = Vec::new();
    assert_eq!(
        select_connection_idx(
            &mut conns,
            None,
            0,
            0,
            &DynamicConfig::new().snapshot(),
            &mut SchedulerShared::default(),
        ),
        None
    );
}

#[tokio::test]
async fn edpf_pipeline_mutates_caller_owned_state() {
    let mut connections = create_test_connections(2).await;
    for c in &mut connections {
        c.bitrate.current_bitrate_bps = 1_000_000.0;
        c.rtt.rtt_min_ms = 30.0;
    }
    let mut state = EdpfSchedulerState::default();
    let selected = edpf_pipeline_select(
        &mut connections,
        &DynamicConfig::new().snapshot(),
        &mut state,
    );
    assert!(selected.is_some());
    assert!(state.iods.last_arrival > 0.0);
}

#[tokio::test]
async fn edpf_two_states_are_independent() {
    let mut connections = create_test_connections(2).await;
    for c in &mut connections {
        c.bitrate.current_bitrate_bps = 1_000_000.0;
        c.rtt.rtt_min_ms = 30.0;
    }
    let mut state_a = EdpfSchedulerState::default();
    let state_b = EdpfSchedulerState::default();
    for _ in 0..3 {
        edpf_pipeline_select(
            &mut connections,
            &DynamicConfig::new().snapshot(),
            &mut state_a,
        );
    }
    assert!(state_a.iods.last_arrival > 0.0);
    assert_eq!(state_b.iods.last_arrival, 0.0);
}
