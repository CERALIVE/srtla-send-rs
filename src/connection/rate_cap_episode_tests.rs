use super::{ClimbMode, RateState, Rig, close};

#[test]
fn b2_idle_holds_target() {
    // Given bootstrapped and never-delivered paths (the 120-tick audit trace).
    for seed in [0, 2_000_000] {
        let mut rig = Rig::new();
        if seed > 0 {
            rig.tick(seed);
        }
        rig.signals.velocity_ms_per_update = 0.0;
        let target = rig.cap.target_bps();
        // When DATA delivery remains zero for 120 housekeeping ticks.
        for idle_tick in 1..=120 {
            rig.tick(0);
            // Then even a HAI-eligible idle path cannot inflate its target.
            close(rig.cap.target_bps(), target);
            assert_eq!(rig.cap.bdp_cap_suspended(), idle_tick >= 10);
            if idle_tick >= 10 {
                assert_eq!(rig.cap.bdp_cap_packets(100.0), u32::MAX);
                close(rig.cap.soft_cap_multiplier(u32::MAX), 1.0);
            }
        }
    }
}

#[test]
fn delivery_resumes_bdp_cap_after_idle_without_rebootstrap() {
    // Given an established controller suspended after ten idle ticks.
    let mut rig = Rig::new();
    rig.tick(2_000_000);
    for _ in 0..10 {
        rig.tick(0);
    }
    // When a small amount of real delivery resumes.
    let target = rig.tick(4);
    // Then the existing target climbs, rather than being reseeded to 1Mbps.
    close(target, 2_040_000.0);
    assert!(!rig.cap.bdp_cap_suspended());
    assert_eq!(rig.cap.bdp_cap_packets(100.0), 32);
    close(rig.cap.soft_cap_multiplier(64), 0.5);
}

#[test]
fn b3_fast_recovery_has_five_full_ticks() {
    // Given either way into recovery, with HAI also eligible afterward.
    for from_drain in [false, true] {
        let mut rig = Rig::new();
        rig.tick(1_000_000);
        rig.signals.srtt_ms = if from_drain { 200.0 } else { 100.0 };
        rig.signals.loss_ewma = Some(if from_drain { 0.0 } else { 0.02 });
        rig.tick(300_000);
        rig.signals.srtt_ms = 100.0;
        rig.signals.loss_ewma = Some(0.0);
        rig.signals.velocity_ms_per_update = 0.0;
        // When five recovery ticks and then one ordinary tick run.
        for remaining in (0..5).rev() {
            let previous = rig.cap.target_bps();
            rig.tick(300_000);
            // Then mode selection precedes decrement, including tick five.
            close(rig.cap.target_bps(), previous * 1.04);
            assert_eq!(
                rig.cap.state(),
                RateState::Climbing {
                    sub: ClimbMode::FastRecovery {
                        ticks_left: remaining
                    },
                }
            );
        }
        let previous = rig.cap.target_bps();
        rig.tick(300_000);
        close(rig.cap.target_bps(), previous * 1.06);
    }
}

#[test]
fn recovery_budget_survives_holding_and_idle_without_consumption() {
    // Given a loss episode followed by delay holding and zero delivery.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.loss_ewma = Some(0.02);
    rig.tick(300_000);
    rig.signals.loss_ewma = Some(0.0);
    rig.signals.srtt_ms = 160.0;
    for _ in 0..5 {
        rig.tick(300_000);
    }
    for _ in 0..5 {
        rig.tick(0);
    }
    rig.signals.srtt_ms = 100.0;
    // When recovery can finally climb.
    let previous = rig.cap.target_bps();
    rig.tick(300_000);
    // Then this is still the first of five full growth ticks.
    close(rig.cap.target_bps(), previous * 1.04);
    assert_eq!(
        rig.cap.state(),
        RateState::Climbing {
            sub: ClimbMode::FastRecovery { ticks_left: 4 },
        }
    );
}

#[test]
fn b4_drain_cuts_once_per_episode() {
    // Given sustained loss-free RTT inflation at exactly twice the baseline.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.srtt_ms = 200.0;
    // When the episode outlasts both the historical 11 ticks and the guard.
    for _ in 0..30 {
        rig.tick(300_000);
        // Then target remains the one-shot entry cut, with no compounding/reseed.
        close(rig.cap.target_bps(), 750_000.0);
        assert_eq!(rig.cap.state(), RateState::Drain);
    }
}

#[test]
fn drain_reentry_guard_blocks_cuts_until_ten_ticks_after_previous_cut() {
    // Given a first cut followed by repeated exits/reentries in the hold band.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.srtt_ms = 200.0;
    rig.tick(300_000);
    // When new episodes enter before and exactly at the ten-tick boundary.
    for elapsed in 1..=10 {
        rig.signals.srtt_ms = if elapsed % 2 == 0 { 200.0 } else { 160.0 };
        rig.tick(300_000);
        // Then early episodes do not cut, but a genuinely new eligible one does.
        close(
            rig.cap.target_bps(),
            if elapsed < 10 { 750_000.0 } else { 562_500.0 },
        );
    }
}

#[test]
fn drain_requires_known_zero_loss_and_valid_baseline() {
    // Given low nonzero/unknown loss, and unknown RTT baseline.
    for (loss, baseline) in [(Some(0.008), 100.0), (None, 100.0), (Some(0.0), 0.0)] {
        let mut rig = Rig::new();
        rig.tick(1_000_000);
        rig.signals.srtt_ms = 200.0;
        rig.signals.rtt_min_ms = baseline;
        rig.signals.loss_ewma = loss;
        // When ticking at an inflated or unmeasured RTT.
        let target = rig.tick(300_000);
        // Then missing evidence is not proof of loss-free congestion.
        assert!(target >= 1_000_000.0);
    }
}

#[test]
fn idle_does_not_rearm_a_sustained_drain_episode() {
    // Given a drain cut followed by an idle gap longer than its reentry guard.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    rig.signals.srtt_ms = 200.0;
    rig.tick(300_000);
    for _ in 0..20 {
        rig.tick(0);
    }
    // When DATA resumes with the same inflated RTT.
    let target = rig.tick(300_000);
    // Then the old episode is still one-shot, regardless of guard expiry.
    close(target, 750_000.0);
}
