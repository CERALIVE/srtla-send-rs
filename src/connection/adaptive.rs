use std::time::Duration;

/// Socket-scoped admission history: follows the connection, never an ips-file index.
#[derive(Debug, Default)]
pub struct AdaptiveLinkState {
    /// Last shared admission/ranking pass; None means held out, never an estimated weight.
    pub(crate) weight: Option<SelectionWeight>,
    pub deadline: DeadlineGate,
    pub sole_failed_until_proof: bool,
    pub last_sole_election_ms: Option<u64>,
    generation: u32,
    observed_proof_ms: Option<u64>,
}

/// Cache the base alongside the multiplier so a later stats read cannot mix packet epochs.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SelectionWeight {
    pub base_score: i32,
    pub quality_multiplier: f64,
    pub effective_multiplier: f64,
}

impl SelectionWeight {
    pub fn score(self) -> f64 {
        f64::from(self.base_score) * self.effective_multiplier
    }
}

impl AdaptiveLinkState {
    pub fn observe(&mut self, generation: u32, proof_ms: Option<u64>) {
        if self.generation != generation {
            *self = Self {
                generation,
                ..Self::default()
            };
        }
        if self.observed_proof_ms != proof_ms {
            self.sole_failed_until_proof = false;
            self.observed_proof_ms = proof_ms;
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum DeadlineGate {
    #[default]
    Admitted,
    Held {
        clear_since_ms: Option<u64>,
    },
}

impl DeadlineGate {
    pub fn update(&mut self, predicted_ms: f64, budget: DeadlineBudget) -> bool {
        let next = match *self {
            Self::Admitted if predicted_ms > 0.5 * f64::from(budget.latency_ms) => Self::Held {
                clear_since_ms: None,
            },
            Self::Admitted => Self::Admitted,
            Self::Held { clear_since_ms } if predicted_ms < 0.4 * f64::from(budget.latency_ms) => {
                let since = clear_since_ms.unwrap_or(budget.now_ms);
                if elapsed_ms(budget.now_ms, since) >= budget.tau_ms {
                    Self::Admitted
                } else {
                    Self::Held {
                        clear_since_ms: Some(since),
                    }
                }
            }
            Self::Held { .. } => Self::Held {
                clear_since_ms: None,
            },
        };
        *self = next;
        matches!(next, Self::Held { .. })
    }
}

#[derive(Clone, Copy)]
pub struct DeadlineBudget {
    pub latency_ms: u32,
    pub tau_ms: f64,
    pub now_ms: u64,
}

pub(crate) fn elapsed_ms(now: u64, then: u64) -> f64 {
    Duration::from_millis(now.saturating_sub(then)).as_secs_f64() * 1000.0
}
