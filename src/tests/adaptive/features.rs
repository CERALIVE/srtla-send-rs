use super::*;
use crate::bind_map::Priority;

#[test]
fn adaptive_features_are_a_complete_bitset() {
    // Given the eight independent switches, when combined, then they equal default ALL.
    let bits = [
        AdaptiveFeatures::STALL,
        AdaptiveFeatures::LOSS,
        AdaptiveFeatures::QUEUE,
        AdaptiveFeatures::DEADLINE,
        AdaptiveFeatures::REJOIN,
        AdaptiveFeatures::SOLE,
        AdaptiveFeatures::PREF,
        AdaptiveFeatures::RATECAP,
        AdaptiveFeatures::QUALITY,
    ];
    assert_eq!(AdaptiveFeatures::default(), AdaptiveFeatures::ALL);
    assert_eq!(
        bits.into_iter().fold(0, |value, bit| value | bit.bits()),
        511_u16
    );
    for bit in bits {
        assert!(!(AdaptiveFeatures::ALL - bit).contains(bit));
    }
}

fn share(conns: &mut [SrtlaConnection], state: &mut AdaptiveState) -> Vec<u8> {
    let mut outstanding = std::collections::VecDeque::new();
    let mut trace = Vec::new();
    let (mut last, mut switched) = (None, 0);
    for i in 0..1000 {
        let now = 10_000 + i * 20;
        let selected = adaptive::select(conns, last, switched, now, &config(), state).unwrap();
        if last != Some(selected) {
            switched = now;
        }
        last = Some(selected);
        trace.push(u8::try_from(selected).unwrap());
        conns[selected].in_flight_packets += 1;
        outstanding.push_back(selected);
        if outstanding.len() > 8 {
            let acked = outstanding.pop_front().unwrap();
            conns[acked].in_flight_packets -= 1;
        }
    }
    trace
}

#[tokio::test]
async fn preference_bias_is_bounded_on_healthy_links() {
    // Given equal Healthy links with the full +.2 preference ramp on link zero.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].priority_baseline = Some(Priority::try_from(0.2).unwrap());
    // When a fixed-latency feedback trace distributes 1000 packets.
    let trace = share(&mut conns, &mut AdaptiveState::default());
    // Then preference shifts share without overriding capacity.
    let preferred = trace.iter().filter(|&&i| i == 0).count();
    assert!(
        (550..700).contains(&preferred),
        "preferred picks: {preferred}"
    );
}

#[tokio::test]
async fn preference_on_rejoining_is_neutral_and_ramp_governs_share() {
    // Given identical Rejoining pools, only one with a +.2 preference.
    let _clock = TestClock::new(10_000);
    let mut preferred = pool().await;
    let mut neutral = pool().await;
    for conns in [&mut preferred, &mut neutral] {
        conns[0].health = HealthMachine::new(HealthState::Rejoining, 10_000);
    }
    preferred[0].priority_baseline = Some(Priority::try_from(0.2).unwrap());
    // When evaluating the real ranking over its ramp, then priority adds no bias.
    assert_eq!(
        adaptive::preference_multiplier(
            preferred[0].effective_priority(),
            preferred[0].window,
            HealthState::Rejoining
        ),
        1.0
    );
    let trace = share(&mut preferred, &mut AdaptiveState::default());
    assert_eq!(trace, share(&mut neutral, &mut AdaptiveState::default()));
    assert!(trace[..100].iter().filter(|&&i| i == 0).count() < 50);
}

#[tokio::test]
async fn degraded_preferred_link_receives_zero_picks() {
    // Given an otherwise dominant preferred link held by health.
    let _clock = TestClock::new(10_000);
    let mut conns = pool().await;
    conns[0].priority_baseline = Some(Priority::try_from(0.2).unwrap());
    conns[0].health = HealthMachine::new(HealthState::Degraded, 0);
    // When offering 1000 packets, then preference cannot bypass admission.
    assert_eq!(
        share(&mut conns, &mut AdaptiveState::default()),
        vec![1; 1000]
    );
}
