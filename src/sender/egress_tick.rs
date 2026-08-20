//! The egress-binding step of a housekeeping tick.
//!
//! `SO_BINDTODEVICE` freezes an ifindex at bind time, and neither the kernel nor
//! the SRTLA protocol tells the sender when that index stops describing the
//! device. Nothing else in the tick would notice: the liveness timeout only
//! measures inbound silence, and a blackholed route produces no error at all. So
//! every tick re-resolves the interface by name and re-reads the route
//! invariant, which bounds detection to one housekeeping interval.
//!
//! Read-only with respect to the host: a name lookup and a route-table read. No
//! rule, route, or policy-routing table is ever installed.

use std::collections::HashMap;

use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, warn};

use super::sequence::SequenceTracker;
use super::uplink::{ConnectionId, ReaderHandle, UplinkPacket, restart_reader_for};
use crate::connection::egress::SystemIfaceResolver;
use crate::connection::route::{RouteTransition, classify_route_transition};
use crate::connection::{EgressPoll, RouteHealth, SrtlaConnection};

/// Whether the rest of the tick should still run for this link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickFlow {
    Continue,
    /// The link was removed or rebound; keepalives and registration would act on
    /// a socket that is gone or brand new.
    Skip,
}

pub async fn handle_egress(
    conn: &mut SrtlaConnection,
    reader_handles: &mut HashMap<ConnectionId, ReaderHandle>,
    packet_tx: &UnboundedSender<UplinkPacket>,
    seq_tracker: &mut SequenceTracker,
) -> TickFlow {
    let route_before = conn.route_health;
    match conn.poll_egress(&SystemIfaceResolver) {
        EgressPoll::Unchanged => {}
        EgressPoll::Reenumerated { from, to } => warn!(
            "{}: egress interface re-enumerated (ifindex {from} -> {to}); rebinding rather than \
             reusing the stale socket",
            conn.label
        ),
        EgressPoll::Removed => warn!(
            "{}: egress interface is gone; link removed, awaiting a reload that names an \
             interface that exists",
            conn.label
        ),
    }
    report_route_transition(conn, route_before);

    // A removed link has no interface to bind to, so reconnecting it would only
    // burn the backoff. It waits for a reload, not a retry.
    if conn.is_removed() {
        return TickFlow::Skip;
    }

    if !(conn.needs_rebind() && conn.should_attempt_reconnect()) {
        return TickFlow::Continue;
    }

    let label = conn.label.clone();
    conn.record_reconnect_attempt();
    if let Err(e) = conn.reconnect().await {
        warn!("{label}: failed to rebind after an egress fault: {e}");
        conn.mark_for_recovery();
    } else {
        restart_reader_for(conn, reader_handles, packet_tx);
    }
    // The rebind discarded this link's in-flight packet log, so any sequence it
    // still owns would misattribute a late NAK to a socket that never sent it.
    seq_tracker.remove_connection(conn.conn_id);
    TickFlow::Skip
}

fn report_route_transition(conn: &SrtlaConnection, before: RouteHealth) {
    match classify_route_transition(before, conn.route_health) {
        RouteTransition::Blackholed => warn!(
            "{}: egress interface lost its default route; traffic pinned to it is silently \
             blackholed (sendto keeps succeeding) — link health is now route={}",
            conn.label,
            conn.route_health.as_str()
        ),
        RouteTransition::Restored => info!(
            "{}: egress interface regained its default route (route={})",
            conn.label,
            conn.route_health.as_str()
        ),
        RouteTransition::Quiet => {}
    }
}
