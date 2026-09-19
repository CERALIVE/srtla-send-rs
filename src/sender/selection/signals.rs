use std::time::Duration;

use super::{SchedulerFeatures, SchedulerShared};
use crate::connection::SrtlaConnection;
use crate::connection::adaptive::{DeadlineBudget, DeadlineGate};
use crate::connection::health::{HealthConstants, HealthState};
use crate::connection::probe::ProbeTarget;

pub(crate) fn srtt(conn: &SrtlaConnection) -> Option<f64> {
    conn.has_rtt_sample().then(|| conn.get_smooth_rtt_ms())
}

pub(crate) fn snapshot(
    conns: &mut [SrtlaConnection],
    state: &mut SchedulerShared,
    now: u64,
    features: SchedulerFeatures,
) {
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
            effective_health(conn, features)
        } else {
            HealthState::Down
        };
        let srtt_ms = srtt(conn);
        let deadline_held = if eligible && features.contains(SchedulerFeatures::DEADLINE) {
            let queue = if features.contains(SchedulerFeatures::QUEUE) {
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

fn effective_health(conn: &SrtlaConnection, features: SchedulerFeatures) -> HealthState {
    match conn.health.state() {
        HealthState::Stalled if !features.contains(SchedulerFeatures::STALL) => {
            HealthState::Healthy
        }
        HealthState::Degraded => {
            let loss = features.contains(SchedulerFeatures::LOSS) && conn.health.loss_latched();
            let queue = features.contains(SchedulerFeatures::QUEUE)
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
