//! Attributed normal-DATA loss evidence, separate from future probe-train loss.

use std::collections::VecDeque;

use crate::ewma::Ewma;

#[path = "loss_debit.rs"]
mod debit;
pub(crate) use debit::{LossDebit, LossSend};

pub const LOSS_COHORT_MIN_SENDS: u32 = 100;
pub(crate) const UNKNOWN_LATENCY_MS: u32 = 500;
const LOSS_COHORT_MS: u64 = 1000;
// Retain the existing ten-second evidence horizon, not an unbounded packet history.
const LOSS_HISTORY_MS: u64 = 10_000;

#[derive(Debug, Clone)]
struct SendCohort {
    start_ms: u64,
    sends: u32,
    naks: u32,
    settle_at_ms: u64,
    settled: bool,
}

#[derive(Debug, Clone)]
pub struct LossTracker {
    recovery_epoch: u64,
    cohort_start_ms: u64,
    cohort_sends: u32,
    cohort_naks: u32,
    latency_ms: u32,
    cohort_settle_at_ms: u64,
    closed: VecDeque<SendCohort>,
    prior_ewma: Ewma,
    prior_last_cohort_ms: Option<u64>,
    last_value: Option<f64>,
    last_cohort_ms: Option<u64>,
    last_settlement_ms: Option<u64>,
    /// Todo 19 supplies unacknowledged probe copies / copies in its rejoin trains.
    pub(crate) probe_loss: Option<f64>,
}

impl LossTracker {
    pub fn new(now_ms: u64) -> Self {
        Self {
            recovery_epoch: 0,
            cohort_start_ms: now_ms,
            cohort_sends: 0,
            cohort_naks: 0,
            latency_ms: UNKNOWN_LATENCY_MS,
            cohort_settle_at_ms: now_ms,
            closed: VecDeque::with_capacity(11),
            prior_ewma: Ewma::new(0.2),
            prior_last_cohort_ms: None,
            last_value: None,
            last_cohort_ms: None,
            last_settlement_ms: None,
            probe_loss: None,
        }
    }

    /// Count only kernel-accepted normal DATA, never queued packets or probe copies.
    pub fn record_send(&mut self, now_ms: u64) {
        self.record_accepted_send(now_ms);
    }

    pub(crate) const fn set_latency_ms(&mut self, latency_ms: u32) {
        self.latency_ms = latency_ms;
    }

    pub(crate) fn record_accepted_send(&mut self, now_ms: u64) -> LossSend {
        self.advance(now_ms);
        self.cohort_sends = self.cohort_sends.saturating_add(1);
        let deadline_ms = now_ms.saturating_add(u64::from(self.latency_ms));
        self.cohort_settle_at_ms = self.cohort_settle_at_ms.max(deadline_ms);
        LossSend {
            epoch: self.recovery_epoch,
            cohort_start_ms: self.cohort_start_ms,
            deadline_ms,
        }
    }

    /// Called once per removed normal packet-log entry, never for a probe-log hit.
    pub fn record_data_nak(&mut self, now_ms: u64) {
        self.record_data_nak_for_send(now_ms, now_ms);
    }

    /// The numerator follows the send cohort, never the arrival cohort. Late feedback
    /// corrects retained history in chronological order without refreshing its date.
    pub fn record_data_nak_for_send(&mut self, sent_ms: u64, now_ms: u64) {
        let _ = self.debit_data_nak(sent_ms, now_ms);
    }

    pub(crate) fn debit_data_nak(&mut self, sent_ms: u64, now_ms: u64) -> Option<LossDebit> {
        self.advance(now_ms);
        if sent_ms > now_ms {
            return None;
        }
        let start = if sent_ms >= self.cohort_start_ms {
            self.cohort_start_ms
        } else {
            self.closed
                .iter()
                .find(|c| sent_ms >= c.start_ms && sent_ms - c.start_ms < LOSS_COHORT_MS)?
                .start_ms
        };
        self.debit_accepted_send(
            LossSend {
                epoch: self.recovery_epoch,
                cohort_start_ms: start,
                deadline_ms: sent_ms.saturating_add(u64::from(self.latency_ms)),
            },
            now_ms,
        )
    }

    pub(crate) fn debit_accepted_send(&mut self, send: LossSend, now_ms: u64) -> Option<LossDebit> {
        self.advance(now_ms);
        if send.epoch != self.recovery_epoch {
            return None;
        }
        if send.cohort_start_ms == self.cohort_start_ms {
            self.cohort_naks = self.cohort_naks.saturating_add(1);
        } else {
            let cohort = self
                .closed
                .iter_mut()
                .find(|c| c.start_ms == send.cohort_start_ms)?;
            cohort.naks = cohort.naks.saturating_add(1);
            self.recompute();
        }
        let debit = LossDebit { send, count: 1 };
        debit.pending_at(now_ms).then_some(debit)
    }

    pub(crate) fn credit_recovered(&mut self, debit: LossDebit, now_ms: u64) {
        self.advance(now_ms);
        if debit.send.epoch != self.recovery_epoch || !debit.pending_at(now_ms) {
            return;
        }
        let naks = if debit.send.cohort_start_ms == self.cohort_start_ms {
            &mut self.cohort_naks
        } else if let Some(cohort) = self
            .closed
            .iter_mut()
            .find(|c| c.start_ms == debit.send.cohort_start_ms && !c.settled)
        {
            &mut cohort.naks
        } else {
            return;
        };
        if let Some(remaining) = naks.checked_sub(debit.count) {
            *naks = remaining;
        }
    }

    /// Close elapsed [start, start+1000) cohorts. Call before sampling idle links.
    /// Empty/sub-floor cohorts neither update EWMA nor refresh its evidence date.
    pub fn advance(&mut self, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.cohort_start_ms);
        let mut changed = false;
        if elapsed >= LOSS_COHORT_MS {
            if self.cohort_sends > 0 {
                self.closed.push_back(SendCohort {
                    start_ms: self.cohort_start_ms,
                    sends: self.cohort_sends,
                    naks: self.cohort_naks,
                    settle_at_ms: self
                        .cohort_settle_at_ms
                        .max(self.cohort_start_ms + LOSS_COHORT_MS),
                    settled: false,
                });
            }
            self.cohort_start_ms = now_ms - elapsed % LOSS_COHORT_MS;
            self.cohort_sends = 0;
            self.cohort_naks = 0;
            self.cohort_settle_at_ms = self.cohort_start_ms;
        }
        for cohort in &mut self.closed {
            if !cohort.settled && now_ms >= cohort.settle_at_ms {
                cohort.settled = true;
                changed = true;
                if cohort.sends >= LOSS_COHORT_MIN_SENDS {
                    self.last_settlement_ms = Some(
                        self.last_settlement_ms
                            .map_or(cohort.settle_at_ms, |last| last.max(cohort.settle_at_ms)),
                    );
                }
            }
        }
        while self
            .closed
            .front()
            .is_some_and(|c| now_ms.saturating_sub(c.start_ms + LOSS_COHORT_MS) >= LOSS_HISTORY_MS)
        {
            changed = true;
            if let Some(cohort) = self.closed.pop_front()
                && cohort.settled
                && cohort.sends >= LOSS_COHORT_MIN_SENDS
            {
                self.prior_ewma
                    .update(f64::from(cohort.naks) / f64::from(cohort.sends));
                self.prior_last_cohort_ms = Some(cohort.start_ms + LOSS_COHORT_MS);
            }
        }
        if changed {
            self.recompute();
        }
    }

    fn recompute(&mut self) {
        let mut ewma = self.prior_ewma.clone();
        let mut last = self.prior_last_cohort_ms;
        for cohort in &self.closed {
            if cohort.settled && cohort.sends >= LOSS_COHORT_MIN_SENDS {
                ewma.update(f64::from(cohort.naks) / f64::from(cohort.sends));
                last = Some(cohort.start_ms + LOSS_COHORT_MS);
            }
        }
        self.last_value = last.map(|_| ewma.value());
        self.last_cohort_ms = last;
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

    /// A fresh settlement can authorize demotion, but never revive stale history.
    /// A retained EWMA remains readable for clearance even after this becomes false.
    pub fn loss_cohort_ok(&self, now_ms: u64, stale_after_ms: u64) -> bool {
        self.last_settlement_ms
            .is_some_and(|at| now_ms.saturating_sub(at) < LOSS_COHORT_MS)
            && !self.is_stale(now_ms, stale_after_ms)
    }

    pub const fn probe_loss(&self) -> Option<f64> {
        self.probe_loss
    }

    /// Only qualified health recovery may discard the previous outage's normal evidence.
    /// Probe evidence remains independent; the next normal cohort must qualify afresh.
    pub(crate) fn begin_recovered_epoch(&mut self, now_ms: u64) {
        self.recovery_epoch = self.recovery_epoch.wrapping_add(1);
        self.prior_ewma.reset();
        self.prior_last_cohort_ms = None;
        self.closed.clear();
        self.last_value = None;
        self.last_cohort_ms = None;
        self.last_settlement_ms = None;
        self.cohort_start_ms = now_ms;
        self.cohort_sends = 0;
        self.cohort_naks = 0;
        self.cohort_settle_at_ms = now_ms;
    }
}

#[cfg(test)]
#[path = "loss_tests.rs"]
mod tests;
