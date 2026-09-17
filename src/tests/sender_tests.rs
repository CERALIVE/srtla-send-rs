// allow: SIZE_OK — lane 23 is restricted to this existing sender regression suite; keep frozen tests in place.
#[cfg(test)]
mod tests {

    use std::io::Write;
    use std::net::{IpAddr, Ipv4Addr};

    use tempfile::NamedTempFile;
    use tokio::time::Duration;

    use crate::config::{ConfigSnapshot, DynamicConfig};
    use crate::connection::SrtlaConnection;
    use crate::mode::SchedulingMode;
    use crate::protocol::{
        CONN_TIMEOUT, SRT_TYPE_ACK, SRT_TYPE_DATA, SRT_TYPE_NAK, get_packet_type, is_srt_ack,
    };
    use crate::registration::SrtlaRegistrationManager;
    use crate::sender::*;
    use crate::test_helpers::{advance_test_clock, create_test_connections};
    use crate::utils::now_ms;

    #[test]
    fn test_select_connection_idx_classic() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        // Test classic mode - should pick connection with highest score
        connections[1].in_flight_packets = 0; // Best score
        connections[0].in_flight_packets = 5; // Lower score
        connections[2].in_flight_packets = 10; // Lowest score

        let config = ConfigSnapshot {
            mode: SchedulingMode::Classic,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        let selected = select_connection_idx(
            &mut connections,
            None,
            0,
            0,
            &config,
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn test_select_connection_idx_quality_scoring() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));
        let current_time = now_ms();

        // Connection 0: Recent NAKs - should get low score
        connections[0].congestion.nak_count = 5;
        connections[0].congestion.last_nak_time_ms = current_time - 1000; // 1 second ago

        // Connection 1: No NAKs - should get bonus
        connections[1].congestion.nak_count = 0;

        // Connection 2: Old NAKs - should get partial penalty
        connections[2].congestion.nak_count = 3;
        connections[2].congestion.last_nak_time_ms = current_time - 8000; // 8 seconds ago

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: true,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        let selected = select_connection_idx(
            &mut connections,
            None,
            0,
            current_time,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        // Should prefer connection 1 (no NAKs)
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn test_select_connection_idx_burst_nak_penalty() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));
        let current_time = now_ms();

        // Connection 0: NAK burst
        connections[0].congestion.nak_count = 5;
        connections[0].congestion.nak_burst_count = 3;
        connections[0].congestion.last_nak_time_ms = current_time - 2000; // 2 seconds ago

        // Connection 1: Same NAK count but no burst
        connections[1].congestion.nak_count = 5;
        connections[1].congestion.nak_burst_count = 0;
        connections[1].congestion.last_nak_time_ms = current_time - 2000; // 2 seconds ago

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: true,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        let selected = select_connection_idx(
            &mut connections,
            None,
            0,
            current_time,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        // Should prefer connection 2 (never had NAKs, best quality)
        assert_eq!(selected, Some(2));
    }

    #[test]
    fn test_time_based_switch_dampening_allows_after_cooldown() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        // Setup: Connection 0 is currently selected, Connection 1 has significantly better score
        connections[0].in_flight_packets = 5; // Lower score
        connections[1].in_flight_packets = 0; // Best score (significantly better, exceeds 2% hysteresis)
        connections[2].in_flight_packets = 10; // Worst score

        let last_switch_time_ms = now_ms();
        let current_time_ms = last_switch_time_ms + 20; // 20ms after last switch (past 15ms cooldown)

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: true,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        // After cooldown: per-packet selection can now choose the better connection
        // From this point forward, all subsequent packets will route via connection 1
        let selected = select_connection_idx(
            &mut connections,
            Some(0),
            last_switch_time_ms,
            current_time_ms,
            &config,
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            selected,
            Some(1),
            "Should switch per-packet routing to better connection after cooldown expires"
        );
    }

    #[test]
    fn test_time_based_switch_dampening_allows_if_current_invalid() {
        use tokio::time::{Duration, Instant};

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        // Setup: Connection 0 is currently selected but becomes timed out
        connections[0].in_flight_packets = 5;
        // Simulate timeout by setting last_received past CONN_TIMEOUT
        connections[0].last_received = Some(Instant::now() - Duration::from_secs(CONN_TIMEOUT + 1));
        connections[1].in_flight_packets = 0; // Best score
        connections[2].in_flight_packets = 10;

        let last_switch_time_ms = now_ms();
        let current_time_ms = last_switch_time_ms + 5; // Within 15ms cooldown

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: true,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        // Cooldown is bypassed when current connection is invalid/timed out
        // Per-packet selection immediately switches to valid connection
        let selected = select_connection_idx(
            &mut connections,
            Some(0),
            last_switch_time_ms,
            current_time_ms,
            &config,
            &mut EdpfSchedulerState::default(),
        );
        assert_eq!(
            selected,
            Some(1),
            "Should immediately route packets via valid connection if current is timed out, \
             bypassing cooldown"
        );
    }

    #[test]
    fn test_classic_mode_ignores_time_dampening() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        // Setup: Connection 0 is currently selected, Connection 1 has better score
        connections[0].in_flight_packets = 5; // Lower score
        connections[1].in_flight_packets = 0; // Best score
        connections[2].in_flight_packets = 10; // Worst score

        let last_switch_time_ms = now_ms();
        let current_time_ms = last_switch_time_ms + 200; // 200ms after last switch (within cooldown)

        let config = ConfigSnapshot {
            mode: SchedulingMode::Classic,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        // Classic mode: per-packet selection ALWAYS picks highest score connection
        // No dampening, no hysteresis - matches original C implementation
        let selected = select_connection_idx(
            &mut connections,
            Some(0),
            last_switch_time_ms,
            current_time_ms,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        // Per-packet routing immediately uses connection 1 (best score)
        assert_eq!(
            selected,
            Some(1),
            "Classic mode per-packet selection should ignore time-based dampening and always \
             route via highest score connection"
        );
    }

    #[test]
    fn test_nak_attribution_to_correct_connection() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        let current_time = now_ms();
        connections[0].register_packet(100, current_time);
        connections[1].register_packet(200, current_time);
        connections[2].register_packet(300, current_time);

        let initial_counts = [
            connections[0].congestion.nak_count,
            connections[1].congestion.nak_count,
            connections[2].congestion.nak_count,
        ];

        let found_0 = connections[0].handle_nak(100);
        assert!(found_0);
        assert_eq!(connections[0].congestion.nak_count, initial_counts[0] + 1);
        assert_eq!(connections[1].congestion.nak_count, initial_counts[1]);
        assert_eq!(connections[2].congestion.nak_count, initial_counts[2]);

        let found_1 = connections[1].handle_nak(200);
        assert!(found_1);
        assert_eq!(connections[0].congestion.nak_count, initial_counts[0] + 1);
        assert_eq!(connections[1].congestion.nak_count, initial_counts[1] + 1);
        assert_eq!(connections[2].congestion.nak_count, initial_counts[2]);

        let not_found_0 = connections[0].handle_nak(999);
        let not_found_1 = connections[1].handle_nak(999);
        let not_found_2 = connections[2].handle_nak(999);
        assert!(!not_found_0);
        assert!(!not_found_1);
        assert!(!not_found_2);
        assert_eq!(connections[0].congestion.nak_count, initial_counts[0] + 1);
        assert_eq!(connections[1].congestion.nak_count, initial_counts[1] + 1);
        assert_eq!(connections[2].congestion.nak_count, initial_counts[2]);
    }

    #[tokio::test]
    async fn test_read_ip_list() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "192.168.1.1").unwrap();
        writeln!(temp_file, "192.168.1.2").unwrap();
        writeln!(temp_file).unwrap(); // Empty line
        writeln!(temp_file, "192.168.1.3").unwrap();
        writeln!(temp_file, "invalid-ip").unwrap(); // Invalid IP

        let ips = read_ip_list(temp_file.path().to_str().unwrap())
            .await
            .unwrap();

        assert_eq!(ips.len(), 3);
        assert_eq!(ips[0], IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(ips[1], IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)));
        assert_eq!(ips[2], IpAddr::V4(Ipv4Addr::new(192, 168, 1, 3)));
    }

    #[tokio::test]
    async fn test_read_ip_list_empty() {
        let temp_file = NamedTempFile::new().unwrap();

        let ips = read_ip_list(temp_file.path().to_str().unwrap())
            .await
            .unwrap();
        assert!(ips.is_empty());
    }

    #[tokio::test]
    async fn test_read_ip_list_nonexistent() {
        let result = read_ip_list("/nonexistent/file.txt").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_connection_changes_remove_stale() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));
        let initial_count = connections.len();

        // New IPs that don't include all current connections
        let new_ips = vec![
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), // Keep first connection
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50)), // New IP
        ];

        let mut last_selected_idx = Some(1);
        let mut seq_tracker = SequenceTracker::new();
        let now = now_ms();
        // Insert entries for connections that will be removed
        seq_tracker.insert(100, connections[1].conn_id, now);
        seq_tracker.insert(200, connections[2].conn_id, now);

        rt.block_on(apply_connection_changes(
            &mut connections,
            &new_ips,
            "127.0.0.1",
            8080,
            &mut last_selected_idx,
            &mut seq_tracker,
            &mut SrtlaRegistrationManager::new(),
        ));

        // Should have removed some connections
        assert!(connections.len() < initial_count);

        // Should have reset selection
        assert_eq!(last_selected_idx, None);

        // Entries for removed connections should now return None
        // (they were cleaned up by remove_connection)
        assert!(seq_tracker.get(100, now).is_none());
        assert!(seq_tracker.get(200, now).is_none());
    }

    /// Relabel test connections as real `host:port via ip` uplinks so they match
    /// the labels `apply_connection_changes` derives, exercising the survivor path.
    fn relabel_as_uplinks(
        connections: &mut [SrtlaConnection],
        host: &str,
        port: u16,
        ips: &[IpAddr],
    ) {
        for (conn, ip) in connections.iter_mut().zip(ips.iter()) {
            conn.local_ip = *ip;
            conn.label = format!("{host}:{port} via {ip}");
        }
    }

    #[test]
    fn test_reload_reorders_conn_ids_to_file_order_keeping_survivors() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(2));
        let host = "127.0.0.1";
        let port = 9000u16;
        let ip_a = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
        let ip_b = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3));
        relabel_as_uplinks(&mut connections, host, port, &[ip_a, ip_b]);
        let cid_a = connections[0].conn_id;
        let cid_b = connections[1].conn_id;

        let mut last_selected_idx = Some(0);
        let mut seq_tracker = SequenceTracker::new();

        // Reload reverses the file order: [B, A]. Both survive (no new binds).
        rt.block_on(apply_connection_changes(
            &mut connections,
            &[ip_b, ip_a],
            host,
            port,
            &mut last_selected_idx,
            &mut seq_tracker,
            &mut SrtlaRegistrationManager::new(),
        ));

        assert_eq!(
            connections.len(),
            2,
            "both uplinks must survive the reorder"
        );
        assert_eq!(
            connections[0].conn_id, cid_b,
            "telemetry conn_id 0 must follow the new file order (B first)"
        );
        assert_eq!(connections[1].conn_id, cid_a, "conn_id 1 must be A");
        assert_eq!(
            last_selected_idx, None,
            "a reorder must invalidate the cached selection index"
        );
    }

    #[test]
    fn test_reload_removes_one_keeps_survivor_and_cleans_seq_tracker() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(2));
        let host = "127.0.0.1";
        let port = 9000u16;
        let ip_a = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
        let ip_b = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3));
        relabel_as_uplinks(&mut connections, host, port, &[ip_a, ip_b]);
        let cid_a = connections[0].conn_id;
        let cid_b = connections[1].conn_id;

        let mut last_selected_idx = Some(1);
        let mut seq_tracker = SequenceTracker::new();
        let now = now_ms();
        seq_tracker.insert(100, cid_b, now);

        // Reload drops B, keeps A.
        rt.block_on(apply_connection_changes(
            &mut connections,
            &[ip_a],
            host,
            port,
            &mut last_selected_idx,
            &mut seq_tracker,
            &mut SrtlaRegistrationManager::new(),
        ));

        assert_eq!(connections.len(), 1, "only the surviving uplink remains");
        assert_eq!(
            connections[0].conn_id, cid_a,
            "survivor A keeps its identity (not torn down and rebuilt)"
        );
        assert!(
            seq_tracker.get(100, now).is_none(),
            "the removed uplink's sequence entry must be cleared"
        );
        assert_eq!(last_selected_idx, None);
    }

    #[test]
    fn test_reload_identical_list_keeps_selection_no_disconnect() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(2));
        let host = "127.0.0.1";
        let port = 9000u16;
        let ip_a = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
        let ip_b = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3));
        relabel_as_uplinks(&mut connections, host, port, &[ip_a, ip_b]);
        let cid_a = connections[0].conn_id;
        let cid_b = connections[1].conn_id;

        let mut last_selected_idx = Some(1);
        let mut seq_tracker = SequenceTracker::new();

        // A SIGHUP whose file is unchanged must not disturb any link.
        rt.block_on(apply_connection_changes(
            &mut connections,
            &[ip_a, ip_b],
            host,
            port,
            &mut last_selected_idx,
            &mut seq_tracker,
            &mut SrtlaRegistrationManager::new(),
        ));

        assert_eq!(connections.len(), 2);
        assert_eq!(connections[0].conn_id, cid_a);
        assert_eq!(connections[1].conn_id, cid_b);
        assert_eq!(
            last_selected_idx,
            Some(1),
            "an unchanged reload must keep the selection index (zero disconnects)"
        );
    }

    #[test]
    fn test_pending_connection_changes() {
        let changes = PendingConnectionChanges {
            new_links: Some(crate::sender::specs_from_ips(&[IpAddr::V4(Ipv4Addr::new(
                192, 168, 1, 100,
            ))])),
            receiver_host: "test-host".to_string(),
            receiver_port: 9090,
        };

        assert!(changes.new_links.is_some());
        assert_eq!(changes.receiver_host, "test-host");
        assert_eq!(changes.receiver_port, 9090);
    }

    #[test]
    fn test_constants() {
        assert!(SEQ_TRACKING_SIZE > 0);
        assert!(GLOBAL_TIMEOUT_MS > 0);

        // Should handle decent throughput (16384 entries)
        assert!(SEQ_TRACKING_SIZE >= 1000);
        // Should allow time for connections
        assert!(GLOBAL_TIMEOUT_MS >= 5000);
    }

    #[tokio::test]
    async fn test_create_connections_from_ips() {
        let ips = vec![
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        ];

        // This will likely fail to connect but should not panic
        let connections = create_connections_from_ips(&ips, "127.0.0.1", 9999).await;

        // Connections may be empty due to connection failures, which is OK for testing
        assert!(connections.len() <= ips.len());
    }

    #[test]
    fn test_sequence_tracking_limits() {
        let mut seq_tracker = SequenceTracker::new();
        let now = now_ms();

        // Fill beyond capacity - ring buffer naturally handles this
        for i in 0..(SEQ_TRACKING_SIZE + 100) {
            seq_tracker.insert(i as u32, 1, now);
        }

        // Ring buffer should have overwritten older entries
        // Recent entries should still be accessible
        let recent_seq = (SEQ_TRACKING_SIZE + 50) as u32;
        assert!(seq_tracker.get(recent_seq, now).is_some());

        // Old entries that were overwritten should not be accessible
        // (due to collision with newer sequence numbers)
        let old_seq = 50u32;
        // The old entry was overwritten when seq (SEQ_TRACKING_SIZE + 50) was inserted
        // because they map to the same index
        assert!(seq_tracker.get(old_seq, now).is_none());
    }

    #[test]
    fn test_connection_selection_with_all_disconnected() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        // Disconnect all connections
        for conn in &mut connections {
            conn.connected = false;
        }

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        let selected = select_connection_idx(
            &mut connections,
            None,
            0,
            0,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        // Should return None when all connections have score -1
        assert_eq!(selected, None);
    }

    #[test]
    fn test_exploration_mode() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: false,
            exploration_enabled: true,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        // Test exploration - this is time-dependent so we just test that it doesn't panic
        let _selected = select_connection_idx(
            &mut connections,
            None,
            0,
            0,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        // The result depends on timing, but should not panic
    }

    #[test]
    fn test_config_integration() {
        let config = DynamicConfig::new();
        let snap = config.snapshot();

        // Default values from DynamicConfig::new()
        assert_eq!(snap.mode, SchedulingMode::Enhanced);
        assert!(snap.quality_enabled);
        assert!(!snap.exploration_enabled);
        assert_eq!(snap.rtt_delta_ms, 30);
    }

    #[test]
    fn test_calculate_quality_multiplier() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conn = rt
            .block_on(create_test_connections(1))
            .into_iter()
            .next()
            .unwrap();

        let current_time = now_ms();
        conn.reconnection.connection_established_ms = current_time - 35000;

        assert_eq!(calculate_quality_multiplier(&conn, current_time), 1.1);

        // Test connection with recent NAK - exponential decay formula
        // With exponential decay: penalty = 0.5 * e^(-age_ms / 2000), multiplier = 1.0 - penalty
        conn.congestion.nak_count = 1;

        // 500ms ago: multiplier ≈ 0.61 (strong penalty, recent NAK)
        conn.congestion.last_nak_time_ms = current_time - 500;
        let mult_500 = calculate_quality_multiplier(&conn, current_time);
        assert!(
            (mult_500 - 0.61).abs() < 0.02,
            "Expected ~0.61, got {}",
            mult_500
        );

        // 2000ms ago (half-life): multiplier ≈ 0.816 (moderate penalty)
        conn.congestion.last_nak_time_ms = current_time - 2000;
        let mult_2000 = calculate_quality_multiplier(&conn, current_time);
        assert!(
            (mult_2000 - 0.816).abs() < 0.02,
            "Expected ~0.816, got {}",
            mult_2000
        );

        // 5000ms ago: multiplier ≈ 0.96 (light penalty)
        conn.congestion.last_nak_time_ms = current_time - 5000;
        let mult_5000 = calculate_quality_multiplier(&conn, current_time);
        assert!(
            (mult_5000 - 0.96).abs() < 0.02,
            "Expected ~0.96, got {}",
            mult_5000
        );

        // 15000ms ago: multiplier ≈ 1.0 (essentially recovered)
        conn.congestion.last_nak_time_ms = current_time - 15000;
        let mult_15000 = calculate_quality_multiplier(&conn, current_time);
        assert!(
            (mult_15000 - 1.0).abs() < 0.02,
            "Expected ~1.0, got {}",
            mult_15000
        );

        // Test connection with no NAKs ever - bonus
        // Need to clear the last_nak_time_ms to simulate truly no NAKs
        conn.congestion.nak_count = 0;
        conn.congestion.last_nak_time_ms = 0; // Clear NAK history
        assert_eq!(calculate_quality_multiplier(&conn, current_time), 1.1);

        // Test burst NAK penalty (requires ≥5 NAKs in burst, within 3s)
        // Burst penalty is 0.7x additional multiplier
        conn.congestion.nak_count = 5;
        conn.congestion.last_nak_time_ms = current_time - 2000;
        conn.congestion.nak_burst_count = 5;
        let mult_burst = calculate_quality_multiplier(&conn, current_time);
        // At 2000ms: base multiplier ≈ 0.816, with burst: 0.816 * 0.7 ≈ 0.571
        assert!(
            (mult_burst - 0.571).abs() < 0.02,
            "Expected ~0.571, got {}",
            mult_burst
        );
    }

    /// SIGHUP IP-list reload contract (Unix-only — `signal(SignalKind::hangup())`
    /// has no Windows arm). These pin the parity-critical telemetry behavior:
    /// surviving uplinks keep their socket + registration across a reload (no
    /// re-handshake, zero disconnect), and the pool is rebuilt in ips-file order
    /// so `conn_id` tracks the file (a reorder reorders `conn_id`). The whole
    /// module is gated so the aarch64 cross and Windows CI jobs still compile.
    #[cfg(unix)]
    mod sighup_tests {
        use std::sync::Arc;

        use super::*;

        /// A survivor's `socket: Arc<BatchUdpSocket>` is moved intact across a
        /// reload, so its Arc backing pointer is stable. A re-handshake would run
        /// `connect_from_ip` (a fresh bind + new `conn_id`), changing this pointer.
        fn socket_ptr(conn: &SrtlaConnection) -> *const () {
            Arc::as_ptr(&conn.socket) as *const ()
        }

        /// Reordering the ips file ([A, B] → [B, A]) reorders the pool: B takes
        /// telemetry index 0, A takes index 1. Both uplinks survive — each keeps
        /// its original socket and `conn_id` (no tear-down/rebuild) — they are
        /// only repositioned to match the new file order.
        #[test]
        fn sighup_reorder_reassigns_conn_id() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut connections = rt.block_on(create_test_connections(2));
            let host = "127.0.0.1";
            let port = 9000u16;
            let ip_a = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
            let ip_b = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3));
            relabel_as_uplinks(&mut connections, host, port, &[ip_a, ip_b]);

            let cid_a = connections[0].conn_id;
            let cid_b = connections[1].conn_id;
            let sock_a = socket_ptr(&connections[0]);
            let sock_b = socket_ptr(&connections[1]);

            let mut last_selected_idx = Some(0);
            let mut seq_tracker = SequenceTracker::new();

            rt.block_on(apply_connection_changes(
                &mut connections,
                &[ip_b, ip_a],
                host,
                port,
                &mut last_selected_idx,
                &mut seq_tracker,
                &mut SrtlaRegistrationManager::new(),
            ));

            assert_eq!(
                connections.len(),
                2,
                "both uplinks must survive the reorder"
            );

            assert_eq!(
                connections[0].conn_id, cid_b,
                "conn_id 0 must track the reordered file (B first)"
            );
            assert_eq!(
                connections[1].conn_id, cid_a,
                "conn_id 1 must be A after the reorder"
            );

            assert_eq!(
                socket_ptr(&connections[0]),
                sock_b,
                "index 0's socket must be B's original socket, not a fresh bind"
            );
            assert_eq!(
                socket_ptr(&connections[1]),
                sock_a,
                "index 1's socket must be A's original socket, not a fresh bind"
            );

            assert_eq!(
                last_selected_idx, None,
                "a reorder must invalidate the cached selection index"
            );
        }

        /// The registration manager's bookkeeping is keyed by the *position* of
        /// an uplink in the pool, so a reorder makes every stale entry point at
        /// the wrong link. A REG3 landing on the new occupant of a recycled
        /// index must not be authorized by the previous occupant's grant.
        #[test]
        fn sighup_reorder_clears_stale_registration_index_state() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut connections = rt.block_on(create_test_connections(2));
            let host = "127.0.0.1";
            let port = 9000u16;
            let ip_a = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
            let ip_b = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3));
            relabel_as_uplinks(&mut connections, host, port, &[ip_a, ip_b]);

            let mut reg = SrtlaRegistrationManager::new();
            reg.arm_reg3_gate(0);
            reg.arm_reg3_gate(1);
            reg.set_pending_reg2_idx(Some(1));
            reg.set_reg1_target_idx(Some(1));

            let mut last_selected_idx = Some(0);
            let mut seq_tracker = SequenceTracker::new();

            rt.block_on(apply_connection_changes(
                &mut connections,
                &[ip_b, ip_a],
                host,
                port,
                &mut last_selected_idx,
                &mut seq_tracker,
                &mut reg,
            ));

            assert!(
                !reg.is_awaiting_reg3(0) && !reg.is_awaiting_reg3(1),
                "a reorder must drop every stale REG3 grant"
            );
            assert_eq!(reg.pending_reg2_idx(), None);
            assert_eq!(reg.reg1_target_idx(), None);

            let reg3 = [
                (crate::protocol::SRTLA_TYPE_REG3 >> 8) as u8,
                (crate::protocol::SRTLA_TYPE_REG3 & 0xff) as u8,
            ];
            reg.process_registration_packet(0, &reg3);
            assert!(
                !reg.has_connected,
                "a REG3 on a recycled index must not be authorized by the previous occupant"
            );
            assert_eq!(reg.out_of_phase_reg3(), 1);
        }

        /// A SIGHUP whose file is byte-for-byte unchanged ([A, B] → [A, B]) must
        /// keep that bookkeeping: nothing moved, so nothing is invalidated.
        #[test]
        fn sighup_unchanged_list_keeps_registration_index_state() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut connections = rt.block_on(create_test_connections(2));
            let host = "127.0.0.1";
            let port = 9000u16;
            let ip_a = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
            let ip_b = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3));
            relabel_as_uplinks(&mut connections, host, port, &[ip_a, ip_b]);

            let mut reg = SrtlaRegistrationManager::new();
            reg.arm_reg3_gate(1);
            reg.set_pending_reg2_idx(Some(1));

            let mut last_selected_idx = Some(1);
            let mut seq_tracker = SequenceTracker::new();

            rt.block_on(apply_connection_changes(
                &mut connections,
                &[ip_a, ip_b],
                host,
                port,
                &mut last_selected_idx,
                &mut seq_tracker,
                &mut reg,
            ));

            assert!(
                reg.is_awaiting_reg3(1),
                "an unchanged reload must not disturb an in-flight handshake"
            );
            assert_eq!(reg.pending_reg2_idx(), Some(1));
        }

        /// A SIGHUP whose file is byte-for-byte unchanged ([A, B] → [A, B]) must
        /// not re-handshake any link: each survivor keeps its `conn_id` and the
        /// same socket Arc (no `connect_from_ip`), and the cached selection index
        /// is preserved (zero disconnects).
        #[test]
        fn sighup_surviving_uplink_no_rehandshake() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut connections = rt.block_on(create_test_connections(2));
            let host = "127.0.0.1";
            let port = 9000u16;
            let ip_a = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
            let ip_b = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 3));
            relabel_as_uplinks(&mut connections, host, port, &[ip_a, ip_b]);

            let cid_a = connections[0].conn_id;
            let cid_b = connections[1].conn_id;
            let sock_a = socket_ptr(&connections[0]);
            let sock_b = socket_ptr(&connections[1]);

            let mut last_selected_idx = Some(1);
            let mut seq_tracker = SequenceTracker::new();

            rt.block_on(apply_connection_changes(
                &mut connections,
                &[ip_a, ip_b],
                host,
                port,
                &mut last_selected_idx,
                &mut seq_tracker,
                &mut SrtlaRegistrationManager::new(),
            ));

            assert_eq!(connections.len(), 2);

            assert_eq!(connections[0].conn_id, cid_a, "A keeps its conn_id");
            assert_eq!(connections[1].conn_id, cid_b, "B keeps its conn_id");
            assert_eq!(
                socket_ptr(&connections[0]),
                sock_a,
                "A keeps its socket (no re-handshake)"
            );
            assert_eq!(
                socket_ptr(&connections[1]),
                sock_b,
                "B keeps its socket (no re-handshake)"
            );

            assert_eq!(
                last_selected_idx,
                Some(1),
                "an unchanged reload must not disturb the selection (zero disconnects)"
            );
        }
    }

    /// Build a minimal 16-byte SRT control packet carrying `srt_type` in the
    /// first two bytes (big-endian), mirroring `make_control_packet` in
    /// `srtla/tests/test_broadcast_ack.cpp`.
    fn make_srt_control(srt_type: u16) -> [u8; 16] {
        let mut pkt = [0u8; 16];
        pkt[0..2].copy_from_slice(&srt_type.to_be_bytes());
        pkt
    }

    /// Mirror the NAK-attribution arm of `process_connection_events`
    /// (`sender/packet_handler.rs`): a NAK is first routed to the uplink the
    /// `SequenceTracker` (`sender/sequence.rs`) recorded as the sender of that
    /// sequence; if that lookup misses (never tracked, or the entry expired past
    /// `SEQUENCE_TRACKING_MAX_AGE_MS`) it falls back to scanning every uplink and
    /// letting the first one still holding the sequence in its packet_log account
    /// it. Crucially, once the tracked uplink IS found the fallback is suppressed
    /// even if its `handle_nak` reports the sequence already gone — that
    /// short-circuit (`handled = true`) is the dedup behavior that stops a
    /// duplicate NAK being re-counted on a different link.
    ///
    /// We replicate exactly that decision (the real `SequenceTracker::get` and
    /// `handle_nak` from `connection/ack_nak.rs` still run) so the contract is
    /// exercised without standing up the async UDP/registration harness — the
    /// same approach `connection_tests.rs::poll_keepalive` uses for the
    /// housekeeping keepalive gate.
    ///
    /// Returns the index of the uplink that actually counted the NAK, or `None`
    /// when it was a no-op (deduped, or attributable to no uplink).
    fn attribute_nak(
        connections: &mut [SrtlaConnection],
        seq_tracker: &SequenceTracker,
        nak: u32,
        current_time_ms: u64,
    ) -> Option<usize> {
        let mut counted_by: Option<usize> = None;
        let mut handled = false;

        if let Some(conn_id) = seq_tracker.get(nak, current_time_ms)
            && let Some(pos) = connections.iter().position(|c| c.conn_id == conn_id)
        {
            if connections[pos].handle_nak(nak as i32) {
                counted_by = Some(pos);
            }
            // Found the tracked uplink: do not fall through even if the sequence
            // was already cleared (duplicate NAK), so it can't be re-counted.
            handled = true;
        }

        if !handled {
            for (i, conn) in connections.iter_mut().enumerate() {
                if conn.handle_nak(nak as i32) {
                    counted_by = Some(i);
                    break;
                }
            }
        }

        counted_by
    }

    /// Port of `srtla/tests/test_broadcast_ack.cpp` (ACK arm) plus the ACK
    /// fan-out in `process_connection_events`: an SRT ACK is broadcast to *every*
    /// uplink and is cumulative, so each link clears its own in-flight packets
    /// with seq ≤ ack; a NAK is never broadcast. We first lock the broadcast
    /// *predicate* the C test pins (an ACK classifies as ACK; a NAK / data packet
    /// does not), then drive the real cumulative `handle_srt_ack`
    /// (`connection/ack_nak.rs`) across a 3-uplink pool and assert in-flight drops
    /// only where seq ≤ ack.
    #[test]
    fn ack_reduces_in_flight() {
        // -- broadcast eligibility predicate (mirrors test_broadcast_ack.cpp) --
        let ack_pkt = make_srt_control(SRT_TYPE_ACK);
        let nak_pkt = make_srt_control(SRT_TYPE_NAK);
        let data_pkt = make_srt_control(SRT_TYPE_DATA);
        assert!(
            is_srt_ack(&ack_pkt),
            "an ACK packet must be ACK-classified (broadcast-eligible)"
        );
        assert!(!is_srt_ack(&nak_pkt), "a NAK packet is not an ACK");
        assert_eq!(
            get_packet_type(&nak_pkt),
            Some(SRT_TYPE_NAK),
            "the NAK packet must classify as NAK"
        );
        assert!(
            !is_srt_ack(&data_pkt),
            "an SRT data packet is never ACK-broadcast-eligible"
        );

        // -- cumulative ACK reduces in-flight on the correct uplink(s) --
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));
        let now = now_ms();

        // uplink 0 sent 10/20/30 ; uplink 1 sent 15/25 ; uplink 2 sent 100 (beyond ack)
        connections[0].register_packet(10, now);
        connections[0].register_packet(20, now);
        connections[0].register_packet(30, now);
        connections[1].register_packet(15, now);
        connections[1].register_packet(25, now);
        connections[2].register_packet(100, now);
        assert_eq!(connections[0].in_flight_packets, 3);
        assert_eq!(connections[1].in_flight_packets, 2);
        assert_eq!(connections[2].in_flight_packets, 1);

        // Broadcast a cumulative ACK of 30 to every uplink, exactly as
        // process_connection_events does (`for c in connections { c.handle_srt_ack }`).
        for c in connections.iter_mut() {
            c.handle_srt_ack(30, now, false);
        }

        assert_eq!(
            connections[0].in_flight_packets, 0,
            "uplink 0: 10/20/30 all ≤ 30, cleared"
        );
        assert_eq!(
            connections[1].in_flight_packets, 0,
            "uplink 1: 15/25 ≤ 30, cleared"
        );
        assert_eq!(
            connections[2].in_flight_packets, 1,
            "uplink 2: seq 100 > 30 stays in-flight (ACK is cumulative, not blanket)"
        );
    }

    /// Port of the NAK-attribution contract (README "Implementation Details":
    /// NAKs are attributed to the uplink that originally sent the sequence,
    /// tracked via the `SequenceTracker`). Forward seq S on uplink 1, record it in
    /// the tracker, inject a NAK for S, and assert only uplink 1 is penalized
    /// (nak_count++ and in-flight−−); the other uplinks are untouched.
    #[test]
    fn nak_attributed_to_sending_uplink() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));
        let now = now_ms();

        let seq: u32 = 500;
        // Uplink 1 is the sender: register in its packet_log + record attribution.
        connections[1].register_packet(seq as i32, now);
        let mut seq_tracker = SequenceTracker::new();
        seq_tracker.insert(seq, connections[1].conn_id, now);

        let before: Vec<i32> = connections.iter().map(|c| c.congestion.nak_count).collect();
        let before_inflight = connections[1].in_flight_packets;

        let counted = attribute_nak(&mut connections, &seq_tracker, seq, now);

        assert_eq!(
            counted,
            Some(1),
            "the NAK must be attributed to the uplink that sent the sequence"
        );
        assert_eq!(
            connections[1].congestion.nak_count,
            before[1] + 1,
            "sending uplink's nak_count increments"
        );
        assert_eq!(
            connections[1].in_flight_packets,
            before_inflight - 1,
            "sending uplink's in-flight decreases"
        );
        assert_eq!(
            connections[0].congestion.nak_count, before[0],
            "uplink 0 (did not send S) is untouched"
        );
        assert_eq!(
            connections[2].congestion.nak_count, before[2],
            "uplink 2 (did not send S) is untouched"
        );
    }

    /// Port of the documented NAK fallback: when the sequence is *not* tracked the
    /// sender scans uplinks and lets the one still holding it in its packet_log
    /// account the NAK (README: "falling back ... if unknown"). Because a sequence
    /// only ever lives in its real sender's packet_log, the fallback still lands
    /// on the originating uplink. A sequence held by NO uplink is silently ignored
    /// — never double-counted, never a panic.
    #[test]
    fn nak_unknown_uplink_fallback() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));
        let now = now_ms();
        let seq_tracker = SequenceTracker::new(); // deliberately empty: nothing tracked

        // (a) untracked but present in uplink 2's packet_log → fallback finds it.
        let known: u32 = 700;
        connections[2].register_packet(known as i32, now);
        let before2 = connections[2].congestion.nak_count;

        let counted = attribute_nak(&mut connections, &seq_tracker, known, now);
        assert_eq!(
            counted,
            Some(2),
            "fallback attributes the NAK to the uplink still holding the sequence"
        );
        assert_eq!(connections[2].congestion.nak_count, before2 + 1);
        assert_eq!(connections[0].congestion.nak_count, 0);
        assert_eq!(connections[1].congestion.nak_count, 0);

        // (b) truly unknown: not tracked and in no packet_log → no-op.
        let counts_before: Vec<i32> = connections.iter().map(|c| c.congestion.nak_count).collect();
        let counted_unknown = attribute_nak(&mut connections, &seq_tracker, 999_999, now);
        assert_eq!(
            counted_unknown, None,
            "a sequence no uplink holds is attributable to none"
        );
        let counts_after: Vec<i32> = connections.iter().map(|c| c.congestion.nak_count).collect();
        assert_eq!(
            counts_before, counts_after,
            "an unattributable NAK must not perturb any uplink"
        );
    }

    /// Port of `srtla/tests/test_nak_dedup.cpp` (dedup within the suppression
    /// window). The Rust sender has no separate `NakDeduplicator`; dedup is
    /// structural — `handle_nak` removes the sequence from the packet_log, so a
    /// *second* NAK for the same sequence finds nothing and `handle_nak` returns
    /// false (not counted). Inside the `SequenceTracker` window both NAKs route to
    /// the same uplink, and the attribution short-circuit keeps the duplicate from
    /// falling through to another link. We assert single accounting under a
    /// *paused* virtual clock (no real sleep); the tracker's own window is driven
    /// with explicit timestamps, exactly as the C test uses explicit ms
    /// (1000, 1000+SUPPRESS_MS, …).
    #[tokio::test(start_paused = true)]
    async fn nak_dedup_within_window() {
        let mut connections = create_test_connections(2).await;
        let base = now_ms();

        let seq: u32 = 800;
        connections[0].register_packet(seq as i32, base);
        let mut seq_tracker = SequenceTracker::new();
        seq_tracker.insert(seq, connections[0].conn_id, base);
        assert_eq!(connections[0].in_flight_packets, 1);

        // First sighting inside the window: counted once on the sending uplink.
        let first = attribute_nak(&mut connections, &seq_tracker, seq, base);
        assert_eq!(first, Some(0));
        assert_eq!(
            connections[0].congestion.nak_count, 1,
            "first NAK is counted"
        );
        assert_eq!(
            connections[0].in_flight_packets, 0,
            "first NAK clears the in-flight packet"
        );

        // Advance the paused clock well within the tracking window (no real sleep).
        advance_test_clock(Duration::from_millis(50)).await;
        let within = base + 50;
        assert!(
            50 < SEQUENCE_TRACKING_MAX_AGE_MS,
            "50ms must be inside the dedup window"
        );
        assert_eq!(
            seq_tracker.get(seq, within),
            Some(connections[0].conn_id),
            "the tracker still resolves the sequence to uplink 0 inside the window"
        );

        // Duplicate NAK inside the window: routed to the same uplink, whose
        // packet_log no longer holds the sequence → not re-counted, and the
        // short-circuit keeps the other uplink clean.
        let dup = attribute_nak(&mut connections, &seq_tracker, seq, within);
        assert_eq!(
            dup, None,
            "the duplicate NAK is a no-op (single accounting)"
        );
        assert_eq!(
            connections[0].congestion.nak_count, 1,
            "duplicate NAK within the window is NOT double-counted"
        );
        assert_eq!(
            connections[0].in_flight_packets, 0,
            "in-flight stays cleared after the duplicate"
        );
        assert_eq!(
            connections[1].congestion.nak_count, 0,
            "the duplicate never leaks onto another uplink"
        );
    }

    // ====================================================================
    // T13 — Selection-scoring gap tests
    //
    // These pin the under-covered scoring logic that the wider suite only
    // exercised indirectly: the enhanced 10% switch hysteresis, the quality
    // RTT bonus (cap + MIN_RTT floor), the startup grace period (light vs full
    // penalty), classic tie-breaking, and the exploration cooldown gate.
    //
    // Scoring math notes (mirrors src/sender/selection/):
    //   * capacity score  = window / (in_flight + 1)               (classic.rs / get_score)
    //   * hysteresis stay  iff best_score < current * 1.10          (enhanced.rs SWITCH_THRESHOLD)
    //   * grace (<30s) NAK → 0.98 light penalty                     (quality.rs STARTUP_NAK_PENALTY)
    //   * post-grace NAK   → 1.0 - 0.5*e^(-age/2000)               (quality.rs exponential decay)
    //   * rtt bonus        = min(200 / max(rtt, 50), 1.03), >= 1.0  (quality.rs RTT bonus)
    //   * perfect (no NAK) → 1.1 quality multiplier                (quality.rs PERFECT_CONNECTION_BONUS)
    //
    // Quality is disabled in the hysteresis/exploration tests so the capacity
    // score alone drives the ranking and a NAK changes only the exploration
    // signal, never the score order. Time-dependent grace tests run on the
    // paused Tokio virtual clock and drive it via `advance_test_clock` (the
    // same seam `nak_dedup_within_window` uses).
    // ====================================================================

    /// Mirrors the private `quality::PERFECT_CONNECTION_BONUS`: a never-NAKed,
    /// post-grace link has quality multiplier 1.1, so dividing the result by it
    /// isolates the RTT bonus factor.
    const PERFECT_BONUS_FOR_TEST: f64 = 1.1;

    /// Enhanced hysteresis: a candidate only 9% better than the current link
    /// (1.09 < SWITCH_THRESHOLD = 1.10) must NOT trigger a switch — the sender
    /// stays on the current connection to avoid noise-driven flip-flopping.
    #[test]
    fn hysteresis_stays_below_threshold() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        connections[0].window = 100; // current link → score 100
        connections[0].in_flight_packets = 0;
        connections[1].window = 109; // 109 / 100 = 1.09 < 1.10 threshold
        connections[1].in_flight_packets = 0;
        connections[2].window = 50;
        connections[2].in_flight_packets = 0;

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        let last_switch = now_ms();
        let current = last_switch + 1000; // well past the 15ms switch cooldown

        let selected = select_connection_idx(
            &mut connections,
            Some(0),
            last_switch,
            current,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        assert_eq!(
            selected,
            Some(0),
            "a 9% improvement is below the 10% hysteresis threshold; must stay on the current link"
        );
    }

    /// Enhanced hysteresis: a candidate 11% better than the current link
    /// (1.11 > SWITCH_THRESHOLD = 1.10) is meaningfully better and MUST trigger
    /// a switch once the switch cooldown has elapsed.
    #[test]
    fn hysteresis_switches_above_threshold() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        connections[0].window = 100; // current link → score 100
        connections[0].in_flight_packets = 0;
        connections[1].window = 111; // 111 / 100 = 1.11 > 1.10 threshold
        connections[1].in_flight_packets = 0;
        connections[2].window = 50;
        connections[2].in_flight_packets = 0;

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        let last_switch = now_ms();
        let current = last_switch + 1000;

        let selected = select_connection_idx(
            &mut connections,
            Some(0),
            last_switch,
            current,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        assert_eq!(
            selected,
            Some(1),
            "an 11% improvement exceeds the 10% hysteresis threshold; must switch to the better \
             link"
        );
    }

    /// RTT bonus is capped at MAX_RTT_BONUS (1.03). An extremely low RTT (10ms)
    /// would otherwise yield 200/50 = 4x; the cap keeps the bonus at 1.03 so RTT
    /// can never dominate the quality score. Verified via the perfect-connection
    /// path where the quality multiplier is the constant 1.1, so the residual
    /// (mult / 1.1) is exactly the RTT bonus.
    #[tokio::test]
    async fn rtt_bonus_capped_at_max() {
        let mut conn = create_test_connections(1).await.into_iter().next().unwrap();
        let base = now_ms();

        // Past the startup grace period, never NAKed → quality multiplier = 1.1.
        conn.reconnection.connection_established_ms = base - 35_000;
        conn.congestion.nak_count = 0;
        conn.congestion.last_nak_time_ms = 0;

        // First Kalman sample sets the smoothed RTT exactly; 10ms is below the
        // MIN_RTT_MS floor so the raw factor would be 200/50 = 4 before the cap.
        conn.rtt.update_estimate(10);

        let mult = calculate_quality_multiplier(&conn, base);
        let rtt_bonus = mult / PERFECT_BONUS_FOR_TEST;

        assert!(
            rtt_bonus <= 1.03 + 1e-9,
            "RTT bonus must never exceed the 1.03 cap, got {rtt_bonus}"
        );
        assert!(
            (rtt_bonus - 1.03).abs() < 1e-6,
            "at 10ms the bonus saturates exactly at the 1.03 cap, got {rtt_bonus}"
        );
    }

    /// RTT bonus applies the MIN_RTT_MS (50ms) floor: any RTT below 50ms is
    /// clamped up to 50ms, so a 1ms link and a 50ms link receive the identical
    /// bonus (both saturate at the 1.03 cap), while a 200ms link earns no bonus
    /// (200/200 = 1.0). This pins both the floor and the no-bonus region.
    #[tokio::test]
    async fn rtt_bonus_uses_min_rtt_floor() {
        let mut connections = create_test_connections(3).await;
        let base = now_ms();

        for c in connections.iter_mut() {
            c.reconnection.connection_established_ms = base - 35_000; // past grace
            c.congestion.nak_count = 0;
            c.congestion.last_nak_time_ms = 0;
        }

        connections[0].rtt.update_estimate(1); // below the 50ms floor → clamped to 50
        connections[1].rtt.update_estimate(50); // exactly the floor
        connections[2].rtt.update_estimate(200); // at the bonus threshold → no bonus

        let bonus_1ms =
            calculate_quality_multiplier(&connections[0], base) / PERFECT_BONUS_FOR_TEST;
        let bonus_50ms =
            calculate_quality_multiplier(&connections[1], base) / PERFECT_BONUS_FOR_TEST;
        let bonus_200ms =
            calculate_quality_multiplier(&connections[2], base) / PERFECT_BONUS_FOR_TEST;

        assert!(
            (bonus_1ms - bonus_50ms).abs() < 1e-9,
            "a sub-floor RTT (1ms) must be clamped to MIN_RTT_MS (50ms): {bonus_1ms} vs \
             {bonus_50ms}"
        );
        assert!(
            (bonus_50ms - 1.03).abs() < 1e-6,
            "at/below the floor the bonus saturates at 1.03, got {bonus_50ms}"
        );
        assert!(
            (bonus_200ms - 1.0).abs() < 1e-6,
            "at the 200ms bonus threshold there is no bonus (factor 1.0), got {bonus_200ms}"
        );
    }

    /// Startup grace period: within the first 30s of a connection's life a NAK
    /// applies only the light STARTUP_NAK_PENALTY (0.98), not the full
    /// exponential penalty — so an early loss cannot permanently demote a link.
    /// Driven on the paused virtual clock to show the link is still in grace 5s in.
    #[tokio::test(start_paused = true)]
    async fn quality_startup_grace_light_penalty() {
        let mut conn = create_test_connections(1).await.into_iter().next().unwrap();
        let base = now_ms();

        conn.reconnection.connection_established_ms = base;
        conn.congestion.nak_count = 1; // a NAK occurred during the grace window

        // Advance the virtual clock 5s — still well inside the 30s grace period.
        advance_test_clock(Duration::from_secs(5)).await;
        let current = base + 5_000; // age 5s < 30s grace

        let mult = calculate_quality_multiplier(&conn, current);

        assert!(
            (mult - 0.98).abs() < 1e-9,
            "during the grace period a NAK yields only the 0.98 light penalty, got {mult}"
        );
    }

    /// Startup grace period: once the connection is older than 30s, the same NAK
    /// state no longer gets the light penalty — the full exponential decay
    /// applies (a fresh NAK → ~0.5), which is materially heavier than the 0.98
    /// grace penalty. Driven on the paused virtual clock to age past the window.
    #[tokio::test(start_paused = true)]
    async fn quality_startup_grace_full_penalty_after() {
        let mut conn = create_test_connections(1).await.into_iter().next().unwrap();
        let base = now_ms();

        conn.reconnection.connection_established_ms = base;
        conn.congestion.nak_count = 1;

        // Age the link past the 30s grace window via the virtual clock.
        advance_test_clock(Duration::from_secs(31)).await;
        let current = base + 31_000; // age 31s >= 30s grace

        // A just-now NAK (age ~0) → exponential penalty 1.0 - 0.5*e^0 = 0.5.
        conn.congestion.last_nak_time_ms = now_ms();

        let mult = calculate_quality_multiplier(&conn, current);

        assert!(
            (mult - 0.5).abs() < 0.02,
            "after grace a fresh NAK applies the full ~0.5 exponential penalty, got {mult}"
        );
        assert!(
            mult < 0.98,
            "the post-grace penalty must be heavier than the 0.98 grace light penalty, got {mult}"
        );
    }

    /// Classic mode tie-break: when two links have identical capacity scores the
    /// FIRST (lowest index) wins, because `select_connection` only replaces the
    /// best on a strictly greater score. This matches the original C behavior.
    #[test]
    fn classic_tie_break_first_wins() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        // conns 0 and 1 tie on score (100); conn 2 is worse.
        connections[0].window = 100;
        connections[0].in_flight_packets = 0;
        connections[1].window = 100;
        connections[1].in_flight_packets = 0;
        connections[2].window = 50;
        connections[2].in_flight_packets = 0;

        let config = ConfigSnapshot {
            mode: SchedulingMode::Classic,
            quality_enabled: false,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        let selected = select_connection_idx(
            &mut connections,
            None,
            0,
            0,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        assert_eq!(
            selected,
            Some(0),
            "equal scores must resolve to the first (lowest-index) connection"
        );
    }

    /// Exploration fallback: with exploration enabled and the switch cooldown
    /// elapsed, when the capacity-best link is degrading (recent NAK) and the
    /// second-best has recovered, the sender explores the second-best link
    /// instead of sticking to the best. Quality is disabled so the NAK changes
    /// only the exploration signal, not the capacity ranking.
    #[test]
    fn exploration_periodic_fallback() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut connections = rt.block_on(create_test_connections(3));

        // Capacity ranking: conn0 best, conn1 second, conn2 worst (and current).
        connections[0].window = 100;
        connections[0].in_flight_packets = 0;
        connections[1].window = 80;
        connections[1].in_flight_packets = 0;
        connections[2].window = 10;
        connections[2].in_flight_packets = 0;

        // Best link is degrading (NAK ~now); second-best has no NAKs (recovered).
        connections[0].congestion.nak_count = 1;
        connections[0].congestion.last_nak_time_ms = now_ms();

        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: false,
            exploration_enabled: true,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };

        let last_switch = now_ms();
        let current = last_switch + 1000; // past the 15ms switch cooldown

        let selected = select_connection_idx(
            &mut connections,
            Some(2),
            last_switch,
            current,
            &config,
            &mut EdpfSchedulerState::default(),
        );

        assert_eq!(
            selected,
            Some(1),
            "exploration must route to the recovered second-best link when the best is degrading"
        );
    }
}

#[cfg(test)]
mod lifecycle {
    use std::collections::HashMap;
    use std::sync::Arc;

    use smallvec::SmallVec;
    use tokio::net::UdpSocket;
    use tokio::time::{Duration, timeout};

    use crate::bind_map::{IfaceName, LinkId};
    use crate::config::ConfigSnapshot;
    use crate::connection::delivery::{DataSend, DeliveryAck};
    use crate::connection::health::{HealthMachine, HealthState};
    use crate::connection::rate_cap::RateState;
    use crate::connection::{RouteHealth, SrtlaConnection};
    use crate::mode::SchedulingMode;
    use crate::protocol::{SRTLA_TYPE_ACK, SRTLA_TYPE_REG3};
    use crate::registration::SrtlaRegistrationManager;
    use crate::sender::housekeeping::{handle_housekeeping, tick_health};
    use crate::sender::packet_handler::handle_uplink_packet;
    use crate::sender::uplink::{UplinkPacket, create_uplink_channel, restart_reader_for};
    use crate::sender::{SequenceTracker, apply_link_changes};
    use crate::stats::SharedStats;
    use crate::test_helpers::create_test_connection;
    use crate::utils::now_ms;

    async fn dispatch(
        conn: &mut SrtlaConnection,
        reg: &mut SrtlaRegistrationManager,
        packet: UplinkPacket,
    ) {
        let local = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        handle_uplink_packet(
            packet,
            std::slice::from_mut(conn),
            reg,
            &tx,
            None,
            &local,
            &SequenceTracker::new(),
            &ConfigSnapshot {
                mode: SchedulingMode::Adaptive,
                ..crate::config::DynamicConfig::new().snapshot()
            },
            &SharedStats::new(),
        )
        .await;
    }

    async fn register(conn: &mut SrtlaConnection) {
        let mut reg = SrtlaRegistrationManager::new();
        reg.arm_reg3_gate(0);
        let packet = UplinkPacket {
            conn_id: conn.conn_id,
            reader_generation: conn.delivery.socket_generation,
            bytes: SmallVec::from_slice_copy(&SRTLA_TYPE_REG3.to_be_bytes()),
        };
        dispatch(conn, &mut reg, packet).await;
    }

    async fn housekeeping(conn: &mut SrtlaConnection) {
        let (tx, _rx) = create_uplink_channel();
        let mut readers = HashMap::new();
        handle_housekeeping(
            std::slice::from_mut(conn),
            &mut SrtlaRegistrationManager::new(),
            false,
            &mut None,
            &mut readers,
            &tx,
            &mut SequenceTracker::new(),
        )
        .await
        .unwrap();
        for reader in readers.into_values() {
            reader.handle.abort();
        }
    }

    fn credit(conn: &mut SrtlaConnection, now: u64) {
        for seq in 0..300 {
            conn.delivery.record_sent(
                seq,
                DataSend {
                    sent_ms: now,
                    len: 1316,
                },
            );
            assert!(conn.delivery.acknowledge(
                DeliveryAck {
                    seq,
                    socket_generation: conn.delivery.socket_generation
                },
                now
            ));
        }
    }

    #[tokio::test]
    async fn housekeeping_enters_rejoining_after_reg3() {
        // Given: the real registration handler has accepted REG3 on a fresh link.
        let mut conn = create_test_connection().await;
        conn.connected = false;
        register(&mut conn).await;
        // When: the production housekeeping entry runs.
        housekeeping(&mut conn).await;
        // Then: startup leaves the fallback-only Down state through the ramp.
        assert_eq!(conn.health.state(), HealthState::Rejoining);
    }

    #[tokio::test]
    async fn reorder_under_load_keeps_rejoining_ramp_progress() {
        // Given: a mapped survivor midway through its ramp with credited DATA.
        let mut conn = create_test_connection().await;
        conn.link_id = Some(LinkId::parse("modem-a").unwrap());
        let start = now_ms();
        tick_health(std::slice::from_mut(&mut conn), &[], start);
        credit(&mut conn, start + 1000);
        tick_health(std::slice::from_mut(&mut conn), &[], start + 1000);
        let cap = conn.rate_cap.target_bps();
        let progress = conn.health.ramp_multiplier(start + 1000, 0);
        assert!(progress > 0.05 && progress < 1.0);
        let socket = conn.socket.clone();
        let generation = conn.delivery.socket_generation;
        let mut other = create_test_connection().await;
        other.link_id = Some(LinkId::parse("modem-b").unwrap());
        other.local_ip = "127.0.0.2".parse().unwrap();
        let reordered = [other.spec(), conn.spec()];
        let mut conns: SmallVec<_, 4> = [conn, other].into_iter().collect();
        let mut reg = SrtlaRegistrationManager::new();
        reg.arm_reg3_gate(0);
        // When: SIGHUP applies the opposite file order.
        apply_link_changes(
            &mut conns,
            &reordered,
            "127.0.0.1",
            8080,
            &mut Some(0),
            &mut SequenceTracker::new(),
            &mut reg,
        )
        .await;
        // Then: identity keeps the same ramp, delivered-rate controller and socket.
        assert_eq!(conns[1].health.state(), HealthState::Rejoining);
        assert_eq!(conns[1].health.ramp_multiplier(start + 1000, 0), progress);
        assert_eq!(conns[1].rate_cap.target_bps(), cap);
        assert_eq!(conns[1].delivery.socket_generation, generation);
        assert!(Arc::ptr_eq(&conns[1].socket, &socket));
        assert!(!reg.is_awaiting_reg3(0));
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn replug_resets_down_then_reg3_rejoins() {
        // Given: a live link with a non-bootstrap rate and health history.
        let mut conn = create_test_connection().await;
        let start = now_ms();
        conn.health = HealthMachine::new(HealthState::Healthy, start);
        credit(&mut conn, start);
        tick_health(std::slice::from_mut(&mut conn), &[], start);
        assert!(conn.rate_cap.target_bps() > 1_000_000.0);
        let generation = conn.delivery.socket_generation;
        struct OldIfindex;
        impl crate::connection::IfaceResolver for OldIfindex {
            fn ifindex(&self, _: &str) -> Option<u32> {
                Some(u32::MAX)
            }
        }
        conn.egress.adopt(Some(IfaceName::parse("lo").unwrap()));
        conn.egress.resolve_for_bind(&OldIfindex).unwrap();
        // When: housekeeping observes a new kernel ifindex for the same name.
        housekeeping(&mut conn).await;
        // Then: old measurements cannot promote the new socket.
        assert_eq!(conn.health.state(), HealthState::Down);
        assert_eq!(conn.rate_cap.state(), RateState::Bootstrap);
        assert_eq!(conn.delivery.latest_data_proof_ms(), None);
        assert_eq!(conn.loss.last_value(), None);
        assert_ne!(conn.delivery.socket_generation, generation);
        register(&mut conn).await;
        tick_health(std::slice::from_mut(&mut conn), &[], start + 1000);
        assert_eq!(conn.health.state(), HealthState::Rejoining);
        assert_eq!(conn.health.ramp_multiplier(start + 1000, 0), 0.05);
    }

    #[tokio::test]
    async fn recovery_resets_rate_and_health_with_delivery_generation() {
        // Given: accumulated delivery, loss and admission state on a live socket.
        let mut conn = create_test_connection().await;
        let now = now_ms();
        conn.health = HealthMachine::new(HealthState::Healthy, now);
        conn.loss = crate::connection::loss::LossTracker::new(now);
        for _ in 0..100 {
            conn.loss.record_send(now);
            conn.loss.record_data_nak(now);
        }
        credit(&mut conn, now);
        tick_health(std::slice::from_mut(&mut conn), &[], now + 1000);
        conn.adaptive.sole_failed_until_proof = true;
        let generation = conn.delivery.socket_generation;
        // When: a transmit failure marks that connection for recovery.
        conn.mark_for_recovery();
        // Then: no socket-scoped adaptive evidence survives the generation boundary.
        assert_eq!(conn.health.state(), HealthState::Down);
        assert_eq!(conn.rate_cap.state(), RateState::Bootstrap);
        assert_eq!(conn.loss.last_value(), None);
        assert_eq!(conn.delivery.latest_data_proof_ms(), None);
        assert!(!conn.adaptive.sole_failed_until_proof);
        assert_eq!(conn.delivery.socket_generation, generation.wrapping_add(1));
    }

    #[tokio::test]
    async fn health_tick_expires_probes_even_on_down_links() {
        // Given: two incomplete trains retained on an unavailable link.
        let mut conn = create_test_connection().await;
        conn.connected = false;
        for (seq, id) in [(1, 1), (2, 2)] {
            conn.probes.record_sent(
                seq,
                crate::connection::probe::ProbeTrain {
                    id,
                    started_ms: 1000,
                    deadline_ms: 3000,
                },
                1000,
            );
        }
        // When: the idle housekeeping sampler passes their deadlines.
        tick_health(std::slice::from_mut(&mut conn), &[], 3001);
        // Then: absent copies count as loss, not successful rejoin rounds.
        assert_eq!(conn.loss.probe_loss(), Some(1.0));
        assert_eq!(conn.probes.rounds_ok(), (0, None));
        assert_eq!(conn.health.state(), HealthState::Down);
        assert!(conn.probes.probe_log.is_empty());
    }

    #[tokio::test]
    async fn ramp_completion_promotes_to_healthy_without_synthetic_proof() {
        // Given: a registered idle link at the start of its unknown-RTT ramp.
        let mut conn = create_test_connection().await;
        tick_health(std::slice::from_mut(&mut conn), &[], 1000);
        // When: the full six-second ramp completes.
        tick_health(std::slice::from_mut(&mut conn), &[], 7000);
        // Then: Healthy is reached without inventing DATA or loss evidence.
        assert_eq!(conn.health.state(), HealthState::Healthy);
        assert_eq!(conn.delivery.latest_data_proof_ms(), None);
        assert_eq!(conn.loss.last_value(), None);
    }

    #[tokio::test]
    async fn rate_ticks_are_one_second_apart_without_catch_up_growth() {
        // Given: delivered DATA and the persistent production tick owner.
        let mut conn = create_test_connection().await;
        let mut ticker = crate::sender::housekeeping::HealthTicker::default();
        credit(&mut conn, 1000);
        ticker.tick(std::slice::from_mut(&mut conn), &[], 1000);
        let seeded = conn.rate_cap.target_bps();
        // When: an early callback and a late callback sample the same controller.
        ticker.tick(std::slice::from_mut(&mut conn), &[], 1999);
        assert_eq!(conn.rate_cap.target_bps(), seeded);
        credit(&mut conn, 5000);
        ticker.tick(std::slice::from_mut(&mut conn), &[], 5000);
        // Then: only one normal 2% growth tick ran, not four catch-up ticks.
        assert!((conn.rate_cap.target_bps() - seeded * 1.02).abs() < 0.001);
    }

    #[tokio::test]
    async fn health_tick_stalls_on_data_silence_despite_live_inbound_rtt() {
        // Given: a live RTT path but 32 accepted DATA sends without proof.
        let mut conn = create_test_connection().await;
        let now = now_ms();
        conn.health = HealthMachine::new(HealthState::Healthy, now);
        conn.rtt.update_estimate(40);
        for seq in 0..32 {
            conn.delivery.record_sent(
                seq,
                DataSend {
                    sent_ms: now,
                    len: 1316,
                },
            );
        }
        // When: one measured-RTT stall dwell passes without a DATA ACK.
        tick_health(std::slice::from_mut(&mut conn), &[], now + 1000);
        // Then: inbound/RTT liveness cannot conceal the DATA obstruction.
        assert!(!conn.is_timed_out());
        assert_eq!(conn.health.state(), HealthState::Stalled);
        assert_eq!(conn.delivery.latest_data_proof_ms(), None);
    }

    #[tokio::test]
    async fn route_loss_degrades_without_down_or_fabricated_measurements() {
        // Given: a connected socket with no loss/queue evidence and a missing route.
        let mut conn = create_test_connection().await;
        conn.health = HealthMachine::new(HealthState::Healthy, 0);
        conn.route_health = RouteHealth::NoDefaultRoute;
        // When: the housekeeping sampler consumes the independent route observation.
        tick_health(std::slice::from_mut(&mut conn), &[], now_ms());
        // Then: route absence is soft degradation, never socket failure or invented loss.
        assert_eq!(conn.health.state(), HealthState::Degraded);
        assert!(conn.health.route_latched());
        assert!(!conn.health.loss_latched());
        assert_eq!(conn.loss.last_value(), None);
        assert_eq!(conn.rtt.queue_delay_ms(), 0.0);
        assert!(conn.connected);
    }

    #[tokio::test]
    async fn queued_old_reader_packet_after_reconnect_is_not_proof() {
        // Given: an actual old-reader ACK queued before replacement, then sequence reuse.
        let mut conn = create_test_connection().await;
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let (tx, mut rx) = create_uplink_channel();
        let mut readers = HashMap::new();
        restart_reader_for(&conn, &mut readers, &tx);
        let mut ack = Vec::from(SRTLA_TYPE_ACK.to_be_bytes());
        ack.extend_from_slice(&[0, 0]);
        ack.extend_from_slice(&42_u32.to_be_bytes());
        peer.send_to(&ack, conn.socket.local_addr().unwrap())
            .await
            .unwrap();
        let old = timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();
        conn.reconnect().await.unwrap();
        restart_reader_for(&conn, &mut readers, &tx);
        register(&mut conn).await;
        conn.delivery.record_sent(
            42,
            DataSend {
                sent_ms: now_ms(),
                len: 1316,
            },
        );
        // When: the old queued packet is dispatched against the replacement link.
        dispatch(&mut conn, &mut SrtlaRegistrationManager::new(), old).await;
        // Then: no proof; the NEW reader's same ACK does provide proof.
        assert_eq!(conn.delivery.latest_data_proof_ms(), None);
        peer.send_to(&ack, conn.socket.local_addr().unwrap())
            .await
            .unwrap();
        let new = timeout(Duration::from_secs(1), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(new.reader_generation, conn.delivery.socket_generation);
        dispatch(&mut conn, &mut SrtlaRegistrationManager::new(), new).await;
        assert!(conn.delivery.latest_data_proof_ms().is_some());
        for reader in readers.into_values() {
            reader.handle.abort();
        }
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn sighup_iface_move_starts_fresh_down_then_rejoins() {
        // Given: an old mapped interface and accumulated state under a stable identity.
        let mut conn = create_test_connection().await;
        conn.link_id = Some(LinkId::parse("modem-a").unwrap());
        conn.egress
            .adopt(Some(IfaceName::parse("old-modem").unwrap()));
        conn.health = HealthMachine::new(HealthState::Healthy, 0);
        credit(&mut conn, now_ms());
        let mut replacement = conn.spec();
        replacement.iface = Some(IfaceName::parse("lo").unwrap());
        let old_socket = conn.socket.clone();
        let mut conns: SmallVec<_, 4> = [conn].into_iter().collect();
        // When: the SIGHUP pool rebuild changes that identity's interface binding.
        apply_link_changes(
            &mut conns,
            &[replacement],
            "127.0.0.1",
            8080,
            &mut None,
            &mut SequenceTracker::new(),
            &mut SrtlaRegistrationManager::new(),
        )
        .await;
        // Then: it really created a fresh socket, not just dropped a failed bind.
        assert_eq!(conns.len(), 1);
        assert!(!Arc::ptr_eq(&conns[0].socket, &old_socket));
        assert_eq!(conns[0].health.state(), HealthState::Down);
        assert_eq!(conns[0].rate_cap.state(), RateState::Bootstrap);
        assert_eq!(conns[0].delivery.latest_data_proof_ms(), None);
        register(&mut conns[0]).await;
        tick_health(&mut conns, &[], now_ms());
        assert_eq!(conns[0].health.state(), HealthState::Rejoining);
    }
}
