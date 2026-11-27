//! Where the sender's uplink set comes from, across startup and every reload.
//!
//! Two sources, and only one of them is new. Without `--bind-map` the sender
//! reads `BIND_IPS_FILE` exactly as it always has and every uplink is unmapped —
//! no hashing, no sidecar, no new failure mode. With `--bind-map` the same IP
//! file is read through the ADR-003 pair protocol, and the resolution decides
//! whether the mapping is in force, whether the last valid one is retained, or
//! whether the sender falls open to legacy behavior with the ambiguity named.
//!
//! This type owns the one piece of state that decision needs: the last mapping
//! actually applied. That is what distinguishes a startup degradation (nothing
//! to retain, so collisions are excluded and reported) from a reload degradation
//! (a live bond is already interface-pinned, so it is kept running).

use anyhow::Result;
use smallvec::SmallVec;
use tracing::{info, warn};

use super::connections::{specs_from_effective_links, specs_from_ips};
use crate::bind_map::{
    BindMapDisposition, BindMapError, BindMapPaths, BindMapReport, BindMapStatus, MappedPool,
    PairRead, ResolvePhase, SystemIfaces, ValidateCtx, read_pair, resolve,
};
use crate::net::UplinkSpec;
use crate::sender::read_weighted_ip_list;

/// The two file paths the sender's link set is derived from.
#[derive(Debug, Clone, Copy)]
pub struct SenderPaths<'a> {
    pub ips_file: &'a str,
    /// `None` means legacy: the bind-map module is never entered.
    pub bind_map: Option<&'a str>,
}

/// The outcome of one bind-map pair read: the IP file that was hashed, plus the
/// mapping derived from exactly those bytes.
///
/// Unix-only because its sole consumer is the `SIGHUP` reload channel, and
/// `SIGHUP` does not exist on Windows.
#[cfg(unix)]
pub type PairReadResult = Result<PairRead, BindMapError>;

/// Read the `(ips_file, sidecar)` pair off disk.
///
/// Free-standing and owning its arguments so a `SIGHUP` can run it on a spawned
/// task: the read is bounded but not instant (a hash mismatch is retried for up
/// to 2 s), and stalling the packet-forwarding loop for that long would be a
/// worse regression than a slightly later reload.
///
/// The file reads themselves are synchronous and the sender runs on one runtime
/// thread, so the pair is read on the blocking pool: a read held up by slow
/// storage would otherwise stop forwarding with it. The retry sleep still runs
/// on the runtime's timer, which the thread running the runtime keeps driving.
pub async fn read_bind_map(
    ips_file: String,
    sidecar: String,
    prior: Option<MappedPool>,
) -> Result<PairRead, BindMapError> {
    let runtime = tokio::runtime::Handle::current();
    tokio::task::spawn_blocking(move || {
        let oracle = SystemIfaces;
        runtime.block_on(read_pair(
            BindMapPaths {
                ips_file: std::path::Path::new(&ips_file),
                sidecar: std::path::Path::new(&sidecar),
            },
            ValidateCtx {
                ifaces: &oracle,
                prior: prior.as_ref(),
            },
        ))
    })
    .await
    .unwrap_or_else(|e| std::panic::resume_unwind(e.into_panic()))
}

/// One `SIGHUP` reload read for a spawned task, numbered in the order the reads
/// were started.
pub struct ReloadRead {
    order: u64,
    ips_file: String,
    sidecar: String,
    prior: Option<MappedPool>,
}

impl ReloadRead {
    /// Run the read, answering with its order number.
    pub async fn run(self) -> (u64, Result<PairRead, BindMapError>) {
        (
            self.order,
            read_bind_map(self.ips_file, self.sidecar, self.prior).await,
        )
    }
}

pub struct LinkSource {
    ips_file: String,
    sidecar: Option<String>,
    last_valid: Option<MappedPool>,
    report: BindMapReport,
    /// Reload reads can finish out of order, so each is numbered and an older
    /// one is never adopted after a newer one.
    reads_started: u64,
    newest_adopted: u64,
}

impl LinkSource {
    #[must_use]
    pub fn new(paths: &SenderPaths<'_>) -> Self {
        Self {
            ips_file: paths.ips_file.to_string(),
            sidecar: paths.bind_map.map(ToString::to_string),
            last_valid: None,
            // A legacy run never resolves, so the default is what it reports for
            // its whole lifetime: absent, running legacy, nothing excluded.
            report: BindMapReport::default(),
            reads_started: 0,
            newest_adopted: 0,
        }
    }

    #[must_use]
    pub fn is_mapped(&self) -> bool {
        self.sidecar.is_some()
    }

    /// The last resolution's telemetry projection (ADR-003 §6.4).
    #[must_use]
    pub fn report(&self) -> &BindMapReport {
        &self.report
    }

    /// What a degradation leaves running: a live bond keeps its last valid
    /// mapped pool, a startup that never had one falls open to legacy.
    fn retention(&self) -> BindMapDisposition {
        if self.last_valid.is_some() {
            BindMapDisposition::RetainedLastValid
        } else {
            BindMapDisposition::LegacyUniqueOnly
        }
    }

    /// The arguments a pair read needs.
    #[must_use]
    pub fn read_args(&self) -> Option<(String, String, Option<MappedPool>)> {
        let sidecar = self.sidecar.clone()?;
        Some((self.ips_file.clone(), sidecar, self.last_valid.clone()))
    }

    /// Start a `SIGHUP` reload read, numbered after every read started before it.
    #[must_use]
    pub fn begin_reload(&mut self) -> Option<ReloadRead> {
        let (ips_file, sidecar, prior) = self.read_args()?;
        self.reads_started += 1;
        Some(ReloadRead {
            order: self.reads_started,
            ips_file,
            sidecar,
            prior,
        })
    }

    /// Adopt a finished reload read, or `None` when a newer one was already
    /// adopted: applying it then would put an older uplink set back.
    pub fn adopt_reload(
        &mut self,
        order: u64,
        read: Result<PairRead, BindMapError>,
    ) -> Option<SmallVec<UplinkSpec, 4>> {
        if order <= self.newest_adopted {
            return None;
        }
        self.newest_adopted = order;
        Some(self.adopt(read))
    }

    /// The uplink set to start with.
    ///
    /// The `Err` arm is reserved for an unreadable `BIND_IPS_FILE`, matching the
    /// legacy contract: an empty start is not fatal, it waits for a `SIGHUP`.
    pub async fn startup(&mut self) -> Result<SmallVec<UplinkSpec, 4>> {
        let Some((ips_file, sidecar, prior)) = self.read_args() else {
            let (ips, weights) = read_weighted_ip_list(&self.ips_file).await?;
            return Ok(specs_from_ips(&ips, &weights));
        };
        let read = read_bind_map(ips_file, sidecar, prior).await;
        Ok(self.adopt(read))
    }

    /// Turn a completed pair read into the uplink set, remembering the mapping
    /// when it was valid.
    pub fn adopt(&mut self, read: Result<PairRead, BindMapError>) -> SmallVec<UplinkSpec, 4> {
        let pair = match read {
            Ok(pair) => pair,
            Err(err) => {
                warn!("bind-map read failed: {err}");
                // The outer Err means BIND_IPS_FILE itself was unusable, so no
                // resolution ran. Report the degradation anyway — leaving the
                // previous status standing would tell a UI the map is still
                // active while the sender is running on nothing new.
                self.report = BindMapReport::degraded(err.reason(), self.retention());
                return SmallVec::new();
            }
        };
        let phase = match self.last_valid.as_ref() {
            Some(last_valid) => ResolvePhase::Reload { last_valid },
            None => ResolvePhase::Startup,
        };
        let resolution = resolve(pair.map, &pair.ips, phase);
        self.report = BindMapReport::from(&resolution);
        log_outcome(resolution.status, resolution.disposition);
        for group in &resolution.excluded {
            warn!(
                "bind-map unusable: source IP {} is claimed by {} uplink(s); running line {} and \
                 excluding line(s) {:?} — the excluded modem(s) will carry no traffic until a \
                 coherent bind-map is published",
                group.ip,
                group.excluded_indices.len() + 1,
                group.effective_index,
                group.excluded_indices
            );
        }
        let specs = specs_from_effective_links(&resolution.links);
        if let Some(applied) = resolution.applied {
            self.last_valid = Some(applied);
        }
        specs
    }
}

fn log_outcome(status: BindMapStatus, disposition: BindMapDisposition) {
    let disposition = disposition.as_str();
    match status {
        BindMapStatus::Active => info!("bind-map active ({disposition})"),
        BindMapStatus::Absent => info!("bind-map absent ({disposition})"),
        BindMapStatus::Degraded(reason) => {
            warn!("bind-map degraded: {reason} ({disposition})");
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::fs::OpenOptions;
    use std::os::unix::ffi::OsStrExt as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    use std::sync::mpsc;
    use std::time::Duration;

    use super::{LinkSource, SenderPaths, read_bind_map};
    use crate::bind_map::BindMapError;

    const BOUND: Duration = Duration::from_secs(5);

    fn mapped_source() -> LinkSource {
        LinkSource::new(&SenderPaths {
            ips_file: "ips.txt",
            bind_map: Some("bind-map.json"),
        })
    }

    fn missing_map() -> BindMapError {
        BindMapError::MissingFile {
            path: "bind-map.json".into(),
        }
    }

    fn malformed_map() -> BindMapError {
        BindMapError::MalformedJson("truncated".into())
    }

    /// Two `SIGHUP`s whose reads finish newest first: the older answer must not
    /// put its result back over the newer one.
    #[test]
    fn a_reload_read_that_finishes_after_a_newer_one_is_not_adopted() {
        let mut source = mapped_source();
        let older = source.begin_reload().unwrap();
        let newer = source.begin_reload().unwrap();

        assert!(
            source
                .adopt_reload(newer.order, Err(missing_map()))
                .is_some()
        );
        let adopted = source.report().clone();
        assert!(
            source
                .adopt_reload(older.order, Err(malformed_map()))
                .is_none(),
            "a read older than the adopted one must not be applied"
        );
        assert_eq!(source.report(), &adopted);
    }

    #[test]
    fn reload_reads_that_finish_in_order_are_each_adopted() {
        let mut source = mapped_source();
        let older = source.begin_reload().unwrap();
        let newer = source.begin_reload().unwrap();

        assert!(
            source
                .adopt_reload(older.order, Err(malformed_map()))
                .is_some()
        );
        let after_older = source.report().clone();
        assert!(
            source
                .adopt_reload(newer.order, Err(missing_map()))
                .is_some()
        );
        assert_ne!(source.report(), &after_older);
    }

    /// The IP file is a FIFO. The read opens it and then waits for data that only
    /// comes when the writer closes its end, and the sender's one runtime thread
    /// has to keep running while that read waits.
    #[test]
    fn a_pending_pair_read_leaves_the_runtime_thread_free() {
        let dir = tempfile::tempdir().unwrap();
        let ips = dir.path().join("ips.txt");
        let fifo = std::ffi::CString::new(ips.as_os_str().as_bytes()).unwrap();
        // SAFETY: a valid NUL-terminated path and a plain permission mode.
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);

        let (read_opened_tx, read_opened_rx) = tokio::sync::oneshot::channel::<()>();
        let (release_tx, release_rx) = mpsc::channel::<()>();
        let writer_path = ips.clone();
        // A blocking open of the write end returns only once the read has opened
        // the FIFO, so it marks the read as started. Holding the write end keeps
        // the read waiting until the test releases it or the bound runs out.
        let writer = std::thread::spawn(move || {
            let write_end = OpenOptions::new().write(true).open(&writer_path).unwrap();
            let _ = read_opened_tx.send(());
            let _ = release_rx.recv_timeout(BOUND);
            drop(write_end);
        });

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (read_started, read_was_pending) = runtime.block_on(async {
            let read = tokio::spawn(read_bind_map(
                ips.display().to_string(),
                dir.path().join("bind-map.json").display().to_string(),
                None,
            ));
            let started = tokio::time::timeout(BOUND, read_opened_rx).await.is_ok();
            let pending = started && !read.is_finished();
            let _ = release_tx.send(());
            let _ = tokio::time::timeout(BOUND, read).await;
            (started, pending)
        });
        runtime.shutdown_timeout(Duration::from_secs(1));
        // Satisfies the writer's open if the read never got that far. Held until
        // the writer is joined, so a writer that reaches its open late still
        // finds a reader instead of blocking forever.
        let fallback_reader = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&ips)
            .unwrap();
        writer.join().unwrap();
        drop(fallback_reader);

        assert!(
            read_started,
            "the bind-map pair read never opened the IP file"
        );
        assert!(
            read_was_pending,
            "the bind-map pair read held the runtime thread while it waited"
        );
    }
}
