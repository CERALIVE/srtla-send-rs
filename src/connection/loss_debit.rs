use super::{LOSS_COHORT_MS, LOSS_HISTORY_MS};

/// Accepted-send identity and frozen deadline; retries never extend an older debit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LossSend {
    pub(super) epoch: u64,
    pub(super) cohort_start_ms: u64,
    pub(super) deadline_ms: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LossDebit {
    pub(super) send: LossSend,
    pub(super) count: u32,
}

impl LossDebit {
    pub const fn pending_at(self, now_ms: u64) -> bool {
        now_ms < self.send.deadline_ms
            && now_ms.saturating_sub(self.send.cohort_start_ms + LOSS_COHORT_MS) < LOSS_HISTORY_MS
    }

    pub fn merge(&mut self, next: Self) -> bool {
        if self.send != next.send {
            return false;
        }
        self.count = self.count.saturating_add(next.count);
        true
    }
}
