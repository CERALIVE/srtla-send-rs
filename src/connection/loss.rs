//! Attributed normal-DATA loss evidence, separate from future probe-train loss.

use crate::ewma::Ewma;

pub const LOSS_COHORT_MIN_SENDS: u32 = 100;
const LOSS_COHORT_MS: u64 = 1000;

#[derive(Debug, Clone)]
pub struct LossTracker {
    cohort_start_ms: u64,
    cohort_sends: u32,
    cohort_naks: u32,
    ewma: Ewma,
    last_value: Option<f64>,
    last_cohort_ms: Option<u64>,
    /// Todo 19 supplies unacknowledged probe copies / copies in its rejoin trains.
    pub(crate) probe_loss: Option<f64>,
}

impl LossTracker {
    pub fn new(now_ms: u64) -> Self {
        Self {
            cohort_start_ms: now_ms,
            cohort_sends: 0,
            cohort_naks: 0,
            ewma: Ewma::new(0.2),
            last_value: None,
            last_cohort_ms: None,
            probe_loss: None,
        }
    }

    /// Count only kernel-accepted normal DATA, never queued packets or probe copies.
    pub fn record_send(&mut self, now_ms: u64) {
        self.advance(now_ms);
        self.cohort_sends = self.cohort_sends.saturating_add(1);
    }

    /// Called once per removed normal packet-log entry, never for a probe-log hit.
    pub fn record_data_nak(&mut self, now_ms: u64) {
        self.advance(now_ms);
        self.cohort_naks = self.cohort_naks.saturating_add(1);
    }

    /// Close elapsed [start, start+1000) cohorts. Call before sampling idle links.
    /// Empty/sub-floor cohorts neither update EWMA nor refresh its evidence date.
    pub fn advance(&mut self, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.cohort_start_ms);
        if elapsed < LOSS_COHORT_MS {
            return;
        }
        if self.cohort_sends >= LOSS_COHORT_MIN_SENDS {
            self.ewma
                .update(f64::from(self.cohort_naks) / f64::from(self.cohort_sends));
            self.last_value = Some(self.ewma.value());
            self.last_cohort_ms = Some(self.cohort_start_ms + LOSS_COHORT_MS);
        }
        self.cohort_start_ms = now_ms - elapsed % LOSS_COHORT_MS;
        self.cohort_sends = 0;
        self.cohort_naks = 0;
    }

    pub const fn last_value(&self) -> Option<f64> {
        self.last_value
    }

    /// End of the last qualified cohort, not the time a late advance observed it.
    pub const fn last_cohort_ms(&self) -> Option<u64> {
        self.last_cohort_ms
    }

    pub fn is_stale(&self, now_ms: u64, stale_after_ms: u64) -> bool {
        self.last_cohort_ms
            .is_none_or(|end| now_ms.saturating_sub(end) >= stale_after_ms)
    }

    /// Only the latest completed cohort can authorize normal-loss demotion.
    /// A retained EWMA remains readable for clearance even after this becomes false.
    pub fn loss_cohort_ok(&self, now_ms: u64, stale_after_ms: u64) -> bool {
        self.last_cohort_ms
            .is_some_and(|end| now_ms.saturating_sub(end) < LOSS_COHORT_MS)
            && !self.is_stale(now_ms, stale_after_ms)
    }

    pub const fn probe_loss(&self) -> Option<f64> {
        self.probe_loss
    }
}

#[cfg(test)]
#[path = "loss_tests.rs"]
mod tests;
