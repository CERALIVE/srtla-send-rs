use tokio::net::UdpSocket;

use super::*;
use crate::connection::health::{HealthConstants, HealthSignals};
use crate::connection::probe::ProbeOpportunity;
use crate::test_helpers::create_test_connection_to;

#[tokio::test]
async fn scenario_d_is_held_probed_and_readmitted_after_rejoining_ramp() {
    // Given the real scenario-D fixture: DATA attempts, keepalive echoes, NAK pruning.
    let clock = TestClock::new(10_000);
    let stalled = crate::tests::health_delivery_tests::scenario_d(&clock).await;
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conns = pool().await;
    conns[0] = stalled;
    // Retain the evidence but give the probe test an observed ephemeral receiver.
    let socket = create_test_connection_to(receiver.local_addr().unwrap()).await;
    conns[0].socket = socket.socket;
    let signals = HealthSignals {
        connected: true,
        socket_valid: true,
        iface_present: true,
        route_health: crate::connection::route::RouteHealth::Unknown,
        attempts_since_proof: conns[0].delivery.attempts_since_proof,
        proof_age_ms: conns[0].delivery.proof_age_ms(14_000),
        keepalive_silence_ms: conns[0].keepalive_liveness.silence_age_ms(14_000),
        srtt_ms: Some(40.0),
        loss_ewma: None,
        loss_cohort_ok: false,
        last_cohort_ms: None,
        probe_loss: None,
        queue_delay_ms: 0.0,
        slow_min_rtt_ms: 40.0,
        probe_rounds_ok: 0,
        probe_rounds_started_ms: None,
        held_links: 1,
        observation_interval_ms: 0,
        now_ms: 14_000,
    };
    conns[0].health = HealthMachine::new(HealthState::Healthy, 10_000);
    conns[0].health.step(&signals, &HealthConstants::default());
    conns[0].window = 20_000;
    conns[1].window = 100;
    let mut state = AdaptiveState::default();
    // When offered traffic arrives, the stalled link is held but gets real duplicates.
    for n in 0..20_u32 {
        let now = 14_000 + u64::from(n) * 100;
        clock.set(now);
        assert_eq!(pick(&mut conns, &mut state, now), 1);
        let mut packet = [0; 1316];
        let seq = 100 + n;
        packet[..4].copy_from_slice(&seq.to_be_bytes());
        let primary = conns[1].conn_id;
        conns[1].queue_data_packet(&packet, Some(seq), now);
        conns[1].flush_batch().await.unwrap();
        let emitted = state
            .probe
            .emit(
                &mut conns,
                ProbeOpportunity {
                    primary_conn_id: primary,
                    packet: &packet,
                    targets: &state.targets,
                },
            )
            .await
            .unwrap();
        emitted.outcome.unwrap();
        let mut received = [0; 1316];
        let count = receiver.recv(&mut received).await.unwrap();
        assert_eq!(&received[..count], packet);
        crate::sender::ack::apply_srtla_ack(
            &mut conns,
            i32::try_from(seq).unwrap(),
            crate::sender::ack::AckContext {
                arrival_idx: 0,
                reader_generation: 0,
                policy: crate::sender::ack::AckPolicy::Adaptive,
            },
        );
    }
    let (rounds, start) = conns[0].probes.rounds_ok();
    conns[0].health.step(
        &HealthSignals {
            attempts_since_proof: 0,
            proof_age_ms: Some(0),
            probe_rounds_ok: rounds,
            probe_rounds_started_ms: start,
            now_ms: 15_900,
            ..signals
        },
        &HealthConstants::default(),
    );
    // Then the actual health machine enters Rejoining and ramped selection resumes.
    assert_eq!(conns[0].health.state(), HealthState::Rejoining);
    assert_eq!(pick(&mut conns, &mut state, 17_900), 0);
}

#[tokio::test]
async fn scenario_d_stall_ablation_selects_the_stalled_link() {
    // Given the same blackhole with its high apparent capacity after NAK pruning.
    let clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0] = crate::tests::health_delivery_tests::scenario_d(&clock).await;
    conns[0].health = HealthMachine::new(HealthState::Stalled, 14_000);
    conns[0].window = 20_000;
    conns[1].window = 100;
    let mut state = AdaptiveState::default();
    state.features = AdaptiveFeatures::ALL - AdaptiveFeatures::STALL;
    // When the stall detector is ablated, then it really re-enters ranking.
    assert_eq!(pick(&mut conns, &mut state, 14_000), 0);
    assert!(state.targets.iter().all(|t| !t.eligible()));
}
