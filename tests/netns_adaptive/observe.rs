use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::profile::{Action, BondRuntime, ProcessEndpoints, TimedEvent};
use network_sim::scenarios::Profile;
use serde::{Deserialize, Deserializer, Serialize};

use super::stack::{Stack, read_sink};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Snapshot {
    pub last_updated_ms: u64,
    pub connections: Vec<Link>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Link {
    pub conn_id: String,
    pub link_id: Option<String>,
    pub iface: Option<String>,
    pub health: String,
    pub weight_percent: u32,
    pub rtt_ms: u32,
    #[serde(
        default,
        deserialize_with = "present_priority",
        skip_serializing_if = "Option::is_none"
    )]
    pub priority: Option<f64>,
}

fn present_priority<'de, D: Deserializer<'de>>(d: D) -> Result<Option<f64>, D::Error> {
    f64::deserialize(d).map(Some)
}

impl Snapshot {
    pub fn link(&self, id: &str) -> &Link {
        self.connections
            .iter()
            .find(|l| l.link_id.as_deref() == Some(id))
            .expect("mapped link_id present")
    }
}

#[derive(Debug, Serialize)]
pub struct Sample {
    pub t: f64,
    pub snapshot: Snapshot,
    pub tx: Vec<u64>,
    pub keepalives: Vec<usize>,
    pub established: usize,
    pub sink_end_ms: i64,
    pub sink_bps: f64,
}

pub struct Run {
    pub samples: Vec<Sample>,
    pub events: Vec<(f64, TimedEvent)>,
    pub priorities: Vec<(f64, Snapshot)>,
    pub restart_duration: Option<Duration>,
    pub restart_completed_at: Option<f64>,
    pub registered_uplinks: usize,
}

impl Run {
    pub fn at(&self, t: f64) -> &Sample {
        self.samples
            .iter()
            .find(|s| s.t >= t)
            .expect("sample inside window")
    }
    pub fn phase(&self, start: f64, end: f64) -> Vec<u64> {
        let before = self.at(start);
        let after = self.at(end);
        after
            .tx
            .iter()
            .zip(&before.tx)
            .map(|(a, b)| a - b)
            .collect()
    }
}

pub fn measure(stack: &mut Stack, profile: &Profile, priorities: bool) -> Result<Run> {
    let events = profile.timeline.expanded_events()?;
    let mut source_rate = |bps| stack.source.rate(bps, Duration::ZERO);
    let mut runtime = BondRuntime::new(
        &profile.timeline,
        &stack.topo,
        ProcessEndpoints {
            sender: &mut stack.sender,
            receiver: &mut stack.receiver,
            offered_rate: &mut source_rate,
        },
    )?;
    let mut run = Run {
        samples: Vec::new(),
        events: Vec::new(),
        priorities: Vec::new(),
        restart_duration: None,
        restart_completed_at: None,
        registered_uplinks: stack.registered_uplinks,
    };
    let mut output = File::create(stack.directory.path().join("observations.jsonl"))?;
    let mut logs = File::create(stack.directory.path().join("sender.log"))?;
    let mut seen = HashSet::new();
    let mut keepalives = vec![0; stack.topo.link_count()];
    let mut established = 0;
    let mut next = 0;
    let mut priority_index = 0;
    let mut pending_priority = None;
    let start = Instant::now();
    loop {
        while let Some(event) = events.get(next).filter(|e| e.at <= start.elapsed()) {
            let began = Instant::now();
            runtime.apply(event)?;
            if matches!(event.action, Action::ReceiverRestart) {
                let duration = runtime
                    .receiver_restart_elapsed()
                    .context("receiver restart duration")?;
                run.restart_duration = Some(duration);
                run.restart_completed_at =
                    Some(began.duration_since(start).as_secs_f64() + duration.as_secs_f64());
                eprintln!(
                    "adaptive receiver process restart={duration:?}, including readiness={:?}",
                    began.elapsed()
                );
            }
            let t = start.elapsed().as_secs_f64();
            eprintln!("adaptive event t={t:.3} {:?}", event.action);
            run.events.push((t, event.clone()));
            next += 1;
        }
        let snapshot: Snapshot = serde_json::from_slice(&std::fs::read(&stack.stats)?)?;
        if let Some((changed_at, version)) = pending_priority
            && snapshot.last_updated_ms > version
        {
            run.priorities
                .push((start.elapsed().as_secs_f64(), snapshot.clone()));
            eprintln!(
                "adaptive priority snapshot after RPC t={changed_at:.3}, observed t={:.3} {}",
                start.elapsed().as_secs_f64(),
                serde_json::to_string(&snapshot)?
            );
            pending_priority = None;
        }
        if priorities
            && priority_index < 2
            && start.elapsed().as_secs_f64() >= [22.0, 25.0][priority_index]
        {
            ensure!(
                pending_priority.is_none(),
                "previous priority snapshot missing"
            );
            let priority = [Some(0.2), None][priority_index];
            let reply = set_priority(&stack.control, priority)?;
            eprintln!(
                "adaptive priority RPC t={:.3} {reply}",
                start.elapsed().as_secs_f64()
            );
            pending_priority = Some((start.elapsed().as_secs_f64(), snapshot.last_updated_ms));
            priority_index += 1;
        }
        for line in runtime.processes.sender.log_snapshot() {
            if seen.insert(line.clone()) {
                writeln!(logs, "{line}")?;
                if line.contains("connection established") {
                    established += 1;
                }
                if line.contains("RTT from keepalive:") {
                    for (i, count) in keepalives.iter_mut().enumerate() {
                        let label = snapshot
                            .connections
                            .iter()
                            .find(|l| l.iface.as_deref() == Some(stack.topo.sender_iface(i)))
                            .and_then(|l| l.link_id.as_ref());
                        if label.is_some_and(|id| line.contains(&format!("[{id}]"))) {
                            *count += 1;
                        }
                    }
                }
            }
        }
        let sink = read_sink(&stack.sink_path)?;
        let buckets = sink.buckets();
        let last = buckets.last().context("sink samples")?;
        let bytes: u64 = buckets.iter().rev().take(10).map(|b| b.bytes).sum();
        let tx = (0..stack.topo.link_count())
            .map(|i| stack.topo.tx_bytes(i))
            .collect::<Result<_>>()?;
        let sample = Sample {
            t: start.elapsed().as_secs_f64(),
            snapshot,
            tx,
            keepalives: keepalives.clone(),
            established,
            sink_end_ms: last.t_ms,
            sink_bps: f64::from(u32::try_from(bytes)?) * 8.0,
        };
        serde_json::to_writer(&mut output, &sample)?;
        writeln!(output)?;
        let finished = sample.t >= profile.timeline.duration.as_secs_f64();
        run.samples.push(sample);
        if finished {
            break;
        }
        // Bounded external-file polling; the sender and tc clocks are not injectable.
        std::thread::sleep(
            Duration::from_millis(100)
                .min(profile.timeline.duration.saturating_sub(start.elapsed())),
        );
    }
    Ok(run)
}

fn set_priority(path: &std::path::Path, priority: Option<f64>) -> Result<String> {
    let mut socket = UnixStream::connect(path)?;
    socket.set_read_timeout(Some(Duration::from_secs(2)))?;
    socket.set_write_timeout(Some(Duration::from_secs(2)))?;
    writeln!(
        socket,
        "{}",
        serde_json::json!({"jsonrpc":"2.0","id":28,"method":"set-link-priority","params":{"link_id":"modem-a","priority":priority}})
    )?;
    let mut reply = String::new();
    BufReader::new(socket).take(65536).read_line(&mut reply)?;
    #[derive(Deserialize)]
    struct Reply {
        id: u32,
        result: Applied,
    }
    #[derive(Deserialize)]
    struct Applied {
        applied: bool,
        link_id: String,
    }
    let parsed: Reply = serde_json::from_str(&reply)?;
    ensure!(
        parsed.id == 28 && parsed.result.applied && parsed.result.link_id == "modem-a",
        "priority reply: {reply}"
    );
    Ok(reply)
}
