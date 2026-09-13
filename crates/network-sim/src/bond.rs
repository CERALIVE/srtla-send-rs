//! N-link namespace bond with explicit carrier and mapping policies.

mod publication;
mod routing;
mod spec;
mod wiring;

use std::path::PathBuf;

use anyhow::{Context, Result};
pub use routing::configure_bond_routing;
use spec::Address;
pub use spec::{BondConfigError, BondRow, CarrierMode, LinkSpec, MappingMode};

use crate::twin::BindMapPublisher;
use crate::{ImpairmentConfig, Namespace, NamespaceProcess, unique_ns_name};

pub const RECEIVER_IP: &str = "10.99.0.1";

struct Link {
    sender: String,
    peer: String,
    address: Address,
    carrier: Carrier,
}

enum Carrier {
    Direct,
    Nat {
        ns: Namespace,
        wan: String,
        receiver: String,
    },
}

/// Owns namespaces and launch files, but not processes. Drop process handles before this value.
pub struct BondTopology {
    pub sender_ns: Namespace,
    pub receiver_ns: Namespace,
    links: Vec<Link>,
    publisher: BindMapPublisher,
    mapping: MappingMode,
}

impl BondTopology {
    /// Validate the whole configuration before creating any namespace.
    pub fn new(test_name: &str, links: &[LinkSpec], mapping: MappingMode) -> Result<Self> {
        spec::validate(links, &mapping)?;
        let sender_ns = Namespace::new(&unique_ns_name(&format!("{test_name}_s")))?;
        let receiver_ns = Namespace::new(&unique_ns_name(&format!("{test_name}_r")))?;
        let addresses = spec::addresses(links);
        let links = links
            .iter()
            .zip(addresses)
            .map(|(spec, address)| {
                let carrier = match spec.carrier {
                    CarrierMode::Direct => Carrier::Direct,
                    CarrierMode::Nat => Carrier::Nat {
                        ns: Namespace::new(&unique_ns_name("bg"))?,
                        wan: unique_ns_name("bw"),
                        receiver: unique_ns_name("br"),
                    },
                };
                Ok(Link {
                    sender: unique_ns_name("bs"),
                    peer: unique_ns_name("bp"),
                    address,
                    carrier,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let topo = Self {
            sender_ns,
            receiver_ns,
            links,
            publisher: BindMapPublisher::new()?,
            mapping,
        };
        topo.wire()?;
        topo.publish()?;
        Ok(topo)
    }

    pub fn sender_iface(&self, index: usize) -> &str {
        &self.links[index].sender
    }
    pub fn sender_ip(&self, index: usize) -> &str {
        &self.links[index].address.ip
    }
    pub fn link_count(&self) -> usize {
        self.links.len()
    }
    pub fn ips_path(&self) -> PathBuf {
        self.publisher.ips_path()
    }
    pub fn sidecar_path(&self) -> Option<PathBuf> {
        match self.mapping {
            MappingMode::BindMap { .. } => Some(self.publisher.sidecar_path()),
            MappingMode::LegacyControl | MappingMode::None => None,
        }
    }

    /// Kernel netdev bytes, including ARP/control traffic; resets after replug.
    pub fn tx_bytes(&self, index: usize) -> Result<u64> {
        let path = format!(
            "/sys/class/net/{}/statistics/tx_bytes",
            self.sender_iface(index)
        );
        let out = self.sender_ns.exec_checked("cat", &[&path])?;
        String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse()
            .context("parse netdev tx_bytes")
    }

    pub fn set_link_up(&self, index: usize, up: bool) -> Result<()> {
        self.sender_ns.exec_checked(
            "ip",
            &[
                "link",
                "set",
                self.sender_iface(index),
                if up { "up" } else { "down" },
            ],
        )?;
        Ok(())
    }

    pub fn delete_default_route(&self, index: usize) -> Result<()> {
        self.default_routes(index, "del")
    }
    pub fn restore_default_route(&self, index: usize) -> Result<()> {
        self.default_routes(index, "add")
    }

    /// Destroy and recreate the access veth under the same name and a new ifindex.
    /// Shaping is reset; callers explicitly reapply their impairment afterwards.
    pub fn replug(&self, index: usize) -> Result<()> {
        self.sender_ns
            .exec_checked("ip", &["link", "del", self.sender_iface(index)])?;
        self.wire_access(index, false)
    }

    pub fn apply_impairment(&self, index: usize, config: &ImpairmentConfig) -> Result<()> {
        crate::apply_impairment(&self.sender_ns, self.sender_iface(index), config.clone())
    }

    /// Append caller-selected scheduler/control arguments after the topology-owned positionals.
    pub fn sender_args(&self, ports: (u16, u16), extra: &[&str]) -> Result<Vec<String>> {
        let ips = self.ips_path();
        let mut args = vec![
            ports.0.to_string(),
            RECEIVER_IP.into(),
            ports.1.to_string(),
            ips.to_str().context("IP path is not UTF-8")?.into(),
        ];
        if let Some(sidecar) = self.sidecar_path() {
            args.extend([
                "--bind-map".into(),
                sidecar
                    .to_str()
                    .context("sidecar path is not UTF-8")?
                    .into(),
            ]);
        }
        args.extend(extra.iter().map(|arg| (*arg).to_owned()));
        Ok(args)
    }

    /// Start only the sender; receiver/profile selection remains explicit at the stack layer.
    pub fn spawn_sender(&self, binary: &str, extra: &[&str]) -> Result<NamespaceProcess> {
        let args = self.sender_args((5555, 5000), extra)?;
        NamespaceProcess::spawn_with_env(
            &self.sender_ns,
            binary,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
            &[("RUST_LOG", "debug")],
        )
    }
}

#[cfg(test)]
mod tests;
