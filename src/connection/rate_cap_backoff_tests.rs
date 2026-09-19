use super::{ClimbMode, RateState, Rig, close};

#[test]
fn backoff_requires_loss_threshold_and_thirty_percent_load() {
    // Given both boundaries, with all delay conditions clear.
    for (loss, delivered, expected) in [
        (0.015, 300_000, 850_000.0),
        (0.015, 299_996, 1_020_000.0),
        (0.01499, 300_000, 1_020_000.0),
    ] {
        let mut rig = Rig::new();
        rig.tick(1_000_000);
        rig.signals.loss_ewma = Some(loss);
        // When a loaded loss observation arrives.
        let target = rig.tick(delivered);
        // Then neither a sub-threshold loss nor a sub-floor load can cut.
        close(target, expected);
    }
}

#[test]
fn backoff_floor_is_ack_delivered_clamped_to_previous_target() {
    // Given delivery below the cut, above it, and above the previous target.
    for (delivered, expected) in [
        (500_000, 850_000.0),
        (900_000, 900_000.0),
        (2_000_000, 1_000_000.0),
    ] {
        let mut rig = Rig::new();
        rig.tick(1_000_000);
        rig.signals.loss_ewma = Some(0.02);
        // When backing off.
        let target = rig.tick(delivered);
        // Then max(.85*prev, min(delivered, prev)) neither grows nor overcuts.
        close(target, expected);
        assert_eq!(rig.cap.state(), RateState::BackingOff);
    }
}

#[test]
fn backoff_cuts_each_of_three_ticks_before_testing_efficacy() {
    // Given persistent loaded loss after bootstrap.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.loss_ewma = Some(0.02);
    // When three complete backoff ticks elapse.
    let trace = [rig.tick(300_000), rig.tick(300_000), rig.tick(300_000)];
    // Then all three ticks apply their cuts before the next efficacy observation.
    for (actual, expected) in trace.into_iter().zip([850_000.0, 722_500.0, 614_125.0]) {
        close(actual, expected);
    }
}

#[test]
fn ineffective_backoff_latches_thirty_ticks_including_delay_only_cuts() {
    // Given three ineffective cuts.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.loss_ewma = Some(0.02);
    for _ in 0..3 {
        rig.tick(300_000);
    }
    // When loss stays exactly 80% of entry, then delay-only episodes alternate.
    rig.signals.loss_ewma = Some(0.016);
    for tick in 0..30 {
        if tick > 0 {
            rig.signals.loss_ewma = Some(0.0);
            rig.signals.srtt_ms = if tick % 2 == 0 { 200.0 } else { 160.0 };
        }
        let previous = rig.cap.target_bps();
        rig.tick(300_000);
        // Then no rate cut occurs anywhere in the full latch window.
        assert!(rig.cap.target_bps() >= previous, "latch tick {tick}");
        assert!(rig.cap.is_uncongestive());
    }
    rig.signals.loss_ewma = Some(0.02);
    let previous = rig.cap.target_bps();
    rig.tick(300_000);
    assert!(!rig.cap.is_uncongestive());
    close(rig.cap.target_bps(), previous * 0.85);
}

#[test]
fn effective_backoff_below_eighty_percent_continues_without_latch() {
    // Given loss 4% on entry with three completed cuts.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.loss_ewma = Some(0.04);
    for _ in 0..3 {
        rig.tick(300_000);
    }
    rig.signals.loss_ewma = Some(0.031);
    // When loss has fallen below 80% but remains above the entry threshold.
    let target = rig.tick(300_000);
    // Then congestion response continues, rather than suppressing effective cuts.
    close(target, 522_006.25);
    assert!(!rig.cap.is_uncongestive());
}

#[test]
fn b5_cellular_loss_below_threshold_never_backs_off() {
    // Given persistent 0.8% cellular loss, the historical 30-tick failure trace.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.loss_ewma = Some(0.008);
    // When the fully loaded link continues delivering.
    for _ in 0..30 {
        let previous = rig.cap.target_bps();
        rig.tick(2_000_000);
        // Then every tick climbs instead of cutting the target.
        close(rig.cap.target_bps(), previous * 1.02);
        assert_eq!(
            rig.cap.state(),
            RateState::Climbing {
                sub: ClimbMode::Normal
            }
        );
    }
}

#[test]
fn persistent_loss_cannot_cut_during_the_full_uncongestive_latch() {
    // Given three failed backoff ticks with continuing 2% loss.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.loss_ewma = Some(0.02);
    for _ in 0..3 {
        rig.tick(300_000);
    }
    rig.signals.srtt_ms = 160.0;
    // When the link remains loaded for the next thirty ticks.
    for tick in 0..30 {
        let previous = rig.cap.target_bps();
        rig.tick(300_000);
        // Then the latch remains active and the target cannot decrease.
        assert!(rig.cap.is_uncongestive(), "latch tick {tick}");
        assert!(rig.cap.target_bps() >= previous);
    }
}
