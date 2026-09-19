//! Pure per-link health transitions; runtime integration is deliberately separate.
// allow: SIZE_OK — keep the single transition function and its private timing state together;
// queue entry and clearance must be reviewed against the same precedence table.

use core::time::Duration;

use super::route::RouteHealth;

#[path = "health_constants.rs"]
mod constants;
pub use constants::HealthConstants;

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
    /// Unknown never invents absence or clears a previously observed absence.
    pub route_health: RouteHealth,
    pub attempts_since_proof: u32,
    /// None is unknown, not infinite age. Before first proof, supply time since
    /// registration/first attempt so an unproven active link can still stall.
    pub proof_age_ms: Option<u64>,
    /// Since last keepalive reply, or first accepted keepalive before any reply.
    /// None means no keepalive has been sent on this socket, not infinite age.
    pub keepalive_silence_ms: Option<u64>,
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
    /// Maximum scheduled interval between health observations; zero for immediate evaluation.
    /// Completed train acquisition and waiting for the next observation are distinct ages.
    pub observation_interval_ms: u64,
    pub now_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    pub from: HealthState,
    pub to: HealthState,
    pub at_ms: u64,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct RecoveryRounds {
    pub count: u32,
    pub started_ms: Option<u64>,
}

#[derive(Clone, Debug)]
struct QueueEntryPending {
    since_ms: u64,
    required_ms: f64,
}

/// Mutate only through step methods: private fields preserve dwell and timestamp invariants.
#[derive(Clone, Debug)]
pub struct HealthMachine {
    state: HealthState,
    entered_at_ms: u64,
    dwell_multiplier: u32,
    last_transition_ms: u64,
    clear_since_ms: Option<u64>,
    queue_entry_pending: Option<QueueEntryPending>,
    loss_latched: bool,
    route_latched: bool,
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
            queue_entry_pending: None,
            loss_latched: matches!(state, HealthState::Degraded),
            route_latched: false,
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

    /// Latched loss cause survives stale cohorts; ablation must not infer it from EWMA alone.
    pub const fn loss_latched(&self) -> bool {
        self.loss_latched
    }

    pub const fn route_latched(&self) -> bool {
        self.route_latched
    }

    /// Applies one observation. Hard failure outranks stall, which outranks degradation.
    pub fn step(&mut self, s: &HealthSignals, k: &HealthConstants) -> Option<Transition> {
        self.step_with_originals(s, k, RecoveryRounds::default())
    }

    pub(crate) fn step_with_originals(
        &mut self,
        s: &HealthSignals,
        k: &HealthConstants,
        originals: RecoveryRounds,
    ) -> Option<Transition> {
        use HealthState::{Degraded, Down, Healthy, Rejoining, Stalled};

        let tau = k.stall_tau(s.srtt_ms);
        let data_overdue = s.proof_age_ms.is_some_and(|age| elapsed_ms(age, 0) >= tau);
        let keepalive_overdue = s
            .keepalive_silence_ms
            .is_some_and(|age| age >= super::keepalive::SILENCE_MS);
        // Control silence covers idle/starved links; healthy echoes cannot veto a
        // DATA-only blackhole, and recent DATA proof still establishes liveness.
        let stalled = (s.attempts_since_proof >= k.stall_attempts && data_overdue)
            || (keepalive_overdue && (data_overdue || s.proof_age_ms.is_none()));
        let loss_entered = s.loss_cohort_ok && s.loss_ewma.is_some_and(|loss| loss >= k.loss_enter);
        self.route_latched = match s.route_health {
            RouteHealth::NoDefaultRoute => true,
            RouteHealth::DefaultRoutePresent => false,
            RouteHealth::Unknown => self.route_latched,
        };
        let queue_above_enter = s.queue_delay_ms >= k.queue_enter(s.slow_min_rtt_ms);
        let queue_entered = if matches!(self.state, Healthy | Rejoining) && queue_above_enter {
            let pending = self.queue_entry_pending.get_or_insert(QueueEntryPending {
                since_ms: s.now_ms,
                required_ms: tau,
            });
            elapsed_ms(s.now_ms, pending.since_ms) >= pending.required_ms
        } else {
            self.queue_entry_pending = None;
            false
        };
        let degraded = self.route_latched || loss_entered || queue_entered;
        let next = if !s.connected || !s.socket_valid || !s.iface_present {
            Down
        } else {
            match self.state {
                Healthy | Degraded | Rejoining if stalled => Stalled,
                Healthy | Rejoining if degraded => Degraded,
                Healthy => Healthy,
                Down => Rejoining,
                Stalled => {
                    let recent = |started_ms: Option<u64>| {
                        started_ms.is_some_and(|start| {
                            start >= self.entered_at_ms
                                && start <= s.now_ms
                                && elapsed_ms(s.now_ms, start)
                                    <= k.rejoin_span(s.srtt_ms, s.held_links)
                                        + elapsed_ms(s.observation_interval_ms, 0)
                        })
                    };
                    if (s.probe_rounds_ok >= k.rejoin_rounds && recent(s.probe_rounds_started_ms))
                        || (originals.count >= k.rejoin_rounds && recent(originals.started_ms))
                    {
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
                    let cleared = !self.route_latched
                        && (!self.loss_latched || loss.is_some_and(|v| v <= k.loss_clear))
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
            Degraded | Stalled | Down => self.queue_entry_pending = None,
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
#[path = "../tests/health_keepalive_tests.rs"]
mod keepalive_tests;
#[cfg(test)]
#[path = "../tests/health_queue_entry_tests.rs"]
mod queue_entry_tests;
#[cfg(test)]
#[path = "../tests/health_recovery_tests.rs"]
mod recovery_tests;
#[cfg(test)]
#[path = "../tests/health_route_tests.rs"]
mod route_tests;
#[cfg(test)]
#[path = "../tests/health_transition_tests.rs"]
mod transition_tests;
