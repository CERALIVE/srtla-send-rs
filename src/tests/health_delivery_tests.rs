//! DATA-only obstruction: real accepted UDP sends, deterministic protocol clocks.

use tokio::net::UdpSocket;

use crate::config::{STALL_ACK_STALE_MS, STALL_MIN_IN_FLIGHT_PACKETS};
use crate::connection::SrtlaConnection;
use crate::connection::health::{HealthConstants, HealthMachine, HealthSignals, HealthState};
use crate::protocol::{SRT_TYPE_NAK, SRTLA_TYPE_KEEPALIVE};
use crate::registration::SrtlaRegistrationManager;
use crate::test_helpers::{create_test_connection_to, create_test_connection_with_failing_send};
use crate::utils::now_ms;
use crate::utils::test_clock::TestClock;

async fn accepted(count: u32) -> (SrtlaConnection, UdpSocket) {
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    for seq in 0..count {
        conn.queue_data_packet(&[0; 1316], Some(seq), now_ms());
    }
    while conn.has_queued_packets() {
        conn.flush_batch().await.unwrap();
    }
    (conn, receiver)
}

fn health(conn: &SrtlaConnection) -> HealthState {
    let mut machine = HealthMachine::new(HealthState::Healthy, 10_000);
    machine.step(
        &HealthSignals {
            connected: conn.connected,
            socket_valid: true,
            iface_present: true,
            attempts_since_proof: conn.delivery.attempts_since_proof,
            proof_age_ms: conn.delivery.proof_age_ms(now_ms()),
            srtt_ms: conn.has_rtt_sample().then(|| conn.get_smooth_rtt_ms()),
            loss_ewma: None,
            loss_cohort_ok: false,
            last_cohort_ms: None,
            probe_loss: None,
            queue_delay_ms: 0.0,
            slow_min_rtt_ms: 40.0,
            probe_rounds_ok: 0,
            probe_rounds_started_ms: None,
            held_links: 1,
            now_ms: now_ms(),
        },
        &HealthConstants::default(),
    );
    machine.state()
}

async fn scenario_d(clock: &TestClock) -> SrtlaConnection {
    let (mut conn, _receiver) = accepted(40).await;
    assert_eq!(conn.in_flight_packets, 40);
    assert_eq!(conn.delivery.attempts_since_proof, 40);

    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let (forwarder, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut reg = SrtlaRegistrationManager::new();
    for now in [10_000_u64, 11_000, 12_000, 13_000, 14_000] {
        clock.set(now);
        conn.rtt.waiting_for_keepalive_response = true;
        let mut reply = SRTLA_TYPE_KEEPALIVE.to_be_bytes().to_vec();
        reply.extend_from_slice(&(now - 40).to_be_bytes());
        conn.process_packet(
            0,
            &mut reg,
            &listener,
            &forwarder,
            None,
            &reply,
            &crate::stats::SharedStats::new(),
        )
        .await
        .unwrap();
        assert_eq!(conn.last_ack_or_rtt_sample_ms, now);
    }
    // Three real NAK frames drain the backlog without proving any DATA delivery.
    for range in [0_u32..14, 14..27, 27..40] {
        let mut nak = vec![0; 16];
        nak[..2].copy_from_slice(&SRT_TYPE_NAK.to_be_bytes());
        for seq in range {
            nak.extend_from_slice(&seq.to_be_bytes());
        }
        let incoming = conn
            .process_packet(
                0,
                &mut reg,
                &listener,
                &forwarder,
                None,
                &nak,
                &crate::stats::SharedStats::new(),
            )
            .await
            .unwrap();
        for seq in incoming.nak_numbers {
            assert!(conn.handle_nak(i32::try_from(seq).unwrap()));
        }
    }
    assert_eq!(conn.in_flight_packets, 0);
    conn
}

#[tokio::test]
async fn scenario_d_keepalive_echo_without_data_delivery_is_stalled() {
    // Given 40 accepted DATA sends, five RTT replies over 4s and three NAKs.
    let clock = TestClock::new(10_000);
    let conn = scenario_d(&clock).await;
    // When evaluating the ledger-fed HealthMachine (not yet runtime-wired).
    let state = health(&conn);
    // Then the DATA-only obstruction must be detected.
    assert_eq!(state, HealthState::Stalled);
    assert_eq!(conn.delivery.attempts_since_proof, 40);
}

#[tokio::test]
async fn legacy_stall_predicate_misses_scenario_d() {
    // Given exactly the same obstruction fixture as the new-health test.
    let clock = TestClock::new(10_000);
    let conn = scenario_d(&clock).await;
    // When evaluating the frozen experimental predicate.
    let stalled =
        conn.is_stall_penalized(now_ms(), STALL_MIN_IN_FLIGHT_PACKETS, STALL_ACK_STALE_MS);
    // Then it misses this obstruction; preserve this falsifiability control.
    assert!(!stalled);
}

#[tokio::test]
async fn data_ack_clears_the_stall_predicate() {
    // Given the same DATA obstruction, with its packet log already empty.
    let clock = TestClock::new(10_000);
    let mut conn = scenario_d(&clock).await;
    assert_eq!(health(&conn), HealthState::Stalled);
    // When a link-specific DATA ACK arrives before its ledger entry expires.
    conn.handle_srtla_ack_specific(39, false);
    // Then the stall predicate clears; this is not a Stalled→Healthy rejoin bypass.
    assert_eq!(health(&conn), HealthState::Healthy);
    assert_eq!(conn.delivery.attempts_since_proof, 0);
    assert_eq!(conn.delivery.last_data_proof_ms, 14_000);
}

#[tokio::test]
async fn cumulative_srt_ack_before_srtla_ack_does_not_erase_proof() {
    // Given a cumulative ACK that raced ahead and pruned the packet log.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted(1).await;
    clock.set(10_040);
    conn.handle_srt_ack(0, now_ms(), true);
    assert!(conn.packet_log.is_empty());
    let window = conn.window;
    // When this link's specific ACK finally arrives.
    let legacy_hit = conn.handle_srtla_ack_specific(0, false);
    // Then proof/rate survive independently, without inventing legacy window growth.
    assert_eq!(conn.delivery.last_data_proof_ms, 10_040);
    assert_eq!(conn.delivery.delivered_bps(now_ms()), 5264.0);
    assert_eq!((legacy_hit, conn.window), (false, window));
}

#[tokio::test]
async fn nak_then_late_srtla_ack_still_counts_delivery() {
    // Given a NAK-pruned packet.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted(1).await;
    assert!(conn.handle_nak(0));
    clock.set(10_100);
    // When a late specific ACK arrives.
    conn.handle_srtla_ack_specific(0, false);
    // Then the ledger credits exactly the accepted wire length.
    assert_eq!(conn.delivery.last_data_proof_ms, 10_100);
    assert_eq!(conn.delivery.delivered_bps(now_ms()), 5264.0);
}

#[tokio::test]
async fn neither_cumulative_ack_nor_nak_is_proof() {
    // Given two accepted DATA sends.
    let _clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted(2).await;
    // When both destructive pruning paths run, including an owned cumulative ACK.
    conn.handle_srt_ack(0, now_ms(), true);
    assert!(conn.handle_nak(1));
    // Then neither event reduces attempts nor stamps DATA proof.
    assert_eq!(conn.delivery.attempts_since_proof, 2);
    assert_eq!(conn.delivery.last_data_proof_ms, 0);
    assert_eq!(conn.delivery.delivered_bps(now_ms()), 0.0);
    assert_eq!(conn.in_flight_packets, 0);
}

#[tokio::test]
async fn old_generation_ack_is_ignored() {
    // Given an outstanding send followed by recovery invalidation.
    let clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted(1).await;
    let generation = conn.delivery.socket_generation;
    conn.mark_for_recovery();
    clock.set(10_040);
    // When the old sequence is ACKed in the new generation.
    conn.handle_srtla_ack_specific(0, false);
    // Then no proof survives the invalidation boundary.
    assert_eq!(conn.delivery.socket_generation, generation + 1);
    assert_eq!(conn.delivery.last_data_proof_ms, 0);
    assert_eq!(conn.delivery.delivered_bps(now_ms()), 0.0);
    assert_eq!(conn.delivery.proof_age_ms(now_ms()), None);
}

#[tokio::test]
async fn socket_replacement_invalidates_delivery() {
    // Given an outstanding send on a live socket.
    let _clock = TestClock::new(10_000);
    let (mut conn, _receiver) = accepted(1).await;
    let generation = conn.delivery.socket_generation;
    // When reconnect replaces that socket without an earlier soft reset.
    conn.reconnect().await.unwrap();
    conn.handle_srtla_ack_specific(0, false);
    // Then its old DATA cannot prove delivery on the replacement.
    assert_eq!(conn.delivery.socket_generation, generation + 1);
    assert_eq!(conn.delivery.last_data_proof_ms, 0);
    assert_eq!(conn.delivery.attempts_since_proof, 0);
}

#[tokio::test]
async fn queued_and_failed_data_never_become_attempts() {
    // Given a deterministic send-failing socket and one queued packet.
    let _clock = TestClock::new(10_000);
    let mut conn = create_test_connection_with_failing_send().await;
    conn.queue_data_packet(&[0; 100], Some(7), now_ms());
    assert_eq!(conn.delivery.attempts_since_proof, 0);
    // When no datagrams are accepted and a spurious ACK arrives.
    assert!(conn.flush_batch().await.is_err());
    conn.handle_srtla_ack_specific(7, false);
    // Then neither queueing nor the failed send supplies delivery evidence.
    assert_eq!(conn.delivery.attempts_since_proof, 0);
    assert_eq!(conn.delivery.last_data_proof_ms, 0);
}
