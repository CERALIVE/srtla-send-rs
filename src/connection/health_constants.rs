#[derive(Clone, Copy, Debug)]
pub struct HealthConstants {
    pub stall_attempts: u32,
    pub loss_enter: f64,
    pub loss_clear: f64,
    pub loss_cohort_min_sends: u32,
    pub loss_stale_after_ms: u64,
    pub rejoin_rounds: u32,
    pub probe_train_len: u32,
    pub probe_max_pps: u32,
    pub dwell_backoff_max: u32,
}

impl Default for HealthConstants {
    fn default() -> Self {
        let tuning = crate::adaptive_env::tuning();
        Self {
            stall_attempts: tuning.stall_attempts,
            loss_enter: tuning.loss_enter,
            loss_clear: 0.05,
            loss_cohort_min_sends: 100,
            loss_stale_after_ms: 10_000,
            rejoin_rounds: 2,
            probe_train_len: 10,
            probe_max_pps: 10,
            dwell_backoff_max: 16,
        }
    }
}

impl HealthConstants {
    pub fn stall_tau(&self, srtt_ms: Option<f64>) -> f64 {
        srtt_ms.map_or(3000.0, |rtt| (4.0 * rtt).clamp(1000.0, 3000.0))
    }
    pub fn queue_enter(&self, slow_min_rtt_ms: f64) -> f64 {
        (0.25 * slow_min_rtt_ms).max(10.0)
    }
    pub fn queue_clear(&self, slow_min_rtt_ms: f64) -> f64 {
        (0.125 * slow_min_rtt_ms).max(5.0)
    }
    pub fn train_period_ms(&self, held_links: u32) -> f64 {
        match self.probe_max_pps {
            0 => f64::INFINITY,
            pps => {
                f64::from(held_links) * f64::from(self.probe_train_len) * 1000.0 / f64::from(pps)
            }
        }
    }
    pub fn rejoin_span(&self, srtt_ms: Option<f64>, held_links: u32) -> f64 {
        (2.0 * self.stall_tau(srtt_ms)).max(
            f64::from(self.rejoin_rounds) * self.train_period_ms(held_links)
                + srtt_ms.unwrap_or(0.0),
        )
    }
}
