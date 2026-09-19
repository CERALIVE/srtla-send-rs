use std::path::Path;

use anyhow::{Context, Result, ensure};

use super::{BondTopology, MappingMode};
use crate::twin::TwinRow;

impl BondTopology {
    pub(super) fn publish(&self) -> Result<()> {
        self.publish_order(&self.order.borrow(), self.generation.get())
    }

    /// Order contains topology indices, not positions in the previously published file.
    /// Mapped publication advances generation and keeps the sidecar hash coherent.
    pub fn reorder(&self, order: &[usize]) -> Result<()> {
        let mut sorted = order.to_vec();
        sorted.sort_unstable();
        ensure!(
            sorted == (0..self.link_count()).collect::<Vec<_>>(),
            "reorder must be a complete permutation"
        );
        let generation = self
            .generation
            .get()
            .checked_add(1)
            .context("bind-map generation overflow")?;
        self.publish_order(order, generation)?;
        self.generation.set(generation);
        *self.order.borrow_mut() = order.to_vec();
        Ok(())
    }

    fn publish_order(&self, order: &[usize], generation: u64) -> Result<()> {
        match &self.mapping {
            MappingMode::BindMap { rows } => {
                let rows = order
                    .iter()
                    .map(|index| {
                        let row = rows
                            .iter()
                            .find(|row| row.iface_index == *index)
                            .context("mapped topology index")?;
                        let iface = self.sender_iface(row.iface_index);
                        let twin = match row.priority {
                            Some(priority) => TwinRow::with_priority(&row.link_id, iface, priority),
                            None => TwinRow::new(&row.link_id, iface),
                        };
                        Ok((self.sender_ip(row.iface_index), twin))
                    })
                    .collect::<Result<Vec<_>>>()?;
                self.publisher.publish_bond_at(&rows, generation)
            }
            MappingMode::LegacyControl | MappingMode::None => {
                let ips = order.iter().map(|i| self.sender_ip(*i)).collect::<Vec<_>>();
                self.publisher.publish_ips(&ips)
            }
        }
    }
}

pub(super) fn launch_args(
    ports: (u16, u16),
    paths: (&Path, Option<&Path>),
    extra: &[&str],
) -> Result<Vec<String>> {
    let mut args = vec![
        ports.0.to_string(),
        super::RECEIVER_IP.into(),
        ports.1.to_string(),
        paths.0.to_str().context("IP path is not UTF-8")?.into(),
    ];
    if let Some(sidecar) = paths.1 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::twin::BindMapPublisher;

    #[test]
    fn legacy_publication_and_sender_args_stay_byte_identical() {
        // Given: a cell with no sidecar policy.
        let publisher = BindMapPublisher::new().unwrap();
        // When: publishing the legacy IP list and composing its sender invocation.
        publisher.publish_ips(&["10.10.1.1", "10.30.2.1"]).unwrap();
        let args = launch_args(
            (5555, 5000),
            (Path::new("ips.txt"), None),
            &["--mode", "classic"],
        )
        .unwrap();
        // Then: exact legacy bytes/args and no sidecar artifact.
        assert_eq!(
            std::fs::read(publisher.ips_path()).unwrap(),
            b"10.10.1.1\n10.30.2.1\n"
        );
        assert!(!publisher.sidecar_path().exists());
        assert_eq!(
            args,
            ["5555", "10.99.0.1", "5000", "ips.txt", "--mode", "classic"]
        );
    }

    #[test]
    fn priority_sidecar_pins_valid_row_bytes_and_sender_flag() {
        // Given: one prioritized link and an omitted priority on the other.
        let publisher = BindMapPublisher::new().unwrap();
        let rows = [
            ("10.30.2.1", TwinRow::with_priority("link-1", "bs1", 0.2)),
            ("10.10.1.1", TwinRow::new("link-0", "bs0")),
        ];
        // When: publishing through the shared twin writer.
        publisher.publish_bond_at(&rows, 7).unwrap();
        let args = launch_args(
            (5555, 5000),
            (Path::new("ips.txt"), Some(Path::new("map.json"))),
            &["--mode", "classic"],
        )
        .unwrap();
        // Then: optional priority is pinned byte-for-byte and the sidecar flag precedes extras.
        assert_eq!(std::fs::read(publisher.sidecar_path()).unwrap(), br#"{"schema_version":1,"generation":7,"ips_file_sha256":"91fdc0764c18854ed9e217c3b337bfd63b3da2df65bfbb540d7a66e4e7cc70c3","links":[{"link_id":"link-1","ip":"10.30.2.1","iface":"bs1","priority":0.2},{"link_id":"link-0","ip":"10.10.1.1","iface":"bs0"}]}"#);
        assert_eq!(
            args,
            [
                "5555",
                "10.99.0.1",
                "5000",
                "ips.txt",
                "--bind-map",
                "map.json",
                "--mode",
                "classic"
            ]
        );
    }

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
