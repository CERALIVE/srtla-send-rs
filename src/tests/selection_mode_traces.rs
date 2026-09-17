//! Frozen before the enhanced cooldown change, through the production dispatcher.

use crate::mode::SchedulingMode;
use crate::sender::selection::adaptive::AdaptiveState;
use crate::sender::selection::{EdpfSchedulerState, select_connection_idx_with_state};
use crate::tests::adaptive_tests::{config, pool_of};
use crate::utils::test_clock::TestClock;

async fn trace(mode: SchedulingMode) -> String {
    let clock = TestClock::new(40_000);
    let mut conns = pool_of(3).await;
    for (i, conn) in conns.iter_mut().enumerate() {
        conn.rtt.kalman_rtt.update([30.0, 50.0, 150.0][i]);
        conn.bitrate.current_bitrate_bps = [1_000_000.0, 5_000_000.0, 5_000_000.0][i];
    }
    let mut cfg = config();
    cfg.mode = mode;
    let mut edpf = EdpfSchedulerState::default();
    let mut adaptive = AdaptiveState::default();
    let (mut last, mut switched) = (None, 0);
    let mut bytes = String::new();
    for packet in 0..80_u32 {
        let now = 40_000 + u64::from(packet) * 3;
        clock.set(now);
        for (i, conn) in conns.iter_mut().enumerate() {
            conn.in_flight_packets =
                i32::try_from((packet / 7 + u32::try_from(i).unwrap() * 9) % 20).unwrap();
        }
        if packet == 40 {
            conns[0].connected = false;
            conns[0].last_received = None;
        }
        let selected = select_connection_idx_with_state(
            &mut conns,
            last,
            switched,
            now,
            &cfg,
            &mut edpf,
            &mut adaptive,
        )
        .expect("live alternatives");
        assert_eq!(
            adaptive.targets.len(),
            conns.len(),
            "admit snapshots every link"
        );
        for conn in conns.iter().filter(|conn| conn.connected) {
            let weight = conn
                .adaptive
                .weight
                .expect("healthy trace link must be admitted");
            assert_eq!(weight.quality_multiplier.to_bits(), 1.1_f64.to_bits());
            assert_eq!(weight.effective_multiplier.to_bits(), 1.1_f64.to_bits());
            assert_eq!(weight.effective_multiplier / weight.quality_multiplier, 1.0);
            assert_eq!(
                weight.effective_multiplier
                    / conns[selected]
                        .adaptive
                        .weight
                        .unwrap()
                        .effective_multiplier,
                1.0
            );
            assert_eq!(
                conn.quality_cache.last_calculated_ms,
                40_000 + ((now - 40_000) / 51) * 51
            );
        }
        if last != Some(selected) {
            switched = now;
        }
        last = Some(selected);
        bytes.push(char::from(b'0' + u8::try_from(selected).unwrap()));
        conns[selected].queue_data_packet(&[0; 1316], Some(packet), now);
        if packet % 5 == 4 {
            for conn in &mut conns {
                conn.batch_sender.reset();
            }
        }
    }
    bytes
}

#[tokio::test]
async fn unaffected_modes_match_pre_fix_bytes() {
    // Given frozen baseline decisions, with backlog changes, holds and a link loss.
    let cases = [
        (
            SchedulingMode::Enhanced,
            "00000000000000222220000022222000002222202222222222222222222222222222222222222111",
        ),
        (
            SchedulingMode::Classic,
            "00000000000000222020220202202022020220202222222222222222222222222222222222222111",
        ),
        (
            SchedulingMode::RttThreshold,
            "00000000000000000000000000000000000000001111111111111111111111111111111111111111",
        ),
        (
            SchedulingMode::Edpf,
            "00010222220001122222011102222211011000221111122222111112222211111222221111122222",
        ),
        (
            SchedulingMode::Adaptive,
            "00000000000000222220000022222000002222202222222222222222222222222222222222222111",
        ),
    ];
    // When each mode runs through the production dispatcher with owned state.
    let mut actual = Vec::new();
    for (mode, _) in cases {
        actual.push(trace(mode).await);
    }
    // Then its complete decision stream remains byte-identical, not just its totals.
    assert_eq!(actual, cases.map(|(_, expected)| expected));
}

#[tokio::test]
async fn shared_admission_excludes_degraded_links_in_every_mode() {
    // Given an otherwise superior degraded link and one healthy alternative.
    use crate::connection::health::{HealthMachine, HealthState};
    let _clock = TestClock::new(10_000);
    for mode in [
        SchedulingMode::Classic,
        SchedulingMode::Enhanced,
        SchedulingMode::RttThreshold,
        SchedulingMode::Edpf,
        SchedulingMode::Adaptive,
    ] {
        let mut conns = pool_of(2).await;
        conns[0].health = HealthMachine::new(HealthState::Degraded, 0);
        conns[1].window = 1;
        let mut cfg = config();
        cfg.mode = mode;
        // When the real dispatcher selects during the incumbent's cooldown.
        let selected = select_connection_idx_with_state(
            &mut conns,
            Some(0),
            10_000,
            10_001,
            &cfg,
            &mut EdpfSchedulerState::default(),
            &mut AdaptiveState::default(),
        );
        // Then admission wins over every mode's ranking and hold behavior.
        assert_eq!(selected, Some(1), "{mode}");
    }
}

#[tokio::test]
async fn admission_weight_oracle_from_pre_lift_adaptive() {
    // Given fixed cache values and each weighting factor in the original path.
    use crate::bind_map::Priority;
    use crate::connection::health::{HealthMachine, HealthState};
    let _clock = TestClock::new(10_000);
    let mut actual = Vec::new();
    for (health, load, priority) in [
        (HealthState::Healthy, 0, 0.0),
        (HealthState::Healthy, 64, 0.2),
        (HealthState::Rejoining, 64, 0.2),
    ] {
        let mut conns = pool_of(2).await;
        conns[0].health = HealthMachine::new(health, 10_000);
        conns[0].in_flight_packets = load;
        conns[0].priority_baseline = Some(Priority::try_from(priority).unwrap());
        conns[0].quality_cache.last_calculated_ms = 10_000;
        conns[0].quality_cache.multiplier = 0.7;
        // When the original adaptive private composition runs.
        crate::sender::selection::adaptive::select(
            &mut conns,
            None,
            0,
            10_000,
            &config(),
            &mut AdaptiveState::default(),
        );
        let weight = conns[0].adaptive.weight.unwrap();
        actual.push((
            weight.base_score,
            weight.quality_multiplier.to_bits(),
            weight.effective_multiplier.to_bits(),
        ));
    }
    // Then record exact IEEE-754 results, not a reimplementation of the formula.
    assert_eq!(
        actual,
        [
            (20_000, 4_604_480_259_023_595_110, 4_604_480_259_023_595_110),
            (307, 4_604_480_259_023_595_110, 4_601_237_667_291_888_353),
            (307, 4_604_480_259_023_595_110, 4_580_701_252_991_078_891),
        ]
    );
}
