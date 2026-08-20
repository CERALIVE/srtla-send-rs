//! Duplicate-IP twin-modem topology: two interfaces, one address, two carriers.
//!
//! Two identical HiLink dongles both hand the host `192.168.8.100`, and each is
//! its own NAT router, so the receiver sees two *distinct* public sources. A
//! plain veth pair cannot model that. With one address on two sender-side
//! interfaces the receiver would answer both uplinks at the same address, the
//! kernel would route every reply down whichever route won, and the second
//! device-bound socket — which only accepts traffic arriving on *its* interface
//! — would never hear a REG2 and never register. The bond would look like a
//! sender bug when it is really a topology artifact.
//!
//! So each twin gets its own **carrier namespace** that masquerades onto its own
//! transit subnet, exactly as a HiLink does:
//!
//! ```text
//!               10.30.9.1/24              10.31.N.1/24
//!   sender_ns ───── ts{N} ═══ tg{N} ─ gateway_ns{N} ─ tw{N} ═══ tr{N} ─ receiver_ns
//!   (both ts0 and ts1 carry the SAME address)      MASQUERADE      10.99.0.1 on lo
//! ```
//!
//! Two settings are load-bearing and were both found the hard way:
//!
//! * **Per-device `rp_filter` must be cleared, not just `conf.all`.** The
//!   effective value is `max(all, dev)`, and a device created before `conf.default`
//!   is lowered keeps `1`. Strict reverse-path then drops every reply arriving on
//!   the second twin, because the best route back to the gateway is the *first*
//!   twin — and the ARP request for the shared address is dropped with it, so the
//!   carrier can never even resolve the host.
//! * **The receiver answers from `10.99.0.1`**, pinned with a `src` hint on both
//!   return routes, so replies come from the address the sender dialed rather
//!   than from the transit interface.

use anyhow::{Context, Result};

use crate::test_util::unique_ns_name;
use crate::topology::Namespace;

/// The address BOTH twins carry. This duplication is the whole point.
pub const TWIN_IP: &str = "10.30.9.1";
/// Every carrier's LAN-side address. Identical across carriers because each one
/// lives in its own namespace — just like two HiLinks both being `192.168.8.1`.
const GATEWAY_LAN_IP: &str = "10.30.9.2";
/// The receiver address the sender dials, held on `lo` so it is reachable
/// through either carrier.
pub const RECEIVER_IP: &str = "10.99.0.1";

/// The four interface names one twin link owns, one per namespace boundary.
struct LinkNames {
    sender: String,
    gateway_lan: String,
    gateway_wan: String,
    receiver: String,
}

/// A duplicate-IP twin bond with one NAT carrier per link.
pub struct TwinTopology {
    pub sender_ns: Namespace,
    pub receiver_ns: Namespace,
    pub gateways: Vec<Namespace>,
    names: Vec<LinkNames>,
}

impl TwinTopology {
    /// Build `links` twins, every one bound to [`TWIN_IP`].
    pub fn new(test_name: &str, links: usize) -> Result<Self> {
        assert!(links > 0, "need at least one twin");
        let sender_ns = Namespace::new(&unique_ns_name(&format!("{test_name}_s")))?;
        let receiver_ns = Namespace::new(&unique_ns_name(&format!("{test_name}_r")))?;
        let gateways = (0..links)
            .map(|i| Namespace::new(&unique_ns_name(&format!("{test_name}_g{i}"))))
            .collect::<Result<Vec<_>>>()?;

        let names = (0..links)
            .map(|i| LinkNames {
                sender: unique_ns_name(&format!("ts{i}")),
                gateway_lan: unique_ns_name(&format!("tg{i}")),
                gateway_wan: unique_ns_name(&format!("tw{i}")),
                receiver: unique_ns_name(&format!("tr{i}")),
            })
            .collect();

        let topo = Self {
            sender_ns,
            receiver_ns,
            gateways,
            names,
        };
        topo.relax_reverse_path()?;
        topo.receiver_ns
            .exec_checked(
                "ip",
                &["addr", "add", &format!("{RECEIVER_IP}/32"), "dev", "lo"],
            )
            .context("give the receiver its dialed address")?;
        for idx in 0..links {
            topo.wire_transit(idx)?;
            topo.wire_access(idx)?;
        }
        Ok(topo)
    }

    /// The sender-side interface name of twin `idx`.
    #[must_use]
    pub fn sender_iface(&self, idx: usize) -> &str {
        &self.names[idx].sender
    }

    #[must_use]
    pub fn link_count(&self) -> usize {
        self.names.len()
    }

    /// Bytes transmitted on twin `idx`'s sender-side interface.
    ///
    /// Read straight from the netdev, so a *recreated* interface starts from
    /// zero — which is what makes "it recovered on the new ifindex" observable
    /// rather than inferred.
    pub fn tx_bytes(&self, idx: usize) -> Result<u64> {
        let path = format!(
            "/sys/class/net/{}/statistics/tx_bytes",
            self.sender_iface(idx)
        );
        let out = self.sender_ns.exec_checked("cat", &[&path])?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().parse()?)
    }

    /// The kernel ifindex of twin `idx`'s interface, or `None` once it is gone.
    pub fn ifindex(&self, idx: usize) -> Option<u32> {
        let path = format!("/sys/class/net/{}/ifindex", self.sender_iface(idx));
        let out = self.sender_ns.exec("cat", &[&path]).ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().parse().ok())
            .flatten()
    }

    /// Unplug twin `idx` — destroys the veth pair, so the interface the socket
    /// is bound to simply stops existing.
    pub fn unplug(&self, idx: usize) -> Result<()> {
        self.sender_ns
            .exec_checked("ip", &["link", "del", self.sender_iface(idx)])
            .with_context(|| format!("unplug twin {idx}"))?;
        Ok(())
    }

    /// Replug twin `idx` under the SAME interface name and a NEW ifindex.
    pub fn replug(&self, idx: usize) -> Result<()> {
        self.wire_access(idx)
    }

    /// Delete twin `idx`'s default route, leaving the interface up.
    ///
    /// This is the silent blackhole: `SO_BINDTODEVICE` still forces packets out
    /// of the interface, IPv4 treats the receiver as on-link, ARPs for it, and
    /// drops every datagram while `sendto` keeps reporting success.
    pub fn blackhole(&self, idx: usize) -> Result<()> {
        self.sender_ns
            .exec_checked("ip", &self.default_route_args("del", idx))
            .with_context(|| format!("blackhole twin {idx}"))?;
        Ok(())
    }

    /// Restore twin `idx`'s default route.
    pub fn restore_route(&self, idx: usize) -> Result<()> {
        self.sender_ns
            .exec_checked("ip", &self.default_route_args("add", idx))
            .with_context(|| format!("restore the route of twin {idx}"))?;
        Ok(())
    }

    /// Cap twin `idx` so neither link can absorb the whole stream on its own.
    ///
    /// Without a cap the scheduler's hysteresis parks every packet on the first
    /// link — correct behavior on an uncongested pair, but it proves nothing
    /// about bonding.
    pub fn shape(&self, idx: usize, rate_kbit: u64, delay_ms: u32) -> Result<()> {
        let iface = self.sender_iface(idx);
        let _ = self
            .sender_ns
            .exec("tc", &["qdisc", "del", "dev", iface, "root"]);
        let rate = format!("{rate_kbit}kbit");
        let burst = (rate_kbit * 1000 / 8 / 10).max(15_400).to_string();
        self.sender_ns.exec_checked(
            "tc",
            &[
                "qdisc", "add", "dev", iface, "root", "handle", "1:", "tbf", "rate", &rate,
                "burst", &burst, "latency", "1s",
            ],
        )?;
        self.sender_ns.exec_checked(
            "tc",
            &[
                "qdisc",
                "add",
                "dev",
                iface,
                "parent",
                "1:1",
                "handle",
                "10:",
                "netem",
                "delay",
                &format!("{delay_ms}ms"),
            ],
        )?;
        Ok(())
    }

    fn default_route_args<'a>(&'a self, verb: &'a str, idx: usize) -> Vec<&'a str> {
        let mut args = vec![
            "route",
            verb,
            "default",
            "via",
            GATEWAY_LAN_IP,
            "dev",
            self.sender_iface(idx),
            "metric",
        ];
        args.push(METRICS[idx]);
        args
    }

    /// Sender <-> carrier: the segment that carries the duplicated address.
    fn wire_access(&self, idx: usize) -> Result<()> {
        let names = &self.names[idx];
        self.sender_ns.add_veth_link(
            &self.gateways[idx],
            &names.sender,
            &names.gateway_lan,
            &format!("{TWIN_IP}/24"),
            &format!("{GATEWAY_LAN_IP}/24"),
        )?;
        // Effective rp_filter is max(all, dev); a freshly created device can
        // still carry 1, and strict reverse-path silently eats every reply (and
        // every ARP request) for the address the twins share.
        self.sender_ns.exec_checked(
            "sysctl",
            &[
                "-qw",
                &format!("net.ipv4.conf.{}.rp_filter=0", names.sender),
            ],
        )?;
        self.restore_route(idx)
    }

    /// Carrier <-> receiver: its own subnet, masqueraded, so the receiver sees
    /// two distinct sources for the one sender address.
    fn wire_transit(&self, idx: usize) -> Result<()> {
        let names = &self.names[idx];
        let gateway = &self.gateways[idx];
        let subnet = idx + 1;
        gateway.add_veth_link(
            &self.receiver_ns,
            &names.gateway_wan,
            &names.receiver,
            &format!("10.31.{subnet}.1/24"),
            &format!("10.31.{subnet}.2/24"),
        )?;
        gateway.exec_checked("sysctl", &["-qw", "net.ipv4.ip_forward=1"])?;
        gateway.exec_checked(
            "ip",
            &[
                "route",
                "add",
                &format!("{RECEIVER_IP}/32"),
                "via",
                &format!("10.31.{subnet}.2"),
                "dev",
                &names.gateway_wan,
            ],
        )?;
        gateway.exec_checked(
            "iptables",
            &[
                "-t",
                "nat",
                "-A",
                "POSTROUTING",
                "-o",
                &names.gateway_wan,
                "-j",
                "MASQUERADE",
            ],
        )?;
        // Answer from the address the sender dialed, not from the transit hop.
        self.receiver_ns.exec_checked(
            "ip",
            &[
                "route",
                "replace",
                &format!("10.31.{subnet}.0/24"),
                "dev",
                &names.receiver,
                "scope",
                "link",
                "src",
                RECEIVER_IP,
            ],
        )?;
        Ok(())
    }

    fn relax_reverse_path(&self) -> Result<()> {
        let namespaces = std::iter::once(&self.sender_ns)
            .chain(std::iter::once(&self.receiver_ns))
            .chain(self.gateways.iter());
        for ns in namespaces {
            for key in ["all", "default"] {
                ns.exec_checked(
                    "sysctl",
                    &["-qw", &format!("net.ipv4.conf.{key}.rp_filter=0")],
                )?;
            }
        }
        Ok(())
    }
}

/// Distinct metrics keep the twins' two default routes from colliding.
const METRICS: [&str; 4] = ["100", "101", "102", "103"];
