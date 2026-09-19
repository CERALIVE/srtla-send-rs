use super::*;
use crate::mode::SchedulingMode;
use crate::sender::selection::select_connection_idx_with_state;

#[tokio::test]
async fn adaptive_dispatch_ignores_legacy_stall_wrapper() {
    // Given a DATA-healthy link that the old keepalive-based wrapper would mask.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].in_flight_packets = 32;
    conns[0].last_stall_reprobe_ms = 10_000;
    conns[1].window = 1;
    let cfg = ConfigSnapshot {
        mode: "enhanced".parse().unwrap(),
        stall_deselect: true,
        ..config()
    };
    let mut state = AdaptiveState::default();
    // When the real mode dispatcher runs, then adaptive owns admission unchanged.
    let selected = select_connection_idx_with_state(&mut conns, None, 0, 10_000, &cfg, &mut state);
    assert_eq!(selected, Some(0));
    assert_eq!(state.targets.len(), 2);
    assert_eq!(conns[0].last_stall_reprobe_ms, 10_000);
}

#[test]
fn adaptive_mode_selects_arrival_scoped_ack_policy() {
    // Given adaptive mode with the legacy flag enabled too.
    let cfg = ConfigSnapshot {
        mode: "enhanced".parse::<SchedulingMode>().unwrap(),
        earned_ack_window: true,
        ..config()
    };
    // When deriving ACK policy, then probes cannot receive legacy window growth.
    assert!(matches!(
        crate::sender::ack::AckPolicy::from_config(&cfg),
        crate::sender::ack::AckPolicy::Adaptive
    ));
}
