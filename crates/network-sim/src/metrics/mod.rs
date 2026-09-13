//! Offline benchmark measurements; clocks are milliseconds relative to measurement start.

pub mod control;
pub mod cpu;
pub mod episodes;
mod error;
pub mod link_counters;
pub mod load_intervals;
pub mod sink;
pub mod srt_stats;
mod srt_window;
pub mod stats_file;
pub use srt_window::SrtWindow;
pub mod identity;
pub mod record;
pub mod sampling;
mod window;
pub use error::MetricError;
pub use record::RunRecord;
pub use window::Window;

pub const SAMPLE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

// Rates deliberately approximate integer counters as f64; stored raw counts stay exact.
fn rate_number(value: u64) -> f64 {
    value as f64
}

#[cfg(test)]
mod srt_tests;

#[cfg(test)]
mod collector_tests;
#[cfg(test)]
mod edge_tests;
#[cfg(test)]
mod episode_tests;
#[cfg(test)]
mod record_tests;
