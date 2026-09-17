//! Compatibility flags are inert; shared health admission governs selection.

use crate::connection::health::{HealthMachine, HealthState};
use crate::mode::SchedulingMode;
use crate::sender::selection::{
    EdpfSchedulerState, SchedulerShared, select_connection_idx_with_state,
};
use crate::tests::adaptive_tests::{config, pool_of};
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn stall_flag_and_tunables_are_ignored_by_every_mode() {
    // Given a superior stalled link and a healthy alternative, under both flag values.
    let _clock = TestClock::new(10_000);
    for mode in [
        SchedulingMode::Classic,
        SchedulingMode::Enhanced,
        SchedulingMode::RttThreshold,
        SchedulingMode::Edpf,
        SchedulingMode::Adaptive,
    ] {
        for enabled in [false, true] {
            let mut conns = pool_of(2).await;
            conns[0].health = HealthMachine::new(HealthState::Stalled, 0);
            conns[1].window = 1;
            let mut cfg = config();
            cfg.mode = mode;
            cfg.stall_deselect = enabled;
            cfg.stall_min_in_flight = i32::MAX;
            cfg.stall_ack_stale_ms = u64::MAX;
            cfg.stall_reprobe_ms = 0;
            // When the dispatcher sees a stall within the old incumbent cooldown.
            let selected = select_connection_idx_with_state(
                &mut conns,
                Some(0),
                10_000,
                10_001,
                &cfg,
                &mut EdpfSchedulerState::default(),
                &mut SchedulerShared::default(),
            );
            // Then shared admission excludes it without touching registration or the window.
            assert_eq!(selected, Some(1), "{mode}/{enabled}");
            assert!(conns[0].connected);
            assert!(!conns[0].is_timed_out());
            assert_eq!(conns[0].window, 20_000);
        }
    }
}

#[tokio::test]
async fn stale_legacy_signal_alone_does_not_override_healthy_admission() {
    // Given Healthy links with stale legacy ACK clocks and load above the old threshold.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(2).await;
    conns[0].in_flight_packets = 32;
    conns[0].window = 60_000;
    conns[1].window = 1;
    let mut cfg = config();
    cfg.stall_deselect = true;
    // When shared admission, not the retired mask, evaluates them.
    let selected = select_connection_idx_with_state(
        &mut conns,
        None,
        0,
        10_000,
        &cfg,
        &mut EdpfSchedulerState::default(),
        &mut SchedulerShared::default(),
    );
    // Then the high-capacity Healthy link remains usable despite the old signal.
    assert_eq!(selected, Some(0));
}

#[tokio::test]
async fn admission_all_stalled_rotates_on_proof_deadline_not_legacy_reprobe() {
    // Given two stalled links and a zero legacy reprobe interval.
    let clock = TestClock::new(10_000);
    let mut conns = pool_of(2).await;
    for (i, c) in conns.iter_mut().enumerate() {
        c.health = HealthMachine::new(HealthState::Stalled, 0);
        c.rtt.kalman_rtt.update(if i == 0 { 20.0 } else { 100.0 });
    }
    let mut cfg = config();
    cfg.stall_deselect = true;
    cfg.stall_reprobe_ms = 0;
    let mut shared = SchedulerShared::default();
    let mut edpf = EdpfSchedulerState::default();
    let mut trace = Vec::new();
    // When election, proof-wait and proof-expiry ticks are replayed sequentially.
    for now in [10_000, 10_100, 11_999, 12_000] {
        clock.set(now);
        trace.push(select_connection_idx_with_state(
            &mut conns,
            None,
            0,
            now,
            &cfg,
            &mut edpf,
            &mut shared,
        ));
    }
    // Then only the real proof deadline rotates the carrier.
    assert_eq!(trace, [Some(0), Some(0), Some(0), Some(1)]);
}

#[tokio::test]
async fn admission_recovered_link_returns_without_legacy_reprobe_delay() {
    // Given a previously held link that now has Healthy recovery state.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(2).await;
    conns[1].window = 1;
    conns[0].health = HealthMachine::new(HealthState::Healthy, 10_000);
    let mut cfg = config();
    cfg.stall_deselect = true;
    cfg.stall_reprobe_ms = u64::MAX;
    // When a packet arrives without waiting for any legacy reprobe timer.
    let selected = select_connection_idx_with_state(
        &mut conns,
        None,
        0,
        10_000,
        &cfg,
        &mut EdpfSchedulerState::default(),
        &mut SchedulerShared::default(),
    );
    // Then the recovered link immediately participates in ranking.
    assert_eq!(selected, Some(0));
}
