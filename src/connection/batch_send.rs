//! Batch send optimization for SRTLA connections
//!
//! This module implements packet batching inspired by Moblin's implementation:
//! - Buffers up to 16 data packets before sending
//! - Flushes on 15ms timer to ensure low latency
//! - Reduces syscall overhead significantly under high load
//!
//! At 10 Mbps with ~1300 byte packets:
//! - Without batching: ~960 syscalls/second per connection
//! - With batching: ~60-67 batch sends/second per connection (~15x reduction)

use std::sync::Arc;

use smallvec::SmallVec;
use tokio::time::Instant;
use tracing::debug;

use super::batch_recv::BatchUdpSocket;
use super::probe::ProbeTrain;

#[path = "batch_budget.rs"]
mod budget;

#[derive(Clone, Copy, Debug)]
enum QueuedKind {
    Normal(Option<u32>),
    Probe(i32, ProbeTrain),
}

#[derive(Debug)]
pub struct AcceptedProbe {
    pub seq: i32,
    pub train: ProbeTrain,
    pub len: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct WireSample {
    pub rate_bps: f64,
    pub accepted_bytes: u64,
}

/// Maximum number of packets to buffer before flushing (Moblin uses 15+1=16)
pub const BATCH_SIZE_THRESHOLD: usize = 16;

/// Maximum datagrams submitted to the kernel in one transmit call.
///
/// Sizes the `sendmmsg` `iovec`/`mmsghdr` arrays on Linux and caps the
/// sequential fallback identically, so both transmit paths drain the same
/// bounded prefix per flush.
pub const BATCH_SEND_SIZE: usize = 32;

/// Maximum time in milliseconds between flushes (Moblin uses 15ms)
const FLUSH_INTERVAL_MS: u64 = 15;

/// Result of one [`BatchSender::flush`].
///
/// `accepted` carries the per-packet tracking records for exactly the datagrams
/// the kernel accepted, in queue order. They are committed by the caller
/// (`register_packet`) **even when `error` is `Some`** — the packets really did
/// go out, so dropping their in-flight registration would corrupt the window
/// accounting. The unsent suffix stays queued.
#[derive(Debug, Default)]
pub struct FlushOutcome {
    /// Queue-order (sequence, queue timestamp in ms, accepted wire length in bytes).
    pub accepted: SmallVec<(Option<i32>, u64, usize), 4>,
    /// One wire R-flag per `accepted` entry, pushed in the same prefix-commit loop.
    pub retransmitted: SmallVec<bool, 4>,
    pub probes: SmallVec<AcceptedProbe, 4>,
    pub error: Option<std::io::Error>,
}

/// Batch sender that queues packets and flushes them efficiently
#[derive(Debug)]
pub struct BatchSender {
    wire_budget: Option<super::wire_budget::WireBudget>,
    /// Queue of packets waiting to be sent
    queue: Vec<SmallVec<u8, 1500>>,

    /// Sequence numbers for queued packets (parallel to queue)
    sequences: Vec<QueuedKind>,

    /// Timestamps when packets were queued (parallel to queue)
    queue_times: Vec<u64>,
    normal_queued: usize,

    /// Last time the queue was flushed
    last_flush_time: Instant,
}

impl Default for BatchSender {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchSender {
    /// Create a new batch sender
    pub fn new() -> Self {
        Self {
            wire_budget: None,
            queue: Vec::with_capacity(BATCH_SIZE_THRESHOLD),
            sequences: Vec::with_capacity(BATCH_SIZE_THRESHOLD),
            queue_times: Vec::with_capacity(BATCH_SIZE_THRESHOLD),
            normal_queued: 0,
            last_flush_time: Instant::now(),
        }
    }

    /// Queue a data packet for batched sending
    ///
    /// Returns true if the queue should be flushed (threshold reached)
    #[inline]
    pub fn queue_packet(&mut self, data: &[u8], seq: Option<u32>, current_time_ms: u64) -> bool {
        self.queue.push(SmallVec::from_slice_copy(data));
        self.sequences.push(QueuedKind::Normal(seq));
        self.queue_times.push(current_time_ms);
        self.normal_queued += 1;

        self.queue.len() >= BATCH_SIZE_THRESHOLD
    }

    pub(crate) fn queue_probe(&mut self, data: &[u8], seq: i32, train: ProbeTrain) {
        self.queue.push(SmallVec::from_slice_copy(data));
        self.sequences.push(QueuedKind::Probe(seq, train));
        self.queue_times.push(train.started_ms);
    }

    /// Check if the queue needs flushing based on time
    #[inline]
    pub fn needs_time_flush(&self) -> bool {
        !self.queue.is_empty()
            && self.last_flush_time.elapsed().as_millis() >= FLUSH_INTERVAL_MS as u128
    }

    /// Check if there are any packets queued
    #[inline]
    pub fn has_queued_packets(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Number of data packets currently queued (not yet flushed).
    /// Used by `get_score()` so the selection algorithm sees the true load,
    /// matching the C behaviour where `reg_pkt()` increments in_flight per packet.
    #[inline]
    pub fn queued_count(&self) -> i32 {
        self.normal_queued as i32
    }

    /// Transmit up to [`BATCH_SEND_SIZE`] queued packets, then commit.
    ///
    /// Transmit-then-commit: candidates are peeked (never removed up front),
    /// handed to the socket's batch transmit path, and only the kernel-accepted
    /// prefix is removed from the queue and reported in
    /// [`FlushOutcome::accepted`]. The unsent suffix is retained in queue order,
    /// so a partial send can neither duplicate nor drop a datagram.
    ///
    /// DATA packets go out verbatim. Unlike control frames (keepalive/REG, see
    /// `packet_io.rs` `send_control_padded`), DATA is NEVER padded to a 32-byte
    /// minimum — padding here would corrupt the SRT byte stream.
    pub async fn flush(&mut self, socket: &Arc<BatchUdpSocket>) -> FlushOutcome {
        if self.queue.is_empty() {
            return FlushOutcome::default();
        }

        let attempted_at_ms = crate::utils::now_ms();
        let candidates = self.funded_prefix(self.queue.len().min(BATCH_SEND_SIZE), attempted_at_ms);
        if candidates == 0 {
            return FlushOutcome::default();
        }
        let (sent_count, mut error) = {
            let packets: SmallVec<&[u8], BATCH_SEND_SIZE> = self.queue[..candidates]
                .iter()
                .map(|packet| &packet[..])
                .collect();
            socket.send_batch(&packets).await
        };

        self.debit_prefix(sent_count, attempted_at_ms);

        let mut accepted = SmallVec::new();
        let mut retransmitted = SmallVec::new();
        let mut probes = SmallVec::new();
        for ((&kind, &time), packet) in self
            .sequences
            .iter()
            .zip(self.queue_times.iter())
            .zip(self.queue.iter())
            .take(sent_count)
        {
            match kind {
                QueuedKind::Normal(seq) => {
                    accepted.push((seq.map(|s| s as i32), time, packet.len()));
                    retransmitted.push(packet.len() >= 16 && packet[4] & 0x04 != 0);
                }
                QueuedKind::Probe(seq, train) => probes.push(AcceptedProbe {
                    seq,
                    train,
                    len: packet.len(),
                }),
            }
        }

        self.queue.drain(..sent_count);
        self.normal_queued -= accepted.len();
        self.sequences.drain(..sent_count);
        self.queue_times.drain(..sent_count);
        self.last_flush_time = Instant::now();

        if error.is_none() && sent_count == 0 {
            error = Some(std::io::Error::new(
                std::io::ErrorKind::WriteZero,
                "batch transmit accepted no datagrams from a non-empty queue",
            ));
        }

        if sent_count > 1 {
            debug!("Batch flush: sent {} packets in one batch", sent_count);
        }

        FlushOutcome {
            accepted,
            retransmitted,
            probes,
            error,
        }
    }

    /// Reset the batch sender state (for reconnection)
    pub fn reset(&mut self) {
        self.wire_budget = None;
        self.queue.clear();
        self.sequences.clear();
        self.queue_times.clear();
        self.normal_queued = 0;
        self.last_flush_time = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_sender_queue() {
        let mut sender = BatchSender::new();
        let data = [0u8; 100];

        // Queue should not trigger flush until threshold
        for i in 0..BATCH_SIZE_THRESHOLD - 1 {
            assert!(!sender.queue_packet(&data, Some(i as u32), 0));
            assert_eq!(sender.queue.len(), i + 1);
        }

        // This one should trigger flush
        assert!(sender.queue_packet(&data, Some(15), 0));
        assert_eq!(sender.queue.len(), BATCH_SIZE_THRESHOLD);
    }

    #[test]
    fn test_batch_sender_time_flush() {
        let mut sender = BatchSender::new();
        let data = [0u8; 100];

        sender.queue_packet(&data, Some(1), 0);
        assert!(!sender.needs_time_flush()); // Just queued, shouldn't need flush

        // Simulate time passing
        sender.last_flush_time = Instant::now() - std::time::Duration::from_millis(20);
        assert!(sender.needs_time_flush()); // Now should need flush
    }

    #[test]
    fn test_batch_sender_reset() {
        let mut sender = BatchSender::new();
        let data = [0u8; 100];

        sender.queue_packet(&data, Some(1), 0);
        sender.queue_packet(&data, Some(2), 0);

        sender.reset();

        assert!(sender.queue.is_empty());
    }
}
