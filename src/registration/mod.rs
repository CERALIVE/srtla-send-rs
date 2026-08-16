mod probing;

use std::collections::HashSet;

use probing::{ProbeResult, ProbingState, default_probing_state, new_probe_id, new_probe_results};
use rand::Rng;
use smallvec::SmallVec;
use tracing::{debug, info, warn};

use crate::connection::SrtlaConnection;
use crate::protocol::*;
use crate::utils::now_ms;

#[derive(Debug)]
pub enum RegistrationEvent {
    RegNgp,
    Reg2,
    Reg3,
    /// A REG3 that arrived on an uplink we never sent a REG2 to. Consumed and
    /// ignored: it must not promote the link to connected.
    Reg3OutOfPhase,
    RegErr,
    /// A REG_ERR that arrived on an uplink which is not mid-registration.
    /// Consumed and ignored: it must not tear down an established link.
    RegErrOutOfPhase,
}

pub struct SrtlaRegistrationManager {
    pub srtla_id: [u8; SRTLA_ID_LEN],
    pending_reg2_idx: Option<usize>,
    pub(crate) pending_timeout_at_ms: u64,
    pub(crate) active_connections: usize,
    pub has_connected: bool,
    broadcast_reg2_pending: bool,
    pub(crate) reg1_target_idx: Option<usize>,
    pub(crate) reg1_next_send_at_ms: u64,
    probing_state: ProbingState,
    probe_id: [u8; SRTLA_ID_LEN],
    probe_results: SmallVec<ProbeResult, 4>,
    /// Uplink indices we have successfully sent a REG2 to. A REG3 is only
    /// honored for a member: SRTLA has no per-packet authentication, so without
    /// this gate any host that can reach an uplink's ephemeral port could
    /// promote a never-registered link to connected.
    awaiting_reg3: HashSet<usize>,
    /// REG3 frames rejected by the phase gate (diagnostic).
    out_of_phase_reg3: u64,
    /// REG_ERR frames rejected by the phase gate (diagnostic).
    out_of_phase_reg_err: u64,
}

impl Default for SrtlaRegistrationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SrtlaRegistrationManager {
    pub fn new() -> Self {
        let mut id = [0u8; SRTLA_ID_LEN];
        rand::rng().fill_bytes(&mut id);
        Self {
            srtla_id: id,
            pending_reg2_idx: None,
            pending_timeout_at_ms: 0,
            active_connections: 0,
            has_connected: false,
            broadcast_reg2_pending: false,
            reg1_target_idx: None,
            reg1_next_send_at_ms: 0,
            probing_state: default_probing_state(),
            probe_id: new_probe_id(),
            probe_results: new_probe_results(),
            awaiting_reg3: HashSet::new(),
            out_of_phase_reg3: 0,
            out_of_phase_reg_err: 0,
        }
    }

    /// Send REG1 and advance the pending-REG2 state ONLY on a successful send.
    ///
    /// A failed send must leave the handshake immediately retriable: arming the
    /// pending window on a packet that never left the host would stall
    /// registration for the whole `REG2_TIMEOUT` on a phantom.
    pub async fn send_reg1_to(&mut self, conn_idx: usize, conn: &mut SrtlaConnection) {
        let pkt = create_reg1_packet(&self.srtla_id);
        debug!("queueing REG1 for uplink #{}", conn_idx);
        info!("REG1 → uplink #{} ({} bytes)", conn_idx, pkt.len());
        if let Err(e) = conn.send_srtla_packet(&pkt).await {
            warn!("Failed to send REG1 to uplink #{}: {:?}", conn_idx, e);
            self.reg1_target_idx = Some(conn_idx);
            self.reg1_next_send_at_ms = now_ms();
            return;
        }

        let now = now_ms();
        self.pending_reg2_idx = Some(conn_idx);
        self.reg1_target_idx = Some(conn_idx);
        self.pending_timeout_at_ms = now + REG2_TIMEOUT * 1000;
        // Allow the driver to retry at a 1s cadence while waiting on REG2
        self.reg1_next_send_at_ms = now + 1000;
    }

    /// Send REG2 and arm the REG3 phase gate for this uplink ONLY on success.
    pub async fn send_reg2_to(&mut self, conn_idx: usize, conn: &mut SrtlaConnection) {
        let pkt = create_reg2_packet(&self.srtla_id);
        debug!("queueing REG2 for uplink #{}", conn_idx);
        info!("REG2 → uplink #{} ({} bytes)", conn_idx, pkt.len());
        if let Err(e) = conn.send_srtla_packet(&pkt).await {
            warn!("Failed to send REG2 to uplink #{}: {:?}", conn_idx, e);
            // A resend that never left the host must also revoke any grant left
            // over from an earlier REG2 on this index: the socket generation it
            // was issued against may already be gone.
            self.awaiting_reg3.remove(&conn_idx);
            return;
        }
        self.awaiting_reg3.insert(conn_idx);
    }

    /// `conn_connected` is the uplink's own authoritative
    /// [`SrtlaConnection::connected`] flag. It is passed in because the
    /// manager's `active_connections` counter is only recomputed by a later
    /// housekeeping tick, so it is briefly stale after a REG3 promotes a link.
    pub fn process_registration_packet(
        &mut self,
        conn_idx: usize,
        buf: &[u8],
        conn_connected: bool,
    ) -> Option<RegistrationEvent> {
        match get_packet_type(buf) {
            Some(SRTLA_TYPE_REG_NGP) => {
                debug!("REG_NGP from uplink #{}", conn_idx);
                self.handle_reg_ngp(conn_idx, conn_connected);
                Some(RegistrationEvent::RegNgp)
            }
            Some(SRTLA_TYPE_REG2) => {
                debug!("REG2 from uplink #{} (len={})", conn_idx, buf.len());
                self.handle_reg2(conn_idx, buf);
                Some(RegistrationEvent::Reg2)
            }
            Some(SRTLA_TYPE_REG3) => {
                debug!("REG3 from uplink #{}", conn_idx);
                if self.handle_reg3(conn_idx) {
                    Some(RegistrationEvent::Reg3)
                } else {
                    Some(RegistrationEvent::Reg3OutOfPhase)
                }
            }
            Some(SRTLA_TYPE_REG_ERR) => {
                debug!("REG_ERR from uplink #{}", conn_idx);
                if self.handle_reg_err(conn_idx) {
                    Some(RegistrationEvent::RegErr)
                } else {
                    Some(RegistrationEvent::RegErrOutOfPhase)
                }
            }
            _ => None,
        }
    }

    pub async fn reg_driver_send_if_needed(&mut self, connections: &mut [SrtlaConnection]) {
        // If nothing connected yet, send REG1. Prefer target from REG_NGP; otherwise,
        // pick the first uplink.
        if self.active_connections == 0 {
            if let Some(idx) = self.reg1_target_idx {
                let now = now_ms();
                if self.pending_reg2_idx.is_none() && now >= self.reg1_next_send_at_ms {
                    let pkt = create_reg1_packet(&self.srtla_id);
                    info!("REG1 → uplink #{} ({} bytes)", idx, pkt.len());
                    match connections[idx].send_srtla_packet(&pkt).await {
                        Ok(()) => {
                            self.pending_reg2_idx = Some(idx);
                            self.pending_timeout_at_ms = now + REG2_TIMEOUT * 1000;
                            // throttle retries until next REG_NGP/timeout
                            self.reg1_next_send_at_ms = now + REG2_TIMEOUT * 1000;
                        }
                        Err(e) => {
                            // Send-gated: stay retriable on the next tick instead
                            // of opening a pending window on a packet that never
                            // left the host.
                            warn!("Failed to send REG1 to uplink #{}: {:?}", idx, e);
                            self.reg1_next_send_at_ms = now;
                        }
                    }
                } else if self.pending_reg2_idx.is_some() {
                    debug!(
                        "REG1 pending for uplink #{} (timeout at {}), skipping send",
                        idx, self.pending_timeout_at_ms
                    );
                }
            } else {
                debug!("No REG1 target selected; awaiting REG_NGP");
            }
        }

        // If flagged by REG2 reception, broadcast REG2 once to all uplinks
        if self.broadcast_reg2_pending {
            let pkt = create_reg2_packet(&self.srtla_id);
            info!(
                "broadcast REG2 to {} uplinks ({} bytes)",
                connections.len(),
                pkt.len()
            );
            let mut any_failed = false;
            for (i, c) in connections.iter_mut().enumerate() {
                // Re-sending REG2 to a link that is already connected, or that
                // still holds a live grant from an earlier pass, would re-arm
                // the ONE-SHOT `awaiting_reg3` gate `handle_reg3` consumed —
                // authorizing a receiver-retransmitted REG3 to wipe a live,
                // forwarding uplink's state.
                if c.connected || self.awaiting_reg3.contains(&i) {
                    debug!(
                        "REG2 → uplink #{} skipped (connected={}, awaiting_reg3={})",
                        i,
                        c.connected,
                        self.awaiting_reg3.contains(&i)
                    );
                    continue;
                }
                match c.send_srtla_packet(&pkt).await {
                    Ok(()) => {
                        self.awaiting_reg3.insert(i);
                        debug!("REG2 → uplink #{} sent", i);
                    }
                    Err(e) => {
                        any_failed = true;
                        warn!("REG2 → uplink #{} failed: {:?}", i, e);
                    }
                }
            }
            // Send-gated: an incomplete broadcast is retried on the next tick.
            self.broadcast_reg2_pending = any_failed;
        }
    }

    /// Accepting a REG_NGP restarts the handshake from REG1, which re-opens the
    /// `pending_reg2_idx` window a REG_ERR is honored in. A forged 2-byte
    /// REG_NGP must therefore never be able to re-arm that window on a session
    /// that is already past it, so acceptance requires that *nothing* is in
    /// flight anywhere: no pending REG2, no outstanding REG3 grant, and no
    /// established link.
    ///
    /// The `active_connections` counter alone is not sufficient for the last
    /// condition: it is recomputed only by housekeeping, so it still reads zero
    /// for a link whose REG3 was processed moments ago. `conn_connected` is that
    /// link's own flag, set in the same dispatch that consumed the REG3, and
    /// closes the staleness window.
    fn handle_reg_ngp(&mut self, conn_idx: usize, conn_connected: bool) {
        if self.probing_state == ProbingState::WaitingForProbes {
            self.handle_probe_response(conn_idx);
            return;
        }

        if self.active_connections == 0
            && !conn_connected
            && self.pending_reg2_idx.is_none()
            && self.awaiting_reg3.is_empty()
        {
            debug!("REG_NGP from uplink #{} accepted as REG1 target", conn_idx);
            self.reg1_target_idx = Some(conn_idx);
            self.reg1_next_send_at_ms = now_ms();
        } else {
            debug!(
                "REG_NGP from uplink #{} ignored (connected={}, active={}, pending_reg2={:?}, \
                 awaiting_reg3={})",
                conn_idx,
                conn_connected,
                self.active_connections,
                self.pending_reg2_idx,
                self.awaiting_reg3.len()
            );
        }
    }

    fn handle_reg2(&mut self, conn_idx: usize, buf: &[u8]) {
        if buf.len() < 2 + SRTLA_ID_LEN {
            return;
        }
        if self.pending_reg2_idx == Some(conn_idx) {
            // server returns full id starting at byte 2
            let full_id = &buf[2..2 + SRTLA_ID_LEN];
            // The receiver only replaces the LAST half of the id, so the first
            // half must still be ours. Rejecting a mismatch stops an off-path
            // host from hijacking the group with a forged REG2.
            let ours = SRTLA_ID_LEN / 2;
            if full_id[..ours] != self.srtla_id[..ours] {
                warn!(
                    "REG2 from uplink #{} rejected: returned id prefix does not match our sender \
                     id",
                    conn_idx
                );
                return;
            }
            self.srtla_id.copy_from_slice(full_id);
            debug!(
                "REG2 from uplink #{} accepted; broadcasting to peers",
                conn_idx
            );
            self.pending_reg2_idx = None;
            self.pending_timeout_at_ms = now_ms() + REG3_TIMEOUT * 1000;
            self.broadcast_reg2_pending = true;
            // stop sending REG1 until next REG_NGP
            self.reg1_target_idx = None;
            self.reg1_next_send_at_ms = 0;
        }
    }

    fn handle_reg3(&mut self, conn_idx: usize) -> bool {
        if !self.awaiting_reg3.contains(&conn_idx) {
            self.out_of_phase_reg3 = self.out_of_phase_reg3.saturating_add(1);
            warn!(
                "REG3 for uplink #{} ignored: no REG2 was sent on it ({} out-of-phase so far)",
                conn_idx, self.out_of_phase_reg3
            );
            return false;
        }
        // The grant is ONE-SHOT: consume it here so a duplicate or replayed
        // REG3 for the same index falls through to the out-of-phase branch.
        // Without this, every later REG3 on an already-connected uplink would
        // re-fire `RegistrationEvent::Reg3` and make the caller wipe live
        // forwarding state (`clear_pre_registration_state`). A legitimate
        // reconnect re-arms the gate through `send_reg2_to`.
        self.awaiting_reg3.remove(&conn_idx);
        self.has_connected = true;
        true
    }

    /// Returns `true` only when the REG_ERR was a meaningful protocol reply to
    /// a handshake this index actually has in flight.
    ///
    /// SRTLA control frames are unauthenticated and the uplink sockets are
    /// unconnected, so anything that can reach an uplink's ephemeral port can
    /// forge a 2-byte REG_ERR. Acting on one for an established link would let
    /// that host force-disconnect a live, forwarding uplink; acting on one with
    /// the global REG1/REG2 fields would additionally abort an *unrelated*
    /// uplink's concurrent handshake.
    fn handle_reg_err(&mut self, conn_idx: usize) -> bool {
        let awaiting_reg2 = self.pending_reg2_idx == Some(conn_idx);
        let awaiting_reg3 = self.awaiting_reg3.contains(&conn_idx);

        if !awaiting_reg2 && !awaiting_reg3 {
            self.out_of_phase_reg_err = self.out_of_phase_reg_err.saturating_add(1);
            warn!(
                "REG_ERR for uplink #{} ignored: no registration in flight on it ({} out-of-phase \
                 so far)",
                conn_idx, self.out_of_phase_reg_err
            );
            return false;
        }

        if awaiting_reg2 {
            debug!("REG_ERR for uplink #{} while awaiting REG2", conn_idx);
            self.pending_reg2_idx = None;
            self.pending_timeout_at_ms = 0;
            self.reg1_target_idx = None;
            // Wait for a fresh REG_NGP to select the next REG1 target
            self.reg1_next_send_at_ms = now_ms() + REG2_TIMEOUT * 1000;
        } else {
            debug!("REG_ERR for uplink #{} while awaiting REG3", conn_idx);
        }
        self.awaiting_reg3.remove(&conn_idx);

        warn!("registration failed for connection {}", conn_idx);
        true
    }

    pub async fn try_send_reg1_immediately(&mut self, conn_idx: usize, conn: &mut SrtlaConnection) {
        if self.active_connections == 0
            && self.pending_reg2_idx.is_none()
            && self.reg1_target_idx == Some(conn_idx)
            && now_ms() >= self.reg1_next_send_at_ms
        {
            debug!("REG_NGP immediate send for uplink #{}", conn_idx);
            self.send_reg1_to(conn_idx, conn).await;
        }
    }

    pub fn update_active_connections(&mut self, connections: &[SrtlaConnection]) {
        // Match C implementation: recalculate from scratch each housekeeping cycle
        // We count connections that are actually connected (received REG3), not just within grace period
        let new_count = connections.iter().filter(|c| c.connected).count();

        // Log any changes in connection count
        if new_count != self.active_connections {
            if new_count > self.active_connections {
                info!("connection established (active={})", new_count);
            } else {
                info!("connection(s) lost - active connections: {}", new_count);
            }
        }

        // Reset and recalculate - this is the authoritative count
        self.active_connections = new_count;
    }

    pub(crate) fn pending_reg2_idx(&self) -> Option<usize> {
        self.pending_reg2_idx
    }

    pub fn clear_pending_if_timed_out(&mut self, now_ms_value: u64) -> Option<usize> {
        if let Some(idx) = self.pending_reg2_idx
            && self.pending_timeout_at_ms != 0
            && now_ms_value >= self.pending_timeout_at_ms
        {
            warn!(
                "REG2 wait exceeded {}ms for uplink #{}; clearing pending handshake",
                REG2_TIMEOUT * 1000,
                idx
            );
            self.pending_reg2_idx = None;
            self.pending_timeout_at_ms = 0;
            self.reg1_target_idx = None;
            self.reg1_next_send_at_ms = now_ms_value;
            return Some(idx);
        }
        None
    }

    pub fn get_selected_connection_idx(&self) -> Option<usize> {
        self.reg1_target_idx
    }

    /// Drop every piece of registration bookkeeping that is keyed by a
    /// **positional index** into the sender's connection vector.
    ///
    /// A SIGHUP reload rebuilds that vector in ips-file order, so an index can
    /// change occupant. Carrying `awaiting_reg3` / `pending_reg2_idx` /
    /// `reg1_target_idx` / probe results across the rebuild would let one
    /// uplink's in-flight registration grant authorize a REG3 on whichever
    /// uplink inherits its index. Only *incomplete* attempts are discarded:
    /// established links keep their own `SrtlaConnection::connected` state and
    /// move with the reorder, so nothing already registered is disturbed.
    pub(crate) fn reset_index_scoped_state(&mut self) {
        self.awaiting_reg3.clear();
        self.pending_reg2_idx = None;
        self.pending_timeout_at_ms = 0;
        self.reg1_target_idx = None;
        self.reg1_next_send_at_ms = now_ms();
        self.reset_probe_state();
    }
}

// Test-only accessor methods for controlled field access
#[cfg(test)]
#[allow(dead_code)]
impl SrtlaRegistrationManager {
    pub(crate) fn srtla_id(&self) -> &[u8; SRTLA_ID_LEN] {
        &self.srtla_id
    }

    pub(crate) fn active_connections(&self) -> usize {
        self.active_connections
    }

    pub(crate) fn has_connected(&self) -> bool {
        self.has_connected
    }

    pub(crate) fn broadcast_reg2_pending(&self) -> bool {
        self.broadcast_reg2_pending
    }

    pub(crate) fn reg1_target_idx(&self) -> Option<usize> {
        self.reg1_target_idx
    }

    pub(crate) fn set_reg1_target_idx(&mut self, value: Option<usize>) {
        self.reg1_target_idx = value;
    }

    pub(crate) fn reg1_next_send_at_ms(&self) -> u64 {
        self.reg1_next_send_at_ms
    }

    pub(crate) fn pending_timeout_at_ms(&self) -> u64 {
        self.pending_timeout_at_ms
    }

    // Mutable accessors for tests that need to modify state
    pub(crate) fn set_pending_reg2_idx(&mut self, value: Option<usize>) {
        self.pending_reg2_idx = value;
    }

    pub(crate) fn set_pending_timeout_at_ms(&mut self, value: u64) {
        self.pending_timeout_at_ms = value;
    }

    pub(crate) fn set_reg1_next_send_at_ms(&mut self, value: u64) {
        self.reg1_next_send_at_ms = value;
    }

    pub(crate) fn set_broadcast_reg2_pending(&mut self, value: bool) {
        self.broadcast_reg2_pending = value;
    }

    pub(crate) fn out_of_phase_reg3(&self) -> u64 {
        self.out_of_phase_reg3
    }

    pub(crate) fn out_of_phase_reg_err(&self) -> u64 {
        self.out_of_phase_reg_err
    }

    pub(crate) fn arm_reg3_gate(&mut self, conn_idx: usize) {
        self.awaiting_reg3.insert(conn_idx);
    }

    pub(crate) fn is_awaiting_reg3(&self, conn_idx: usize) -> bool {
        self.awaiting_reg3.contains(&conn_idx)
    }
}
