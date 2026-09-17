use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, ensure};
use network_sim::bond::{BondTopology, CarrierMode, LinkSpec, MappingMode};
use network_sim::harness::{ReceiverSpec, SrtProfile};
use network_sim::metrics::link_counters;
use network_sim::profile::{
    Action, BondRuntime, LinkProfile, ProcessEndpoints, Profile, Scheduler, TimedEvent,
};
use network_sim::{
    ImpairmentConfig, NamespaceProcess, wait_for_registered_uplinks, wait_for_udp_listener,
};

pub fn run(sender_binary: &str) -> Result<()> {
    for carrier in [CarrierMode::Direct, CarrierMode::Nat] {
        if carrier == CarrierMode::Nat && network_sim::check_binary("iptables").is_none() {
            eprintln!("Skipping NAT AddLink: iptables unavailable");
            continue;
        }
        // Given: one registered, carrying link and a second link absent from the topology.
        let link = LinkProfile {
            base: ImpairmentConfig {
                delay_ms: Some(10),
                rate_kbit: Some(5000),
                ..Default::default()
            },
            carrier,
        };
        let profile = Profile {
            links: vec![link.clone()],
            events: vec![TimedEvent {
                at: Duration::from_secs(2),
                link: None,
                action: Action::AddLink(link),
                horizon: Duration::from_secs(16),
                graded: false,
            }],
            duration: Duration::from_secs(18),
        };
        let mut topology = BondTopology::new(
            "addlink",
            &[LinkSpec {
                carrier,
                shared_ip_with: None,
            }],
            MappingMode::None,
        )?;
        match carrier {
            CarrierMode::Nat => topology.set_priority_sidecar(serde_json::from_str(
                r#"[{"link":0,"priority":-0.2},{"link":1,"priority":0.2}]"#,
            )?)?,
            CarrierMode::Direct => {}
        }
        let receiver_spec = ReceiverSpec::from_env()?.resolved()?;
        let args = receiver_spec.listener_argv(SrtProfile::LEGACY_DEFAULT, 4001, None)?;
        let _listener = NamespaceProcess::spawn_process_only(
            &topology.receiver_ns,
            receiver_spec
                .srt_live_transmit_bin
                .to_str()
                .context("SRT tool path")?,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topology.receiver_ns, 4001, Duration::from_secs(5))?;
        let args = receiver_spec.kind()?.argv(5000, "127.0.0.1", 4001);
        let mut receiver = NamespaceProcess::spawn_process_only(
            &topology.receiver_ns,
            receiver_spec
                .srtla_rec_bin
                .to_str()
                .context("receiver path")?,
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        wait_for_udp_listener(&topology.receiver_ns, 5000, Duration::from_secs(5))?;
        let args = topology.sender_args((5555, 5000), &["--mode", "classic"])?;
        let mut launch = vec!["RUST_LOG=info", sender_binary];
        launch.extend(args.iter().map(String::as_str));
        let mut sender = NamespaceProcess::spawn_process_only(&topology.sender_ns, "env", &launch)?;
        wait_for_registered_uplinks(&sender, 1, Duration::from_secs(15))?;
        let _caller = NamespaceProcess::spawn_process_only(
            &topology.sender_ns,
            receiver_spec
                .srt_live_transmit_bin
                .to_str()
                .context("SRT tool path")?,
            &["udp://:6000", "srt://127.0.0.1:5555?mode=caller"],
        )?;
        wait_for_udp_listener(&topology.sender_ns, 6000, Duration::from_secs(5))?;
        let ns = Arc::clone(&topology.sender_ns);
        let mut offered = |_| Ok(());
        let mut runtime = BondRuntime::with_link_additions(
            &profile,
            &mut topology,
            ProcessEndpoints {
                sender: &mut sender,
                receiver: &mut receiver,
                offered_rate: &mut offered,
            },
        )?;
        let initial = link_counters::sample(Some(&ns), runtime.topology().sender_iface(0), 0)?;
        let mut samples = Vec::new();
        let mut scheduler = Scheduler::new(&profile)?;
        // When: real SRT traffic continues while the scheduler creates and publishes the new link.
        std::thread::scope(|scope| -> Result<()> {
            let source = scope.spawn(|| {
                network_sim::inject_udp_stream(
                    &ns,
                    "127.0.0.1",
                    6000,
                    2000,
                    Duration::from_secs(20),
                )
            });
            let result = scheduler.run(&mut |event| {
                ensure!(
                    runtime.topology().link_count() == 1,
                    "link was created before its scheduled event"
                );
                let before =
                    link_counters::sample(Some(&ns), runtime.topology().sender_iface(0), 2000)?;
                ensure!(
                    before.tx_bytes > initial.tx_bytes + 10_000,
                    "initial link never carried traffic"
                );
                runtime.apply(event)?;
                let start = Instant::now();
                loop {
                    let t = i64::try_from(start.elapsed().as_millis())?;
                    samples.push(link_counters::sample(
                        Some(&ns),
                        runtime.topology().sender_iface(1),
                        t,
                    )?);
                    if samples.last().context("latest sample")?.tx_bytes
                        > samples[0].tx_bytes + 10_000
                    {
                        break;
                    }
                    ensure!(
                        start.elapsed() < Duration::from_secs(15),
                        "new link carried no DATA: {:?}",
                        runtime.processes.sender.log_snapshot()
                    );
                    std::thread::sleep(Duration::from_millis(200));
                }
                wait_for_registered_uplinks(runtime.processes.sender, 2, Duration::from_secs(1))?;
                Ok(())
            });
            source
                .join()
                .map_err(|_| anyhow::anyhow!("source panicked"))??;
            result
        })?;
        // Then: link-specific wire samples exceed control traffic and the live log proves registration.
        ensure!(
            samples.last().context("final sample")?.tx_packets > samples[0].tx_packets + 10,
            "no new-link packet growth"
        );
        ensure!(
            scheduler.log().entries.len() == 1,
            "addition must fire once"
        );
        eprintln!(
            "{carrier:?} AddLink at 2s: new-link samples={samples:?}\nsender log:\n{}",
            runtime.processes.sender.log_snapshot().join("\n")
        );
    }
    Ok(())
}
