//! Connection selection strategies for SRTLA bonding
//!
//! This module provides four connection selection strategies:
//!
//! ## Classic Mode
//! Matches the original C implementation exactly:
//! - Simple capacity-based selection
//! - No quality awareness
//! - Pure "pick highest window/(in_flight+1)" algorithm
//!
//! ## Enhanced Mode
//! Improved selection with quality awareness:
//! - Exponential NAK decay (smooth ~8s recovery)
//! - NAK burst detection and penalties
//! - RTT-aware scoring (small bonus for low latency)
//! - Hysteresis (10%) to prevent flip-flopping
//! - Optional smart exploration
//! - Time-based switch dampening to prevent rapid thrashing
//!
//! ## RTT-Threshold Mode
//! Groups links by RTT to reduce packet reordering:
//! - Links within min_rtt + delta are "fast"
//! - Strongly prefers fast links over slow ones
//! - Quality scoring applied within fast link group
//! - Falls back to slow links only when fast links saturated
//!
//! ## EDPF Mode
//! Earliest Delivery Path First — picks the link with the lowest predicted
//! packet-arrival time, run through a BLEST → IoDS → EDPF pipeline:
//! - BLEST head-of-line-blocking guard: a static one-way-delay (OWD) filter
//!   (50ms threshold, no penalty term) excludes links whose OWD would stall
//!   the in-order byte stream
//! - IoDS in-order-delivery constraint: bounds candidates to those that keep
//!   delivery monotonic, resetting when the admitted set is empty so no link
//!   is permanently starved
//! - EDPF argmin: among admitted links, selects the lowest predicted arrival
//!   `(in_flight_bytes + pkt) / effective_capacity + owd`
//! - Per-loop owned scheduler state (no thread-local), threaded through the
//!   send path so selection is deterministic and allocation-free on the hot path

pub mod blest;
mod classic;
pub mod edpf;
mod enhanced;
mod exploration;
pub mod iods;
mod quality;

#[cfg(feature = "test-internals")]
pub mod rtt_threshold;
#[cfg(not(feature = "test-internals"))]
mod rtt_threshold;

// Re-export for backward compatibility
pub use quality::calculate_quality_multiplier;

use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::mode::SchedulingMode;

/// Minimum time in milliseconds between connection switches
/// Prevents rapid thrashing when scores fluctuate due to bursty ACK/NAK patterns.
/// Aligned with FLUSH_INTERVAL_MS (15ms) so connections can rotate between batches
/// while avoiding intra-batch flip-flopping.
pub const MIN_SWITCH_INTERVAL_MS: u64 = 15;

/// Loop-owned BLEST/IoDS scheduler state for the EDPF pipeline.
///
/// Owned by the run loop (single-threaded over the connection pool) instead of a
/// `thread_local!`, which under the multi-thread tokio runtime gave each worker its
/// own fragmented copy of the BLEST/IoDS history. Non-EDPF modes ignore it.
#[derive(Debug, Default)]
pub struct EdpfSchedulerState {
    pub(crate) blest: blest::BlestFilter,
    pub(crate) iods: iods::IodsFilter,
}

/// Select the best connection index based on mode and configuration
///
/// # Arguments
/// * `conns` - Mutable slice of connections (for quality cache updates in enhanced mode)
/// * `last_idx` - Previously selected connection (for hysteresis)
/// * `last_switch_time_ms` - Time of last switch (for time-based dampening)
/// * `current_time_ms` - Current timestamp in milliseconds
/// * `config` - Configuration snapshot with mode and settings
///
/// # Returns
/// The index of the selected connection, or None if no valid connections
#[inline(always)]
pub fn select_connection_idx(
    conns: &mut [SrtlaConnection],
    last_idx: Option<usize>,
    last_switch_time_ms: u64,
    current_time_ms: u64,
    config: &ConfigSnapshot,
    edpf_state: &mut EdpfSchedulerState,
) -> Option<usize> {
    match config.mode {
        SchedulingMode::Classic => {
            // Classic mode: simple capacity-based selection (no dampening, matches original C)
            classic::select_connection(conns)
        }
        SchedulingMode::Enhanced => {
            // Enhanced mode: quality-aware selection with optional exploration and time-based dampening
            enhanced::select_connection(
                conns,
                last_idx,
                last_switch_time_ms,
                current_time_ms,
                config.effective_quality_enabled(),
                config.effective_exploration_enabled(),
            )
        }
        SchedulingMode::RttThreshold => {
            // RTT-threshold mode: prefer low-RTT links to reduce reordering
            rtt_threshold::select_connection(
                conns,
                last_idx,
                last_switch_time_ms,
                current_time_ms,
                config.rtt_delta_ms,
                config.effective_quality_enabled(),
            )
        }
        SchedulingMode::Edpf => {
            // EDPF mode: BLEST → IoDS → EDPF pipeline
            edpf_pipeline_select(conns, config, edpf_state)
        }
    }
}

/// EDPF pipeline: BLEST filters → IoDS ordering → EDPF argmin.
///
/// Matches strata's bonding.rs:30-35:
/// 1. BLEST filters out HoL-blocking links
/// 2. IoDS filters for monotonic ordering
/// 3. EDPF selects argmin(predicted_arrival) from remaining
fn edpf_pipeline_select(
    conns: &[SrtlaConnection],
    _config: &ConfigSnapshot,
    edpf_state: &mut EdpfSchedulerState,
) -> Option<usize> {
    use edpf::SRT_PKT_SIZE;

    let EdpfSchedulerState { blest, iods } = edpf_state;

    // 1. BLEST filters out HoL-blocking links
    let candidates = blest.filter(conns);

    // 2. IoDS filters for monotonic ordering
    let ordered = iods.filter_valid(&candidates, |idx| {
        edpf::arrival_time(&conns[idx], SRT_PKT_SIZE)
    });

    // IoDS filtered every candidate: select via fallback for this tick and reset
    // the ordering state so the next tick starts unconstrained (no permanent
    // self-starvation from a monotonically-ratcheting last_arrival).
    if ordered.is_empty() {
        iods.reset();
        return edpf::select_from_indices(conns, &candidates, SRT_PKT_SIZE)
            .or_else(|| edpf::select_from(conns, SRT_PKT_SIZE));
    }

    // 3. EDPF selects argmin from the IoDS-ordered set. Only an IoDS-path
    // selection may ratchet last_arrival; a fallback selection must not.
    if let Some(idx) = edpf::select_from_indices(conns, &ordered, SRT_PKT_SIZE) {
        if let Some(arrival) = edpf::arrival_time(&conns[idx], SRT_PKT_SIZE) {
            iods.record_scheduled(arrival);
        }
        return Some(idx);
    }

    edpf::select_from_indices(conns, &candidates, SRT_PKT_SIZE)
        .or_else(|| edpf::select_from(conns, SRT_PKT_SIZE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_connections;
    use crate::utils::now_ms;

    #[test]
    fn test_select_connection_idx_classic() {
        // Test that classic mode always picks highest score, ignoring dampening
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        connections[0].in_flight_packets = 5; // Lower score
        connections[1].in_flight_packets = 0; // Highest score
        connections[2].in_flight_packets = 10; // Lowest score

        let last_switch_time_ms = now_ms();
        let current_time_ms = last_switch_time_ms + 100; // Within cooldown

        let config = ConfigSnapshot {
            mode: SchedulingMode::Classic,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
        };

        // Classic mode should pick connection 1 (highest score) even during cooldown
        let result = select_connection_idx(
            &mut connections,
            Some(0),
            last_switch_time_ms,
            current_time_ms,
            &config,
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            result,
            Some(1),
            "Classic mode should pick highest score connection"
        );
    }

    #[test]
    fn test_select_connection_idx_enhanced() {
        // Test that enhanced mode enforces cooldown dampening
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        connections[0].in_flight_packets = 5; // Currently selected, lower score
        connections[1].in_flight_packets = 0; // Highest score
        connections[2].in_flight_packets = 10; // Lowest score

        let last_switch_time_ms = now_ms();
        let current_time_ms = last_switch_time_ms + 5; // Within 15ms cooldown

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: true,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
        };

        // Enhanced mode should stay with connection 0 due to cooldown
        let result = select_connection_idx(
            &mut connections,
            Some(0),
            last_switch_time_ms,
            current_time_ms,
            &config,
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            result,
            Some(0),
            "Enhanced mode should enforce cooldown and stay with current connection"
        );

        // After cooldown expires, should allow switching
        let current_time_after_cooldown = last_switch_time_ms + 20; // Past 15ms cooldown
        let result_after = select_connection_idx(
            &mut connections,
            Some(0),
            last_switch_time_ms,
            current_time_after_cooldown,
            &config,
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            result_after,
            Some(1),
            "Enhanced mode should allow switching after cooldown expires"
        );
    }

    #[test]
    fn test_select_connection_idx_empty() {
        let mut conns: Vec<SrtlaConnection> = vec![];
        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
        };
        let result = select_connection_idx(
            &mut conns,
            None,
            0,
            0,
            &config,
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(result, None);
    }

    #[test]
    fn edpf_pipeline_mutates_caller_owned_state() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(2));
        for c in connections.iter_mut() {
            c.bitrate.current_bitrate_bps = 1_000_000.0;
            c.rtt.rtt_min_ms = 30.0;
        }

        let config = ConfigSnapshot {
            mode: SchedulingMode::Edpf,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
        };

        let mut state = EdpfSchedulerState::default();
        let selected = select_connection_idx(&mut connections, None, 0, 0, &config, &mut state);

        assert!(selected.is_some(), "EDPF should select a connection");
        assert!(
            state.iods.last_arrival > 0.0,
            "the caller-owned IoDS state must be mutated by the pipeline"
        );
    }

    #[test]
    fn edpf_two_states_are_independent() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(2));
        for c in connections.iter_mut() {
            c.bitrate.current_bitrate_bps = 1_000_000.0;
            c.rtt.rtt_min_ms = 30.0;
        }

        let config = ConfigSnapshot {
            mode: SchedulingMode::Edpf,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
        };

        let mut state_a = EdpfSchedulerState::default();
        let state_b = EdpfSchedulerState::default();

        for _ in 0..3 {
            let _ = select_connection_idx(&mut connections, None, 0, 0, &config, &mut state_a);
        }

        assert!(
            state_a.iods.last_arrival > 0.0,
            "state_a should accumulate scheduled arrival history"
        );
        assert_eq!(
            state_b.iods.last_arrival, 0.0,
            "state_b is untouched and must remain at its Default"
        );
        assert!(
            state_a.iods.last_arrival != state_b.iods.last_arrival,
            "two separate EdpfSchedulerState instances must diverge independently"
        );
    }
}
