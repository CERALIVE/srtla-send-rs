//! Enhanced mode selection with score hysteresis and optional exploration.
//! The 15 ms MIN_SWITCH_INTERVAL_MS cooldown applies to exploration pacing only,
//! not to load-driven selection. Score hysteresis (10%) gates all switches.
//!
//! The unaccepted cooldown-removal candidate had tests that asserted per-packet
//! load feedback would override the incumbent hold. Those tests are deleted here
//! because the cooldown has been reverted. The golden trace tests in
//! `selection_mode_traces.rs` verify that all modes (classic, rtt-threshold, edpf,
//! adaptive) produce byte-identical decision streams with the restored cooldown.

use crate::config::DynamicConfig;
use crate::sender::selection::{SchedulerShared, select_connection_idx};
use crate::test_helpers::create_selection_test_connections as create_test_connections;
use crate::utils::now_ms;

#[tokio::test]
async fn enhanced_cooldown_holds_valid_incumbent_within_15ms() {
    // Given: last_idx = Some(0), connection 0 is valid and connected,
    // connection 1 has a strictly better score, but we're within the 15ms cooldown.
    let mut conns = create_test_connections(3).await;
    let mut cfg = DynamicConfig::new().snapshot();
    cfg.quality_enabled = false;
    cfg.exploration_enabled = false;

    // Set up scores: connection 1 is better (lower in_flight = higher score)
    conns[0].in_flight_packets = 10; // Lower score (incumbent)
    conns[1].in_flight_packets = 0; // Higher score (challenger)
    conns[2].in_flight_packets = 5; // Medium score

    let last_switch_time_ms = now_ms();
    let current_time_ms = last_switch_time_ms + 5; // 5ms after last switch (within 15ms cooldown)

    // When: selection runs with the incumbent still valid and within cooldown
    let selected = select_connection_idx(
        &mut conns,
        Some(0),
        last_switch_time_ms,
        current_time_ms,
        &cfg,
        &mut SchedulerShared::default(),
    );

    // Then: the incumbent is held despite the better score, because cooldown is active
    assert_eq!(
        selected,
        Some(0),
        "cooldown should hold the valid incumbent within 15ms"
    );
}

#[tokio::test]
async fn enhanced_cooldown_allows_switch_after_15ms_expires() {
    // Given: same setup as above, but now we're past the 15ms cooldown
    let mut conns = create_test_connections(3).await;
    let mut cfg = DynamicConfig::new().snapshot();
    cfg.quality_enabled = false;
    cfg.exploration_enabled = false;

    conns[0].in_flight_packets = 10; // Lower score (incumbent)
    conns[1].in_flight_packets = 0; // Higher score (challenger)
    conns[2].in_flight_packets = 5; // Medium score

    let last_switch_time_ms = now_ms();
    let current_time_ms = last_switch_time_ms + 20; // 20ms after last switch (past 15ms cooldown)

    // When: selection runs after cooldown expires
    let selected = select_connection_idx(
        &mut conns,
        Some(0),
        last_switch_time_ms,
        current_time_ms,
        &cfg,
        &mut SchedulerShared::default(),
    );

    // Then: the better connection is selected because cooldown has expired
    assert_eq!(
        selected,
        Some(1),
        "selection should switch to better connection after cooldown expires"
    );
}

#[tokio::test]
async fn enhanced_cooldown_does_not_apply_on_first_selection() {
    // Given: last_idx = None (no prior selection), so cooldown logic should not apply
    let mut conns = create_test_connections(3).await;
    let mut cfg = DynamicConfig::new().snapshot();
    cfg.quality_enabled = false;
    cfg.exploration_enabled = false;

    conns[0].in_flight_packets = 10;
    conns[1].in_flight_packets = 0; // Best score
    conns[2].in_flight_packets = 5;

    let current_time_ms = now_ms();

    // When: selection runs with no prior selection (last_idx = None)
    let selected = select_connection_idx(
        &mut conns,
        None,
        0,
        current_time_ms,
        &cfg,
        &mut SchedulerShared::default(),
    );

    // Then: the best connection is selected immediately (cooldown does not apply)
    assert_eq!(
        selected,
        Some(1),
        "first selection should pick best connection regardless of time"
    );
}

#[tokio::test]
async fn enhanced_cooldown_does_not_hold_invalid_incumbent() {
    // Given: last_idx = Some(0), but connection 0 is now disconnected/timed-out,
    // within the 15ms cooldown window
    let mut conns = create_test_connections(3).await;
    let mut cfg = DynamicConfig::new().snapshot();
    cfg.quality_enabled = false;
    cfg.exploration_enabled = false;

    conns[0].in_flight_packets = 10;
    conns[0].connected = false; // Connection 0 is now invalid
    conns[1].in_flight_packets = 0; // Best score
    conns[2].in_flight_packets = 5;

    let last_switch_time_ms = now_ms();
    let current_time_ms = last_switch_time_ms + 5; // Within cooldown, but incumbent is invalid

    // When: selection runs with an invalid incumbent within cooldown
    let selected = select_connection_idx(
        &mut conns,
        Some(0),
        last_switch_time_ms,
        current_time_ms,
        &cfg,
        &mut SchedulerShared::default(),
    );

    // Then: the best valid connection is selected (cooldown does not hold invalid incumbent)
    assert_eq!(
        selected,
        Some(1),
        "cooldown should not hold an invalid/disconnected incumbent"
    );
}

#[tokio::test]
async fn enhanced_cooldown_boundary_at_exactly_15ms() {
    // Given: last_idx = Some(0), connection 0 is valid,
    // current_time_ms - last_switch_time_ms == MIN_SWITCH_INTERVAL_MS exactly (15ms)
    // The comparison is `time_since_last_switch_ms < MIN_SWITCH_INTERVAL_MS`,
    // so exactly 15ms means NOT in cooldown (15 < 15 is false).
    let mut conns = create_test_connections(3).await;
    let mut cfg = DynamicConfig::new().snapshot();
    cfg.quality_enabled = false;
    cfg.exploration_enabled = false;

    conns[0].in_flight_packets = 10; // Lower score (incumbent)
    conns[1].in_flight_packets = 0; // Higher score (challenger)
    conns[2].in_flight_packets = 5;

    let last_switch_time_ms = now_ms();
    let current_time_ms = last_switch_time_ms + 15; // Exactly 15ms (boundary)

    // When: selection runs at exactly the 15ms boundary
    let selected = select_connection_idx(
        &mut conns,
        Some(0),
        last_switch_time_ms,
        current_time_ms,
        &cfg,
        &mut SchedulerShared::default(),
    );

    // Then: cooldown is NOT active (15 < 15 is false), so the better connection is selected
    assert_eq!(
        selected,
        Some(1),
        "at exactly 15ms, cooldown should be expired (< comparison), allowing switch"
    );
}
