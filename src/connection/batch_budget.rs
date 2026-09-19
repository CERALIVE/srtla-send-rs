use super::{BatchSender, WireSample};
use crate::connection::wire_budget::WireBudget;

impl BatchSender {
    pub(crate) fn wire_sample(&self) -> Option<WireSample> {
        self.wire_totals()
            .map(|(rate_bps, accepted_bytes)| WireSample {
                rate_bps,
                accepted_bytes,
            })
    }
    pub(crate) fn wire_totals(&self) -> Option<(f64, u64)> {
        self.wire_budget.as_ref().map(WireBudget::totals)
    }
    pub(crate) fn configure_wire_rate(&mut self, rate_bps: Option<f64>, now_ms: u64) {
        match (self.wire_budget.as_mut(), rate_bps) {
            (Some(budget), Some(rate)) => budget.configure(rate, now_ms),
            (None, Some(rate)) => self.wire_budget = Some(WireBudget::new(rate, now_ms)),
            (Some(_), None) => self.wire_budget = None,
            (None, None) => {}
        }
    }

    pub(crate) fn wire_limited(&self) -> bool {
        self.wire_budget.is_some()
    }

    pub(crate) fn can_queue_wire(&self, bytes: usize, now_ms: u64) -> bool {
        self.wire_budget.as_ref().is_none_or(|budget| {
            let reserved: usize = self.queue.iter().map(|packet| packet.len()).sum();
            // Runtime datagrams and reservations are bounded by the two-MTU burst.
            reserved.saturating_add(bytes) as f64 <= budget.available(now_ms)
        })
    }

    pub(super) fn funded_prefix(&self, limit: usize, now_ms: u64) -> usize {
        let Some(budget) = &self.wire_budget else {
            return limit;
        };
        let available = budget.available(now_ms);
        let mut bytes = 0;
        self.queue
            .iter()
            .take(limit)
            .take_while(|packet| {
                bytes += packet.len();
                bytes as f64 <= available
            })
            .count()
    }

    pub(super) fn debit_prefix(&mut self, accepted: usize, now_ms: u64) {
        if let Some(budget) = &mut self.wire_budget {
            let bytes = self
                .queue
                .iter()
                .take(accepted)
                .map(|packet| packet.len())
                .sum();
            budget.debit(bytes, now_ms);
        }
    }
}
