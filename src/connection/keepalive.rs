//! Socket-scoped keepalive evidence, independent of DATA selection and RTT sampling.

use crate::protocol::{IDLE_TIME, extract_keepalive_timestamp};

pub(crate) const SILENCE_MS: u64 = 3 * IDLE_TIME * 1000;

#[derive(Debug, Default)]
pub(crate) struct KeepaliveLiveness {
    anchor_ms: Option<u64>,
    confirmed_sent_ms: Option<u64>,
    pending: [Option<u64>; 16],
    next: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::SRTLA_TYPE_KEEPALIVE;

    fn echo(sent: u64) -> [u8; 10] {
        let mut packet = [0; 10];
        packet[..2].copy_from_slice(&SRTLA_TYPE_KEEPALIVE.to_be_bytes());
        packet[2..].copy_from_slice(&sent.to_be_bytes());
        packet
    }

    #[test]
    fn same_millisecond_and_bare_replies_are_liveness_without_rtt() {
        // Given: a sent keepalive with either interoperable reply shape.
        for packet in [&echo(100)[..], &SRTLA_TYPE_KEEPALIVE.to_be_bytes()[..]] {
            let mut liveness = KeepaliveLiveness::default();
            liveness.record_sent(100);
            // When: it is echoed in the same millisecond.
            liveness.record_reply(packet, 100);
            // Then: it is real liveness even though RTT sampling rejects zero.
            assert_eq!(liveness.confirmed_sent_ms, Some(100));
            assert_eq!(liveness.silence_age_ms(200), Some(100));
        }
    }

    #[test]
    fn replay_reordering_unsent_and_future_echoes_cannot_refresh_liveness() {
        // Given: two outstanding sends, the newer already acknowledged.
        let mut liveness = KeepaliveLiveness::default();
        liveness.record_sent(100);
        liveness.record_sent(200);
        liveness.record_reply(&echo(200), 240);
        // When: old, duplicate, unsent, future, or truncated replies arrive.
        for packet in [
            &echo(100)[..],
            &echo(200),
            &echo(250),
            &echo(1000),
            &[0x90, 0, 0],
        ] {
            liveness.record_reply(packet, 500);
        }
        // Then: none renews the original reply's deadline.
        assert_eq!(liveness.silence_age_ms(500), Some(260));
    }

    #[test]
    fn repeated_sends_do_not_extend_an_unanswered_deadline() {
        // Given: the first successful send at monotonic zero.
        let mut liveness = KeepaliveLiveness::default();
        assert_eq!(liveness.silence_age_ms(0), None);
        // When: the bounded pending ring wraps without any reply.
        for now in 0..100 {
            liveness.record_sent(now);
        }
        // Then: send traffic never fabricates receive evidence.
        assert_eq!(liveness.silence_age_ms(100), Some(100));
        assert_eq!(liveness.pending.iter().flatten().count(), 16);
    }
}

impl KeepaliveLiveness {
    pub(crate) fn record_sent(&mut self, sent_ms: u64) {
        self.anchor_ms.get_or_insert(sent_ms);
        self.pending[self.next] = Some(sent_ms);
        self.next = (self.next + 1) % self.pending.len();
    }

    pub(crate) fn record_reply(&mut self, data: &[u8], now_ms: u64) {
        let sent = if data.len() == 2 {
            // Bare interop replies prove liveness, not RTT; at most once per send.
            self.pending.iter().flatten().copied().max()
        } else {
            extract_keepalive_timestamp(data)
        };
        let Some(sent) = sent else { return };
        if sent > now_ms
            || now_ms - sent > super::rtt::MAX_PLAUSIBLE_RTT_MS
            || self.confirmed_sent_ms.is_some_and(|prior| sent <= prior)
            || !self.pending.contains(&Some(sent))
        {
            return;
        }
        self.anchor_ms = Some(now_ms);
        self.confirmed_sent_ms = Some(sent);
        for pending in &mut self.pending {
            if pending.is_some_and(|at| at <= sent) {
                *pending = None;
            }
        }
    }

    pub(crate) fn silence_age_ms(&self, now_ms: u64) -> Option<u64> {
        self.anchor_ms.map(|at| now_ms.saturating_sub(at))
    }
}
