use proptest::prelude::*;

use super::SrtlaConnection;
use crate::connection::delivery::DataSend;
use crate::test_helpers::create_test_connection;
use crate::utils::now_ms;
use crate::utils::test_clock::TestClock;

// Frozen pre-Todo-8 handle_nak body (3ad8998), intentionally independent of the gate.
fn reference_handle_nak(conn: &mut SrtlaConnection, seq: i32) -> bool {
    let sent = conn.packet_log.remove(&seq);
    if let Some(queued_ms) = sent {
        let debit = match conn.delivery.loss_send(seq) {
            Some(send) => conn.loss.debit_accepted_send(send, now_ms()),
            None => conn
                .loss
                .debit_data_nak(conn.delivery.sent_ms(seq).unwrap_or(queued_ms), now_ms()),
        };
        if let Some(debit) = debit {
            conn.delivery.record_loss_debit(seq, debit);
        }
        conn.in_flight_packets = conn.packet_log.len() as i32;
        conn.congestion
            .handle_nak(&mut conn.window, seq, &conn.label);
    }
    sent.is_some()
}

proptest! {
    #[test]
    fn premature_nak_mature_trajectory_matches_old_body(
        rtt in 1_u16..=10_000,
        window in 1000_i32..100_000,
        reports in prop::collection::vec((0_i32..64, 0_u64..3000, 0_u64..16, any::<bool>()), 1..150),
    ) {
        // Given the original handler and the new handler with identical send histories.
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let clock = TestClock::new(10_000);
            let mut actual = create_test_connection().await;
            let mut reference = create_test_connection().await;
            actual.window = window;
            reference.window = window;
            actual.rtt.update_estimate(u64::from(rtt));
            reference.rtt.update_estimate(u64::from(rtt));
            let threshold = u64::from(rtt).div_ceil(2).clamp(5, 500);
            let mut accepted = 10_000;
            for (seq, extra_age, queue_delay, tracked) in reports {
                clock.set(accepted);
                for conn in [&mut actual, &mut reference] {
                    conn.register_packet(seq, accepted - queue_delay);
                    conn.delivery.record_sent(seq, DataSend { sent_ms: accepted, len: 1316 });
                    let send = conn.loss.record_accepted_send(accepted);
                    if tracked { conn.delivery.record_loss_send(seq, send); }
                }
                clock.set(accepted + threshold + extra_age);
                // When every NAK is mature, including repeated sequence use and varied batch delay.
                let handled = actual.handle_nak(seq);
                let old_handled = reference_handle_nak(&mut reference, seq);
                // Then the complete window trajectory and existing accounting match bit-for-bit.
                prop_assert_eq!(handled, old_handled);
                prop_assert_eq!(actual.window, reference.window);
                prop_assert_eq!(&actual.packet_log, &reference.packet_log);
                prop_assert_eq!(actual.in_flight_packets, reference.in_flight_packets);
                prop_assert_eq!(format!("{:?}", actual.congestion), format!("{:?}", reference.congestion));
                prop_assert_eq!(format!("{:?}", actual.loss), format!("{:?}", reference.loss));
                prop_assert_eq!(actual.premature_nak_count, 0);
                accepted = now_ms() + 1;
            }
            Ok(())
        })?;
    }
}
