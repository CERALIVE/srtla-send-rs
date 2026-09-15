use tokio::net::UdpSocket;

use super::SrtlaConnection;
use super::health::{HealthMachine, HealthState};
use crate::registration::SrtlaRegistrationManager;
use crate::sender::housekeeping::tick_health;
use crate::stats::SharedStats;
use crate::test_helpers::create_test_connection_to;
use crate::utils::test_clock::TestClock;

async fn reply(conn: &mut SrtlaConnection, packet: &[u8]) {
    let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let (forwarder, _rx) = tokio::sync::mpsc::unbounded_channel();
    conn.process_packet(
        0,
        &mut SrtlaRegistrationManager::new(),
        &listener,
        &forwarder,
        None,
        packet,
        &SharedStats::new(),
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn silent_keepalive_path_stalls_without_any_data_attempts() {
    // Given: a registered idle link with a successful keepalive exchange.
    let clock = TestClock::new(10_000);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    conn.health = HealthMachine::new(HealthState::Healthy, 10_000);
    conn.send_keepalive().await.unwrap();
    let mut packet = [0; 38];
    let n = receiver.recv(&mut packet).await.unwrap();
    clock.set(10_040);
    reply(&mut conn, &packet[..n]).await;
    // When: sends continue, but replies stop and no DATA is scheduled.
    for now in [11_040, 12_040, 13_039] {
        clock.set(now);
        conn.send_keepalive().await.unwrap();
        tick_health(std::slice::from_mut(&mut conn), &[], now);
        assert_eq!(conn.health.state(), HealthState::Healthy);
    }
    clock.set(13_040);
    tick_health(std::slice::from_mut(&mut conn), &[], 13_040);
    // Then: three keepalive intervals suffice without 32 DATA attempts.
    assert_eq!(conn.delivery.attempts_since_proof, 0);
    assert_eq!(conn.health.state(), HealthState::Stalled);
}

#[tokio::test]
async fn healthy_keepalive_replies_do_not_stall_a_data_starved_link() {
    // Given: no DATA attempts and RTT sampling suppressed by recent measurements.
    let clock = TestClock::new(10_000);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    conn.health = HealthMachine::new(HealthState::Healthy, 10_000);
    // When: periodic replies continue beyond the silence deadline.
    for now in (10_000..20_000).step_by(1000) {
        clock.set(now);
        conn.rtt.last_rtt_measurement_ms = now;
        conn.send_keepalive().await.unwrap();
        assert!(!conn.rtt.waiting_for_keepalive_response);
        let mut packet = [0; 38];
        let n = receiver.recv(&mut packet).await.unwrap();
        clock.set(now + 40);
        reply(&mut conn, &packet[..n]).await;
        tick_health(std::slice::from_mut(&mut conn), &[], now + 40);
        // Then: liveness does not require DATA or an accepted RTT sample.
        assert_eq!(conn.delivery.attempts_since_proof, 0);
        assert_eq!(conn.health.state(), HealthState::Healthy);
    }
}

#[tokio::test]
async fn late_data_ack_cannot_permanently_starve_silent_link_detection() {
    // Given: DATA accepted before a failure, then acknowledged late.
    let clock = TestClock::new(10_000);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    conn.health = HealthMachine::new(HealthState::Healthy, 10_000);
    conn.send_keepalive().await.unwrap();
    conn.queue_data_packet(&[0; 1316], Some(7), 10_000);
    conn.flush_batch().await.unwrap();
    clock.set(12_000);
    conn.handle_srtla_ack_specific(7, false);
    assert_eq!(conn.delivery.attempts_since_proof, 0);
    // When: no more DATA is routed, no keepalive reply arrives, and proof expires.
    clock.set(15_000);
    conn.send_keepalive().await.unwrap();
    tick_health(std::slice::from_mut(&mut conn), &[], 15_000);
    // Then: the reset attempts counter no longer makes silence undetectable.
    assert_eq!(conn.delivery.attempts_since_proof, 0);
    assert_eq!(conn.health.state(), HealthState::Stalled);
}

#[tokio::test]
async fn recovery_reset_discards_keepalive_evidence_and_pending_echoes() {
    // Given: an outstanding keepalive on the old socket lifetime.
    let clock = TestClock::new(10_000);
    let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let mut conn = create_test_connection_to(receiver.local_addr().unwrap()).await;
    conn.send_keepalive().await.unwrap();
    let mut packet = [0; 38];
    let n = receiver.recv(&mut packet).await.unwrap();
    // When: recovery clears the lifetime before the old echo arrives.
    conn.mark_for_recovery();
    clock.set(11_000);
    reply(&mut conn, &packet[..n]).await;
    // Then: neither a false old deadline nor old proof survives.
    assert_eq!(conn.keepalive_liveness.silence_age_ms(11_000), None);
}
