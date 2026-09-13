//! Deterministic traces through the real adaptive selector (no scoring mock).

use smallvec::SmallVec;

use crate::config::{ConfigSnapshot, DynamicConfig};
use crate::connection::SrtlaConnection;
use crate::connection::health::{HealthMachine, HealthState};
use crate::sender::selection::adaptive::{self, AdaptiveFeatures, AdaptiveState};
use crate::sender::selection::{EdpfSchedulerState, select_connection_idx};
use crate::stats::SharedStats;
use crate::test_helpers::create_test_connections;
use crate::utils::test_clock::TestClock;

#[path = "adaptive/boundaries.rs"]
mod boundaries;
#[path = "adaptive/dispatch.rs"]
mod dispatch;
#[path = "adaptive/features.rs"]
mod features;
#[path = "adaptive/forwarding.rs"]
mod forwarding;
#[path = "adaptive/io.rs"]
mod io;

fn config() -> ConfigSnapshot {
    DynamicConfig::new().snapshot()
}

async fn pool() -> SmallVec<SrtlaConnection, 4> {
    let mut conns = create_test_connections(2).await;
    for conn in &mut conns {
        conn.health = HealthMachine::new(HealthState::Healthy, 0);
        conn.window = 20_000;
    }
    conns
}

fn rtt(conn: &mut SrtlaConnection, ms: f64) {
    conn.rtt.kalman_rtt.update(ms);
}

fn pick(conns: &mut [SrtlaConnection], state: &mut AdaptiveState, now: u64) -> usize {
    adaptive::select(conns, None, 0, now, &config(), state).unwrap()
}

#[tokio::test]
async fn all_healthy_trace_is_byte_identical_to_enhanced() {
    // Given identical neutral pools and independent history, with quality enabled.
    let clock = TestClock::new(10_000);
    let mut enhanced = pool().await;
    let mut adaptive = pool().await;
    let (mut old_state, mut state) = (EdpfSchedulerState::default(), AdaptiveState::default());
    let (mut old_last, mut new_last) = (None, None);
    let (mut old_switch, mut new_switch) = (0, 0);
    let (mut old_bytes, mut new_bytes) = (Vec::new(), Vec::new());
    // When 200 packets experience queued backlog, ACK-like drains and score changes.
    for packet in 0..200_u64 {
        let now = 10_000 + packet * 3;
        clock.set(now);
        for conns in [&mut enhanced, &mut adaptive] {
            for (i, conn) in conns.iter_mut().enumerate() {
                conn.in_flight_packets =
                    i32::try_from((packet / 7 + u64::try_from(i).unwrap() * 9) % 20).unwrap();
            }
        }
        let old = select_connection_idx(
            &mut enhanced,
            old_last,
            old_switch,
            now,
            &config(),
            &mut old_state,
        )
        .unwrap();
        let new = adaptive::select(
            &mut adaptive,
            new_last,
            new_switch,
            now,
            &config(),
            &mut state,
        )
        .unwrap();
        old_bytes.push(u8::try_from(old).unwrap());
        new_bytes.push(u8::try_from(new).unwrap());
        if old_last != Some(old) {
            old_switch = now;
        }
        if new_last != Some(new) {
            new_switch = now;
        }
        old_last = Some(old);
        new_last = Some(new);
        // Real queued_count participates in get_score, not a parallel formula.
        for (conns, selected) in [(&mut enhanced, old), (&mut adaptive, new)] {
            conns[selected].queue_data_packet(&[0; 16], Some(u32::try_from(packet).unwrap()), now);
            if packet % 5 == 4 {
                for conn in conns {
                    conn.batch_sender.reset();
                }
            }
        }
    }
    // Then the full ordered byte traces match, including real switches and holds.
    assert_eq!(new_bytes, old_bytes);
    assert!(old_bytes.windows(2).any(|w| w[0] != w[1]));
    assert!(old_bytes.windows(2).any(|w| w[0] == w[1]));
}

#[tokio::test]
async fn all_stalled_sole_carrier_holds_for_two_seconds() {
    // Given two stalled links and no DATA proof.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    for conn in &mut conns {
        conn.health = HealthMachine::new(HealthState::Stalled, 0);
    }
    rtt(&mut conns[0], 20.0);
    rtt(&mut conns[1], 100.0);
    let mut state = AdaptiveState::default();
    // When offers arrive throughout the mandatory hold, then the carrier never flips.
    for now in 10_000..12_000 {
        assert_eq!(pick(&mut conns, &mut state, now), 0);
    }
    assert_eq!(state.sole_carrier, Some((0, 10_000)));
}

#[tokio::test]
async fn sole_carrier_rotates_off_a_blackhole() {
    // Given A: fast blackhole; B: healthy but deadline-held, delivering probe DATA.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].health = HealthMachine::new(HealthState::Stalled, 0);
    rtt(&mut conns[0], 20.0);
    rtt(&mut conns[1], 600.0);
    let mut state = AdaptiveState::default();
    assert_eq!(pick(&mut conns, &mut state, 10_000), 0);
    conns[1].delivery.record_probe_proof(11_900);
    // When A's proof deadline expires, then B wins despite its much higher RTT.
    assert_eq!(pick(&mut conns, &mut state, 12_000), 1);
    assert!(conns[0].adaptive.sole_failed_until_proof);
    for now in [13_000, 14_000, 16_000, 18_000] {
        conns[1].delivery.record_sent(
            1,
            crate::connection::delivery::DataSend {
                sent_ms: now,
                len: 1316,
            },
        );
        assert!(conns[1].delivery.acknowledge(
            crate::connection::delivery::DeliveryAck {
                seq: 1,
                socket_generation: 0
            },
            now
        ));
        assert_eq!(pick(&mut conns, &mut state, now), 1);
    }
    // Fresh DATA proof, not elapsed time or keepalive, rehabilitates A.
    conns[0].delivery.record_probe_proof(18_001);
    assert_eq!(pick(&mut conns, &mut state, 18_001), 0);
    assert!(!conns[0].adaptive.sole_failed_until_proof);
}

#[tokio::test]
async fn cooldown_cannot_hold_a_link_that_left_the_admitted_set() {
    // Given the current link leaves via each independent admission veto.
    let _clock = TestClock::new(10_000);
    for health in [
        HealthState::Stalled,
        HealthState::Degraded,
        HealthState::Down,
        HealthState::Healthy,
    ] {
        let mut conns = pool().await;
        conns[0].health = HealthMachine::new(health, 0);
        match health {
            HealthState::Healthy => rtt(&mut conns[0], 600.0),
            HealthState::Stalled
            | HealthState::Degraded
            | HealthState::Down
            | HealthState::Rejoining => {}
        }
        // When selection is within the 15ms cooldown, then admission wins.
        assert_eq!(
            adaptive::select(
                &mut conns,
                Some(0),
                10_000,
                10_001,
                &config(),
                &mut AdaptiveState::default()
            ),
            Some(1)
        );
    }
}

#[tokio::test]
async fn deadline_uses_negotiated_budget() {
    // Given the same 2400ms link and two separately negotiated budgets.
    let _clock = TestClock::new(10_000);
    for (latency, expected) in [(2000, 1), (4000, 0)] {
        let mut conns = pool().await;
        rtt(&mut conns[0], 2400.0);
        conns[1].window = 1;
        let stats = SharedStats::new();
        stats.set_negotiated_latency_ms(latency);
        // When applying the deadline gate, then 1200ms is held only at L=2000.
        assert_eq!(
            pick(&mut conns, &mut AdaptiveState::new(stats), 10_000),
            expected
        );
    }
}

#[tokio::test]
async fn failed_sole_carriers_rotate_instead_of_repeating() {
    // Given three equally stalled blackholes with ascending RTT.
    let _clock = TestClock::new(10_000);
    let mut conns = create_test_connections(3).await;
    for (i, conn) in conns.iter_mut().enumerate() {
        conn.health = HealthMachine::new(HealthState::Stalled, 0);
        rtt(conn, 20.0 + f64::from(u32::try_from(i).unwrap()));
    }
    let mut state = AdaptiveState::default();
    // When every carrier times out without proof, then no fast blackhole monopolizes.
    let trace: Vec<_> = (0..9)
        .map(|i| pick(&mut conns, &mut state, 10_000 + i * 2000))
        .collect();
    assert_eq!(trace, [0, 1, 2, 0, 1, 2, 0, 1, 2]);
}

#[tokio::test]
async fn fallback_never_empties_a_connected_pool() {
    // Given connected but hard-ineligible links, even with negative base scores.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    for conn in &mut conns {
        conn.health = HealthMachine::new(HealthState::Down, 0);
        conn.window = -20;
    }
    conns[1].window = -10;
    // When no eligible link exists, then the highest connected base still wins.
    assert_eq!(pick(&mut conns, &mut AdaptiveState::default(), 10_000), 1);
    for conn in &mut conns {
        conn.connected = false;
    }
    assert_eq!(
        adaptive::select(
            &mut conns,
            None,
            0,
            10_000,
            &config(),
            &mut AdaptiveState::default()
        ),
        None
    );
}
