// allow: SIZE_OK — existing housekeeping orchestrator plus its tests; lifecycle sampling stays in the lane-owned tick module.
use std::collections::HashMap;

use anyhow::{Result, anyhow};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Instant;
use tracing::{debug, error, info, warn};

use super::egress_tick::{TickFlow, handle_egress};
use super::sequence::SequenceTracker;
use super::uplink::{ConnectionId, ReaderHandle, UplinkPacket, restart_reader_for};
use crate::connection::adaptive::DeadlineGate;
use crate::connection::health::{HealthConstants, HealthSignals, HealthState};
use crate::connection::probe::ProbeTarget;
use crate::connection::rate_cap::RateSignals;
use crate::connection::{STARTUP_GRACE_MS, SrtlaConnection};
use crate::registration::SrtlaRegistrationManager;
use crate::utils::now_ms;

pub const GLOBAL_TIMEOUT_MS: u64 = 10_000;

#[derive(Default)]
pub(crate) struct HealthTicker {
    observations: HashMap<ConnectionId, HealthObservation>,
}

struct HealthObservation {
    generation: u32,
    reported: HealthState,
    last_log_ms: Option<u64>,
    last_rate_tick_ms: Option<u64>,
}

/// Handle periodic housekeeping tasks.
///
/// With the ring buffer sequence tracker, we no longer need periodic cleanup
/// since old entries are naturally overwritten.
#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub async fn handle_housekeeping(
    connections: &mut [SrtlaConnection],
    reg: &mut SrtlaRegistrationManager,
    classic: bool,
    all_failed_at: &mut Option<Instant>,
    reader_handles: &mut HashMap<ConnectionId, ReaderHandle>,
    packet_tx: &UnboundedSender<UplinkPacket>,
    seq_tracker: &mut SequenceTracker,
) -> Result<()> {
    handle_housekeeping_with_targets(
        connections,
        reg,
        classic,
        all_failed_at,
        reader_handles,
        packet_tx,
        seq_tracker,
        &[],
        &mut HealthTicker::default(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn handle_housekeeping_with_targets(
    connections: &mut [SrtlaConnection],
    reg: &mut SrtlaRegistrationManager,
    classic: bool,
    all_failed_at: &mut Option<Instant>,
    reader_handles: &mut HashMap<ConnectionId, ReaderHandle>,
    packet_tx: &UnboundedSender<UplinkPacket>,
    seq_tracker: &mut SequenceTracker,
    targets: &[ProbeTarget],
    health_ticks: &mut HealthTicker,
) -> Result<()> {
    // If we're waiting on a REG2 response past the timeout, proactively retry REG1
    let current_ms = now_ms();
    let _ = reg.clear_pending_if_timed_out(current_ms);

    if reg.is_probing() {
        let was_probing = true;
        reg.check_probing_complete();
        // If probing just completed, reset grace period for the selected connection
        if !reg.is_probing()
            && was_probing
            && let Some(idx) = reg.get_selected_connection_idx()
            && let Some(conn) = connections.get_mut(idx)
        {
            conn.reconnection.startup_grace_deadline_ms = current_ms + STARTUP_GRACE_MS;
            debug!(
                "{}: Reset grace period after being selected for initial registration",
                conn.label
            );
        }
    }

    // housekeeping: drive registration, send keepalives
    for (i, conn) in connections.iter_mut().enumerate() {
        if handle_egress(conn, reader_handles, packet_tx, seq_tracker).await == TickFlow::Skip {
            continue;
        }

        // Simple reconnect-on-timeout, then allow reg driver to proceed
        if conn.is_timed_out() {
            if conn.should_attempt_reconnect() {
                let label = conn.label.clone();
                conn.record_reconnect_attempt();
                if conn.connection_established_ms() == 0 {
                    debug!("{} initial registration timed out; retrying", label);
                } else {
                    warn!("{} timed out; attempting full socket reconnection", label);
                }
                // Perform full socket reconnection
                if let Err(e) = conn.reconnect().await {
                    warn!("{} failed to reconnect: {}", label, e);
                    // Fall back to mark_for_recovery if reconnect fails
                    conn.mark_for_recovery();
                }
                // Both arms discard this link's in-flight packet log, so any
                // sequence it still owns in the tracker would misattribute a
                // late NAK to a link that never transmitted it.
                seq_tracker.remove_connection(conn.conn_id);

                match reg.pending_reg2_idx() {
                    Some(idx) if idx == i => {
                        info!("{} marked for recovery; re-sending REG1", label);
                        reg.send_reg1_to(i, conn).await;
                    }
                    Some(_) => {
                        debug!(
                            "{} timed out but another uplink is awaiting REG2; deferring",
                            label
                        );
                    }
                    None => {
                        info!("{} marked for recovery; re-sending REG2", label);
                        reg.send_reg2_to(i, conn).await;
                    }
                }
            } else {
                debug!("{} timed out but in retry interval", conn.label);
            }
            restart_reader_for(conn, reader_handles, packet_tx);
            continue;
        }

        if conn.needs_keepalive() {
            let _ = conn.send_keepalive().await;
        }
        if conn.needs_rtt_measurement() {
            let _ = conn.send_keepalive().await;
        }
        if !classic {
            conn.perform_window_recovery();
        }
        // Update bitrate calculation (from Android C implementation)
        conn.calculate_bitrate();

        // The reader task self-heals via CONN_TIMEOUT (15s), but a task death
        // (panic / early return) would otherwise go undetected until that window.
        // Poll its handle cheaply each tick and respawn it for a still-active link
        // so inbound ACK/NAK/keepalive traffic resumes immediately, not 15s later.
        let reader_dead = reader_handles
            .get(&conn.conn_id)
            .is_some_and(|reader| reader.handle.is_finished());
        let reader_stale = health_ticks
            .observations
            .get(&conn.conn_id)
            .is_none_or(|observation| observation.generation != conn.delivery.socket_generation);
        if reader_dead || reader_stale {
            if reader_dead {
                warn!("{}: uplink reader task ended; restarting", conn.label);
            }
            restart_reader_for(conn, reader_handles, packet_tx);
        }
    }

    // No early-exit link may skip expiry or a hard-failure health observation.
    health_ticks.tick(connections, targets, now_ms());

    // Update active connections count (matches C implementation behavior)
    // C code resets active_connections=0 then counts non-timed-out connections
    reg.update_active_connections(connections);

    // drive registration (send REG1/REG2 as needed)
    reg.reg_driver_send_if_needed(connections).await;

    // Check for connection failures and output appropriate error messages
    // This matches the C implementation's connection_housekeeping logic
    let active_connections = connections.iter().filter(|c| !c.is_timed_out()).count();

    if connections.is_empty() {
        // Empty pool (empty-start, awaiting a SIGHUP reload) is an idle wait, not
        // an all-links-failed recovery condition; don't arm the failure timeout.
        *all_failed_at = None;
    } else if active_connections == 0 {
        if all_failed_at.is_none() {
            // tokio::time::Instant (not std) so the all-links-failed timeout below is
            // driven by the same virtual clock the fake-clock tests advance.
            *all_failed_at = Some(Instant::now());
        }

        if reg.has_connected {
            error!("warning: no available connections");
        }

        // Timeout when all connections have failed. Measure elapsed-time-since-failure
        // (`failed_at.elapsed()`) so a transient all-down blip only trips after a full
        // GLOBAL_TIMEOUT_MS of sustained failure, not the instant uptime exceeds it.
        if let Some(failed_at) = all_failed_at
            && failed_at.elapsed().as_millis() as u64 > GLOBAL_TIMEOUT_MS
        {
            if reg.has_connected {
                error!("Failed to re-establish any connections");
                return Err(anyhow!("Failed to re-establish any connections"));
            } else {
                error!("Failed to establish any initial connections");
                return Err(anyhow!("Failed to establish any initial connections"));
            }
        }
    } else {
        *all_failed_at = None;
    }

    // NOTE: With the ring buffer sequence tracker, no cleanup is needed.
    // Old entries are naturally overwritten when the buffer wraps around.

    Ok(())
}

#[cfg(test)]
pub(crate) fn tick_health(connections: &mut [SrtlaConnection], targets: &[ProbeTarget], now: u64) {
    HealthTicker::default().tick(connections, targets, now);
}

impl HealthTicker {
    /// Once per housekeeping interval, including idle and unavailable links.
    pub(crate) fn tick(
        &mut self,
        connections: &mut [SrtlaConnection],
        targets: &[ProbeTarget],
        now: u64,
    ) {
        self.observations
            .retain(|id, _| connections.iter().any(|conn| conn.conn_id == *id));
        let k = HealthConstants::default();
        let held_links = connections
            .iter()
            .filter(|conn| {
                let prior = targets.iter().find(|target| {
                    target.conn_id == conn.conn_id
                        && target.socket_generation == conn.delivery.socket_generation
                });
                ProbeTarget {
                    conn_id: conn.conn_id,
                    socket_generation: conn.delivery.socket_generation,
                    health: if conn.connected
                        && !conn.is_timed_out()
                        && !conn.is_removed()
                        && !conn.needs_rebind()
                    {
                        conn.health.state()
                    } else {
                        HealthState::Down
                    },
                    deadline_held: matches!(conn.adaptive.deadline, DeadlineGate::Held { .. }),
                    sole_carrier: prior.is_some_and(|target| target.sole_carrier),
                    srtt_ms: 0,
                }
                .eligible()
            })
            .fold(0_u32, |count, _| count.saturating_add(1));
        for conn in connections {
            let observation = self
                .observations
                .entry(conn.conn_id)
                .or_insert(HealthObservation {
                    generation: conn.delivery.socket_generation,
                    reported: conn.health.state(),
                    last_log_ms: None,
                    last_rate_tick_ms: None,
                });
            let replaced = observation.generation != conn.delivery.socket_generation;
            if replaced {
                observation.generation = conn.delivery.socket_generation;
                observation.last_rate_tick_ms = None;
            }
            conn.loss.advance(now);
            conn.advance_probes(now);
            let srtt_ms = conn.has_rtt_sample().then(|| conn.get_smooth_rtt_ms());
            let queue_delay_ms = conn.rtt.queue_delay_ms();
            let (probe_rounds_ok, probe_rounds_started_ms) = conn.probes.rounds_ok();
            let signals = HealthSignals {
                connected: conn.connected && !conn.is_timed_out(),
                socket_valid: !conn.needs_rebind(),
                iface_present: !conn.is_removed(),
                route_health: conn.route_health,
                attempts_since_proof: conn.delivery.attempts_since_proof,
                proof_age_ms: conn.delivery.proof_age_ms(now),
                srtt_ms,
                loss_ewma: conn.loss.last_value(),
                loss_cohort_ok: conn.loss.loss_cohort_ok(now, k.loss_stale_after_ms),
                last_cohort_ms: conn.loss.last_cohort_ms(),
                probe_loss: conn.loss.probe_loss(),
                queue_delay_ms,
                slow_min_rtt_ms: conn.rtt.slow_min_rtt_ms(),
                probe_rounds_ok,
                probe_rounds_started_ms,
                held_links,
                observation_interval_ms: super::HOUSEKEEPING_INTERVAL_MS,
                now_ms: now,
            };
            let originals = conn.adaptive.original_recovery.rounds(now);
            if let Some(transition) = conn.health.step_with_originals(&signals, &k, originals)
                && matches!(
                    transition.from,
                    HealthState::Degraded | HealthState::Stalled
                )
                && transition.to == HealthState::Rejoining
            {
                conn.loss.begin_recovered_epoch(now);
            }
            let to = if replaced {
                HealthState::Down
            } else {
                conn.health.state()
            };
            if observation.reported != to
                && observation
                    .last_log_ms
                    .is_none_or(|last| now.saturating_sub(last) >= 1000)
            {
                let reason = match to {
                    HealthState::Down => "socket or registration unavailable",
                    HealthState::Stalled => "DATA proof overdue",
                    HealthState::Degraded if conn.health.route_latched() => "no default route",
                    HealthState::Degraded if conn.health.loss_latched() => "normal DATA loss",
                    HealthState::Degraded => "queue delay",
                    HealthState::Rejoining if observation.reported == HealthState::Down => {
                        "registered"
                    }
                    HealthState::Rejoining => "recovery evidence",
                    HealthState::Healthy => "ramp complete",
                };
                info!(
                    "link {} health {}→{} ({reason})",
                    conn.label,
                    observation.reported.as_str(),
                    to.as_str()
                );
                observation.reported = to;
                observation.last_log_ms = Some(now);
            }
            if observation
                .last_rate_tick_ms
                .is_some_and(|last| now.saturating_sub(last) < 1000)
            {
                continue;
            }
            observation.last_rate_tick_ms = Some(now);
            conn.rate_cap.tick(
                &conn.delivery,
                &RateSignals {
                    now_ms: now,
                    srtt_ms: srtt_ms.unwrap_or(0.0),
                    rtt_min_ms: conn.rtt.slow_min_rtt_ms(),
                    queue_delay_ms,
                    velocity_ms_per_update: conn.get_rtt_velocity(),
                    jitter_ms: conn.get_rtt_jitter_ms(),
                    loss_ewma: conn.loss.last_value(),
                },
            );
            if let Some(sample) = conn.batch_sender.wire_sample() {
                tracing::debug!(link = %conn.label, generation = conn.delivery.socket_generation,
                    now_ms = now, wire_budget_bps = sample.rate_bps, attempted_wire_bytes = sample.accepted_bytes,
                    wire_rate_phase = ?conn.wire_rate.phase(),
                    target_bps = conn.rate_cap.target_bps(), "adaptive wire budget");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::Duration;

    use super::*;
    use crate::sender::uplink::{create_uplink_channel, sync_readers};
    use crate::test_helpers::{
        advance_test_clock, create_test_connection, create_test_connections,
    };

    #[tokio::test]
    async fn dead_reader_is_restarted_for_active_connection() {
        let mut connections = vec![create_test_connection().await];
        let conn_id = connections[0].conn_id;
        let mut reg = SrtlaRegistrationManager::new();
        let mut all_failed_at: Option<Instant> = None;

        let (packet_tx, _packet_rx) = create_uplink_channel();
        let mut reader_handles: HashMap<ConnectionId, ReaderHandle> = HashMap::new();
        sync_readers(&connections, &mut reader_handles, &packet_tx);

        // Abort the reader and let the runtime drive the cancellation to completion,
        // reproducing the latent task-death S5 targets (a handle that is_finished()).
        reader_handles.get(&conn_id).unwrap().handle.abort();
        for _ in 0..1000 {
            if reader_handles.get(&conn_id).unwrap().handle.is_finished() {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(
            reader_handles.get(&conn_id).unwrap().handle.is_finished(),
            "reader task should be dead after abort"
        );

        handle_housekeeping(
            &mut connections,
            &mut reg,
            false,
            &mut all_failed_at,
            &mut reader_handles,
            &packet_tx,
            &mut SequenceTracker::new(),
        )
        .await
        .expect("housekeeping on an active connection must not fail");

        // A finished handle can never un-finish itself; a live handle proves
        // housekeeping spawned a fresh reader in its place.
        assert!(
            !reader_handles.get(&conn_id).unwrap().handle.is_finished(),
            "housekeeping must respawn the dead reader for the still-active connection"
        );
    }

    /// Regression for S9: the all-uplinks-failed timeout must measure time *since failure*
    /// (`failed_at.elapsed()`), not the uptime captured at failure. The pre-fix call site
    /// compared `STARTUP.elapsed() - failed_at.elapsed()` (= `failed_at - STARTUP`), so it
    /// tripped on the first all-down pass once uptime alone exceeded `GLOBAL_TIMEOUT_MS`,
    /// erroring on a transient blip — making the `armed.is_ok()` assertion below fail.
    #[tokio::test(start_paused = true)]
    async fn all_failed_timeout_measures_elapsed_since_failure() {
        // Force STARTUP_INSTANT to init at the paused-clock base before simulating uptime,
        // so the pre-fix misfire reproduces (the fixed path never reads it).
        let _ = crate::utils::elapsed_ms();

        let mut connections = create_test_connections(2).await;
        let mut reg = SrtlaRegistrationManager::new();
        // Models a stream that was established and then lost every link.
        reg.has_connected = true;
        let mut reader_handles: HashMap<ConnectionId, ReaderHandle> = HashMap::new();
        let (packet_tx, _packet_rx) = tokio::sync::mpsc::unbounded_channel::<UplinkPacket>();
        let mut all_failed_at: Option<Instant> = None;

        advance_test_clock(Duration::from_millis(GLOBAL_TIMEOUT_MS + 1000)).await;

        // Drop all uplinks; pin the reconnect backoff so housekeeping reaches the timeout
        // branch instead of attempting socket reconnection.
        for conn in connections.iter_mut() {
            conn.mark_for_recovery();
            conn.reconnection.last_reconnect_attempt_ms = now_ms();
        }

        let armed = handle_housekeeping(
            &mut connections,
            &mut reg,
            false,
            &mut all_failed_at,
            &mut reader_handles,
            &packet_tx,
            &mut SequenceTracker::new(),
        )
        .await;
        assert!(
            armed.is_ok(),
            "arming the all-failed timer must not error on a transient blip (uptime already \
             exceeds {GLOBAL_TIMEOUT_MS}ms)"
        );
        assert!(all_failed_at.is_some(), "the failure timer should be armed");

        advance_test_clock(Duration::from_millis(GLOBAL_TIMEOUT_MS - 1000)).await;
        let within = handle_housekeeping(
            &mut connections,
            &mut reg,
            false,
            &mut all_failed_at,
            &mut reader_handles,
            &packet_tx,
            &mut SequenceTracker::new(),
        )
        .await;
        assert!(
            within.is_ok(),
            "no error until a full {GLOBAL_TIMEOUT_MS}ms has elapsed since the links failed"
        );

        advance_test_clock(Duration::from_millis(2000)).await;
        let fired = handle_housekeeping(
            &mut connections,
            &mut reg,
            false,
            &mut all_failed_at,
            &mut reader_handles,
            &packet_tx,
            &mut SequenceTracker::new(),
        )
        .await;
        assert!(
            fired.is_err(),
            "the all-failed timeout must fire once a full {GLOBAL_TIMEOUT_MS}ms has elapsed since \
             failure"
        );
    }
}
