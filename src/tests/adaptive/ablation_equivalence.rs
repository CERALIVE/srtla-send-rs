//! The ranking-core equivalence: adaptive with NO feature bits must reproduce
//! enhanced byte for byte, so every later divergence is attributable to a bit.

use crate::sender::selection::adaptive::{self, AdaptiveFeatures, AdaptiveState};
use crate::sender::selection::{EdpfSchedulerState, select_connection_idx};
use crate::tests::adaptive_tests::{config, pool};
use crate::utils::test_clock::TestClock;

#[tokio::test]
async fn all_off_equals_enhanced_trace() {
    // Given an all-Healthy pool, adaptive with NO feature bits, and enhanced.
    let clock = TestClock::new(10_000);
    let mut enhanced = pool().await;
    let mut ablated = pool().await;
    let (mut old_state, mut state) = (EdpfSchedulerState::default(), AdaptiveState::default());
    let cfg = crate::config::ConfigSnapshot {
        features: AdaptiveFeatures::NONE,
        ..config()
    };
    let (mut old_last, mut new_last) = (None, None);
    let (mut old_switch, mut new_switch) = (0, 0);
    let (mut old_bytes, mut new_bytes) = (Vec::new(), Vec::new());
    // When 200 packets drive both selectors through the same backlog history.
    for packet in 0..200_u64 {
        let now = 10_000 + packet * 3;
        clock.set(now);
        for conns in [&mut enhanced, &mut ablated] {
            for (i, conn) in conns.iter_mut().enumerate() {
                conn.in_flight_packets =
                    i32::try_from((packet / 7 + u64::try_from(i).expect("small") * 9) % 20)
                        .expect("in range");
            }
        }
        let old = select_connection_idx(
            &mut enhanced,
            old_last,
            old_switch,
            now,
            &cfg,
            &mut old_state,
        )
        .expect("connected pool");
        let new = adaptive::select(&mut ablated, new_last, new_switch, now, &cfg, &mut state)
            .expect("connected pool");
        old_bytes.push(u8::try_from(old).expect("two links"));
        new_bytes.push(u8::try_from(new).expect("two links"));
        if old_last != Some(old) {
            old_switch = now;
        }
        if new_last != Some(new) {
            new_switch = now;
        }
        old_last = Some(old);
        new_last = Some(new);
        for (conns, selected) in [(&mut enhanced, old), (&mut ablated, new)] {
            conns[selected].queue_data_packet(
                &[0; 16],
                Some(u32::try_from(packet).expect("in range")),
                now,
            );
            if packet % 5 == 4 {
                for conn in conns {
                    conn.batch_sender.reset();
                }
            }
        }
    }
    // Then the ranking core alone reproduces enhanced byte for byte.
    assert_eq!(new_bytes, old_bytes);
    assert!(old_bytes.windows(2).any(|w| w[0] != w[1]));
}
