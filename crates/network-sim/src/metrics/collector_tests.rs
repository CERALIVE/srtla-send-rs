#[cfg(unix)]
use std::io::{BufRead, BufReader};
#[cfg(unix)]
use std::net::UdpSocket;
use std::num::NonZeroU64;
#[cfg(unix)]
use std::process::{Command, Stdio};

use super::MetricError;
#[cfg(unix)]
use super::Window;
use super::control::parse_reply;
use super::cpu::{CpuSample, ProcText};
#[cfg(target_os = "linux")]
use super::link_counters;
#[cfg(unix)]
use super::sink::{PROGRAM, SinkSeries};
use super::stats_file::StatsFileSample;

#[test]
fn control_counters_delta_and_effective_config_are_preserved() {
    // Given: process-owned feature/tuning payloads from a future candidate.
    let start = parse_reply(r#"{"switch_count":10,"nak_count":20,"cooldown_hold_count":1}"#)
        .unwrap()
        .unwrap();
    let end = parse_reply(r#"{"switch_count":15,"nak_count":29,"cooldown_hold_count":3,"adaptive_features":{"probe":true},"adaptive_tuning":{"new_knob":0.7}}"#).unwrap().unwrap();
    // When / Then: cumulative deltas exclude warm-up and configuration is not invented.
    let delta = end.delta(&start).unwrap();
    assert_eq!(delta.switch_count, 5);
    assert_eq!(delta.nak_count, 9);
    assert!(start.effective_config().is_none());
    assert_eq!(
        serde_json::to_value(end.effective_config().unwrap()).unwrap()["adaptive_tuning"]
            ["new_knob"],
        0.7
    );
    assert!(matches!(
        start.delta(&end),
        Err(MetricError::CounterReset(_))
    ));
}

#[test]
fn unsupported_control_is_distinct_from_malformed_supported_reply() {
    // Given / When / Then: optional capability does not mask a broken metrics reply.
    assert_eq!(parse_reply("unknown command: metrics").unwrap(), None);
    assert_eq!(parse_reply("").unwrap(), None);
    assert!(
        parse_reply(r#"{"switch_count":"broken","nak_count":1,"cooldown_hold_count":1}"#).is_err()
    );
    assert_eq!(parse_reply(r#"{"error":"unsupported"}"#).unwrap(), None);
}

#[test]
fn telemetry_fields_are_optional_only_when_the_producer_omits_them() {
    // Given: a legacy producer's committed fixture, then an additive future record.
    let legacy = include_str!("../../../../tests/fixtures/telemetry-legacy-producer.json");
    // When / Then: signed window/in-flight and absent health remain faithful.
    let sample = StatsFileSample::parse(1000, legacy).unwrap();
    assert!(sample.document.connections[0].health.is_none());
    let mapped = StatsFileSample::parse(2000, r#"{"schema_version":1,"last_updated_ms":123,"connections":[{"conn_id":"0","weight_percent":70,"window":-1,"in_flight":2,"health":"rejoining","priority":1.1,"link_id":"modem"}]}"#).unwrap();
    assert_eq!(
        mapped.document.connections[0].health.as_deref(),
        Some("rejoining")
    );
    assert_eq!(mapped.document.connections[0].window, -1);
    assert_eq!(mapped.document.connections[0].priority, Some(1.1));
    assert!(
        StatsFileSample::parse(
            0,
            r#"{"schema_version":1,"last_updated_ms":0,"connections":[{"conn_id":"0"}]}"#
        )
        .is_err()
    );
}

fn proc_stat(ticks: u64) -> String {
    let mut fields = vec!["0".to_owned(); 22];
    fields[0] = "S".into();
    fields[11] = ticks.to_string();
    fields[12] = "3".into();
    fields[19] = "12345".into();
    format!("42 (name with ) parentheses) {}", fields.join(" "))
}

#[test]
fn cpu_stat_uses_utime_stime_not_comm_word_positions() {
    // Given: process names can contain spaces and right parentheses.
    let before = CpuSample::parse(
        0,
        ProcText {
            stat: &proc_stat(7),
            status: "VmHWM:\t4096 kB\n",
            loadavg: "0.50 0.4 0.3 1/2 3",
        },
    )
    .unwrap();
    let after = CpuSample::parse(
        1000,
        ProcText {
            stat: &proc_stat(17),
            status: "VmHWM:\t8192 kB\n",
            loadavg: "0.60 0.4 0.3 1/2 3",
        },
    )
    .unwrap();
    // When / Then: 10 ticks at 100 Hz is 100 ms; RSS uses the kernel high-water mark.
    assert_eq!(
        after
            .delta(&before, NonZeroU64::new(100).unwrap())
            .unwrap()
            .cpu_ms,
        100.0
    );
    assert_eq!(after.peak_rss_kb, 8192);
    assert_eq!(after.loadavg_1m, 0.6);
}

#[test]
#[cfg(target_os = "linux")]
fn loopback_counters_and_self_cpu_are_readable_without_privileges() {
    // Given / When: real Linux proc/sys files, no namespaces or sudo.
    let link = link_counters::sample(None, "lo", 1000).unwrap();
    let cpu = CpuSample::sample(std::process::id(), 1000).unwrap();
    // Then: correct interface identity and actual process identity are captured.
    assert_eq!(link.iface, "lo");
    assert!(link.ifindex > 0);
    assert_eq!(cpu.pid, std::process::id());
    assert!(link_counters::sample(None, "../escape", 0).is_err());
}

#[test]
#[cfg(unix)]
fn sink_process_counts_udp_and_emits_zero_traffic_buckets() {
    // Given: the actual namespace sink program, run unprivileged on an ephemeral port.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sink.csv");
    let mut child = Command::new("python3")
        .args(["-u", "-c", PROGRAM, "0"])
        .arg(&path)
        .arg("5")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut ready = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut ready)
        .unwrap();
    let port = ready.split(',').nth(1).unwrap().parse::<u16>().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    // When: send one payload after readiness, wait for bounded self-termination.
    socket
        .send_to(&[1; 1316], (std::net::Ipv4Addr::LOCALHOST, port))
        .unwrap();
    assert!(child.wait().unwrap().success());
    let series = SinkSeries::parse(&std::fs::read_to_string(path).unwrap(), 0).unwrap();
    // Then: bytes, packets, zero buckets and bits/s all reflect actual sink delivery.
    assert_eq!(series.buckets().iter().map(|b| b.pkts).sum::<u64>(), 1);
    assert_eq!(series.buckets().iter().filter(|b| b.bytes == 0).count(), 4);
    assert_eq!(
        series
            .useful_goodput_bps(Window::new(0, 500).unwrap())
            .unwrap(),
        21056.0
    );
}

#[test]
#[cfg(unix)]
fn control_query_uses_the_real_unix_socket_protocol() {
    use std::io::Write;
    use std::os::unix::net::UnixListener;
    use std::time::Duration;
    // Given: a real socket implementing the sender's metrics line protocol.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("control.sock");
    let listener = UnixListener::bind(&path).unwrap();
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut command = String::new();
            BufReader::new(&stream).read_line(&mut command).unwrap();
            assert_eq!(command, "metrics\n");
            writeln!(
                stream,
                "{{\"switch_count\":3,\"nak_count\":4,\"cooldown_hold_count\":5}}"
            )
            .unwrap();
        });
        // When / Then: the collector issues the command and reads the peer's counters.
        assert_eq!(
            super::control::query(&path, Duration::from_secs(2))
                .unwrap()
                .unwrap()
                .nak_count,
            4
        );
    });
}
