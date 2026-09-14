use anyhow::Result;
use tokio::time::Instant;
use tracing::warn;

use super::SrtlaConnection;
use super::delivery::DataSend;
use crate::bind_map::IfaceName;
use crate::utils::now_ms;

#[cfg(test)]
#[path = "loss_io_tests.rs"]
mod loss_tests;

impl SrtlaConnection {
    /// Flush the batch queue, committing exactly the datagrams that went out.
    ///
    /// The accepted prefix is registered for in-flight tracking even when the
    /// transmit ended in a hard error — those packets are genuinely on the wire.
    /// An `Err` return means the caller must recover the link (mark it for
    /// recovery and drop its sequence-tracker entries); the unsent suffix stays
    /// queued and is discarded by that reset.
    pub async fn flush_batch(&mut self) -> Result<()> {
        if !self.batch_sender.has_queued_packets() {
            return Ok(());
        }

        let outcome = self.batch_sender.flush(&self.socket).await;
        let transmitted = !outcome.accepted.is_empty() || !outcome.probes.is_empty();
        let accepted_at_ms = now_ms();
        for ((seq, send_time_ms, len), retransmitted) in
            outcome.accepted.into_iter().zip(outcome.retransmitted)
        {
            if let Some(s) = seq {
                self.register_packet(s, send_time_ms);
                self.delivery.record_sent(
                    s,
                    DataSend {
                        sent_ms: accepted_at_ms,
                        len: u16::try_from(len)?,
                    },
                );
                if retransmitted {
                    self.delivery.mark_retransmitted(s);
                }
                let loss_send = self.loss.record_accepted_send(accepted_at_ms);
                self.delivery.record_loss_send(s, loss_send);
                if self.health.state() == super::health::HealthState::Stalled {
                    let window = super::adaptive::RecoveryWindow {
                        epoch_ms: self.health.entered_at_ms(),
                        // Original trains share no probe token bucket: m=1, same ACK deadline.
                        timeout_ms: 2000_u64.saturating_add(self.get_smooth_rtt_ms() as u64),
                    };
                    self.adaptive
                        .original_recovery
                        .record_sent(s, accepted_at_ms, window);
                }
            }
        }
        for probe in outcome.probes {
            self.probes
                .record_sent(probe.seq, probe.train, accepted_at_ms);
            self.bitrate.update_on_send(u64::try_from(probe.len)?);
        }
        self.advance_probes(accepted_at_ms);
        if transmitted {
            self.last_sent = Some(Instant::now());
        }
        match outcome.error {
            Some(e) => {
                if let Some(fault) = self.egress.note_send_error(&e) {
                    warn!(
                        "{}: egress fault {} on {}; the socket's interface binding is dead and \
                         will not be reused",
                        self.label,
                        fault.as_str(),
                        self.egress.iface().map_or("-", IfaceName::as_str)
                    );
                }
                Err(anyhow::anyhow!("batch flush failed: {}", e))
            }
            None => Ok(()),
        }
    }

    pub fn advance_probes(&mut self, now_ms: u64) {
        self.probes.advance(now_ms);
        self.loss.probe_loss = self.probes.probe_loss();
    }
}
