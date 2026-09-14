//! Hard budget admission, after health/deadline ranking and before enqueue.

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
                Some(conn.wire_rate.rate_bps())
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
