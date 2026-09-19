use crate::connection::health::{HealthMachine, HealthState};
use crate::connection::loss::LossTracker;
use crate::connection::probe::{ProbeLog, ProbeScheduler, ProbeTarget, ProbeTrain};
use crate::sender::housekeeping::tick_health;
use crate::test_helpers::create_test_connection;
use crate::utils::test_clock::TestClock;

fn train(log: &mut ProbeLog, id: u64, start: u64) {
    let spec = ProbeTrain {
        id,
        started_ms: start,
        deadline_ms: start + 2060,
    };
    for offset in 0..10 {
        let seq = i32::try_from(id * 10 + offset).unwrap();
        log.record_sent(seq, spec, start + offset * 100);
        assert!(log.acknowledge(seq, start + offset * 100 + 60));
    }
}

#[test]
fn pending_probe_train_does_not_erase_two_successful_rounds() {
    // Given two completed healthy trains, followed by a fully sent in-flight train.
    let mut log = ProbeLog::default();
    train(&mut log, 0, 0);
    train(&mut log, 1, 1000);
    let spec = ProbeTrain {
        id: 2,
        started_ms: 2000,
        deadline_ms: 4060,
    };
    // When no ACK has arrived for the newest train yet, it is pending, not failed.
    for seq in 20..30 {
        log.record_sent(seq, spec, 2000);
    }
    // Then the earlier consecutive evidence survives until a genuine failure/expiry.
    assert_eq!(log.rounds_ok().0, 2);
    log.advance(4061);
    assert_eq!(log.rounds_ok().0, 0);
}

#[tokio::test]
async fn recovered_link_does_not_mix_pre_outage_loss_into_new_cohorts() {
    // Given stale all-loss evidence, cleared by two successful real probe-log trains.
    let clock = TestClock::new(0);
    let mut conn = create_test_connection().await;
    conn.health = HealthMachine::new(HealthState::Degraded, 0);
    conn.loss = LossTracker::new(0);
    conn.rtt.update_estimate(60);
    for _ in 0..100 {
        conn.loss.record_send(0);
        conn.loss.record_data_nak(0);
    }
    conn.loss.advance(1000);
    train(&mut conn.probes, 0, 10000);
    train(&mut conn.probes, 1, 11000);
    clock.set(13000);
    tick_health(std::slice::from_mut(&mut conn), &[], 13000);
    clock.set(14000);
    tick_health(std::slice::from_mut(&mut conn), &[], 14000);
    assert_eq!(conn.health.state(), HealthState::Rejoining);
    // When the recovered path completes a qualifying loss-free normal cohort.
    for _ in 0..100 {
        conn.loss.record_send(14000);
    }
    conn.loss.advance(15000);
    // Then an old episode cannot immediately demote it through an inherited EWMA.
    assert_eq!(conn.loss.last_value(), Some(0.0));
}

#[tokio::test]
async fn completed_probe_pair_survives_next_housekeeping_observation() {
    // Given real 10pps trains on one held twin, with the measured 105ms opportunities.
    let clock = TestClock::new(10_000);
    let mut conn = create_test_connection().await;
    conn.health = HealthMachine::new(HealthState::Stalled, 10_000);
    conn.rtt.update_estimate(60);
    let targets = [ProbeTarget {
        conn_id: conn.conn_id,
        socket_generation: conn.delivery.socket_generation,
        health: HealthState::Stalled,
        deadline_held: false,
        sole_carrier: false,
        srtt_ms: 60,
    }];
    let mut scheduler = ProbeScheduler::default();
    for seq in 0..20 {
        let at = 10_000 + u64::try_from(seq).unwrap() * 105;
        clock.set(at);
        let dispatch = scheduler.next(&targets, at).unwrap();
        conn.probes.record_sent(seq, dispatch.train, at);
        clock.set(at + 60);
        assert!(conn.probes.acknowledge(seq, at + 60));
    }
    assert_eq!(conn.probes.rounds_ok(), (2, Some(10_000)));
    assert_eq!(conn.probes.probe_loss(), Some(0.0));
    // When housekeeping observes at the measured oldest-start age2697ms, 642ms after completion.
    clock.set(12_697);
    tick_health(std::slice::from_mut(&mut conn), &targets, 12_697);
    // Then successful evidence is usable at the first observation, not indefinitely delayed.
    assert_eq!(conn.health.state(), HealthState::Rejoining);
}

#[tokio::test]
async fn sampled_probe_recovery_rejects_stale_and_previous_epoch_pairs() {
    // Given complete trains whose proof is either stale or predates this Stalled epoch.
    for (entry, observed) in [(10_000, 13_061), (10_001, 12_697)] {
        let clock = TestClock::new(10_000);
        let mut conn = create_test_connection().await;
        conn.health = HealthMachine::new(HealthState::Stalled, entry);
        conn.rtt.update_estimate(60);
        train(&mut conn.probes, 0, 10_000);
        train(&mut conn.probes, 1, 11_050);
        // When evaluated after the bounded sampling allowance or across the epoch boundary.
        clock.set(observed);
        tick_health(std::slice::from_mut(&mut conn), &[], observed);
        // Then a sampling correction must not authorize old recovery proof.
        assert_eq!(
            conn.health.state(),
            HealthState::Stalled,
            "{entry}/{observed}"
        );
    }
}

#[tokio::test]
async fn sampled_probe_recovery_still_requires_two_qualified_trains() {
    // Given one successful train and a second with only four ACKs.
    let clock = TestClock::new(10_000);
    let mut conn = create_test_connection().await;
    conn.health = HealthMachine::new(HealthState::Stalled, 10_000);
    conn.rtt.update_estimate(60);
    train(&mut conn.probes, 0, 10_000);
    let spec = ProbeTrain {
        id: 1,
        started_ms: 11_050,
        deadline_ms: 13_110,
    };
    for seq in 10..20 {
        let at = 11_050 + u64::try_from(seq - 10).unwrap() * 105;
        conn.probes.record_sent(seq, spec, at);
        if seq < 14 {
            assert!(conn.probes.acknowledge(seq, at + 60));
        }
    }
    // When observation is delayed by the same sampling phase as the valid pair.
    clock.set(12_697);
    tick_health(std::slice::from_mut(&mut conn), &[], 12_697);
    // Then observation slack cannot manufacture the missing recovery proof.
    assert_eq!(conn.health.state(), HealthState::Stalled);
}
