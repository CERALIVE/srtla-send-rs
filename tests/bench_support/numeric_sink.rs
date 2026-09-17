use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use network_sim::NamespaceProcess;
use network_sim::bond::BondTopology;

use super::record::{Request, binary_hash};
use super::stack::utf8;

pub fn arguments(request: &Request) -> Result<Vec<String>> {
    let cell = &request.manifest.cells[request.work.cell];
    ensure!(
        request.manifest.caller_bin.is_some(),
        "numeric sink requires a separate caller_bin"
    );
    ensure!(
        matches!(
            request.receiver_spec.lineage.as_str(),
            "irlserver-prod" | "irlserver-next" | "belabox"
        ),
        "numeric sink option IDs require an onsmith lineage"
    );
    let mut options = std::collections::BTreeMap::from([(
        42,
        crate::manifest::preset(&cell.srt_profile)?.lossmaxttl,
    )]);
    for pair in request
        .receiver_spec
        .listener_uri_extra
        .trim_start_matches('&')
        .split('&')
        .filter(|s| !s.is_empty())
    {
        let (name, value) = pair.split_once('=').context("numeric sink option")?;
        let id = match name {
            "srtlapatches" => 120,
            "lossmaxttl" => 42,
            _ => bail!("unsupported numeric sink option {name}"),
        };
        options.insert(id, value.parse()?);
    }
    let mut args = vec![
        utf8(&request.srt_binary)?.into(),
        "--port".into(),
        cell.port.to_string(),
        "--latency".into(),
        crate::manifest::preset(&cell.srt_profile)?
            .latency_ms
            .to_string(),
        "--statsout".into(),
        utf8(&request.result.record.raw.stats_csv_path)?.into(),
        "--out".into(),
        "/dev/stdout".into(),
    ];
    for (id, value) in options {
        args.extend(["--sockopt".into(), format!("{id}={value}")]);
    }
    Ok(args)
}

pub fn listener(request: &Request, topo: &BondTopology) -> Result<NamespaceProcess> {
    let cell = &request.manifest.cells[request.work.cell];
    if request.srt_binary.file_name().and_then(|n| n.to_str()) != Some("srt-sink-min") {
        let args = request.receiver_spec.listener_argv(
            crate::manifest::preset(&cell.srt_profile)?,
            cell.port,
            Some(&request.result.record.raw.stats_csv_path),
        )?;
        return NamespaceProcess::spawn_process_only(
            &topo.receiver_ns,
            utf8(&request.srt_binary)?,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        );
    }
    let args = arguments(request)?;
    let mut launch = vec!["-u", "-c", BRIDGE];
    launch.extend(args.iter().map(String::as_str));
    NamespaceProcess::spawn_process_only(&topo.receiver_ns, "python3", &launch)
}

pub fn caller(request: &Request) -> Result<&Path> {
    let path = request
        .manifest
        .caller_bin
        .as_deref()
        .unwrap_or(&request.srt_binary);
    crate::checkpoint::atomic(
        &request.artifacts.join("caller-identity.json"),
        &serde_json::json!({"path": path, "sha256": binary_hash(path)?}),
    )?;
    Ok(path)
}

// Read only application-delivered bytes from the native sink; never infer goodput
// from its SRT counters. A pipe prevents disk I/O from becoming the measured sink.
const BRIDGE: &str = r#"
import signal, socket, subprocess, sys
with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as udp:
    with subprocess.Popen(sys.argv[1:], stdout=subprocess.PIPE) as child:
        def stop(signum, frame):
            child.terminate()
        signal.signal(signal.SIGTERM, stop)
        signal.signal(signal.SIGINT, stop)
        while True:
            chunk = child.stdout.read1(8192)
            if not chunk:
                break
            udp.sendto(chunk, ('127.0.0.1', 9999))
        sys.exit(child.wait())
"#;
