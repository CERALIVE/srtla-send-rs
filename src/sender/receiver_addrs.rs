//! Per-link receiver address failover for a receiver given as a hostname.
//!
//! # The problem
//!
//! Every uplink resolves the receiver hostname when it is first dialed and
//! would otherwise stay on that answer for the life of the process. A name
//! with two A records where one is stale (a moved server, a leftover record)
//! can put one uplink on the dead address for the whole session: it never
//! registers, carries nothing, and the bond silently runs on fewer links than
//! it has. Nobody notices until the links on the live address go down, and
//! then the outage is total although a working path is right there. The
//! whole-bond re-home in [`super::rehome`] does not help: it only fires once
//! *every* address the bond is on has left DNS, and a stale record that stays
//! published never leaves.
//!
//! # The policy
//!
//! The address book keeps the hostname's answers fresh (a lookup every
//! [`REFRESH_INTERVAL_MS`], sooner after the bond has tried every answer)
//! without ever blocking the housekeeping tick on DNS, and keeps them in a
//! stable, de-duplicated order so a resolver that reorders its answers cannot
//! make the bond skip one. Each time a failing link is about to rebuild its
//! socket (the existing timed-out reconnect path), it picks the address to
//! rebuild against, within its own address family:
//!
//! 1. If other uplinks are registered (REG3) on an address, that is where the
//!    bond lives. A link not already there moves to it straight away, whether
//!    or not DNS still lists it.
//! 2. Otherwise, if the bond was registered on an address within the last
//!    [`REGISTRATION_HOLD_MS`] (this link included), the link stays on or
//!    returns to it. A blip that takes every link down at once (a tunnel, a
//!    shared backhaul, a single-uplink bond) therefore retries the address
//!    that just worked instead of wandering off to another answer.
//! 3. Otherwise (the bond has no registration in this family, at startup with
//!    a stale first answer or after the hold), every link follows one shared
//!    trial cursor: the bond gives an address at least [`ADDRESS_TRIAL_MS`],
//!    then moves the cursor to the next answer at the next retry, wrapping
//!    around. Sharing the cursor keeps unregistered links on the same address
//!    as far as their retry times allow.
//!
//! A registered link is never touched, and nothing here sends anything: a moved
//! link goes through the same socket rebuild and REG1/REG2 resend it would have
//! made against its old address, and the move only counts once that rebuild
//! succeeded. A receiver given as an IP literal is left as before, and
//! `--no-rehome` turns all of this off.
//!
//! # Timing
//!
//! The hold counts from the last housekeeping tick on which a link was
//! registered, which is about the moment the link timeout declares it lost.
//! Reconnect attempts after that come at +0, 1, 2 and 3 s, then every 5 s, so
//! with a 10 s hold the first attempt on another address is the one at +13 s.
//! For a receiver that really moved while its old record still answers, that
//! is about 18 s after the last packet at the default 5 s link timeout, and
//! 28 s with `--conn-timeout-ms 15000`.
//!
//! # Limits
//!
//! A link dialed before its first retry still uses the first answer it
//! resolved, so links started while nothing is registered can sit on
//! different answers until they first time out. Links retry at different
//! times, so even after that they can briefly try different answers. Rule 1
//! is one-way: a link that joined the address the rest of the bond is
//! registered on stays there, even if only its previous address was reachable
//! over its own carrier.

use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;

use smallvec::SmallVec;
use srtla_core::connection::SrtlaConnection;
use tokio::sync::oneshot;
use tokio::time::Duration;
use tracing::{debug, info};

use super::housekeeping::GLOBAL_TIMEOUT_MS;
use super::rehome::{ReceiverResolver, ResolvedAddrs};
use super::uplink::ConnIoMap;
use crate::net::{resolve_remote_all, same_family};

/// The shortest time the bond gives an address to produce a registration
/// before the shared cursor moves to the next answer, when the bond has no
/// registration in that family. The move itself happens at the next reconnect
/// attempt after that. Long enough for a REG2/REG3 round trip over a slow
/// cellular link.
pub const ADDRESS_TRIAL_MS: u64 = 5_000;

/// Slack on the trial check. Reconnect attempts after the fast ones are also
/// 5 s apart and stamped with the time their housekeeping tick was serviced,
/// so a strict comparison would miss by a millisecond about half the time and
/// double the step to 10 s. The retry pacing in `srtla-core` uses the same
/// half-tick slack for the same reason.
const TRIAL_SLACK_MS: u64 = 500;

/// How long an address the bond was registered on keeps counting as proven
/// after the last link registered there was declared lost. The same 10 s as
/// the all-links-failed window (`GLOBAL_TIMEOUT_MS`), which is this sender's
/// existing line between "a blip" and "the bond is down", and as the stock
/// `srtla_rec` group timeout, after which the old address has no group left
/// to resume anyway. It covers every fast retry and the first slow one.
pub const REGISTRATION_HOLD_MS: u64 = GLOBAL_TIMEOUT_MS;

/// Routine re-resolution interval. A fixed interval rather than the record TTL:
/// `getaddrinfo` does not report one. Whether a lookup is answered from a
/// cache, and for how long, depends on the host's resolver.
pub const REFRESH_INTERVAL_MS: u64 = 30_000;

/// Floor between lookups asked for because the bond has tried every answer.
pub const URGENT_REFRESH_MIN_MS: u64 = 5_000;

/// Floor between info-level "resolves to" lines. A name that hands out
/// rotating subsets of a pool changes its answer set on most lookups.
const CHANGE_LOG_INTERVAL_MS: u64 = 600_000;

/// Ceiling on one background lookup.
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(3);

/// Where a failing link should rebuild its socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RemoteChoice {
    /// Keep the current address.
    Stay,
    /// Move to an address other uplinks are registered on now.
    Registered(SocketAddr),
    /// Return to an address the bond was registered on recently.
    Recent(SocketAddr),
    /// Follow the bond's shared trial cursor.
    Cursor(SocketAddr),
}

impl RemoteChoice {
    fn target(self) -> Option<SocketAddr> {
        match self {
            RemoteChoice::Stay => None,
            RemoteChoice::Registered(a) | RemoteChoice::Recent(a) | RemoteChoice::Cursor(a) => {
                Some(a)
            }
        }
    }
}

/// Pure address choice for one failing link, within its own family.
///
/// `registered` lists the addresses other uplinks are registered on now,
/// most-used first; `recent` the addresses the bond was registered on within
/// the hold, most recent first (this link included); `cursor` is the bond's
/// current trial address.
pub(super) fn choose_remote(
    current: SocketAddr,
    source: IpAddr,
    registered: &[SocketAddr],
    recent: &[SocketAddr],
    cursor: Option<SocketAddr>,
) -> RemoteChoice {
    let in_family = |list: &[SocketAddr]| -> SmallVec<SocketAddr, 4> {
        list.iter()
            .copied()
            .filter(|a| same_family(*a, source))
            .collect()
    };
    let registered = in_family(registered);
    if let Some(best) = registered.first() {
        return if registered.contains(&current) {
            RemoteChoice::Stay
        } else {
            RemoteChoice::Registered(*best)
        };
    }
    let recent = in_family(recent);
    if let Some(last) = recent.first() {
        return if recent.contains(&current) {
            RemoteChoice::Stay
        } else {
            RemoteChoice::Recent(*last)
        };
    }
    match cursor {
        Some(c) if c != current && same_family(c, source) => RemoteChoice::Cursor(c),
        _ => RemoteChoice::Stay,
    }
}

/// What happened to the shared trial cursor on this call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CursorStep {
    /// Still inside the current address's trial.
    Held,
    /// Moved to another answer (trial over, or the address left DNS).
    /// `wrapped` is set when the step went back to the start of the list,
    /// i.e. the bond has been through every answer.
    Advanced { wrapped: bool },
    /// Trial over but there is no other answer to try.
    Exhausted,
}

/// Advance the bond's trial cursor `(address, trial start)`. An unset cursor is
/// seeded from the asking link: its address, and the time it was pointed there
/// or last registered there. `candidates` is the book's answer list in the
/// family, de-duplicated and in stable order.
pub(super) fn advance_cursor(
    cursor: Option<(SocketAddr, u64)>,
    seed: (SocketAddr, u64),
    candidates: &[SocketAddr],
    now: u64,
) -> ((SocketAddr, u64), CursorStep) {
    let (addr, start) = cursor.unwrap_or(seed);
    if let Some(first) = candidates.first()
        && !candidates.contains(&addr)
    {
        return ((*first, now), CursorStep::Advanced { wrapped: false });
    }
    if now.saturating_sub(start) + TRIAL_SLACK_MS < ADDRESS_TRIAL_MS {
        return ((addr, start), CursorStep::Held);
    }
    match candidates.iter().position(|a| *a == addr) {
        Some(i) if candidates.len() > 1 => {
            let j = (i + 1) % candidates.len();
            (
                (candidates[j], now),
                CursorStep::Advanced { wrapped: j == 0 },
            )
        }
        _ => ((addr, start), CursorStep::Exhausted),
    }
}

/// Merge a fresh answer into the book's list: drop duplicates and addresses
/// that are gone, keep the survivors in the order they already had, and append
/// new ones in resolver order. A resolver that reorders an unchanged answer
/// therefore changes nothing, and the cursor's "next" stays well defined.
pub(super) fn merge_answers(current: &[SocketAddr], fresh: &[SocketAddr]) -> ResolvedAddrs {
    let mut merged: ResolvedAddrs = current
        .iter()
        .copied()
        .filter(|a| fresh.contains(a))
        .collect();
    merged.dedup();
    for a in fresh {
        if !merged.contains(a) {
            merged.push(*a);
        }
    }
    merged
}

/// Every uplink that is registered and not timed out, with its address.
pub(super) fn registered_remotes(
    connections: &[SrtlaConnection],
    conn_io: &ConnIoMap,
    now: u64,
) -> SmallVec<(u64, SocketAddr), 4> {
    connections
        .iter()
        .filter(|c| c.connected && !c.is_timed_out(now))
        .filter_map(|c| conn_io.get(&c.conn_id).map(|io| (c.conn_id, io.remote)))
        .collect()
}

/// Addresses that uplinks other than `conn_id` are registered on, most-used
/// first (ties keep connection order). A link's own current registration is
/// not counted here; the bond's recent registrations are (see
/// [`ReceiverAddresses::note_registered`]).
pub(super) fn proven_for(
    registered: &[(u64, SocketAddr)],
    conn_id: u64,
) -> SmallVec<SocketAddr, 4> {
    let mut counts: SmallVec<(SocketAddr, usize), 4> = SmallVec::new();
    for (_, remote) in registered.iter().filter(|(id, _)| *id != conn_id) {
        match counts.iter_mut().find(|(a, _)| a == remote) {
            Some((_, n)) => *n += 1,
            None => counts.push((*remote, 1)),
        }
    }
    // Stable sort: equal counts keep first-seen order.
    counts.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    counts.into_iter().map(|(a, _)| a).collect()
}

/// The production lookup for the address book: `getaddrinfo` under a timeout.
/// Failures are routine here (DNS is often the first thing to go when links
/// flap) and only logged at debug; the book keeps its last answer.
pub struct AddressBookDns;

impl ReceiverResolver for AddressBookDns {
    fn resolve(
        &self,
        host: String,
        port: u16,
    ) -> Pin<Box<dyn Future<Output = Option<ResolvedAddrs>> + Send>> {
        Box::pin(async move {
            match tokio::time::timeout(LOOKUP_TIMEOUT, resolve_remote_all(&host, port)).await {
                Ok(Ok(addrs)) if !addrs.is_empty() => Some(addrs),
                Ok(Ok(_)) => {
                    debug!("receiver address refresh: {host} returned no addresses");
                    None
                }
                Ok(Err(e)) => {
                    debug!("receiver address refresh: {host} failed ({e})");
                    None
                }
                Err(_) => {
                    debug!("receiver address refresh: {host} timed out after {LOOKUP_TIMEOUT:?}");
                    None
                }
            }
        })
    }
}

fn family_slot(source: IpAddr) -> usize {
    usize::from(!source.to_canonical().is_ipv4())
}

fn same_set(a: &[SocketAddr], b: &[SocketAddr]) -> bool {
    a.len() == b.len() && a.iter().all(|x| b.contains(x))
}

fn join(addrs: &[SocketAddr]) -> String {
    addrs
        .iter()
        .map(|a| a.to_string())
        .collect::<SmallVec<String, 4>>()
        .join(", ")
}

type Lookup = oneshot::Receiver<Option<ResolvedAddrs>>;

/// A move the address book planned for a failing link. It only takes effect
/// through [`ReceiverAddresses::commit_move`], once the socket toward `to` has
/// been built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PlannedMove {
    pub from: SocketAddr,
    pub choice: RemoteChoice,
}

impl PlannedMove {
    pub fn to(&self) -> SocketAddr {
        self.choice.target().unwrap_or(self.from)
    }
}

/// The receiver hostname's latest answers, refreshed off the housekeeping tick,
/// plus the bond's recent registrations and its shared trial cursors.
pub struct ReceiverAddresses {
    enabled: bool,
    resolver: Arc<dyn ReceiverResolver>,
    /// De-duplicated, in stable order (see [`merge_answers`]).
    answers: ResolvedAddrs,
    last_lookup_ms: Option<u64>,
    last_change_log_ms: Option<u64>,
    refresh_wanted: bool,
    pending: Option<Lookup>,
    /// Addresses the bond was registered on, with the last tick it was.
    recent: SmallVec<(SocketAddr, u64), 4>,
    /// Shared trial cursor per family (`[IPv4, IPv6]`): address and trial start.
    cursors: [Option<(SocketAddr, u64)>; 2],
    moves: u64,
}

impl ReceiverAddresses {
    pub fn new(enabled: bool, resolver: Arc<dyn ReceiverResolver>) -> Self {
        Self {
            enabled,
            resolver,
            answers: SmallVec::new(),
            last_lookup_ms: None,
            last_change_log_ms: None,
            refresh_wanted: false,
            pending: None,
            recent: SmallVec::new(),
            cursors: [None, None],
            moves: 0,
        }
    }

    /// Whether failover applies to `host` at all: never for an IP literal, and
    /// never under `--no-rehome`.
    pub fn applies_to(&self, host: &str) -> bool {
        self.enabled && host.parse::<IpAddr>().is_err()
    }

    /// The resolver behind the book, for other detect-only lookups of the same
    /// name.
    pub(super) fn resolver(&self) -> Arc<dyn ReceiverResolver> {
        self.resolver.clone()
    }

    /// Install a fresh answer, logging a changed set (rate-limited).
    fn install(&mut self, host: &str, fresh: &[SocketAddr], now: u64) {
        let merged = merge_answers(&self.answers, fresh);
        if !same_set(&merged, &self.answers) {
            let due = self
                .last_change_log_ms
                .is_none_or(|last| now.saturating_sub(last) >= CHANGE_LOG_INTERVAL_MS);
            if due {
                self.last_change_log_ms = Some(now);
                info!("receiver {host} resolves to {}", join(&merged));
            } else {
                debug!("receiver {host} resolves to {}", join(&merged));
            }
        }
        self.answers = merged;
    }

    /// Before the book's own first successful lookup, adopt the answer the
    /// uplinks were dialed from, so its alternatives survive a DNS outage.
    pub(super) fn seed(&mut self, dialed: &[SocketAddr]) {
        if self.answers.is_empty() {
            self.answers = merge_answers(&[], dialed);
        }
    }

    /// Once per housekeeping tick: collect a finished lookup and start the next
    /// one when it is due. Never waits on DNS.
    pub fn poll(&mut self, host: &str, port: u16, now: u64) {
        if !self.applies_to(host) {
            return;
        }
        if let Some(rx) = self.pending.as_mut() {
            match rx.try_recv() {
                Ok(answer) => {
                    self.pending = None;
                    if let Some(fresh) = answer {
                        self.install(host, &fresh, now);
                    }
                }
                Err(oneshot::error::TryRecvError::Empty) => return,
                Err(oneshot::error::TryRecvError::Closed) => self.pending = None,
            }
        }
        let due = match self.last_lookup_ms {
            None => true,
            Some(last) => {
                let age = now.saturating_sub(last);
                age >= REFRESH_INTERVAL_MS || (self.refresh_wanted && age >= URGENT_REFRESH_MIN_MS)
            }
        };
        if !due {
            return;
        }
        self.last_lookup_ms = Some(now);
        self.refresh_wanted = false;
        let (tx, rx) = oneshot::channel();
        let lookup: Pin<Box<dyn Future<Output = Option<ResolvedAddrs>> + Send>> =
            self.resolver.resolve(host.to_string(), port);
        tokio::spawn(async move {
            let _ = tx.send(lookup.await);
        });
        self.pending = Some(rx);
    }

    /// The latest answer (empty until the first lookup lands or a seed).
    pub fn answers(&self) -> &[SocketAddr] {
        &self.answers
    }

    /// Once per housekeeping tick, before any link is handled: record where the
    /// bond is registered. A family with a registration drops its trial cursor;
    /// registrations older than the hold are forgotten.
    pub(super) fn note_registered(&mut self, registered: &[(u64, SocketAddr)], now: u64) {
        for (_, addr) in registered {
            match self.recent.iter_mut().find(|(a, _)| a == addr) {
                Some((_, at)) => *at = now,
                None => self.recent.push((*addr, now)),
            }
            self.cursors[family_slot(addr.ip())] = None;
        }
        self.recent
            .retain(|(_, at)| now.saturating_sub(*at) < REGISTRATION_HOLD_MS);
    }

    /// The whole bond was just re-homed to an answer from `fresh` and
    /// registers from scratch: adopt that answer, forget the old address's
    /// registrations and trials, and discard any lookup still in flight, which
    /// was started before the move and must not undo it.
    pub(super) fn adopt_rehome(&mut self, fresh: &[SocketAddr], now: u64) {
        self.answers = merge_answers(&[], fresh);
        self.pending = None;
        self.last_lookup_ms = Some(now);
        self.refresh_wanted = false;
        self.recent.clear();
        self.cursors = [None, None];
    }

    /// Plan the address a failing link rebuilds against. `since_ms` is when
    /// the link was pointed at `current` or last registered there;
    /// `registered` is [`proven_for`] this link. `None` means stay. Nothing is
    /// counted or logged until [`Self::commit_move`].
    pub(super) fn choose_for(
        &mut self,
        current: SocketAddr,
        since_ms: u64,
        source: IpAddr,
        registered: &[SocketAddr],
        now: u64,
    ) -> Option<PlannedMove> {
        let mut recent: SmallVec<(SocketAddr, u64), 4> = self
            .recent
            .iter()
            .copied()
            .filter(|(_, at)| now.saturating_sub(*at) < REGISTRATION_HOLD_MS)
            .collect();
        recent.sort_by_key(|(_, at)| std::cmp::Reverse(*at));
        let recent: SmallVec<SocketAddr, 4> = recent.into_iter().map(|(a, _)| a).collect();

        let has_proof = registered
            .iter()
            .chain(recent.iter())
            .any(|a| same_family(*a, source));
        let cursor = if has_proof {
            None
        } else {
            let slot = family_slot(source);
            let candidates: SmallVec<SocketAddr, 4> = self
                .answers
                .iter()
                .copied()
                .filter(|a| same_family(*a, source))
                .collect();
            let before = self.cursors[slot];
            let (cursor, step) = advance_cursor(before, (current, since_ms), &candidates, now);
            self.cursors[slot] = Some(cursor);
            match step {
                CursorStep::Held => {}
                CursorStep::Advanced { wrapped } => {
                    // Every answer has had its trial: ask DNS again early.
                    if wrapped {
                        self.refresh_wanted = true;
                    }
                    let (old, start) = before.unwrap_or((current, since_ms));
                    info!(
                        "no registration from receiver address {old} in {}ms; the bond tries {} \
                         next",
                        now.saturating_sub(start),
                        cursor.0
                    );
                }
                CursorStep::Exhausted => self.refresh_wanted = true,
            }
            Some(cursor.0)
        };

        let choice = choose_remote(current, source, registered, &recent, cursor);
        choice.target().map(|_| PlannedMove {
            from: current,
            choice,
        })
    }

    /// Record a planned move whose socket rebuild succeeded.
    pub(super) fn commit_move(&mut self, label: &str, planned: PlannedMove) {
        self.moves = self.moves.saturating_add(1);
        let (from, to) = (planned.from, planned.to());
        match planned.choice {
            RemoteChoice::Registered(_) => info!(
                "{label}: not registered on receiver address {from}; moved to {to}, where other \
                 uplinks are registered"
            ),
            RemoteChoice::Recent(_) => info!(
                "{label}: moved from receiver address {from} back to {to}, where the bond was \
                 registered until recently"
            ),
            RemoteChoice::Cursor(_) => {
                info!("{label}: moved from receiver address {from} to {to}")
            }
            RemoteChoice::Stay => {}
        }
    }

    /// Links moved to another address so far. Log-only, like the re-home count.
    #[cfg(any(test, feature = "test-internals"))]
    pub fn move_count(&self) -> u64 {
        self.moves
    }

    #[cfg(any(test, feature = "test-internals"))]
    pub fn lookup_pending(&self) -> bool {
        self.pending.is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;
    use crate::sender::rehome::StubResolver;

    fn a(last: u8) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, last)), 5000)
    }
    fn v6(last: u16) -> SocketAddr {
        SocketAddr::new(
            IpAddr::V6(std::net::Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, last)),
            5000,
        )
    }
    const SRC4: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    fn src6() -> IpAddr {
        "2001:db8:1::2".parse().unwrap()
    }

    const T: u64 = 100_000;

    #[test]
    fn a_link_where_others_are_registered_stays() {
        assert_eq!(
            choose_remote(a(1), SRC4, &[a(1)], &[], Some(a(2))),
            RemoteChoice::Stay
        );
    }

    #[test]
    fn a_link_moves_to_where_others_are_registered_at_once() {
        assert_eq!(
            choose_remote(a(2), SRC4, &[a(1)], &[a(2)], None),
            RemoteChoice::Registered(a(1)),
            "a live registration beats a recent one"
        );
        assert_eq!(
            choose_remote(a(3), SRC4, &[a(1), a(2)], &[], None),
            RemoteChoice::Registered(a(1)),
            "most-used first"
        );
    }

    #[test]
    fn with_nobody_registered_the_recent_registration_holds_the_link() {
        // The whole-bond blip: nobody is registered right now, but the bond
        // was on a(1). Stay there rather than following any cursor.
        assert_eq!(
            choose_remote(a(1), SRC4, &[], &[a(1)], Some(a(2))),
            RemoteChoice::Stay
        );
        assert_eq!(
            choose_remote(a(2), SRC4, &[], &[a(1)], None),
            RemoteChoice::Recent(a(1))
        );
    }

    #[test]
    fn without_any_proof_links_follow_the_cursor() {
        assert_eq!(
            choose_remote(a(1), SRC4, &[], &[], Some(a(2))),
            RemoteChoice::Cursor(a(2))
        );
        assert_eq!(
            choose_remote(a(2), SRC4, &[], &[], Some(a(2))),
            RemoteChoice::Stay
        );
        assert_eq!(
            choose_remote(a(1), SRC4, &[], &[], None),
            RemoteChoice::Stay
        );
    }

    #[test]
    fn proof_in_the_other_family_does_not_count() {
        assert_eq!(
            choose_remote(v6(1), src6(), &[a(1)], &[a(1)], Some(v6(2))),
            RemoteChoice::Cursor(v6(2))
        );
        assert_eq!(
            choose_remote(a(2), SRC4, &[v6(1)], &[v6(1)], None),
            RemoteChoice::Stay
        );
    }

    #[test]
    fn the_cursor_seeds_from_the_link_and_gives_each_address_its_trial() {
        let answers = [a(1), a(2), a(3)];
        // Seeded from the asking link; its trial is still running.
        let since = T - ADDRESS_TRIAL_MS + TRIAL_SLACK_MS + 1;
        assert_eq!(
            advance_cursor(None, (a(1), since), &answers, T),
            ((a(1), since), CursorStep::Held)
        );
        // Trial over: next answer, new trial.
        assert_eq!(
            advance_cursor(None, (a(1), since - 1), &answers, T),
            ((a(2), T), CursorStep::Advanced { wrapped: false })
        );
        // An existing cursor wins over the seed: one trial for the bond.
        assert_eq!(
            advance_cursor(Some((a(2), T)), (a(1), 0), &answers, T + 1_000),
            ((a(2), T), CursorStep::Held)
        );
        // Wrap-around marks a full pass.
        assert_eq!(
            advance_cursor(Some((a(3), 0)), (a(3), 0), &answers, T),
            ((a(1), T), CursorStep::Advanced { wrapped: true })
        );
        // An address that left DNS is replaced by the first answer at once.
        assert_eq!(
            advance_cursor(Some((a(9), T)), (a(9), T), &answers, T),
            ((a(1), T), CursorStep::Advanced { wrapped: false })
        );
        // Nothing else to try.
        assert_eq!(
            advance_cursor(Some((a(1), 0)), (a(1), 0), &[a(1)], T),
            ((a(1), 0), CursorStep::Exhausted)
        );
        assert_eq!(
            advance_cursor(None, (a(1), 0), &[], T),
            ((a(1), 0), CursorStep::Exhausted)
        );
    }

    /// A retry stamped a millisecond early must still end a 5 s trial; the
    /// retries themselves are 5 s apart.
    #[test]
    fn the_trial_has_the_same_slack_as_the_retry_pacing() {
        let answers = [a(1), a(2)];
        let start = T;
        assert_eq!(
            advance_cursor(Some((a(1), start)), (a(1), start), &answers, start + 4_999).1,
            CursorStep::Advanced { wrapped: false }
        );
        assert_eq!(
            advance_cursor(
                Some((a(1), start)),
                (a(1), start),
                &answers,
                start + ADDRESS_TRIAL_MS - TRIAL_SLACK_MS - 1
            )
            .1,
            CursorStep::Held
        );
    }

    #[test]
    fn answers_merge_into_a_stable_deduplicated_order() {
        assert_eq!(
            merge_answers(&[], &[a(1), a(1), a(3)]).as_slice(),
            &[a(1), a(3)]
        );
        // A reordered answer changes nothing.
        assert_eq!(
            merge_answers(&[a(1), a(2), a(3)], &[a(2), a(1), a(3)]).as_slice(),
            &[a(1), a(2), a(3)]
        );
        // Removals drop out, additions append in resolver order.
        assert_eq!(
            merge_answers(&[a(1), a(2), a(3)], &[a(5), a(3), a(4), a(1)]).as_slice(),
            &[a(1), a(3), a(5), a(4)]
        );
    }

    fn book_with(answers: &[SocketAddr]) -> ReceiverAddresses {
        let mut book = ReceiverAddresses::new(true, StubResolver::new(vec![]));
        book.answers = merge_answers(&[], answers);
        book
    }

    /// Plan and commit, as housekeeping does when the rebuild succeeds.
    fn step(
        book: &mut ReceiverAddresses,
        current: SocketAddr,
        since: u64,
        source: IpAddr,
        now: u64,
    ) -> Option<SocketAddr> {
        let planned = book.choose_for(current, since, source, &[], now)?;
        book.commit_move("l", planned);
        Some(planned.to())
    }

    /// Three answers, only the last one live, and a resolver
    /// that alternates between two orders. Every answer must get its turn.
    #[test]
    fn a_reordering_resolver_cannot_make_the_bond_skip_an_answer() {
        let mut book = book_with(&[a(1), a(2), a(3)]);
        let orders = [[a(1), a(2), a(3)], [a(2), a(1), a(3)]];
        let mut current = a(1);
        let mut visited = vec![current];
        let mut now = T;
        for n in 0..6 {
            book.install("rec.example.com", &orders[n % 2], now);
            if let Some(next) = step(&mut book, current, 0, SRC4, now) {
                current = next;
                visited.push(next);
            }
            now += ADDRESS_TRIAL_MS;
        }
        assert!(
            visited[..4].contains(&a(3)),
            "the live third answer is reached within one pass: {visited:?}"
        );
    }

    #[test]
    fn a_duplicated_answer_does_not_trap_the_cursor() {
        let mut book = ReceiverAddresses::new(true, StubResolver::new(vec![]));
        book.install("rec.example.com", &[a(1), a(1), a(3)], T);
        assert_eq!(book.answers(), &[a(1), a(3)]);
        assert_eq!(step(&mut book, a(1), 0, SRC4, T), Some(a(3)));
    }

    /// A bond registered on a(1) for a long time loses every link at once. It
    /// keeps retrying a(1) through the hold, not jump to the stale answer.
    #[test]
    fn a_whole_bond_blip_keeps_retrying_the_address_that_worked() {
        let mut book = book_with(&[a(1), a(2)]);
        book.note_registered(&[(1, a(1)), (2, a(1))], T);
        // Every link has timed out: nobody is registered any more. These are
        // the reconnect attempts of a link that lost registration at T.
        for dt in [1_000, 2_000, 3_000, 4_000, 9_000] {
            book.note_registered(&[], T + dt);
            assert_eq!(
                step(&mut book, a(1), T, SRC4, T + dt),
                None,
                "{dt}ms into the blip the link stays on the address that worked"
            );
        }
        assert_eq!(book.move_count(), 0);
        // A link that had wandered off comes back.
        assert_eq!(step(&mut book, a(2), T, SRC4, T + 2_000), Some(a(1)));
    }

    #[test]
    fn the_hold_lasts_the_all_failed_window_and_then_the_cursor_takes_over() {
        assert_eq!(REGISTRATION_HOLD_MS, GLOBAL_TIMEOUT_MS);
        let mut book = book_with(&[a(1), a(2)]);
        book.note_registered(&[(1, a(1))], T);
        let inside = T + REGISTRATION_HOLD_MS - 1;
        book.note_registered(&[], inside);
        assert_eq!(step(&mut book, a(1), T, SRC4, inside), None);
        let after = T + REGISTRATION_HOLD_MS;
        book.note_registered(&[], after);
        // Trial counted from the last registration, which is long over.
        assert_eq!(step(&mut book, a(1), T, SRC4, after), Some(a(2)));
    }

    #[test]
    fn unregistered_links_share_one_trial() {
        // Links dialed at different times still try addresses together.
        let mut book = book_with(&[a(1), a(2), a(3)]);
        // Link 1 (dialed long ago) seeds the cursor and moves it to a(2).
        assert_eq!(step(&mut book, a(1), 0, SRC4, T), Some(a(2)));
        // Link 2, freshly dialed on a(3), follows the bond rather than
        // running its own trial on a(3).
        assert_eq!(step(&mut book, a(3), T - 100, SRC4, T + 500), Some(a(2)));
        // Link 3, already dialed on a(2), stays.
        assert_eq!(step(&mut book, a(2), 0, SRC4, T + 600), None);
        // One trial later the whole bond moves on together.
        let t2 = T + ADDRESS_TRIAL_MS;
        assert_eq!(step(&mut book, a(2), 0, SRC4, t2), Some(a(3)));
        assert_eq!(step(&mut book, a(2), T, SRC4, t2 + 1), Some(a(3)));
    }

    #[test]
    fn a_planned_move_counts_only_when_committed() {
        let mut book = book_with(&[a(1), a(2)]);
        let planned = book.choose_for(a(1), 0, SRC4, &[], T).unwrap();
        assert_eq!(planned.to(), a(2));
        assert_eq!(book.move_count(), 0, "planning alone is not a move");
        book.commit_move("l", planned);
        assert_eq!(book.move_count(), 1);
    }

    #[test]
    fn a_registration_clears_the_cursor() {
        let mut book = book_with(&[a(1), a(2)]);
        assert_eq!(step(&mut book, a(1), 0, SRC4, T), Some(a(2)));
        book.note_registered(&[(1, a(2))], T + 1_000);
        assert_eq!(book.cursors, [None, None]);
    }

    /// Re-home moved the bond to b; the book must not send it back to the
    /// answer it cached before, nor let an older lookup overwrite the move.
    #[tokio::test]
    async fn a_rehome_is_not_undone_by_the_older_answer() {
        let old = a(1);
        let new = a(9);
        let resolver = StubResolver::new(vec![Some(vec![old])]);
        let mut book = ReceiverAddresses::new(true, resolver.clone());
        book.answers = merge_answers(&[], &[old]);
        book.note_registered(&[(1, old)], T - 60_000);
        // A routine lookup is in flight when the re-home lands.
        book.poll("rec.example.com", 5000, T);
        assert!(book.lookup_pending());
        book.adopt_rehome(&[new], T);
        assert!(!book.lookup_pending(), "the pre-move lookup is discarded");
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }
        book.poll("rec.example.com", 5000, T + 1_000);
        assert_eq!(book.answers(), &[new]);
        assert_eq!(
            resolver.calls(),
            1,
            "the refresh clock restarted at the move"
        );
        // B's first handshake needs a retry: it stays on B.
        assert_eq!(step(&mut book, new, T, SRC4, T + 1_000), None);
    }

    #[test]
    fn the_dial_answer_seeds_an_empty_book_only() {
        let mut book = ReceiverAddresses::new(true, StubResolver::new(vec![]));
        book.seed(&[a(1), a(1), a(2)]);
        assert_eq!(book.answers(), &[a(1), a(2)]);
        book.seed(&[a(7)]);
        assert_eq!(
            book.answers(),
            &[a(1), a(2)],
            "a populated book keeps its answer"
        );
    }

    #[test]
    fn families_keep_separate_cursors() {
        let mut book = book_with(&[v6(1), a(1), v6(2), a(2)]);
        assert_eq!(step(&mut book, a(1), 0, SRC4, T), Some(a(2)));
        assert_eq!(step(&mut book, v6(2), 0, src6(), T), Some(v6(1)));
        let mapped: IpAddr = "::ffff:10.0.0.2".parse().unwrap();
        assert_eq!(step(&mut book, a(1), 0, mapped, T), Some(a(2)));
    }

    #[test]
    fn answer_sets_compare_without_order() {
        assert!(same_set(&[a(1), a(2)], &[a(2), a(1)]));
        assert!(!same_set(&[a(1), a(2)], &[a(1)]));
    }

    #[tokio::test]
    async fn ip_literals_and_the_opt_out_never_resolve() {
        let resolver = StubResolver::new(vec![Some(vec![a(1)])]);
        let mut book = ReceiverAddresses::new(true, resolver.clone());
        book.poll("203.0.113.1", 5000, T);
        book.poll("2001:db8::1", 5000, T);
        assert_eq!(resolver.calls(), 0);
        assert!(!book.applies_to("203.0.113.1"));

        let mut off = ReceiverAddresses::new(false, resolver.clone());
        off.poll("rec.example.com", 5000, T);
        assert_eq!(resolver.calls(), 0);
        assert!(!off.applies_to("rec.example.com"));
    }

    async fn settle(book: &mut ReceiverAddresses, now: u64) {
        for _ in 0..100 {
            if !book.lookup_pending() {
                return;
            }
            tokio::task::yield_now().await;
            book.poll("rec.example.com", 5000, now);
        }
        panic!("lookup never finished");
    }

    #[tokio::test]
    async fn lookups_are_paced_and_a_failed_lookup_keeps_the_last_answer() {
        let resolver = StubResolver::new(vec![Some(vec![a(1), a(2)]), None, Some(vec![a(3)])]);
        let mut book = ReceiverAddresses::new(true, resolver.clone());

        book.poll("rec.example.com", 5000, T);
        settle(&mut book, T).await;
        assert_eq!(book.answers(), &[a(1), a(2)]);
        assert_eq!(resolver.calls(), 1);

        book.poll("rec.example.com", 5000, T + REFRESH_INTERVAL_MS - 1);
        assert_eq!(resolver.calls(), 1, "no lookup before the refresh interval");

        let t1 = T + REFRESH_INTERVAL_MS;
        book.poll("rec.example.com", 5000, t1);
        settle(&mut book, t1).await;
        assert_eq!(resolver.calls(), 2);
        assert_eq!(
            book.answers(),
            &[a(1), a(2)],
            "a failed lookup tells us nothing and keeps the last answer"
        );

        // The bond running out of addresses asks for an early refresh, which
        // is still floored.
        book.answers = SmallVec::from_slice_copy(&[a(2)]);
        let _ = book.choose_for(a(2), 0, SRC4, &[], t1);
        book.poll("rec.example.com", 5000, t1 + URGENT_REFRESH_MIN_MS - 1);
        assert_eq!(resolver.calls(), 2);
        book.poll("rec.example.com", 5000, t1 + URGENT_REFRESH_MIN_MS);
        settle(&mut book, t1 + URGENT_REFRESH_MIN_MS).await;
        assert_eq!(resolver.calls(), 3);
        assert_eq!(book.answers(), &[a(3)]);
    }

    #[tokio::test]
    async fn proof_ranks_by_registered_links_and_never_counts_the_link_itself() {
        let mut connections = srtla_core::test_helpers::create_test_connections(4).await;
        let mut conn_io = crate::test_helpers::create_test_conn_io_map(&connections);
        let now = srtla_core::utils::now_ms();
        let remotes = [a(2), a(1), a(1), a(3)];
        for (conn, remote) in connections.iter_mut().zip(remotes) {
            conn.connected = true;
            conn.last_received = Some(now);
            conn_io.get_mut(&conn.conn_id).unwrap().remote = remote;
        }
        // The fourth link is registered but has gone silent: no proof.
        connections[3].last_received = Some(now - 60_000);
        let registered = registered_remotes(&connections, &conn_io, now);
        assert_eq!(registered.len(), 3);

        let outsider = 0;
        assert_eq!(proven_for(&registered, outsider).as_slice(), &[a(1), a(2)]);
        // Link 0 is the only one on a(2); its own registration is not proof.
        assert_eq!(
            proven_for(&registered, connections[0].conn_id).as_slice(),
            &[a(1)]
        );

        connections[1].connected = false;
        connections[2].connected = false;
        let registered = registered_remotes(&connections, &conn_io, now);
        assert_eq!(proven_for(&registered, outsider).as_slice(), &[a(2)]);
    }
}
