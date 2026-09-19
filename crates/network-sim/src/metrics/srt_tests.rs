use super::srt_stats::SrtStats;
use super::{MetricError, Window};

#[test]
fn native_sink_captures_parse_without_schema_changes() {
    // Given: unedited loopback captures from f2297192/164d51bb, 2500 x 1316B sent.
    let captures = [
        (
            include_str!("fixtures/srt-sink-min-prod.csv"),
            2046,
            2_782_560,
        ),
        (
            include_str!("fixtures/srt-sink-min-next.csv"),
            2047,
            2_783_920,
        ),
    ];
    for (csv, packets, bytes) in captures {
        // When: use the existing explicit interval-mode parser, including its clock offset.
        let stats = SrtStats::parse_with_semantics(
            csv,
            -1000,
            super::srt_stats::CaptureSemantics::Interval,
        )
        .unwrap();
        // Then: native counters include buffered wire packets, not just application reads.
        assert_eq!(stats.rows.len(), 2);
        let row = &stats.rows[1];
        assert_eq!(
            (row.pkt_recv_total, row.pkt_recv_unique),
            (packets, packets)
        );
        assert_eq!(row.byte_recv, bytes);
        assert_eq!(
            (
                row.pkt_loss_total,
                row.pkt_drop_total,
                row.pkt_retrans_total,
                row.pkt_belated
            ),
            (0, 0, 0, 0)
        );
        assert_eq!(row.ms_rcv_buf, Some(44.0));
        assert_eq!(row.ms_rcv_tsbpd_delay, Some(50.0));
        assert_eq!(row.raw, csv.lines().nth(2).unwrap());
        assert_eq!(row.reorder_distance, None);
        assert_eq!(
            stats.header.iter().filter(|name| *name == "Time").count(),
            2
        );
    }
}

const CSV: &str = r#"Timepoint,Time,SocketID,Time,pktRecv,pktRecvUnique,pktRcvLoss,pktRcvDrop,pktRcvRetrans,pktRcvBelated,byteRecv,msRTT,mbpsRecvRate
stamp,90000,999,1000,100,90,4,2,5,10,10000,10,1
stamp,90001,999,2000,140,120,7,5,8,3,14000,20,2
stamp,90002,999,6000,200,180,12,8,10,7,20000,40,4
"#;

#[test]
fn csv_delta_window_across_gap() {
    // Given: unequal packet-cadence times, misleading first Time and SocketID.
    let stats = SrtStats::parse(CSV, -1000).unwrap();
    // When: edges lie between rows, in the aligned sink/event clock.
    let metrics = stats.window(Window::new(500, 5500).unwrap(), 1000).unwrap();
    // Then: last-at-or-before edges, not row count, determine the delta.
    assert_eq!(metrics.pkt_recv_total, 100);
    assert_eq!(metrics.pkt_recv_unique, 90);
    assert_eq!(metrics.pkt_drop_total, 6);
    assert_eq!(metrics.viewer_loss_ratio, 6.0 / 96.0);
    assert_eq!(metrics.reorder_distance_max, None);
    assert_eq!(metrics.ms_rtt_median, Some(30.0));
    assert_eq!(metrics.mbps_recv_rate_mean, Some(3.0));
    assert_eq!(stats.rows[0].raw, CSV.lines().nth(1).unwrap());
}

#[test]
fn belated_is_summed_not_differenced() {
    // Given: belated interval counters fall then rise.
    let stats = SrtStats::parse(CSV, 0).unwrap();
    // When: the starting row is excluded from interval aggregation.
    let metrics = stats.window(Window::new(1000, 6000).unwrap(), 0).unwrap();
    // Then: 3 + 7, not 7 - 10.
    assert_eq!(metrics.pkt_belated_sum, 10);
}

#[test]
fn missing_drop_column_is_typed() {
    // Given: the mandatory drop field is absent.
    let csv = CSV.replace("pktRcvDrop", "other");
    // When / Then: expose the real wire name in the typed error.
    assert!(
        matches!(SrtStats::parse(&csv, 0), Err(MetricError::MissingColumn(name)) if name == "pktRcvDrop")
    );
}

#[test]
fn cumulative_reset_is_not_silently_a_zero_loss_window() {
    // Given: an interval-mode capture or restarted socket resets counters.
    let csv = CSV.replace("200,180,12,8,10", "20,18,1,0,1");
    let stats = SrtStats::parse(&csv, 0).unwrap();
    // When / Then: never saturate a reset into a healthy benchmark.
    assert!(matches!(
        stats.window(Window::new(1000, 6000).unwrap(), 0),
        Err(MetricError::CounterReset(_))
    ));
}

#[test]
fn empty_window_marks_no_traffic_only_when_sink_is_empty() {
    // Given: both window edges select the same CSV row.
    let stats = SrtStats::parse(CSV, 0).unwrap();
    let window = Window::new(2100, 5900).unwrap();
    // When / Then: absence is not inferred solely from the CSV.
    assert!(stats.window(window, 0).unwrap().no_traffic);
    assert!(!stats.window(window, 1316).unwrap().no_traffic);
}

#[test]
fn explicit_interval_capture_is_normalized_without_rewriting_raw_rows() {
    // Given: Todo 8's real non-fullstats capture semantics, explicitly selected.
    let csv = CSV.replace("200,180,12,8,10", "20,18,1,0,1");
    // When: normalize interval packet counters before applying the frozen delta window.
    let stats =
        SrtStats::parse_with_semantics(&csv, 0, super::srt_stats::CaptureSemantics::Interval)
            .unwrap();
    let window = stats.window(Window::new(1000, 6000).unwrap(), 1).unwrap();
    // Then: packet counts are interval sums, belated is unchanged, raw evidence is retained.
    assert_eq!(window.pkt_recv_total, 160);
    assert_eq!(window.pkt_recv_unique, 138);
    assert_eq!(window.pkt_drop_total, 5);
    assert_eq!(window.pkt_belated_sum, 10);
    assert_eq!(stats.rows[2].raw, csv.lines().nth(3).unwrap());
}

#[test]
fn reconnected_interval_capture_keeps_outage_and_counts_both_sockets() {
    // Given a reconnect after an 18-second gap, with a new socket-relative clock.
    let csv = CSV.replace("stamp,90002,999,6000", "stamp,90002,997,500");
    // When each socket is aligned with independently observed monotonic offsets.
    let stats = SrtStats::parse_intervals_with_clock(&csv, 0, |socket, time| {
        Ok(time + if socket == 997 { 19_500 } else { 0 })
    })
    .unwrap();
    // Then the gap survives, and interval counters are summed across the reconnect.
    assert_eq!(
        stats.rows.iter().map(|r| r.t_ms).collect::<Vec<_>>(),
        [1000, 2000, 20000]
    );
    let metrics = stats.window(Window::new(1000, 20000).unwrap(), 1).unwrap();
    assert_eq!(metrics.pkt_recv_unique, 300);
    assert_eq!(metrics.pkt_drop_total, 13);
    assert_eq!(metrics.viewer_loss_ratio, 13.0 / 313.0);
    assert_eq!(metrics.pkt_belated_sum, 10);
    assert_eq!(stats.rows[2].socket_id, 997);
    assert_eq!(stats.rows[2].raw, csv.lines().nth(3).unwrap());
    assert_eq!(
        stats
            .window(Window::new(2000, 19000).unwrap(), 0)
            .unwrap()
            .pkt_recv_total,
        0
    );
}

#[test]
fn mapped_interval_clock_still_rejects_backwards_time() {
    // Given a backwards timestamp within one socket, not a new clock epoch.
    let csv = CSV.replace("999,6000", "999,500");
    // When an unchanged socket clock is applied, then ordering remains mandatory.
    assert!(matches!(
        SrtStats::parse_intervals_with_clock(&csv, 0, |_, time| Ok(time)),
        Err(MetricError::InvalidField(name)) if name == "Time ordering"
    ));
}

#[test]
fn cumulative_capture_still_rejects_socket_changes() {
    // Given globally ordered times but a different cumulative counter owner.
    let csv = CSV.replace("999,6000", "997,6000");
    // When windowed, then the explicit interval exception does not hide a cumulative reset.
    assert!(matches!(
        SrtStats::parse(&csv, 0).unwrap().window(Window::new(1000, 6000).unwrap(), 1),
        Err(MetricError::CounterReset(name)) if name == "SocketID"
    ));
}

#[test]
fn metrics_v2_excludes_warmup_and_preserves_interval_belated_delta() {
    // Given synthetic rows with a depleted warmup buffer and known post-settle gauges.
    let csv = CSV
        .lines()
        .zip([
            "msRcvBuf,msRcvTsbPdDelay,byteAvailRcvBuf",
            "0,2000,99",
            "500,2000,80",
            "400,2000,70",
        ])
        .map(|(row, gauges)| format!("{row},{gauges}\n"))
        .collect::<String>();
    // When measuring strictly after the settle boundary.
    let window = SrtStats::parse(&csv, 0)
        .unwrap()
        .window(Window::new(1000, 6000).unwrap(), 1)
        .unwrap();
    // Then only post-settle samples determine the floor and interval-count sum.
    assert_eq!(window.ms_rcv_buf_min, Some(400.0));
    assert_eq!(window.ms_rcv_tsbpd_delay, Some(2000.0));
    assert_eq!(window.pkt_belated_delta, 10);
}

#[test]
fn metrics_v2_missing_gauges_remain_unknown() {
    // Given an older capture without buffer columns.
    let stats = SrtStats::parse(CSV, 0).unwrap();
    // When deriving the window, then missing is not converted to a zero or passing floor.
    let window = stats.window(Window::new(1000, 6000).unwrap(), 1).unwrap();
    assert_eq!(window.ms_rcv_buf_min, None);
    assert_eq!(window.ms_rcv_tsbpd_delay, None);
}

#[test]
fn metrics_v2_rejects_nonfinite_buffer_gauges() {
    // Given a malformed new gauge on otherwise valid rows.
    let csv = CSV
        .lines()
        .enumerate()
        .map(|(i, row)| format!("{row},{}\n", if i == 0 { "msRcvBuf" } else { "NaN" }))
        .collect::<String>();
    // When parsing, then corrupt input is rejected at the boundary.
    assert!(
        matches!(SrtStats::parse(&csv, 0), Err(MetricError::InvalidField(name)) if name == "msRcvBuf")
    );
}
