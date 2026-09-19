//! Queue-bound active search for a per-link attempted-wire rate.

mod queue;
mod search;

use queue::QueueEvidence;

const INITIAL_RATE_BPS: f64 = 1_000_000.0;
const MIN_RATE_BPS: f64 = (crate::protocol::MTU * 8) as f64;
const FULL_UTILIZATION: f64 = 0.90;
const SEARCH_GAIN: f64 = 2.0;
const REPROBE_GAIN: f64 = 1.10;
const BRACKET_RATIO: f64 = 1.10;
const PROBE_MIN_MS: u64 = 200;
const PROBE_MAX_MS: u64 = 500;
const FAST_QUEUE_SAMPLES: usize = 3;
const FAST_QUEUE_RISE_MS: f64 = 2.0;
const CUT_SETTLE_MS: u64 = 1_000;
const REPEAT_CONGESTION_MS: u64 = 1_000;
const REPEAT_SAMPLE_MAX_GAP_MS: u64 = PROBE_MAX_MS;
const RATE_STEP_EVIDENCE_MS: u64 = 1_000;
const CAPACITY_REPROBE_MS: u64 = 30_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireRatePhase {
    Searching,
    Draining,
    DemandLimited,
    CapacityHeld,
}

#[derive(Clone, Copy, Debug)]
pub struct WireRttSample {
    pub observed_ms: u64,
    pub rtt_ms: f64,
}

/// One monotonic observation from the socket-owned wire, delivery and RTT trackers.
#[derive(Clone, Copy, Debug)]
pub struct WireRateInput {
    pub now_ms: u64,
    pub delivery_generation: u32,
    pub accepted_bytes: Option<u64>,
    pub latest_original_delivery_ms: Option<u64>,
    pub latest_rtt: Option<WireRttSample>,
    pub slow_min_rtt_ms: f64,
    pub queue_delay_ms: f64,
}

#[derive(Clone, Copy, Debug)]
struct Epoch {
    started_ms: u64,
    accepted_bytes: u64,
    span_ms: f64,
}

#[derive(Clone, Copy)]
struct EpochResult {
    proven: bool,
    fully_used: bool,
    clean: bool,
}

#[derive(Debug)]
pub struct WireRateEstimator {
    rate_bps: f64,
    phase: WireRatePhase,
    highest_clean: Option<f64>,
    blocked: Option<f64>,
    epoch: Option<Epoch>,
    generation: Option<u32>,
    generation_started_ms: u64,
    last_accepted_bytes: u64,
    queue: QueueEvidence,
    full_clear_since_ms: Option<u64>,
    capacity_probe: bool,
    last_cut_ms: Option<u64>,
    repeat_high_since_ms: Option<u64>,
    repeat_last_sample_ms: Option<u64>,
    repeat_last_queue_delay_ms: Option<f64>,
}

impl Default for WireRateEstimator {
    fn default() -> Self {
        Self {
            rate_bps: INITIAL_RATE_BPS,
            phase: WireRatePhase::Searching,
            highest_clean: None,
            blocked: None,
            epoch: None,
            generation: None,
            generation_started_ms: 0,
            last_accepted_bytes: 0,
            queue: QueueEvidence::default(),
            full_clear_since_ms: None,
            capacity_probe: false,
            last_cut_ms: None,
            repeat_high_since_ms: None,
            repeat_last_sample_ms: None,
            repeat_last_queue_delay_ms: None,
        }
    }
}

impl WireRateEstimator {
    pub fn update(&mut self, input: &WireRateInput) {
        let accepted = input.accepted_bytes.unwrap_or(0);
        let new_generation = self.generation != Some(input.delivery_generation);
        if new_generation {
            self.generation = Some(input.delivery_generation);
            self.generation_started_ms = input.now_ms;
            self.highest_clean = None;
            self.blocked = None;
            self.phase = WireRatePhase::Searching;
            self.capacity_probe = false;
            self.queue = QueueEvidence::default();
            self.last_cut_ms = None;
            self.clear_repeat_confirmation();
        }
        let counter_reset = accepted < self.last_accepted_bytes || input.accepted_bytes.is_none();
        self.last_accepted_bytes = accepted;
        if new_generation || counter_reset {
            self.full_clear_since_ms = None;
            self.clear_repeat_confirmation();
            self.restart_epoch(input);
            return;
        }
        let Some(epoch) = self.epoch else {
            self.restart_epoch(input);
            return;
        };
        let elapsed = input.now_ms.saturating_sub(epoch.started_ms);
        let due = elapsed as f64 >= epoch.span_ms;
        if !due && !self.queue.newer(input.latest_rtt) {
            return;
        }

        let causal_after_ms = epoch
            .started_ms
            .max(self.last_cut_ms.unwrap_or(self.generation_started_ms));
        let queue = self
            .queue
            .observe(input, self.generation_started_ms, causal_after_ms);
        let epoch_result = due.then(|| {
            let attempted_bps =
                8_000.0 * accepted.saturating_sub(epoch.accepted_bytes) as f64 / elapsed as f64;
            EpochResult {
                proven: input
                    .latest_original_delivery_ms
                    .is_some_and(|at| at >= epoch.started_ms && at <= input.now_ms),
                fully_used: attempted_bps >= FULL_UTILIZATION * self.rate_bps,
                clean: queue.current_rate_clear,
            }
        });
        if !queue.clear {
            self.full_clear_since_ms = None;
        }
        match self.phase {
            WireRatePhase::Draining => {
                if queue.drained {
                    self.clear_repeat_confirmation();
                    self.resume_search(input);
                } else if self.congestion_authorized(&queue, epoch_result, input.queue_delay_ms) {
                    self.congestion(input);
                }
            }
            WireRatePhase::Searching
            | WireRatePhase::DemandLimited
            | WireRatePhase::CapacityHeld => {
                if queue.congested {
                    if self.congestion_authorized(&queue, epoch_result, input.queue_delay_ms) {
                        self.congestion(input);
                    }
                } else if let Some(result) = epoch_result {
                    self.finish_epoch(input, result);
                }
            }
        }
    }

    fn restart_epoch(&mut self, input: &WireRateInput) {
        let rtt_span_ms =
            (2.0 * input.slow_min_rtt_ms).clamp(PROBE_MIN_MS as f64, PROBE_MAX_MS as f64);
        let span_ms = if self.last_cut_ms.is_some() {
            rtt_span_ms.max(RATE_STEP_EVIDENCE_MS as f64)
        } else {
            rtt_span_ms
        };
        self.epoch = Some(Epoch {
            started_ms: input.now_ms,
            accepted_bytes: self.last_accepted_bytes,
            span_ms,
        });
    }

    fn congestion_authorized(
        &mut self,
        queue: &queue::QueueVerdict,
        epoch_result: Option<EpochResult>,
        queue_delay_ms: f64,
    ) -> bool {
        let Some(last_cut_ms) = self.last_cut_ms else {
            return queue.congested;
        };
        let Some(result) = epoch_result else {
            return false;
        };
        if !result.proven || !result.fully_used {
            self.clear_repeat_confirmation();
            return false;
        }
        let Some(sample_ms) = queue.new_causal_sample_ms else {
            return false;
        };
        let settle_end = last_cut_ms.saturating_add(CUT_SETTLE_MS);
        if sample_ms < settle_end || !queue.window_high {
            self.clear_repeat_confirmation();
            return false;
        }
        if self
            .repeat_last_queue_delay_ms
            .is_some_and(|previous| queue_delay_ms < previous)
        {
            self.clear_repeat_confirmation();
        }
        if self
            .repeat_last_sample_ms
            .is_none_or(|previous| sample_ms.saturating_sub(previous) > REPEAT_SAMPLE_MAX_GAP_MS)
        {
            self.repeat_high_since_ms = Some(sample_ms);
        }
        self.repeat_last_sample_ms = Some(sample_ms);
        self.repeat_last_queue_delay_ms = Some(queue_delay_ms);
        let since = *self.repeat_high_since_ms.get_or_insert(sample_ms);
        sample_ms.saturating_sub(since) >= REPEAT_CONGESTION_MS
    }

    pub(super) const fn clear_repeat_confirmation(&mut self) {
        self.repeat_high_since_ms = None;
        self.repeat_last_sample_ms = None;
        self.repeat_last_queue_delay_ms = None;
    }
    pub const fn rate_bps(&self) -> f64 {
        self.rate_bps
    }
    pub const fn phase(&self) -> WireRatePhase {
        self.phase
    }
}

#[cfg(test)]
#[path = "wire_rate/edge_tests.rs"]
mod edge_tests;
#[cfg(test)]
#[path = "wire_rate/hysteresis_tests.rs"]
mod hysteresis_tests;
#[cfg(test)]
#[path = "wire_rate/rate_step_tests.rs"]
mod rate_step_tests;
#[cfg(test)]
#[path = "wire_rate/tests.rs"]
mod tests;
