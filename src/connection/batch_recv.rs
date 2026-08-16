//! Batch UDP I/O using `recvmmsg`/`sendmmsg` on Linux platforms.
//!
//! This module provides efficient batch reception of UDP packets by using the
//! `recvmmsg` syscall on Linux, which can receive multiple datagrams in a
//! single kernel transition, and the symmetric `sendmmsg` batch transmit path.
//! On non-Linux platforms, it falls back to single-packet receives/sends.
//!
//! ## Ownership map
//!
//! * [`BatchUdpSocket`] owns the socket, the **resolved peer address**, readiness
//!   handling, and the raw syscall layer. Uplink sockets are deliberately
//!   **unconnected** (no `connect(2)`), so every send names the peer explicitly
//!   and every receive accepts datagrams from any source (a multi-homed / NAT
//!   receiver may legitimately reply from a different address — the C reference
//!   sender has always behaved this way).
//! * `BatchSender` (`super::batch_send`) owns the queue and the drain
//!   bookkeeping.
//! * Socket creation/binding stays in `super::socket` (the `UplinkBinder` pins
//!   the egress *before* any traffic).
//!
//! ## Send contract (datagram semantics)
//!
//! A UDP datagram is all-or-nothing. Every send path in this module obeys the
//! same rules:
//!
//! * `Ok(n)` with `n == packet.len()` ⇒ that datagram was accepted.
//! * `Ok(n)` with `n != packet.len()` ⇒ **hard error**. The remainder is never
//!   re-sent as a fresh datagram (that would corrupt the byte stream).
//! * `Interrupted` (EINTR) ⇒ retry without clearing readiness.
//! * `WouldBlock` ⇒ clear readiness and await writability.
//! * anything else ⇒ hard error.
//!
//! For `sendmmsg` the same rules apply per message: **every** accepted
//! `mmsghdr.msg_len` is validated against its packet length, and a short
//! `msg_len` is a hard error *at that index* — the accepted prefix is the
//! messages strictly before it.
//!
//! ## Test-only send-failure injection
//!
//! Tests that exercise the send-failure recovery paths need a send that fails
//! *synchronously and deterministically*. Naming a port-0 peer does that on
//! Linux (`EINVAL`) but not on macOS, where an unconnected `sendto` to port 0
//! is accepted, so every such test silently stopped testing recovery. Rather
//! than depend on per-datagram destination-validation semantics — which differ
//! across Linux/macOS/Windows now that uplink sockets are unconnected —
//! [`BatchUdpSocket::fail_sends`] flips a flag that makes every send path
//! return a synthetic error. The flag exists only under `cfg(test)` /
//! `test-internals`; outside them the check is an `#[inline(always)]` `None`
//! and the field is absent, exactly like `crate::ab_metrics`.
//!
//! Based on the rustorrent implementation:
//! https://github.com/sebastiencs/rustorrent/blob/master/src/utp/udp_socket.rs

use crate::protocol::MTU;

/// The synthetic error every send path returns while failure injection is on.
#[cfg(any(test, feature = "test-internals"))]
fn injected_send_error() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "test-injected send failure",
    )
}

/// Number of packets to receive in a single `recvmmsg` call.
/// 32 is a good balance between syscall reduction and memory usage.
#[cfg(target_os = "linux")]
pub const BATCH_RECV_SIZE: usize = 32;

/// Minimum gap between foreign-source diagnostic log lines on one socket, in ms.
const FOREIGN_SOURCE_LOG_INTERVAL_MS: u64 = 1000;

/// A UDP datagram is all-or-nothing, so a short accepted length is a hard error:
/// the remainder must never be re-sent as a fresh datagram.
fn short_datagram_error(sent: usize, expected: usize) -> Option<std::io::Error> {
    (sent != expected).then(|| {
        std::io::Error::new(
            std::io::ErrorKind::WriteZero,
            format!("short UDP send: {sent} of {expected} bytes"),
        )
    })
}

// ============================================================================
// Linux implementation with recvmmsg
// ============================================================================

#[cfg(target_os = "linux")]
mod unix_impl {
    use std::io::ErrorKind;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::task::{Context, Poll, ready};

    use socket2::{SockAddr, Socket};
    use tokio::io::Interest;
    use tokio::io::unix::AsyncFd;

    use super::super::batch_send::BATCH_SEND_SIZE;
    use super::{BATCH_RECV_SIZE, FOREIGN_SOURCE_LOG_INTERVAL_MS, MTU, short_datagram_error};

    const SOCKADDR_STORAGE_LENGTH: libc::socklen_t =
        std::mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;

    /// What the read loop should do after `recvmmsg` returns an error.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum RecvAction {
        /// EINTR: interrupted by a signal — re-issue the syscall.
        Retry,
        /// EAGAIN/EWOULDBLOCK: no datagram ready — wait for readiness.
        WouldBlock,
        /// Any other errno — propagate to the caller.
        Hard,
    }

    /// Classify a `recvmmsg` error. Pure so it is unit-testable with no syscall.
    fn recv_retry_action(err: &std::io::Error) -> RecvAction {
        match err.kind() {
            ErrorKind::Interrupted => RecvAction::Retry,
            ErrorKind::WouldBlock => RecvAction::WouldBlock,
            _ => RecvAction::Hard,
        }
    }

    /// Async UDP socket with batch send/receive support via `sendmmsg`/`recvmmsg`.
    ///
    /// This wraps a `socket2::Socket` in tokio's `AsyncFd` for proper async
    /// readiness polling, then uses the `mmsg` syscalls to move multiple packets
    /// per kernel transition. The socket is **unconnected**; the resolved peer is
    /// owned here and named explicitly on every send.
    pub struct BatchUdpSocket {
        inner: AsyncFd<Socket>,
        /// Resolved receiver address in kernel form, owned by this socket.
        ///
        /// `try_send_batch` hands the kernel `peer.as_ptr()` in every
        /// `msg_name`. The pointer is stable for the whole synchronous
        /// `sendmmsg` call because `&self` keeps this field alive and it is
        /// never mutated after construction.
        peer: SockAddr,
        peer_addr: SocketAddr,
        foreign_source_datagrams: AtomicU64,
        last_foreign_source_log_ms: AtomicU64,
        /// Test-only send-failure injection; see [`BatchUdpSocket::fail_sends`].
        #[cfg(any(test, feature = "test-internals"))]
        force_send_error: std::sync::atomic::AtomicBool,
    }

    impl BatchUdpSocket {
        /// Create a new BatchUdpSocket from a bound, non-blocking socket2::Socket
        /// and the resolved receiver address. The socket must NOT be connected.
        pub fn new(socket: Socket, peer_addr: SocketAddr) -> std::io::Result<Self> {
            Ok(Self {
                inner: AsyncFd::with_interest(socket, Interest::READABLE | Interest::WRITABLE)?,
                peer: SockAddr::from(peer_addr),
                peer_addr,
                foreign_source_datagrams: AtomicU64::new(0),
                last_foreign_source_log_ms: AtomicU64::new(0),
                #[cfg(any(test, feature = "test-internals"))]
                force_send_error: std::sync::atomic::AtomicBool::new(false),
            })
        }

        /// Make every subsequent send on this socket fail with a synthetic
        /// `ConnectionRefused`, with no dependency on OS destination-validation
        /// semantics. See the module-level "Test-only send-failure injection"
        /// note for why this replaces the port-0 peer trick.
        #[cfg(any(test, feature = "test-internals"))]
        pub fn fail_sends(&self) {
            self.force_send_error.store(true, Ordering::Relaxed);
        }

        #[cfg(any(test, feature = "test-internals"))]
        #[inline]
        fn injected_send_error(&self) -> Option<std::io::Error> {
            self.force_send_error
                .load(Ordering::Relaxed)
                .then(super::injected_send_error)
        }

        /// Zero-cost no-op outside test builds: the flag field does not exist and
        /// every send-path call folds away.
        #[cfg(not(any(test, feature = "test-internals")))]
        #[inline(always)]
        fn injected_send_error(&self) -> Option<std::io::Error> {
            None
        }

        /// The receiver address every send on this socket is addressed to.
        pub fn peer_addr(&self) -> SocketAddr {
            self.peer_addr
        }

        /// Count a received datagram's source against the expected peer.
        ///
        /// Foreign-source datagrams are processed normally (deliberate
        /// C-reference parity: multi-homed / NAT receivers reply from other
        /// addresses) but are counted. Returns the running total when the 1/s
        /// log rate limit admits a diagnostic line, otherwise `None`.
        pub fn observe_source(&self, src: Option<SocketAddr>, now_ms: u64) -> Option<u64> {
            if src.is_none_or(|addr| addr == self.peer_addr) {
                return None;
            }
            let total = self
                .foreign_source_datagrams
                .fetch_add(1, Ordering::Relaxed)
                .saturating_add(1);
            let last = self.last_foreign_source_log_ms.load(Ordering::Relaxed);
            if now_ms.saturating_sub(last) < FOREIGN_SOURCE_LOG_INTERVAL_MS {
                return None;
            }
            self.last_foreign_source_log_ms
                .store(now_ms, Ordering::Relaxed);
            Some(total)
        }

        /// Datagrams received from an address other than the resolved peer.
        pub fn foreign_source_datagrams(&self) -> u64 {
            self.foreign_source_datagrams.load(Ordering::Relaxed)
        }

        /// The socket's bound local address.
        pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
            self.inner
                .get_ref()
                .local_addr()?
                .as_socket()
                .ok_or_else(|| {
                    std::io::Error::new(ErrorKind::InvalidData, "uplink local address is not IP")
                })
        }

        /// Get the raw file descriptor.
        pub fn as_raw_fd(&self) -> RawFd {
            self.inner.get_ref().as_raw_fd()
        }

        /// Poll for readability and receive multiple packets.
        pub fn poll_recv_batch(
            &self,
            cx: &mut Context<'_>,
            buffer: &mut RecvMmsgBuffer,
        ) -> Poll<std::io::Result<usize>> {
            loop {
                let mut guard = ready!(self.inner.poll_read_ready(cx))?;

                match buffer.recvmmsg(self.as_raw_fd()) {
                    Ok(count) => return Poll::Ready(Ok(count)),
                    Err(e) => match recv_retry_action(&e) {
                        // EINTR: re-issue without dropping readiness (the fd is
                        // still ready, so the next poll returns immediately).
                        RecvAction::Retry => continue,
                        RecvAction::WouldBlock => {
                            guard.clear_ready();
                            continue;
                        }
                        RecvAction::Hard => return Poll::Ready(Err(e)),
                    },
                }
            }
        }

        /// Receive multiple packets asynchronously.
        ///
        /// Returns the number of packets received. Packets can be accessed
        /// via `buffer.iter()`.
        pub async fn recv_batch(&self, buffer: &mut RecvMmsgBuffer) -> std::io::Result<usize> {
            std::future::poll_fn(|cx| self.poll_recv_batch(cx, buffer)).await
        }

        /// Send one datagram to the resolved peer, per the module send contract.
        pub async fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
            if let Some(e) = self.injected_send_error() {
                return Err(e);
            }
            loop {
                let mut guard = self.inner.ready(Interest::WRITABLE).await?;

                match self.inner.get_ref().send_to(buf, &self.peer) {
                    Ok(n) => return short_datagram_error(n, buf.len()).map_or(Ok(n), Err),
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        guard.clear_ready();
                        continue;
                    }
                    Err(e) => return Err(e),
                }
            }
        }

        /// Try to send one datagram without blocking, retrying on EINTR.
        ///
        /// Returns WouldBlock if the socket is not ready.
        #[allow(dead_code)]
        pub fn try_send(&self, buf: &[u8]) -> std::io::Result<usize> {
            if let Some(e) = self.injected_send_error() {
                return Err(e);
            }
            loop {
                match self.inner.get_ref().send_to(buf, &self.peer) {
                    Ok(n) => return short_datagram_error(n, buf.len()).map_or(Ok(n), Err),
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        /// Transmit up to [`BATCH_SEND_SIZE`] datagrams, returning the number the
        /// kernel accepted as a prefix plus the hard error that stopped it (if
        /// any). A `WouldBlock` after partial progress returns the short count
        /// rather than blocking: the caller retains the unsent suffix.
        pub async fn send_batch(&self, packets: &[&[u8]]) -> (usize, Option<std::io::Error>) {
            if let Some(e) = self.injected_send_error() {
                return (0, Some(e));
            }
            let cap = packets.len().min(BATCH_SEND_SIZE);
            let mut sent = 0usize;
            while sent < cap {
                let mut guard = match self.inner.ready(Interest::WRITABLE).await {
                    Ok(guard) => guard,
                    Err(e) => return (sent, Some(e)),
                };
                match self.try_send_batch(&packets[sent..cap]) {
                    Ok(0) => guard.clear_ready(),
                    Ok(n) => sent += n,
                    Err((accepted, e)) => {
                        sent += accepted;
                        match e.kind() {
                            ErrorKind::Interrupted => continue,
                            ErrorKind::WouldBlock => {
                                guard.clear_ready();
                                if sent > 0 {
                                    return (sent, None);
                                }
                            }
                            _ => return (sent, Some(e)),
                        }
                    }
                }
            }
            (sent, None)
        }

        /// One `sendmmsg` syscall. `Ok(n)` is the kernel-accepted message prefix;
        /// `Err((accepted, e))` reports a hard error after `accepted` messages
        /// were fully accepted.
        pub fn try_send_batch(&self, packets: &[&[u8]]) -> Result<usize, (usize, std::io::Error)> {
            if let Some(e) = self.injected_send_error() {
                return Err((0, e));
            }
            let mut batch = SendMmsgBatch::new();
            let count = batch.init(packets, &self.peer);
            if count == 0 {
                return Ok(0);
            }

            // Safety: `batch` owns the `mmsghdr`/`iovec` arrays for the whole
            // call; `init` just repointed every `msg_iov` into `batch.iov` and
            // every `msg_name` at `self.peer`, both of which outlive this
            // synchronous syscall. `count <= BATCH_SEND_SIZE == hdrs.len()`.
            let result = unsafe {
                libc::sendmmsg(
                    self.as_raw_fd(),
                    batch.hdrs.as_mut_ptr(),
                    count as u32,
                    libc::MSG_DONTWAIT,
                )
            };

            if result < 0 {
                return Err((0, std::io::Error::last_os_error()));
            }
            validate_sent_prefix(&batch.hdrs, packets, result as usize)
        }

        /// Try to receive data without blocking.
        ///
        /// Returns WouldBlock if no data is available.
        #[allow(dead_code)]
        pub fn try_recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            use std::mem::MaybeUninit;

            // Safety: We're using MaybeUninit slice for the socket2 API,
            // but the recv call will initialize the bytes it writes.
            let buf_uninit: &mut [MaybeUninit<u8>] =
                unsafe { &mut *(buf as *mut [u8] as *mut [MaybeUninit<u8>]) };
            self.inner.get_ref().recv(buf_uninit)
        }

        /// Get a reference to the underlying socket.
        #[allow(dead_code)]
        pub fn get_ref(&self) -> &Socket {
            self.inner.get_ref()
        }
    }

    impl AsRawFd for BatchUdpSocket {
        fn as_raw_fd(&self) -> RawFd {
            self.inner.get_ref().as_raw_fd()
        }
    }

    /// Scratch `iovec`/`mmsghdr` arrays for one `sendmmsg` call.
    ///
    /// Self-referential: each `msg_hdr.msg_iov` points at this struct's own
    /// `iov[i]`. `init` rebuilds every pointer from the *current* field
    /// addresses immediately before the syscall, so a move of the value can
    /// never leave them dangling.
    pub struct SendMmsgBatch {
        #[cfg(feature = "test-internals")]
        pub iov: [libc::iovec; BATCH_SEND_SIZE],
        #[cfg(not(feature = "test-internals"))]
        iov: [libc::iovec; BATCH_SEND_SIZE],
        #[cfg(feature = "test-internals")]
        pub hdrs: [libc::mmsghdr; BATCH_SEND_SIZE],
        #[cfg(not(feature = "test-internals"))]
        hdrs: [libc::mmsghdr; BATCH_SEND_SIZE],
        len: usize,
    }

    impl SendMmsgBatch {
        pub fn new() -> Self {
            // Safety: every field is a plain-old-data array of `Copy` structs
            // whose all-zero bit pattern is valid; `init` fully populates the
            // `len` entries actually handed to the kernel.
            let mut batch: Self = unsafe { std::mem::zeroed() };
            batch.len = 0;
            batch
        }

        /// Point the headers at `packets` (capped at [`BATCH_SEND_SIZE`]) and at
        /// `peer`, returning the message count to submit.
        pub fn init(&mut self, packets: &[&[u8]], peer: &SockAddr) -> usize {
            self.len = packets.len().min(BATCH_SEND_SIZE);
            self.rebuild_pointers(packets, peer);
            self.len
        }

        fn rebuild_pointers(&mut self, packets: &[&[u8]], peer: &SockAddr) {
            let count = self.len.min(packets.len()).min(BATCH_SEND_SIZE);
            for (index, packet) in packets.iter().take(count).enumerate() {
                self.iov[index] = libc::iovec {
                    iov_base: packet.as_ptr() as *mut libc::c_void,
                    iov_len: packet.len(),
                };
            }

            let iov = self.iov.as_mut_ptr();
            let peer_name = peer.as_ptr() as *mut libc::c_void;
            let peer_namelen = peer.len();
            for index in 0..count {
                let header = &mut self.hdrs[index];
                header.msg_hdr.msg_name = peer_name;
                header.msg_hdr.msg_namelen = peer_namelen;
                header.msg_hdr.msg_iov = unsafe { iov.add(index) };
                header.msg_hdr.msg_iovlen = 1;
                header.msg_hdr.msg_control = std::ptr::null_mut();
                header.msg_hdr.msg_controllen = 0;
                header.msg_hdr.msg_flags = 0;
                header.msg_len = 0;
            }
        }
    }

    /// Validate the kernel-reported prefix of a `sendmmsg` call.
    ///
    /// A datagram is all-or-nothing, so a `msg_len` that differs from its
    /// packet length is a hard error at that index and the usable accepted
    /// prefix is everything strictly before it. Pure, so miri can vet the
    /// bounds arithmetic without executing the syscall.
    pub(super) fn validate_sent_prefix(
        hdrs: &[libc::mmsghdr],
        packets: &[&[u8]],
        accepted: usize,
    ) -> Result<usize, (usize, std::io::Error)> {
        let bound = accepted.min(hdrs.len()).min(packets.len());
        for index in 0..bound {
            let sent = hdrs[index].msg_len as usize;
            let expected = packets[index].len();
            if sent != expected {
                return Err((
                    index,
                    std::io::Error::new(
                        ErrorKind::WriteZero,
                        format!(
                            "sendmmsg accepted a short datagram at index {index}: {sent} of \
                             {expected} bytes"
                        ),
                    ),
                ));
            }
        }
        if accepted > bound {
            return Err((
                bound,
                std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("sendmmsg reported {accepted} accepted messages, submitted {bound}"),
                ),
            ));
        }
        Ok(bound)
    }

    /// Buffer for batch receiving multiple UDP packets via `recvmmsg`.
    pub struct RecvMmsgBuffer {
        /// Storage for source addresses
        #[cfg(feature = "test-internals")]
        pub addr_storage: [libc::sockaddr_storage; BATCH_RECV_SIZE],
        #[cfg(not(feature = "test-internals"))]
        addr_storage: [libc::sockaddr_storage; BATCH_RECV_SIZE],
        /// IO vectors pointing to packet buffers
        #[cfg(feature = "test-internals")]
        pub iov: [libc::iovec; BATCH_RECV_SIZE],
        #[cfg(not(feature = "test-internals"))]
        iov: [libc::iovec; BATCH_RECV_SIZE],
        /// Message headers for recvmmsg
        #[cfg(feature = "test-internals")]
        pub mmsghdr: [libc::mmsghdr; BATCH_RECV_SIZE],
        #[cfg(not(feature = "test-internals"))]
        mmsghdr: [libc::mmsghdr; BATCH_RECV_SIZE],
        /// Packet data buffers
        #[cfg(feature = "test-internals")]
        pub buffers: [[u8; MTU]; BATCH_RECV_SIZE],
        #[cfg(not(feature = "test-internals"))]
        buffers: [[u8; MTU]; BATCH_RECV_SIZE],
        /// Number of packets received in last call
        nrecv: u32,
    }

    // Safety: every raw pointer in `iov`/`mmsghdr` points into this struct's own
    // `buffers`/`addr_storage`, never outside it. `init()` rebuilds all of those
    // pointers from the *current* field addresses before every `recvmmsg`
    // (via `rebuild_pointers`), so even a safe move of the value — which
    // relocates the fields (e.g. `*RecvMmsgBuffer::new()`, `mem::swap`) — cannot
    // leave them dangling: the next receive re-derives them. The struct is
    // otherwise self-contained (all fields are `Copy` or owned arrays), so it is
    // sound to send across threads.
    unsafe impl Send for RecvMmsgBuffer {}

    impl RecvMmsgBuffer {
        /// Create a new batch receive buffer.
        ///
        /// This allocates the buffer on the heap due to its large size (~50KB).
        pub fn new() -> Box<Self> {
            // Safety: We're zeroing memory that will be properly initialized
            // before use. The iov and mmsghdr pointers are set up by
            // rebuild_pointers below.
            let mut ptr: Box<Self> = Box::new(unsafe { std::mem::zeroed() });
            ptr.rebuild_pointers();
            ptr.nrecv = 0;
            ptr
        }

        /// (Re)point the `iov`/`mmsghdr` self-pointers at the *current* field
        /// addresses. Run from both `new()` and `init()` (before every
        /// `recvmmsg`) so a safe move of the value can never leave them
        /// dangling — after a move the next receive simply re-derives them.
        fn rebuild_pointers(&mut self) {
            let buffers = self.buffers.as_mut_ptr();

            self.iov.iter_mut().enumerate().for_each(|(index, iov)| {
                let buffer = unsafe { &mut *buffers.add(index) };
                *iov = libc::iovec {
                    iov_base: buffer.as_mut_ptr() as *mut libc::c_void,
                    iov_len: buffer.len(),
                }
            });

            let addrs = self.addr_storage.as_mut_ptr();
            let iov = self.iov.as_mut_ptr();

            self.mmsghdr.iter_mut().enumerate().for_each(|(index, h)| {
                h.msg_hdr.msg_name = unsafe { addrs.add(index) as *mut libc::c_void };
                h.msg_hdr.msg_namelen = SOCKADDR_STORAGE_LENGTH;
                h.msg_hdr.msg_iov = unsafe { iov.add(index) };
                h.msg_hdr.msg_iovlen = 1;
                h.msg_hdr.msg_control = std::ptr::null_mut();
                h.msg_hdr.msg_controllen = 0;
                h.msg_hdr.msg_flags = 0;
            });
        }

        /// Reset the buffer for the next recvmmsg call.
        ///
        /// Rebuilds self-referential iovec pointers (S1: move-hazard fix) and zeroes
        /// per-header fields that recvmmsg only writes for filled slots — stale
        /// msg_flags/msg_len from a previous larger batch must not leak into a later
        /// smaller one (S2).
        #[cfg(feature = "test-internals")]
        pub fn init(&mut self) {
            self.rebuild_pointers();
            self.mmsghdr.iter_mut().for_each(|h| {
                h.msg_hdr.msg_namelen = SOCKADDR_STORAGE_LENGTH;
                h.msg_hdr.msg_flags = 0;
                h.msg_len = 0;
            });
        }

        /// Reset the buffer for the next recvmmsg call.
        #[cfg(not(feature = "test-internals"))]
        fn init(&mut self) {
            self.rebuild_pointers();
            self.mmsghdr.iter_mut().for_each(|h| {
                h.msg_hdr.msg_namelen = SOCKADDR_STORAGE_LENGTH;
                h.msg_hdr.msg_flags = 0;
                h.msg_len = 0;
            });
        }

        /// Receive multiple packets using recvmmsg.
        ///
        /// Returns Ok(count) with the number of packets received, or Err if the syscall failed.
        /// WouldBlock errors indicate no data is available (non-blocking socket).
        pub fn recvmmsg(&mut self, fd: RawFd) -> std::io::Result<usize> {
            self.init();

            let result = unsafe {
                libc::recvmmsg(
                    fd,
                    self.mmsghdr.as_mut_ptr(),
                    self.mmsghdr.len() as u32,
                    libc::MSG_DONTWAIT, // Non-blocking
                    std::ptr::null_mut(),
                )
            };

            if result == -1 {
                self.nrecv = 0;
                return Err(std::io::Error::last_os_error());
            }

            self.nrecv = result as u32;
            Ok(result as usize)
        }

        /// Get an iterator over the received packets.
        pub fn iter(&self) -> RecvMmsgIter<'_> {
            RecvMmsgIter {
                buffer: self,
                current: 0,
            }
        }

        /// Get the number of packets received in the last call.
        #[cfg(test)]
        pub fn len(&self) -> usize {
            self.nrecv as usize
        }

        /// Check if no packets were received.
        #[cfg(test)]
        pub fn is_empty(&self) -> bool {
            self.nrecv == 0
        }

        /// Test seam: forge `nrecv` "received" packets and set message `idx`'s
        /// reported `msg_len`. Lets a test feed an out-of-range length without a
        /// live socket to prove the iterator clamps the exposed slice to MTU.
        #[cfg(test)]
        pub fn test_forge_packet(&mut self, idx: usize, msg_len: u32, nrecv: u32) {
            self.mmsghdr[idx].msg_hdr.msg_namelen = SOCKADDR_STORAGE_LENGTH;
            self.mmsghdr[idx].msg_len = msg_len;
            self.nrecv = nrecv;
        }
    }

    /// Iterator over received packets in a RecvMmsgBuffer.
    pub struct RecvMmsgIter<'a> {
        buffer: &'a RecvMmsgBuffer,
        current: u32,
    }

    impl<'a> Iterator for RecvMmsgIter<'a> {
        /// Returns (source_address, packet_data)
        type Item = (Option<SocketAddr>, &'a [u8]);

        fn next(&mut self) -> Option<Self::Item> {
            if self.current >= self.buffer.nrecv {
                return None;
            }

            let idx = self.current as usize;
            self.current += 1;

            let msg = &self.buffer.mmsghdr[idx];
            let storage = &self.buffer.addr_storage[idx];

            // Convert sockaddr_storage to SocketAddr
            let addr = sockaddr_storage_to_socket_addr(storage);

            // The per-message buffer is exactly MTU bytes. No MSG_TRUNC is
            // requested so msg_len is capped at MTU in practice, but clamp
            // defensively so a mis-reported length can never index past it.
            let len = (msg.msg_len as usize).min(MTU);
            let data = &self.buffer.buffers[idx][..len];
            Some((addr, data))
        }
    }

    /// Convert a libc::sockaddr_storage to a std::net::SocketAddr
    fn sockaddr_storage_to_socket_addr(storage: &libc::sockaddr_storage) -> Option<SocketAddr> {
        // Safety: We're reading from a sockaddr_storage that was filled by recvmmsg
        unsafe {
            match storage.ss_family as libc::c_int {
                libc::AF_INET => {
                    let addr_in = storage as *const _ as *const libc::sockaddr_in;
                    let ip = Ipv4Addr::from(u32::from_be((*addr_in).sin_addr.s_addr));
                    let port = u16::from_be((*addr_in).sin_port);
                    Some(SocketAddr::V4(SocketAddrV4::new(ip, port)))
                }
                libc::AF_INET6 => {
                    let addr_in6 = storage as *const _ as *const libc::sockaddr_in6;
                    let ip = Ipv6Addr::from((*addr_in6).sin6_addr.s6_addr);
                    let port = u16::from_be((*addr_in6).sin6_port);
                    let flowinfo = (*addr_in6).sin6_flowinfo;
                    let scope_id = (*addr_in6).sin6_scope_id;
                    Some(SocketAddr::V6(SocketAddrV6::new(
                        ip, port, flowinfo, scope_id,
                    )))
                }
                _ => None,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use std::io::{Error, ErrorKind};
        use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

        use super::{
            RecvAction, RecvMmsgBuffer, recv_retry_action, sockaddr_storage_to_socket_addr,
        };
        use crate::protocol::MTU;

        /// Syscall-free proof of the `sockaddr_storage` → `SocketAddr` cast and
        /// big-endian decode: this is the pure pointer logic the miri lane vets
        /// for UB (`cargo miri test … sockaddr_storage_roundtrip`), since miri
        /// cannot run the real `recvmmsg` that normally fills the storage.
        #[test]
        fn sockaddr_storage_roundtrip() {
            let mut storage: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
            let v4 = Ipv4Addr::new(192, 0, 2, 7);
            let port_v4: u16 = 6000;
            // Safety: `sockaddr_in` is smaller than `sockaddr_storage` and the
            // storage is correctly aligned for it; we only write initialized
            // fields the decoder reads back.
            unsafe {
                let sin = std::ptr::addr_of_mut!(storage).cast::<libc::sockaddr_in>();
                (*sin).sin_family = libc::AF_INET as libc::sa_family_t;
                (*sin).sin_port = port_v4.to_be();
                (*sin).sin_addr.s_addr = u32::from(v4).to_be();
            }
            assert_eq!(
                sockaddr_storage_to_socket_addr(&storage),
                Some(SocketAddr::new(v4.into(), port_v4)),
                "IPv4 sockaddr_storage must round-trip",
            );

            let mut storage6: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
            let v6 = Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0x1);
            let port_v6: u16 = 7000;
            // Safety: as above, for the IPv6 view.
            unsafe {
                let sin6 = std::ptr::addr_of_mut!(storage6).cast::<libc::sockaddr_in6>();
                (*sin6).sin6_family = libc::AF_INET6 as libc::sa_family_t;
                (*sin6).sin6_port = port_v6.to_be();
                (*sin6).sin6_addr.s6_addr = v6.octets();
            }
            match sockaddr_storage_to_socket_addr(&storage6) {
                Some(SocketAddr::V6(s)) => {
                    assert_eq!(*s.ip(), v6, "IPv6 address must round-trip");
                    assert_eq!(s.port(), port_v6, "IPv6 port must round-trip");
                }
                other => panic!("expected V6 SocketAddr, got {other:?}"),
            }

            let storage_unspec: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
            assert_eq!(
                sockaddr_storage_to_socket_addr(&storage_unspec),
                None,
                "unknown ss_family must decode to None",
            );
        }

        #[test]
        fn iter_clamps_oversized_msg_len_to_mtu() {
            let mut buffer = RecvMmsgBuffer::new();
            buffer.test_forge_packet(0, (MTU as u32) * 4, 1);

            let mut iter = buffer.iter();
            let (_addr, data) = iter.next().expect("one forged packet");
            assert_eq!(data.len(), MTU, "oversized msg_len must clamp to MTU");
            assert!(iter.next().is_none(), "only one packet was forged");
        }

        #[test]
        fn recv_retry_action_classifies_errors() {
            assert_eq!(
                recv_retry_action(&Error::from(ErrorKind::Interrupted)),
                RecvAction::Retry,
            );
            assert_eq!(
                recv_retry_action(&Error::from(ErrorKind::WouldBlock)),
                RecvAction::WouldBlock,
            );
            assert_eq!(
                recv_retry_action(&Error::from_raw_os_error(libc::ECONNREFUSED)),
                RecvAction::Hard,
            );
        }
    }
}

// ============================================================================
// Non-Linux fallback implementation
// ============================================================================

#[cfg(not(target_os = "linux"))]
mod fallback_impl {
    use std::io::ErrorKind;
    use std::net::SocketAddr;
    use std::sync::atomic::{AtomicU64, Ordering};

    use socket2::Socket;
    use tokio::net::UdpSocket;

    use super::super::batch_send::BATCH_SEND_SIZE;
    use super::{FOREIGN_SOURCE_LOG_INTERVAL_MS, MTU, short_datagram_error};

    /// Fallback async UDP socket for non-Linux platforms.
    ///
    /// Uses tokio's UdpSocket directly since recvmmsg/sendmmsg are not
    /// available. The socket is unconnected; the peer is named on every send.
    pub struct BatchUdpSocket {
        inner: UdpSocket,
        peer_addr: SocketAddr,
        foreign_source_datagrams: AtomicU64,
        last_foreign_source_log_ms: AtomicU64,
        /// Test-only send-failure injection; see [`BatchUdpSocket::fail_sends`].
        #[cfg(any(test, feature = "test-internals"))]
        force_send_error: std::sync::atomic::AtomicBool,
    }

    impl BatchUdpSocket {
        /// Create a new BatchUdpSocket from a bound, non-blocking socket2::Socket
        /// and the resolved receiver address. The socket must NOT be connected.
        pub fn new(socket: Socket, peer_addr: SocketAddr) -> std::io::Result<Self> {
            // Convert socket2::Socket to std::net::UdpSocket
            let std_socket: std::net::UdpSocket = socket.into();
            Ok(Self {
                inner: UdpSocket::from_std(std_socket)?,
                peer_addr,
                foreign_source_datagrams: AtomicU64::new(0),
                last_foreign_source_log_ms: AtomicU64::new(0),
                #[cfg(any(test, feature = "test-internals"))]
                force_send_error: std::sync::atomic::AtomicBool::new(false),
            })
        }

        /// Make every subsequent send on this socket fail with a synthetic
        /// `ConnectionRefused`; see the module-level note on why this replaces
        /// the port-0 peer trick.
        #[cfg(any(test, feature = "test-internals"))]
        pub fn fail_sends(&self) {
            self.force_send_error.store(true, Ordering::Relaxed);
        }

        #[cfg(any(test, feature = "test-internals"))]
        #[inline]
        fn injected_send_error(&self) -> Option<std::io::Error> {
            self.force_send_error
                .load(Ordering::Relaxed)
                .then(super::injected_send_error)
        }

        /// Zero-cost no-op outside test builds: the flag field does not exist and
        /// every send-path call folds away.
        #[cfg(not(any(test, feature = "test-internals")))]
        #[inline(always)]
        fn injected_send_error(&self) -> Option<std::io::Error> {
            None
        }

        /// The receiver address every send on this socket is addressed to.
        pub fn peer_addr(&self) -> SocketAddr {
            self.peer_addr
        }

        /// Count a received datagram's source against the expected peer; see the
        /// Linux implementation for the risk-acceptance rationale.
        pub fn observe_source(&self, src: Option<SocketAddr>, now_ms: u64) -> Option<u64> {
            if src.is_none_or(|addr| addr == self.peer_addr) {
                return None;
            }
            let total = self
                .foreign_source_datagrams
                .fetch_add(1, Ordering::Relaxed)
                .saturating_add(1);
            let last = self.last_foreign_source_log_ms.load(Ordering::Relaxed);
            if now_ms.saturating_sub(last) < FOREIGN_SOURCE_LOG_INTERVAL_MS {
                return None;
            }
            self.last_foreign_source_log_ms
                .store(now_ms, Ordering::Relaxed);
            Some(total)
        }

        /// Datagrams received from an address other than the resolved peer.
        pub fn foreign_source_datagrams(&self) -> u64 {
            self.foreign_source_datagrams.load(Ordering::Relaxed)
        }

        /// The socket's bound local address.
        pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
            self.inner.local_addr()
        }

        /// Receive packets (single packet at a time on non-Unix).
        pub async fn recv_batch(&self, buffer: &mut RecvMmsgBuffer) -> std::io::Result<usize> {
            match self.inner.recv_from(&mut buffer.buffer).await {
                Ok((n, addr)) => {
                    buffer.len = n;
                    buffer.addr = Some(addr);
                    buffer.has_packet = true;
                    Ok(1)
                }
                Err(e) => {
                    buffer.has_packet = false;
                    Err(e)
                }
            }
        }

        /// Send one datagram to the resolved peer, per the module send contract.
        pub async fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
            if let Some(e) = self.injected_send_error() {
                return Err(e);
            }
            loop {
                match self.inner.send_to(buf, self.peer_addr).await {
                    Ok(n) => return short_datagram_error(n, buf.len()).map_or(Ok(n), Err),
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        /// Try to send one datagram without blocking, retrying on EINTR.
        #[allow(dead_code)]
        pub fn try_send(&self, buf: &[u8]) -> std::io::Result<usize> {
            if let Some(e) = self.injected_send_error() {
                return Err(e);
            }
            loop {
                match self.inner.try_send_to(buf, self.peer_addr) {
                    Ok(n) => return short_datagram_error(n, buf.len()).map_or(Ok(n), Err),
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        /// Sequential fallback for the batch transmit path, capped at
        /// [`BATCH_SEND_SIZE`] for parity with the `sendmmsg` path. Returns the
        /// accepted prefix count plus the hard error that stopped it.
        pub async fn send_batch(&self, packets: &[&[u8]]) -> (usize, Option<std::io::Error>) {
            let cap = packets.len().min(BATCH_SEND_SIZE);
            for (index, packet) in packets.iter().take(cap).enumerate() {
                if let Err(e) = self.send(packet).await {
                    return if index > 0 {
                        (index, None)
                    } else {
                        (0, Some(e))
                    };
                }
            }
            (cap, None)
        }

        /// Try to receive data without blocking.
        #[allow(dead_code)]
        pub fn try_recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
            self.inner.try_recv(buf)
        }

        /// Get a reference to the underlying socket.
        #[allow(dead_code)]
        pub fn get_ref(&self) -> &UdpSocket {
            &self.inner
        }
    }

    /// Fallback buffer for non-Unix platforms.
    pub struct RecvMmsgBuffer {
        /// Single packet buffer
        pub(super) buffer: [u8; MTU],
        /// Length of received packet
        pub(super) len: usize,
        /// Source address
        pub(super) addr: Option<SocketAddr>,
        /// Whether a packet was received
        pub(super) has_packet: bool,
    }

    impl RecvMmsgBuffer {
        pub fn new() -> Box<Self> {
            Box::new(Self {
                buffer: [0u8; MTU],
                len: 0,
                addr: None,
                has_packet: false,
            })
        }

        pub fn iter(&self) -> RecvMmsgIter<'_> {
            RecvMmsgIter {
                buffer: self,
                yielded: false,
            }
        }

        #[cfg(test)]
        pub fn len(&self) -> usize {
            if self.has_packet { 1 } else { 0 }
        }

        #[cfg(test)]
        pub fn is_empty(&self) -> bool {
            !self.has_packet
        }
    }

    impl Default for RecvMmsgBuffer {
        fn default() -> Self {
            Self {
                buffer: [0u8; MTU],
                len: 0,
                addr: None,
                has_packet: false,
            }
        }
    }

    pub struct RecvMmsgIter<'a> {
        buffer: &'a RecvMmsgBuffer,
        yielded: bool,
    }

    impl<'a> Iterator for RecvMmsgIter<'a> {
        type Item = (Option<SocketAddr>, &'a [u8]);

        fn next(&mut self) -> Option<Self::Item> {
            if self.yielded || !self.buffer.has_packet {
                return None;
            }
            self.yielded = true;
            Some((self.buffer.addr, &self.buffer.buffer[..self.buffer.len]))
        }
    }
}

// Re-export the appropriate implementation
#[cfg(not(target_os = "linux"))]
pub use fallback_impl::{BatchUdpSocket, RecvMmsgBuffer};
#[cfg(target_os = "linux")]
pub use unix_impl::{BatchUdpSocket, RecvMmsgBuffer};
#[cfg(all(target_os = "linux", test, feature = "test-internals"))]
use unix_impl::{SendMmsgBatch, validate_sent_prefix};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recv_buffer_creation() {
        let buffer = RecvMmsgBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_buffer_size() {
        // Verify buffer size is reasonable
        let size = std::mem::size_of::<RecvMmsgBuffer>();
        // Should be roughly 32 * 1500 + overhead ≈ 48KB + headers
        assert!(size > 32 * 1400);
        assert!(size < 100_000);
    }
}

#[cfg(all(target_os = "linux", test, feature = "test-internals"))]
mod soundness_tests {
    use std::net::SocketAddr;

    use socket2::SockAddr;

    use super::{BATCH_RECV_SIZE, RecvMmsgBuffer, SendMmsgBatch, validate_sent_prefix};
    use crate::connection::batch_send::BATCH_SEND_SIZE;
    use crate::protocol::MTU;

    fn test_peer() -> SockAddr {
        SockAddr::from("192.0.2.9:5000".parse::<SocketAddr>().unwrap())
    }

    /// Moving a `SendMmsgBatch` relocates its `iov`/`hdrs` arrays, so the
    /// `msg_iov` self-pointers handed to `sendmmsg` must be rebuilt by `init`
    /// from the current addresses. `msg_name` must point at the socket-owned
    /// peer with the peer's own `msg_namelen`, and each `iovec` must describe
    /// the caller's packet exactly — no length rounding, no aliasing.
    #[test]
    fn sendmmsg_pointers_rebuilt_after_move() {
        let peer = test_peer();
        let payloads: [Vec<u8>; 3] = [vec![7u8; 1], vec![9u8; 700], vec![3u8; MTU]];
        let packets: Vec<&[u8]> = payloads.iter().map(Vec::as_slice).collect();

        let mut a = SendMmsgBatch::new();
        let mut b = SendMmsgBatch::new();
        assert_eq!(a.init(&packets, &peer), packets.len());
        assert_eq!(b.init(&packets, &peer), packets.len());

        // Relocate: `a` now holds bytes whose pointers reference `b`'s storage.
        std::mem::swap(&mut a, &mut b);

        let count = a.init(&packets, &peer);
        assert_eq!(count, packets.len(), "init must submit every packet");

        for i in 0..count {
            let msg_iov = a.hdrs[i].msg_hdr.msg_iov.cast_const();
            assert_eq!(
                msg_iov,
                std::ptr::addr_of!(a.iov[i]),
                "hdrs[{i}].msg_iov must point at this batch's own iov[{i}]"
            );
            assert_eq!(a.hdrs[i].msg_hdr.msg_iovlen, 1, "one iovec per datagram");

            assert_eq!(
                a.iov[i].iov_base.cast_const().cast::<u8>(),
                packets[i].as_ptr(),
                "iov[{i}].iov_base must point at the caller's packet"
            );
            assert_eq!(
                a.iov[i].iov_len,
                packets[i].len(),
                "iov[{i}].iov_len must be the exact packet length"
            );

            assert_eq!(
                a.hdrs[i].msg_hdr.msg_name.cast_const().cast::<u8>(),
                peer.as_ptr().cast::<u8>(),
                "hdrs[{i}].msg_name must point at the socket-owned peer"
            );
            assert_eq!(
                a.hdrs[i].msg_hdr.msg_namelen,
                peer.len(),
                "hdrs[{i}].msg_namelen must be the peer's own length"
            );
            assert_eq!(a.hdrs[i].msg_len, 0, "msg_len must be cleared before use");
        }
    }

    /// The accepted-prefix extraction never reads past what was submitted, and a
    /// short `msg_len` is a hard error whose reported prefix is the messages
    /// strictly before it.
    #[test]
    fn sendmmsg_prefix_extraction_bounded() {
        let peer = test_peer();
        let payloads: [Vec<u8>; 4] = [vec![1u8; 10], vec![2u8; 64], vec![3u8; 1300], vec![4u8; 2]];
        let packets: Vec<&[u8]> = payloads.iter().map(Vec::as_slice).collect();

        let mut batch = SendMmsgBatch::new();
        let count = batch.init(&packets, &peer);
        for i in 0..count {
            batch.hdrs[i].msg_len = packets[i].len() as u32;
        }

        for accepted in 0..=count {
            assert_eq!(
                validate_sent_prefix(&batch.hdrs, &packets, accepted)
                    .expect("full-length msg_len is a clean prefix"),
                accepted,
            );
        }

        // A kernel count above the submitted message count must clamp, never slice
        // past the packet slice.
        let over = validate_sent_prefix(&batch.hdrs, &packets, BATCH_SEND_SIZE + 99);
        let (clamped, _) = over.expect_err("an over-long accepted count is a hard error");
        assert_eq!(
            clamped, count,
            "the usable prefix clamps to what was submitted"
        );

        batch.hdrs[2].msg_len = (packets[2].len() - 1) as u32;
        let (prefix, err) = validate_sent_prefix(&batch.hdrs, &packets, count)
            .expect_err("a short msg_len must be a hard error");
        assert_eq!(
            prefix, 2,
            "the accepted prefix is the messages before the short one"
        );
        assert_eq!(err.kind(), std::io::ErrorKind::WriteZero);

        assert_eq!(
            validate_sent_prefix(&batch.hdrs, &packets, 0).expect("zero accepted is bounded"),
            0,
        );
    }

    /// Moving a `RecvMmsgBuffer` relocates its `iov`/`mmsghdr`/`buffers`/
    /// `addr_storage` fields, so the raw self-pointers `recvmmsg` hands the
    /// kernel must be rebuilt by `init()` from the current addresses — else they
    /// point at the freed/other allocation. Pre-fix (pointers cached once in
    /// `new()`) every assertion below fails after the move; this is the RED
    /// proof for the fix.
    #[test]
    fn init_rebuilds_self_pointers_after_move() {
        let mut a = *RecvMmsgBuffer::new();
        let mut b = *RecvMmsgBuffer::new();

        // Relocate: `a` now holds bytes whose cached pointers reference `b`'s
        // (now-dropped) allocation, and vice versa.
        std::mem::swap(&mut a, &mut b);

        a.init();

        for i in 0..BATCH_RECV_SIZE {
            let msg_iov = a.mmsghdr[i].msg_hdr.msg_iov.cast_const();
            let iov_addr = std::ptr::addr_of!(a.iov[i]);
            assert_eq!(
                msg_iov, iov_addr,
                "mmsghdr[{i}].msg_iov must point at iov[{i}]"
            );

            let iov_base = a.iov[i].iov_base.cast_const().cast::<u8>();
            let buf_addr = a.buffers[i].as_ptr();
            assert_eq!(
                iov_base, buf_addr,
                "iov[{i}].iov_base must point at buffers[{i}]"
            );

            let msg_name = a.mmsghdr[i]
                .msg_hdr
                .msg_name
                .cast_const()
                .cast::<libc::sockaddr_storage>();
            let addr_addr = std::ptr::addr_of!(a.addr_storage[i]);
            assert_eq!(
                msg_name, addr_addr,
                "mmsghdr[{i}].msg_name must point at addr_storage[{i}]"
            );

            assert_eq!(a.iov[i].iov_len, MTU, "iov[{i}].iov_len must equal MTU");
        }
    }
}
