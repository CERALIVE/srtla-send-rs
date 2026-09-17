use crate::config::DynamicConfig;
use crate::protocol::srt_handshake::HsrspInfo;
use crate::protocol::{SRT_OPT_NAKREPORT, SRT_OPT_REXMITFLG};
use crate::stats::SharedStats;

#[test]
fn receiver_handshake_unknown_is_omitted_and_policy_defaults_to_nak_on() {
    let stats = SharedStats::new();
    let receiver = stats.receiver_handshake();
    assert_eq!(receiver.receiver_nak_report, None);
    assert_eq!(receiver.receiver_srt_version, None);
    assert_eq!(receiver.receiver_rexmit_flag, None);
    assert!(receiver.nak_report_enabled());
    assert_eq!(
        serde_json::to_value(receiver).unwrap(),
        serde_json::json!({})
    );
}

#[test]
fn receiver_handshake_updates_all_observations_on_encoder_reconnect() {
    let stats = SharedStats::new();
    stats.set_receiver_handshake(HsrspInfo {
        srt_version: 0x010505,
        flags: SRT_OPT_NAKREPORT | SRT_OPT_REXMITFLG,
        tsbpd_delay_ms: 2000,
    });
    let consumer = stats.clone();
    stats.set_receiver_handshake(HsrspInfo {
        srt_version: 0x010506,
        flags: 0,
        tsbpd_delay_ms: 500,
    });
    let receiver = consumer.receiver_handshake();
    assert_eq!(receiver.receiver_nak_report, Some(false));
    assert_eq!(receiver.receiver_rexmit_flag, Some(false));
    assert_eq!(receiver.receiver_srt_version.as_deref(), Some("1.5.6"));
    assert!(!receiver.nak_report_enabled());
    assert_eq!(consumer.negotiated_latency_ms(), Some(500));
}

#[tokio::test]
async fn receiver_handshake_survives_pool_addition_reorder_and_replacement() {
    let stats = SharedStats::new();
    stats.set_receiver_handshake(HsrspInfo {
        srt_version: 0x010506,
        flags: SRT_OPT_REXMITFLG,
        tsbpd_delay_ms: 2000,
    });
    let mut conns = crate::test_helpers::create_test_connections(2).await;
    let config = DynamicConfig::new().snapshot();
    stats.update(&conns[..1], &config);
    stats.update(&conns, &config);
    conns.reverse();
    stats.update(&conns, &config);
    let replacement = crate::test_helpers::create_test_connection().await;
    stats.update(&[replacement], &config);
    let snapshot = stats.get();
    assert_eq!(snapshot.receiver.receiver_nak_report, Some(false));
    assert_eq!(snapshot.receiver.receiver_rexmit_flag, Some(true));
    assert_eq!(
        snapshot.receiver.receiver_srt_version.as_deref(),
        Some("1.5.6")
    );
}

#[test]
fn receiver_handshake_get_status_omits_unknown_and_preserves_false() {
    let stats = SharedStats::new();
    let config = DynamicConfig::new();
    let request = r#"{"jsonrpc":"2.0","id":7,"method":"get-status"}"#;
    let unknown: serde_json::Value =
        serde_json::from_str(&crate::jsonrpc::dispatch_jsonrpc(request, &config, &stats)).unwrap();
    assert_eq!(unknown["result"]["receiver"], serde_json::json!({}));
    stats.set_receiver_handshake(HsrspInfo {
        srt_version: 0x010506,
        flags: 0,
        tsbpd_delay_ms: 2000,
    });
    let observed: serde_json::Value =
        serde_json::from_str(&crate::jsonrpc::dispatch_jsonrpc(request, &config, &stats)).unwrap();
    assert_eq!(observed["result"]["receiver"]["nak_report"], false);
    assert_eq!(observed["result"]["receiver"]["srt_version"], "1.5.6");
}

#[test]
fn receiver_handshake_telemetry_omits_unknown_and_preserves_both_bools() {
    let stats = SharedStats::new();
    let unknown: serde_json::Value = serde_json::from_str(
        &crate::telemetry_doc::build_telemetry_json_from_stats(1, &stats.get()),
    )
    .unwrap();
    assert!(unknown.get("receiver_nak_report").is_none());
    for flags in [0, SRT_OPT_NAKREPORT] {
        stats.set_receiver_handshake(HsrspInfo {
            srt_version: 0x010506,
            flags,
            tsbpd_delay_ms: 2000,
        });
        let json = crate::telemetry_doc::build_telemetry_json_from_stats(1, &stats.get());
        let observed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(observed["receiver_nak_report"], flags != 0);
        assert_eq!(observed["schema_version"], 1);
    }
}

#[test]
fn receiver_handshake_status_text_distinguishes_unknown_off_and_on() {
    let stats = SharedStats::new();
    assert_eq!(
        stats.receiver_handshake().to_string(),
        "receiver: nak_report=unknown srt=unknown"
    );
    for (flags, text) in [(0, "off"), (SRT_OPT_NAKREPORT, "on")] {
        stats.set_receiver_handshake(HsrspInfo {
            srt_version: 0x010506,
            flags,
            tsbpd_delay_ms: 2000,
        });
        assert_eq!(
            stats.receiver_handshake().to_string(),
            format!("receiver: nak_report={text} srt=1.5.6")
        );
    }
}
