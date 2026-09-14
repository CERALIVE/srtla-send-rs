//! Attempted-wire credit, independent of ACK accounting and ranking weights.

#[derive(Debug)]
pub(crate) struct WireBudget {
    rate_bps: f64,
    credit_bytes: f64,
    last_ms: u64,
    accepted_bytes: u64,
}

impl WireBudget {
    pub const BURST_BYTES: usize = 2 * crate::protocol::MTU;

    pub fn new(rate_bps: f64, now_ms: u64) -> Self {
        Self {
            rate_bps,
            credit_bytes: Self::BURST_BYTES as f64,
            last_ms: now_ms,
            accepted_bytes: 0,
        }
    }

    pub fn configure(&mut self, rate_bps: f64, now_ms: u64) {
        self.credit_bytes = self.available(now_ms);
        self.last_ms = now_ms;
        self.rate_bps = rate_bps;
    }

    pub fn available(&self, now_ms: u64) -> f64 {
        let elapsed = std::time::Duration::from_millis(now_ms.saturating_sub(self.last_ms));
        (self.credit_bytes + elapsed.as_secs_f64() * self.rate_bps / 8.0)
            .min(Self::BURST_BYTES as f64)
    }

    pub fn debit(&mut self, bytes: usize, now_ms: u64) {
        // Datagram/burst lengths are bounded by MTU and exactly representable here.
        self.credit_bytes = self.available(now_ms) - bytes as f64;
        self.last_ms = now_ms;
        self.accepted_bytes = self.accepted_bytes.saturating_add(bytes as u64);
    }

    pub const fn totals(&self) -> (f64, u64) {
        (self.rate_bps, self.accepted_bytes)
    }
}

#[cfg(test)]
#[path = "wire_budget_tests.rs"]
mod tests;
