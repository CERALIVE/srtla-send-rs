use tracing::{info, warn};

use crate::config::DynamicConfig;
use crate::connection::SrtlaConnection;
use crate::protocol::PKT_LOG_SIZE;
use crate::utils::now_ms;

#[cfg(test)]
#[path = "premature_nak_status_tests.rs"]
mod premature_nak_tests;

/// Comprehensive status monitoring for connections
///
/// Optimized to reduce CPU overhead:
/// - Early exit if INFO logging is disabled
/// - Single pass over connections to collect stats
/// - Avoided redundant iterations
#[cfg_attr(not(unix), allow(dead_code))]
pub(crate) fn log_connection_status(
    connections: &[SrtlaConnection],
    last_selected_idx: Option<usize>,
    config: &DynamicConfig,
) {
    // Early exit if INFO logging is not enabled - avoid all computation
    if !tracing::enabled!(tracing::Level::INFO) {
        return;
    }

    let total_connections = connections.len();

    // Single pass over connections to collect all stats
    let mut active_connections = 0usize;
    let mut total_bitrate_mbps = 0.0f64;
    let mut total_in_flight = 0usize;

    for conn in connections.iter() {
        if !conn.is_timed_out() {
            active_connections += 1;
        }
        total_bitrate_mbps += conn.current_bitrate_mbps();
        total_in_flight += conn.in_flight_packets as usize;
    }

    let timed_out_connections = total_connections - active_connections;

    // Packet log utilization
    let max_possible_entries = total_connections * PKT_LOG_SIZE;
    let log_utilization = if max_possible_entries > 0 {
        (total_in_flight as f64 / max_possible_entries as f64) * 100.0
    } else {
        0.0
    };

    // Bond-level retransmit counter
    let total_rexmit_forwarded: u64 = connections.iter().map(|c| c.rexmit_forwarded).sum();

    // Get current config snapshot
    let snap = config.snapshot();

    info!("Connection Status Report:");
    info!("  Total connections: {}", total_connections);
    info!("  Total bitrate: {:.2} Mbps", total_bitrate_mbps);
    info!("  Retransmits forwarded: {}", total_rexmit_forwarded);
    info!(
        "  Active connections: {} ({:.1}%)",
        active_connections,
        if total_connections > 0 {
            (active_connections as f64 / total_connections as f64) * 100.0
        } else {
            0.0
        }
    );
    info!("  Timed out connections: {}", timed_out_connections);

    // Show mode and relevant settings
    info!("  Mode: {}", snap.mode);
    match snap.mode {
        crate::mode::SchedulingMode::Enhanced => {
            info!(
                "    Quality: {}, Exploration: {}",
                if snap.quality_enabled { "ON" } else { "OFF" },
                if snap.exploration_enabled {
                    "ON"
                } else {
                    "OFF"
                }
            );
        }
    }

    // Show packet log utilization
    info!(
        "  Packet log: {} entries used ({:.1}% of capacity)",
        total_in_flight, log_utilization
    );

    // Show last selected connection
    if let Some(idx) = last_selected_idx {
        if idx < connections.len() {
            info!("  Last selected: {}", connections[idx].label);
        } else {
            warn!("  Last selected index {} is out of bounds!", idx);
        }
    } else {
        info!("  Last selected: none");
    }

    // Show individual connection details
    for (i, conn) in connections.iter().enumerate() {
        let status = if conn.is_timed_out() {
            "TIMED_OUT"
        } else {
            "ACTIVE"
        };
        let score = conn.get_score();

        // Avoid String allocation for common score cases
        let score_desc: std::borrow::Cow<'static, str> = match score {
            -1 => "DISCONNECTED".into(),
            0 => "AT_CAPACITY".into(),
            s => s.to_string().into(),
        };

        // Use elapsed seconds directly
        let last_recv = conn
            .last_received
            .map(|t| format!("{:.1}s ago", t.elapsed().as_secs_f64()))
            .unwrap_or_else(|| "never".into());

        let last_send = conn
            .last_sent
            .map(|t| format!("{:.1}s ago", t.elapsed().as_secs_f64()))
            .unwrap_or_else(|| "never".into());

        info!(
            "    [{}] {} {} - Score: {} - Last recv/send: {}/{} - Window: {} - In-flight: {} - \
             Bitrate: {:.2} Mbps",
            i,
            status,
            conn.label,
            score_desc,
            last_recv,
            last_send,
            conn.window,
            conn.in_flight_packets,
            conn.current_bitrate_mbps()
        );

        {
            info!(
                premature_naks = conn.premature_nak_count,
                rexmit_fwd = conn.rexmit_forwarded,
                "        health={} pref={:.2} cap={:.0}",
                conn.health.state().as_str(),
                conn.effective_priority()
                    .map_or(0.0, |priority| priority.get()),
                conn.rate_cap.target_bps()
            );
        }

        // Egress health is reported SEPARATELY from the ACTIVE/TIMED_OUT line
        // above, because they answer different questions: that line is ACK
        // liveness, this one is whether the interface can still carry traffic
        // at all. A link can be ACK-live and route-blackholed at the same time.
        if let Some(iface) = conn.egress.iface() {
            info!(
                "        Egress: iface={} link={} route={}",
                iface.as_str(),
                if conn.is_removed() {
                    "REMOVED"
                } else {
                    "BOUND"
                },
                conn.route_health.as_str()
            );
        }

        let foreign = conn.foreign_source_datagrams();
        if conn.probes.probes_sent > 0 {
            info!(
                conn_id = conn.conn_id,
                probes_sent = conn.probes.probes_sent,
                "Duplicate DATA probe status"
            );
        }
        if foreign > 0 {
            info!(
                "        Foreign-source datagrams: {} (processed, not dropped)",
                foreign
            );
        }

        if conn.rtt.estimated_rtt_ms > 0.0 {
            info!(
                "        RTT: kalman={:.1}ms, velocity={:.2}ms/sample, jitter={:.1}ms, stable={} \
                 (last: {:.1}s ago)",
                conn.get_smooth_rtt_ms(),
                conn.get_rtt_velocity(),
                conn.get_rtt_jitter_ms(),
                conn.is_rtt_stable(),
                (now_ms().saturating_sub(conn.rtt.last_rtt_measurement_ms) as f64) / 1000.0
            );
        }
    }

    // Show any warnings
    if active_connections == 0 {
        warn!("No active connections available!");
    } else if active_connections < total_connections / 2 {
        warn!("Less than half of connections are active");
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Write};
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::fmt::MakeWriter;

    use super::*;
    use crate::test_helpers::create_test_connection;

    #[derive(Clone, Default)]
    struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

    impl Write for CapturedLogs {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for CapturedLogs {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    pub(super) fn capture_logs(run: impl FnOnce()) -> String {
        let logs = CapturedLogs::default();
        let output = logs.0.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .with_writer(logs)
            .finish();
        tracing::subscriber::with_default(subscriber, run);
        String::from_utf8(output.lock().unwrap().clone()).unwrap()
    }

    #[tokio::test]
    async fn status_renders_rtt_velocity_in_milliseconds_per_sample() {
        let mut conn = create_test_connection().await;
        conn.rtt.estimated_rtt_ms = 42.0;

        let rendered = capture_logs(|| {
            log_connection_status(&[conn], None, &DynamicConfig::new());
        });

        assert!(rendered.contains("velocity=0.00ms/sample"));
        assert!(!rendered.contains("velocity=0.00ms/s,"));
    }

    #[tokio::test]
    async fn adaptive_status_reports_health_preference_and_target_cap() {
        let mut conn = create_test_connection().await;
        conn.priority_baseline = Some(crate::bind_map::Priority::try_from(0.1).unwrap());
        let config = DynamicConfig::new();
        config.set_mode(crate::mode::SchedulingMode::Enhanced);
        let rendered = capture_logs(|| {
            log_connection_status(&[conn], None, &config);
        });
        assert!(rendered.contains("health=down pref=0.10 cap=1000000"));
    }

    #[tokio::test]
    async fn health_transitions_log_first_event_and_rate_limit_followups() {
        let mut conn = create_test_connection().await;
        let mut ticker = crate::sender::housekeeping::HealthTicker::default();
        let rendered = capture_logs(|| {
            ticker.tick(std::slice::from_mut(&mut conn), &[], 1000);
            conn.route_health = crate::connection::RouteHealth::NoDefaultRoute;
            ticker.tick(std::slice::from_mut(&mut conn), &[], 1001);
            ticker.tick(std::slice::from_mut(&mut conn), &[], 2000);
        });
        assert_eq!(rendered.matches("link test-connection health").count(), 2);
        assert!(rendered.contains("health down→rejoining (registered)"));
        assert!(rendered.contains("health rejoining→degraded (no default route)"));
    }
}
