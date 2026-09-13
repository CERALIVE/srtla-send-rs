use super::srt_stats::SrtStats;
use super::{MetricError, Window};

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
