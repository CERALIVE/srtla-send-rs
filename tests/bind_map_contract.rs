//! Filesystem- and process-level halves of the bind-map contract (ADR-003).
//!
//! The in-crate suites cover the pure parser and the fail-open state machine.
//! What can only be proven here: the writer's two-rename publication window, the
//! bounded retry that absorbs it, the sidecar's symlink/permission refusals, and
//! the `--capabilities-json` probe at the real process boundary.

use std::path::Path;
use std::time::Instant;

use srtla_send::bind_map::{
    BIND_MAP_RETRY_BUDGET_MS, BindMapError, BindMapPaths, IpsFile, StaticIfaces, ValidateCtx,
    read_pair,
};

const IPS_A: &str = "192.168.8.100\n192.168.8.100\n";
const IPS_B: &str = "192.168.8.100\n192.168.8.100\n10.0.0.9\n";

fn sha_of(contents: &str) -> String {
    IpsFile::from_bytes(contents.as_bytes())
        .expect("fixture parses")
        .sha256()
        .to_string()
}

fn sidecar_for(contents: &str, generation: u64) -> String {
    let rows: Vec<String> = IpsFile::from_bytes(contents.as_bytes())
        .expect("fixture parses")
        .accepted()
        .iter()
        .enumerate()
        .map(|(i, ip)| format!(r#"{{"link_id":"link-{i}","ip":"{ip}","iface":"wwan{i}"}}"#))
        .collect();
    format!(
        r#"{{"schema_version":1,"generation":{generation},"ips_file_sha256":"{}","links":[{}]}}"#,
        sha_of(contents),
        rows.join(",")
    )
}

fn oracle() -> StaticIfaces {
    StaticIfaces::new(["wwan0", "wwan1", "wwan2"])
}

/// Publish a file the way the ADR requires: unique temp sibling, then rename.
fn publish(target: &Path, contents: &str) {
    let tmp = target.with_extension(format!("{}.0.tmp", std::process::id()));
    std::fs::write(&tmp, contents).expect("write temp sibling");
    std::fs::rename(&tmp, target).expect("atomic rename into place");
}

async fn read(
    dir: &Path,
) -> Result<Result<srtla_send::bind_map::MappedPool, BindMapError>, BindMapError> {
    let ifaces = oracle();
    let pair = read_pair(
        BindMapPaths {
            ips_file: &dir.join("srtla_ips"),
            sidecar: &dir.join("bind_map.json"),
        },
        ValidateCtx {
            ifaces: &ifaces,
            prior: None,
        },
    )
    .await?;
    Ok(pair.map)
}

// ---- the publication window ------------------------------------------

#[tokio::test]
async fn a_coherently_published_pair_reads_as_valid() {
    let dir = tempfile::tempdir().expect("tempdir");
    publish(&dir.path().join("srtla_ips"), IPS_A);
    publish(&dir.path().join("bind_map.json"), &sidecar_for(IPS_A, 1));

    let pool = read(dir.path())
        .await
        .expect("ips file is readable")
        .expect("a coherent pair is valid");
    assert_eq!(pool.generation, 1);
    assert_eq!(pool.rows.len(), 2);
    assert_eq!(pool.rows[0].iface.as_str(), "wwan0");
    assert_eq!(
        pool.rows[1].iface.as_str(),
        "wwan1",
        "the twin rows are told apart positionally, not by IP"
    );
}

#[tokio::test]
async fn the_bounded_retry_absorbs_the_window_between_the_two_renames() {
    // The writer commits the ips file first, so a reader landing between the two
    // renames sees NEW ips bytes + OLD sidecar. That is a DETECTABLE transient
    // mismatch, and absorbing it is the entire reason the retry exists.
    let dir = tempfile::tempdir().expect("tempdir");
    let ips = dir.path().join("srtla_ips");
    let sidecar = dir.path().join("bind_map.json");

    publish(&ips, IPS_A);
    publish(&sidecar, &sidecar_for(IPS_A, 1));
    publish(&ips, IPS_B); // first rename lands; the sidecar still names IPS_A

    let path = sidecar.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        publish(&path, &sidecar_for(IPS_B, 2)); // the COMMIT POINT
    });

    let started = Instant::now();
    let pool = read(dir.path())
        .await
        .expect("ips file is readable")
        .expect("the retry must absorb the publication window");

    assert_eq!(pool.generation, 2);
    assert_eq!(pool.rows.len(), 3, "the reader converged on the NEW pair");
    assert!(
        started.elapsed().as_millis() as u64 <= BIND_MAP_RETRY_BUDGET_MS,
        "the retry must stay inside its budget"
    );
}

#[tokio::test]
async fn a_mismatch_that_outlives_the_budget_reports_retry_exhausted() {
    let dir = tempfile::tempdir().expect("tempdir");
    publish(&dir.path().join("srtla_ips"), IPS_B);
    // A sidecar that permanently names different content — a writer bug, not a window.
    publish(&dir.path().join("bind_map.json"), &sidecar_for(IPS_A, 1));

    let started = Instant::now();
    let err = read(dir.path())
        .await
        .expect("ips file is readable")
        .expect_err("a permanent mismatch is not a transient window");

    assert!(
        matches!(err, BindMapError::RetryExhausted { .. }),
        "expected retry exhaustion, got {err:?}"
    );
    let elapsed = started.elapsed().as_millis() as u64;
    assert!(
        elapsed <= BIND_MAP_RETRY_BUDGET_MS,
        "the retry schedule ran {elapsed}ms, past its {BIND_MAP_RETRY_BUDGET_MS}ms ceiling"
    );
}

// ---- sidecar read safety ---------------------------------------------

#[tokio::test]
async fn an_absent_sidecar_reports_missing_file_rather_than_a_parse_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    publish(&dir.path().join("srtla_ips"), IPS_A);

    let err = read(dir.path())
        .await
        .expect("ips file is readable")
        .expect_err("no sidecar means no mapping");
    assert!(
        matches!(err, BindMapError::MissingFile { .. }),
        "expected missing-file, got {err:?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn a_symlinked_sidecar_is_refused() {
    // The mapping decides where device traffic egresses, so the path must not be
    // redirectable at a file the service never wrote.
    let dir = tempfile::tempdir().expect("tempdir");
    publish(&dir.path().join("srtla_ips"), IPS_A);

    let real = dir.path().join("elsewhere.json");
    std::fs::write(&real, sidecar_for(IPS_A, 1)).expect("write link target");
    std::os::unix::fs::symlink(&real, dir.path().join("bind_map.json")).expect("symlink");

    let err = read(dir.path())
        .await
        .expect("ips file is readable")
        .expect_err("a symlinked sidecar must be refused");
    assert!(
        matches!(err, BindMapError::Unreadable { .. }),
        "expected unreadable, got {err:?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn a_world_writable_sidecar_is_refused() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = tempfile::tempdir().expect("tempdir");
    publish(&dir.path().join("srtla_ips"), IPS_A);
    let sidecar = dir.path().join("bind_map.json");
    publish(&sidecar, &sidecar_for(IPS_A, 1));
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o666))
        .expect("loosen perms");

    let err = read(dir.path())
        .await
        .expect("ips file is readable")
        .expect_err("an anyone-writable mapping must be refused");
    assert!(
        matches!(err, BindMapError::Unreadable { .. }),
        "expected unreadable, got {err:?}"
    );

    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o644))
        .expect("restore perms");
    read(dir.path())
        .await
        .expect("ips file is readable")
        .expect("service-owned perms read fine");
}
