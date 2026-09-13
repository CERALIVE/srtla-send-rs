use std::collections::VecDeque;

use super::RttTracker;
use crate::utils::now_ms;

pub(super) const FAST_MS: u64 = 1000;
pub(super) const SLOW_MS: u64 = 30_000;

impl RttTracker {
    pub(super) fn record_observation(&mut self, now: u64, rtt: f64) {
        for (window, span) in [
            (&mut self.rtt_obs_fast, FAST_MS),
            (&mut self.rtt_obs_slow, SLOW_MS),
        ] {
            while window
                .front()
                .is_some_and(|(ms, _)| now.saturating_sub(*ms) >= span)
            {
                window.pop_front();
            }
            // Keep one minimum per millisecond, bounding storage even at high ACK rates.
            if window
                .back()
                .is_some_and(|&(ms, value)| ms == now && value <= rtt)
            {
                continue;
            }
            // A newer lower sample dominates older higher samples for their entire lifetime.
            while window.back().is_some_and(|&(_, value)| value >= rtt) {
                window.pop_back();
            }
            window.push_back((now, rtt));
        }
    }

    /// Time-based slow floor for future queue thresholds; absent evidence reads as 0.
    pub fn slow_min_rtt_ms(&self) -> f64 {
        window_min(&self.rtt_obs_slow, now_ms(), SLOW_MS).unwrap_or(0.0)
    }
}

pub(super) fn window_min(window: &VecDeque<(u64, f64)>, now: u64, span: u64) -> Option<f64> {
    window
        .iter()
        .find(|&&(ms, _)| now.saturating_sub(ms) < span)
        .map(|&(_, rtt)| rtt)
}

#[cfg(test)]
mod tests {
    use crate::connection::RttTracker;
    use crate::utils::test_clock::TestClock;

    #[test]
    fn queue_delay_tracks_a_synthetic_ramp() {
        // Given a 50ms floor and a gradual ramp to 250ms, followed by a 1s plateau.
        let clock = TestClock::new(0);
        let mut tracker = RttTracker::default();
        for step in 0..=200 {
            clock.set(step * 100);
            tracker.update_estimate(50 + step);
        }
        for step in 1..=10 {
            clock.set(20_000 + step * 100);
            tracker.update_estimate(250);
        }
        // When measuring the one-way queue component.
        let delay = tracker.queue_delay_ms();
        // Then only the fast floor has risen, by 200ms round-trip.
        assert!((delay - 100.0).abs() <= 0.5, "delay={delay}");
        assert_eq!(tracker.slow_min_rtt_ms(), 50.0);
    }

    #[test]
    fn queue_delay_has_no_false_queue_on_pure_jitter() {
        // Given deterministic ±20ms noise, spanning both observation windows.
        let clock = TestClock::new(0);
        let mut tracker = RttTracker::default();
        // When repeatedly measuring the detector throughout the jitter trace.
        for step in 0..1000 {
            clock.set(step * 50);
            tracker.update_estimate(50 + (step * 17) % 41);
            // Then jitter alone never crosses the 10ms entry floor.
            assert!(tracker.queue_delay_ms() < 10.0);
        }
    }

    #[test]
    fn queue_delay_expires_both_windows_even_without_new_samples() {
        // Given a low floor and a later high sample.
        let clock = TestClock::new(0);
        let mut tracker = RttTracker::default();
        tracker.update_estimate(50);
        clock.set(999);
        tracker.update_estimate(250);
        assert_eq!(tracker.queue_delay_ms(), 0.0);
        // When the exact fast-window boundary passes with no further samples.
        clock.set(1000);
        // Then the old fast floor expires on read; the slow floor is retained.
        assert_eq!(tracker.queue_delay_ms(), 100.0);
        clock.set(1999);
        assert_eq!(tracker.queue_delay_ms(), 0.0);
        clock.set(30_000);
        assert_eq!(tracker.slow_min_rtt_ms(), 250.0);
        clock.set(30_999);
        assert_eq!(tracker.slow_min_rtt_ms(), 0.0);
    }

    #[test]
    fn legacy_rtt_min_windows_unchanged() {
        // Given one low sample followed by 115 high samples with frozen time.
        let _clock = TestClock::new(0);
        let mut tracker = RttTracker::default();
        tracker.update_estimate(50);
        // When sample-count eviction has fully displaced the old baseline.
        for _ in 0..115 {
            tracker.update_estimate(250);
        }
        // Then legacy BLEST sees 250, while separate time windows still see 50.
        assert_eq!(tracker.rtt_min_ms, 250.0);
        assert_eq!(tracker.slow_min_rtt_ms(), 50.0);
        assert_eq!(tracker.queue_delay_ms(), 0.0);
    }

    #[test]
    fn queue_delay_reset_discards_previous_path_observations() {
        // Given a measured queue on the old path.
        let clock = TestClock::new(0);
        let mut tracker = RttTracker::default();
        tracker.update_estimate(50);
        clock.set(1000);
        tracker.update_estimate(250);
        assert_eq!(tracker.queue_delay_ms(), 100.0);
        // When reconnect resets the RTT tracker.
        tracker.reset();
        // Then no old observations influence the new path.
        assert_eq!(tracker.queue_delay_ms(), 0.0);
        assert_eq!(tracker.slow_min_rtt_ms(), 0.0);
    }

    #[test]
    fn queue_delay_ignores_implausible_round_trips() {
        // Given a fresh RTT tracker.
        let _clock = TestClock::new(20_000);
        let mut tracker = RttTracker::default();
        // When same-ms and stale round trips are offered.
        assert_eq!(tracker.record_round_trip(20_000, 20_000), None);
        assert_eq!(tracker.record_round_trip(0, 20_000), None);
        // Then neither time window receives rejected evidence.
        assert_eq!(tracker.queue_delay_ms(), 0.0);
        assert_eq!(tracker.slow_min_rtt_ms(), 0.0);
    }
}
