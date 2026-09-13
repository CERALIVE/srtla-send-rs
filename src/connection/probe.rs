//! Owned bond-wide probe pacing; selection policy supplies eligibility in Todo 22.

mod log;
mod send;
pub use log::ProbeLog;
pub use send::{ProbeEmission, ProbeOpportunity};

use super::health::HealthState;

pub const PROBE_MAX_PPS: u32 = 10;
pub const PROBE_TRAIN_LEN: u8 = 10;
pub const PROBE_LOG_CAPACITY: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct ProbeTarget {
    pub conn_id: u64,
    pub socket_generation: u32,
    pub health: HealthState,
    pub deadline_held: bool,
    pub sole_carrier: bool,
    pub srtt_ms: u64,
}

impl ProbeTarget {
    pub const fn eligible(self) -> bool {
        let soft = match self.health {
            HealthState::Stalled | HealthState::Degraded => true,
            HealthState::Down | HealthState::Healthy | HealthState::Rejoining => false,
        };
        soft && self.deadline_held && !self.sole_carrier
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProbeTrain {
    pub id: u64,
    pub started_ms: u64,
    pub deadline_ms: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct ProbeDispatch {
    pub conn_id: u64,
    pub socket_generation: u32,
    pub train: ProbeTrain,
}

/// A one-token bucket deliberately forbids bursts, even after idle. The caller
/// offers only DATA already forwarded normally; no selector history is borrowed.
#[derive(Debug)]
pub struct ProbeScheduler {
    rate: u32,
    credit: u64,
    last_ms: Option<u64>,
    active: Option<(ProbeDispatch, u8)>,
    last_link: Option<u64>,
    next_train: u64,
}

impl Default for ProbeScheduler {
    fn default() -> Self {
        Self::with_rate(PROBE_MAX_PPS)
    }
}

impl ProbeScheduler {
    pub const fn with_rate(rate: u32) -> Self {
        Self {
            rate: if rate > PROBE_MAX_PPS {
                PROBE_MAX_PPS
            } else {
                rate
            },
            credit: 1000,
            last_ms: None,
            active: None,
            last_link: None,
            next_train: 0,
        }
    }

    pub fn next(&mut self, targets: &[ProbeTarget], now_ms: u64) -> Option<ProbeDispatch> {
        if self.rate == 0 {
            return None;
        }
        if let Some(last) = self.last_ms {
            self.credit = self
                .credit
                .saturating_add(
                    now_ms
                        .saturating_sub(last)
                        .saturating_mul(u64::from(self.rate)),
                )
                .min(1000);
        }
        self.last_ms = Some(now_ms);
        if self.credit < 1000 {
            return None;
        }
        if let Some((active, _)) = self.active
            && (now_ms > active.train.deadline_ms
                || !targets.iter().any(|t| {
                    t.conn_id == active.conn_id
                        && t.socket_generation == active.socket_generation
                        && t.eligible()
                }))
        {
            self.active = None;
        }
        if self.active.is_none() {
            // Stable connection IDs retain train rotation across ips-file reorders.
            let start = self
                .last_link
                .and_then(|id| targets.iter().position(|t| t.conn_id == id))
                .map_or(0, |i| i + 1);
            let target = (0..targets.len())
                .map(|n| &targets[(start + n) % targets.len()])
                .find(|t| t.eligible())?;
            let held = u64::try_from(targets.iter().filter(|t| t.eligible()).count()).ok()?;
            self.next_train = self.next_train.wrapping_add(1);
            self.active = Some((
                ProbeDispatch {
                    conn_id: target.conn_id,
                    socket_generation: target.socket_generation,
                    train: ProbeTrain {
                        id: self.next_train,
                        started_ms: now_ms,
                        deadline_ms: now_ms
                            .saturating_add(held.saturating_mul(2000))
                            .saturating_add(target.srtt_ms),
                    },
                },
                0,
            ));
        }
        let (dispatch, sent) = self.active.as_mut()?;
        *sent += 1;
        let result = *dispatch;
        if *sent == PROBE_TRAIN_LEN {
            self.last_link = Some(result.conn_id);
            self.active = None;
        }
        self.credit -= 1000;
        Some(result)
    }
}
