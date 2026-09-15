//! Wire admission configuration after health/deadline ranking and before enqueue.

use crate::connection::SrtlaConnection;
use crate::connection::wire_rate::{WireRateInput, WireRttSample};
use crate::mode::SchedulingMode;
use crate::utils::now_ms;

#[cfg(test)]
#[path = "../connection/wire_rate/integration_tests.rs"]
mod wire_rate_tests;

pub(super) fn configure(conns: &mut [SrtlaConnection], mode: SchedulingMode) {
    let now = now_ms();
    for conn in conns {
        let rate = match mode {
            SchedulingMode::Adaptive => {
                let input = WireRateInput {
                    now_ms: now,
                    delivery_generation: conn.delivery.socket_generation,
                    accepted_bytes: conn
                        .batch_sender
                        .wire_sample()
                        .map(|sample| sample.accepted_bytes),
                    latest_original_delivery_ms: conn.delivery.latest_original_delivery_ms(),
                    latest_rtt: conn.has_rtt_sample().then_some(WireRttSample {
                        observed_ms: conn.rtt.last_rtt_measurement_ms,
                        rtt_ms: conn.rtt.prev_rtt_ms,
                    }),
                    slow_min_rtt_ms: conn.rtt.slow_min_rtt_ms(),
                    queue_delay_ms: conn.rtt.queue_delay_ms(),
                };
                conn.wire_rate.update(&input);
                // Adaptive's rate cap is a soft ranking penalty, never a hard
                // budget: enforcing the estimator can starve its own feedback.
                None
            }
            SchedulingMode::Classic
            | SchedulingMode::Enhanced
            | SchedulingMode::RttThreshold
            | SchedulingMode::Edpf => None,
        };
        conn.batch_sender.configure_wire_rate(rate, now);
    }
}

pub(super) fn funded_link(
    conns: &[SrtlaConnection],
    preferred: usize,
    bytes: usize,
) -> Option<usize> {
    let now = now_ms();
    if conns[preferred].batch_sender.can_queue_wire(bytes, now) {
        return Some(preferred);
    }
    conns
        .iter()
        .enumerate()
        .filter_map(|(index, conn)| {
            let weight = conn.adaptive.weight?;
            conn.batch_sender
                .can_queue_wire(bytes, now)
                .then_some((index, weight.score()))
        })
        .max_by(|(a, sa), (b, sb)| sa.total_cmp(sb).then_with(|| b.cmp(a)))
        .map(|(index, _)| index)
}

pub(super) fn queued(conns: &[SrtlaConnection]) -> bool {
    conns
        .iter()
        .any(|conn| conn.batch_sender.wire_limited() && conn.has_queued_packets())
}

#[cfg(test)]
mod tests {
    use tokio::net::UdpSocket;

    use super::*;
    use crate::connection::wire_rate::WireRatePhase;
    use crate::protocol::MTU;
    use crate::test_helpers::create_test_connection_to;
    use crate::utils::test_clock::TestClock;

    #[tokio::test]
    async fn adaptive_admits_and_flushes_beyond_a_congested_estimators_burst() {
        // Given an estimator cut from 1 Mbit/s to 500 kbit/s by queue evidence.
        let _clock = TestClock::new(2000);
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let mut conns = [create_test_connection_to(peer.local_addr().unwrap()).await];
        for now_ms in [1000, 1200] {
            conns[0].wire_rate.update(&WireRateInput {
                now_ms,
                delivery_generation: conns[0].delivery.socket_generation,
                accepted_bytes: Some(0),
                latest_original_delivery_ms: None,
                latest_rtt: Some(WireRttSample {
                    observed_ms: now_ms,
                    rtt_ms: 100.0,
                }),
                slow_min_rtt_ms: 60.0,
                queue_delay_ms: 20.0,
            });
        }
        assert_eq!(conns[0].wire_rate.rate_bps(), 500_000.0);
        assert_eq!(conns[0].wire_rate.phase(), WireRatePhase::Draining);

        // When adaptive configures a link with two MTUs already reserved, at a
        // fixed clock so the old hard budget cannot acquire more credit.
        configure(&mut conns, SchedulingMode::Adaptive);
        for seq in 0..2 {
            conns[0]
                .batch_sender
                .queue_packet(&[0; MTU], Some(seq), 2000);
        }
        configure(&mut conns, SchedulingMode::Adaptive);

        // Then a third packet is admitted and all three reach the actual socket:
        // neither reservation nor kernel-prefix funding can enforce the estimate.
        assert_eq!(funded_link(&conns, 0, MTU), Some(0));
        assert!(conns[0].batch_sender.can_queue_wire(MTU, 2000));
        assert!(
            !queued(&conns),
            "adaptive must not arm the wire-pacing wakeup"
        );
        conns[0].batch_sender.queue_packet(&[0; MTU], Some(2), 2000);
        let sent = conns[0].batch_sender.flush(&conns[0].socket).await;
        assert!(sent.error.is_none());
        assert_eq!(sent.accepted.len(), 3);
        for _ in 0..3 {
            assert_eq!(
                tokio::time::timeout(std::time::Duration::from_secs(1), peer.recv(&mut [0; MTU]))
                    .await
                    .unwrap()
                    .unwrap(),
                MTU
            );
        }
    }
}
