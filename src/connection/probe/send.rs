use super::{ProbeScheduler, ProbeTarget};
use crate::connection::SrtlaConnection;

pub struct ProbeOpportunity<'a> {
    pub primary_conn_id: u64,
    pub packet: &'a [u8],
    pub targets: &'a [ProbeTarget],
}

/// On a flush error, the forwarding owner must recover conn_id and remove its
/// normal SequenceTracker entries, just as for every other batch-flush error.
pub struct ProbeEmission {
    pub conn_id: u64,
    pub outcome: anyhow::Result<()>,
}

impl ProbeScheduler {
    /// Offer DATA only after its normal send succeeded. No production selector
    /// calls this yet; Todo 22 owns the health/deadline/sole-carrier snapshot.
    pub async fn emit(
        &mut self,
        connections: &mut [SrtlaConnection],
        offer: ProbeOpportunity<'_>,
    ) -> Option<ProbeEmission> {
        if offer.packet.len() < 16 || offer.packet[0] & 0x80 != 0 {
            return None;
        }
        let dispatch = self.next(offer.targets, crate::utils::now_ms())?;
        if dispatch.conn_id == offer.primary_conn_id {
            return None;
        }
        let conn = connections
            .iter_mut()
            .find(|c| c.conn_id == dispatch.conn_id)?;
        if !conn.connected
            || conn.is_timed_out()
            || conn.delivery.socket_generation != dispatch.socket_generation
        {
            return None;
        }
        // Never stack paced probes behind an unsent suffix and later burst them.
        if conn.has_queued_packets() {
            return None;
        }
        if !conn.queue_probe_packet(offer.packet, dispatch) {
            return None;
        }
        let outcome = conn.flush_batch().await;
        self.last_ms = Some(crate::utils::now_ms());
        self.credit = 0;
        Some(ProbeEmission {
            conn_id: conn.conn_id,
            outcome,
        })
    }
}
