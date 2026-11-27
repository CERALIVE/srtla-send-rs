//! Overlapping `--bind-map` reloads (ADR-003 §4.3), driven through real pair
//! reads so the uplink set each read leaves in force is what is checked.
#![cfg(target_os = "linux")]

use std::path::PathBuf;

use crate::bind_map::{
    BindMapDisposition, BindMapStatus, BindMapStatusRecord, DegradedReason, IpsFile, LinkId,
};
use crate::net::UplinkSpec;
use crate::sender::{LinkSource, SenderPaths};

/// One `ips` + sidecar pair on disk. Every mapping uses the same IP list, so
/// only `generation` and the row's `link_id` tell two mappings apart.
struct Pair {
    dir: tempfile::TempDir,
}

impl Pair {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn ips(&self) -> PathBuf {
        self.dir.path().join("ips")
    }

    fn sidecar(&self) -> PathBuf {
        self.dir.path().join("map.json")
    }

    fn publish(&self, generation: u64, link_id: &str) {
        let ips = "127.0.0.1\n";
        std::fs::write(self.ips(), ips).unwrap();
        let sha = IpsFile::from_bytes(ips.as_bytes())
            .unwrap()
            .sha256()
            .to_string();
        std::fs::write(
            self.sidecar(),
            format!(
                r#"{{"schema_version":1,"generation":{generation},"ips_file_sha256":"{sha}",
                "links":[{{"link_id":"{link_id}","ip":"127.0.0.1","iface":"lo"}}]}}"#
            ),
        )
        .unwrap();
    }

    async fn started_source(&self) -> LinkSource {
        let mut source = LinkSource::new(&SenderPaths {
            ips_file: self.ips().to_str().unwrap(),
            bind_map: Some(self.sidecar().to_str().unwrap()),
        });
        source.startup().await.unwrap();
        source
    }
}

fn link_ids(specs: &[UplinkSpec]) -> Vec<Option<LinkId>> {
    specs.iter().map(|spec| spec.link_id.clone()).collect()
}

fn id(raw: &str) -> Option<LinkId> {
    Some(LinkId::parse(raw).unwrap())
}

async fn reload_now(source: &mut LinkSource) -> Vec<Option<LinkId>> {
    let (order, read) = source.begin_reload().unwrap().run().await;
    link_ids(&source.adopt_reload(order, read).unwrap())
}

#[tokio::test]
async fn an_older_reload_finishing_last_does_not_replace_the_newer_links() {
    let pair = Pair::new();
    pair.publish(1, "start");
    let mut source = pair.started_source().await;
    let older = source.begin_reload().unwrap();
    let newer = source.begin_reload().unwrap();
    pair.publish(2, "older");
    let (older_order, older_read) = older.run().await;
    pair.publish(3, "newer");
    let (newer_order, newer_read) = newer.run().await;

    let applied = source.adopt_reload(newer_order, newer_read).unwrap();
    assert_eq!(link_ids(&applied), [id("newer")]);
    let report = source.report().clone();
    assert!(
        source.adopt_reload(older_order, older_read).is_none(),
        "the older read finished after the newer one was adopted, so it must be dropped"
    );
    assert_eq!(source.report(), &report);

    // Generation 3 with other rows is stale only against the newer mapping, so
    // this proves which mapping stayed in force.
    pair.publish(3, "other");
    assert_eq!(reload_now(&mut source).await, [id("newer")]);
}

/// Both reads were validated against generation 1. The first applies
/// generation 2; the second carries generation 2 with other rows, which against
/// what is applied by then is a stale generation.
#[tokio::test]
async fn a_second_overlapping_read_with_the_same_generation_and_other_rows_is_refused() {
    let pair = Pair::new();
    pair.publish(1, "start");
    let mut source = pair.started_source().await;
    let first = source.begin_reload().unwrap();
    let second = source.begin_reload().unwrap();
    pair.publish(2, "first");
    let (first_order, first_read) = first.run().await;
    pair.publish(2, "second");
    let (second_order, second_read) = second.run().await;

    let applied = source.adopt_reload(first_order, first_read).unwrap();
    assert_eq!(link_ids(&applied), [id("first")]);
    let kept = source.adopt_reload(second_order, second_read).unwrap();
    assert_eq!(link_ids(&kept), [id("first")]);
    assert_eq!(
        source.report().disposition.state,
        BindMapDisposition::RetainedLastValid.as_str()
    );
    assert_eq!(
        source.report().bind_map_status,
        BindMapStatusRecord::from(BindMapStatus::Degraded(DegradedReason::Malformed))
    );
}

#[tokio::test]
async fn reload_reads_that_finish_in_order_are_each_applied() {
    let pair = Pair::new();
    pair.publish(1, "start");
    let mut source = pair.started_source().await;
    let older = source.begin_reload().unwrap();
    let newer = source.begin_reload().unwrap();
    pair.publish(2, "older");
    let (older_order, older_read) = older.run().await;
    pair.publish(3, "newer");
    let (newer_order, newer_read) = newer.run().await;

    let first = source.adopt_reload(older_order, older_read).unwrap();
    assert_eq!(link_ids(&first), [id("older")]);
    let second = source.adopt_reload(newer_order, newer_read).unwrap();
    assert_eq!(link_ids(&second), [id("newer")]);
    assert_eq!(
        source.report().disposition.state,
        BindMapDisposition::Mapped.as_str()
    );
}

/// The newer read found no IP file, so it applies nothing and the running
/// links stay; the older usable read that finishes after it is still dropped.
#[tokio::test]
async fn a_failed_newer_read_still_supersedes_an_older_one() {
    let pair = Pair::new();
    pair.publish(1, "start");
    let mut source = pair.started_source().await;
    let older = source.begin_reload().unwrap();
    let newer = source.begin_reload().unwrap();
    pair.publish(2, "older");
    let (older_order, older_read) = older.run().await;
    std::fs::remove_file(pair.ips()).unwrap();
    let (newer_order, newer_read) = newer.run().await;

    assert!(
        source
            .adopt_reload(newer_order, newer_read)
            .unwrap()
            .is_empty()
    );
    assert!(source.adopt_reload(older_order, older_read).is_none());
    // Generation 1 with other rows is stale only if "start" is still the
    // applied mapping, so this proves it stayed in force.
    pair.publish(1, "other");
    assert_eq!(reload_now(&mut source).await, [id("start")]);
}

/// Both reads started against generation 1. The first applies generation 2;
/// the second sees a restarted writer's generation 1 with other rows, which is
/// stale against "start" but a valid change against what is applied by then.
#[tokio::test]
async fn a_writer_restart_during_an_overlap_is_judged_against_the_applied_mapping() {
    let pair = Pair::new();
    pair.publish(1, "start");
    let mut source = pair.started_source().await;
    let first = source.begin_reload().unwrap();
    let second = source.begin_reload().unwrap();
    pair.publish(2, "first");
    let (first_order, first_read) = first.run().await;
    pair.publish(1, "restarted");
    let (second_order, second_read) = second.run().await;

    let applied = source.adopt_reload(first_order, first_read).unwrap();
    assert_eq!(link_ids(&applied), [id("first")]);
    let restarted = source.adopt_reload(second_order, second_read).unwrap();
    assert_eq!(link_ids(&restarted), [id("restarted")]);
    assert_eq!(
        source.report().disposition.state,
        BindMapDisposition::Mapped.as_str()
    );
}
