use std::collections::VecDeque;

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use super::{PROBE_LOG_CAPACITY, PROBE_TRAIN_LEN, ProbeTrain};

#[derive(Debug)]
struct Train {
    spec: ProbeTrain,
    sequences: SmallVec<i32, 10>,
    acknowledged: u8,
    expired: bool,
}

/// Socket-lifetime probe evidence. The owner resets this alongside DeliveryLedger;
/// ACK callers must fence the captured reader generation before accessing it.
#[derive(Debug)]
pub struct ProbeLog {
    pub(crate) probe_log: FxHashMap<i32, u64>,
    lru: VecDeque<i32>,
    trains: VecDeque<Train>,
    pub probes_sent: u64,
}

impl Default for ProbeLog {
    fn default() -> Self {
        Self {
            probe_log: FxHashMap::with_capacity_and_hasher(PROBE_LOG_CAPACITY, Default::default()),
            lru: VecDeque::with_capacity(PROBE_LOG_CAPACITY),
            trains: VecDeque::with_capacity(32),
            probes_sent: 0,
        }
    }
}

impl ProbeLog {
    pub fn record_sent(&mut self, seq: i32, spec: ProbeTrain, now_ms: u64) {
        self.advance(now_ms);
        self.probes_sent = self.probes_sent.saturating_add(1);
        if now_ms > spec.deadline_ms {
            return;
        }
        if let Some(train) = self.trains.iter().find(|t| t.sequences.contains(&seq)) {
            if train.spec.id == spec.id && self.probe_log.contains_key(&seq) {
                self.remove(seq);
                self.probe_log.insert(seq, now_ms);
                self.lru.push_back(seq);
            }
            return;
        }
        if !self.trains.iter().any(|t| t.spec.id == spec.id) {
            if self.trains.len() == 32
                && let Some(old) = self.trains.pop_front()
            {
                for seq in old.sequences {
                    self.remove(seq);
                }
            }
            self.trains.push_back(Train {
                spec,
                sequences: SmallVec::new(),
                acknowledged: 0,
                expired: false,
            });
        }
        let Some(train) = self.trains.iter_mut().find(|t| t.spec.id == spec.id) else {
            return;
        };
        if train.expired || train.sequences.len() >= usize::from(PROBE_TRAIN_LEN) {
            return;
        }
        train.sequences.push(seq);
        self.remove(seq);
        if self.probe_log.len() == PROBE_LOG_CAPACITY
            && let Some(oldest) = self.lru.front().copied()
        {
            self.remove(oldest);
        }
        self.probe_log.insert(seq, now_ms);
        self.lru.push_back(seq);
    }

    pub fn acknowledge(&mut self, seq: i32, now_ms: u64) -> bool {
        self.advance(now_ms);
        if !self.probe_log.get(&seq).is_some_and(|&sent| sent <= now_ms) {
            return false;
        }
        let Some(train) = self
            .trains
            .iter_mut()
            .find(|t| !t.expired && t.sequences.contains(&seq))
        else {
            return false;
        };
        train.acknowledged += 1;
        self.remove(seq);
        true
    }

    pub fn advance(&mut self, now_ms: u64) {
        for train in &mut self.trains {
            if now_ms > train.spec.deadline_ms {
                train.expired = true;
                for seq in &train.sequences {
                    self.probe_log.remove(seq);
                }
            }
        }
        self.lru.retain(|seq| self.probe_log.contains_key(seq));
        self.trains.retain(|t| {
            now_ms.saturating_sub(t.spec.started_ms)
                <= t.spec
                    .deadline_ms
                    .saturating_sub(t.spec.started_ms)
                    .saturating_mul(2)
        });
        let mut finished = self
            .trains
            .iter()
            .filter(|t| t.expired || t.acknowledged == PROBE_TRAIN_LEN)
            .count();
        self.trains.retain(|t| {
            if finished > 2 && (t.expired || t.acknowledged == PROBE_TRAIN_LEN) {
                finished -= 1;
                false
            } else {
                true
            }
        });
    }

    pub(crate) fn has_seen(&self, seq: i32) -> bool {
        self.trains.iter().any(|t| t.sequences.contains(&seq))
    }

    /// Last two complete/expired trains, including losses from failed trains.
    pub fn probe_loss(&self) -> Option<f64> {
        let mut finished = self
            .trains
            .iter()
            .rev()
            .filter(|t| t.expired || t.acknowledged == PROBE_TRAIN_LEN);
        let a = finished.next()?;
        let b = finished.next()?;
        Some(
            f64::from(2 * PROBE_TRAIN_LEN - a.acknowledged - b.acknowledged)
                / f64::from(2 * PROBE_TRAIN_LEN),
        )
    }

    /// Consecutive successful trains and their oldest start, for HealthSignals.
    pub fn rounds_ok(&self) -> (u32, Option<u64>) {
        let mut count = 0;
        let mut start = None;
        for train in self.trains.iter().rev().filter(|t| {
            t.expired || (t.acknowledged >= 5 && t.sequences.len() == usize::from(PROBE_TRAIN_LEN))
        }) {
            if train.acknowledged < 5 || train.sequences.len() != usize::from(PROBE_TRAIN_LEN) {
                break;
            }
            count += 1;
            start = Some(train.spec.started_ms);
            if count == 2 {
                break;
            }
        }
        (count, start)
    }

    pub fn reset(&mut self) {
        self.probe_log.clear();
        self.lru.clear();
        self.trains.clear();
    }

    fn remove(&mut self, seq: i32) {
        self.probe_log.remove(&seq);
        self.lru.retain(|&s| s != seq);
    }
}
