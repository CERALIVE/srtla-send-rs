//! End-to-end in-process tests for the EDPF scheduling pipeline.
//!
//! These exercise the public `select_connection_idx` seam with `mode = Edpf`,
//! driving the full `edpf_pipeline_select` chain (BLEST head-of-line filter →
//! IoDS monotonic-ordering filter → EDPF earliest-predicted-arrival argmin) over
//! heterogeneous connection pools built by `create_test_connections`.
//!
//! Determinism notes (no wall-clock, no thread_local — T1/T2/T3):
//! - The EDPF predictor (`edpf::predicted_arrival`) is a pure function of the
//!   connection's `{connected, bitrate.current_bitrate_bps, in_flight_packets,
//!   quality_cache.multiplier, rtt}` fields. We set those fields directly.
//! - `CachedQuality::default().multiplier == 1.0` ⇒ `loss == 0.0` ⇒
//!   `effective_capacity == bitrate_bps / 8.0` (full capacity).
//! - `KalmanFilter::new().value() == 0.0` until the first `update()`, so leaving
//!   the Kalman filter untouched makes `predicted_arrival` use `rtt.rtt_min_ms`
//!   for the propagation term. We therefore set `rtt_min_ms` explicitly on every
//!   connection and never touch `kalman_rtt`.
//! - The BLEST/IoDS state lives in a caller-owned `EdpfSchedulerState` (T1), so
//!   tests own it explicitly and can prime it to force a fallback arm.
//!
//! With those invariants, for `pkt = SRT_PKT_SIZE = 1316`:
//!   `arrival = (in_flight_packets * 1316 + 1316) / (bitrate_bps / 8) + rtt_min_ms / 1000`
//! which is the formula the golden-sequence oracle (`edpf_golden_selection_sequence`)
//! pins byte-for-byte across N calls.

#[cfg(test)]
mod tests {
    use smallvec::SmallVec;

    use crate::config::ConfigSnapshot;
    use crate::connection::SrtlaConnection;
    use crate::mode::SchedulingMode;
    use crate::sender::selection::{EdpfSchedulerState, select_connection_idx};
    use crate::test_helpers::create_test_connections;

    /// Build `n` loopback test connections (blocking on the async helper).
    fn make_conns(n: usize) -> SmallVec<SrtlaConnection, 4> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(create_test_connections(n))
    }

    /// A `ConfigSnapshot` pinned to EDPF mode. The non-EDPF fields are inert for
    /// this pipeline but must be present.
    fn edpf_config() -> ConfigSnapshot {
        ConfigSnapshot {
            mode: SchedulingMode::Edpf,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        }
    }

    /// Set the three predictor-relevant fields on a connection in one place.
    fn set_link(conn: &mut SrtlaConnection, bitrate_bps: f64, in_flight: i32, rtt_min_ms: f64) {
        conn.connected = true;
        conn.bitrate.current_bitrate_bps = bitrate_bps;
        conn.in_flight_packets = in_flight;
        conn.rtt.rtt_min_ms = rtt_min_ms;
    }

    /// One EDPF select through the public seam (positional args inert for EDPF).
    fn edpf_select(
        conns: &mut [SrtlaConnection],
        config: &ConfigSnapshot,
        state: &mut EdpfSchedulerState,
    ) -> Option<usize> {
        select_connection_idx(conns, None, 0, 0, config, state)
    }

    // ---------------------------------------------------------------------
    // (a) Full BLEST → IoDS → EDPF path picks the lowest-predicted-arrival link
    // ---------------------------------------------------------------------
    #[test]
    fn edpf_full_pipeline_selects_lowest_predicted_arrival() {
        let mut conns = make_conns(3);
        // Close RTTs ⇒ BLEST admits all three; fresh IoDS ⇒ none filtered.
        // arrivals: c0≈0.16581, c1≈0.04526 (min), c2≈0.12317.
        set_link(&mut conns[0], 1_000_000.0, 10, 50.0);
        set_link(&mut conns[1], 2_000_000.0, 0, 40.0);
        set_link(&mut conns[2], 1_000_000.0, 5, 60.0);

        let config = edpf_config();
        let mut state = EdpfSchedulerState::default();

        let selected = edpf_select(&mut conns, &config, &mut state);

        assert_eq!(
            selected,
            Some(1),
            "EDPF must pick the eligible link with the lowest predicted arrival"
        );
        assert!(
            state.iods.last_arrival > 0.0,
            "an IoDS-ordered selection must ratchet the caller-owned last_arrival"
        );
    }

    // ---------------------------------------------------------------------
    // (b) Fallback arm 1: IoDS filters everything ⇒ candidate-set argmin engages
    //     (edpf_pipeline_select: `ordered.is_empty()` → select_from_indices(candidates))
    // ---------------------------------------------------------------------
    #[test]
    fn edpf_fallback_iods_empty_uses_candidate_argmin() {
        let mut conns = make_conns(2);
        // Both close-RTT ⇒ BLEST admits both. arrivals: c0≈0.11317, c1≈0.04526.
        set_link(&mut conns[0], 1_000_000.0, 5, 50.0);
        set_link(&mut conns[1], 2_000_000.0, 0, 40.0);

        let config = edpf_config();
        let mut state = EdpfSchedulerState::default();
        // Prime the IoDS ratchet above every candidate arrival so filter_valid
        // returns empty, forcing the candidate-set fallback (with a reset).
        state.iods.last_arrival = 1.0e9;

        let selected = edpf_select(&mut conns, &config, &mut state);

        assert_eq!(
            selected,
            Some(1),
            "candidate-set fallback must still pick the lowest-arrival admitted link"
        );
        assert_eq!(
            state.iods.last_arrival, 0.0,
            "the empty-IoDS arm resets last_arrival and must NOT re-ratchet on a fallback select"
        );
    }

    // ---------------------------------------------------------------------
    // (b) Fallback arm 2: candidate set yields no valid arrival ⇒ global
    //     select_from(conns) `.or_else` engages and reaches a BLEST-excluded link.
    // ---------------------------------------------------------------------
    #[test]
    fn edpf_fallback_global_when_candidates_have_no_arrival() {
        let mut conns = make_conns(2);
        // c0: low OWD ⇒ admitted by BLEST, but zero bitrate ⇒ no predicted arrival.
        set_link(&mut conns[0], 0.0, 0, 20.0);
        // c1: high OWD (block_time 80ms > 50ms) ⇒ BLEST-excluded, but a valid arrival.
        set_link(&mut conns[1], 1_000_000.0, 0, 180.0);

        let config = edpf_config();
        let mut state = EdpfSchedulerState::default();

        let selected = edpf_select(&mut conns, &config, &mut state);

        assert_eq!(
            selected,
            Some(1),
            "when no admitted candidate has a valid arrival, the global fallback must reach the \
             only link with a finite predicted arrival, even one BLEST excluded"
        );
    }

    // ---------------------------------------------------------------------
    // (b) Fallback arm 3: no link anywhere has a valid arrival ⇒ the global
    //     `.or_else` returns None (terminal fallback), without panicking.
    // ---------------------------------------------------------------------
    #[test]
    fn edpf_fallback_terminal_none_when_no_valid_arrival() {
        let mut conns = make_conns(2);
        // Both admitted by BLEST (close RTT) but zero bitrate ⇒ no arrival anywhere.
        set_link(&mut conns[0], 0.0, 0, 30.0);
        set_link(&mut conns[1], 0.0, 0, 40.0);

        let config = edpf_config();
        let mut state = EdpfSchedulerState::default();

        let selected = edpf_select(&mut conns, &config, &mut state);

        assert_eq!(
            selected, None,
            "with no finite predicted arrival on any link, every fallback yields None"
        );
    }

    // ---------------------------------------------------------------------
    // (c) Consecutive selects share the caller-owned EdpfSchedulerState (T1):
    //     the same state accumulates a monotonic last_arrival across N calls,
    //     while a separate state instance stays at its Default.
    // ---------------------------------------------------------------------
    #[test]
    fn edpf_consecutive_selects_share_owned_state() {
        let mut conns = make_conns(3);
        set_link(&mut conns[0], 3_000_000.0, 0, 30.0);
        set_link(&mut conns[1], 2_000_000.0, 2, 40.0);
        set_link(&mut conns[2], 1_000_000.0, 0, 20.0);

        let config = edpf_config();
        let mut shared = EdpfSchedulerState::default();
        let untouched = EdpfSchedulerState::default();

        let mut history = Vec::new();
        for _ in 0..5 {
            let _ = edpf_select(&mut conns, &config, &mut shared);
            history.push(shared.iods.last_arrival);
        }

        assert!(
            history[0] > 0.0,
            "the first IoDS-ordered select must record a positive arrival into the shared state"
        );
        for w in history.windows(2) {
            assert!(
                w[0] <= w[1],
                "the shared IoDS ratchet must be monotonic non-decreasing across consecutive \
                 selects (saw {} then {})",
                w[0],
                w[1]
            );
        }
        assert_eq!(
            untouched.iods.last_arrival, 0.0,
            "a separate EdpfSchedulerState must remain at its Default — state is not global"
        );
    }

    // ---------------------------------------------------------------------
    // (d) Disconnected and zero-bitrate links are never selected.
    // ---------------------------------------------------------------------
    #[test]
    fn edpf_skips_disconnected_and_zero_bitrate_links() {
        let mut conns = make_conns(3);
        // c0: disconnected but otherwise the "best" link — must be ignored.
        set_link(&mut conns[0], 10_000_000.0, 0, 10.0);
        conns[0].connected = false;
        // c1: connected, low RTT, but zero bitrate ⇒ no valid arrival.
        set_link(&mut conns[1], 0.0, 0, 20.0);
        // c2: the only viable link.
        set_link(&mut conns[2], 1_000_000.0, 0, 50.0);

        let config = edpf_config();
        let mut state = EdpfSchedulerState::default();

        for _ in 0..4 {
            let selected = edpf_select(&mut conns, &config, &mut state);
            assert_eq!(
                selected,
                Some(2),
                "EDPF must skip the disconnected link and the zero-bitrate link every tick"
            );
        }
    }

    // ---------------------------------------------------------------------
    // (e) No panic on an empty pool or an all-disconnected pool.
    // ---------------------------------------------------------------------
    #[test]
    fn edpf_empty_pool_returns_none_without_panic() {
        let mut conns: Vec<SrtlaConnection> = vec![];
        let config = edpf_config();
        let mut state = EdpfSchedulerState::default();

        let selected = edpf_select(&mut conns, &config, &mut state);
        assert_eq!(selected, None, "empty pool must yield None, not panic");
    }

    #[test]
    fn edpf_all_disconnected_pool_returns_none_without_panic() {
        let mut conns = make_conns(3);
        for c in conns.iter_mut() {
            set_link(c, 5_000_000.0, 0, 30.0);
            c.connected = false;
        }

        let config = edpf_config();
        let mut state = EdpfSchedulerState::default();

        let selected = edpf_select(&mut conns, &config, &mut state);
        assert_eq!(
            selected, None,
            "all-disconnected pool must yield None, not panic"
        );
    }

    // ---------------------------------------------------------------------
    // (f) The oracle: a fixed input set produces a byte-stable selection
    //     SEQUENCE across N calls. Locks the deterministic EDPF behaviour so any
    //     drift in BLEST/IoDS/EDPF or the predictor formula breaks this test.
    //
    //     Fixed pool (multiplier 1.0, untouched Kalman ⇒ rtt_min_ms propagation):
    //       c0: 3 Mbps, in_flight 0, rtt_min 30 → arrival ≈ 0.033509
    //       c1: 2 Mbps, in_flight 2, rtt_min 40 → arrival ≈ 0.055792
    //       c2: 1 Mbps, in_flight 0, rtt_min 20 → arrival ≈ 0.030528  (min)
    //     BLEST admits all (OWDs 15/20/10, block_times 5/10/0 ≤ 50ms). The global
    //     minimum is c2; because the inputs never change, the IoDS ratchet never
    //     excludes c2 (its arrival always equals last_arrival), so EDPF re-selects
    //     c2 on every call. The oracle sequence is therefore Some(2) repeated.
    // ---------------------------------------------------------------------
    #[test]
    fn edpf_golden_selection_sequence() {
        const N: usize = 8;

        let mut conns = make_conns(3);
        set_link(&mut conns[0], 3_000_000.0, 0, 30.0);
        set_link(&mut conns[1], 2_000_000.0, 2, 40.0);
        set_link(&mut conns[2], 1_000_000.0, 0, 20.0);

        let config = edpf_config();
        let mut state = EdpfSchedulerState::default();

        let mut sequence: Vec<Option<usize>> = Vec::with_capacity(N);
        for _ in 0..N {
            sequence.push(edpf_select(&mut conns, &config, &mut state));
        }

        let expected: [Option<usize>; N] = [Some(2); N];
        assert_eq!(
            sequence.as_slice(),
            expected.as_slice(),
            "EDPF golden selection sequence drifted — a scheduler/predictor change altered \
             deterministic link selection for a fixed input set"
        );
    }
}
