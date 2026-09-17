use anyhow::Result;

use super::routing::{SourceRoute, ip, relax, source_table};
use super::{BondTopology, Carrier, RECEIVER_IP};

impl BondTopology {
    pub(super) fn wire(&self) -> Result<()> {
        for ns in [self.sender_ns.as_ref(), &self.receiver_ns] {
            for scope in ["all", "default"] {
                relax(ns, scope)?;
            }
        }
        ip(
            &self.receiver_ns,
            &format!("addr add {RECEIVER_IP}/32 dev lo"),
        )?;
        Ok(())
    }

    pub(super) fn wire_link(&self, index: usize) -> Result<()> {
        let link = &self.links[index];
        match &link.carrier {
            Carrier::Direct => {}
            Carrier::Nat { ns, wan, receiver } => {
                for scope in ["all", "default"] {
                    relax(ns, scope)?;
                }
                let subnet = index + 1;
                ns.add_veth_link(
                    &self.receiver_ns,
                    wan,
                    receiver,
                    &format!("100.64.{subnet}.1/24"),
                    &format!("100.64.{subnet}.2/24"),
                )?;
                relax(ns, wan)?;
                relax(&self.receiver_ns, receiver)?;
                ns.exec_checked("sysctl", &["-qw", "net.ipv4.ip_forward=1"])?;
                ip(
                    ns,
                    &format!("route replace {RECEIVER_IP} via 100.64.{subnet}.2 dev {wan}"),
                )?;
                ns.exec_checked(
                    "iptables",
                    &[
                        "-t",
                        "nat",
                        "-A",
                        "POSTROUTING",
                        "-o",
                        wan,
                        "-j",
                        "MASQUERADE",
                    ],
                )?;
                ip(
                    &self.receiver_ns,
                    &format!(
                        "route replace 100.64.{subnet}.0/24 dev {receiver} scope link src \
                         {RECEIVER_IP}"
                    ),
                )?;
            }
        }
        self.wire_access(index, true)
    }

    pub(super) fn wire_access(&self, index: usize, install_rule: bool) -> Result<()> {
        let link = &self.links[index];
        let peer_ns = match &link.carrier {
            Carrier::Direct => &self.receiver_ns,
            Carrier::Nat { ns, .. } => ns,
        };
        self.sender_ns.add_veth_link(
            peer_ns,
            &link.sender,
            &link.peer,
            &format!("{}/24", link.address.ip),
            &format!("{}/24", link.address.gateway),
        )?;
        relax(&self.sender_ns, &link.sender)?;
        relax(peer_ns, &link.peer)?;
        match link.carrier {
            Carrier::Direct => ip(
                &self.receiver_ns,
                &format!(
                    "route replace {} dev {} scope link src {RECEIVER_IP}",
                    link.address.subnet, link.peer
                ),
            )?,
            Carrier::Nat { .. } => {}
        }
        if !link.address.shared {
            let route = SourceRoute {
                iface: &link.sender,
                subnet: &link.address.subnet,
                gateway: &link.address.gateway,
                ip: &link.address.ip,
                table: source_table(index),
            };
            let [connected, default, rule] = route.commands();
            ip(&self.sender_ns, &connected)?;
            ip(&self.sender_ns, &default)?;
            if install_rule {
                ip(&self.sender_ns, &rule)?;
            }
        }
        ip(&self.sender_ns, &self.main_default(index, "add"))
    }

    fn main_default(&self, index: usize, verb: &str) -> String {
        let link = &self.links[index];
        format!(
            "route {verb} default via {} dev {} metric {}",
            link.address.gateway,
            link.sender,
            100 + index
        )
    }

    pub(super) fn default_routes(&self, index: usize, verb: &str) -> Result<()> {
        let link = &self.links[index];
        if !link.address.shared {
            ip(
                &self.sender_ns,
                &format!(
                    "route {verb} default via {} dev {} table {}",
                    link.address.gateway,
                    link.sender,
                    source_table(index)
                ),
            )?;
        }
        ip(&self.sender_ns, &self.main_default(index, verb))
    }

    pub(super) fn restore_link_routes(&self, index: usize) -> Result<()> {
        let link = &self.links[index];
        if !link.address.shared {
            ip(
                &self.sender_ns,
                &format!(
                    "route replace {} dev {} table {}",
                    link.address.subnet,
                    link.sender,
                    source_table(index)
                ),
            )?;
        }
        self.default_routes(index, "replace")
    }
}
