use crate::utils::now_ms;

/// Bitrate measurement and tracking
#[derive(Debug, Clone)]
pub struct BitrateTracker {
    /// Wire bytes this uplink has handed to the socket for the whole process
    /// lifetime — the cumulative counter behind the operator-facing "total
    /// transferred" figure (ADR-002).
    ///
    /// **Monotonic.** [`reset`] deliberately does NOT clear it: a socket
    /// replacement (`SrtlaConnection::reconnect`) is a transient link event, not
    /// a new streaming session, and a total that restarts on every radio blip is
    /// worthless. Only a fresh process starts it at 0.
    pub bytes_sent_total: u64,
    /// `bytes_sent_total` as of the last rate calculation — the baseline the
    /// 2-second window subtracts. [`reset`] rebases this to `bytes_sent_total`
    /// (not 0), so the first post-reset window still measures a zero delta.
    pub bytes_sent_window: u64,
    pub last_rate_update_ms: u64,
    pub current_bitrate_bps: f64,
}

impl Default for BitrateTracker {
    fn default() -> Self {
        Self {
            bytes_sent_total: 0,
            bytes_sent_window: 0,
            last_rate_update_ms: now_ms(),
            current_bitrate_bps: 0.0,
        }
    }
}

impl BitrateTracker {
    /// Restart the rate-measurement window, preserving `bytes_sent_total`.
    ///
    /// Called on socket replacement. Rebasing the window to the current total
    /// (rather than zeroing both) keeps the next `calculate()` delta at 0 —
    /// byte-identical rate behavior to zeroing both — while the cumulative
    /// counter survives the reconnect.
    pub fn reset(&mut self) {
        self.bytes_sent_window = self.bytes_sent_total;
        self.last_rate_update_ms = now_ms();
        self.current_bitrate_bps = 0.0;
    }

    /// Update bitrate tracking when bytes are sent (matches Android C implementation)
    #[inline]
    pub fn update_on_send(&mut self, bytes_sent: u64) {
        self.bytes_sent_total = self.bytes_sent_total.saturating_add(bytes_sent);
    }

    /// Calculate current bitrate over a 2-second window (matching Android C implementation)
    pub fn calculate(&mut self) {
        const BITRATE_UPDATE_INTERVAL_MS: u64 = 2000;

        let now = now_ms();
        let time_diff_ms = now.saturating_sub(self.last_rate_update_ms);

        if time_diff_ms >= BITRATE_UPDATE_INTERVAL_MS {
            let bytes_diff = self.bytes_sent_total.saturating_sub(self.bytes_sent_window);

            if time_diff_ms > 0 {
                // Convert to bits per second: (bytes * 8 * 1000) / milliseconds
                let bits = bytes_diff.saturating_mul(8);
                self.current_bitrate_bps = (bits as f64 * 1000.0) / time_diff_ms as f64;
            }

            self.last_rate_update_ms = now;
            self.bytes_sent_window = self.bytes_sent_total;
        }
    }

    /// Get current bitrate in Mbps
    pub fn mbps(&self) -> f64 {
        self.current_bitrate_bps / 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrate_send_updates_ewma() {
        let mut t = BitrateTracker::default();
        assert_eq!(t.current_bitrate_bps, 0.0);

        // Backdate the window so the next calculate() crosses the 2s interval.
        t.last_rate_update_ms = now_ms().saturating_sub(2_500);
        t.update_on_send(500_000);
        assert_eq!(t.bytes_sent_total, 500_000);

        t.calculate();
        assert!(
            t.current_bitrate_bps > 0.0,
            "sending bytes must raise the estimate, got {}",
            t.current_bitrate_bps
        );
    }

    #[test]
    fn bitrate_idle_decay() {
        let mut t = BitrateTracker::default();

        // Establish a non-zero estimate.
        t.last_rate_update_ms = now_ms().saturating_sub(2_500);
        t.update_on_send(500_000);
        t.calculate();
        assert!(t.current_bitrate_bps > 0.0);

        // Next window with no further sends: bytes_diff == 0 -> estimate decays to 0.
        t.last_rate_update_ms = now_ms().saturating_sub(2_500);
        t.calculate();
        assert_eq!(
            t.current_bitrate_bps, 0.0,
            "an idle window must decay the estimate to zero"
        );
    }

    #[test]
    fn bitrate_wire_bytes_basis() {
        let mut t = BitrateTracker::default();

        let before = now_ms().saturating_sub(4_000);
        t.last_rate_update_ms = before;
        t.bytes_sent_window = 0;
        t.update_on_send(1_000_000);

        t.calculate();

        // calculate() stamps last_rate_update_ms with the now_ms() it used, so the
        // exact elapsed window is recoverable for a precise expectation.
        let elapsed = t.last_rate_update_ms.saturating_sub(before);
        let expected = (1_000_000u64 * 8) as f64 * 1000.0 / elapsed as f64;
        assert!(
            (t.current_bitrate_bps - expected).abs() < 1.0,
            "bitrate is wire-bytes/s x8 (bps): got {}, expected {}",
            t.current_bitrate_bps,
            expected
        );
    }

    #[test]
    fn reset_preserves_cumulative_total() {
        let mut t = BitrateTracker::default();
        t.update_on_send(1_000_000);

        t.reset();

        assert_eq!(
            t.bytes_sent_total, 1_000_000,
            "a socket replacement is a transient link event, not a new session: the cumulative \
             total must survive it"
        );
        assert_eq!(
            t.bytes_sent_window, t.bytes_sent_total,
            "the rate window must be rebased to the total, not zeroed"
        );
        assert_eq!(t.current_bitrate_bps, 0.0);
    }

    #[test]
    fn reset_does_not_spike_the_next_rate_window() {
        // Rebasing rather than zeroing is only safe if the first post-reset
        // window still measures a zero delta; a naive "keep total, zero window"
        // would report the entire session's bytes as one window's worth.
        let mut t = BitrateTracker::default();
        t.update_on_send(10_000_000);
        t.reset();

        t.last_rate_update_ms = now_ms().saturating_sub(2_500);
        t.calculate();

        assert_eq!(
            t.current_bitrate_bps, 0.0,
            "no bytes sent since the reset, so the window must read zero"
        );
    }

    #[test]
    fn cumulative_total_accrues_across_a_reset() {
        let mut t = BitrateTracker::default();
        t.update_on_send(400);
        t.reset();
        t.update_on_send(600);

        assert_eq!(t.bytes_sent_total, 1_000);
    }
}
