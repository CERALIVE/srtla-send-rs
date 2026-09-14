//! Per-feature-bit ablation traces through the real adaptive selector.
//!
//! Every trace runs the production `selection::adaptive::select` over ONE fixed
//! synthetic pool built from Todo 22's generator (`adaptive_tests::pool_of`).
//! Each link is engineered so exactly one feature bit decides its admission or
//! its rank, which is what makes a single-bit-off trace mean something.

use std::collections::VecDeque;

use smallvec::SmallVec;

use super::adaptive_tests::{config, pool_of};
use crate::bind_map::Priority;
use crate::connection::SrtlaConnection;
use crate::connection::health::{HealthMachine, HealthState};
use crate::sender::selection::adaptive::{self, AdaptiveFeatures, AdaptiveState};
use crate::utils::test_clock::TestClock;

/// Pinned for the whole trace so the queue-delay observation windows on link 3
/// neither expire nor drift; `now` is passed to the selector explicitly.
const CLOCK_MS: u64 = 11_000;
const RANKED_PICKS: usize = 40;
const STALLED_PICKS: usize = 20;

/// Eight links, each engineered so ONE bit decides its admission or its rank,
/// plus the all-stalled second phase that is the only way to reach the
/// sole-carrier path.
///
/// - 0 `pref` — the only link carrying a `+0.2` priority; without the bias it
///   loses the rotation to link 6, which is otherwise its twin.
/// - 1 `stall` — Stalled; `stall` off re-reads it as Healthy.
/// - 2 `loss` — Degraded with a latched loss cause; `loss` off re-reads it as
///   Healthy because its queue delay is 0.
/// - 3 `queue` — a measured 150 ms queue on a ~300 ms sRTT, so its predicted
///   arrival exceeds half the 500 ms default budget ONLY while `queue` feeds
///   the deadline gate.
/// - 4 `rejoin` — Rejoining, so only the ramp multiplier scales its rank.
/// - 5 `ratecap` — 64 packets in flight against the 32-packet floor cap, with a
///   window large enough that it wins the rotation once the cap stops halving it.
/// - 6 — the neutral competitor that gives links 0 and 5 something to lose to.
/// - 7 `deadline` — a 2400 ms sRTT alone puts it over the budget, so `deadline`
///   off admits it while `queue` off does not.
async fn ablation_pool(clock: &TestClock) -> SmallVec<SrtlaConnection, 4> {
    let mut conns = pool_of(8).await;

    conns[0].priority_baseline = Some(Priority::try_from(0.2).expect("in range"));
    conns[0].rtt.kalman_rtt.update(30.0);

    conns[1].health = HealthMachine::new(HealthState::Stalled, 0);
    conns[1].rtt.kalman_rtt.update(40.0);

    conns[2].health = HealthMachine::new(HealthState::Degraded, 0);
    conns[2].rtt.kalman_rtt.update(50.0);

    conns[3].rtt.update_estimate(200);
    clock.set(CLOCK_MS);
    conns[3].rtt.update_estimate(500);
    for _ in 0..40 {
        conns[3].rtt.kalman_rtt.update(300.0);
    }

    conns[4].health = HealthMachine::new(HealthState::Rejoining, CLOCK_MS);
    conns[4].rtt.kalman_rtt.update(60.0);

    conns[5].rtt.kalman_rtt.update(70.0);
    conns[5].in_flight_packets = 64;
    conns[5].window = 390_000;

    conns[6].rtt.kalman_rtt.update(30.0);

    conns[7].rtt.kalman_rtt.update(2400.0);

    conns
}

/// One pick per character, with real in-flight feedback: eight outstanding
/// packets drain in FIFO order, so a link that is chosen repeatedly loses rank.
fn drive(
    conns: &mut [SrtlaConnection],
    state: &mut AdaptiveState,
    start_ms: u64,
    picks: usize,
) -> String {
    let mut outstanding: VecDeque<usize> = VecDeque::new();
    let mut trace = String::new();
    let (mut last, mut switched) = (None, 0);
    for step in 0..picks {
        let now = start_ms + u64::try_from(step).expect("small") * 5;
        let selected = adaptive::select(conns, last, switched, now, &config(), state)
            .expect("a connected pool always yields a link");
        if last != Some(selected) {
            switched = now;
        }
        last = Some(selected);
        trace.push(char::from(
            b'0' + u8::try_from(selected).expect("eight links"),
        ));
        conns[selected].in_flight_packets += 1;
        outstanding.push_back(selected);
        if outstanding.len() > 8 {
            let acked = outstanding.pop_front().expect("non-empty");
            conns[acked].in_flight_packets -= 1;
        }
    }
    trace
}

async fn ablation_trace(features: AdaptiveFeatures) -> String {
    let clock = TestClock::new(10_000);
    let mut conns = ablation_pool(&clock).await;
    let mut state = AdaptiveState::default();
    state.features = features;
    let mut trace = drive(&mut conns, &mut state, CLOCK_MS, RANKED_PICKS);
    // Nothing is admitted once every link is stalled, which is the only state in
    // which the sole-carrier election runs at all.
    for conn in conns.iter_mut() {
        conn.health = HealthMachine::new(HealthState::Stalled, CLOCK_MS);
    }
    trace.push_str(&drive(
        &mut conns,
        &mut state,
        CLOCK_MS + 1000,
        STALLED_PICKS,
    ));
    trace
}

fn feature_set(label: &str) -> AdaptiveFeatures {
    match label {
        "all" => AdaptiveFeatures::ALL,
        "no-stall" => AdaptiveFeatures::ALL - AdaptiveFeatures::STALL,
        "no-loss" => AdaptiveFeatures::ALL - AdaptiveFeatures::LOSS,
        "no-queue" => AdaptiveFeatures::ALL - AdaptiveFeatures::QUEUE,
        "no-deadline" => AdaptiveFeatures::ALL - AdaptiveFeatures::DEADLINE,
        "no-rejoin" => AdaptiveFeatures::ALL - AdaptiveFeatures::REJOIN,
        "no-sole" => AdaptiveFeatures::ALL - AdaptiveFeatures::SOLE,
        "no-pref" => AdaptiveFeatures::ALL - AdaptiveFeatures::PREF,
        "no-ratecap" => AdaptiveFeatures::ALL - AdaptiveFeatures::RATECAP,
        other => panic!("unknown ablation label {other}"),
    }
}

/// Frozen selector output, sorted by label. The first 40 characters are the
/// ranked phase, the last 20 the all-stalled phase.
///
/// Regenerate deliberately: a failing row prints its own actual trace, and a
/// change here means the selector changed, not that the table is stale.
const ABLATION_GOLDEN: [(&str, &str); 9] = [
    (
        "all",
        "000666000666000666000666000666000666000600000000000000000000",
    ),
    (
        "no-deadline",
        "000333666777000333666777000333666777000300000000000000000000",
    ),
    (
        "no-loss",
        "000222666000222666000222666000222666000200000000000000000000",
    ),
    (
        "no-pref",
        "000666600000666660000066666000006666600000000000000000000000",
    ),
    (
        "no-queue",
        "000333666000333666000333666000333666000300000000000000000000",
    ),
    (
        "no-ratecap",
        "000666555000666000555666000055566600005500000000000000000000",
    ),
    (
        "no-rejoin",
        "000444666000444666000444666000444666000400000000000000000000",
    ),
    (
        "no-sole",
        "000666000666000666000666000666000666000612347123412347123412",
    ),
    (
        "no-stall",
        "000111666000111666000111666000111666000122244411122244440002",
    ),
];

/// Which single-bit-off sets this pool actually distinguishes. A bit whose row
/// equals `all` is NOT proven inert in general — it is unexercised by this pool,
/// and freezing that fact is what stops the table from quietly going uniform.
const DIVERGING: [&str; 8] = [
    "no-deadline",
    "no-loss",
    "no-pref",
    "no-queue",
    "no-ratecap",
    "no-rejoin",
    "no-sole",
    "no-stall",
];

#[test]
fn the_golden_table_is_sorted_and_covers_every_bit() {
    // Given the frozen table, when read in order, then it is sorted and total.
    let labels: Vec<&str> = ABLATION_GOLDEN.iter().map(|(label, _)| *label).collect();
    let mut sorted = labels.clone();
    sorted.sort_unstable();
    assert_eq!(labels, sorted, "the expectation table must stay sorted");
    for (label, _) in ABLATION_GOLDEN {
        let _ = feature_set(label);
    }
    assert_eq!(
        labels.len(),
        9,
        "one baseline row plus one row per feature bit"
    );
}

#[tokio::test]
async fn single_feature_off_traces_match_the_golden_table() {
    // Given the fixed synthetic pool and one row per single-feature-off set.
    let mut actual = Vec::new();
    // When the real selector drives both phases for every row.
    for (label, _) in ABLATION_GOLDEN {
        actual.push((label, ablation_trace(feature_set(label)).await));
    }
    // Then every trace is byte-identical to its frozen expectation. The whole
    // table is compared at once so one regression prints all nine rows.
    let expected: Vec<(&str, String)> = ABLATION_GOLDEN
        .iter()
        .map(|(label, trace)| (*label, (*trace).to_string()))
        .collect();
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn the_pool_distinguishes_exactly_the_frozen_feature_set() {
    // Given the baseline trace with every bit enabled.
    let baseline = ablation_trace(AdaptiveFeatures::ALL).await;
    // When each bit is removed in turn.
    let mut diverging = Vec::new();
    for (label, _) in ABLATION_GOLDEN {
        if label == "all" {
            continue;
        }
        if ablation_trace(feature_set(label)).await != baseline {
            diverging.push(label);
        }
    }
    // Then exactly the frozen subset changes the selector's output.
    assert_eq!(diverging, DIVERGING, "ablation coverage of the fixed pool");
}

#[path = "adaptive/ablation_equivalence.rs"]
mod equivalence;

#[cfg(feature = "test-internals")]
#[path = "adaptive/ablation_env.rs"]
mod env;
