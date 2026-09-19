use crate::connection::health::RecoveryRounds;
use crate::connection::probe::{PROBE_TRAIN_LEN, ProbeLog, ProbeTrain};

/// Observes originals without sending copies or changing original/probe accounting.
/// Qualification reuses the existing bounded ten-packet train implementation.
#[derive(Debug, Default)]
pub(crate) struct OriginalRecovery {
    log: ProbeLog,
    epoch: Option<u64>,
    samples: u8,
    active: Option<ProbeTrain>,
}

pub(crate) struct RecoveryWindow {
    pub epoch_ms: u64,
    pub timeout_ms: u64,
}

impl OriginalRecovery {
    pub fn record_sent(&mut self, seq: i32, now: u64, window: RecoveryWindow) {
        if self.epoch != Some(window.epoch_ms)
            || self.active.is_some_and(|train| now > train.deadline_ms)
        {
            self.log.reset();
            self.samples = 0;
            self.active = None;
            self.epoch = Some(window.epoch_ms);
        }
        if self.samples == 2 * PROBE_TRAIN_LEN || self.log.has_seen(seq) {
            return;
        }
        if self.samples.is_multiple_of(PROBE_TRAIN_LEN) {
            self.active = Some(ProbeTrain {
                id: u64::from(self.samples / PROBE_TRAIN_LEN),
                started_ms: now,
                deadline_ms: now.saturating_add(window.timeout_ms),
            });
        }
        if let Some(train) = self.active {
            self.log.record_sent(seq, train, now);
            self.samples += 1;
        }
    }

    pub fn acknowledge(&mut self, seq: i32, now: u64) {
        self.log.acknowledge(seq, now);
    }

    pub fn rounds(&mut self, now: u64) -> RecoveryRounds {
        self.log.advance(now);
        let (count, started_ms) = self.log.rounds_ok();
        RecoveryRounds { count, started_ms }
    }
}
