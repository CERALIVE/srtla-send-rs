//! ACK-driven RTT attribution and wrap-aware cumulative-ACK ordering.
//!
//! Two contracts are locked here. First, ownership: an SRT cumulative ACK is
//! broadcast to every uplink, so measuring a round trip from it on every uplink
//! would credit each link with a latency it never observed; only the link the
//! sequence tracker says carried the acked sequence may sample it, while all
//! links still prune their packet logs. Second, ordering: `highest_acked_seq`
//! and every comparison against it are 31-bit serial (RFC1982), so an ACK stream
//! crossing the `0x7FFF_FFFF → 0` wrap keeps advancing instead of stalling
//! forever behind a numerically-larger stale value.

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use smallvec::SmallVec;
    use tokio::net::UdpSocket;

    use crate::connection::{SrtlaConnection, SrtlaIncoming};
    use crate::protocol::{SrtSeq, create_ack_packet, parse_srt_ack, parse_srtla_ack};
    use crate::registration::SrtlaRegistrationManager;
    use crate::sender::SequenceTracker;
    use crate::sender::packet_handler::process_connection_events;
    use crate::test_helpers::{create_test_connection, create_test_connections};
    use crate::utils::now_ms;

    /// Half the 31-bit serial space: the RFC1982 antipodal distance at which no
    /// ordering exists in either direction.
    const SERIAL_MIDPOINT: u32 = 1 << 30;

    fn has_rtt_sample(conn: &SrtlaConnection) -> bool {
        conn.rtt.kalman_rtt.is_initialized()
    }

    /// Scaffolding for driving the REAL production ACK fan-out
    /// (`process_connection_events`), so ownership is resolved by the code that
    /// ships rather than by the test.
    ///
    /// Built BEFORE the test reads the clock: binding a socket and constructing
    /// the registration manager cost real milliseconds, and the handler takes
    /// its own `now_ms()` reading, so any setup done after the test's send
    /// timestamp inflates the measured round trip.
    struct AckHarness {
        listener: UdpSocket,
        instant_tx: crate::sender::packet_handler::InstantForwarder,
        _instant_rx: tokio::sync::mpsc::UnboundedReceiver<(SocketAddr, SmallVec<u8, 64>)>,
        reg: SrtlaRegistrationManager,
    }

    impl AckHarness {
        async fn new() -> Self {
            let (instant_tx, _instant_rx) = tokio::sync::mpsc::unbounded_channel();
            Self {
                listener: UdpSocket::bind("127.0.0.1:0").await.unwrap(),
                instant_tx,
                _instant_rx,
                reg: SrtlaRegistrationManager::new(),
            }
        }

        async fn broadcast_cumulative_ack(
            &mut self,
            connections: &mut [SrtlaConnection],
            seq_tracker: &SequenceTracker,
            ack: u32,
        ) {
            let mut incoming = SrtlaIncoming {
                read_any: true,
                ..Default::default()
            };
            incoming.ack_numbers.push(ack);

            process_connection_events(
                0,
                connections,
                &mut self.reg,
                &self.instant_tx,
                None,
                &self.listener,
                seq_tracker,
                false,
                false,
                Some(incoming),
            )
            .await
            .unwrap();
        }
    }

    /// Assert a measured round trip is the injected one, tolerating scheduler
    /// delay. The lower bound is EXACT — the handler cannot measure less than
    /// the injected gap — while the upper bound only has to stay below the
    /// plausibility ceiling, so a loaded machine cannot make this flake.
    fn assert_measured_round_trip(conn: &SrtlaConnection, injected_ms: f64) {
        let measured = conn.rtt.estimated_rtt_ms;
        assert!(
            measured >= injected_ms && measured < injected_ms + 500.0,
            "expected a round trip of at least the injected {injected_ms}ms, got {measured}"
        );
    }

    /// The upstream-shaped attribution test. Both uplinks hold a log entry for
    /// the acked sequence (the pathological case a broadcast creates), but the
    /// tracker names uplink 0 as its sender: only uplink 0 may turn the ACK into
    /// an RTT sample. Deleting the `owns_acked_seq` gate makes this fail on the
    /// uplink-1 assertion.
    #[tokio::test(flavor = "current_thread")]
    async fn cumulative_ack_only_measures_rtt_on_the_link_that_carried_the_seq() {
        let mut connections = create_test_connections(2).await;
        let mut harness = AckHarness::new().await;
        let sent_ms = now_ms() - 50;

        connections[0].register_packet(100, sent_ms);
        connections[1].register_packet(100, sent_ms);

        let mut seq_tracker = SequenceTracker::new();
        seq_tracker.insert(100, connections[0].conn_id, sent_ms);

        harness
            .broadcast_cumulative_ack(&mut connections, &seq_tracker, 100)
            .await;

        assert!(
            has_rtt_sample(&connections[0]),
            "the link that carried seq 100 must record the round trip"
        );
        assert_measured_round_trip(&connections[0], 50.0);
        assert!(
            !has_rtt_sample(&connections[1]),
            "a link that merely received the broadcast must not fabricate an RTT sample"
        );

        assert_eq!(
            connections[0].in_flight_packets, 0,
            "the owning link prunes its packet log"
        );
        assert_eq!(
            connections[1].in_flight_packets, 0,
            "every link prunes its packet log, ownership only gates the RTT sample"
        );
    }

    /// A tracker miss (expired entry, ring collision, departed link) is answered
    /// with no sample anywhere rather than a guess — pruning still happens.
    #[tokio::test(flavor = "current_thread")]
    async fn cumulative_ack_with_tracker_miss_samples_no_link() {
        let mut connections = create_test_connections(2).await;
        let mut harness = AckHarness::new().await;
        let sent_ms = now_ms() - 50;

        connections[0].register_packet(100, sent_ms);
        connections[1].register_packet(100, sent_ms);

        let seq_tracker = SequenceTracker::new();

        harness
            .broadcast_cumulative_ack(&mut connections, &seq_tracker, 100)
            .await;

        assert!(
            !has_rtt_sample(&connections[0]),
            "an unattributed ACK must not produce an RTT sample"
        );
        assert!(!has_rtt_sample(&connections[1]));
        assert_eq!(connections[0].in_flight_packets, 0);
        assert_eq!(connections[1].in_flight_packets, 0);
    }

    /// An SRTLA ACK names one specific sequence, and a packet-log hit is proof
    /// this link sent it — so it feeds the smoothed RTT with no ownership
    /// question to resolve.
    #[tokio::test(flavor = "current_thread")]
    async fn srtla_ack_feeds_smoothed_rtt() {
        let mut conn = create_test_connection().await;
        let sent_ms = now_ms() - 50;
        conn.register_packet(200, sent_ms);

        assert!(!has_rtt_sample(&conn));
        assert!(conn.handle_srtla_ack_specific(200, false));

        assert!(
            has_rtt_sample(&conn),
            "an earned SRTLA ACK must feed the RTT filter"
        );
        assert_measured_round_trip(&conn, 50.0);
    }

    /// A miss cannot be attributed to this link, so it yields nothing.
    #[tokio::test(flavor = "current_thread")]
    async fn srtla_ack_miss_records_no_rtt() {
        let mut conn = create_test_connection().await;
        conn.register_packet(200, now_ms() - 50);

        assert!(!conn.handle_srtla_ack_specific(999, false));
        assert!(!has_rtt_sample(&conn));
    }

    /// The shared plausibility gate: a 0ms sample (same-millisecond reply, or a
    /// send stamp at/after now) and an absurdly large one are both discarded.
    /// The zero arm is S7 parity and must not be relaxed to `rtt < 0`.
    #[test]
    fn implausible_round_trips_are_rejected() {
        let mut tracker = crate::connection::RttTracker::default();
        let now = 1_000_000_u64;

        assert_eq!(tracker.record_round_trip(now, now), None, "0ms rejected");
        assert_eq!(
            tracker.record_round_trip(now + 5, now),
            None,
            "a send stamp in the future saturates to 0ms and is rejected"
        );
        assert_eq!(
            tracker.record_round_trip(now - 20_000, now),
            None,
            "20000ms is far past the plausibility bound"
        );
        assert!(
            !tracker.kalman_rtt.is_initialized(),
            "no rejected sample may reach the filter"
        );

        assert_eq!(tracker.record_round_trip(now - 50, now), Some(50));
        assert!(tracker.kalman_rtt.is_initialized());
    }

    /// The same gate on the SRTLA-ACK data path: the ACK is still handled (log
    /// pruned, window updated), only the unusable sample is dropped.
    #[tokio::test(flavor = "current_thread")]
    async fn srtla_ack_with_implausible_round_trip_records_no_sample() {
        let mut conn = create_test_connection().await;
        let now = now_ms();

        conn.register_packet(300, now);
        assert!(conn.handle_srtla_ack_specific(300, false));
        assert!(!has_rtt_sample(&conn), "0ms sample must be rejected");

        conn.register_packet(301, now - 20_000);
        assert!(conn.handle_srtla_ack_specific(301, false));
        assert!(!has_rtt_sample(&conn), "20000ms sample must be rejected");
        assert_eq!(conn.in_flight_packets, 0, "both ACKs still pruned the log");
    }

    /// Cumulative ACKs that cross the `0x7FFF_FFFF → 0` wrap keep advancing and
    /// prune the wrapped range. Under the retired raw-integer comparison, `1`
    /// would read as behind `0x7FFF_FFFE` and every post-wrap ACK would be
    /// dropped, stranding the packet log until the connection reset.
    #[tokio::test(flavor = "current_thread")]
    async fn cumulative_ack_advances_across_the_sequence_wrap() {
        let mut conn = create_test_connection().await;
        let now = now_ms();
        for seq in [0x7fff_fffd_u32, 0x7fff_fffe, 0x7fff_ffff, 0, 1, 2] {
            conn.register_packet(seq as i32, now);
        }
        assert_eq!(conn.in_flight_packets, 6);

        conn.handle_srt_ack(0x7fff_fffe, now, false);
        assert_eq!(
            conn.highest_acked_seq,
            Some(SrtSeq::new(0x7fff_fffe)),
            "a pre-wrap ACK advances normally"
        );
        assert_eq!(conn.in_flight_packets, 4);

        conn.handle_srt_ack(1, now, false);
        assert_eq!(
            conn.highest_acked_seq,
            Some(SrtSeq::new(1)),
            "seq 1 follows 0x7FFF_FFFE in the serial domain and must be accepted"
        );
        assert_eq!(
            conn.in_flight_packets, 1,
            "0x7FFF_FFFF, 0 and 1 are pruned across the wrap; only seq 2 remains"
        );
        assert!(
            conn.packet_log.contains_key(&2),
            "the un-acked sequence beyond the wrap must survive"
        );
    }

    /// The wide-gap arm of the same wrap: a jump too large to walk falls back to
    /// the whole-log scan, which must also order serially — entries behind the
    /// ACK across the wrap are pruned, entries ahead of it are kept.
    #[tokio::test(flavor = "current_thread")]
    async fn wide_gap_ack_across_the_wrap_prunes_serially() {
        let mut conn = create_test_connection().await;
        let now = now_ms();

        conn.handle_srt_ack(0x7fff_ff00, now, false);

        conn.register_packet(0x7fff_fff0_u32 as i32, now);
        conn.register_packet(50, now);
        conn.register_packet(200, now);

        conn.handle_srt_ack(100, now, false);

        assert_eq!(conn.highest_acked_seq, Some(SrtSeq::new(100)));
        assert!(
            !conn.packet_log.contains_key(&(0x7fff_fff0_u32 as i32)),
            "a pre-wrap sequence is behind the post-wrap ACK and is pruned"
        );
        assert!(!conn.packet_log.contains_key(&50), "50 <= 100, pruned");
        assert!(
            conn.packet_log.contains_key(&200),
            "200 is ahead of the ACK and stays in flight"
        );
    }

    /// Duplicate and reordered ACKs are still ignored, now under serial ordering.
    #[tokio::test(flavor = "current_thread")]
    async fn stale_and_duplicate_acks_are_ignored() {
        let mut conn = create_test_connection().await;
        let now = now_ms();
        conn.register_packet(150, now);

        conn.handle_srt_ack(100, now, false);
        assert_eq!(conn.highest_acked_seq, Some(SrtSeq::new(100)));

        conn.handle_srt_ack(100, now, false);
        conn.handle_srt_ack(90, now, false);
        assert_eq!(
            conn.highest_acked_seq,
            Some(SrtSeq::new(100)),
            "neither a duplicate nor a stale ACK may move the high-water mark"
        );
        assert!(
            conn.packet_log.contains_key(&150),
            "an ignored ACK prunes nothing"
        );
    }

    /// RFC1982 has no answer for values exactly `2^30` apart, so such an ACK is
    /// REJECTED rather than guessed at: acting on it could prune live packets.
    /// One step inside the half-space the ordering is defined and the ACK lands.
    #[tokio::test(flavor = "current_thread")]
    async fn midpoint_ambiguous_ack_is_rejected() {
        let mut conn = create_test_connection().await;
        let now = now_ms();

        conn.handle_srt_ack(10, now, false);
        assert_eq!(conn.highest_acked_seq, Some(SrtSeq::new(10)));

        let antipodal = 10 + SERIAL_MIDPOINT;
        assert_eq!(
            SrtSeq::new(10).distance(SrtSeq::new(antipodal)),
            SERIAL_MIDPOINT
        );
        conn.handle_srt_ack(antipodal as i32, now, false);
        assert_eq!(
            conn.highest_acked_seq,
            Some(SrtSeq::new(10)),
            "an unorderable ACK must be ignored, not applied"
        );

        let just_inside = 10 + SERIAL_MIDPOINT - 1;
        conn.handle_srt_ack(just_inside as i32, now, false);
        assert_eq!(
            conn.highest_acked_seq,
            Some(SrtSeq::new(just_inside)),
            "one step inside the half-space the ACK is orderable and applies"
        );
    }

    /// A fresh link has no high-water mark at all, so the first ACK always
    /// applies — there is no in-domain sentinel value that could shadow a real
    /// sequence number.
    #[tokio::test(flavor = "current_thread")]
    async fn first_ack_on_a_fresh_link_always_applies() {
        let mut conn = create_test_connection().await;
        assert_eq!(conn.highest_acked_seq, None);

        conn.handle_srt_ack(0, now_ms(), false);
        assert_eq!(
            conn.highest_acked_seq,
            Some(SrtSeq::ZERO),
            "sequence 0 is a real ACK, not an 'unset' marker"
        );
    }

    /// Parser outputs are 31-bit by construction, so no ACK value can reach the
    /// `as i32` packet-log cast as a negative number.
    #[test]
    fn ack_entries_with_the_high_bit_are_discarded() {
        let mut frame = vec![0u8; 20];
        frame[0..2].copy_from_slice(&crate::protocol::SRT_TYPE_ACK.to_be_bytes());
        frame[16..20].copy_from_slice(&0x8000_0064_u32.to_be_bytes());
        assert_eq!(
            parse_srt_ack(&frame),
            None,
            "a cumulative ACK with bit 31 set is malformed, not sequence 100"
        );

        let srtla = create_ack_packet(&[7, 0x8000_0008, 9]);
        let parsed = parse_srtla_ack(&srtla);
        assert_eq!(
            parsed.as_slice(),
            &[7, 9],
            "the malformed word is dropped and its valid neighbours survive"
        );
        assert!(parsed.iter().all(|s| s & 0x8000_0000 == 0));
    }
}
