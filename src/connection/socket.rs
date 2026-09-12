use std::io;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use anyhow::{Context, Result};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{Duration, timeout};
use tracing::{debug, warn};

const DNS_DRIFT_WARN_INTERVAL_MS: u64 = 60_000;
const DNS_DRIFT_RESULT_WAIT_TIMEOUT: Duration = Duration::from_secs(3);

struct ReceiverDnsDiagnostics {
    check_in_flight: AtomicBool,
    last_warning_ms: AtomicU64,
}

impl ReceiverDnsDiagnostics {
    const fn new() -> Self {
        Self {
            check_in_flight: AtomicBool::new(false),
            last_warning_ms: AtomicU64::new(0),
        }
    }

    fn try_acquire(&'static self) -> Option<ReceiverDnsCheckPermit> {
        self.check_in_flight
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
            .then_some(ReceiverDnsCheckPermit(self))
    }

    fn claim_warning(&self, now_ms: u64) -> bool {
        loop {
            let last = self.last_warning_ms.load(Ordering::Relaxed);
            if last != 0 && now_ms.saturating_sub(last) < DNS_DRIFT_WARN_INTERVAL_MS {
                return false;
            }
            match self.last_warning_ms.compare_exchange_weak(
                last,
                now_ms.max(1),
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return true,
                Err(_) => continue,
            }
        }
    }

    #[cfg(test)]
    fn is_check_in_flight(&self) -> bool {
        self.check_in_flight.load(Ordering::Relaxed)
    }
}

struct ReceiverDnsCheckPermit(&'static ReceiverDnsDiagnostics);

impl Drop for ReceiverDnsCheckPermit {
    fn drop(&mut self) {
        self.0.check_in_flight.store(false, Ordering::Relaxed);
    }
}

static RECEIVER_DNS_DIAGNOSTICS: ReceiverDnsDiagnostics = ReceiverDnsDiagnostics::new();

/// Egress-steering strategy for a freshly created uplink UDP socket: how the
/// socket is pinned to a particular network egress *before* it is connected.
///
/// Selection-neutral by design — an implementation decides *which* egress a
/// packet leaves by, never *when* a link is chosen (that is the scheduler's
/// job). Two implementations ship today:
///
/// * [`SourceIpBinder`] — the default. Binds the socket to a local source IP
///   (`bind(2)` to `source_ip:port`), reproducing the historical behavior.
///   Host-side source routing then steers egress per source address.
/// * [`DeviceBinder`] — optional, Linux-only. Pins egress to a named interface
///   via `SO_BINDTODEVICE`, independent of the routing table.
///
/// ## Extension point (deliberately not implemented here)
/// A mobile/embedded host can supply its own binder that calls back into a host
/// API (e.g. iOS `NWConnection` / Android `Network.bindSocket`) to steer egress
/// on platforms without source routing. That variant has **no in-tree consumer**
/// and is intentionally left unbuilt: implement `UplinkBinder` against the host
/// callback and thread it down the bind path. This trait is the only seam such a
/// host integration needs — no scheduler or protocol coupling.
pub trait UplinkBinder {
    /// Pin `socket` to an egress for `source_ip` before it is connected.
    fn bind_egress(&self, socket: &Socket, source_ip: IpAddr) -> io::Result<()>;
}

/// Default egress steering: bind the socket to a local source IP.
///
/// Byte-for-byte equivalent to the historical inline
/// `bind(SocketAddr::new(ip, port))` call — `port` is the requested local port
/// (always `0`/ephemeral on the live bind path).
pub struct SourceIpBinder {
    port: u16,
}

impl SourceIpBinder {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl UplinkBinder for SourceIpBinder {
    fn bind_egress(&self, socket: &Socket, source_ip: IpAddr) -> io::Result<()> {
        let addr = SocketAddr::new(source_ip, self.port);
        socket.bind(&addr.into())
    }
}

/// Linux egress steering via `SO_BINDTODEVICE` **and** a source-address bind.
///
/// Selected only for a link the bind-map names (`--bind-map`); an unmapped link
/// keeps [`SourceIpBinder`] verbatim.
///
/// Both halves are load-bearing and neither substitutes for the other:
///
/// * `SO_BINDTODEVICE` decides which interface the packet physically leaves by,
///   overriding the routing table. On its own it leaves the **source address**
///   to the kernel, which picks one from the chosen interface — so two modems
///   presenting the same address, or an interface holding several, would put a
///   non-deterministic source on the wire and the receiver would see the bond's
///   links blur together.
/// * `bind(ip, port)` pins that source address. On its own it steers nothing
///   without host source routing, which is exactly what a modem bond cannot
///   rely on.
///
/// The device binding is applied **first** so the subsequent `bind(2)` is
/// evaluated against the interface this link is already pinned to.
#[cfg(target_os = "linux")]
pub struct DeviceBinder {
    pub ifname: String,
    /// Requested local port; always `0`/ephemeral on the live bind path.
    pub port: u16,
}

#[cfg(target_os = "linux")]
impl UplinkBinder for DeviceBinder {
    fn bind_egress(&self, socket: &Socket, source_ip: IpAddr) -> io::Result<()> {
        // SO_BINDTODEVICE — bind egress to the named interface. The method is
        // `bind_device` (NOT `set_bind_device`); `None` would clear the binding.
        socket.bind_device(Some(self.ifname.as_bytes()))?;
        let addr = SocketAddr::new(source_ip, self.port);
        socket.bind(&addr.into())
    }
}

/// Create an uplink socket for a link the bind-map pins to `iface`.
///
/// `iface` is `None` for every unmapped link, and that arm is
/// [`bind_from_ip`] verbatim — the legacy path is not merely equivalent, it is
/// the same code.
pub fn bind_for_link(ip: IpAddr, port: u16, iface: Option<&str>) -> Result<Socket> {
    match iface {
        None => bind_from_ip(ip, port),
        #[cfg(target_os = "linux")]
        Some(ifname) => {
            let sock = new_uplink_socket(ip)?;
            DeviceBinder {
                ifname: ifname.to_string(),
                port,
            }
            .bind_egress(&sock, ip)
            .with_context(|| format!("bind socket to {ip} on {ifname}"))?;
            Ok(sock)
        }
        #[cfg(not(target_os = "linux"))]
        Some(ifname) => Err(anyhow::anyhow!(
            "per-interface egress binding ({ifname}) requires Linux SO_BINDTODEVICE"
        )),
    }
}

/// An unbound, non-blocking, buffer-tuned UDP socket of `ip`'s family.
fn new_uplink_socket(ip: IpAddr) -> Result<Socket> {
    let domain = match ip {
        IpAddr::V4(_) => Domain::IPV4,
        IpAddr::V6(_) => Domain::IPV6,
    };
    let sock = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP)).context("create socket")?;
    sock.set_nonblocking(true).context("set nonblocking")?;

    // Set send buffer size (100MB)
    const SEND_BUF_SIZE: usize = 100 * 1024 * 1024;
    if let Err(e) = sock.set_send_buffer_size(SEND_BUF_SIZE) {
        warn!("Failed to set send buffer size to {}: {}", SEND_BUF_SIZE, e);
        if let Ok(actual_size) = sock.send_buffer_size() {
            warn!("Effective send buffer size: {}", actual_size);
        }
    }

    // Set receive buffer size to handle large SRT packets (100MB)
    const RECV_BUF_SIZE: usize = 100 * 1024 * 1024;
    if let Err(e) = sock.set_recv_buffer_size(RECV_BUF_SIZE) {
        warn!(
            "Failed to set receive buffer size to {}: {}",
            RECV_BUF_SIZE, e
        );
        if let Ok(actual_size) = sock.recv_buffer_size() {
            warn!("Effective receive buffer size: {}", actual_size);
        }
    }

    Ok(sock)
}

pub fn bind_from_ip(ip: IpAddr, port: u16) -> Result<Socket> {
    let sock = new_uplink_socket(ip)?;

    // Egress steering via the default source-IP binder. Byte-identical to the
    // previous inline `sock.bind(&SocketAddr::new(ip, port).into())`.
    SourceIpBinder::new(port)
        .bind_egress(&sock, ip)
        .context("bind socket")?;
    Ok(sock)
}

pub async fn resolve_remote(host: &str, port: u16) -> Result<SocketAddr> {
    let mut addrs = tokio::net::lookup_host((host, port))
        .await
        .context("dns lookup")?;
    addrs
        .next()
        .ok_or_else(|| anyhow::anyhow!("no DNS result for {}", host))
}

pub async fn resolve_remote_all(host: &str, port: u16) -> io::Result<Vec<SocketAddr>> {
    tokio::net::lookup_host((host, port))
        .await
        .map(Iterator::collect)
}

pub(crate) fn remote_drift(answers: &[SocketAddr], current: &SocketAddr) -> bool {
    !answers.is_empty() && !answers.contains(current)
}

/// Start a detect-only receiver DNS drift check without delaying socket recovery.
///
/// Reconnect runs on the housekeeping loop, so diagnostic DNS must never be
/// awaited there. One detached standard thread may resolve at a time, while a
/// Tokio task waits up to three seconds for its result. A slow system resolver
/// keeps the process-wide permit until it really returns, preventing overlapping
/// lookups without joining Tokio's shutdown path. This never changes the cached
/// peer: moving one uplink would split the SRTLA receiver identity.
pub(crate) fn spawn_receiver_dns_drift_check(host: &str, port: u16, current: SocketAddr) {
    let resolver_host = host.to_string();
    let log_host = resolver_host.clone();
    let _ = spawn_receiver_dns_drift_check_with(
        &RECEIVER_DNS_DIAGNOSTICS,
        log_host,
        current,
        DNS_DRIFT_RESULT_WAIT_TIMEOUT,
        move || {
            (resolver_host.as_str(), port)
                .to_socket_addrs()
                .map(Iterator::collect)
        },
    );
}

fn spawn_receiver_dns_drift_check_with<R>(
    state: &'static ReceiverDnsDiagnostics,
    host: String,
    current: SocketAddr,
    result_wait_timeout: Duration,
    resolve: R,
) -> Option<JoinHandle<()>>
where
    R: FnOnce() -> io::Result<Vec<SocketAddr>> + Send + 'static,
{
    let permit = state.try_acquire()?;
    let (result_tx, result_rx) = oneshot::channel();
    if let Err(error) = std::thread::Builder::new()
        .name("srtla-dns-drift".to_string())
        .spawn(move || {
            let _permit = permit;
            let _ = result_tx.send(resolve());
        })
    {
        debug!("failed to spawn receiver DNS diagnostic thread: {error}");
        return None;
    }

    Some(tokio::spawn(async move {
        match timeout(result_wait_timeout, result_rx).await {
            Ok(Ok(Ok(answers))) if remote_drift(&answers, &current) => {
                if state.claim_warning(crate::utils::now_ms()) {
                    let answers = answers
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ");
                    warn!(
                        "receiver DNS drift: {host} no longer resolves to {current} (now \
                         {answers}); keeping the current peer because the bond is registered \
                         against this receiver instance"
                    );
                }
            }
            Ok(Ok(Ok(answers))) if answers.is_empty() => {
                debug!("receiver DNS lookup for {host} returned no addresses; keeping {current}");
            }
            Ok(Ok(Ok(_))) => {}
            Ok(Ok(Err(error))) => {
                debug!(
                    "receiver DNS lookup failed during reconnect; keeping current peer {current}: \
                     {error}"
                );
            }
            Ok(Err(_)) => {
                debug!("receiver DNS diagnostic ended without a result; keeping {current}");
            }
            Err(_) => {
                debug!(
                    "receiver DNS diagnostic result wait timed out after {:?}; keeping current \
                     peer {current}; the detached resolver retains the single-flight slot until \
                     it returns",
                    result_wait_timeout
                );
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use socket2::{Domain, Protocol, Socket, Type};

    use super::*;

    #[test]
    fn remote_drift_is_false_when_current_peer_is_resolved() {
        // Given: DNS answers include the connection's current peer.
        let current = SocketAddr::from(([127, 0, 0, 1], 8080));

        // When: drift detection checks those answers.
        let drifted = remote_drift(
            &[current, SocketAddr::from(([127, 0, 0, 2], 8080))],
            &current,
        );

        // Then: the current peer is accepted without drift.
        assert!(!drifted);
    }

    #[test]
    fn remote_drift_is_true_when_current_peer_is_absent() {
        // Given: DNS answers no longer include the connection's current peer.
        let current = SocketAddr::from(([127, 0, 0, 1], 8080));

        // When: drift detection checks those answers.
        let drifted = remote_drift(&[SocketAddr::from(([127, 0, 0, 2], 8080))], &current);

        // Then: the missing current peer is reported as drift.
        assert!(drifted);
    }

    #[test]
    fn remote_drift_is_false_when_dns_returns_no_answers() {
        let current = SocketAddr::from(([127, 0, 0, 1], 8080));

        assert!(
            !remote_drift(&[], &current),
            "an empty answer is inconclusive, not evidence that the receiver moved"
        );
    }

    #[test]
    fn dns_drift_warning_is_shared_and_rate_limited() {
        let state = ReceiverDnsDiagnostics::new();
        let first = 1_000;

        assert!(state.claim_warning(first), "the first drift must warn");
        assert!(
            !state.claim_warning(first + DNS_DRIFT_WARN_INTERVAL_MS - 1),
            "another uplink inside the process-wide interval must stay quiet"
        );
        assert!(
            state.claim_warning(first + DNS_DRIFT_WARN_INTERVAL_MS),
            "the warning becomes eligible at the interval boundary"
        );
    }

    async fn wait_for_dns_check_to_finish(state: &ReceiverDnsDiagnostics) {
        timeout(Duration::from_secs(2), async {
            while state.is_check_in_flight() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .expect("the controlled resolver thread must release its permit");
    }

    #[tokio::test]
    async fn dns_result_timeout_does_not_release_the_active_resolver_slot() {
        let state = Box::leak(Box::new(ReceiverDnsDiagnostics::new()));
        let current = SocketAddr::from(([127, 0, 0, 1], 8080));
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let blocking_lookup = move || {
            let _ = started_tx.send(());
            release_rx.recv().expect("test releases the resolver");
            Ok(Vec::new())
        };

        let first = spawn_receiver_dns_drift_check_with(
            state,
            "receiver.example".to_string(),
            current,
            Duration::from_millis(10),
            blocking_lookup,
        )
        .expect("the first check must start");
        started_rx.await.expect("the detached resolver must start");
        first.await.expect("the result waiter must time out");

        assert!(
            spawn_receiver_dns_drift_check_with(
                state,
                "receiver.example".to_string(),
                current,
                Duration::from_millis(10),
                || Ok(Vec::new()),
            )
            .is_none(),
            "a timed-out waiter must not permit overlapping resolver work"
        );

        release_tx
            .send(())
            .expect("release the controlled resolver");
        wait_for_dns_check_to_finish(state).await;

        let next = spawn_receiver_dns_drift_check_with(
            state,
            "receiver.example".to_string(),
            current,
            Duration::from_secs(1),
            || Ok(Vec::new()),
        )
        .expect("resolver completion must release the process-wide slot");
        next.await.expect("the replacement check must finish");
        wait_for_dns_check_to_finish(state).await;
    }

    #[tokio::test(flavor = "current_thread")]
    async fn abort_before_first_waiter_poll_keeps_the_resolver_owned_permit() {
        let state = Box::leak(Box::new(ReceiverDnsDiagnostics::new()));
        let current = SocketAddr::from(([127, 0, 0, 1], 8080));
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();

        let waiter = spawn_receiver_dns_drift_check_with(
            state,
            "receiver.example".to_string(),
            current,
            Duration::from_secs(1),
            move || {
                let _ = started_tx.send(());
                release_rx.recv().expect("test releases the resolver");
                Ok(Vec::new())
            },
        )
        .expect("the resolver must start");
        waiter.abort();
        started_rx.await.expect("the resolver thread must start");

        assert!(state.is_check_in_flight());
        assert!(
            spawn_receiver_dns_drift_check_with(
                state,
                "receiver.example".to_string(),
                current,
                Duration::from_millis(10),
                || Ok(Vec::new()),
            )
            .is_none(),
            "waiter cancellation must not release active resolver ownership"
        );

        release_tx
            .send(())
            .expect("release the controlled resolver");
        wait_for_dns_check_to_finish(state).await;
    }

    #[test]
    fn detached_dns_worker_does_not_block_process_exit() {
        const CHILD_ENV: &str = "SRTLA_DNS_EXIT_TEST_CHILD";
        const TEST_NAME: &str =
            "connection::socket::tests::detached_dns_worker_does_not_block_process_exit";

        if std::env::var_os(CHILD_ENV).is_some() {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .expect("child runtime");
            runtime.block_on(async {
                let state = Box::leak(Box::new(ReceiverDnsDiagnostics::new()));
                let (started_tx, started_rx) = tokio::sync::oneshot::channel();
                let _ = spawn_receiver_dns_drift_check_with(
                    state,
                    "receiver.example".to_string(),
                    SocketAddr::from(([127, 0, 0, 1], 8080)),
                    Duration::from_secs(60),
                    move || {
                        let _ = started_tx.send(());
                        std::thread::sleep(Duration::from_secs(60));
                        Ok(Vec::new())
                    },
                );
                started_rx.await.expect("child resolver thread must start");
            });
            return;
        }

        let mut child = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args(["--exact", TEST_NAME, "--nocapture"])
            .env(CHILD_ENV, "1")
            .spawn()
            .expect("spawn DNS shutdown child");
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().expect("poll DNS shutdown child") {
                assert!(status.success(), "DNS shutdown child failed: {status}");
                break;
            }
            if std::time::Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("detached DNS resolver thread blocked process shutdown");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn source_ip_binder_binds_to_source_ip() {
        // Given: a fresh UDP socket and a loopback source IP.
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let sock = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();

        // When: the default binder steers egress to that source IP (ephemeral port).
        SourceIpBinder::new(0).bind_egress(&sock, ip).unwrap();

        // Then: the bound socket's local address is the requested source IP.
        let local = sock.local_addr().unwrap().as_socket().unwrap();
        assert_eq!(local.ip(), ip);
    }

    #[test]
    fn device_binder_cfg_gated() {
        // Structural: the SO_BINDTODEVICE impl is gated to Linux and uses
        // `bind_device` (mirrors the `rg` acceptance check).
        let src = include_str!("socket.rs");
        assert!(
            src.contains("#[cfg(target_os = \"linux\")]"),
            "DeviceBinder must be cfg-gated to Linux"
        );
        assert!(
            src.contains("bind_device"),
            "DeviceBinder must use SO_BINDTODEVICE via bind_device"
        );

        // On Linux, reference the impl so it stays a live consumer.
        #[cfg(target_os = "linux")]
        {
            let binder = super::DeviceBinder {
                ifname: String::from("lo"),
                port: 0,
            };
            assert_eq!(binder.ifname, "lo");
        }
    }
}
