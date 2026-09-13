//! Pure per-link health transitions; runtime integration is deliberately separate.

use core::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Stalled,
    Rejoining,
    Down,
}

impl HealthState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Stalled => "stalled",
            Self::Rejoining => "rejoining",
            Self::Down => "down",
        }
    }
}

/// Caller-owned observations: finite nonnegative RTT/delay, loss fractions in [0, 1],
/// and timestamps in the same monotonic millisecond domain as `now_ms`.
#[derive(Clone, Copy, Debug)]
pub struct HealthSignals {
    pub connected: bool,
    pub socket_valid: bool,
    pub iface_present: bool,
    pub attempts_since_proof: u32,
    /// None is unknown, not infinite age. Before first proof, supply time since
    /// registration/first attempt so an unproven active link can still stall.
    pub proof_age_ms: Option<u64>,
    pub srtt_ms: Option<f64>,
    pub loss_ewma: Option<f64>,
    /// True only for a qualifying normal-DATA cohort; probes never set this.
    pub loss_cohort_ok: bool,
    pub last_cohort_ms: Option<u64>,
    /// Unacknowledged fraction over the last `rejoin_rounds` complete probe trains.
    /// None until that evidence exists; a partial train is not zero loss.
    pub probe_loss: Option<f64>,
    pub queue_delay_ms: f64,
    pub slow_min_rtt_ms: f64,
    pub probe_rounds_ok: u32,
    /// Start of the oldest train contributing to `probe_rounds_ok`, not state entry.
    /// The probe tracker must expire/reset the count and this timestamp together.
    pub probe_rounds_started_ms: Option<u64>,
    pub held_links: u32,
    pub now_ms: u64,
}

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
        Self {
            stall_attempts: 32,
            loss_enter: 0.10,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    pub from: HealthState,
    pub to: HealthState,
    pub at_ms: u64,
}

/// Mutate only through `step`: private fields preserve dwell and timestamp invariants.
#[derive(Clone, Debug)]
pub struct HealthMachine {
    state: HealthState,
    entered_at_ms: u64,
    dwell_multiplier: u32,
    last_transition_ms: u64,
    clear_since_ms: Option<u64>,
    loss_latched: bool,
    ramp_tau_ms: f64,
    ramp_train_ms: f64,
}

impl HealthMachine {
    /// Fresh runtime links start Down. Explicit state initialization also supports
    /// isolated policy evaluation; direct Rejoining uses default, unknown-RTT timing.
    pub const fn new(state: HealthState, now_ms: u64) -> Self {
        Self {
            state,
            entered_at_ms: now_ms,
            dwell_multiplier: 1,
            last_transition_ms: now_ms,
            clear_since_ms: None,
            loss_latched: matches!(state, HealthState::Degraded),
            ramp_tau_ms: 3000.0,
            ramp_train_ms: 1000.0,
        }
    }
    pub const fn state(&self) -> HealthState {
        self.state
    }
    pub const fn entered_at_ms(&self) -> u64 {
        self.entered_at_ms
    }
    pub const fn dwell_multiplier(&self) -> u32 {
        self.dwell_multiplier
    }
    pub const fn last_transition_ms(&self) -> u64 {
        self.last_transition_ms
    }

    /// Applies one observation. Hard failure outranks stall, which outranks degradation.
    pub fn step(&mut self, s: &HealthSignals, k: &HealthConstants) -> Option<Transition> {
        use HealthState::{Degraded, Down, Healthy, Rejoining, Stalled};

        let tau = k.stall_tau(s.srtt_ms);
        let stalled = s.attempts_since_proof >= k.stall_attempts
            && s.proof_age_ms.is_some_and(|age| elapsed_ms(age, 0) >= tau);
        let loss_entered = s.loss_cohort_ok && s.loss_ewma.is_some_and(|loss| loss >= k.loss_enter);
        let degraded = loss_entered || s.queue_delay_ms >= k.queue_enter(s.slow_min_rtt_ms);
        let next = if !s.connected || !s.socket_valid || !s.iface_present {
            Down
        } else {
            match self.state {
                Healthy | Degraded | Rejoining if stalled => Stalled,
                Healthy | Rejoining if degraded => Degraded,
                Healthy => Healthy,
                Down => Rejoining,
                Stalled => {
                    let recent = s.probe_rounds_started_ms.is_some_and(|start| {
                        start >= self.entered_at_ms
                            && start <= s.now_ms
                            && elapsed_ms(s.now_ms, start) <= k.rejoin_span(s.srtt_ms, s.held_links)
                    });
                    if s.probe_rounds_ok >= k.rejoin_rounds && recent {
                        Rejoining
                    } else {
                        Stalled
                    }
                }
                Degraded => {
                    self.loss_latched |= loss_entered;
                    let fresh = s
                        .last_cohort_ms
                        .is_some_and(|at| s.now_ms.saturating_sub(at) < k.loss_stale_after_ms);
                    let loss = match s.loss_ewma {
                        Some(value) if fresh => Some(value),
                        Some(_) | None => s.probe_loss,
                    };
                    let cleared = (!self.loss_latched || loss.is_some_and(|v| v <= k.loss_clear))
                        && s.queue_delay_ms <= k.queue_clear(s.slow_min_rtt_ms);
                    if cleared {
                        let since = *self.clear_since_ms.get_or_insert(s.now_ms);
                        if elapsed_ms(s.now_ms, since) >= tau {
                            Rejoining
                        } else {
                            Degraded
                        }
                    } else {
                        self.clear_since_ms = None;
                        Degraded
                    }
                }
                Rejoining => {
                    if elapsed_ms(s.now_ms, self.entered_at_ms)
                        >= self.ramp_span_effective(s.held_links)
                    {
                        Healthy
                    } else {
                        Rejoining
                    }
                }
            }
        };
        if next == self.state {
            return None;
        }
        let transition = Transition {
            from: self.state,
            to: next,
            at_ms: s.now_ms,
        };
        if matches!((self.state, next), (Rejoining, Stalled | Degraded)) {
            self.dwell_multiplier =
                (self.dwell_multiplier * 2).min(k.dwell_backoff_max.clamp(1, 16));
        }
        match next {
            Healthy => self.dwell_multiplier = 1,
            Rejoining => {
                self.ramp_tau_ms = tau;
                self.ramp_train_ms = k.train_period_ms(1);
            }
            Degraded | Stalled | Down => {}
        }
        self.loss_latched = matches!(next, Degraded) && loss_entered;
        self.clear_since_ms = None;
        self.state = next;
        self.entered_at_ms = s.now_ms;
        self.last_transition_ms = s.now_ms;
        Some(transition)
    }

    fn ramp_span_effective(&self, held_links: u32) -> f64 {
        (2.0 * self.ramp_tau_ms).max(f64::from(held_links) * self.ramp_train_ms)
            * f64::from(self.dwell_multiplier)
    }

    /// Entry-time RTT/tuning are frozen for the ramp; held-link count stays live.
    pub fn ramp_multiplier(&self, now_ms: u64, held_links: u32) -> f64 {
        match self.state {
            HealthState::Rejoining => {
                let progress =
                    elapsed_ms(now_ms, self.entered_at_ms) / self.ramp_span_effective(held_links);
                0.05 + 0.95 * progress.clamp(0.0, 1.0)
            }
            HealthState::Healthy
            | HealthState::Degraded
            | HealthState::Stalled
            | HealthState::Down => 1.0,
        }
    }
}

fn elapsed_ms(now_ms: u64, then_ms: u64) -> f64 {
    Duration::from_millis(now_ms.saturating_sub(then_ms)).as_secs_f64() * 1000.0
}

#[cfg(test)]
#[path = "../tests/health_invariant_tests.rs"]
mod invariant_tests;
#[cfg(test)]
#[path = "../tests/health_recovery_tests.rs"]
mod recovery_tests;
#[cfg(test)]
#[path = "../tests/health_transition_tests.rs"]
mod transition_tests;
