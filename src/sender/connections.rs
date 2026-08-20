use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use smallvec::SmallVec;
use tracing::{info, warn};

use super::sequence::SequenceTracker;
use crate::bind_map::EffectiveLink;
use crate::connection::{SocketKey, SrtlaConnection, UplinkSpec};
use crate::registration::SrtlaRegistrationManager;

pub struct PendingConnectionChanges {
    pub new_links: Option<SmallVec<UplinkSpec, 4>>,
    pub receiver_host: String,
    pub receiver_port: u16,
}

/// Turn a resolved bind-map decision into uplink specs, in file order.
pub fn specs_from_effective_links(links: &[EffectiveLink]) -> SmallVec<UplinkSpec, 4> {
    links
        .iter()
        .map(|link| UplinkSpec {
            ip: link.ip,
            iface: link.iface.clone(),
            link_id: link.link_id.clone(),
        })
        .collect()
}

/// Legacy shape: uplinks identified by source IP alone, no bind-map.
pub fn specs_from_ips(ips: &[IpAddr]) -> SmallVec<UplinkSpec, 4> {
    ips.iter().copied().map(UplinkSpec::unmapped).collect()
}

/// Apply a reloaded uplink IP list to the live connection pool.
///
/// Thin legacy shim over [`apply_link_changes`]: every IP becomes an unmapped
/// spec, so dedup falls back to the source IP and nothing is device-bound.
pub async fn apply_connection_changes(
    connections: &mut SmallVec<SrtlaConnection, 4>,
    new_ips: &[IpAddr],
    receiver_host: &str,
    receiver_port: u16,
    last_selected_idx: &mut Option<usize>,
    seq_tracker: &mut SequenceTracker,
    reg: &mut SrtlaRegistrationManager,
) {
    apply_link_changes(
        connections,
        &specs_from_ips(new_ips),
        receiver_host,
        receiver_port,
        last_selected_idx,
        seq_tracker,
        reg,
    )
    .await;
}

/// Apply a reloaded uplink set to the live connection pool.
///
/// The pool is rebuilt in **ips-file order**: a surviving uplink keeps its
/// existing socket and registration state (no re-handshake, no disconnect) but
/// is repositioned to match the file, and a removed uplink is torn down. Because
/// the telemetry `conn_id` is the link's index in this vector (Task 10 ADR-001
/// contract), reordering the file reorders `conn_id` consistently — matching the
/// C sender's "reload reassigns by file order" semantics.
///
/// Survival is decided by **identity**, not by socket key: a mapped link whose
/// `link_id` is unchanged survives even when its IP or interface moved — but it
/// survives as a *fresh socket*, because every piece of state the old socket
/// held is scoped to an ifindex that no longer describes it.
///
/// Any change to the vector also invalidates `reg`'s index-keyed registration
/// bookkeeping, which is reset here — see
/// [`SrtlaRegistrationManager::reset_index_scoped_state`].
pub async fn apply_link_changes(
    connections: &mut SmallVec<SrtlaConnection, 4>,
    new_links: &[UplinkSpec],
    receiver_host: &str,
    receiver_port: u16,
    last_selected_idx: &mut Option<usize>,
    seq_tracker: &mut SequenceTracker,
    reg: &mut SrtlaRegistrationManager,
) {
    let mut seen = HashSet::<SocketKey>::new();
    let desired: SmallVec<UplinkSpec, 4> = new_links
        .iter()
        .filter(|spec| seen.insert(spec.socket_key()))
        .cloned()
        .collect();

    let previous_order: SmallVec<u64, 4> = connections.iter().map(|c| c.conn_id).collect();
    let mut survivors = Survivors::take(connections);

    // Two passes: claim every survivor first, so a link whose socket key moved
    // cannot be outbid by a later row that happens to want its old key.
    let claimed: SmallVec<Option<SrtlaConnection>, 4> =
        desired.iter().map(|spec| survivors.claim(spec)).collect();
    let attempted_new = claimed.iter().filter(|c| c.is_none()).count();

    let mut fresh = create_missing(&desired, &claimed, receiver_host, receiver_port).await;
    let added = fresh.len();

    // Drop the leaving links' sequence entries so a stale NAK can't be
    // misattributed to whichever uplink later reuses that index.
    let removed = survivors.retire(seq_tracker);

    for (spec, existing) in desired.iter().zip(claimed) {
        match existing {
            Some(conn) => connections.push(conn),
            None => {
                if let Some(conn) = fresh.remove(&spec.socket_key()) {
                    connections.push(conn);
                }
            }
        }
    }

    if removed > 0 {
        info!("removed {removed} stale connection(s)");
    }
    if added > 0 {
        info!("added {added} new connection(s)");
    } else if attempted_new > 0 {
        warn!("failed to add any new connections (attempted {attempted_new})");
    }

    // A reorder invalidates the cached selection index, not just a removal.
    let new_order: SmallVec<u64, 4> = connections.iter().map(|c| c.conn_id).collect();
    if previous_order != new_order {
        *last_selected_idx = None;
        reg.reset_index_scoped_state();
    }
}

/// The pre-reload pool, indexed both ways so a link can be matched by identity
/// first and by socket key second.
struct Survivors {
    by_link_id: HashMap<String, SrtlaConnection>,
    by_key: HashMap<SocketKey, SrtlaConnection>,
}

impl Survivors {
    fn take(connections: &mut SmallVec<SrtlaConnection, 4>) -> Self {
        let mut by_link_id = HashMap::new();
        let mut by_key = HashMap::new();
        for conn in std::mem::take(connections) {
            match conn.link_id.as_ref() {
                Some(id) => {
                    by_link_id.insert(id.as_str().to_string(), conn);
                }
                None => {
                    by_key.insert(conn.spec().socket_key(), conn);
                }
            }
        }
        Self { by_link_id, by_key }
    }

    /// Claim the live connection that `spec` describes, if its socket is still
    /// the right one.
    ///
    /// A mapped link whose socket key moved is deliberately **not** returned:
    /// its window, packet log, in-flight count, and registration are all scoped
    /// to the interface it was bound to, so carrying them onto a new interface
    /// would credit the new link with the old one's history.
    fn claim(&mut self, spec: &UplinkSpec) -> Option<SrtlaConnection> {
        let key = spec.socket_key();
        let Some(id) = spec.link_id.as_ref() else {
            // Legacy identity: the socket key IS the link. A previously-mapped
            // survivor still matches on it, so falling back to legacy does not
            // needlessly tear down a bond that is otherwise unchanged.
            return self.by_key.remove(&key).or_else(|| {
                let matched = self
                    .by_link_id
                    .iter()
                    .find(|(_, conn)| conn.spec().socket_key() == key)
                    .map(|(id, _)| id.clone())?;
                self.by_link_id.remove(&matched)
            });
        };
        let existing = self.by_link_id.get(id.as_str())?;
        (existing.spec().socket_key() == key)
            .then(|| self.by_link_id.remove(id.as_str()))
            .flatten()
    }

    /// Retire everything left unclaimed, returning how many links left the pool.
    fn retire(self, seq_tracker: &mut SequenceTracker) -> usize {
        let leaving: SmallVec<u64, 4> = self
            .by_link_id
            .values()
            .chain(self.by_key.values())
            .map(|c| c.conn_id)
            .collect();
        for conn_id in &leaving {
            seq_tracker.remove_connection(*conn_id);
        }
        leaving.len()
    }
}

/// Dial every desired link that no survivor covers.
async fn create_missing(
    desired: &[UplinkSpec],
    claimed: &[Option<SrtlaConnection>],
    receiver_host: &str,
    receiver_port: u16,
) -> HashMap<SocketKey, SrtlaConnection> {
    let missing: SmallVec<UplinkSpec, 4> = desired
        .iter()
        .zip(claimed)
        .filter(|(_, existing)| existing.is_none())
        .map(|(spec, _)| spec.clone())
        .collect();
    if missing.is_empty() {
        return HashMap::new();
    }
    create_connections_from_links(&missing, receiver_host, receiver_port)
        .await
        .into_iter()
        .map(|c| (c.spec().socket_key(), c))
        .collect()
}

pub async fn create_connections_from_ips(
    ips: &[IpAddr],
    receiver_host: &str,
    receiver_port: u16,
) -> SmallVec<SrtlaConnection, 4> {
    create_connections_from_links(&specs_from_ips(ips), receiver_host, receiver_port).await
}

pub async fn create_connections_from_links(
    links: &[UplinkSpec],
    receiver_host: &str,
    receiver_port: u16,
) -> SmallVec<SrtlaConnection, 4> {
    let mut connections = SmallVec::new();
    for spec in links {
        match SrtlaConnection::connect(spec, receiver_host, receiver_port).await {
            Ok(conn) => {
                info!("added uplink {}", conn.label);
                connections.push(conn);
            }
            Err(e) => warn!(
                "failed to add uplink {} -> {}:{}: {}",
                spec.origin(),
                receiver_host,
                receiver_port,
                e
            ),
        }
    }
    connections
}
