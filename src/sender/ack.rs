use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::connection::delivery::DeliveryAck;
use crate::mode::SchedulingMode;

#[derive(Clone, Copy, Debug)]
pub enum AckPolicy {
    Legacy {
        classic: bool,
        earned_ack_window: bool,
    },
    Adaptive,
}

impl AckPolicy {
    /// Exhaustive integration seam: adding Adaptive to SchedulingMode requires
    /// explicitly selecting arrival-scoped attribution here, never a wildcard.
    pub const fn from_config(config: &ConfigSnapshot) -> Self {
        match config.mode {
            SchedulingMode::Adaptive => Self::Adaptive,
            SchedulingMode::Classic => Self::Legacy {
                classic: true,
                earned_ack_window: config.earned_ack_window,
            },
            SchedulingMode::Enhanced | SchedulingMode::RttThreshold | SchedulingMode::Edpf => {
                Self::Legacy {
                    classic: false,
                    earned_ack_window: config.earned_ack_window,
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AckContext {
    pub arrival_idx: usize,
    pub reader_generation: u32,
    pub policy: AckPolicy,
}

#[cfg(test)]
pub(crate) fn legacy_test_ack(
    connections: &mut [SrtlaConnection],
    seq: i32,
    classic: bool,
    earned_ack_window: bool,
) {
    apply_srtla_ack(
        connections,
        seq,
        AckContext::legacy((0, 0), classic, earned_ack_window),
    );
}

impl AckContext {
    pub const fn legacy(arrival: (usize, u32), classic: bool, earned_ack_window: bool) -> Self {
        Self {
            arrival_idx: arrival.0,
            reader_generation: arrival.1,
            policy: AckPolicy::Legacy {
                classic,
                earned_ack_window,
            },
        }
    }
}

pub fn apply_srtla_ack(connections: &mut [SrtlaConnection], srtla_ack: i32, context: AckContext) {
    match context.policy {
        AckPolicy::Adaptive => {
            let Some(conn) = connections.get_mut(context.arrival_idx) else {
                return;
            };
            if context.reader_generation != conn.delivery.socket_generation {
                return;
            }
            let now = crate::utils::now_ms();
            conn.advance_probes(now);
            if conn.probes.acknowledge(srtla_ack, now) {
                conn.delivery.record_probe_proof(now);
                conn.advance_probes(now);
            } else if conn.delivery.acknowledge(
                DeliveryAck {
                    seq: srtla_ack,
                    socket_generation: context.reader_generation,
                },
                now,
            ) {
                // Only an original on THIS link may clear its congestion accounting.
                conn.handle_srtla_ack_specific(srtla_ack, false);
            }
        }
        AckPolicy::Legacy {
            classic,
            earned_ack_window,
        } => {
            let mut earned_idx: Option<usize> = None;
            for (i, c) in connections.iter_mut().enumerate() {
                if c.handle_srtla_ack_specific(srtla_ack, classic) {
                    earned_idx = Some(i);
                    break;
                }
            }
            if earned_ack_window {
                let now_ms = crate::utils::now_ms();
                for (i, c) in connections.iter_mut().enumerate() {
                    c.handle_srtla_ack_earned(Some(i) == earned_idx, now_ms);
                }
            } else {
                for c in connections.iter_mut() {
                    c.handle_srtla_ack_global();
                }
            }
        }
    }
}
