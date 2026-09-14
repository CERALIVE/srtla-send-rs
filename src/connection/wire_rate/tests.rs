use super::{WireRateEstimator, WireRateInput, WireRatePhase, WireRttSample};

pub(super) fn input(now: u64, bytes: u64, raw_queue: f64) -> WireRateInput {
    WireRateInput {
        now_ms: now,
        delivery_generation: 0,
        accepted_bytes: Some(bytes),
        latest_original_delivery_ms: Some(now),
        latest_rtt: Some(WireRttSample {
            observed_ms: now,
            rtt_ms: 60.0 + 2.0 * raw_queue,
        }),
        slow_min_rtt_ms: 60.0,
        queue_delay_ms: raw_queue,
    }
}

fn simulate(capacity: f64, demand: f64, duration_ms: u64) -> WireRateEstimator {
    let mut estimator = WireRateEstimator::default();
    let mut bytes = 0;
    let mut backlog = 0.0_f64;
    estimator.update(&input(1000, bytes, 0.0));
    for elapsed in (20..=duration_ms).step_by(20) {
        let attempted = estimator.rate_bps().min(demand);
        bytes += (attempted * 20.0 / 8000.0) as u64;
        backlog = (backlog + (attempted - capacity) * 20.0 / 8000.0).max(0.0);
        estimator.update(&input(1000 + elapsed, bytes, backlog * 8000.0 / capacity));
    }
    estimator
}

#[test]
fn finds_queue_free_lower_bound_for_one_mbit() {
    // Given a1Mbit bottleneck and ample demand, when actively searching its boundary.
    let estimator = simulate(1_000_000.0, 20_000_000.0, 12_000);
    // Then the settled rate is a queue-free lower bound within10% of capacity.
    assert_eq!(estimator.phase(), WireRatePhase::CapacityHeld);
    assert!((1_000_000.0 / 1.10..=1_000_000.0).contains(&estimator.rate_bps()));
}

#[test]
fn finds_queue_free_lower_bound_for_five_mbit() {
    // Given a5Mbit bottleneck, when probing above it and refining the bracket.
    let estimator = simulate(5_000_000.0, 20_000_000.0, 12_000);
    // Then it discovers spare capacity instead of inheriting the1Mbit bootstrap.
    assert_eq!(estimator.phase(), WireRatePhase::CapacityHeld);
    assert!((5_000_000.0 / 1.10..=5_000_000.0).contains(&estimator.rate_bps()));
}

#[test]
fn eight_mbit_path_holds_demand_limited_rate_without_self_throttling() {
    // Given8Mbit capacity but only6.4Mbit demand, when unbracketed search runs for one second.
    let estimator = simulate(8_000_000.0, 6_400_000.0, 1000);
    // Then1→2→4→8 restores fast demand-limited exploration before any congestion cut.
    assert_eq!(estimator.rate_bps(), 8_000_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::DemandLimited);
}

#[test]
fn missing_original_proof_never_authorizes_growth() {
    // Given fully used wire credit but no current original delivery proof.
    for proof in [None, Some(999)] {
        let mut estimator = WireRateEstimator::default();
        estimator.update(&input(1000, 0, 0.0));
        // When many fully utilized epochs pass with only absent/stale original evidence.
        for n in 1..=20 {
            estimator.update(&WireRateInput {
                latest_original_delivery_ms: proof,
                ..input(1000 + n * 200, n * 25_000, 0.0)
            });
        }
        // Then accepted throughput alone cannot increase or decrease capacity.
        assert_eq!(estimator.rate_bps(), 1_000_000.0);
    }
}

#[test]
fn authorized_repeat_still_requires_three_new_clear_samples_to_drain() {
    // Given sustained queue that authorizes one repeat cut after settling.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    estimator.update(&input(1010, 1000, 8.0));
    assert_eq!(estimator.rate_bps(), 500_000.0);
    for now in [2010, 2510, 3010] {
        estimator.update(&input(now, now * 100, 8.0));
    }
    assert_eq!(estimator.rate_bps(), 250_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
    // When only two distinct clear measurements arrive.
    for now in [3020, 3030] {
        estimator.update(&input(now, 302_000, 0.0));
    }
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
    // Then only a third distinct clear measurement can leave Draining.
    estimator.update(&input(3040, 302_000, 0.0));
    assert_eq!(estimator.phase(), WireRatePhase::Searching);
    assert_eq!(estimator.rate_bps(), 250_000.0);
}

#[test]
fn generation_rollover_retains_rate_but_revalidates_new_path() {
    // Given a learned demand-limited8Mbit path.
    let mut estimator = simulate(8_000_000.0, 6_400_000.0, 5000);
    assert_eq!(estimator.rate_bps(), 8_000_000.0);
    // When delivery generation and accepted-byte counter restart.
    estimator.update(&WireRateInput {
        delivery_generation: 1,
        latest_original_delivery_ms: None,
        ..input(3000, 0, 0.0)
    });
    // Then rate survives, while the new path is validated rather than trusted blindly.
    assert_eq!(estimator.rate_bps(), 8_000_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Searching);
    estimator.update(&WireRateInput {
        delivery_generation: 1,
        ..input(3010, 1000, 8.0)
    });
    assert_eq!(estimator.rate_bps(), 4_000_000.0);
    assert_eq!(WireRateEstimator::default().rate_bps(), 1_000_000.0);
}
