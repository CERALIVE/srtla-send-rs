//! Per-interface egress lifecycle: ifindex staleness, removal, and rebinding.
//!
//! `SO_BINDTODEVICE` resolves an interface *name* to an ifindex **once**, at
//! `setsockopt` time. After an unplug/replug the socket still holds the old
//! ifindex, and the kernel answers every `sendto` with `ENODEV` (the device is
//! gone) or `ENETUNREACH` (it is down). Neither error heals: the socket can
//! never be made to work again, so it must be recreated and the name
//! re-resolved.
//!
//! That is the whole reason this type exists separately from the socket. The
//! decisions — "re-resolve before every bind", "an `ENODEV` send invalidates
//! this socket", "a changed ifindex is a re-enumeration, not a no-op" — are pure
//! state transitions, so they are unit-testable against an injected resolver
//! without a real interface disappearing under the test.
//!
//! An **unmapped** link (no `--bind-map` row) has no interface at all. Every
//! method below is then a no-op that reports `Ok(None)` / `Unchanged`, which is
//! what keeps the legacy `SourceIpBinder` path byte-identical.

use std::io;

use crate::bind_map::IfaceName;

/// Resolves an interface name to its current kernel ifindex.
///
/// Injected rather than called directly so the staleness transitions can be
/// driven deterministically in tests — a real interface cannot be made to
/// disappear mid-test without privileges.
pub trait IfaceResolver {
    /// The interface's current ifindex, or `None` if no such interface exists.
    fn ifindex(&self, iface: &str) -> Option<u32>;
}

/// The live host, via `if_nametoindex(3)`.
///
/// Never caches: a cached answer would authorize a bind against hardware that
/// is already gone, which is precisely the failure this module exists to catch.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemIfaceResolver;

#[cfg(unix)]
impl IfaceResolver for SystemIfaceResolver {
    fn ifindex(&self, iface: &str) -> Option<u32> {
        let name = std::ffi::CString::new(iface).ok()?;
        // SAFETY: `name` is a valid NUL-terminated C string that outlives the
        // call, and `if_nametoindex` only reads through it. It returns 0 on
        // failure, which is never a valid interface index.
        let index = unsafe { libc::if_nametoindex(name.as_ptr()) };
        (index != 0).then_some(index)
    }
}

#[cfg(not(unix))]
impl IfaceResolver for SystemIfaceResolver {
    fn ifindex(&self, _iface: &str) -> Option<u32> {
        None
    }
}

/// A send/bind failure that means the socket's egress binding is unusable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EgressFault {
    /// `ENODEV` — the bound interface no longer exists.
    InterfaceGone,
    /// `ENETUNREACH` — the bound interface exists but has no usable route.
    NetworkUnreachable,
}

impl EgressFault {
    /// The operator-facing token for this fault.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InterfaceGone => "interface_gone",
            Self::NetworkUnreachable => "network_unreachable",
        }
    }
}

/// Classify an I/O error as an egress fault, if it is one.
///
/// Anything else (`EAGAIN`, `EMSGSIZE`, a peer-side refusal) is an ordinary
/// send error the existing recovery path already handles; only these two mean
/// the *binding* is dead.
#[cfg(unix)]
#[must_use]
pub fn classify_egress_fault(err: &io::Error) -> Option<EgressFault> {
    match err.raw_os_error() {
        Some(libc::ENODEV) => Some(EgressFault::InterfaceGone),
        Some(libc::ENETUNREACH) => Some(EgressFault::NetworkUnreachable),
        _ => None,
    }
}

/// Non-Unix hosts have no `SO_BINDTODEVICE`, so no send can carry an egress
/// fault: every error stays an ordinary send error.
#[cfg(not(unix))]
#[must_use]
pub fn classify_egress_fault(_err: &io::Error) -> Option<EgressFault> {
    None
}

/// Whether this link currently has a usable egress interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    /// The interface resolves (or the link is unmapped and needs none).
    Active,
    /// The named interface is gone. No socket is bound for this link until a
    /// reload names an interface that exists — reconnecting on a name the
    /// kernel does not know would just burn the backoff.
    Removed,
}

/// What a periodic re-resolution found.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EgressPoll {
    /// Same ifindex as the bound socket was created with.
    Unchanged,
    /// The interface re-enumerated under the same name — the socket holds a
    /// stale ifindex and must be recreated.
    Reenumerated { from: u32, to: u32 },
    /// The interface disappeared.
    Removed,
}

/// The egress-binding half of an uplink's lifecycle.
///
/// Owns exactly two facts: which interface this link egresses through (if any),
/// and whether the socket currently bound to it is still trustworthy.
#[derive(Debug, Clone)]
pub struct EgressLifecycle {
    iface: Option<IfaceName>,
    state: LinkState,
    /// ifindex the current socket was bound against. `None` before the first
    /// successful resolution, or while removed.
    bound_ifindex: Option<u32>,
    rebind_requested: bool,
}

impl Default for EgressLifecycle {
    fn default() -> Self {
        Self::unmapped()
    }
}

impl EgressLifecycle {
    /// A link with no bind-map row: legacy source-IP binding only.
    #[must_use]
    pub fn unmapped() -> Self {
        Self {
            iface: None,
            state: LinkState::Active,
            bound_ifindex: None,
            rebind_requested: false,
        }
    }

    /// A link pinned to `iface` via `SO_BINDTODEVICE`.
    #[must_use]
    pub fn for_iface(iface: IfaceName) -> Self {
        Self {
            iface: Some(iface),
            ..Self::unmapped()
        }
    }

    #[must_use]
    pub fn iface(&self) -> Option<&IfaceName> {
        self.iface.as_ref()
    }

    #[must_use]
    pub fn state(&self) -> LinkState {
        self.state
    }

    /// True once something invalidated the current socket (a fatal send error
    /// or a re-enumeration). The socket must be recreated, never reused.
    #[must_use]
    pub fn needs_rebind(&self) -> bool {
        self.rebind_requested
    }

    /// Re-resolve the interface by name. **Call this before every socket
    /// creation** — that is what keeps a replugged device from inheriting the
    /// previous ifindex.
    ///
    /// Returns the ifindex the new socket will bind against (`None` for an
    /// unmapped link).
    pub fn resolve_for_bind(
        &mut self,
        resolver: &dyn IfaceResolver,
    ) -> Result<Option<u32>, EgressFault> {
        let Some(iface) = self.iface.as_ref() else {
            self.state = LinkState::Active;
            self.rebind_requested = false;
            return Ok(None);
        };
        match resolver.ifindex(iface.as_str()) {
            Some(index) => {
                self.state = LinkState::Active;
                self.bound_ifindex = Some(index);
                self.rebind_requested = false;
                Ok(Some(index))
            }
            None => {
                self.state = LinkState::Removed;
                self.bound_ifindex = None;
                // Nothing to rebind to; a reload, not a retry, is the way out.
                self.rebind_requested = false;
                Err(EgressFault::InterfaceGone)
            }
        }
    }

    /// Re-resolve without binding, to catch a replug the send path has not hit
    /// yet. A changed ifindex requests a rebind; a vanished interface removes
    /// the link.
    pub fn poll(&mut self, resolver: &dyn IfaceResolver) -> EgressPoll {
        let Some(iface) = self.iface.as_ref() else {
            return EgressPoll::Unchanged;
        };
        match resolver.ifindex(iface.as_str()) {
            Some(index) => match self.bound_ifindex {
                Some(bound) if bound != index => {
                    self.state = LinkState::Active;
                    self.rebind_requested = true;
                    EgressPoll::Reenumerated {
                        from: bound,
                        to: index,
                    }
                }
                _ => {
                    self.state = LinkState::Active;
                    EgressPoll::Unchanged
                }
            },
            None => {
                self.state = LinkState::Removed;
                self.bound_ifindex = None;
                self.rebind_requested = false;
                EgressPoll::Removed
            }
        }
    }

    /// Record a send failure. Returns the fault when the socket is now unusable,
    /// so the caller can recreate it instead of retrying on a dead binding.
    pub fn note_send_error(&mut self, err: &io::Error) -> Option<EgressFault> {
        // An unmapped link's socket is not device-bound, so these errno values
        // carry no ifindex meaning for it — leave the legacy path untouched.
        self.iface.as_ref()?;
        let fault = classify_egress_fault(err)?;
        match fault {
            EgressFault::InterfaceGone => {
                self.state = LinkState::Removed;
                self.bound_ifindex = None;
                self.rebind_requested = false;
            }
            EgressFault::NetworkUnreachable => self.rebind_requested = true,
        }
        Some(fault)
    }

    /// Adopt the interface a reload names for this link. Returns true when the
    /// binding changed, which obliges the caller to recreate the socket rather
    /// than carry ifindex-scoped state across.
    pub fn adopt(&mut self, iface: Option<IfaceName>) -> bool {
        let changed = self.iface != iface;
        if changed {
            self.iface = iface;
            self.bound_ifindex = None;
            self.rebind_requested = false;
        }
        // A reload is the sanctioned way out of `Removed`, even when the name
        // is unchanged: the operator may have replugged the device.
        self.state = LinkState::Active;
        changed
    }
}
