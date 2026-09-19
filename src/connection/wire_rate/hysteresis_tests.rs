use super::tests::input;
use super::{WireRateEstimator, WireRateInput, WireRatePhase, WireRttSample};

fn high(now_ms: u64, sent_ms: u64) -> WireRateInput {
    WireRateInput {
        now_ms,
        latest_rtt: Some(WireRttSample {
            observed_ms: now_ms,
            rtt_ms: now_ms.saturating_sub(sent_ms) as f64,
        }),
        queue_delay_ms: 8.0,
        ..input(now_ms, now_ms * 100, 8.0)
    }
}

fn after_first_cut() -> WireRateEstimator {
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    estimator.update(&input(1010, 1000, 8.0));
    assert_eq!(estimator.rate_bps(), 500_000.0);
    estimator
}

fn after_drain_resumes_search() -> WireRateEstimator {
    let mut estimator = after_first_cut();
    for now_ms in [1020, 1030, 1040] {
        estimator.update(&input(now_ms, 1000, 0.0));
    }
    assert_eq!(estimator.phase(), WireRatePhase::Searching);
    estimator
}

#[test]
fn no_second_cut_before_two_seconds() {
    // Given one immediate congestion cut.
    let mut estimator = after_first_cut();
    // When high causal samples persist until one millisecond before the repeat boundary.
    for now_ms in [2010, 2500, 3009] {
        estimator.update(&high(now_ms, now_ms - 60));
    }
    // Then the rate and bracket remain unchanged.
    assert_eq!(estimator.rate_bps(), 500_000.0);
    assert_eq!(estimator.blocked, Some(1_000_000.0));
}

#[test]
fn sustained_queue_permits_exactly_one_repeat_at_boundary() {
    // Given one immediate cut followed by post-settle high queue evidence.
    let mut estimator = after_first_cut();
    // When continuous causal samples span the full confirmation second.
    for now_ms in [2010, 2510, 3010] {
        estimator.update(&high(now_ms, now_ms - 60));
    }
    // Then exactly one repeat cut occurs at the two-second boundary.
    assert_eq!(estimator.rate_bps(), 250_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
}

#[test]
fn underused_draining_epoch_cannot_repeat_cut() {
    // Given one immediate cut followed by persistent high queue at an underused candidate.
    let mut estimator = after_first_cut();
    // When current-epoch accepted wire rate stays below full utilization.
    for (now_ms, queue_delay_ms) in [(2010, 8.0), (2510, 9.0), (3010, 10.0)] {
        estimator.update(&WireRateInput {
            accepted_bytes: Some(1000),
            ..input(now_ms, 1000, queue_delay_ms)
        });
    }
    // Then stale queue pressure cannot drive another cut.
    assert_eq!(estimator.rate_bps(), 500_000.0);
}

#[test]
fn unproven_draining_epoch_cannot_repeat_cut() {
    // Given one immediate cut followed by fully utilized high queue.
    let mut estimator = after_first_cut();
    // When no original DATA delivery proves the current epoch.
    for (now_ms, queue_delay_ms) in [(2010, 8.0), (2510, 9.0), (3010, 10.0)] {
        estimator.update(&WireRateInput {
            latest_original_delivery_ms: None,
            ..input(now_ms, now_ms * 100, queue_delay_ms)
        });
    }
    // Then accepted bytes alone cannot authorize another cut.
    assert_eq!(estimator.rate_bps(), 500_000.0);
}

#[test]
fn falling_queue_during_draining_cannot_repeat_cut() {
    // Given one immediate cut and fully utilized, proven post-cut epochs.
    let mut estimator = after_first_cut();
    // When persistent queue evidence falls across the confirmation interval.
    for (now_ms, queue_delay_ms) in [(2010, 10.0), (2510, 9.0), (3010, 8.0)] {
        estimator.update(&input(now_ms, now_ms * 100, queue_delay_ms));
    }
    // Then the falling queue cannot authorize another cut.
    assert_eq!(estimator.rate_bps(), 500_000.0);
}

#[test]
fn underused_search_after_draining_cannot_bypass_repeat_cut_gate() {
    // Given a post-cut estimator that drained and resumed Searching.
    let mut estimator = after_drain_resumes_search();
    // When due epochs have continuous high queue but no accepted wire utilization.
    for (now_ms, queue_delay_ms) in [(2040, 8.0), (2540, 9.0), (3040, 10.0)] {
        estimator.update(&WireRateInput {
            accepted_bytes: Some(1000),
            ..input(now_ms, 1000, queue_delay_ms)
        });
    }
    // Then the phase transition cannot bypass repeat-cut qualification.
    assert_eq!(estimator.rate_bps(), 500_000.0);
}

#[test]
fn fully_used_search_after_draining_can_repeat_cut() {
    // Given a post-cut estimator that drained and resumed Searching.
    let mut estimator = after_drain_resumes_search();
    // When due epochs have proven, fully-used and non-falling queue evidence.
    for (now_ms, accepted_bytes, queue_delay_ms) in [
        (2040, 63_500, 8.0),
        (2540, 94_750, 9.0),
        (3040, 126_000, 10.0),
    ] {
        estimator.update(&input(now_ms, accepted_bytes, queue_delay_ms));
    }
    // Then the centralized repeat-cut gate still permits qualified congestion.
    assert_eq!(estimator.rate_bps(), 250_000.0);
}

#[test]
fn recurring_half_second_waveform_never_confirms_repeat_congestion() {
    // Given one cut and D-shaped 215ms spike plus 285ms capacity-dip pulses.
    let mut estimator = after_first_cut();
    // When each half-second pulse clears before another pulse fifteen seconds later.
    for onset in [2010, 17_010, 32_010] {
        estimator.update(&high(onset, onset - 60));
        estimator.update(&high(onset + 215, onset + 155));
        estimator.update(&input(onset + 500, (onset + 500) * 100, 0.0));
    }
    // Then no recurring pulse poisons the bracket with a repeat cut.
    assert_eq!(estimator.rate_bps(), 500_000.0);
    assert_eq!(estimator.blocked, Some(1_000_000.0));
}

#[test]
fn absorbed_detection_preserves_both_bracket_endpoints() {
    // Given a bracketed estimator after its immediate cut.
    let mut estimator = after_first_cut();
    estimator.highest_clean = Some(400_000.0);
    let endpoints = (estimator.highest_clean, estimator.blocked);
    // When congestion is observed but has not met repeat persistence.
    estimator.update(&high(2010, 1950));
    estimator.update(&high(2510, 2450));
    // Then suppression changes neither endpoint.
    assert_eq!((estimator.highest_clean, estimator.blocked), endpoints);
}

#[test]
fn generation_rollover_rearms_immediate_first_cut() {
    // Given a generation that already consumed its immediate cut.
    let mut estimator = after_first_cut();
    // When a new delivery generation starts and immediately reports congestion.
    estimator.update(&WireRateInput {
        delivery_generation: 1,
        ..input(2000, 0, 0.0)
    });
    estimator.update(&WireRateInput {
        delivery_generation: 1,
        ..input(2010, 1000, 8.0)
    });
    // Then the new generation may take its own immediate first cut.
    assert_eq!(estimator.rate_bps(), 250_000.0);
}

#[test]
fn pre_cut_rtt_cannot_confirm_repeat_congestion() {
    // Given one cut and RTTs accepted before that rate generation.
    let mut estimator = after_first_cut();
    // When their delayed observations span the complete repeat interval.
    for now_ms in [2010, 2510, 3010] {
        estimator.update(&high(now_ms, 1000));
    }
    // Then old-tail evidence cannot authorize another cut.
    assert_eq!(estimator.rate_bps(), 500_000.0);
}
