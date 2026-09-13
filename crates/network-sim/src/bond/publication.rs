use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use super::{BondTopology, MappingMode};
use crate::twin::{BindMapPublisher, TwinRow};

impl BondTopology {
    pub(super) fn publish(&self) -> Result<()> {
        match &self.mapping {
            MappingMode::BindMap { rows } => {
                let rows = rows
                    .iter()
                    .map(|row| {
                        let iface = self.sender_iface(row.iface_index);
                        let twin = match row.priority {
                            Some(priority) => TwinRow::with_priority(&row.link_id, iface, priority),
                            None => TwinRow::new(&row.link_id, iface),
                        };
                        (self.sender_ip(row.iface_index), twin)
                    })
                    .collect::<Vec<_>>();
                self.publisher.publish_bond_at(&rows, 1)
            }
            MappingMode::LegacyControl | MappingMode::None => {
                let ips = (0..self.link_count())
                    .map(|i| self.sender_ip(i))
                    .collect::<Vec<_>>()
                    .join("\n")
                    + "\n";
                atomic(&self.publisher.ips_path(), ips.as_bytes())
            }
        }
    }
}

impl BindMapPublisher {
    // The twin API takes one repeated IP. Keep the heterogeneous-IP extension here
    // so the frozen twin publication API and implementation need no restructuring.
    fn publish_bond_at(&self, rows: &[(&str, TwinRow)], generation: u64) -> Result<()> {
        let ips = rows
            .iter()
            .map(|(ip, _)| *ip)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let digest: String = Sha256::digest(ips.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let links = rows
            .iter()
            .map(|(ip, row)| {
                let priority = match row.priority {
                    Some(value) => format!(r#","priority":{value}"#),
                    None => String::new(),
                };
                // Configuration validation guarantees printable ASCII IDs; Debug escapes
                // quotes/backslashes as JSON requires. IPs and interfaces are generated.
                format!(
                    r#"{{"link_id":{:?},"ip":{ip:?},"iface":{:?}{priority}}}"#,
                    row.link_id, row.iface
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let sidecar = format!(
            r#"{{"schema_version":1,"generation":{generation},"ips_file_sha256":"{digest}","links":[{links}]}}"#
        );
        atomic(&self.ips_path(), ips.as_bytes())?;
        atomic(&self.sidecar_path(), sidecar.as_bytes())
    }
}

fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut temp = tempfile::NamedTempFile::new_in(path.parent().context("publication parent")?)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist(path)
        .with_context(|| format!("publish {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bond_publication_preserves_distinct_ip_order_and_priority() {
        // Given: two distinct IPs in deliberately reversed interface order.
        let publisher = BindMapPublisher::new().unwrap();
        let rows = [
            ("10.30.2.1", TwinRow::with_priority("b", "bs1", 0.25)),
            ("10.10.1.1", TwinRow::new("a", "bs0")),
        ];
        // When: publishing the heterogeneous pair.
        publisher.publish_bond_at(&rows, 7).unwrap();
        // Then: the IP bytes, digest and sidecar rows refer to the same ordering.
        let ips = std::fs::read(publisher.ips_path()).unwrap();
        assert_eq!(ips, b"10.30.2.1\n10.10.1.1\n");
        let digest = "91fdc0764c18854ed9e217c3b337bfd63b3da2df65bfbb540d7a66e4e7cc70c3";
        let expected = format!(
            r#"{{"schema_version":1,"generation":7,"ips_file_sha256":"{digest}","links":[{{"link_id":"b","ip":"10.30.2.1","iface":"bs1","priority":0.25}},{{"link_id":"a","ip":"10.10.1.1","iface":"bs0"}}]}}"#
        );
        assert_eq!(
            std::fs::read_to_string(publisher.sidecar_path()).unwrap(),
            expected
        );
    }

    #[test]
    fn bond_publication_matches_the_twin_publisher_for_shared_ips() {
        // Given: the original two-row shared-IP writer and the bond extension.
        let twin = BindMapPublisher::new().unwrap();
        let bond = BindMapPublisher::new().unwrap();
        let rows = [TwinRow::new("a", "bs0"), TwinRow::new("b", "bs1")];
        twin.publish_at("10.30.9.1", &rows, 1).unwrap();
        // When: the bond extension writes those same rows.
        let paired = rows
            .into_iter()
            .map(|row| ("10.30.9.1", row))
            .collect::<Vec<_>>();
        bond.publish_bond_at(&paired, 1).unwrap();
        // Then: both artifacts are byte-identical to the existing publisher.
        assert_eq!(
            std::fs::read(bond.ips_path()).unwrap(),
            std::fs::read(twin.ips_path()).unwrap()
        );
        assert_eq!(
            std::fs::read(bond.sidecar_path()).unwrap(),
            std::fs::read(twin.sidecar_path()).unwrap()
        );
    }
}
