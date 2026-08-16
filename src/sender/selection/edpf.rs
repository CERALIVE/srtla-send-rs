//! EDPF (Earliest Delivery Path First) link selection.
//!
//! Selects the link with the lowest predicted arrival time, considering
//! in-flight data, link capacity, loss rate, and base RTT.

use crate::connection::SrtlaConnection;

/// SRT payload packet size in bytes.
///
/// Single shared home for the constant; `selection::mod` imports this rather than
/// redeclaring it, so the EDPF pipeline and the predictor can never drift apart.
pub(crate) const SRT_PKT_SIZE: usize = 1316;

/// Neutral capacity estimate (bits/s) for a link with no measured send rate.
///
/// `current_bitrate_bps` measures what an uplink has *already* sent, so it is
/// `0.0` until the link carries DATA. Dropping unmeasured links from the
/// candidate set deadlocks EDPF at startup — nothing is selected, so nothing is
/// sent, so nothing is ever measured — and starves any link idle longer than the
/// 2 s bitrate window. A flat placeholder keeps unmeasured links comparable, so
/// in-flight bytes and OWD decide between them until a measurement replaces it.
const BOOTSTRAP_CAPACITY_BPS: f64 = 1_000_000.0;

/// Seconds of arrival penalty per (ms/update) of RTT rise.
///
/// UNIT NOTE — `SrtlaConnection::get_rtt_velocity()` returns the Kalman
/// filter's velocity state, which is **ms per Kalman update**, NOT ms/s: the
/// 2-state transition is `x_pred = x + v` with no `dt` term (`src/kalman.rs`),
/// so the same physical RTT slope reports a different number at a different
/// update cadence. Upstream's own "ms/s" labelling is wrong; this factor is
/// calibrated against the per-update quantity.
///
/// Heuristic from upstream 57525c7, sim-calibrated only — EDPF is an opt-in
/// mode and this term has NOT been validated on real bond hardware.
const VELOCITY_PENALTY_FACTOR: f64 = 0.005;

/// Multiple of the bandwidth-delay product above which a link's outstanding
/// bytes are treated as queue-standing rather than in-transit.
///
/// Overrun is a RANKING penalty only — an over-BDP link is pushed later in the
/// argmin, never removed from the candidate set. Excluding it would let the
/// candidate pool empty out and re-deadlock EDPF (see `BOOTSTRAP_CAPACITY_BPS`).
const BDP_OVERRUN_MULT: f64 = 1.5;

/// Floor applied to the propagation term when sizing the BDP cap.
///
/// A link whose RTT is unset or measured as zero would otherwise produce
/// `bdp_bytes == 0`, and the `bdp_bytes > 0.0` guard would silently disable the
/// overrun penalty exactly on the links with no latency signal. A 1 ms floor
/// keeps the cap active there.
const MIN_PROPAGATION_S: f64 = 0.001;

/// Arrival penalty (seconds) for a rising RTT trend.
///
/// `rtt_velocity` is in **ms per Kalman update** (see `VELOCITY_PENALTY_FACTOR`).
/// A zero, negative (RTT falling) or non-finite (never-measured / degenerate
/// filter) velocity contributes nothing.
#[inline]
fn velocity_penalty_s(rtt_velocity: f64) -> f64 {
    if rtt_velocity.is_finite() && rtt_velocity > 0.0 {
        rtt_velocity * VELOCITY_PENALTY_FACTOR
    } else {
        0.0
    }
}

/// Arrival penalty (seconds) for standing queue beyond `BDP_OVERRUN_MULT` × BDP.
///
/// Returns `0.0` when the link is under the cap or when the inputs are
/// degenerate. Never signals exclusion — the caller adds this to the arrival.
#[inline]
fn bdp_overrun_penalty_s(effective_capacity: f64, propagation_s: f64, in_flight_bytes: f64) -> f64 {
    // Documented 1 ms floor — a zero/unset RTT must not silently disable the cap.
    let propagation_floor_s = propagation_s.max(MIN_PROPAGATION_S);
    let bdp_bytes = effective_capacity * propagation_floor_s;
    let cap_bytes = bdp_bytes * BDP_OVERRUN_MULT;
    if bdp_bytes.is_finite() && bdp_bytes > 0.0 && in_flight_bytes > cap_bytes {
        (in_flight_bytes - cap_bytes) / effective_capacity
    } else {
        0.0
    }
}

/// Compute predicted arrival time for a connection.
///
/// Returns `None` ONLY for a disconnected link. Every connected link yields a
/// value: a degenerate capacity or a non-finite intermediate ranks the link LAST
/// (`f64::MAX`) instead of removing it from the candidate set, so the pool can
/// never empty out and strand the scheduler with nothing to select.
fn predicted_arrival(conn: &SrtlaConnection, pkt_size: usize) -> Option<f64> {
    if !conn.connected {
        return None;
    }

    let measured_bps = conn.bitrate.current_bitrate_bps;
    let bitrate_bps = if measured_bps > 0.0 {
        measured_bps
    } else {
        BOOTSTRAP_CAPACITY_BPS
    };
    let capacity_bytes_per_sec = bitrate_bps / 8.0;

    // Loss from quality multiplier
    let loss = (1.0 - conn.quality_cache.multiplier).clamp(0.0, 0.99);
    let effective_capacity = capacity_bytes_per_sec * (1.0 - loss);
    if !effective_capacity.is_finite() || effective_capacity <= 0.0 {
        // Rank last, but stay selectable.
        return Some(f64::MAX);
    }

    // Outstanding bytes INCLUDE packets already assigned to this link but still
    // sitting in its batch queue; ignoring them would hide up to a full flush
    // batch of committed DATA from the BDP cap.
    let outstanding_packets = (conn.in_flight_packets.max(0) as u64)
        .saturating_add(conn.batch_sender.queued_count().max(0) as u64);
    let in_flight_bytes = outstanding_packets.saturating_mul(SRT_PKT_SIZE as u64) as f64;

    // Use Kalman-smoothed RTT as propagation delay estimate.
    // Falls back to rtt_min_ms if Kalman hasn't initialized yet.
    let smooth_rtt = conn.rtt.kalman_rtt.value();
    let propagation_s = if smooth_rtt > 0.0 {
        smooth_rtt / 1000.0
    } else {
        conn.rtt.rtt_min_ms / 1000.0
    };

    let arrival = (in_flight_bytes + pkt_size as f64) / effective_capacity
        + propagation_s
        + velocity_penalty_s(conn.get_rtt_velocity())
        + bdp_overrun_penalty_s(effective_capacity, propagation_s, in_flight_bytes);
    if !arrival.is_finite() {
        return Some(f64::MAX);
    }
    Some(arrival)
}

/// Select the connection with lowest predicted arrival time from all connections.
///
/// The argmin admits the first candidate unconditionally (`best_idx.is_none()`)
/// rather than comparing against a `f64::MAX` seed, so a pool in which every
/// link ranks last still returns the lowest-index candidate instead of `None`.
/// Ties break deterministically to the lowest index.
pub fn select_from(conns: &[SrtlaConnection], pkt_size: usize) -> Option<usize> {
    let mut best_idx = None;
    let mut best_arrival = f64::MAX;

    for (i, conn) in conns.iter().enumerate() {
        if let Some(arrival) = predicted_arrival(conn, pkt_size)
            && (best_idx.is_none() || arrival < best_arrival)
        {
            best_arrival = arrival;
            best_idx = Some(i);
        }
    }

    best_idx
}

/// Select the connection with lowest predicted arrival time from a filtered subset.
///
/// `indices` contains the indices of candidate connections in `conns`.
pub fn select_from_indices(
    conns: &[SrtlaConnection],
    indices: &[usize],
    pkt_size: usize,
) -> Option<usize> {
    let mut best_idx = None;
    let mut best_arrival = f64::MAX;

    for &i in indices {
        if i < conns.len()
            && let Some(arrival) = predicted_arrival(&conns[i], pkt_size)
            && (best_idx.is_none() || arrival < best_arrival)
        {
            best_arrival = arrival;
            best_idx = Some(i);
        }
    }

    best_idx
}

/// Compute predicted arrival time for a connection (public for IoDS integration).
pub fn arrival_time(conn: &SrtlaConnection, pkt_size: usize) -> Option<f64> {
    predicted_arrival(conn, pkt_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_connections;

    #[test]
    fn test_select_prefers_lower_arrival() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(3));

        // Make conn 1 have lowest arrival (low in-flight, high bitrate, low RTT)
        conns[0].in_flight_packets = 10;
        conns[0].bitrate.current_bitrate_bps = 1_000_000.0;
        conns[0].rtt.rtt_min_ms = 50.0;

        conns[1].in_flight_packets = 0;
        conns[1].bitrate.current_bitrate_bps = 2_000_000.0;
        conns[1].rtt.rtt_min_ms = 20.0;

        conns[2].in_flight_packets = 20;
        conns[2].bitrate.current_bitrate_bps = 500_000.0;
        conns[2].rtt.rtt_min_ms = 100.0;

        let result = select_from(&conns, SRT_PKT_SIZE);
        assert_eq!(
            result,
            Some(1),
            "Should pick conn with lowest predicted arrival"
        );
    }

    #[test]
    fn test_select_skips_disconnected() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));

        conns[0].connected = false;
        conns[0].bitrate.current_bitrate_bps = 10_000_000.0;

        conns[1].in_flight_packets = 5;
        conns[1].bitrate.current_bitrate_bps = 1_000_000.0;
        conns[1].rtt.rtt_min_ms = 50.0;

        let result = select_from(&conns, SRT_PKT_SIZE);
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_select_empty() {
        let conns: Vec<SrtlaConnection> = vec![];
        assert_eq!(select_from(&conns, SRT_PKT_SIZE), None);
    }

    #[test]
    fn test_select_from_indices() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(3));

        conns[0].in_flight_packets = 0;
        conns[0].bitrate.current_bitrate_bps = 5_000_000.0;
        conns[0].rtt.rtt_min_ms = 10.0;

        conns[1].in_flight_packets = 0;
        conns[1].bitrate.current_bitrate_bps = 1_000_000.0;
        conns[1].rtt.rtt_min_ms = 50.0;

        conns[2].in_flight_packets = 0;
        conns[2].bitrate.current_bitrate_bps = 2_000_000.0;
        conns[2].rtt.rtt_min_ms = 20.0;

        // Only consider indices 1 and 2 (exclude the best one, 0)
        let result = select_from_indices(&conns, &[1, 2], SRT_PKT_SIZE);
        assert_eq!(result, Some(2), "Should pick best from subset");
    }

    #[test]
    fn predicted_arrival_saturates_huge_in_flight() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(1));

        conns[0].in_flight_packets = i32::MAX;
        conns[0].bitrate.current_bitrate_bps = 1_000_000.0;
        conns[0].rtt.rtt_min_ms = 50.0;

        let arrival = predicted_arrival(&conns[0], SRT_PKT_SIZE)
            .expect("huge in-flight must still yield a value");
        assert!(
            arrival.is_finite(),
            "saturating widen must keep arrival finite, got {arrival}"
        );
    }

    #[test]
    fn predicted_arrival_never_nan() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));

        conns[0].bitrate.current_bitrate_bps = 0.0;
        conns[1].bitrate.current_bitrate_bps = -1.0;

        for conn in &conns {
            let arrival = predicted_arrival(conn, SRT_PKT_SIZE)
                .expect("an unmeasured link must still yield an arrival estimate");
            assert!(
                arrival.is_finite() && arrival > 0.0,
                "non-positive bitrate must fall back to a finite estimate, got {arrival}"
            );
        }
    }

    #[test]
    fn select_bootstraps_when_no_link_has_a_measured_bitrate() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));

        for conn in conns.iter_mut() {
            conn.connected = true;
            conn.bitrate.current_bitrate_bps = 0.0;
            conn.rtt.rtt_min_ms = 0.0;
        }

        assert!(
            select_from(&conns, SRT_PKT_SIZE).is_some(),
            "EDPF must select a link before any bitrate has been measured, otherwise no DATA is \
             ever sent and no bitrate can ever be measured"
        );
    }

    #[test]
    fn measured_link_outranks_idle_link_of_equal_in_flight() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));

        for conn in conns.iter_mut() {
            conn.connected = true;
            conn.in_flight_packets = 4;
            conn.rtt.rtt_min_ms = 30.0;
        }
        conns[0].bitrate.current_bitrate_bps = 0.0;
        conns[1].bitrate.current_bitrate_bps = BOOTSTRAP_CAPACITY_BPS * 4.0;

        assert_eq!(
            select_from(&conns, SRT_PKT_SIZE),
            Some(1),
            "a measured high-capacity link must beat the bootstrap placeholder"
        );
    }

    #[test]
    fn velocity_penalty_only_applies_to_a_rising_finite_trend() {
        assert_eq!(velocity_penalty_s(0.0), 0.0);
        assert_eq!(velocity_penalty_s(-4.0), 0.0);
        assert_eq!(velocity_penalty_s(f64::NAN), 0.0);
        assert_eq!(velocity_penalty_s(f64::INFINITY), 0.0);
        assert_eq!(velocity_penalty_s(f64::NEG_INFINITY), 0.0);
        assert!((velocity_penalty_s(4.0) - 4.0 * VELOCITY_PENALTY_FACTOR).abs() < 1e-12);
    }

    #[test]
    fn bdp_overrun_penalty_is_zero_under_the_cap_and_grows_above_it() {
        let capacity = 125_000.0;
        let propagation_s = 0.030;
        let cap_bytes = capacity * propagation_s * BDP_OVERRUN_MULT;

        assert_eq!(
            bdp_overrun_penalty_s(capacity, propagation_s, cap_bytes),
            0.0,
            "exactly at the cap is not an overrun"
        );
        let over = cap_bytes + capacity;
        assert!(
            (bdp_overrun_penalty_s(capacity, propagation_s, over) - 1.0).abs() < 1e-9,
            "one extra second of capacity above the cap must cost one second"
        );
    }

    #[test]
    fn bdp_cap_stays_active_on_a_zero_rtt_link() {
        let capacity = 125_000.0;
        let bytes_over_floor_cap = capacity * MIN_PROPAGATION_S * BDP_OVERRUN_MULT * 4.0;

        assert!(
            bdp_overrun_penalty_s(capacity, 0.0, bytes_over_floor_cap) > 0.0,
            "the 1 ms propagation floor must keep the BDP cap active on a zero/unset-RTT link"
        );
    }

    #[test]
    fn zero_rtt_link_with_deep_backlog_is_penalized() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));

        for conn in conns.iter_mut() {
            conn.connected = true;
            conn.bitrate.current_bitrate_bps = 1_000_000.0;
            conn.rtt.rtt_min_ms = 0.0;
        }
        conns[0].in_flight_packets = 200;
        conns[1].in_flight_packets = 0;

        let deep = predicted_arrival(&conns[0], SRT_PKT_SIZE).unwrap();
        let idle = predicted_arrival(&conns[1], SRT_PKT_SIZE).unwrap();
        let capacity = 1_000_000.0 / 8.0;
        let base = (200.0 * SRT_PKT_SIZE as f64 + SRT_PKT_SIZE as f64) / capacity;

        assert!(
            deep > base,
            "a zero-RTT link over the floored BDP cap must carry an overrun penalty ({deep} vs \
             base {base})"
        );
        assert!(deep > idle);
    }

    #[test]
    fn rising_rtt_adds_exactly_the_velocity_penalty() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(1));

        conns[0].connected = true;
        conns[0].bitrate.current_bitrate_bps = 1_000_000.0;
        for step in 0..12 {
            conns[0].rtt.kalman_rtt.update(30.0 + f64::from(step) * 5.0);
        }

        let velocity = conns[0].get_rtt_velocity();
        assert!(
            velocity > 0.0,
            "a rising RTT ramp must yield a positive velocity, got {velocity}"
        );

        let capacity = 1_000_000.0 / 8.0;
        let expected = SRT_PKT_SIZE as f64 / capacity
            + conns[0].rtt.kalman_rtt.value() / 1000.0
            + velocity * VELOCITY_PENALTY_FACTOR;
        let arrival = predicted_arrival(&conns[0], SRT_PKT_SIZE).unwrap();

        assert!(
            (arrival - expected).abs() < 1e-12,
            "arrival {arrival} must equal the base prediction plus the velocity penalty {expected}"
        );
    }

    #[test]
    fn queued_packets_count_toward_the_bdp_cap() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));

        for conn in conns.iter_mut() {
            conn.connected = true;
            conn.in_flight_packets = 4;
            conn.rtt.rtt_min_ms = 30.0;
        }
        let payload = [0_u8; SRT_PKT_SIZE];
        for _ in 0..8 {
            conns[1].queue_data_packet(&payload, None, 0);
        }
        for conn in conns.iter_mut() {
            conn.bitrate.current_bitrate_bps = 1_000_000.0;
        }

        let without_queue = predicted_arrival(&conns[0], SRT_PKT_SIZE).unwrap();
        let with_queue = predicted_arrival(&conns[1], SRT_PKT_SIZE).unwrap();

        assert!(
            with_queue > without_queue,
            "queued-but-unflushed packets must raise predicted arrival ({with_queue} vs \
             {without_queue})"
        );
        assert_eq!(
            select_from(&conns, SRT_PKT_SIZE),
            Some(0),
            "the link with a full batch queue must rank behind the drained one"
        );
    }

    #[test]
    fn degenerate_capacity_ranks_last_but_stays_selectable() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));

        for conn in conns.iter_mut() {
            conn.connected = true;
            conn.bitrate.current_bitrate_bps = 1_000_000.0;
            conn.rtt.rtt_min_ms = 30.0;
        }
        conns[0].quality_cache.multiplier = f64::NAN;

        assert_eq!(
            predicted_arrival(&conns[0], SRT_PKT_SIZE),
            Some(f64::MAX),
            "a NaN capacity must rank last, not drop the link from the pool"
        );
        assert_eq!(
            select_from(&conns, SRT_PKT_SIZE),
            Some(1),
            "the healthy link must outrank the degenerate one"
        );
    }

    #[test]
    fn all_degenerate_pool_still_selects_the_lowest_index() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(3));

        for conn in conns.iter_mut() {
            conn.connected = true;
            conn.quality_cache.multiplier = f64::NAN;
        }

        assert_eq!(
            select_from(&conns, SRT_PKT_SIZE),
            Some(0),
            "a pool where every link ranks f64::MAX must still yield a link, never None"
        );
        assert_eq!(
            select_from_indices(&conns, &[1, 2], SRT_PKT_SIZE),
            Some(1),
            "the subset argmin must also admit its first candidate rather than returning None"
        );
    }
}
