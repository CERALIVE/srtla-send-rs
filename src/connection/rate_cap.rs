//! Pure per-link delivered-rate policy; no send/selection or legacy window effects.

use super::delivery::DeliveryLedger;

pub const BOOTSTRAP_BPS: f64 = 1_000_000.0;
pub const HAI_MAX_VELOCITY_MS_PER_UPDATE: f64 = 0.1;
const BACKOFF_LOSS: f64 = 0.015;
const RECOVERY_TICKS: u32 = 5;
const DRAIN_GUARD_TICKS: u32 = 10;
const IDLE_SUSPEND_TICKS: u64 = 10;
const UNCONGESTIVE_TICKS: u64 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClimbMode {
    Normal,
    Hai,
    /// Remaining growth ticks AFTER the tick reported by this state.
    FastRecovery {
        ticks_left: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RateState {
    Bootstrap,
    Climbing { sub: ClimbMode },
    Holding,
    BackingOff,
    Drain,
}

/// Same-link tracker snapshot. RTT/delay/jitter are finite nonnegative milliseconds,
/// velocity is finite ms/Kalman update, loss is a fraction in [0,1] or unknown.
/// `now_ms` is monotonic ledger time, NOT wall-clock telemetry time.
#[derive(Clone, Copy, Debug)]
pub struct RateSignals {
    pub now_ms: u64,
    pub srtt_ms: f64,
    pub rtt_min_ms: f64,
    pub loss_ewma: Option<f64>,
    pub queue_delay_ms: f64,
    pub velocity_ms_per_update: f64,
    pub jitter_ms: f64,
}

/// Owned by one link/socket generation; reconstruct alongside its delivery ledger.
/// Private state prevents external target/counter mutation from bypassing policy.
#[derive(Debug)]
pub struct RateCap {
    state: RateState,
    target_bps: f64,
    delivered_bps: f64,
    loss_entry: Option<f64>,
    backoff_ticks: u32,
    uncongestive_until_tick: Option<u64>,
    drain_episode_guard: u32,
    tick: u64,
    stale_since: Option<u64>,
    drain_episode: bool,
    recovery_ticks_left: u32,
    rtt_min_ms: f64,
}

impl Default for RateCap {
    fn default() -> Self {
        Self {
            state: RateState::Bootstrap,
            target_bps: BOOTSTRAP_BPS,
            delivered_bps: 0.0,
            loss_entry: None,
            backoff_ticks: 0,
            uncongestive_until_tick: None,
            drain_episode_guard: 0,
            tick: 0,
            stale_since: None,
            drain_episode: false,
            recovery_ticks_left: 0,
            rtt_min_ms: 0.0,
        }
    }
}

impl RateCap {
    /// Call exactly once per 1s housekeeping tick, including idle links. No clock
    /// reads or catch-up iterations: `now_ms` only selects the ledger's 2s window.
    /// Reading the ledger here makes transmitted bitrate an impossible rate input.
    pub fn tick(&mut self, delivery: &DeliveryLedger, signals: &RateSignals) {
        self.tick = self.tick.saturating_add(1);
        self.drain_episode_guard = self.drain_episode_guard.saturating_sub(1);
        if self
            .uncongestive_until_tick
            .is_some_and(|until| self.tick >= until)
        {
            self.uncongestive_until_tick = None;
        }
        self.rtt_min_ms = signals.rtt_min_ms;
        self.delivered_bps = delivery.delivered_bps(signals.now_ms);
        if self.delivered_bps == 0.0 {
            self.stale_since.get_or_insert(self.tick);
            self.loss_entry = None;
            self.backoff_ticks = 0;
            // Freeze the active mode/episode and recovery budget too: no idle ramp
            // and no fictitious backoff efficacy observation without delivery.
            return;
        }
        self.stale_since = None;
        match self.state {
            RateState::Bootstrap => {
                self.target_bps = BOOTSTRAP_BPS.max(self.delivered_bps);
                self.state = RateState::Climbing {
                    sub: ClimbMode::Normal,
                };
                return;
            }
            RateState::Climbing { .. }
            | RateState::Holding
            | RateState::BackingOff
            | RateState::Drain => {}
        }

        // Observe the result of THREE completed cuts, before considering a fourth.
        if self.backoff_ticks == 3
            && let (Some(entry), Some(loss)) = (self.loss_entry, signals.loss_ewma)
            && loss >= 0.8 * entry
        {
            self.uncongestive_until_tick = Some(self.tick.saturating_add(UNCONGESTIVE_TICKS));
        }
        let measured_rtt = signals.rtt_min_ms > 0.0 && signals.srtt_ms > 0.0;
        let drain = measured_rtt
            && signals.srtt_ms >= 2.0 * signals.rtt_min_ms
            && signals.loss_ewma == Some(0.0);
        let drain_entry = drain && !self.drain_episode;
        self.drain_episode = drain;

        if !self.is_uncongestive()
            && signals.loss_ewma.is_some_and(|loss| loss >= BACKOFF_LOSS)
            && self.delivered_bps >= 0.3 * self.target_bps
        {
            if self.backoff_ticks == 0 {
                self.loss_entry = signals.loss_ewma;
            }
            let backoff = crate::adaptive_env::tuning().ratecap_loss_backoff;
            self.target_bps =
                (backoff * self.target_bps).max(self.delivered_bps.min(self.target_bps));
            self.backoff_ticks = self.backoff_ticks.saturating_add(1);
            self.recovery_ticks_left = RECOVERY_TICKS;
            self.state = RateState::BackingOff;
            return;
        }
        self.loss_entry = None;
        self.backoff_ticks = 0;
        if drain {
            if drain_entry && self.drain_episode_guard == 0 && !self.is_uncongestive() {
                self.target_bps *= 0.75;
                self.drain_episode_guard = DRAIN_GUARD_TICKS;
            }
            self.recovery_ticks_left = RECOVERY_TICKS;
            self.state = RateState::Drain;
            return;
        }
        if measured_rtt && signals.srtt_ms > 1.5 * signals.rtt_min_ms {
            self.state = RateState::Holding;
            return;
        }
        let sub = if self.recovery_ticks_left > 0 {
            ClimbMode::FastRecovery {
                ticks_left: self.recovery_ticks_left,
            }
        } else if measured_rtt
            && signals.velocity_ms_per_update.abs() <= HAI_MAX_VELOCITY_MS_PER_UPDATE
            && signals.jitter_ms <= 0.1 * signals.srtt_ms
            && signals.queue_delay_ms == 0.0
        {
            ClimbMode::Hai
        } else {
            ClimbMode::Normal
        };
        let (factor, sub) = match sub {
            ClimbMode::Normal => (1.02, ClimbMode::Normal),
            ClimbMode::Hai => (1.06, ClimbMode::Hai),
            ClimbMode::FastRecovery { ticks_left } => {
                self.recovery_ticks_left = ticks_left - 1;
                (
                    1.04,
                    ClimbMode::FastRecovery {
                        ticks_left: self.recovery_ticks_left,
                    },
                )
            }
        };
        self.target_bps = (self.target_bps * factor).min(f64::MAX);
        self.state = RateState::Climbing { sub };
    }

    pub const fn state(&self) -> RateState {
        self.state
    }
    pub const fn target_bps(&self) -> f64 {
        self.target_bps
    }
    pub const fn delivered_bps(&self) -> f64 {
        self.delivered_bps
    }

    pub fn is_uncongestive(&self) -> bool {
        self.uncongestive_until_tick
            .is_some_and(|until| self.tick < until)
    }

    pub fn bdp_cap_suspended(&self) -> bool {
        self.stale_since
            .is_some_and(|since| self.tick.saturating_sub(since) >= IDLE_SUSPEND_TICKS - 1)
    }

    /// `u32::MAX` denotes suspension; otherwise return the floor-32 BDP ranking cap.
    pub fn bdp_cap_packets(&self, rtt_min_ms: f64) -> u32 {
        if self.bdp_cap_suspended() {
            return u32::MAX;
        }
        let packets = self.target_bps * (rtt_min_ms / 1000.0) / 8.0 * 1.5 / 1316.0;
        // Rust's saturating float-to-int cast intentionally floors nonnegative
        // packet counts and bounds overflow at u32::MAX; the stall floor is 32.
        (packets as u32).max(32)
    }

    /// Uses the latest tick's baseline. Every finite load retains a positive rank.
    pub fn soft_cap_multiplier(&self, in_flight: u32) -> f64 {
        let cap = self.bdp_cap_packets(self.rtt_min_ms);
        if in_flight <= cap {
            1.0
        } else {
            f64::from(cap) / f64::from(in_flight)
        }
    }
}
