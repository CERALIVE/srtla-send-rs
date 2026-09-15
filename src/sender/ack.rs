use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::connection::delivery::DeliveryAck;
use crate::mode::SchedulingMode;

#[cfg(test)]
#[path = "adaptive_ack_rtt_tests.rs"]
mod adaptive_ack_rtt_tests;
#[cfg(test)]
#[path = "loss_recovery_tests.rs"]
mod loss_recovery_tests;
#[cfg(test)]
#[path = "recovery_evidence_tests.rs"]
mod recovery_evidence_tests;
#[cfg(test)]
#[path = "retransmit_accounting_tests.rs"]
mod retransmit_accounting_tests;
#[cfg(test)]
#[path = "sole_recovery_tests.rs"]
mod sole_recovery_tests;

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

#[cfg(test)]
pub fn apply_srtla_ack(connections: &mut [SrtlaConnection], srtla_ack: i32, context: AckContext) {
    apply_srtla_ack_frame(connections, &[u32::try_from(srtla_ack).unwrap()], context);
}

/// Credit every ACK, but sample adaptive RTT only from the final receiver-order
/// entry. Earlier entries include receiver coalescing delay; a final probe or
/// ambiguous/missing original must not fall back to one of those earlier entries.
pub fn apply_srtla_ack_frame(
    connections: &mut [SrtlaConnection],
    sequences: &[u32],
    context: AckContext,
) {
    let mut sample = None;
    for &seq in sequences {
        // The wire parser supplies only checked 31-bit sequence numbers.
        let Ok(seq) = i32::try_from(seq) else {
            sample = None;
            continue;
        };
        sample = apply_srtla_ack_entry(connections, seq, context);
    }
    if let Some((sent_ms, now)) = sample {
        connections[context.arrival_idx]
            .rtt
            .record_round_trip(sent_ms, now);
    }
}

fn apply_srtla_ack_entry(
    connections: &mut [SrtlaConnection],
    srtla_ack: i32,
    context: AckContext,
) -> Option<(u64, u64)> {
    match context.policy {
        AckPolicy::Adaptive => {
            let conn = connections.get_mut(context.arrival_idx)?;
            if context.reader_generation != conn.delivery.socket_generation {
                return None;
            }
            let now = crate::utils::now_ms();
            conn.advance_probes(now);
            let rtt_sent_ms = conn.delivery.rtt_sent_ms(srtla_ack);
            let loss_debits = conn.delivery.loss_debits(srtla_ack);
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
                for debit in loss_debits {
                    conn.loss.credit_recovered(debit, now);
                }
                conn.adaptive.original_recovery.acknowledge(srtla_ack, now);
                conn.apply_specific_ack(srtla_ack, false, false);
                return rtt_sent_ms.map(|sent_ms| (sent_ms, now));
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
    None
}
