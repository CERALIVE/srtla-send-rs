//! Reference-backed benchmark scenario library.
//!
//! Citation numbers follow the approved draft; see docs/notes/bench-scenarios.md.
//! Values are netem delays verbatim, not silently halved as RTT estimates.

mod definition;
mod dynamic;
mod satellite;

use std::time::Duration;

pub use definition::{
    Profile, SourceRamp, WARMUP_SETTLE_RATIO, WARMUP_SETTLE_SECONDS, WARMUP_TIMEOUT,
};
pub use dynamic::{BENCH_SOAK, M6_PRIORITY_SIDECAR, validate_dynamic};

use crate::bond::CarrierMode;
use crate::impairment::DelayDistribution;
use crate::profile::{Action, LinkProfile, TimedEvent};
use crate::{GemodelConfig, ImpairmentConfig};

/// A: three 60±15ms, 10Mbit LTE links, loss 0.2%, limit 500; offered 80%.
/// Draft citations [6] 10.1145/3618257.3624814, [7] Mahimahi, [13] tc-netem(8).
pub fn scenario_a() -> Profile {
    Profile::new(vec![lte(60, 10_000, 15); 3], 24_000_000, 45)
}

/// B1: historical 20/60/120ms topology at 4Mbit each; offered 9.6Mbit.
/// Historical synthetic control; draft citations [6–9] motivate LTE heterogeneity.
pub fn scenario_b1() -> Profile {
    Profile::new(
        [20, 60, 120].map(|ms| lte(ms, 4_000, 0)).to_vec(),
        9_600_000,
        45,
    )
}

/// B2: 35±5/90±20ms at 8Mbit each; offered 12.8Mbit.
/// Draft citations [6–9] LTE delay range; [13] normal netem jitter.
pub fn scenario_b2() -> Profile {
    Profile::new(vec![lte(35, 8_000, 5), lte(90, 8_000, 20)], 12_800_000, 45)
}

/// C: NAT Starlink 45±5ms/20Mbit plus LTE 65ms/8Mbit/0.2%, 60s.
/// Draft [1] nature s44459-026-00044-z, [2] OASIcs.NINeS.2026.7,
/// [3] sigcomm26-dissect-starlink: +74ms/215ms and half capacity/500ms every 15s.
pub fn scenario_c() -> Profile {
    satellite::profile(60, true)
}

/// D: C plus DATA-only obstruction at 20–28s, 30s rejoin horizon, 75s total.
/// Draft [1–3], [13] tc-netem(8), [24] tc-u32(8); hypothesis from the
/// workspace srtla-starlink-lan-diagnosis note, not hardware validation.
pub fn scenario_d() -> Profile {
    let mut profile = satellite::profile(75, true);
    profile.timeline.events.extend([
        edge(20, Some(0), Action::DataBlackhole { on: true }),
        TimedEvent {
            horizon: Duration::ZERO,
            ..edge(28, Some(0), Action::DataBlackhole { on: false })
        },
    ]);
    profile
}

/// E: 10Mbit TBF latency 2s, netem 60ms/limit 1000, 9Mbit cross-load at 15–45s.
/// Healthy second link 8Mbit; offered 14.4Mbit. Draft [9] Alfredsson WoWMoM13,
/// [13] tc-netem(8): synthetic bufferbloat stress, not a queue-delay guarantee.
pub fn scenario_e() -> Profile {
    let mut loaded = lte(60, 10_000, 0);
    loaded.base.tbf_latency_ms = Some(2_000);
    loaded.base.queue_limit = Some(1_000);
    let mut profile = Profile::new(vec![loaded, lte(60, 8_000, 0)], 14_400_000, 75);
    profile.timeline.events.extend([
        TimedEvent {
            horizon: Duration::from_secs(30),
            ..edge(15, Some(0), Action::CrossTraffic { mbit: 9, on: true })
        },
        edge(45, Some(0), Action::CrossTraffic { mbit: 0, on: false }),
    ]);
    profile
}

/// F: two 60ms/8Mbit links, link 1 GE 1%/0.2/0.5/0.01; offered 12.8Mbit.
/// Draft [6–9] LTE usable-loss stress (inferred), [13] tc-netem(8) GE argument order.
pub fn scenario_f() -> Profile {
    let mut profile = Profile::new(vec![lte(60, 8_000, 0); 2], 12_800_000, 45);
    profile.timeline.links[1].base.gemodel = Some(GemodelConfig {
        p: 1.0,
        r: 0.2,
        one_h: 0.5,
        one_k: 0.01,
    });
    profile
}

/// G: marginal 60ms/1Mbit GE 5%/0.1/0.8/0.02 beside three 60ms/5Mbit links.
/// Offered 12.8Mbit; draft [6–9] inferred weak LTE stress, [13] tc-netem(8).
pub fn scenario_g() -> Profile {
    let mut links = vec![lte(60, 1_000, 0)];
    links[0].base.gemodel = Some(GemodelConfig {
        p: 5.0,
        r: 0.1,
        one_h: 0.8,
        one_k: 0.02,
    });
    links.extend(vec![lte(60, 5_000, 0); 3]);
    Profile::new(links, 12_800_000, 45)
}

/// H: three 60ms/5Mbit links, offered 12Mbit, lifecycle faults over 90s.
/// Synthetic lifecycle control; draft [13], [24] and the workspace diagnosis note.
/// The final reorder has a 20s horizon (70+20=90); other recovery horizons are 30s.
pub fn scenario_h() -> Profile {
    let mut profile = Profile::new(vec![lte(60, 5_000, 0); 3], 12_000_000, 90);
    profile.timeline.events.extend([
        edge(15, Some(2), Action::LinkUp(false)),
        TimedEvent {
            horizon: Duration::ZERO,
            ..edge(25, Some(2), Action::LinkUp(true))
        },
        edge(35, Some(1), Action::Replug),
        edge(50, Some(0), Action::DefaultRoute(false)),
        TimedEvent {
            horizon: Duration::ZERO,
            ..edge(60, Some(0), Action::DefaultRoute(true))
        },
        TimedEvent {
            horizon: Duration::from_secs(20),
            ..edge(70, None, Action::SighupReorder(vec![2, 0, 1]))
        },
    ]);
    profile
}

/// I: two 60ms/8Mbit links, offered 12.8Mbit; receiver restart at 20s, 60s total.
/// Draft [20–23] SRT/receiver references; restart gap is an ungraded control.
/// The runner must kill/respawn within two seconds, not wait for registration.
pub fn scenario_i() -> Profile {
    let mut profile = Profile::new(vec![lte(60, 8_000, 0); 2], 12_800_000, 60);
    profile.receiver_restart_budget = Some(Duration::from_secs(2));
    profile.timeline.events.push(TimedEvent {
        graded: false,
        ..edge(20, None, Action::ReceiverRestart)
    });
    profile
}

/// J: two 60ms/8Mbit links, offered 12.8Mbit; 100% loss on BOTH at 20–23s.
/// Draft [6–9] inferred stalls and [13] tc-netem(8); forced gap is ungraded.
pub fn scenario_j() -> Profile {
    let mut profile = Profile::new(vec![lte(60, 8_000, 0); 2], 12_800_000, 60);
    for (link, base) in profile.timeline.links.iter().enumerate() {
        let loss = ImpairmentConfig {
            loss_percent: Some(100.0),
            ..base.base.clone()
        };
        profile.timeline.events.extend([
            TimedEvent {
                graded: false,
                ..edge(20, Some(link), Action::SetImpairment(loss))
            },
            TimedEvent {
                graded: false,
                horizon: Duration::ZERO,
                ..edge(23, Some(link), Action::SetImpairment(base.base.clone()))
            },
        ]);
    }
    profile
}

/// K: 15s Starlink half-capacity/500ms cycle plus LTE random walk, seed 42, step 2s.
/// Draft [1–3] satellite cycle and [6–9] synthetic LTE bounds; offered 12Mbit.
pub fn scenario_k() -> Profile {
    satellite::random_walk()
}

/// L: two 60ms/8Mbit links; warm-up 12.8Mbit, overload 20Mbit at t=0 for 20s,
/// idle 10s, then 0→10Mbit/400ms with a 10s acquisition horizon; 60s total.
/// Draft [13] rate-controlled synthetic load; [20–23] SRT metric guidance.
pub fn scenario_l() -> Profile {
    let mut profile = Profile::new(vec![lte(60, 8_000, 0); 2], 20_000_000, 60);
    profile.warmup_offered_bps = 12_800_000;
    profile.timeline.events.extend([
        TimedEvent {
            graded: false,
            ..edge(20, None, Action::OfferedRate { bps: 0 })
        },
        TimedEvent {
            horizon: Duration::from_secs(10),
            ..edge(30, None, Action::OfferedRate { bps: 10_000_000 })
        },
    ]);
    profile.source_ramp = Some(SourceRamp {
        at: Duration::from_secs(30),
        duration: Duration::from_millis(400),
        from_bps: 0,
        to_bps: 10_000_000,
    });
    profile
}

/// Thirteen concrete profiles in stable family order (B expands to B1/B2).
pub fn all() -> Vec<(&'static str, Profile)> {
    vec![
        ("A", scenario_a()),
        ("B1", scenario_b1()),
        ("B2", scenario_b2()),
        ("C", scenario_c()),
        ("D", scenario_d()),
        ("E", scenario_e()),
        ("F", scenario_f()),
        ("G", scenario_g()),
        ("H", scenario_h()),
        ("I", scenario_i()),
        ("J", scenario_j()),
        ("K", scenario_k()),
        ("L", scenario_l()),
        ("M1", dynamic::scenario_m1()),
        ("M2", dynamic::scenario_m2()),
        ("M3", dynamic::scenario_m3()),
        ("M4", dynamic::scenario_m4()),
        ("M5", dynamic::scenario_m5()),
        ("M6", dynamic::scenario_m6()),
        ("M7", dynamic::scenario_m7()),
        ("M8", dynamic::scenario_m8()),
    ]
}

fn lte(delay_ms: u32, rate_kbit: u64, jitter_ms: u32) -> LinkProfile {
    LinkProfile {
        base: ImpairmentConfig {
            delay_ms: Some(delay_ms),
            rate_kbit: Some(rate_kbit),
            jitter_ms: (jitter_ms > 0).then_some(jitter_ms),
            delay_distribution: (jitter_ms > 0).then_some(DelayDistribution::Normal),
            loss_percent: Some(0.2),
            queue_limit: Some(500),
            tbf_shaping: true,
            ..Default::default()
        },
        carrier: CarrierMode::Direct,
    }
}

fn edge(at_secs: u64, link: Option<usize>, action: Action) -> TimedEvent {
    TimedEvent::new(Duration::from_secs(at_secs), link, action)
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod waveform_tests;
