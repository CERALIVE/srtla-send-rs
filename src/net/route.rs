//! Read-only per-interface default-route observation.
//!
//! A device-bound socket does not fail when its interface loses its default
//! route. `SO_BINDTODEVICE` forces the packet out of that interface, and with no
//! matching route IPv4 falls back to treating the destination as **on-link**: it
//! ARPs for the receiver's public address, gets no answer, and drops the packet.
//! `sendto` still returns success, so the link looks healthy right up until the
//! bond notices it has stopped delivering. That silent blackhole is why the
//! route invariant is an explicit, separately-reported health signal instead of
//! something inferred from send success or from ACK liveness.
//!
//! This module only ever **reads** `/proc/net/route`. It installs no rule, no
//! route, and no policy-routing table — an observation that mutated the routing
//! table would be a different (and much more dangerous) feature.

use std::collections::HashSet;

/// Whether the interface backing a link still has a default route.
///
/// Deliberately distinct from ACK liveness (`SrtlaConnection::is_timed_out`):
/// a link can be ACK-live and route-blackholed, or route-healthy and dead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RouteHealth {
    /// Not observed: no interface is bound, or the host exposes no route table.
    #[default]
    Unknown,
    /// The interface has a default route; egress can leave the link.
    DefaultRoutePresent,
    /// The interface has no default route — traffic pinned to it will be
    /// silently blackholed rather than rejected.
    NoDefaultRoute,
}

impl RouteHealth {
    /// The operator-facing token for this state.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::DefaultRoutePresent => "default_route_present",
            Self::NoDefaultRoute => "no_default_route",
        }
    }
}

/// What an operator must be told when the route invariant moves.
///
/// The invariant is re-read every housekeeping tick but only *printed* in the
/// periodic status log, so without a transition report a blackhole that is
/// detected in one second stays unsaid for up to one status interval — long
/// enough for the operator to blame the bond instead of the route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteTransition {
    /// The interface just lost its default route.
    Blackholed,
    /// The interface just regained it.
    Restored,
    /// Nothing worth an unsolicited line: unchanged, or in or out of
    /// [`RouteHealth::Unknown`], which asserts nothing either way.
    Quiet,
}

/// Classify a change in the route invariant.
///
/// `Unknown` on either side stays [`RouteTransition::Quiet`]: an unreadable
/// route table is not evidence that a route appeared or vanished, and the first
/// observation of every link crosses out of `Unknown`.
#[must_use]
pub const fn classify_route_transition(before: RouteHealth, after: RouteHealth) -> RouteTransition {
    match (before, after) {
        (RouteHealth::DefaultRoutePresent, RouteHealth::NoDefaultRoute) => {
            RouteTransition::Blackholed
        }
        (RouteHealth::NoDefaultRoute, RouteHealth::DefaultRoutePresent) => {
            RouteTransition::Restored
        }
        _ => RouteTransition::Quiet,
    }
}

/// Path of the kernel's IPv4 route table.
const PROC_NET_ROUTE: &str = "/proc/net/route";

/// Interface names that currently hold an IPv4 default route.
///
/// The table is whitespace-separated with a header line; a default route is the
/// row whose destination *and* genmask are both all-zero. Columns after the mask
/// are ignored, so a kernel that adds fields does not break the parse.
#[must_use]
pub fn parse_default_route_ifaces(table: &str) -> HashSet<String> {
    table
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let iface = fields.next()?;
            let destination = fields.next()?;
            let mask = fields.nth(5)?; // gateway, flags, refcnt, use, metric, mask
            let is_default = destination.trim_start_matches('0').is_empty()
                && mask.trim_start_matches('0').is_empty();
            is_default.then(|| iface.to_string())
        })
        .collect()
}

/// Observe whether `iface` holds an IPv4 default route right now.
///
/// [`RouteHealth::Unknown`] when the route table cannot be read at all (a
/// non-Linux host, a container without `/proc`) — an unreadable table is not
/// evidence of a missing route.
#[must_use]
pub fn observe_default_route(iface: &str) -> RouteHealth {
    match std::fs::read_to_string(PROC_NET_ROUTE) {
        Ok(table) => {
            if parse_default_route_ifaces(&table).contains(iface) {
                RouteHealth::DefaultRoutePresent
            } else {
                RouteHealth::NoDefaultRoute
            }
        }
        Err(_) => RouteHealth::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real two-interface table: `eth0` carries the default route, `wwan0`
    /// only has its on-link subnet.
    const TABLE: &str = r"Iface	Destination	Gateway 	Flags	RefCnt	Use	Metric	Mask		MTU	Window	IRTT
eth0	00000000	0102A8C0	0003	0	0	100	00000000	0	0	0
eth0	0002A8C0	00000000	0001	0	0	100	00FFFFFF	0	0	0
wwan0	0008A8C0	00000000	0001	0	0	700	00FFFFFF	0	0	0
";

    #[test]
    fn an_interface_with_a_default_route_is_reported_as_healthy() {
        // Given: a route table in which eth0 holds the 0.0.0.0/0 entry.
        // When: the default-route set is parsed.
        let defaults = parse_default_route_ifaces(TABLE);

        // Then: only that interface is named.
        assert!(defaults.contains("eth0"));
        assert_eq!(defaults.len(), 1);
    }

    #[test]
    fn an_interface_with_only_an_on_link_subnet_has_no_default_route() {
        // Given: wwan0 appears in the table but only with a /24.
        // When/Then: it is absent from the default-route set — this is the
        // silent-blackhole case the check exists to name.
        assert!(!parse_default_route_ifaces(TABLE).contains("wwan0"));
    }

    #[test]
    fn a_header_only_table_names_no_interface() {
        // Given: a route table with no routes at all (header line only).
        let header = TABLE.lines().next().unwrap();

        // When/Then: nothing is claimed to have a default route.
        assert!(parse_default_route_ifaces(header).is_empty());
    }

    #[test]
    fn a_truncated_row_is_skipped_rather_than_misread() {
        // Given: a row that ends before the mask column.
        let table = format!(
            "{}\nwwan1\t00000000\t00000000\n",
            TABLE.lines().next().unwrap()
        );

        // When/Then: the short row cannot be proven to be a default route, so
        // it is not reported as one.
        assert!(parse_default_route_ifaces(&table).is_empty());
    }

    #[test]
    fn losing_and_regaining_a_default_route_are_both_announced() {
        // Given: a link whose route invariant moves in each direction.
        // When/Then: both crossings are worth an unsolicited line, because the
        // periodic status log is up to one interval away.
        assert_eq!(
            classify_route_transition(
                RouteHealth::DefaultRoutePresent,
                RouteHealth::NoDefaultRoute
            ),
            RouteTransition::Blackholed
        );
        assert_eq!(
            classify_route_transition(
                RouteHealth::NoDefaultRoute,
                RouteHealth::DefaultRoutePresent
            ),
            RouteTransition::Restored
        );
    }

    #[test]
    fn an_unchanged_invariant_is_never_announced() {
        // Given: the tick that re-reads the same answer it read a second ago.
        // When/Then: silence — otherwise every link would log once per second.
        for health in [
            RouteHealth::Unknown,
            RouteHealth::DefaultRoutePresent,
            RouteHealth::NoDefaultRoute,
        ] {
            assert_eq!(
                classify_route_transition(health, health),
                RouteTransition::Quiet
            );
        }
    }

    #[test]
    fn crossing_into_or_out_of_unknown_claims_nothing() {
        // Given: an unreadable route table on one side of the comparison — the
        // first observation of a link, or a table that stopped being readable.
        // When/Then: neither direction may be reported as a route change; an
        // absent observation is not evidence of an absent route.
        for known in [
            RouteHealth::DefaultRoutePresent,
            RouteHealth::NoDefaultRoute,
        ] {
            assert_eq!(
                classify_route_transition(RouteHealth::Unknown, known),
                RouteTransition::Quiet
            );
            assert_eq!(
                classify_route_transition(known, RouteHealth::Unknown),
                RouteTransition::Quiet
            );
        }
    }

    #[test]
    fn an_unreadable_route_table_reports_unknown_not_missing() {
        // Given: a host whose route table cannot be read (non-Linux, or no
        // /proc). Then: the observation must not claim the route is absent.
        if std::fs::read_to_string(PROC_NET_ROUTE).is_err() {
            assert_eq!(observe_default_route("eth0"), RouteHealth::Unknown);
        }
    }
}
