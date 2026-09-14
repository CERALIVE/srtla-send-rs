use std::time::Duration;

use super::{AdaptiveFeatures, AdaptiveState};
use crate::connection::SrtlaConnection;
use crate::connection::adaptive::{DeadlineBudget, DeadlineGate};
use crate::connection::health::{HealthConstants, HealthState};
use crate::connection::probe::ProbeTarget;

pub(super) fn srtt(conn: &SrtlaConnection) -> Option<f64> {
    conn.has_rtt_sample().then(|| conn.get_smooth_rtt_ms())
}

pub(super) fn snapshot(conns: &mut [SrtlaConnection], state: &mut AdaptiveState, now: u64) {
    let k = HealthConstants::default();
    let latency_ms = state
        .stats
        .negotiated_latency_ms()
        .unwrap_or(crate::connection::loss::UNKNOWN_LATENCY_MS);
    state.targets.clear();
    for conn in conns {
        conn.loss.set_latency_ms(latency_ms);
        conn.adaptive.observe(
            conn.delivery.socket_generation,
            conn.delivery.latest_data_proof_ms(),
        );
        let eligible = conn.connected
            && !conn.is_timed_out()
            && !matches!(conn.health.state(), HealthState::Down);
        let health = if eligible {
            effective_health(conn, state.features)
        } else {
            HealthState::Down
        };
        let srtt_ms = srtt(conn);
        let deadline_held = if eligible && state.features.contains(AdaptiveFeatures::DEADLINE) {
            let queue = if state.features.contains(AdaptiveFeatures::QUEUE) {
                conn.rtt.queue_delay_ms()
            } else {
                0.0
            };
            conn.adaptive.deadline.update(
                srtt_ms.unwrap_or(0.0) / 2.0 + queue,
                DeadlineBudget {
                    latency_ms,
                    tau_ms: k.stall_tau(srtt_ms),
                    now_ms: now,
                },
            )
        } else {
            conn.adaptive.deadline = DeadlineGate::Admitted;
            false
        };
        state.targets.push(ProbeTarget {
            conn_id: conn.conn_id,
            socket_generation: conn.delivery.socket_generation,
            health,
            deadline_held,
            sole_carrier: false,
            srtt_ms: u64::try_from(
                Duration::from_secs_f64(srtt_ms.unwrap_or(0.0) / 1000.0).as_millis(),
            )
            .unwrap_or(u64::MAX),
        });
    }
}

fn effective_health(conn: &SrtlaConnection, features: AdaptiveFeatures) -> HealthState {
    match conn.health.state() {
        HealthState::Stalled if !features.contains(AdaptiveFeatures::STALL) => HealthState::Healthy,
        HealthState::Degraded => {
            let loss = features.contains(AdaptiveFeatures::LOSS) && conn.health.loss_latched();
            let queue = features.contains(AdaptiveFeatures::QUEUE)
                && (!conn.health.loss_latched()
                    || conn.rtt.queue_delay_ms()
                        > HealthConstants::default().queue_clear(conn.rtt.slow_min_rtt_ms()));
            if loss || queue {
                HealthState::Degraded
            } else {
                HealthState::Healthy
            }
        }
        HealthState::Healthy
        | HealthState::Rejoining
        | HealthState::Down
        | HealthState::Stalled => conn.health.state(),
    }
}
