use super::tests::input;
use super::{WireRateEstimator, WireRateInput, WireRatePhase, WireRttSample};

#[test]
fn fast_queue_requires_three_distinct_newer_measurements() {
    // Given no persistent-window queue but a rising sequence of raw RTT measurements.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    let first = WireRateInput {
        queue_delay_ms: 0.0,
        ..input(1010, 1000, 8.0)
    };
    estimator.update(&first);
    // When pacing polls repeat the same sample, it cannot become three observations.
    for now in 1011..1040 {
        estimator.update(&WireRateInput {
            now_ms: now,
            ..first
        });
    }
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    estimator.update(&WireRateInput {
        queue_delay_ms: 0.0,
        ..input(1040, 2000, 9.0)
    });
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    estimator.update(&WireRateInput {
        queue_delay_ms: 0.0,
        ..input(1070, 3000, 10.0)
    });
    // Then the third distinct high sample with a2ms rise causes immediate backoff.
    assert_eq!(estimator.rate_bps(), 500_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
}

fn held() -> WireRateEstimator {
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 0, 0.0));
    estimator.update(&input(1000, 0, 0.0));
    estimator.phase = WireRatePhase::CapacityHeld;
    estimator.highest_clean = Some(1_000_000.0);
    estimator.blocked = Some(1_050_000.0);
    estimator
}

#[test]
fn capacity_reprobe_is_one_ten_percent_trial_after_thirty_qualified_seconds() {
    // Given a settled1Mbit lower bound and continuous full, proven, clear epochs.
    let mut estimator = held();
    for n in 1..30 {
        estimator.update(&input(1000 + n * 1000, n * 125_000, 0.0));
        assert_eq!(estimator.rate_bps(), 1_000_000.0);
    }
    // When the continuous qualification reaches exactly30seconds.
    estimator.update(&input(31_000, 3_750_000, 0.0));
    assert_eq!(estimator.rate_bps(), 1_100_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Searching);
    estimator.update(&input(32_000, 3_887_500, 0.0));
    // Then a successful single1.10x probe settles, rather than launching a2x climb.
    assert_eq!(estimator.rate_bps(), 1_100_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::CapacityHeld);
}

#[test]
fn capacity_reprobe_failure_returns_to_the_held_lower_bound() {
    // Given a due capacity probe above the settled rate.
    let mut estimator = held();
    for n in 1..=30 {
        estimator.update(&input(1000 + n * 1000, n * 125_000, 0.0));
    }
    // When that trial creates queue growth.
    estimator.update(&input(31_020, 3_755_000, 8.0));
    // Then the failed upper value is never retained as settled capacity.
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Draining);
}

#[test]
fn underuse_breaks_the_continuous_capacity_reprobe_clock() {
    // Given20seconds of qualification followed by one underused epoch.
    let mut estimator = held();
    for n in 1..=20 {
        estimator.update(&input(1000 + n * 1000, n * 125_000, 0.0));
    }
    estimator.update(&input(22_000, 2_512_500, 0.0));
    // When full utilization resumes, it must earn a fresh30seconds.
    for n in 1..30 {
        estimator.update(&input(22_000 + n * 1000, 2_512_500 + n * 125_000, 0.0));
        assert_eq!(estimator.rate_bps(), 1_000_000.0);
    }
    estimator.update(&input(52_000, 6_262_500, 0.0));
    // Then only the newly completed continuous interval authorizes the trial.
    assert_eq!(estimator.rate_bps(), 1_100_000.0);
}

#[test]
fn counter_rollover_rebases_utilization_without_lowering_rate() {
    // Given a used search epoch; the batch counter is reset without changing the socket.
    let mut estimator = WireRateEstimator::default();
    estimator.update(&input(1000, 100_000, 0.0));
    estimator.update(&input(2000, 225_000, 0.0));
    estimator.update(&input(3000, 350_000, 0.0));
    assert_eq!(estimator.rate_bps(), 2_000_000.0);
    // When the counter decreases, old accepted bytes must not count toward the new epoch.
    estimator.update(&input(3100, 0, 0.0));
    estimator.update(&WireRateInput {
        latest_original_delivery_ms: None,
        ..input(4100, 0, 0.0)
    });
    // Then the rate remains2Mbit, neither collapsing on zero nor growing from an underflow.
    assert_eq!(estimator.rate_bps(), 2_000_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Searching);
}

#[test]
fn old_generation_rtt_cannot_back_off_a_retained_rate() {
    // Given a learned rate and stale congestion observations from the old generation.
    let mut estimator = held();
    let old = WireRateInput {
        delivery_generation: 1,
        latest_original_delivery_ms: None,
        latest_rtt: Some(WireRttSample {
            observed_ms: 1900,
            rtt_ms: 1000.0,
        }),
        queue_delay_ms: 400.0,
        ..input(2000, 0, 0.0)
    };
    estimator.update(&old);
    // When the same old RTT is presented through multiple validation epochs.
    for now in [2010, 2020, 2200, 2400] {
        estimator.update(&WireRateInput { now_ms: now, ..old });
    }
    // Then only observations from the new generation's time interval may reduce the rate.
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    assert_eq!(estimator.phase(), WireRatePhase::Searching);
}

#[test]
fn stale_generation_rtt_is_consumed_for_the_work_guard_not_for_queue_proof() {
    // Given a revalidation epoch still exposing the prior generation's RTT snapshot.
    let mut estimator = held();
    let stale = WireRateInput {
        delivery_generation: 1,
        latest_rtt: Some(WireRttSample {
            observed_ms: 1900,
            rtt_ms: 1000.0,
        }),
        ..input(2000, 0, 0.0)
    };
    estimator.update(&stale);
    // When that unchanged snapshot is seen on the next pacing wakeup.
    estimator.update(&WireRateInput {
        now_ms: 2010,
        ..stale
    });
    // Then it is not repeatedly processed as a new measurement on every1ms wakeup.
    assert!(!estimator.queue.newer(stale.latest_rtt));
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
}

#[test]
fn mid_epoch_queue_excursion_cannot_count_as_continuous_clear_time() {
    // Given a capacity-held link interrupted by a non-congesting but non-clear RTT.
    let mut estimator = held();
    for n in 1..=20 {
        estimator.update(&input(1000 + n * 1000, n * 125_000, 0.0));
    }
    estimator.update(&input(21_100, 2_512_500, 4.0));
    estimator.update(&input(21_150, 2_518_750, 0.0));
    estimator.update(&input(22_000, 2_625_000, 0.0));
    // When qualification resumes halfway through the earlier accounting epoch.
    for n in 1..30 {
        estimator.update(&input(22_000 + n * 1000, 2_625_000 + n * 125_000, 0.0));
    }
    // Then the uncleared prefix cannot make the30second reprobe happen early.
    assert_eq!(estimator.rate_bps(), 1_000_000.0);
    estimator.update(&input(52_000, 6_375_000, 0.0));
    assert_eq!(estimator.rate_bps(), 1_100_000.0);
}
