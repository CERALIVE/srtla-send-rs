use super::{
    BRACKET_RATIO, CAPACITY_REPROBE_MS, EpochResult, MIN_RATE_BPS, REPROBE_GAIN, SEARCH_GAIN,
    WireRateEstimator, WireRateInput, WireRatePhase,
};

impl WireRateEstimator {
    pub(super) fn congestion(&mut self, input: &WireRateInput) {
        self.blocked = Some(
            self.blocked
                .map_or(self.rate_bps, |old| old.min(self.rate_bps)),
        );
        self.highest_clean = self.highest_clean.filter(|clean| *clean < self.rate_bps);
        self.rate_bps = self
            .highest_clean
            .unwrap_or((self.rate_bps / 2.0).max(MIN_RATE_BPS));
        self.phase = WireRatePhase::Draining;
        self.capacity_probe = false;
        self.full_clear_since_ms = None;
        self.last_cut_ms = Some(input.now_ms);
        self.clear_repeat_confirmation();
        self.queue.begin_drain();
        self.restart_epoch(input);
    }

    pub(super) fn resume_search(&mut self, input: &WireRateInput) {
        let old_rate = self.rate_bps;
        match (self.highest_clean, self.blocked) {
            (Some(low), Some(high)) if high / low <= BRACKET_RATIO => {
                self.rate_bps = low;
                self.phase = WireRatePhase::CapacityHeld;
            }
            (Some(low), Some(high)) => {
                self.rate_bps = (low * high).sqrt();
                self.phase = WireRatePhase::Searching;
            }
            (Some(_), None) | (None, Some(_)) | (None, None) => {
                self.phase = WireRatePhase::Searching
            }
        }
        self.full_clear_since_ms = None;
        if self.rate_bps != old_rate {
            self.clear_repeat_confirmation();
        }
        self.restart_epoch(input);
    }

    pub(super) fn finish_epoch(&mut self, input: &WireRateInput, result: EpochResult) {
        if !result.proven {
            self.full_clear_since_ms = None;
            self.restart_epoch(input);
            return;
        }
        let old_rate = self.rate_bps;
        match self.phase {
            WireRatePhase::Searching => {
                if !result.fully_used {
                    self.phase = WireRatePhase::DemandLimited;
                    self.capacity_probe = false;
                } else if result.clean {
                    self.highest_clean = Some(self.rate_bps);
                    if self.capacity_probe {
                        self.blocked = self.blocked.filter(|high| *high > self.rate_bps);
                        self.phase = WireRatePhase::CapacityHeld;
                        self.capacity_probe = false;
                    } else {
                        match self.blocked {
                            Some(high) if high / self.rate_bps <= BRACKET_RATIO => {
                                self.phase = WireRatePhase::CapacityHeld
                            }
                            Some(high) => self.rate_bps = (self.rate_bps * high).sqrt(),
                            None => self.rate_bps *= SEARCH_GAIN,
                        }
                    }
                }
                self.full_clear_since_ms = None;
            }
            WireRatePhase::DemandLimited => {
                if result.fully_used && result.clean {
                    self.highest_clean = Some(self.rate_bps);
                    self.blocked = self.blocked.filter(|high| *high > self.rate_bps);
                    self.rate_bps *= SEARCH_GAIN;
                    self.phase = WireRatePhase::Searching;
                }
            }
            WireRatePhase::CapacityHeld => {
                if result.fully_used && result.clean {
                    let start = self.epoch.map_or(input.now_ms, |epoch| epoch.started_ms);
                    let start = start.max(self.queue.clear_since_ms.unwrap_or(input.now_ms));
                    let since = *self.full_clear_since_ms.get_or_insert(start);
                    if input.now_ms.saturating_sub(since) >= CAPACITY_REPROBE_MS {
                        self.rate_bps *= REPROBE_GAIN;
                        self.capacity_probe = true;
                        self.phase = WireRatePhase::Searching;
                        self.full_clear_since_ms = None;
                    }
                } else {
                    self.full_clear_since_ms = None;
                }
            }
            WireRatePhase::Draining => {}
        }
        if self.rate_bps != old_rate {
            self.clear_repeat_confirmation();
        }
        self.restart_epoch(input);
    }
}
