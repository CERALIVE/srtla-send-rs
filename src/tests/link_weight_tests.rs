//! Tests for operator link weights (Moblin's "connection priorities"): the
//! optional second column of the IPs file, normalised so the lowest link is 1,
//! applied by classic selection only through a three-band window multiplier.
//! Parsing and the reload guard are covered in `sender::reload`'s own tests.

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use srtla_core::connection::{
        LINK_WEIGHT_FULL_ABOVE_WINDOW, LINK_WEIGHT_NONE_AT_OR_BELOW_WINDOW, link_weight_multiplier,
        normalise_link_weights,
    };
    use srtla_core::mode::SchedulingMode;
    use srtla_core::selection::select_connection_idx;
    use srtla_core::utils::now_ms;

    use crate::config::{ConfigSnapshot, STALL_ACK_STALE_MS, STALL_MIN_IN_FLIGHT_PACKETS};
    use crate::sender::{SequenceTracker, apply_connection_changes, specs_from_ips};
    use crate::stats::SharedStats;
    use crate::test_helpers::{create_test_conn_io_map, create_test_connections};

    fn classic() -> ConfigSnapshot {
        ConfigSnapshot {
            mode: SchedulingMode::Classic,
            quality_enabled: false,
            ..ConfigSnapshot::default()
        }
    }

    /// Feed `packets` packets through classic selection, counting each pick as
    /// one more in-flight packet on the chosen link (no ACKs): the share each
    /// link ends up with is the scheduler's steady preference at these windows.
    fn picks(conns: &mut [srtla_core::connection::SrtlaConnection], packets: usize) -> Vec<usize> {
        let config = classic();
        let mut counts = vec![0usize; conns.len()];
        for _ in 0..packets {
            let now = now_ms();
            let idx = select_connection_idx(conns, None, now, &config).expect("a link");
            conns[idx].in_flight_packets += 1;
            counts[idx] += 1;
        }
        counts
    }

    #[test]
    fn multiplier_full_weight_above_20k() {
        assert_eq!(link_weight_multiplier(60_000, 10), 10.0);
        assert_eq!(link_weight_multiplier(20_001, 10), 10.0);
        assert_eq!(
            link_weight_multiplier(LINK_WEIGHT_FULL_ABOVE_WINDOW, 10),
            10.0
        );
    }

    #[test]
    fn multiplier_fades_linearly_between_10k_and_20k() {
        assert_eq!(link_weight_multiplier(15_000, 10), 5.5);
        assert_eq!(link_weight_multiplier(12_500, 5), 2.0);
        let just_above = link_weight_multiplier(LINK_WEIGHT_NONE_AT_OR_BELOW_WINDOW + 1, 10);
        assert!(just_above > 1.0 && just_above < 1.01, "{just_above}");
    }

    #[test]
    fn multiplier_ignores_weight_at_or_below_10k() {
        assert_eq!(
            link_weight_multiplier(LINK_WEIGHT_NONE_AT_OR_BELOW_WINDOW, 10),
            1.0
        );
        assert_eq!(link_weight_multiplier(1_000, 10), 1.0);
    }

    #[test]
    fn weight_one_is_neutral_in_every_band() {
        for window in [60_000, 20_000, 15_000, 10_000, 1] {
            assert_eq!(link_weight_multiplier(window, 1), 1.0);
        }
    }

    #[test]
    fn normalisation_makes_the_lowest_weight_one() {
        let mut w = [10u8, 1];
        normalise_link_weights(&mut w);
        assert_eq!(w, [10, 1]);
        let mut w = [10u8, 5, 7];
        normalise_link_weights(&mut w);
        assert_eq!(w, [6, 1, 3]);
        let mut w = [4u8, 4];
        normalise_link_weights(&mut w);
        assert_eq!(w, [1, 1]);
        let mut w = [0u8, 200];
        normalise_link_weights(&mut w);
        assert_eq!(w, [1, 10], "clamped to 1..10 before normalising");
        let mut w: [u8; 0] = [];
        normalise_link_weights(&mut w);
    }

    #[test]
    fn unweighted_links_share_evenly() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));
        let counts = picks(&mut conns, 200);
        assert_eq!(counts, vec![100, 100]);
    }

    #[test]
    fn classic_prefers_the_weighted_link_while_healthy() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));
        for c in conns.iter_mut() {
            c.window = 40_000; // healthy band
        }
        conns[0].link_weight = 10;
        let counts = picks(&mut conns, 220);
        let share_b = counts[1] as f64 / 220.0;
        assert!(counts[0] > counts[1], "{counts:?}");
        assert!(
            share_b > 0.0 && share_b < 0.12,
            "weight 10 leaves roughly one packet in ten to the other link: {counts:?}"
        );
    }

    #[test]
    fn preference_fades_as_the_weighted_links_window_shrinks() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let share_b_at = |window_a: i32| {
            let mut conns = rt.block_on(create_test_connections(2));
            conns[0].window = window_a;
            conns[1].window = window_a;
            conns[0].link_weight = 10;
            let counts = picks(&mut conns, 400);
            counts[1] as f64 / 400.0
        };
        let healthy = share_b_at(40_000);
        let shedding = share_b_at(15_000);
        let congested = share_b_at(10_000);
        assert!(healthy < shedding, "{healthy} {shedding}");
        assert!(shedding < congested, "{shedding} {congested}");
        assert!(
            (congested - 0.5).abs() < 0.01,
            "at or below 10k the weight is ignored: {congested}"
        );
    }

    #[test]
    fn a_small_window_on_the_weighted_link_hands_traffic_to_the_other() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));
        conns[0].link_weight = 10;
        conns[0].window = 5_000; // A is congested: no multiplier
        conns[1].window = 40_000; // B healthy, weight 1
        let counts = picks(&mut conns, 90);
        assert!(counts[1] > counts[0] * 4, "{counts:?}");
    }

    #[test]
    fn timed_out_weighted_link_is_still_skipped() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));
        let now = now_ms();
        conns[0].link_weight = 10;
        conns[0].window = 60_000;
        conns[0].last_received = Some(now.saturating_sub(60_000));
        conns[1].in_flight_packets = 30;
        let selected = select_connection_idx(&mut conns, None, now, &classic());
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn stall_gated_weighted_link_is_still_skipped() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));
        let now = now_ms();
        conns[0].link_weight = 10;
        conns[0].in_flight_packets = STALL_MIN_IN_FLIGHT_PACKETS;
        conns[0].last_ack_or_rtt_sample_ms = now.saturating_sub(STALL_ACK_STALE_MS + 1000);
        conns[1].in_flight_packets = STALL_MIN_IN_FLIGHT_PACKETS * 2;
        conns[1].last_ack_or_rtt_sample_ms = now;
        let selected = select_connection_idx(&mut conns, None, now, &classic());
        assert_eq!(selected, Some(1));
        assert!(conns[0].stall_gated);
    }

    #[test]
    fn enhanced_mode_ignores_weights() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));
        conns[0].link_weight = 10;
        conns[0].in_flight_packets = 5;
        conns[1].in_flight_packets = 0;
        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: false,
            ..ConfigSnapshot::default()
        };
        let selected = select_connection_idx(&mut conns, None, now_ms(), &config);
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn reload_reweights_existing_links_and_unnamed_links_fall_back_to_one() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));
        let a = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10));
        let b = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 11));
        assert_eq!(conns[0].local_ip, a);
        let mut io = rt.block_on(async { create_test_conn_io_map(&conns) });
        let binder: std::sync::Arc<dyn crate::net::UplinkBinder> =
            std::sync::Arc::new(crate::net::SourceIpBinder);
        let mut selection = None;
        let mut tracker = SequenceTracker::new();

        rt.block_on(apply_connection_changes(
            &mut conns,
            &mut io,
            &specs_from_ips(&[a, b], &[10, 1]),
            "127.0.0.1",
            5000,
            &mut selection,
            &mut tracker,
            &binder,
        ));
        assert_eq!((conns[0].link_weight, conns[1].link_weight), (10, 1));

        // A reload without weights puts everyone back to 1.
        rt.block_on(apply_connection_changes(
            &mut conns,
            &mut io,
            &specs_from_ips(&[a, b], &[1, 1]),
            "127.0.0.1",
            5000,
            &mut selection,
            &mut tracker,
            &binder,
        ));
        assert_eq!((conns[0].link_weight, conns[1].link_weight), (1, 1));

        // Weight moves to the other link.
        rt.block_on(apply_connection_changes(
            &mut conns,
            &mut io,
            &specs_from_ips(&[a, b], &[1, 7]),
            "127.0.0.1",
            5000,
            &mut selection,
            &mut tracker,
            &binder,
        ));
        assert_eq!((conns[0].link_weight, conns[1].link_weight), (1, 7));

        // An empty weight list (legacy caller) means 1.
        rt.block_on(apply_connection_changes(
            &mut conns,
            &mut io,
            &specs_from_ips(&[a, b], &[]),
            "127.0.0.1",
            5000,
            &mut selection,
            &mut tracker,
            &binder,
        ));
        assert_eq!((conns[0].link_weight, conns[1].link_weight), (1, 1));
    }

    #[test]
    fn weight_is_exported_per_link_in_metrics() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut conns = rt.block_on(create_test_connections(2));
        conns[0].link_weight = 10;
        let stats = SharedStats::new();
        stats.update(&conns, &classic(), None, None, None);
        let text = crate::metrics::render(
            &stats,
            &crate::config::DynamicConfig::new(),
            &srtla_core::priority::CriticalWindow::new(),
        );
        assert!(
            text.contains(r#"srtla_send_link_weight{ip="192.168.1.10"} 10"#),
            "{text}"
        );
        assert!(text.contains(r#"srtla_send_link_weight{ip="192.168.1.11"} 1"#));
    }

    // ---- Fork: weights belong to specs, never to socket identity ------------

    #[tokio::test]
    async fn fork_weight_only_reload_preserves_registration_and_reports_weights() {
        use crate::net::{SourceIpBinder, UplinkBinder};
        use crate::sender::{
            ConnIoMap, LinkSource, SenderPaths, create_connections_from_links,
            read_weighted_ip_list,
        };

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ips.txt");
        std::fs::write(&path, "127.0.0.1 10\n127.0.0.2 1\n").unwrap();
        let path = path.to_str().unwrap();
        let mut source = LinkSource::new(&SenderPaths {
            ips_file: path,
            bind_map: None,
        });
        let initial = source.startup().await.unwrap();
        assert_eq!(
            initial.iter().map(|s| s.weight).collect::<Vec<_>>(),
            [10, 1]
        );
        let binder: std::sync::Arc<dyn UplinkBinder> = std::sync::Arc::new(SourceIpBinder);
        let mut io = ConnIoMap::new();
        let mut conns =
            create_connections_from_links(&initial, "127.0.0.1", 9999, &binder, &mut io).await;
        assert_eq!(conns.len(), 2);
        assert_eq!((conns[0].link_weight, conns[1].link_weight), (10, 1));
        for conn in &mut conns {
            conn.connected = true;
            conn.phase = srtla_core::connection::LinkPhase::Live;
            conn.reconnection.connection_established_ms = 1234;
        }
        let ids: Vec<_> = conns.iter().map(|c| c.conn_id).collect();
        let sockets: Vec<_> = ids.iter().map(|id| io[id].socket.clone()).collect();
        let mut selection = Some(1);
        let mut tracker = SequenceTracker::new();
        let now = now_ms();
        tracker.insert(77, ids[0], now);
        conns[0].register_packet(77, now);

        std::fs::write(path, "127.0.0.1 1\n127.0.0.2 7\n").unwrap();
        let (ips, weights) = read_weighted_ip_list(path).await.unwrap();
        let desired = specs_from_ips(&ips, &weights);
        apply_connection_changes(
            &mut conns,
            &mut io,
            &desired,
            "127.0.0.1",
            9999,
            &mut selection,
            &mut tracker,
            &binder,
        )
        .await;

        assert_eq!(conns.iter().map(|c| c.conn_id).collect::<Vec<_>>(), ids);
        assert_eq!(selection, Some(1));
        assert_eq!(tracker.get(77, now), Some(ids[0]));
        assert!(conns[0].packet_log.contains_key(&77));
        for ((conn, spec), socket) in conns.iter().zip(&desired).zip(sockets) {
            assert!(conn.connected);
            assert_eq!(conn.phase, srtla_core::connection::LinkPhase::Live);
            assert_eq!(conn.reconnection.connection_established_ms, 1234);
            assert_eq!(conn.link_weight, spec.weight);
            assert_eq!(io[&conn.conn_id].spec, *spec);
            assert!(std::sync::Arc::ptr_eq(&io[&conn.conn_id].socket, &socket));
        }
        let stats = SharedStats::new();
        stats.update(&conns, &classic(), None, None, None);
        let response = crate::control::dispatch(
            &crate::config::DynamicConfig::new(),
            Some(&stats),
            None,
            r#"{"jsonrpc":"2.0","id":1,"method":"get_stats"}"#,
        )
        .unwrap();
        let response: serde_json::Value = serde_json::from_str(&response.to_json()).unwrap();
        assert_eq!(response["result"]["links"][0]["weight"], 1);
        assert_eq!(response["result"]["links"][1]["weight"], 7);
    }

    #[tokio::test]
    async fn fork_weight_is_outside_socket_identity_and_claims_survivor() {
        let mut conns = create_test_connections(1).await;
        let id = conns[0].conn_id;
        let mut io = create_test_conn_io_map(&conns);
        let before = io[&id].spec.clone();
        let after = crate::net::UplinkSpec {
            weight: 7,
            ..before.clone()
        };
        assert_eq!(before.socket_key(), after.socket_key());
        assert_eq!(before.link_id, after.link_id);
        let socket = io[&id].socket.clone();
        let binder: std::sync::Arc<dyn crate::net::UplinkBinder> =
            std::sync::Arc::new(crate::net::SourceIpBinder);
        apply_connection_changes(
            &mut conns,
            &mut io,
            std::slice::from_ref(&after),
            "127.0.0.1",
            9999,
            &mut None,
            &mut SequenceTracker::new(),
            &binder,
        )
        .await;
        assert_eq!(conns.len(), 1, "the survivor must be claimed");
        assert_eq!(conns[0].conn_id, id);
        assert!(conns[0].connected);
        assert_eq!(conns[0].link_weight, 7);
        assert_eq!(io[&id].spec, after);
        assert!(std::sync::Arc::ptr_eq(&io[&id].socket, &socket));
    }

    #[tokio::test]
    async fn fork_unweighted_file_gives_every_spec_weight_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ips.txt");
        std::fs::write(&path, "192.0.2.10\n198.51.100.23\n2001:db8::1\n").unwrap();
        let (ips, weights) = crate::sender::read_weighted_ip_list(path.to_str().unwrap())
            .await
            .unwrap();
        assert_eq!(weights.as_slice(), [1, 1, 1]);
        let specs = specs_from_ips(&ips, &weights);
        assert_eq!(specs.len(), 3);
        assert_eq!(
            specs.iter().map(|s| s.weight).collect::<Vec<_>>(),
            [1, 1, 1]
        );
        assert_eq!(specs_from_ips(&ips, &[]), specs);
        assert_eq!(specs_from_ips(&ips, &[7])[1..], specs[1..]);
    }
}
