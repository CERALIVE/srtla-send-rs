//! DATA delivery evidence independent of destructive congestion packet-log pruning.

use std::collections::VecDeque;

use rustc_hash::FxHashMap;

pub const DELIVERY_CAPACITY: usize = 4096;
pub const DELIVERY_MAX_AGE_MS: u64 = 6000;
pub const DELIVERY_WINDOW_MS: u64 = 2000;

#[derive(Clone, Copy, Debug)]
pub struct DataSend {
    pub sent_ms: u64,
    pub len: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct DeliveryAck {
    pub seq: i32,
    pub socket_generation: u32,
}

#[derive(Clone, Copy, Debug)]
struct Entry {
    send: DataSend,
    generation: u32,
    previous: Option<i32>,
    next: Option<i32>,
}

/// One socket lifetime's evidence. Only accepted DATA belongs here, never probes.
/// Intrusive sequence links give O(1) removal/LRU without a tombstone queue or scans.
#[derive(Debug)]
pub struct DeliveryLedger {
    entries: FxHashMap<i32, Entry>,
    oldest: Option<i32>,
    newest: Option<i32>,
    pub(crate) attempts_since_proof: u32,
    pub(crate) last_data_proof_ms: u64,
    has_data_proof: bool,
    proof_anchor_ms: Option<u64>,
    delivered_bytes_window: VecDeque<(u64, u64)>,
    pub(crate) socket_generation: u32,
}

impl Default for DeliveryLedger {
    fn default() -> Self {
        Self {
            entries: FxHashMap::with_capacity_and_hasher(DELIVERY_CAPACITY, Default::default()),
            oldest: None,
            newest: None,
            attempts_since_proof: 0,
            last_data_proof_ms: 0,
            has_data_proof: false,
            proof_anchor_ms: None,
            delivered_bytes_window: VecDeque::with_capacity(2000),
            socket_generation: 0,
        }
    }
}

impl DeliveryLedger {
    /// `sent_ms` is the monotonic acceptance time, not the queue timestamp.
    pub fn record_sent(&mut self, seq: i32, send: DataSend) {
        self.expire(send.sent_ms);
        self.remove(seq);
        if self.entries.len() == DELIVERY_CAPACITY
            && let Some(oldest) = self.oldest
        {
            self.remove(oldest);
        }
        let entry = Entry {
            send,
            generation: self.socket_generation,
            previous: self.newest,
            next: None,
        };
        if let Some(newest) = self.newest {
            if let Some(previous) = self.entries.get_mut(&newest) {
                previous.next = Some(seq);
            }
        } else {
            self.oldest = Some(seq);
        }
        self.entries.insert(seq, entry);
        self.newest = Some(seq);
        self.attempts_since_proof = self.attempts_since_proof.saturating_add(1);
        self.proof_anchor_ms.get_or_insert(send.sent_ms);
    }

    /// Reject stale reader-generation tokens before looking up a reused sequence.
    /// SRTLA's wire ACK has no generation; the receiver-side caller owns the token.
    pub fn acknowledge(&mut self, ack: DeliveryAck, now_ms: u64) -> bool {
        if ack.socket_generation != self.socket_generation {
            return false;
        }
        self.expire(now_ms);
        let Some(entry) = self.entries.get(&ack.seq) else {
            return false;
        };
        if entry.generation != self.socket_generation || entry.send.sent_ms > now_ms {
            return false;
        }
        let len = entry.send.len;
        self.remove(ack.seq);
        self.record_probe_proof(now_ms);
        while self
            .delivered_bytes_window
            .front()
            .is_some_and(|&(ms, _)| now_ms.saturating_sub(ms) >= DELIVERY_WINDOW_MS)
        {
            self.delivered_bytes_window.pop_front();
        }
        // Aggregate equal milliseconds: at most 2000 buckets, even at high PPS.
        match self.delivered_bytes_window.back_mut() {
            Some((ms, bytes)) if *ms == now_ms => *bytes += u64::from(len),
            Some(_) | None => self
                .delivered_bytes_window
                .push_back((now_ms, u64::from(len))),
        }
        true
    }

    /// Before first proof, age starts at the first accepted attempt; idle is unknown.
    pub fn proof_age_ms(&self, now_ms: u64) -> Option<u64> {
        self.proof_anchor_ms
            .map(|anchor| now_ms.saturating_sub(anchor))
    }

    /// Unlike proof_age_ms, this never mistakes an initial attempt for DATA proof.
    pub const fn latest_data_proof_ms(&self) -> Option<u64> {
        if self.has_data_proof {
            Some(self.last_data_proof_ms)
        } else {
            None
        }
    }

    /// ACK-credited wire bits/s over (now-2000ms, now], with a fixed 2s denominator.
    pub fn delivered_bps(&self, now_ms: u64) -> f64 {
        self.delivered_bytes_window.iter()
            .filter(|&&(ms, _)| ms <= now_ms && now_ms - ms < DELIVERY_WINDOW_MS)
            // Widening to an approximate rate is intentional; no integer count is narrowed.
            .map(|&(_, bytes)| bytes as f64 * 4.0)
            .sum()
    }

    pub fn reset(&mut self) {
        self.entries.clear();
        self.oldest = None;
        self.newest = None;
        self.attempts_since_proof = 0;
        self.last_data_proof_ms = 0;
        self.has_data_proof = false;
        self.proof_anchor_ms = None;
        self.delivered_bytes_window.clear();
        self.socket_generation = self.socket_generation.wrapping_add(1);
    }

    /// Probe proof refreshes health, but is not original delivered goodput.
    pub(crate) fn record_probe_proof(&mut self, now_ms: u64) {
        self.has_data_proof = true;
        self.last_data_proof_ms = now_ms;
        self.proof_anchor_ms = Some(now_ms);
        self.attempts_since_proof = 0;
    }

    pub(crate) fn contains(&self, seq: i32) -> bool {
        self.entries.contains_key(&seq)
    }

    fn expire(&mut self, now_ms: u64) {
        while let Some(seq) = self.oldest {
            if self.entries.get(&seq).is_some_and(|entry| {
                now_ms.saturating_sub(entry.send.sent_ms) <= DELIVERY_MAX_AGE_MS
            }) {
                break;
            }
            self.remove(seq);
        }
    }

    fn remove(&mut self, seq: i32) {
        let Some(entry) = self.entries.remove(&seq) else {
            return;
        };
        match entry.previous {
            Some(previous) => {
                if let Some(previous) = self.entries.get_mut(&previous) {
                    previous.next = entry.next;
                }
            }
            None => self.oldest = entry.next,
        }
        match entry.next {
            Some(next) => {
                if let Some(next) = self.entries.get_mut(&next) {
                    next.previous = entry.previous;
                }
            }
            None => self.newest = entry.previous,
        }
    }
}

#[cfg(test)]
#[path = "../tests/health_delivery_ledger_tests.rs"]
mod health_delivery_ledger_tests;
