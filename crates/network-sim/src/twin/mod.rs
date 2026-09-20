//! Duplicate-IP twin-modem simulation: topology, bind-map publication, stack.
//!
//! This is the only harness in the crate that can reproduce the failure the
//! bind-map exists for — two modems presenting the *same* source address, where
//! legacy source-IP identity silently collapses the second one. See
//! [`topology`] for why a plain veth pair cannot model it.

pub mod publish;
pub mod stack;
pub mod topology;

pub use publish::{BindMapPublisher, TwinRow};
pub use stack::{Mapping, TwinStack};
pub use topology::{RECEIVER_IP, TWIN_IP, TwinTopology};
