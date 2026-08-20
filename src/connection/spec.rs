//! What an uplink *is*, separated from what its socket currently is.
//!
//! Before the bind-map, a link's identity and its socket were the same thing:
//! the source IP. That collapses on a real modem bond, where two HiLink dongles
//! both present `192.168.8.100` — one silently disappears — and where a replug
//! can hand the same modem a different address or a different interface.
//!
//! So the two are split:
//!
//! * [`UplinkSpec::link_id`] is the **identity**. It is writer-assigned, opaque,
//!   and stable across reloads, reconnects, and interface changes. Registration,
//!   stats, and telemetry state belong to it.
//! * [`SocketKey`] is the **current socket**, `(ip, iface)`. It is what dedup
//!   runs on, and it is allowed to change under a stable `link_id` — when it
//!   does, the socket is recreated rather than carried over.
//!
//! An unmapped link has no `link_id` and no `iface`, so its key degenerates to
//! the source IP and its behavior is the legacy behavior.

use std::net::IpAddr;

use crate::bind_map::{IfaceName, LinkId};

/// The identity of the socket an uplink currently owns.
///
/// Two rows sharing a key would name the same socket twice, so this is the
/// dedup key — never the link's identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketKey {
    pub ip: IpAddr,
    pub iface: Option<IfaceName>,
}

/// Everything needed to create one uplink socket, plus the identity that
/// outlives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UplinkSpec {
    pub ip: IpAddr,
    pub iface: Option<IfaceName>,
    pub link_id: Option<LinkId>,
}

impl UplinkSpec {
    /// A link with no bind-map row: legacy source-IP binding, no identity.
    #[must_use]
    pub fn unmapped(ip: IpAddr) -> Self {
        Self {
            ip,
            iface: None,
            link_id: None,
        }
    }

    #[must_use]
    pub fn socket_key(&self) -> SocketKey {
        SocketKey {
            ip: self.ip,
            iface: self.iface.clone(),
        }
    }

    /// How this uplink names its own egress in a failure line.
    ///
    /// Bare `ip` for an unmapped link, so the shipped
    /// `failed to add uplink <ip> -> <host>:<port>` line is unchanged.
    #[must_use]
    pub fn origin(&self) -> String {
        match self.iface.as_ref() {
            None => self.ip.to_string(),
            Some(iface) => format!("{} on {}", self.ip, iface.as_str()),
        }
    }

    /// The operator-facing label for this uplink.
    ///
    /// The unmapped form is byte-identical to the historical
    /// `"{host}:{port} via {ip}"` — it appears in shipped log lines that
    /// operators and tests both read.
    #[must_use]
    pub fn label(&self, host: &str, port: u16) -> String {
        match (self.iface.as_ref(), self.link_id.as_ref()) {
            (None, _) => format!("{}:{} via {}", host, port, self.ip),
            (Some(iface), None) => {
                format!("{}:{} via {} on {}", host, port, self.ip, iface.as_str())
            }
            (Some(iface), Some(link_id)) => format!(
                "{}:{} via {} on {} [{}]",
                host,
                port,
                self.ip,
                iface.as_str(),
                link_id.as_str()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    fn ip(last: u8) -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(192, 168, 8, last))
    }

    #[test]
    fn an_unmapped_links_label_is_the_historical_format() {
        // Given: a link with no bind-map row.
        let spec = UplinkSpec::unmapped(ip(100));

        // When: its label is derived.
        // Then: it is exactly the string shipped log lines already carry.
        assert_eq!(
            spec.label("rec.example.com", 5000),
            "rec.example.com:5000 via 192.168.8.100"
        );
    }

    #[test]
    fn twin_modems_on_one_ip_have_distinct_socket_keys() {
        // Given: two rows with the SAME source IP on different interfaces —
        // the HiLink twin case that legacy identity collapses.
        let a = UplinkSpec {
            ip: ip(100),
            iface: Some(IfaceName::parse("wwan0").unwrap()),
            link_id: Some(LinkId::parse("modem-a").unwrap()),
        };
        let b = UplinkSpec {
            ip: ip(100),
            iface: Some(IfaceName::parse("wwan1").unwrap()),
            link_id: Some(LinkId::parse("modem-b").unwrap()),
        };

        // When/Then: the dedup key separates them.
        assert_ne!(a.socket_key(), b.socket_key());
    }

    #[test]
    fn a_mapped_link_keeps_its_identity_when_its_socket_key_changes() {
        // Given: one link_id observed before and after an interface change.
        let link_id = LinkId::parse("modem-a").unwrap();
        let before = UplinkSpec {
            ip: ip(100),
            iface: Some(IfaceName::parse("wwan0").unwrap()),
            link_id: Some(link_id.clone()),
        };
        let after = UplinkSpec {
            ip: ip(101),
            iface: Some(IfaceName::parse("wwan3").unwrap()),
            link_id: Some(link_id),
        };

        // When/Then: the socket key moved but the identity did not.
        assert_ne!(before.socket_key(), after.socket_key());
        assert_eq!(before.link_id, after.link_id);
    }
}
