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
        let transmitted = !outcome.accepted.is_empty();
        let accepted_at_ms = now_ms();
        for (seq, send_time_ms, len) in outcome.accepted {
            if let Some(s) = seq {
                self.register_packet(s, send_time_ms);
                self.delivery.record_sent(
                    s,
                    DataSend {
                        sent_ms: accepted_at_ms,
                        len: u16::try_from(len)?,
                    },
                );
                self.loss.record_send(accepted_at_ms);
            }
        }
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
}
