use std::time::Duration;

use super::*;
use crate::bond::CarrierMode;
use crate::impairment::ImpairmentConfig;

fn profile(events: Vec<TimedEvent>, seconds: u64) -> Profile {
    Profile {
        links: vec![LinkProfile {
            base: ImpairmentConfig::default(),
            carrier: CarrierMode::Nat,
        }],
        events,
        duration: Duration::from_secs(seconds),
    }
}

fn periodic() -> TimedEvent {
    TimedEvent::new(
        Duration::from_secs(10),
        Some(0),
        Action::Periodic {
            every: Duration::from_secs(15),
            action: Box::new(Action::DataBlackhole { on: true }),
            hold: Duration::from_secs(2),
            until: Duration::from_secs(54),
        },
    )
}

#[test]
fn periodic_expands_until_not_duration() {
    // Given: until=54 is between actual onsets 40 and 55; duration is longer.
    let p = profile(vec![periodic()], 75);
    // When: expanding the profile.
    let events = p.expanded_events().unwrap();
    // Then: only the three actual onsets and their restorations are emitted.
    assert_eq!(
        events.iter().map(|e| e.at.as_secs()).collect::<Vec<_>>(),
        [10, 12, 25, 27, 40, 42]
    );
    assert!(matches!(
        events[5].action,
        Action::DataBlackhole { on: false }
    ));
}

#[test]
fn validate_rejects_horizon_past_duration() {
    // Given: final actual onset=40, hold=2, default periodic tail=5.
    let p = profile(vec![periodic()], 46);
    // When/Then: 47 seconds is required, not 54+2+5 or merely 42.
    assert!(p.validate().is_err());
    assert!(profile(vec![periodic()], 47).validate().is_ok());
}

#[test]
fn one_shot_observation_starts_after_restore() {
    // Given: a 10-second obstruction with a 30-second post-restore tail.
    let events = vec![
        TimedEvent::new(
            Duration::from_secs(5),
            Some(0),
            Action::DataBlackhole { on: true },
        ),
        TimedEvent::new(
            Duration::from_secs(15),
            Some(0),
            Action::DataBlackhole { on: false },
        ),
    ];
    // When/Then: a window ending at 44 is too short; 45 is sufficient.
    assert!(profile(events.clone(), 44).validate().is_err());
    assert!(profile(events, 45).validate().is_ok());
}

#[test]
fn events_sorted_and_applied_once() {
    // Given: deliberately unordered events, including a same-time pair.
    let mut later = TimedEvent::new(Duration::from_millis(2), Some(0), Action::LinkUp(true));
    later.horizon = Duration::ZERO;
    let mut early = later.clone();
    early.at = Duration::ZERO;
    early.action = Action::LinkUp(false);
    let p = profile(vec![later.clone(), early.clone(), later], 0);
    let p = Profile {
        duration: Duration::from_millis(3),
        ..p
    };
    let mut scheduler = Scheduler::new(&p).unwrap();
    let mut applied = Vec::new();
    // When: running twice on the same scheduler.
    scheduler
        .run(&mut |e: &TimedEvent| {
            applied.push(e.clone());
            Ok(())
        })
        .unwrap();
    scheduler
        .run(&mut |e: &TimedEvent| {
            applied.push(e.clone());
            Ok(())
        })
        .unwrap();
    // Then: stable time ordering, no re-application, and actual monotonic timestamps.
    assert_eq!(applied.len(), 3);
    assert_eq!(applied[0].at, Duration::ZERO);
    assert_eq!(scheduler.log().entries.len(), 3);
    assert!(
        scheduler
            .log()
            .entries
            .windows(2)
            .all(|w| w[0].t_actual_ms <= w[1].t_actual_ms)
    );
}

#[test]
fn blackhole_chain_commands_match_spike() {
    // Given/When: the exact root and filter command generator.
    let commands = qdisc::installation_commands("eth0");
    // Then: all sixteen priomap zeros and the proven IP-length mask/offset stay frozen.
    assert_eq!(
        commands[0].join(" "),
        "qdisc add dev eth0 root handle 1: prio bands 2 priomap 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"
    );
    assert_eq!(
        commands[1].join(" "),
        "qdisc add dev eth0 parent 1:2 handle 20: netem loss 100%"
    );
    assert_eq!(
        qdisc::filter_command("eth0", true).join(" "),
        "filter add dev eth0 parent 1: protocol ip prio 10 u32 match u16 0x0400 0xfc00 at 2 \
         flowid 1:2"
    );
    assert_eq!(
        qdisc::filter_command("eth0", false).join(" "),
        "filter del dev eth0 parent 1: protocol ip prio 10"
    );
}

#[test]
fn apply_replaces_only_band_one() {
    // Given: an updated rate-limited impairment while the classifier is installed.
    let config = ImpairmentConfig {
        rate_kbit: Some(20_000),
        tbf_shaping: true,
        delay_ms: Some(60),
        jitter_ms: Some(0),
        loss_percent: Some(0.0),
        queue_limit: Some(1000),
        ..Default::default()
    };
    // When: generating an update.
    let commands = qdisc::apply_commands("eth0", &config).unwrap();
    // Then: neither root nor band two nor the filter is touched.
    assert_eq!(
        commands[0].join(" "),
        "qdisc replace dev eth0 parent 1:1 handle 10: tbf rate 20000kbit burst 250000 latency 1s"
    );
    assert_eq!(
        commands[1].join(" "),
        "qdisc replace dev eth0 parent 10:1 handle 11: netem delay 60ms 0ms loss 0% limit 1000"
    );
}

#[test]
fn tbf_latency_and_limit_are_emitted() {
    // Given: explicit byte-queue latency and packet queue limit plus a distribution.
    let config = ImpairmentConfig {
        rate_kbit: Some(10000),
        tbf_shaping: true,
        tbf_latency_ms: Some(70),
        queue_limit: Some(42),
        delay_ms: Some(20),
        jitter_ms: Some(5),
        delay_distribution: Some(crate::impairment::DelayDistribution::Normal),
        ..Default::default()
    };
    // When: generating the band-one chain.
    let commands = qdisc::apply_commands("eth0", &config).unwrap();
    // Then: TBF latency and netem packet limit are separate, correctly ordered parameters.
    assert_eq!(
        commands[0].join(" "),
        "qdisc replace dev eth0 parent 1:1 handle 10: tbf rate 10000kbit burst 125000 latency 70ms"
    );
    assert_eq!(
        commands[1].join(" "),
        "qdisc replace dev eth0 parent 10:1 handle 11: netem delay 20ms 5ms distribution normal \
         limit 42"
    );
}
