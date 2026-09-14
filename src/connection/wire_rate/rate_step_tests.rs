use std::collections::VecDeque;

use super::tests::input;
use super::{WireRateEstimator, WireRateInput, WireRatePhase, WireRttSample};

#[test]
fn unbracketed_candidate_can_grow_after_rtt_sized_evidence() {
    // Given a fresh unbracketed candidate with continuous clear evidence.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    estimator.queue.clear = true;
    estimator.queue.clear_since_ms = Some(1000);
    // When the RTT-sized epoch reaches its exact boundary.
    estimator.update(&input(1199, 24_875, 0.0));
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    estimator.update(&input(1200, 25_000, 0.0));
    // Then initial exploration advances without waiting a full second.
    assert_eq!(estimator.rate_bps(), 2_000_000.0);
}

#[test]
fn congestion_history_keeps_one_second_floor_after_blocked_clears() {
    // Given prior congestion whose mutable bracket was later cleared.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    estimator.last_cut_ms = Some(1000);
    estimator.blocked = None;
    estimator.queue.clear = true;
    estimator.queue.clear_since_ms = Some(1000);
    estimator.restart_epoch(&input(1000, 0, 0.0));
    // When utilization reaches the RTT-sized boundary but not the post-cut floor.
    estimator.update(&input(1999, 124_875, 0.0));
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    estimator.update(&input(2000, 125_000, 0.0));
    // Then congestion history keeps the full one-second qualification regime.
    assert_eq!(estimator.rate_bps(), 2_000_000.0);
}

#[test]
fn pre_epoch_rtt_cannot_certify_current_candidate_clean() {
    // Given a fully utilized candidate and an RTT accepted before its epoch.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    let stale = WireRateInput {
        latest_rtt: Some(WireRttSample {
            observed_ms: 2000,
            rtt_ms: 1100.0,
        }),
        queue_delay_ms: 0.0,
        ..input(2000, 125_000, 0.0)
    };
    // When that stale observation is the only evidence at the epoch boundary.
    estimator.update(&stale);
    // Then utilization alone cannot promote the candidate.
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    assert_eq!(estimator.highest_clean, None);
}

#[test]
fn delayed_fast_window_converges_to_one_mbit_lower_bound() {
    // Given a1Mbit bottleneck after a real overshoot and congestion cut.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(780, 0, 0.0));
    estimator.update(&input(780, 0, 0.0));
    estimator.update(&input(980, 25_000, 0.0));
    estimator.update(&input(1000, 30_000, 8.0));
    assert_eq!(estimator.last_cut_ms, Some(1000));
    assert_eq!(estimator.highest_clean, Some(1_000_000.0));
    assert_eq!(estimator.blocked, Some(2_000_000.0));
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
    let mut accepted_bytes = 30_000_u64;
    let mut backlog_bits = 0.0_f64;
    let mut window = VecDeque::from([0.0_f64]);
    // When active search runs with a one-second-delayed queue signal.
    for elapsed_ms in (20..=20_000).step_by(20) {
        let attempted_bps = estimator.rate_bps();
        accepted_bytes += (attempted_bps * 20.0 / 8000.0) as u64;
        backlog_bits = (backlog_bits + (attempted_bps - 1_000_000.0) * 0.02).max(0.0);
        window.push_back(backlog_bits / 1_000_000.0 * 1000.0);
        if window.len() > 50 {
            window.pop_front();
        }
        let visible_queue_ms = window.iter().copied().fold(f64::INFINITY, f64::min);
        estimator.update(&input(1000 + elapsed_ms, accepted_bytes, visible_queue_ms));
    }
    // Then the settled clean lower bound is within10% of physical capacity.
    assert_eq!(
        estimator.phase(),
        WireRatePhase::CapacityHeld,
        "rate={} low={:?} high={:?}",
        estimator.rate_bps(),
        estimator.highest_clean,
        estimator.blocked
    );
    assert!(
        (1_000_000.0 / 1.10..=1_000_000.0).contains(&estimator.rate_bps()),
        "rate={} low={:?} high={:?}",
        estimator.rate_bps(),
        estimator.highest_clean,
        estimator.blocked
    );
}

#[test]
fn capacity_reprobe_requires_current_rate_clear_sample() {
    // Given a capacity-held link whose old clear bit is true but has no current RTT sample.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    estimator.phase = WireRatePhase::CapacityHeld;
    estimator.highest_clean = Some(1_000_000.0);
    estimator.blocked = Some(1_050_000.0);
    estimator.queue.clear = true;
    estimator.queue.clear_since_ms = Some(1000);
    // When fully utilized accounting epochs pass for more than30seconds without RTT evidence.
    for n in 1..=31 {
        estimator.update(&WireRateInput {
            latest_rtt: None,
            ..input(1000 + n * 1000, n * 125_000, 0.0)
        });
    }
    // Then stale clear state cannot authorize the upward reprobe.
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::CapacityHeld);
}

#[test]
fn demand_limited_growth_rejects_the_queue_gray_zone() {
    // Given a fully utilized demand-limited rate with neither clear nor congested queue evidence.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    estimator.phase = WireRatePhase::DemandLimited;
    // When the accounting epoch completes in the queue threshold gap.
    estimator.update(&input(2000, 125_000, 4.0));
    // Then utilization alone cannot resume upward search.
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::DemandLimited);
}
