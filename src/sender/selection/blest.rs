//! BLEST head-of-line blocking guard.
//!
//! Prevents head-of-line blocking by filtering out links whose one-way delay
//! would cause excessive waiting at the receiver relative to the fastest link.

use crate::connection::SrtlaConnection;

/// Maximum acceptable block time in milliseconds.
const DEFAULT_BLOCK_THRESHOLD_MS: f64 = 50.0;

/// BLEST filter state.
///
/// A static one-way-delay head-of-line guard: a link is admitted while its OWD
/// stays within `threshold_ms` of the fastest link's OWD. The threshold is fixed
/// (no dynamic penalty machinery).
#[derive(Debug)]
pub struct BlestFilter {
    /// Maximum acceptable block time in ms.
    threshold_ms: f64,
}

impl BlestFilter {
    pub fn new() -> Self {
        Self {
            threshold_ms: DEFAULT_BLOCK_THRESHOLD_MS,
        }
    }

    /// Filter connections, returning indices of non-blocked links.
    ///
    /// A link is blocked if its OWD estimate exceeds min_OWD + threshold.
    /// OWD is estimated as rtt_min_ms / 2.0.
    pub fn filter(&self, conns: &[SrtlaConnection]) -> Vec<usize> {
        if conns.is_empty() {
            return vec![];
        }

        // Find minimum OWD across all connected links with valid RTT
        let min_owd = conns
            .iter()
            .filter(|c| c.connected && c.rtt.rtt_min_ms < 200.0)
            .map(|c| c.rtt.rtt_min_ms / 2.0)
            .fold(f64::MAX, f64::min);

        if min_owd == f64::MAX {
            // No valid RTT data — return all connected indices
            return conns
                .iter()
                .enumerate()
                .filter(|(_, c)| c.connected)
                .map(|(i, _)| i)
                .collect();
        }

        let threshold = self.threshold_ms;

        conns
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                if !c.connected {
                    return false;
                }
                let owd = c.rtt.rtt_min_ms / 2.0;
                let block_time = owd - min_owd;
                block_time <= threshold
            })
            .map(|(i, _)| i)
            .collect()
    }
}

impl Default for BlestFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_connections;

    #[test]
    fn test_filter_passes_all_close_rtt() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(3));

        // All links have similar RTT
        conns[0].rtt.rtt_min_ms = 40.0;
        conns[1].rtt.rtt_min_ms = 50.0;
        conns[2].rtt.rtt_min_ms = 60.0;

        let filter = BlestFilter::new();
        let result = filter.filter(&conns);
        assert_eq!(result, vec![0, 1, 2], "All should pass with close RTTs");
    }

    #[test]
    fn test_filter_rejects_high_owd() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(3));

        conns[0].rtt.rtt_min_ms = 20.0; // OWD = 10
        conns[1].rtt.rtt_min_ms = 40.0; // OWD = 20, block_time = 10 < 50 → pass
        conns[2].rtt.rtt_min_ms = 200.0; // excluded by rtt_min_ms < 200 check

        // Give conn 2 a very high RTT that's still under the valid threshold
        conns[2].rtt.rtt_min_ms = 180.0; // OWD = 90, block_time = 80 > 50 → blocked

        let filter = BlestFilter::new();
        let result = filter.filter(&conns);
        assert_eq!(result, vec![0, 1], "High-OWD link should be filtered out");
    }

    #[test]
    fn test_static_threshold_admits_within_50ms() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));

        conns[0].rtt.rtt_min_ms = 20.0; // OWD = 10
        conns[1].rtt.rtt_min_ms = 120.0; // OWD = 60, block_time = 50 <= 50 → pass

        let filter = BlestFilter::new();
        let result = filter.filter(&conns);
        assert_eq!(
            result,
            vec![0, 1],
            "Static 50ms threshold should admit a link exactly at the boundary"
        );
    }
}
