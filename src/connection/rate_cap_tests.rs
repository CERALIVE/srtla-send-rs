use super::delivery::{DataSend, DeliveryAck, DeliveryLedger};
use super::rate_cap::{ClimbMode, RateCap, RateSignals, RateState};

#[path = "rate_cap_backoff_tests.rs"]
mod backoff;
#[path = "rate_cap_episode_tests.rs"]
mod episodes;

struct Rig {
    cap: RateCap,
    signals: RateSignals,
}

impl Rig {
    fn new() -> Self {
        Self {
            cap: RateCap::default(),
            signals: RateSignals {
                now_ms: 0,
                srtt_ms: 100.0,
                rtt_min_ms: 100.0,
                loss_ewma: Some(0.0),
                queue_delay_ms: 0.0,
                velocity_ms_per_update: 1.0,
                jitter_ms: 0.0,
            },
        }
    }

    // Real ACK credits, not a mock rate: fixed 2s window means bytes * 4 = bps.
    fn tick(&mut self, delivered_bps: u32) -> f64 {
        self.signals.now_ms += 1000;
        let mut ledger = DeliveryLedger::default();
        let mut bytes = delivered_bps / 4;
        let mut seq = 0;
        while bytes > 0 {
            let len = u16::try_from(bytes.min(1316)).unwrap();
            ledger.record_sent(
                seq,
                DataSend {
                    sent_ms: self.signals.now_ms,
                    len,
                },
            );
            assert!(ledger.acknowledge(
                DeliveryAck {
                    seq,
                    socket_generation: 0
                },
                self.signals.now_ms,
            ));
            bytes -= u32::from(len);
            seq += 1;
        }
        self.cap.tick(&ledger, &self.signals);
        self.cap.target_bps()
    }
}

fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < expected.abs().max(1.0) * 1e-12,
        "actual={actual}, expected={expected}"
    );
}

#[test]
fn bootstrap_uses_first_delivered_rate_with_one_mbps_floor() {
    // Given fresh controllers, including a previously idle one.
    for first in [100_000, 2_000_000] {
        let mut rig = Rig::new();
        rig.tick(0);
        // When the first DATA delivery is observed.
        let target = rig.tick(first);
        // Then bootstrap seeds once, without also applying a climb increment.
        close(target, f64::from(first.max(1_000_000)));
    }
}

#[test]
fn normal_climbing_grows_two_percent_per_tick() {
    // Given a bootstrapped path with a changing RTT trend.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    // When it receives a housekeeping tick with delivery.
    let target = rig.tick(1_000_000);
    // Then normal climb applies exactly once.
    close(target, 1_020_000.0);
    assert_eq!(
        rig.cap.state(),
        RateState::Climbing {
            sub: ClimbMode::Normal
        }
    );
}

#[test]
fn hai_requires_near_zero_velocity_low_jitter_and_clear_queue() {
    // Given each boundary of the HAI predicate (velocity unit is ms/update).
    for (velocity, jitter, queue, factor) in [
        (0.0, 10.0, 0.0, 1.06),
        (0.1, 10.0, 0.0, 1.06),
        (-0.1, 10.0, 0.0, 1.06),
        (0.101, 10.0, 0.0, 1.02),
        (-0.101, 10.0, 0.0, 1.02),
        (0.0, 10.01, 0.0, 1.02),
        (0.0, 0.0, 1.0, 1.02),
    ] {
        let mut rig = Rig::new();
        rig.tick(1_000_000);
        rig.signals.velocity_ms_per_update = velocity;
        rig.signals.jitter_ms = jitter;
        rig.signals.queue_delay_ms = queue;
        // When climbing with the supplied path evidence.
        let target = rig.tick(1_000_000);
        // Then acceleration requires every stability condition.
        close(target, 1_000_000.0 * factor);
    }
}

#[test]
fn holding_starts_strictly_above_one_and_a_half_baseline() {
    // Given the threshold and a value just above it.
    for (srtt, expected) in [(150.0, 1_020_000.0), (150.01, 1_000_000.0)] {
        let mut rig = Rig::new();
        rig.tick(1_000_000);
        rig.signals.srtt_ms = srtt;
        // When ticking with positive delivery.
        let target = rig.tick(1_000_000);
        // Then the strict delay gate holds only above the threshold.
        close(target, expected);
    }
}

#[test]
fn bdp_formula_floors_packets_and_never_falls_below_stall_threshold() {
    // Given a 2Mbps ACK-derived target.
    let mut rig = Rig::new();
    rig.tick(2_000_000);
    // When converting baseline RTT into packets.
    let caps = [0.0, 100.0, 1000.0].map(|rtt| rig.cap.bdp_cap_packets(rtt));
    // Then fractional packets round down: 2e6 * 1s /8 *1.5 /1316 = 284.95...
    assert_eq!(caps, [32, 32, 284]);
}

#[test]
fn soft_cap_penalizes_ranking_without_excluding_any_link() {
    // Given the cached 100ms baseline and a minimum 32-packet cap.
    let mut rig = Rig::new();
    rig.tick(1_000_000);
    // When comparing below, at, and above the cap, including extreme load.
    let multipliers = [0, 32, 64, u32::MAX].map(|load| rig.cap.soft_cap_multiplier(load));
    // Then every score remains positive, with no admission verdict.
    assert_eq!(&multipliers[..3], &[1.0, 1.0, 0.5]);
    assert!(multipliers[3] > 0.0 && multipliers[3] < 1.0);
}

#[test]
fn r1_no_share_verdict_exists() {
    // Given a very slow but delivering link; no peer/share input exists.
    let mut rig = Rig::new();
    rig.tick(4);
    // When the controller observes continued delivery.
    rig.tick(4);
    // Then adding ANY state or climb verdict requires editing this exhaustive match.
    let is_climbing = match rig.cap.state() {
        RateState::Bootstrap | RateState::Holding | RateState::BackingOff | RateState::Drain => {
            false
        }
        RateState::Climbing { sub } => match sub {
            ClimbMode::Normal | ClimbMode::Hai | ClimbMode::FastRecovery { ticks_left: _ } => true,
        },
    };
    assert!(is_climbing);
    assert!(rig.cap.soft_cap_multiplier(u32::MAX) > 0.0);
}

#[test]
fn transmitted_bytes_without_ack_do_not_grow_target() {
    // Given accepted DATA without an ACK (transmission alone is not delivery).
    let mut rig = Rig::new();
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        7,
        DataSend {
            sent_ms: 0,
            len: 1316,
        },
    );
    rig.signals.now_ms = 1000;
    // When observing the unacknowledged ledger.
    rig.cap.tick(&ledger, &rig.signals);
    // Then target stays at bootstrap and delivered rate is zero.
    assert_eq!(rig.cap.state(), RateState::Bootstrap);
    close(rig.cap.delivered_bps(), 0.0);
    close(rig.cap.target_bps(), 1_000_000.0);
}

#[test]
fn expired_ack_window_is_idle_even_with_previous_delivery_proof() {
    // Given one real ACK credit at t=0, observed by the new controller.
    let mut rig = Rig::new();
    let mut ledger = DeliveryLedger::default();
    ledger.record_sent(
        7,
        DataSend {
            sent_ms: 0,
            len: 1316,
        },
    );
    assert!(ledger.acknowledge(
        DeliveryAck {
            seq: 7,
            socket_generation: 0
        },
        0
    ));
    rig.cap.tick(&ledger, &rig.signals);
    let target = rig.cap.target_bps();
    rig.signals.now_ms = 2000;
    // When the fixed two-second delivery window expires.
    rig.cap.tick(&ledger, &rig.signals);
    // Then historical proof does not count as a current delivered rate.
    close(rig.cap.delivered_bps(), 0.0);
    close(rig.cap.target_bps(), target);
}
