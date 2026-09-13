use super::*;
use crate::impairment::ImpairmentConfig;

#[test]
fn blackhole_chain_commands_match_spike() {
    // Given/When: the exact root and filter command generator.
    let commands = qdisc::installation_commands("eth0");
    // Then: all sixteen priomap zeros and the proven IP-length mask/offset stay frozen.
    assert_eq!(
        commands[0].join(" "),
        "qdisc add dev eth0 root handle 1: prio bands 2 priomap 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0"
    );
    assert_eq!(
        commands[1].join(" "),
        "qdisc add dev eth0 parent 1:2 handle 20: netem loss 100%"
    );
    assert_eq!(
        qdisc::filter_command("eth0", true).join(" "),
        "filter add dev eth0 parent 1: protocol ip prio 10 u32 match u16 0x0400 0xfc00 at 2 \
         flowid 1:2"
    );
    assert_eq!(
        qdisc::filter_command("eth0", false).join(" "),
        "filter del dev eth0 parent 1: protocol ip prio 10"
    );
}

#[test]
fn apply_replaces_only_band_one() {
    // Given: an updated rate-limited impairment while the classifier is installed.
    let config = ImpairmentConfig {
        rate_kbit: Some(20_000),
        tbf_shaping: true,
        delay_ms: Some(60),
        jitter_ms: Some(0),
        loss_percent: Some(0.0),
        queue_limit: Some(1000),
        ..Default::default()
    };
    // When: generating an update.
    let commands = qdisc::apply_commands("eth0", &config).unwrap();
    // Then: neither root nor band two nor the filter is touched.
    assert_eq!(
        commands[0].join(" "),
        "qdisc replace dev eth0 parent 1:1 handle 10: tbf rate 20000kbit burst 250000 latency 1s"
    );
    assert_eq!(
        commands[1].join(" "),
        "qdisc replace dev eth0 parent 10:1 handle 11: netem delay 60ms 0ms loss 0% limit 1000"
    );
}

#[test]
fn tbf_latency_and_limit_are_emitted() {
    // Given: explicit byte-queue latency and packet queue limit plus a distribution.
    let config = ImpairmentConfig {
        rate_kbit: Some(10000),
        tbf_shaping: true,
        tbf_latency_ms: Some(70),
        queue_limit: Some(42),
        delay_ms: Some(20),
        jitter_ms: Some(5),
        delay_distribution: Some(crate::impairment::DelayDistribution::Normal),
        ..Default::default()
    };
    // When: generating the band-one chain.
    let commands = qdisc::apply_commands("eth0", &config).unwrap();
    // Then: TBF latency and netem packet limit are separate, correctly ordered parameters.
    assert_eq!(
        commands[0].join(" "),
        "qdisc replace dev eth0 parent 1:1 handle 10: tbf rate 10000kbit burst 125000 latency 70ms"
    );
    assert_eq!(
        commands[1].join(" "),
        "qdisc replace dev eth0 parent 10:1 handle 11: netem delay 20ms 5ms distribution normal \
         limit 42"
    );
}
