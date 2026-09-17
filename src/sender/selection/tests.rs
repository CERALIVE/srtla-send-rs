use super::*;
use crate::test_helpers::create_selection_test_connections as create_test_connections;
use crate::utils::now_ms;

#[test]
fn test_select_connection_idx_classic() {
    // Test that classic mode always picks highest score, ignoring dampening
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut connections = rt.block_on(create_test_connections(3));

    connections[0].in_flight_packets = 5; // Lower score
    connections[1].in_flight_packets = 0; // Highest score
    connections[2].in_flight_packets = 10; // Lowest score

    let last_switch_time_ms = now_ms();
    let current_time_ms = last_switch_time_ms + 100; // Within cooldown

    let config = ConfigSnapshot {
        features: Default::default(),
        mode: SchedulingMode::Classic,
        quality_enabled: false,
        exploration_enabled: false,
        rtt_delta_ms: 30,
        earned_ack_window: false,
        stall_deselect: false,
        stall_min_in_flight: 32,
        stall_ack_stale_ms: 3000,
        stall_reprobe_ms: 1000,
    };

    // Classic mode should pick connection 1 (highest score) even during cooldown
    let result = select_connection_idx(
        &mut connections,
        Some(0),
        last_switch_time_ms,
        current_time_ms,
        &config,
        &mut EdpfSchedulerState::default(),
    );
    assert_eq!(
        result,
        Some(1),
        "Classic mode should pick highest score connection"
    );
}

#[test]
fn test_select_connection_idx_empty() {
    let mut conns: Vec<SrtlaConnection> = vec![];
    let config = ConfigSnapshot {
        features: Default::default(),
        mode: SchedulingMode::Enhanced,
        quality_enabled: false,
        exploration_enabled: false,
        rtt_delta_ms: 30,
        earned_ack_window: false,
        stall_deselect: false,
        stall_min_in_flight: 32,
        stall_ack_stale_ms: 3000,
        stall_reprobe_ms: 1000,
    };
    let result = select_connection_idx(
        &mut conns,
        None,
        0,
        0,
        &config,
        &mut EdpfSchedulerState::default(),
    );
    assert_eq!(result, None);
}

#[test]
fn edpf_pipeline_mutates_caller_owned_state() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut connections = rt.block_on(create_test_connections(2));
    for c in connections.iter_mut() {
        c.bitrate.current_bitrate_bps = 1_000_000.0;
        c.rtt.rtt_min_ms = 30.0;
    }

    let config = ConfigSnapshot {
        features: Default::default(),
        mode: SchedulingMode::Edpf,
        quality_enabled: false,
        exploration_enabled: false,
        rtt_delta_ms: 30,
        earned_ack_window: false,
        stall_deselect: false,
        stall_min_in_flight: 32,
        stall_ack_stale_ms: 3000,
        stall_reprobe_ms: 1000,
    };

    let mut state = EdpfSchedulerState::default();
    let selected = select_connection_idx(&mut connections, None, 0, 0, &config, &mut state);

    assert!(selected.is_some(), "EDPF should select a connection");
    assert!(
        state.iods.last_arrival > 0.0,
        "the caller-owned IoDS state must be mutated by the pipeline"
    );
}

#[test]
fn edpf_two_states_are_independent() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut connections = rt.block_on(create_test_connections(2));
    for c in connections.iter_mut() {
        c.bitrate.current_bitrate_bps = 1_000_000.0;
        c.rtt.rtt_min_ms = 30.0;
    }

    let config = ConfigSnapshot {
        features: Default::default(),
        mode: SchedulingMode::Edpf,
        quality_enabled: false,
        exploration_enabled: false,
        rtt_delta_ms: 30,
        earned_ack_window: false,
        stall_deselect: false,
        stall_min_in_flight: 32,
        stall_ack_stale_ms: 3000,
        stall_reprobe_ms: 1000,
    };

    let mut state_a = EdpfSchedulerState::default();
    let state_b = EdpfSchedulerState::default();

    for _ in 0..3 {
        let _ = select_connection_idx(&mut connections, None, 0, 0, &config, &mut state_a);
    }

    assert!(
        state_a.iods.last_arrival > 0.0,
        "state_a should accumulate scheduled arrival history"
    );
    assert_eq!(
        state_b.iods.last_arrival, 0.0,
        "state_b is untouched and must remain at its Default"
    );
    assert!(
        state_a.iods.last_arrival != state_b.iods.last_arrival,
        "two separate EdpfSchedulerState instances must diverge independently"
    );
}
