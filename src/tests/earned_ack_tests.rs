//! Tests for the EXPERIMENTAL `earned_ack_window` valve (default OFF).
//!
//! The valve replaces the unconditional broadcast-ACK window `+1`
//! (`handle_srtla_ack_global`) with an earner-gated variant
//! (`handle_srtla_ack_earned`): the link that owned the acked sequence keeps the
//! full `+1`, while every other link gets only rate-limited PROBE growth. These
//! tests lock (i) default-equivalence, (ii) the flag-ON valve semantics, and
//! (iii) the fairness model. All are deterministic — the valve's clock is an
//! injected `now_ms` argument, so no wall-clock timing is involved.
//!
//! Hypothesis-only mechanism; NOT validated on real bond hardware.

#[cfg(test)]
mod tests {
    use crate::config::PROBE_GROWTH_INTERVAL_MS;
    use crate::connection::SrtlaConnection;
    use crate::protocol::{WINDOW_DEF, WINDOW_MAX, WINDOW_MULT};
    use crate::sender::apply_srtla_ack;
    use crate::test_helpers::create_test_connections;

    /// Initial per-link window every fresh test connection starts at.
    fn initial_window() -> i32 {
        WINDOW_DEF * WINDOW_MULT
    }

    fn max_window() -> i32 {
        WINDOW_MAX * WINDOW_MULT
    }

    fn pool(count: usize) -> smallvec::SmallVec<SrtlaConnection, 4> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(create_test_connections(count))
    }

    /// The pure per-link score enhanced selection maximises: `window /
    /// (in_flight + 1)`. Returns the index of the highest-scoring connected link,
    /// ties resolved to the lowest index (deterministic).
    fn selected_idx(conns: &[SrtlaConnection]) -> Option<usize> {
        let mut best: Option<(usize, i32)> = None;
        for (i, c) in conns.iter().enumerate() {
            let score = c.get_score();
            if score < 0 {
                continue;
            }
            match best {
                Some((_, bs)) if bs >= score => {}
                _ => best = Some((i, score)),
            }
        }
        best.map(|(i, _)| i)
    }

    // ---- (i) GOLDEN-TRACE default-equivalence --------------------------------

    /// One event in a fixed multi-link trace.
    #[derive(Clone, Copy)]
    enum Ev {
        Send { conn: usize, seq: i32 },
        SrtlaAck { seq: i32 },
        SrtAck { cum: i32 },
        Nak { conn: usize, seq: i32 },
    }

    /// Which broadcast-ACK arm to drive.
    #[derive(Clone, Copy)]
    enum Arm {
        /// The pre-valve behaviour, reimplemented independently: specific step
        /// down the pool (break on the earner) then an unconditional global `+1`
        /// on every link. This stands in for "flag absent".
        Baseline,
        /// The real production `apply_srtla_ack` with the flag OFF
        /// ("present-but-off").
        FlagOff,
    }

    type Snapshot = (Option<usize>, Vec<(i32, i32)>);

    fn snapshot(conns: &[SrtlaConnection]) -> Snapshot {
        (
            selected_idx(conns),
            conns
                .iter()
                .map(|c| (c.window, c.in_flight_packets))
                .collect(),
        )
    }

    fn apply_arm(arm: Arm, conns: &mut [SrtlaConnection], seq: i32) {
        match arm {
            Arm::Baseline => {
                for c in conns.iter_mut() {
                    if c.handle_srtla_ack_specific(seq, false) {
                        break;
                    }
                }
                for c in conns.iter_mut() {
                    c.handle_srtla_ack_global();
                }
            }
            Arm::FlagOff => apply_srtla_ack(conns, seq, false, false),
        }
    }

    fn run_trace(trace: &[Ev], arm: Arm) -> Vec<Snapshot> {
        let mut conns = pool(3);
        let now = crate::utils::now_ms();
        let mut out = Vec::with_capacity(trace.len());
        for ev in trace {
            match *ev {
                Ev::Send { conn, seq } => conns[conn].register_packet(seq, now),
                Ev::SrtlaAck { seq } => apply_arm(arm, conns.as_mut_slice(), seq),
                Ev::SrtAck { cum } => {
                    for c in conns.iter_mut() {
                        c.handle_srt_ack(cum);
                    }
                }
                Ev::Nak { conn, seq } => {
                    conns[conn].handle_nak(seq);
                }
            }
            out.push(snapshot(&conns));
        }
        out
    }

    fn golden_trace() -> Vec<Ev> {
        vec![
            Ev::Send { conn: 0, seq: 10 },
            Ev::Send { conn: 0, seq: 11 },
            Ev::Send { conn: 1, seq: 20 },
            Ev::Send { conn: 2, seq: 30 },
            Ev::Send { conn: 2, seq: 31 },
            Ev::Send { conn: 2, seq: 32 },
            Ev::SrtlaAck { seq: 20 }, // conn1 earns
            Ev::SrtAck { cum: 11 },   // cumulative: conn0 clears 10,11
            Ev::Nak { conn: 2, seq: 30 },
            Ev::SrtlaAck { seq: 31 }, // conn2 earns
            Ev::Send { conn: 0, seq: 40 },
            Ev::SrtlaAck { seq: 40 }, // conn0 earns
            Ev::Nak { conn: 2, seq: 32 },
            Ev::SrtlaAck { seq: 999 }, // no owner: earner is None
            Ev::SrtAck { cum: 40 },
        ]
    }

    #[test]
    fn golden_trace_flag_off_is_byte_identical_to_baseline() {
        let trace = golden_trace();
        let baseline = run_trace(&trace, Arm::Baseline);
        let flag_off = run_trace(&trace, Arm::FlagOff);

        assert_eq!(
            baseline.len(),
            trace.len(),
            "one snapshot recorded per event"
        );
        for (step, (b, f)) in baseline.iter().zip(flag_off.iter()).enumerate() {
            assert_eq!(
                b, f,
                "step {step}: flag-off must reproduce the baseline selected-link + per-link \
                 (window, in_flight) exactly"
            );
        }
    }

    #[test]
    fn golden_trace_is_non_trivial() {
        // Guards the equivalence test against vacuity: the trace must actually
        // move windows/in-flight and selection around, so an identical result
        // is a real agreement, not two no-ops.
        let snaps = run_trace(&golden_trace(), Arm::FlagOff);
        let distinct_selected: std::collections::BTreeSet<Option<usize>> =
            snaps.iter().map(|(sel, _)| *sel).collect();
        assert!(
            distinct_selected.len() >= 2,
            "selection must visit at least two distinct links across the trace"
        );
        let touched_window = snaps
            .iter()
            .any(|(_, links)| links.iter().any(|(w, _)| *w != initial_window()));
        assert!(touched_window, "at least one link's window must change");
    }

    // ---- (ii) flag-ON valve semantics ----------------------------------------

    #[test]
    fn nonearner_zero_inflight_gets_at_most_one_probe_per_interval() {
        let mut conns = pool(1);
        let c = &mut conns[0];
        let w0 = c.window;
        assert_eq!(
            c.in_flight_packets, 0,
            "link starts with zero earned in-flight"
        );

        // First broadcast ACK: the initial probe fires (last_probe_growth_ms == 0
        // is arbitrarily old), a single +1 — never the earner's unconditional step.
        c.handle_srtla_ack_earned(false, 10_000);
        assert_eq!(c.window, w0 + 1, "first probe grants exactly one +1");
        assert_eq!(c.last_probe_growth_ms, 10_000);

        // A second broadcast ACK inside the same interval grants nothing.
        c.handle_srtla_ack_earned(false, 10_000 + PROBE_GROWTH_INTERVAL_MS - 1);
        assert_eq!(
            c.window,
            w0 + 1,
            "second ACK within 1000ms yields zero growth"
        );

        // Exactly one interval later, one more probe is allowed.
        c.handle_srtla_ack_earned(false, 10_000 + PROBE_GROWTH_INTERVAL_MS);
        assert_eq!(c.window, w0 + 2, "one probe per elapsed interval");
    }

    #[test]
    fn nonearner_burst_within_interval_collapses_to_single_probe() {
        let mut conns = pool(1);
        let c = &mut conns[0];
        let w0 = c.window;
        for t in [50_000, 50_100, 50_200, 50_400, 50_800] {
            c.handle_srtla_ack_earned(false, t);
        }
        assert_eq!(
            c.window,
            w0 + 1,
            "five broadcast ACKs inside one interval → at most one probe +1 (never full growth)"
        );
    }

    #[test]
    fn earner_grows_exactly_as_flag_off_global_step() {
        let mut earner_pool = pool(1);
        let mut global_pool = pool(1);
        // The earner is never rate-limited: it takes the full +1 on every ACK,
        // identically to the flag-off global path.
        for (n, t) in [10_000_u64, 10_050, 10_100, 10_150].into_iter().enumerate() {
            earner_pool[0].handle_srtla_ack_earned(true, t);
            global_pool[0].handle_srtla_ack_global();
            assert_eq!(
                earner_pool[0].window,
                global_pool[0].window,
                "after {} ACK(s) the earner tracks the flag-off window exactly",
                n + 1
            );
        }
        assert_eq!(
            earner_pool[0].window,
            initial_window() + 4,
            "earner grew +1 per ACK (no rate limit)"
        );
    }

    #[test]
    fn earner_outgrows_throttled_nonearner_under_burst() {
        let mut earner = pool(1);
        let mut nonearner = pool(1);
        for t in [70_000, 70_100, 70_200] {
            earner[0].handle_srtla_ack_earned(true, t);
            nonearner[0].handle_srtla_ack_earned(false, t);
        }
        assert_eq!(earner[0].window, initial_window() + 3, "earner: +1 per ACK");
        assert_eq!(
            nonearner[0].window,
            initial_window() + 1,
            "non-earner throttled to a single probe across the burst"
        );
    }

    #[test]
    fn production_fanout_grows_earner_and_throttles_nonearner() {
        // Drive the REAL apply_srtla_ack (flag ON). conn0 owns the acked
        // sequences (earner); conn1 has zero earned in-flight (probing non-earner).
        let mut conns = pool(2);
        let now = crate::utils::now_ms();
        conns[0].register_packet(100, now);
        conns[0].register_packet(101, now);

        apply_srtla_ack(conns.as_mut_slice(), 100, false, true);
        let earner_after_first = conns[0].window;
        let nonearner_after_first = conns[1].window;
        assert_eq!(
            earner_after_first,
            initial_window() + 1,
            "earner takes the full +1"
        );
        assert_eq!(
            nonearner_after_first,
            initial_window() + 1,
            "non-earner's first probe fires"
        );

        // Immediately (same interval) apply another broadcast ACK conn0 still owns.
        apply_srtla_ack(conns.as_mut_slice(), 101, false, true);
        assert_eq!(
            conns[0].window,
            initial_window() + 2,
            "earner grows again on the immediate second ACK"
        );
        assert_eq!(
            conns[1].window, nonearner_after_first,
            "non-earner is throttled on the immediate second broadcast ACK"
        );
    }

    #[test]
    fn disconnected_or_never_received_link_never_grows() {
        let mut conns = pool(2);
        conns[0].connected = false;
        conns[1].last_received = None;
        let w0 = conns[0].window;
        let w1 = conns[1].window;
        conns[0].handle_srtla_ack_earned(true, 10_000);
        conns[0].handle_srtla_ack_earned(false, 10_000);
        conns[1].handle_srtla_ack_earned(true, 10_000);
        conns[1].handle_srtla_ack_earned(false, 10_000);
        assert_eq!(conns[0].window, w0, "a disconnected link never grows");
        assert_eq!(conns[1].window, w1, "a never-received link never grows");
    }

    // ---- (iii) FAIRNESS MODEL -------------------------------------------------

    #[test]
    fn equal_links_grow_fairly_with_no_starvation() {
        // Three equal healthy links; a symmetric round-robin trace with the clock
        // advancing exactly one interval per round. Every link — earner via the
        // full step, non-earners via the elapsed-interval probe — grows +1 each
        // round, so windows stay identical and no link is starved.
        let mut conns = pool(3);
        let rounds = 6u64;
        for r in 0..rounds {
            let t = 10_000 + r * PROBE_GROWTH_INTERVAL_MS;
            let earner = (r % 3) as usize;
            for (i, c) in conns.iter_mut().enumerate() {
                c.handle_srtla_ack_earned(i == earner, t);
            }
        }
        let expected = initial_window() + rounds as i32;
        for (i, c) in conns.iter().enumerate() {
            assert_eq!(
                c.window, expected,
                "link {i} grew fairly to the shared window (no starvation)"
            );
        }
        // Selection has no single permanent winner: all scores are tied.
        assert_eq!(
            selected_idx(&conns),
            Some(0),
            "tie resolves to lowest index"
        );
    }

    #[test]
    fn idle_link_re_enters_via_probe_growth() {
        // A degraded, never-earning link is not frozen out: rate-limited probe
        // growth lifts its window (hence its selection score) by +1 every elapsed
        // interval, monotonically, so it keeps climbing back toward the healthy
        // links instead of collapsing to the floor.
        let mut conns = pool(1);
        conns[0].window = 3000;
        conns[0].last_probe_growth_ms = 0;
        let mut prev_window = conns[0].window;
        let mut prev_score = conns[0].get_score();
        let intervals = 12u64;
        for k in 1..=intervals {
            let t = 20_000 + k * PROBE_GROWTH_INTERVAL_MS;
            conns[0].handle_srtla_ack_earned(false, t);
            assert_eq!(
                conns[0].window,
                prev_window + 1,
                "interval {k}: idle link recovers exactly one probe step"
            );
            assert!(
                conns[0].get_score() > prev_score,
                "interval {k}: recovered window raises the selection score"
            );
            prev_window = conns[0].window;
            prev_score = conns[0].get_score();
        }
        assert_eq!(
            conns[0].window,
            3000 + intervals as i32,
            "idle link climbed back via probe growth"
        );
    }

    #[test]
    fn idle_link_without_elapsed_interval_stays_put() {
        // Contrast to the recovery test: with no interval elapsing, the probe is
        // withheld — proving probe growth (not the mere broadcast ACK) is the
        // recovery mechanism.
        let mut conns = pool(1);
        conns[0].window = 3000;
        conns[0].handle_srtla_ack_earned(false, 20_000);
        let after_first = conns[0].window;
        for t in [20_100, 20_200, 20_300] {
            conns[0].handle_srtla_ack_earned(false, t);
        }
        assert_eq!(
            conns[0].window, after_first,
            "without an elapsed interval the idle link does not climb"
        );
    }

    #[test]
    fn earning_link_window_does_not_collapse_vs_flag_off() {
        // A link that earns every ACK carries the traffic; the valve must not
        // penalise it. Its window under the flag (earner path) tracks the flag-off
        // global path exactly, so total window across earning links never collapses.
        let mut on = pool(1);
        let mut off = pool(1);
        for k in 0..40u64 {
            on[0].handle_srtla_ack_earned(true, 10_000 + k);
            off[0].handle_srtla_ack_global();
        }
        assert_eq!(
            on[0].window, off[0].window,
            "earning link's window is identical with the valve on vs off"
        );
    }

    #[test]
    fn per_second_regime_keeps_total_window_in_step_with_flag_off() {
        // When broadcast ACKs arrive about once per interval (each link either
        // earns or probes on the elapsed interval), the valve-on total window
        // across the pool stays equal to flag-off: earners and probing links all
        // take +1, so nothing collapses relative to the baseline.
        let mut on = pool(3);
        let mut off = pool(3);
        for r in 0..8u64 {
            let t = 30_000 + r * PROBE_GROWTH_INTERVAL_MS;
            let earner = (r % 3) as usize;
            for (i, c) in on.iter_mut().enumerate() {
                c.handle_srtla_ack_earned(i == earner, t);
            }
            for c in off.iter_mut() {
                c.handle_srtla_ack_global();
            }
        }
        let sum_on: i32 = on.iter().map(|c| c.window).sum();
        let sum_off: i32 = off.iter().map(|c| c.window).sum();
        assert_eq!(
            sum_on, sum_off,
            "per-interval ACK cadence keeps the valve-on total window level with flag-off"
        );
    }

    #[test]
    fn window_growth_is_clamped_to_max() {
        let mut conns = pool(1);
        conns[0].window = max_window() - 1;
        conns[0].handle_srtla_ack_earned(true, 10_000);
        assert_eq!(
            conns[0].window,
            max_window(),
            "earner +1 clamps at WINDOW_MAX"
        );
        conns[0].handle_srtla_ack_earned(true, 20_000);
        assert_eq!(conns[0].window, max_window(), "clamp holds at the ceiling");

        let mut probe = pool(1);
        probe[0].window = max_window() - 1;
        probe[0].handle_srtla_ack_earned(false, 40_000);
        assert_eq!(
            probe[0].window,
            max_window(),
            "probe +1 clamps at WINDOW_MAX too"
        );
    }
}
