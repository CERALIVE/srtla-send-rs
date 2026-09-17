use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CarrierMode {
    Direct,
    Nat,
}

#[derive(Debug, Clone, Copy)]
pub struct LinkSpec {
    pub carrier: CarrierMode,
    /// Zero-based reference to an earlier link. Every member of the group must be NAT.
    pub shared_ip_with: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct BondRow {
    pub link_id: String,
    /// Zero-based topology index; row order determines IP-file order.
    pub iface_index: usize,
    pub priority: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum MappingMode {
    BindMap { rows: Vec<BondRow> },
    LegacyControl,
    None,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BondConfigError {
    LinkCount { count: usize },
    SharedIpRequiresExplicitMappingMode,
    InvalidSharedIndex { index: usize, target: usize },
    SharedIpRequiresNat { index: usize },
    InvalidMappingRow { index: usize },
    MappingRowCount { expected: usize, actual: usize },
}

impl fmt::Display for BondConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LinkCount { count } => write!(f, "bond needs 1..=253 links, got {count}"),
            Self::SharedIpRequiresExplicitMappingMode => {
                f.write_str("shared IPs require BindMap or LegacyControl")
            }
            Self::InvalidSharedIndex { index, target } => write!(
                f,
                "link {index} must reference an earlier link, got {target}"
            ),
            Self::SharedIpRequiresNat { index } => {
                write!(f, "shared-IP link {index} requires its own NAT carrier")
            }
            Self::InvalidMappingRow { index } => {
                write!(f, "invalid or duplicate bond mapping row {index}")
            }
            Self::MappingRowCount { expected, actual } => {
                write!(f, "expected {expected} mapping rows, got {actual}")
            }
        }
    }
}

impl std::error::Error for BondConfigError {}

pub(super) fn validate(links: &[LinkSpec], mapping: &MappingMode) -> Result<(), BondConfigError> {
    if links.is_empty() || links.len() > 253 {
        return Err(BondConfigError::LinkCount { count: links.len() });
    }
    match mapping {
        MappingMode::None => {
            if links.iter().any(|link| link.shared_ip_with.is_some()) {
                return Err(BondConfigError::SharedIpRequiresExplicitMappingMode);
            }
        }
        MappingMode::LegacyControl => {}
        MappingMode::BindMap { rows } => {
            if rows.len() != links.len() {
                return Err(BondConfigError::MappingRowCount {
                    expected: links.len(),
                    actual: rows.len(),
                });
            }
            let mut identities = HashSet::new();
            let mut interfaces = HashSet::new();
            for (index, row) in rows.iter().enumerate() {
                if row.iface_index >= links.len()
                    || !interfaces.insert(row.iface_index)
                    || !identities.insert(&row.link_id)
                    || row.link_id.is_empty()
                    || row.link_id.len() > 64
                    || !row
                        .link_id
                        .bytes()
                        .all(|byte| (0x21..=0x7e).contains(&byte))
                    || row.priority.is_some_and(|value| !value.is_finite())
                {
                    return Err(BondConfigError::InvalidMappingRow { index });
                }
            }
        }
    }
    for (index, link) in links.iter().enumerate() {
        if let Some(target) = link.shared_ip_with {
            if target >= index {
                return Err(BondConfigError::InvalidSharedIndex { index, target });
            }
            for member in [target, index] {
                match links[member].carrier {
                    CarrierMode::Direct => {
                        return Err(BondConfigError::SharedIpRequiresNat { index: member });
                    }
                    CarrierMode::Nat => {}
                }
            }
        }
    }
    Ok(())
}

pub(super) struct Address {
    pub ip: String,
    pub gateway: String,
    pub subnet: String,
    pub shared: bool,
}

pub(super) fn addresses(links: &[LinkSpec]) -> Vec<Address> {
    let mut roots = Vec::with_capacity(links.len());
    for (index, link) in links.iter().enumerate() {
        roots.push(match link.shared_ip_with {
            Some(target) => roots[target],
            None => index,
        });
    }
    let first_shared = roots
        .iter()
        .copied()
        .find(|root| roots.iter().filter(|candidate| *candidate == root).count() > 1);
    links
        .iter()
        .enumerate()
        .map(|(index, link)| {
            let root = roots[index];
            let shared = roots.iter().filter(|candidate| **candidate == root).count() > 1;
            let subnet = if Some(root) == first_shared {
                9
            } else if root < 8 {
                root + 1
            } else {
                root + 2
            };
            let prefix = match link.carrier {
                CarrierMode::Direct => "10.10",
                CarrierMode::Nat => "10.30",
            };
            Address {
                ip: format!("{prefix}.{subnet}.1"),
                gateway: format!("{prefix}.{subnet}.2"),
                subnet: format!("{prefix}.{subnet}.0/24"),
                shared,
            }
        })
        .collect()
}
