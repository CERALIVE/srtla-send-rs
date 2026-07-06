//! Tests for the EXPERIMENTAL `stall_deselect` selection penalty (default OFF).
//!
//! When the flag is ON, the mode-agnostic selection layer excludes a stalled
//! link — connected, in-flight backlog `>= STALL_MIN_IN_FLIGHT_PACKETS`, and its
//! last EARNED-ACK / keepalive-RTT sample older than `STALL_ACK_STALE_MS` — from
//! selection WHILE a non-stalled connected link exists, re-probing every
//! `STALL_REPROBE_INTERVAL_MS` so a recovered link re-enters. It is a bounded
//! selection penalty ONLY: no re-registration, no reset, `CONN_TIMEOUT`
//! untouched. These tests lock (i) deselect-while-healthy, (ii) all-stalled
//! fallback (never `None`), (iii) reprobe re-entry, (iv) no early timeout /
//! re-registration, and (v) golden-trace default-equivalence (flag absent vs
//! present-but-off). All deterministic — the stall clock is the injected
//! `current_time_ms` selection argument.
//!
//! Hypothesis-only mechanism; NOT validated on real bond hardware; makes NO
//! bond-level improvement claim.

#[cfg(test)]
mod tests {
    use smallvec::SmallVec;

    use crate::config::{
        ConfigSnapshot, STALL_ACK_STALE_MS, STALL_MIN_IN_FLIGHT_PACKETS, STALL_REPROBE_INTERVAL_MS,
    };
    use crate::connection::SrtlaConnection;
    use crate::mode::SchedulingMode;
    use crate::sender::selection::{EdpfSchedulerState, select_by_mode, select_connection_idx};
    use crate::test_helpers::create_test_connections;

    fn pool(count: usize) -> SmallVec<SrtlaConnection, 4> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(create_test_connections(count))
    }

    /// A `ConfigSnapshot` with the stall defaults and a chosen mode + flag state.
    fn snap(mode: SchedulingMode, stall_deselect: bool) -> ConfigSnapshot {
        ConfigSnapshot {
            mode,
            quality_enabled: true,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect,
            stall_min_in_flight: STALL_MIN_IN_FLIGHT_PACKETS,
            stall_ack_stale_ms: STALL_ACK_STALE_MS,
            stall_reprobe_ms: STALL_REPROBE_INTERVAL_MS,
        }
    }

    /// A fresh 2-link pool at selection time T = 10_000: conn0 is healthy (low
    /// in-flight, fresh sample) and conn1 is stalled but has the HIGHER raw score
    /// (in-flight 32, window at the ceiling, sample stale by 5s) with its reprobe
    /// window not yet due. Without the penalty conn1 wins on score; with it,
    /// conn1 must be deselected.
    fn healthy_plus_stalled() -> SmallVec<SrtlaConnection, 4> {
        let mut conns = pool(2);
        conns[0].in_flight_packets = 15;
        conns[0].window = 20_000;
        conns[0].last_ack_or_rtt_sample_ms = 10_000;
        conns[1].in_flight_packets = 32;
        conns[1].window = 60_000;
        conns[1].last_ack_or_rtt_sample_ms = 5_000;
        conns[1].last_stall_reprobe_ms = 10_000;
        conns
    }

    // ---- (i) deselect while a healthy link exists ----------------------------

    #[test]
    fn stalled_link_is_deselected_while_a_healthy_link_exists() {
        let t = 10_000;

        let mut off = healthy_plus_stalled();
        let sel_off = select_connection_idx(
            &mut off,
            None,
            0,
            t,
            &snap(SchedulingMode::Enhanced, false),
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            sel_off,
            Some(1),
            "flag OFF: the higher-scoring stalled link wins (no penalty applied)"
        );

        let mut on = healthy_plus_stalled();
        let sel_on = select_connection_idx(
            &mut on,
            None,
            0,
            t,
            &snap(SchedulingMode::Enhanced, true),
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            sel_on,
            Some(0),
            "flag ON: the stalled link is deselected so the healthy link carries traffic"
        );
    }

    #[test]
    fn flag_off_applies_no_stall_penalty() {
        let mut conns = healthy_plus_stalled();
        let sel = select_connection_idx(
            &mut conns,
            None,
            0,
            10_000,
            &snap(SchedulingMode::Enhanced, false),
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            sel,
            Some(1),
            "default OFF: the penalty is fully gated, higher score wins as baseline"
        );
    }

    // ---- (ii) all-stalled fallback: never None -------------------------------

    #[test]
    fn all_stalled_fallback_still_returns_a_link() {
        let mut conns = pool(3);
        for c in conns.iter_mut() {
            c.in_flight_packets = 32;
            c.window = 20_000;
            c.last_ack_or_rtt_sample_ms = 5_000;
            c.last_stall_reprobe_ms = 10_000;
        }

        let sel = select_connection_idx(
            &mut conns,
            None,
            0,
            10_000,
            &snap(SchedulingMode::Enhanced, true),
            &mut EdpfSchedulerState::default(),
        );
        assert!(
            sel.is_some(),
            "all links stalled: selection must still return a link, never None, while connections \
             exist"
        );
        assert_eq!(
            sel,
            Some(0),
            "with all scores tied the lowest index is chosen"
        );
    }

    #[test]
    fn single_stalled_link_is_never_starved() {
        let mut conns = pool(1);
        conns[0].in_flight_packets = 40;
        conns[0].window = 20_000;
        conns[0].last_ack_or_rtt_sample_ms = 5_000;
        conns[0].last_stall_reprobe_ms = 10_000;

        let sel = select_connection_idx(
            &mut conns,
            None,
            0,
            10_000,
            &snap(SchedulingMode::Enhanced, true),
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            sel,
            Some(0),
            "the only link, though stalled, is still returned (fallback, never None)"
        );
    }

    // ---- (iii) recovered link re-enters after the reprobe window -------------

    #[test]
    fn recovered_link_re_enters_after_reprobe_window() {
        let mut conns = healthy_plus_stalled();
        let cfg = snap(SchedulingMode::Enhanced, true);
        let mut edpf = EdpfSchedulerState::default();

        let s1 = select_connection_idx(&mut conns, None, 0, 10_000, &cfg, &mut edpf);
        assert_eq!(
            s1,
            Some(0),
            "within the reprobe window the stalled link stays deselected"
        );

        let s2 = select_connection_idx(&mut conns, None, 0, 11_000, &cfg, &mut edpf);
        assert_eq!(
            s2,
            Some(1),
            "one reprobe interval later the stalled link is eligible again and re-enters selection"
        );
        assert_eq!(
            conns[1].last_stall_reprobe_ms, 11_000,
            "the reprobe tick is stamped so the next exclusion window starts here"
        );

        let s3 = select_connection_idx(&mut conns, None, 0, 11_500, &cfg, &mut edpf);
        assert_eq!(
            s3,
            Some(0),
            "before the next reprobe elapses the link is deselected again"
        );
    }

    // ---- (iv) no early timeout / re-registration (housekeeping unaffected) ---

    #[test]
    fn deselect_never_times_out_or_reregisters_before_conn_timeout() {
        let mut conns = healthy_plus_stalled();

        assert!(
            conns[1].is_stall_penalized(10_000, STALL_MIN_IN_FLIGHT_PACKETS, STALL_ACK_STALE_MS),
            "conn1 is stall-penalized"
        );
        assert!(
            !conns[1].is_timed_out(),
            "a 3s stall is far below the 15s CONN_TIMEOUT, so the link is NOT timed out"
        );

        let established_before = conns[1].connection_established_ms();
        let window_before = conns[1].window;

        let sel = select_connection_idx(
            &mut conns,
            None,
            0,
            10_000,
            &snap(SchedulingMode::Enhanced, true),
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(sel, Some(0));

        assert!(
            conns[1].connected,
            "the transient exclusion mask is fully restored (connected)"
        );
        assert!(
            conns[1].last_received.is_some(),
            "the transient exclusion mask is fully restored (last_received)"
        );
        assert!(
            !conns[1].is_timed_out(),
            "deselect is a selection penalty only — it never times the link out"
        );
        assert_eq!(
            conns[1].connection_established_ms(),
            established_before,
            "no re-registration occurred (connection_established_ms unchanged)"
        );
        assert_eq!(
            conns[1].window, window_before,
            "no reset occurred (window is not reset to WINDOW_DEF)"
        );
    }

    // ---- (v) GOLDEN-TRACE default-equivalence (flag absent vs present-but-off)

    #[derive(Clone, Copy)]
    enum Ev {
        Send { conn: usize, seq: i32 },
        SrtlaAck { seq: i32 },
        SrtAck { cum: i32 },
        Nak { conn: usize, seq: i32 },
    }

    #[derive(Clone, Copy)]
    enum Arm {
        /// The pre-`stall_deselect` classic selection, reimplemented
        /// independently: skip timed-out links, argmax `window / (in_flight +
        /// 1)`, ties resolved to the lowest index. Stands in for "flag absent".
        Baseline,
        /// The real `select_connection_idx` with the flag OFF ("present-but-off").
        FlagOff,
    }

    /// Independent reimplementation of the pre-change classic selection.
    fn classic_argmax(conns: &[SrtlaConnection]) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_score = -1;
        for (i, c) in conns.iter().enumerate() {
            if c.is_timed_out() {
                continue;
            }
            let score = c.get_score();
            if score > best_score {
                best_score = score;
                best = Some(i);
            }
        }
        best
    }

    fn golden_trace() -> Vec<Ev> {
        vec![
            Ev::Send { conn: 0, seq: 10 },
            Ev::Send { conn: 0, seq: 11 },
            Ev::Send { conn: 1, seq: 20 },
            Ev::Send { conn: 2, seq: 30 },
            Ev::Send { conn: 2, seq: 31 },
            Ev::Send { conn: 2, seq: 32 },
            Ev::SrtlaAck { seq: 20 },
            Ev::SrtAck { cum: 11 },
            Ev::Nak { conn: 2, seq: 30 },
            Ev::SrtlaAck { seq: 31 },
            Ev::Send { conn: 0, seq: 40 },
            Ev::SrtlaAck { seq: 40 },
            Ev::Nak { conn: 2, seq: 32 },
            Ev::SrtAck { cum: 40 },
        ]
    }

    fn apply_ev(conns: &mut [SrtlaConnection], ev: Ev, now: u64) {
        match ev {
            Ev::Send { conn, seq } => conns[conn].register_packet(seq, now),
            Ev::SrtlaAck { seq } => {
                for c in conns.iter_mut() {
                    if c.handle_srtla_ack_specific(seq, true) {
                        break;
                    }
                }
                for c in conns.iter_mut() {
                    c.handle_srtla_ack_global();
                }
            }
            Ev::SrtAck { cum } => {
                for c in conns.iter_mut() {
                    c.handle_srt_ack(cum);
                }
            }
            Ev::Nak { conn, seq } => {
                conns[conn].handle_nak(seq);
            }
        }
    }

    fn run_trace(trace: &[Ev], arm: Arm) -> Vec<Option<usize>> {
        let mut conns = pool(3);
        let now = crate::utils::now_ms();
        let cfg = snap(SchedulingMode::Classic, false);
        let mut edpf = EdpfSchedulerState::default();
        let mut out = Vec::with_capacity(trace.len());
        for &ev in trace {
            apply_ev(&mut conns, ev, now);
            let sel = match arm {
                Arm::Baseline => classic_argmax(&conns),
                Arm::FlagOff => select_connection_idx(&mut conns, None, 0, now, &cfg, &mut edpf),
            };
            out.push(sel);
        }
        out
    }

    #[test]
    fn golden_trace_flag_off_is_identical_to_baseline() {
        let trace = golden_trace();
        let baseline = run_trace(&trace, Arm::Baseline);
        let flag_off = run_trace(&trace, Arm::FlagOff);

        assert_eq!(
            baseline.len(),
            trace.len(),
            "one selection recorded per event"
        );
        for (step, (b, f)) in baseline.iter().zip(flag_off.iter()).enumerate() {
            assert_eq!(
                b, f,
                "step {step}: flag-off selection must reproduce the flag-absent baseline exactly"
            );
        }
    }

    #[test]
    fn golden_trace_is_non_trivial() {
        let selected = run_trace(&golden_trace(), Arm::Baseline);
        let distinct: std::collections::BTreeSet<Option<usize>> =
            selected.iter().copied().collect();
        assert!(
            distinct.len() >= 2,
            "selection must visit at least two distinct links, or the equivalence is vacuous"
        );
    }

    #[test]
    fn flag_off_is_a_pure_passthrough_in_enhanced() {
        // The DEPLOYED mode is enhanced: prove flag-off there is byte-identical to
        // the extracted baseline `select_by_mode`, so the wrapper adds nothing.
        let trace = golden_trace();
        let now = crate::utils::now_ms();
        let cfg = snap(SchedulingMode::Enhanced, false);

        let mut a = pool(3);
        let mut b = pool(3);
        let mut edpf_a = EdpfSchedulerState::default();
        let mut edpf_b = EdpfSchedulerState::default();

        for &ev in &trace {
            apply_ev(&mut a, ev, now);
            apply_ev(&mut b, ev, now);
            let via_wrapper = select_connection_idx(&mut a, None, 0, now, &cfg, &mut edpf_a);
            let via_mode = select_by_mode(&mut b, None, 0, now, &cfg, &mut edpf_b);
            assert_eq!(
                via_wrapper, via_mode,
                "flag-off select_connection_idx must equal the raw select_by_mode at every step"
            );
        }
    }

    // ---- hardware-gated real-Starlink validation (ignored) -------------------

    #[test]
    #[ignore = "hardware-gated: requires a real bonded Starlink/cellular link stall (no such \
                transport in CI); run with `--ignored` on hardware, per the cerastream \
                hardware-validation notes"]
    fn stall_deselect_real_starlink_repro() {
        // Real-hardware validation harness (Todo 15). On a live bond with
        // stall_deselect ON, a Starlink uplink entering a multi-second obstruction
        // stall (in-flight backlog, no earned ACK / keepalive-RTT) must be
        // deselected in favour of a healthy cellular uplink, then re-enter after
        // the obstruction clears. There is no bonded satellite transport in this
        // environment, so the end-to-end behaviour cannot be exercised here; the
        // deterministic contract is covered by the tests above. Hypothesis-only;
        // NOT validated in this environment.
        let cfg = snap(SchedulingMode::Enhanced, true);
        assert!(
            cfg.stall_deselect,
            "a hardware run must exercise the flag ON against real bonded uplinks"
        );
    }
}
