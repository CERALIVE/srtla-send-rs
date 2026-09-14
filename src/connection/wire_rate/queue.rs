use std::time::Duration;

use super::{FAST_QUEUE_RISE_MS, FAST_QUEUE_SAMPLES, WireRateInput, WireRttSample};
use crate::connection::health::HealthConstants;

#[derive(Debug, Default)]
pub(super) struct QueueEvidence {
    last_rtt_ms: Option<u64>,
    high: [f64; FAST_QUEUE_SAMPLES],
    high_count: usize,
    clear_count: usize,
    pub clear: bool,
    pub clear_since_ms: Option<u64>,
}

pub(super) struct QueueVerdict {
    pub congested: bool,
    pub window_high: bool,
    pub new_causal_sample_ms: Option<u64>,
    pub current_rate_clear: bool,
    pub clear: bool,
    pub drained: bool,
}

impl QueueEvidence {
    pub fn newer(&self, sample: Option<WireRttSample>) -> bool {
        sample.is_some_and(|rtt| self.last_rtt_ms.is_none_or(|last| rtt.observed_ms > last))
    }

    pub fn begin_drain(&mut self) {
        self.clear_count = 0;
    }

    pub fn observe(
        &mut self,
        input: &WireRateInput,
        generation_started_ms: u64,
        causal_after_ms: u64,
    ) -> QueueVerdict {
        let sample = input.latest_rtt.filter(|rtt| {
            rtt.observed_ms >= generation_started_ms && rtt.observed_ms <= input.now_ms
        });
        let enter = HealthConstants::default().queue_clear(input.slow_min_rtt_ms);
        let raw = sample.map(|rtt| ((rtt.rtt_ms - input.slow_min_rtt_ms) / 2.0).max(0.0));
        let persistent = sample.is_some() && input.queue_delay_ms >= enter;
        self.clear = raw.is_some_and(|q| q <= 0.5 * enter) && input.queue_delay_ms <= 0.5 * enter;
        if self.clear {
            self.clear_since_ms.get_or_insert(input.now_ms);
        } else {
            self.clear_since_ms = None;
        }
        let mut fast = false;
        let observed = input
            .latest_rtt
            .filter(|rtt| rtt.observed_ms <= input.now_ms);
        let distinct = self.newer(observed);
        let causal_sample =
            sample.filter(|rtt| accepted_send_ms(*rtt).is_some_and(|sent| sent >= causal_after_ms));
        let current_rate_clear = self.clear
            && causal_sample.is_some()
            && self
                .clear_since_ms
                .is_some_and(|since| since <= causal_after_ms);
        let new_causal_sample_ms = distinct
            .then_some(causal_sample)
            .flatten()
            .map(|rtt| rtt.observed_ms);
        if distinct {
            self.last_rtt_ms = observed.map(|rtt| rtt.observed_ms);
            if let Some(raw) = raw {
                if raw >= enter {
                    self.high.rotate_left(1);
                    self.high[FAST_QUEUE_SAMPLES - 1] = raw;
                    self.high_count = (self.high_count + 1).min(FAST_QUEUE_SAMPLES);
                    fast = self.high_count == FAST_QUEUE_SAMPLES
                        && self.high[FAST_QUEUE_SAMPLES - 1] - self.high[0] >= FAST_QUEUE_RISE_MS;
                } else {
                    self.high_count = 0;
                }
            }
            self.clear_count = if self.clear {
                (self.clear_count + 1).min(FAST_QUEUE_SAMPLES)
            } else {
                0
            };
        }
        if !self.clear {
            self.clear_count = 0;
        }
        QueueVerdict {
            congested: fast || persistent,
            window_high: persistent,
            new_causal_sample_ms,
            current_rate_clear,
            clear: self.clear,
            drained: self.clear_count == FAST_QUEUE_SAMPLES,
        }
    }
}

fn accepted_send_ms(sample: WireRttSample) -> Option<u64> {
    let elapsed = Duration::try_from_secs_f64(sample.rtt_ms / 1000.0).ok()?;
    let elapsed_ms = u64::try_from(elapsed.as_millis()).ok()?;
    Some(sample.observed_ms.saturating_sub(elapsed_ms))
}
