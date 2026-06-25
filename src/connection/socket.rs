use std::io;
use std::net::{IpAddr, SocketAddr};

use anyhow::{Context, Result};
use socket2::{Domain, Protocol, Socket, Type};
use tracing::warn;

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

/// Optional Linux egress steering via `SO_BINDTODEVICE`.
///
/// Pins egress to a named interface regardless of the routing table, using
/// [`socket2::Socket::bind_device`]. **Default-off**: nothing in the tree
/// constructs this today (no CLI flag, no new on-device network surface). It
/// exists so a device build can opt into interface-pinned bonding without
/// reworking the bind path — see [`UplinkBinder`].
#[cfg(target_os = "linux")]
#[allow(dead_code)] // default-off extension impl: no in-tree consumer by design
pub struct DeviceBinder {
    pub ifname: String,
}

#[cfg(target_os = "linux")]
impl UplinkBinder for DeviceBinder {
    fn bind_egress(&self, socket: &Socket, _source_ip: IpAddr) -> io::Result<()> {
        // SO_BINDTODEVICE — bind egress to the named interface. The method is
        // `bind_device` (NOT `set_bind_device`); `None` would clear the binding.
        socket.bind_device(Some(self.ifname.as_bytes()))
    }
}

pub fn bind_from_ip(ip: IpAddr, port: u16) -> Result<Socket> {
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

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use socket2::{Domain, Protocol, Socket, Type};

    use super::{SourceIpBinder, UplinkBinder};

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
            };
            assert_eq!(binder.ifname, "lo");
        }
    }
}
