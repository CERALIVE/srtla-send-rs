//! N-link namespace bond with explicit carrier and mapping policies.

mod lifecycle;
mod priority;
mod publication;
mod routing;
mod spec;
mod wiring;

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
pub use priority::PrioritySidecar;
pub use routing::configure_bond_routing;
use spec::Address;
pub use spec::{BondConfigError, BondRow, CarrierMode, LinkSpec, MappingMode};

use crate::profile::qdisc::LinkQdisc;
use crate::twin::BindMapPublisher;
use crate::{ImpairmentConfig, Namespace, NamespaceProcess, unique_ns_name};

pub const RECEIVER_IP: &str = "10.99.0.1";

struct Link {
    sender: String,
    peer: String,
    address: Address,
    carrier: Carrier,
    qdisc: LinkQdisc,
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
    pub sender_ns: Arc<Namespace>,
    pub receiver_ns: Namespace,
    links: Vec<Link>,
    specs: Vec<LinkSpec>,
    order: RefCell<Vec<usize>>,
    priorities: Option<PrioritySidecar>,
    publisher: BindMapPublisher,
    mapping: MappingMode,
    generation: Cell<u64>,
}

impl BondTopology {
    /// Validate the whole configuration before creating any namespace.
    pub fn new(test_name: &str, links: &[LinkSpec], mapping: MappingMode) -> Result<Self> {
        spec::validate(links, &mapping)?;
        let sender_ns = Arc::new(Namespace::new(&unique_ns_name(&format!("{test_name}_s")))?);
        let receiver_ns = Namespace::new(&unique_ns_name(&format!("{test_name}_r")))?;
        let addresses = spec::addresses(links);
        let order = match &mapping {
            MappingMode::BindMap { rows } => rows.iter().map(|row| row.iface_index).collect(),
            MappingMode::None | MappingMode::LegacyControl => (0..links.len()).collect(),
        };
        let mut topo = Self {
            sender_ns,
            receiver_ns,
            links: Vec::new(),
            specs: links.to_vec(),
            order: RefCell::new(order),
            priorities: None,
            publisher: BindMapPublisher::new()?,
            mapping,
            generation: Cell::new(1),
        };
        topo.wire()?;
        for (spec, address) in links.iter().zip(addresses) {
            topo.setup_link(
                &crate::profile::LinkProfile {
                    carrier: spec.carrier,
                    base: ImpairmentConfig::default(),
                },
                address,
            )?;
        }
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
    pub fn carrier_mode(&self, index: usize) -> CarrierMode {
        match self.links[index].carrier {
            Carrier::Direct => CarrierMode::Direct,
            Carrier::Nat { .. } => CarrierMode::Nat,
        }
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
        if up {
            self.restore_link_routes(index)?;
        }
        Ok(())
    }

    pub fn delete_default_route(&self, index: usize) -> Result<()> {
        self.default_routes(index, "del")
    }
    pub fn restore_default_route(&self, index: usize) -> Result<()> {
        self.default_routes(index, "replace")
    }

    /// Destroy and recreate the access veth under the same name and a new ifindex.
    /// Shaping is reset; callers explicitly reapply their impairment afterwards.
    pub fn replug(&self, index: usize) -> Result<()> {
        self.sender_ns
            .exec_checked("ip", &["link", "del", self.sender_iface(index)])?;
        self.wire_access(index, false)?;
        self.links[index].qdisc.reset_after_replug();
        self.links[index].qdisc.apply(&ImpairmentConfig::default())
    }

    pub fn apply_impairment(&self, index: usize, config: &ImpairmentConfig) -> Result<()> {
        self.links[index].qdisc.apply(config)
    }

    pub fn set_data_blackhole(&self, index: usize, on: bool) -> Result<()> {
        self.links[index].qdisc.set_blackhole(on)
    }

    /// Append caller-selected scheduler/control arguments after the topology-owned positionals.
    pub fn sender_args(&self, ports: (u16, u16), extra: &[&str]) -> Result<Vec<String>> {
        publication::launch_args(
            ports,
            (&self.ips_path(), self.sidecar_path().as_deref()),
            extra,
        )
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
