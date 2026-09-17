use std::cmp::min;

use super::SrtlaConnection;
use crate::config::PROBE_GROWTH_INTERVAL_MS;
use crate::protocol::*;
use crate::utils::now_ms;

/// Forward serial distance up to which a cumulative ACK prunes the packet log by
/// walking the newly-ACKed range instead of scanning the whole log.
///
/// Walking costs one map lookup per sequence in `(old_highest, ack]`, scanning
/// costs one per logged packet; 64 is the crossover for a log of a few hundred
/// entries. Unlike the retired `(ack - old).abs()` form, the bound is a modular
/// forward distance, so a range that crosses the `0x7FFF_FFFF → 0` wrap is still
/// recognized as small and walked rather than being mistaken for a huge gap.
const ACK_WALK_MAX_DISTANCE: u32 = 64;

#[cfg(test)]
#[path = "premature_nak_io_tests.rs"]
mod premature_nak_io_tests;
#[cfg(test)]
#[path = "premature_nak_property_tests.rs"]
mod premature_nak_property_tests;
#[cfg(test)]
#[path = "premature_nak_tests.rs"]
mod premature_nak_tests;

impl SrtlaConnection {
    pub(crate) fn record_ack_round_trip(&mut self, sent_ms: u64, now: u64) {
        if self.rtt.record_round_trip(sent_ms, now).is_some() {
            self.premature_streak = 0;
        }
    }

    pub(crate) fn premature_nak_threshold_ms(&self) -> f64 {
        if self.has_rtt_sample() {
            (self.get_rtt_min_ms() / 2.0).clamp(5.0, 500.0)
        } else {
            0.0
        }
    }

    /// Register a packet as in-flight. O(1) insert.
    #[inline]
    pub fn register_packet(&mut self, seq: i32, send_time_ms: u64) {
        self.packet_log.insert(seq, send_time_ms);
        self.in_flight_packets = self.packet_log.len() as i32;
    }

    /// Handle an SRT cumulative ACK: prune every logged packet at or before
    /// `ack` in 31-bit serial order, and — only on the link that actually
    /// carried `ack` — feed the round trip to the RTT filter.
    ///
    /// A cumulative ACK is broadcast to every link, but the sequence it names
    /// was sent over exactly one of them. `owns_acked_seq` is that ownership
    /// answer, resolved by the caller against the shared sequence tracker; a
    /// link that merely relayed the ACK measures nothing, because subtracting
    /// its own unrelated send time would fabricate a latency sample. Pruning
    /// still happens on ALL links: the sequence is gone from the wire regardless
    /// of who sent it, so leaving it in-flight anywhere would leak window.
    ///
    /// Ordering is RFC1982 modular, not integer: an ACK is accepted only when it
    /// strictly follows `highest_acked_seq`. Duplicates, reordered ACKs, and the
    /// ambiguous antipodal case (exactly `2^30` away, where serial order carries
    /// no information in either direction) are all ignored — the conservative
    /// branch, since acting on an ACK we cannot order could prune live packets.
    pub fn handle_srt_ack(&mut self, ack: i32, now_ms: u64, owns_acked_seq: bool) {
        let ack_seq = SrtSeq::new(ack as u32);

        if let Some(prev) = self.highest_acked_seq
            && !prev.serial_lt(ack_seq)
        {
            return;
        }

        let ack_send_time_ms = self.packet_log.get(&ack).copied();

        let old_highest = self.highest_acked_seq.replace(ack_seq);

        match old_highest {
            Some(prev) if prev.distance(ack_seq) <= ACK_WALK_MAX_DISTANCE => {
                let mut seq = prev.next();
                loop {
                    self.packet_log.remove(&(seq.value() as i32));
                    if seq == ack_seq {
                        break;
                    }
                    seq = seq.next();
                }
            }
            _ => {
                self.packet_log
                    .retain(|&seq, _| SrtSeq::new(seq as u32).serial_gt(ack_seq));
            }
        }
        self.in_flight_packets = self.packet_log.len() as i32;

        if owns_acked_seq && let Some(sent_ms) = ack_send_time_ms {
            self.record_ack_round_trip(sent_ms, now_ms);
        }
    }

    /// Handle an owned NAK in O(1). Suppression still claims it, stopping fallback attribution.
    #[inline]
    pub fn handle_nak(&mut self, seq: i32) -> bool {
        let sent = self.packet_log.get(&seq);
        if sent.is_some()
            && let Some(accepted_at) = self.delivery.sent_ms(seq)
            // Widen age only for comparison with the fractional RTT-derived threshold.
            && (now_ms().saturating_sub(accepted_at) as f64) < self.premature_nak_threshold_ms()
            && self.premature_streak < 3
        {
            self.premature_streak += 1;
            self.premature_nak_count = self.premature_nak_count.saturating_add(1);
            return true;
        }
        let sent = self.packet_log.remove(&seq);
        if let Some(queued_ms) = sent {
            self.premature_streak = 0;
            // Normal-log removal is unique attribution. Todo 19's separate probe-log
            // lookup must not enter this branch or feed normal loss evidence.
            let debit = match self.delivery.loss_send(seq) {
                Some(send) => self.loss.debit_accepted_send(send, now_ms()),
                None => self
                    .loss
                    .debit_data_nak(self.delivery.sent_ms(seq).unwrap_or(queued_ms), now_ms()),
            };
            if let Some(debit) = debit {
                self.delivery.record_loss_debit(seq, debit);
            }
            self.in_flight_packets = self.packet_log.len() as i32;
            self.congestion
                .handle_nak(&mut self.window, seq, &self.label);
        }
        sent.is_some()
    }

    /// Handle SRTLA ACK for a specific sequence. O(1) remove.
    ///
    /// A hit proves this link carried the sequence, so the send time being
    /// removed from the packet log is a complete, correctly-attributed round
    /// trip — the same evidence a keepalive reply gives, on the data path and at
    /// data-path cadence. There is no ownership question to resolve here: unlike
    /// a cumulative SRT ACK, a miss simply means another link owns it.
    #[inline]
    pub fn handle_srtla_ack_specific(&mut self, seq: i32, classic_mode: bool) -> bool {
        self.apply_specific_ack(seq, classic_mode, true)
    }

    pub(crate) fn apply_specific_ack(
        &mut self,
        seq: i32,
        classic_mode: bool,
        sample_rtt: bool,
    ) -> bool {
        let now = now_ms();
        self.delivery.acknowledge(
            super::delivery::DeliveryAck {
                seq,
                socket_generation: self.delivery.socket_generation,
            },
            now,
        );
        if let Some(sent_ms) = self.packet_log.remove(&seq) {
            self.in_flight_packets = self.packet_log.len() as i32;

            if sample_rtt {
                self.record_ack_round_trip(sent_ms, now);
            }

            // Stall signal (EXPERIMENTAL `stall_deselect`): this link EARNED the
            // ACK (it owned the acked seq) — the strongest per-link delivery
            // proof. Stamped here + at the keepalive-RTT-response site ONLY, never
            // on generic inbound traffic, so a stalled-but-echoing link stays stale.
            self.last_ack_or_rtt_sample_ms = now;

            if classic_mode {
                self.congestion.handle_srtla_ack_specific_classic(
                    &mut self.window,
                    self.in_flight_packets,
                    seq,
                    &self.label,
                );
            } else {
                self.congestion.handle_srtla_ack_enhanced(
                    &mut self.window,
                    self.in_flight_packets,
                    &self.label,
                );
            }
            true
        } else {
            false
        }
    }

    pub fn handle_srtla_ack_global(&mut self) {
        // Global +1 window increase for connections that have received data (from
        // original implementation)
        // This matches C version: if (c->last_rcvd != 0)
        // In Rust, we check if last_received is Some (i.e., has been set when data was
        // received)
        if self.connected && self.last_received.is_some() {
            self.window = min(self.window + 1, WINDOW_MAX * WINDOW_MULT);
        }
    }

    /// EXPERIMENTAL earned-ACK valve (flag `earned_ack_window`, default OFF);
    /// used in place of `handle_srtla_ack_global` only when the flag is on.
    ///
    /// The earner keeps the full `+1` (identical to the flag-off global step).
    /// Every other connected link that has received data gets rate-limited PROBE
    /// growth of `+1` at most once per `PROBE_GROWTH_INTERVAL_MS`, preserving the
    /// C mechanism's under-selected-link probing without a free `+1` per broadcast
    /// ACK. Timed-out/disconnected links get nothing. Hypothesis-only; not
    /// validated on real bond hardware.
    pub fn handle_srtla_ack_earned(&mut self, is_earner: bool, now_ms: u64) {
        if !(self.connected && self.last_received.is_some()) {
            return;
        }
        if is_earner {
            self.window = min(self.window + 1, WINDOW_MAX * WINDOW_MULT);
            return;
        }
        if now_ms.saturating_sub(self.last_probe_growth_ms) >= PROBE_GROWTH_INTERVAL_MS {
            self.window = min(self.window + 1, WINDOW_MAX * WINDOW_MULT);
            self.last_probe_growth_ms = now_ms;
        }
    }
}
