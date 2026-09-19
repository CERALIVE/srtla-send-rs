use crate::connection::wire_rate::{WireRateInput, WireRatePhase, WireRttSample};
use crate::mode::SchedulingMode;
use crate::test_helpers::create_test_connection;
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn learned_wire_rate_survives_recovery_and_socket_recreation() -> anyhow::Result<()> {
    // Given a learned8Mbit wire rate independent of the soft controller's1Mbit target.
    let clock = TestClock::new(1000);
    let mut conn = create_test_connection().await;
    for (now_ms, bytes) in [
        (1000, 0),
        (2000, 125_000),
        (3000, 250_000),
        (4000, 500_000),
        (5000, 1_000_000),
    ] {
        conn.wire_rate.update(&WireRateInput {
            now_ms,
            delivery_generation: conn.delivery.socket_generation,
            accepted_bytes: Some(bytes),
            latest_original_delivery_ms: Some(now_ms),
            latest_rtt: Some(WireRttSample {
                observed_ms: now_ms,
                rtt_ms: 60.0,
            }),
            slow_min_rtt_ms: 60.0,
            queue_delay_ms: 0.0,
        });
    }
    assert_eq!(conn.wire_rate.rate_bps(), 8_000_000.0);
    assert_eq!(conn.rate_cap.target_bps(), 1_000_000.0);
    // When production configuration disables hard admission and rebuilds the socket.
    clock.set(2000);
    super::configure(std::slice::from_mut(&mut conn), SchedulingMode::Enhanced);
    assert_eq!(conn.wire_rate.rate_bps(), 8_000_000.0);
    assert!(!conn.batch_sender.wire_limited());
    conn.mark_for_recovery();
    conn.reconnect().await?;
    clock.set(3000);
    super::configure(std::slice::from_mut(&mut conn), SchedulingMode::Enhanced);
    // Then learned rate survives, with new-generation validation rather than cold bootstrap.
    assert_eq!(conn.wire_rate.rate_bps(), 8_000_000.0);
    assert_eq!(conn.wire_rate.phase(), WireRatePhase::Searching);
    assert!(!conn.batch_sender.wire_limited());
    assert_eq!(conn.delivery.latest_original_delivery_ms(), None);
    assert_eq!(conn.rate_cap.target_bps(), 1_000_000.0);
    Ok(())
}
