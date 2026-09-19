use anyhow::{Context, Result, ensure};

use super::{
    Address, BondRow, BondTopology, Carrier, CarrierMode, Link, LinkSpec, MappingMode,
    PrioritySidecar, spec,
};
use crate::profile::LinkProfile;
use crate::profile::qdisc::LinkQdisc;
use crate::{Namespace, unique_ns_name};

impl BondTopology {
    pub(super) fn setup_link(&mut self, profile: &LinkProfile, address: Address) -> Result<usize> {
        let carrier = match profile.carrier {
            CarrierMode::Direct => Carrier::Direct,
            CarrierMode::Nat => Carrier::Nat {
                ns: Namespace::new(&unique_ns_name("bg"))?,
                wan: unique_ns_name("bw"),
                receiver: unique_ns_name("br"),
            },
        };
        let sender = unique_ns_name("bs");
        let index = self.links.len();
        self.links.push(Link {
            qdisc: LinkQdisc::new(&self.sender_ns, &sender),
            sender,
            peer: unique_ns_name("bp"),
            address,
            carrier,
        });
        self.wire_link(index)?;
        self.apply_impairment(index, &profile.base)?;
        Ok(index)
    }

    /// Create and publish a new unique-IP link; the runtime signals the sender afterwards.
    /// Partial setup remains owned by this topology until scenario teardown on error.
    pub fn add_link(&mut self, profile: &LinkProfile) -> Result<usize> {
        crate::profile::qdisc::apply_commands("validation", &profile.base)?;
        let index = self.link_count();
        ensure!(index < 253, "bond exceeds 253 links");
        let row = BondRow {
            link_id: format!("link-{index}"),
            iface_index: index,
            priority: self.priorities.as_ref().and_then(|p| p.priority(index)),
        };
        match &self.mapping {
            MappingMode::BindMap { rows } => ensure!(
                !rows.iter().any(|r| r.link_id == row.link_id),
                "new link identity collides with mapping"
            ),
            MappingMode::None | MappingMode::LegacyControl => {}
        }
        let added = LinkSpec {
            carrier: profile.carrier,
            shared_ip_with: None,
        };
        let mut specs = self.specs.clone();
        specs.push(added);
        let address = spec::addresses(&specs).pop().context("new link address")?;
        self.setup_link(profile, address)?;
        self.specs.push(added);
        match &mut self.mapping {
            MappingMode::BindMap { rows } => rows.push(row),
            MappingMode::None | MappingMode::LegacyControl => {}
        }
        let mut order = self.order.borrow().clone();
        order.push(index);
        self.reorder(&order)?;
        Ok(index)
    }

    /// Opt in before spawning; future additions retain their configured priority by index.
    pub fn set_priority_sidecar(&mut self, priorities: PrioritySidecar) -> Result<()> {
        self.mapping = MappingMode::BindMap {
            rows: (0..self.link_count())
                .map(|index| BondRow {
                    link_id: format!("link-{index}"),
                    iface_index: index,
                    priority: priorities.priority(index),
                })
                .collect(),
        };
        self.priorities = Some(priorities);
        let order = self.order.borrow().clone();
        self.reorder(&order)
    }
}
