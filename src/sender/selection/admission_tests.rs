use super::*;
use crate::connection::health::{HealthMachine, HealthState};
use crate::mode::SchedulingMode;
use crate::sender::selection::{EdpfSchedulerState, select_connection_idx_with_state};
use crate::tests::adaptive_tests::{config, pool_of};
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn admission_classifies_every_health_state() {
    // Given one link in each health state and no deadline pressure.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(5).await;
    for (c, state) in conns.iter_mut().zip([
        HealthState::Healthy,
        HealthState::Rejoining,
        HealthState::Degraded,
        HealthState::Stalled,
        HealthState::Down,
    ]) {
        c.health = HealthMachine::new(state, 10_000);
    }
    // When shared admission snapshots the pool.
    let admission = admit(
        &mut conns,
        10_000,
        &config(),
        &mut SchedulerShared::default(),
    );
    // Then only normal DATA candidates are eligible; Down is not probe-held.
    assert_eq!(admission.eligible.as_slice(), &[0, 1]);
    assert_eq!(admission.held.as_slice(), &[2, 3]);
    assert_eq!(admission.held_links, 2);
    assert!(!admission.fallback);
}

#[tokio::test]
async fn admission_deadline_holds_healthy_and_rejoining() {
    // Given healthy and rejoining links whose OWD exceeds the unknown-latency budget.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(3).await;
    conns[0].rtt.kalman_rtt.update(600.0);
    conns[1].rtt.kalman_rtt.update(600.0);
    conns[1].health = HealthMachine::new(HealthState::Rejoining, 10_000);
    // When deadline admission runs.
    let admission = admit(
        &mut conns,
        10_000,
        &config(),
        &mut SchedulerShared::default(),
    );
    // Then both held targets are counted before any sole election.
    assert_eq!(admission.eligible.as_slice(), &[2]);
    assert_eq!(admission.held.as_slice(), &[0, 1]);
}

#[tokio::test]
async fn admission_fallback_matches_adaptive_connected_only() {
    // Given a Down-but-connected link, a held connected link, and a disconnected link.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(3).await;
    conns[0].health = HealthMachine::new(HealthState::Down, 0);
    conns[1].health = HealthMachine::new(HealthState::Stalled, 0);
    conns[2].connected = false;
    // When normal eligibility is empty.
    let admission = admit(
        &mut conns,
        10_000,
        &config(),
        &mut SchedulerShared::default(),
    );
    // Then fallback contains exactly connected links, even Down.
    assert!(admission.fallback);
    assert_eq!(admission.eligible.as_slice(), &[0, 1]);
}

#[tokio::test]
async fn admission_empty_only_when_no_connected_link_exists() {
    // Given every combination of connectivity in an otherwise Down pool.
    let _clock = TestClock::new(10_000);
    for mask in 0..8 {
        let mut conns = pool_of(3).await;
        for (i, c) in conns.iter_mut().enumerate() {
            c.health = HealthMachine::new(HealthState::Down, 0);
            c.connected = mask & (1 << i) != 0;
        }
        // When the admission fallback is built.
        let admission = admit(
            &mut conns,
            10_000,
            &config(),
            &mut SchedulerShared::default(),
        );
        // Then no connected combination is stranded.
        assert_eq!(admission.eligible.is_empty(), mask == 0);
    }
}

#[tokio::test]
async fn admission_down_connected_fallback_preserves_negative_base_order_in_every_mode() {
    // Given hard-ineligible connected links with negative capacity scores.
    let _clock = TestClock::new(10_000);
    for mode in [
        SchedulingMode::Classic,
        SchedulingMode::Enhanced,
        SchedulingMode::RttThreshold,
        SchedulingMode::Edpf,
        SchedulingMode::Adaptive,
    ] {
        let mut conns = pool_of(2).await;
        for c in &mut conns {
            c.health = HealthMachine::new(HealthState::Down, 0);
        }
        conns[0].window = -20;
        conns[1].window = -10;
        let cfg = ConfigSnapshot { mode, ..config() };
        // When the full dispatcher reaches the connected escape.
        let selected = select_connection_idx_with_state(
            &mut conns,
            None,
            0,
            10_000,
            &cfg,
            &mut EdpfSchedulerState::default(),
            &mut SchedulerShared::default(),
        );
        // Then the old base-only fallback wins without quality computation on held links.
        assert_eq!(selected, Some(1), "{mode}");
        assert_eq!(conns[1].adaptive.weight.unwrap().effective_multiplier, 1.0);
        assert!(conns[0].adaptive.weight.is_none());
    }
}

#[tokio::test]
async fn admission_held_cache_does_not_advance_until_readmission() {
    // Given a held link with a stale, deliberately distinctive quality cache.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(2).await;
    conns[0].health = HealthMachine::new(HealthState::Stalled, 0);
    conns[0].quality_cache.multiplier = 0.7;
    conns[0].quality_cache.last_calculated_ms = 9000;
    let mut shared = SchedulerShared::default();
    // When a held tick is followed by an admitted tick using the same stores.
    let mut held = admit(&mut conns, 10_000, &config(), &mut shared);
    super::super::adaptive::ranking::refresh(&mut conns, &mut shared, 10_000, &mut held);
    assert_eq!(conns[0].quality_cache.last_calculated_ms, 9000);
    assert_eq!(conns[0].quality_cache.multiplier, 0.7);
    conns[0].health = HealthMachine::new(HealthState::Healthy, 10_030);
    let mut admitted = admit(&mut conns, 10_030, &config(), &mut shared);
    let weight = admitted.compute_weight(0, &mut conns[0]);
    // Then the first admitted tick, not the held tick, refreshes quality.
    assert_eq!(weight.quality_multiplier.to_bits(), 1.1_f64.to_bits());
    assert_eq!(conns[0].quality_cache.last_calculated_ms, 10_030);
}

#[tokio::test]
async fn admission_quality_cache_obeys_the_old_fifty_ms_boundary() {
    // Given an initial quality refresh, followed by new NAK evidence.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(1).await;
    let mut shared = SchedulerShared::default();
    let mut actual = Vec::new();
    // When replaying t, t+30 and t+60 through separate admission snapshots.
    for now in [10_000, 10_030, 10_060] {
        let mut admission = admit(&mut conns, now, &config(), &mut shared);
        actual.push(
            admission
                .compute_weight(0, &mut conns[0])
                .quality_multiplier
                .to_bits(),
        );
        conns[0].congestion.nak_count = 1;
    }
    // Then the middle tick reuses the old cache and the last sees the startup penalty.
    assert_eq!(
        actual,
        [1.1_f64.to_bits(), 1.1_f64.to_bits(), 0.98_f64.to_bits()]
    );
}

#[tokio::test]
async fn admission_quality_bit_off_is_neutral_without_cache_mutation() {
    // Given disabled QUALITY and a distinctive stale cache.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(1).await;
    conns[0].quality_cache.multiplier = 0.7;
    let cfg = ConfigSnapshot {
        features: SchedulerFeatures::ALL - SchedulerFeatures::QUALITY,
        ..config()
    };
    // When weighting the eligible link.
    let mut admission = admit(&mut conns, 10_000, &cfg, &mut SchedulerShared::default());
    let weight = admission.compute_weight(0, &mut conns[0]);
    // Then quality is exactly neutral and its cache remains untouched.
    assert_eq!(weight.effective_multiplier.to_bits(), 1.0_f64.to_bits());
    assert_eq!(conns[0].quality_cache.last_calculated_ms, 0);
    assert_eq!(conns[0].quality_cache.multiplier, 0.7);
}

#[tokio::test]
async fn admission_weight_is_lazy_and_cached_within_one_snapshot() {
    // Given an eligible link whose published base has not been computed yet.
    let _clock = TestClock::new(10_000);
    let mut conns = pool_of(1).await;
    let mut admission = admit(
        &mut conns,
        10_000,
        &config(),
        &mut SchedulerShared::default(),
    );
    assert!(admission.weight[0].is_none());
    // When the same candidate is requested twice despite intervening window mutation.
    let first = admission.compute_weight(0, &mut conns[0]);
    conns[0].window = 1;
    let second = admission.compute_weight(0, &mut conns[0]);
    // Then one snapshot keeps the first base/multiplier pair exactly.
    assert_eq!(first.base_score, second.base_score);
    assert_eq!(
        first.effective_multiplier.to_bits(),
        second.effective_multiplier.to_bits()
    );
}
