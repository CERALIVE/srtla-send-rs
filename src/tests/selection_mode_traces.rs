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
