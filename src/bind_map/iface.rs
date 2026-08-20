//! Does this interface name exist on this host?
//!
//! Injected rather than called directly so the validator stays pure and the
//! unknown-iface rejection class is testable without touching the machine's
//! real network configuration.

use std::collections::HashSet;

/// Answers interface-existence questions for bind-map validation.
///
/// `Sync` so a `&dyn IfaceOracle` can cross an `.await` on a spawned task: a
/// `SIGHUP` reload runs the bounded pair read off the packet-forwarding loop.
pub trait IfaceOracle: Sync {
    /// True iff an interface with this exact name exists right now.
    fn exists(&self, iface: &str) -> bool;
}

/// The real host, via `if_nametoindex(3)`.
///
/// Deliberately re-resolved on every call: an interface can appear or disappear
/// between one validation and the next, and a cached answer would authorize a
/// binding against hardware that is gone.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemIfaces;

#[cfg(unix)]
impl IfaceOracle for SystemIfaces {
    fn exists(&self, iface: &str) -> bool {
        let Ok(name) = std::ffi::CString::new(iface) else {
            return false; // an interior NUL cannot name a real interface
        };
        // SAFETY: `name` is a valid NUL-terminated C string that outlives the
        // call, and `if_nametoindex` only reads through it. It returns 0 on
        // failure, which is never a valid interface index.
        unsafe { libc::if_nametoindex(name.as_ptr()) != 0 }
    }
}

#[cfg(not(unix))]
impl IfaceOracle for SystemIfaces {
    fn exists(&self, _iface: &str) -> bool {
        false
    }
}

/// An oracle over a fixed, known set of names.
///
/// For tests, and for any embedder that already knows its interface set and
/// does not want a syscall per row.
#[derive(Debug, Clone, Default)]
pub struct StaticIfaces(HashSet<String>);

impl StaticIfaces {
    pub fn new<I, S>(names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self(names.into_iter().map(Into::into).collect())
    }
}

impl IfaceOracle for StaticIfaces {
    fn exists(&self, iface: &str) -> bool {
        self.0.contains(iface)
    }
}
