use super::*;
use crate::bind_map::Priority;
use crate::connection::health::{HealthMachine, HealthState};
use crate::sender::selection::adaptive::ranking;
use crate::tests::adaptive_tests::{config, pool_of};
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn admission_stall_loss_and_deadline_bits_control_the_eligible_set() {
    // Given a separate distinguishing pool for each exclusion mechanism.
    let _clock = TestClock::new(10_000);
    for (bit, health, rtt) in [
        (SchedulerFeatures::STALL, HealthState::Stalled, 20.0),
        (SchedulerFeatures::LOSS, HealthState::Degraded, 20.0),
        (SchedulerFeatures::DEADLINE, HealthState::Healthy, 600.0),
    ] {
        let mut conns = pool_of(2).await;
        conns[0].health = HealthMachine::new(health, 0);
        conns[0].rtt.kalman_rtt.update(rtt);
        // When one bit is on and then off at the same clock.
        let mut results = Vec::new();
        for features in [SchedulerFeatures::ALL, SchedulerFeatures::ALL - bit] {
            let cfg = ConfigSnapshot {
                features,
                ..config()
            };
            results.push(admit(&mut conns, 10_000, &cfg, &mut SchedulerShared::default()).eligible);
        }
        // Then only the relevant held link re-enters, without affecting its neighbour.
        assert_eq!(results[0].as_slice(), &[1]);
        assert_eq!(results[1].as_slice(), &[0, 1]);
    }
}

#[tokio::test]
async fn admission_queue_bit_removes_only_queue_delay_from_deadline_prediction() {
    // Given a measured queue exceeding the budget while smoothed OWD alone fits.
    let clock = TestClock::new(10_000);
    let mut conns = pool_of(2).await;
    conns[0].rtt.update_estimate(20);
    clock.set(11_001);
    conns[0].rtt.update_estimate(500);
    conns[0].rtt.kalman_rtt.reset();
    conns[0].rtt.kalman_rtt.update(100.0);
    let mut shared = SchedulerShared::default();
    assert_eq!(
        admit(&mut conns, 11_001, &config(), &mut shared)
            .eligible
            .as_slice(),
        &[1]
    );
    let cfg = ConfigSnapshot {
        features: SchedulerFeatures::ALL - SchedulerFeatures::QUEUE,
        ..config()
    };
    // When QUEUE is disabled over the continuous clearance dwell.
    let waiting = admit(&mut conns, 11_001, &cfg, &mut shared);
    let released = admit(&mut conns, 12_001, &cfg, &mut shared);
    // Then hysteresis is preserved before the eligible set expands.
    assert_eq!(waiting.eligible.as_slice(), &[1]);
    assert_eq!(released.eligible.as_slice(), &[0, 1]);
}

#[tokio::test]
async fn admission_each_weight_factor_can_be_disabled_independently() {
    // Given a distinct synthetic input for each of the four weighting bits.
    let _clock = TestClock::new(10_000);
    for (bit, enabled, disabled) in [
        (SchedulerFeatures::QUALITY, 0.7, 1.0),
        (SchedulerFeatures::REJOIN, 0.7 * 0.05, 0.7),
        (SchedulerFeatures::PREF, 0.7 * 0.8, 0.7),
        (SchedulerFeatures::RATECAP, 0.7 * 0.5, 0.7),
    ] {
        let mut conns = pool_of(1).await;
        conns[0].quality_cache.multiplier = 0.7;
        conns[0].quality_cache.last_calculated_ms = 10_000;
        if bit == SchedulerFeatures::REJOIN {
            conns[0].health = HealthMachine::new(HealthState::Rejoining, 10_000);
        }
        if bit == SchedulerFeatures::PREF {
            conns[0].priority_baseline = Some(Priority::try_from(-0.2).unwrap());
        }
        if bit == SchedulerFeatures::RATECAP {
            conns[0].in_flight_packets = 64;
        }
        // When only the selected factor is disabled.
        let mut actual = Vec::new();
        for features in [SchedulerFeatures::ALL, SchedulerFeatures::ALL - bit] {
            let cfg = ConfigSnapshot {
                features,
                ..config()
            };
            let mut admission = admit(&mut conns, 10_000, &cfg, &mut SchedulerShared::default());
            actual.push(
                admission
                    .compute_weight(0, &mut conns[0])
                    .effective_multiplier,
            );
        }
        // Then the exact ordered composition loses only that factor.
        assert_eq!(actual, [enabled, disabled], "{bit:?}");
    }
}

#[tokio::test]
async fn admission_sole_bit_preserves_election_and_base_only_escape() {
    // Given two held links where RTT election and base-score fallback disagree.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(2).await;
    for (c, rtt) in conns.iter_mut().zip([100.0, 20.0]) {
        c.health = HealthMachine::new(HealthState::Stalled, 0);
        c.rtt.kalman_rtt.update(rtt);
    }
    // When SOLE is on versus off.
    let mut actual = Vec::new();
    for features in [
        SchedulerFeatures::ALL,
        SchedulerFeatures::ALL - SchedulerFeatures::SOLE,
    ] {
        let mut shared = SchedulerShared::default();
        let cfg = ConfigSnapshot {
            features,
            ..config()
        };
        let mut admission = admit(&mut conns, 10_000, &cfg, &mut shared);
        let choice = match ranking::refresh(&mut conns, &mut shared, 10_000, &mut admission) {
            ranking::Selection::Carrier(index) => index,
            ranking::Selection::Ranked { .. } => panic!("all links held"),
        };
        actual.push(choice);
    }
    // Then election prefers RTT, while the unchanged escape breaks base ties by index.
    assert_eq!(actual, [Some(1), Some(0)]);
}

#[tokio::test]
async fn preference_is_reached_from_admission() {
    // Given two otherwise-identical Healthy links with wide windows and opposite
    // bounded preferences, on the real admission path with only PREF enabled.
    let _clock = TestClock::new(10_000);
    let mut conns = crate::test_helpers::create_selection_test_connections(2).await;
    for conn in &mut conns {
        conn.window = 20_000;
    }
    conns[0].priority_baseline = Some(Priority::try_from(0.2).unwrap());
    conns[1].priority_baseline = Some(Priority::try_from(-0.2).unwrap());
    let cfg = ConfigSnapshot {
        features: SchedulerFeatures::PREF,
        ..config()
    };
    // When admission composes the selection weight for both links.
    let mut admission = admit(&mut conns, 10_000, &cfg, &mut SchedulerShared::default());
    let preferred = admission
        .compute_weight(0, &mut conns[0])
        .effective_multiplier;
    let disfavoured = admission
        .compute_weight(1, &mut conns[1])
        .effective_multiplier;
    // Then the preference formula reaches the product: +0.2 -> 1.2x, -0.2 -> 0.8x.
    println!("priority +0.20 -> effective_multiplier {preferred}");
    println!("priority -0.20 -> effective_multiplier {disfavoured}");
    assert!(
        (preferred - 1.2).abs() < 1e-12,
        "preferred effective_multiplier: {preferred}"
    );
    assert!(
        (disfavoured - 0.8).abs() < 1e-12,
        "disfavoured effective_multiplier: {disfavoured}"
    );

    // Control: equal priorities must produce identical multipliers, so the
    // divergence above is attributable to priority alone, not link position.
    conns[0].priority_baseline = Some(Priority::try_from(0.0).unwrap());
    conns[1].priority_baseline = Some(Priority::try_from(0.0).unwrap());
    let mut admission = admit(&mut conns, 10_000, &cfg, &mut SchedulerShared::default());
    let first = admission
        .compute_weight(0, &mut conns[0])
        .effective_multiplier;
    let second = admission
        .compute_weight(1, &mut conns[1])
        .effective_multiplier;
    assert!(
        (first - second).abs() < 1e-12,
        "equal priorities: {first} vs {second}"
    );
}

#[tokio::test]
async fn admission_sole_sequential_replay_preserves_proof_wait_and_challenger() {
    // Given an elected stalled carrier and a faster challenger learned during its hold.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(2).await;
    for c in &mut conns {
        c.health = HealthMachine::new(HealthState::Stalled, 0);
    }
    conns[0].rtt.kalman_rtt.update(100.0);
    conns[1].rtt.kalman_rtt.update(200.0);
    let mut shared = SchedulerShared::default();
    let mut actual = Vec::new();
    // When fresh proof keeps the incumbent alive through the two-second election hold.
    for now in [10_000, 11_999, 12_000] {
        if now > 10_000 {
            conns[0].delivery.record_probe_proof(now);
            conns[1].rtt.kalman_rtt.reset();
            conns[1].rtt.kalman_rtt.update(20.0);
        }
        let mut admission = admit(&mut conns, now, &config(), &mut shared);
        match ranking::refresh(&mut conns, &mut shared, now, &mut admission) {
            ranking::Selection::Carrier(index) => actual.push(index),
            ranking::Selection::Ranked { .. } => panic!("all links held"),
        }
    }
    // Then the challenger waits for the hold and wins without a false proof failure.
    assert_eq!(actual, [Some(0), Some(0), Some(1)]);
    assert!(!conns[0].adaptive.sole_failed_until_proof);
}
