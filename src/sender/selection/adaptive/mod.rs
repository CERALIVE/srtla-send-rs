//! Shared admission support retained from Todo 32; no Adaptive selector remains.

pub(crate) mod admission;
mod preference;
pub(crate) mod ranking;
mod sole;
mod state;

pub use preference::preference_multiplier;
pub use state::AdaptiveState;

#[cfg(test)]
pub use super::SchedulerFeatures as AdaptiveFeatures;
#[cfg(test)]
pub use super::select_connection_idx_with_state as select;
