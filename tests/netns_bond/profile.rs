use std::time::Duration;

use network_sim::bond::{BondTopology, MappingMode};
use network_sim::profile::{
    Action, BondRuntime, LinkProfile, ProcessEndpoints, Profile, Scheduler, TimedEvent,
};
use network_sim::{ImpairmentConfig, NamespaceProcess, wait_for_udp_listener};

use super::{NAT, stack};

fn event(at: u64, link: Option<usize>, action: Action) -> TimedEvent {
    TimedEvent {
        horizon: Duration::ZERO,
        graded: false,
        ..TimedEvent::new(Duration::from_secs(at), link, action)
    }
}

fn base(delay: u32, shaped: bool) -> ImpairmentConfig {
    ImpairmentConfig {
        delay_ms: Some(delay),
        jitter_ms: Some(0),
        loss_percent: Some(0.0),
        rate_kbit: shaped.then_some(10000),
        tbf_shaping: shaped,
        queue_limit: Some(1000),
        ..Default::default()
    }
}

fn profile(config: ImpairmentConfig) -> Profile {
    Profile {
        links: vec![
            LinkProfile {
                base: config,
                carrier: network_sim::bond::CarrierMode::Nat
            };
            2
        ],
        events: vec![],
        duration: Duration::from_secs(10),
    }
}

fn probe(topo: &BondTopology) -> Vec<(usize, f64)> {
    let script = r#"import socket,sys,time
with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as sock:
    sock.bind((sys.argv[1],0))
    sock.settimeout(0.3)
    for size in (38,1316):
        started=time.monotonic()
        sock.sendto(bytes(size),('10.99.0.1',40100))
        try:
            data,_=sock.recvfrom(2048)
            print(len(data),(time.monotonic()-started)*1000,flush=True)
        except TimeoutError:
            print(size,-1,flush=True)
"#;
    let output = topo
        .sender_ns
        .exec_checked("python3", &["-c", script, topo.sender_ip(1)])
        .unwrap();
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let (size, elapsed) = line.split_once(' ').unwrap();
            (size.parse().unwrap(), elapsed.parse().unwrap())
        })
        .collect()
}

#[test]
fn profile_blackhole_survives_impairment_update_and_restores_data() {
    if !stack::available() {
        return;
    }
    for shaped in [false, true] {
        // Given: a real NAT link, echo server, and the composed plain/TBF layout.
        let topo = BondTopology::new("profilebh", &[NAT, NAT], MappingMode::None).unwrap();
        let mut stack = stack::Stack::start(topo).unwrap();
        let script = r#"import socket
with socket.socket(socket.AF_INET,socket.SOCK_DGRAM) as sock:
    sock.bind(('0.0.0.0',40100))
    while True:
        data,peer=sock.recvfrom(2048)
        sock.sendto(data,peer)
"#;
        let _echo =
            NamespaceProcess::spawn(&stack.topo.receiver_ns, "python3", &["-c", script]).unwrap();
        wait_for_udp_listener(&stack.topo.receiver_ns, 40100, Duration::from_secs(3)).unwrap();
        let mut offered = |_| Ok(());
        let endpoints = ProcessEndpoints {
            sender: &mut stack.sender,
            receiver: &mut stack.processes[1],
            offered_rate: &mut offered,
        };
        let mut runtime =
            BondRuntime::new(&profile(base(20, shaped)), &stack.topo, endpoints).unwrap();
        let baseline = probe(&stack.topo);
        assert!(baseline.iter().all(|(_, ms)| *ms >= 15.0));
        // When: blackholing DATA, then updating delay without removing the filter.
        runtime
            .apply(&event(0, Some(1), Action::DataBlackhole { on: true }))
            .unwrap();
        let blocked = probe(&stack.topo);
        runtime
            .apply(&event(0, Some(1), Action::SetImpairment(base(60, shaped))))
            .unwrap();
        let updated = probe(&stack.topo);
        // Then: keepalives pass at the new delay, DATA still drops, and filter removal restores it.
        assert!(blocked[0].1 >= 15.0 && blocked[0].1 < 150.0);
        assert_eq!(blocked[1], (1316, -1.0));
        assert!(updated[0].1 >= 55.0 && updated[0].1 < 200.0);
        assert_eq!(updated[1], (1316, -1.0));
        runtime
            .apply(&event(0, Some(1), Action::SetImpairment(base(60, !shaped))))
            .unwrap();
        let switched = probe(&stack.topo);
        assert!(switched[0].1 >= 55.0 && switched[0].1 < 200.0);
        assert_eq!(switched[1], (1316, -1.0));
        runtime
            .apply(&event(0, Some(1), Action::DataBlackhole { on: false }))
            .unwrap();
        let restored = probe(&stack.topo);
        assert!(restored.iter().all(|(_, ms)| *ms >= 55.0 && *ms < 200.0));
        eprintln!(
            "shaped={shaped}: baseline={baseline:?}, blocked={blocked:?}, updated={updated:?}, \
             switched={switched:?}, restored={restored:?}"
        );
    }
}

#[test]
fn profile_ten_seconds_keeps_echoing_while_data_stalls() {
    if !stack::available() {
        return;
    }
    // Given: a registered two-carrier bond carrying sustained DATA.
    let topo = BondTopology::new("profile10", &[NAT, NAT], MappingMode::None).unwrap();
    let mut stack = stack::Stack::start(topo).unwrap();
    stack.registered(2).unwrap();
    stack.start_traffic().unwrap();
    let mut p = profile(base(25, true));
    p.events = vec![
        event(4, Some(1), Action::DataBlackhole { on: true }),
        event(8, Some(1), Action::DataBlackhole { on: false }),
    ];
    let mut offered = |_| Ok(());
    let endpoints = ProcessEndpoints {
        sender: &mut stack.sender,
        receiver: &mut stack.processes[1],
        offered_rate: &mut offered,
    };
    let mut runtime = BondRuntime::new(&p, &stack.topo, endpoints).unwrap();
    let mut scheduler = Scheduler::new(&p).unwrap();
    let mut before = 0;
    let mut keepalives = 0;
    let label = format!("{}: RTT from keepalive", stack.topo.sender_ip(1));
    // When: applying the timed blackhole for four seconds.
    scheduler
        .run(&mut |event| {
            if event.at == Duration::from_secs(8) {
                let delta = stack.topo.tx_bytes(1)? - before;
                let after = runtime
                    .processes
                    .sender
                    .log_snapshot()
                    .iter()
                    .filter(|line| line.contains(&label))
                    .count();
                assert!(delta < 20_000, "DATA did not stall: {delta}");
                assert!(after > keepalives, "keepalive RTT stopped during blackhole");
                eprintln!(
                    "blackhole 4..8s tx_delta={delta}, keepalive RTT lines {keepalives}->{after}"
                );
            }
            runtime.apply(event)?;
            before = stack.topo.tx_bytes(1)?;
            keepalives = runtime
                .processes
                .sender
                .log_snapshot()
                .iter()
                .filter(|line| line.contains(&label))
                .count();
            Ok(())
        })
        .unwrap();
    // Then: DATA resumes during the final two-second observation tail.
    assert!(stack.topo.tx_bytes(1).unwrap() - before > 100_000);
    assert_eq!(scheduler.log().entries.len(), 2);
    eprintln!("event log: {:?}", scheduler.log());
}

#[test]
fn profile_receiver_restart_preserves_listener_and_sink_pids() {
    if !stack::available() {
        return;
    }
    // Given: receiver and listener share a namespace with a live UDP sink.
    let topo = BondTopology::new("profilerest", &[NAT, NAT], MappingMode::None).unwrap();
    let mut stack = stack::Stack::start(topo).unwrap();
    let mut sink = NamespaceProcess::spawn(
        &stack.topo.receiver_ns,
        "python3",
        &[
            "-c",
            "import socket; s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM); \
             s.bind(('127.0.0.1',9999)); s.recvfrom(65536)",
        ],
    )
    .unwrap();
    let listener_pid = stack.processes[0].pid().unwrap();
    let sink_pid = sink.pid().unwrap();
    let old_receiver = stack.processes[1].pid().unwrap();
    let mut offered = |_| Ok(());
    let endpoints = ProcessEndpoints {
        sender: &mut stack.sender,
        receiver: &mut stack.processes[1],
        offered_rate: &mut offered,
    };
    let mut runtime = BondRuntime::new(&profile(base(20, false)), &stack.topo, endpoints).unwrap();
    // When: the runtime dispatches ReceiverRestart.
    runtime
        .apply(&event(0, None, Action::ReceiverRestart))
        .unwrap();
    let new_receiver = runtime.processes.receiver.pid().unwrap();
    drop(runtime);
    // Then: only the receiver changed identity; listener and sink remain alive.
    assert_ne!(old_receiver, new_receiver);
    assert_eq!(stack.processes[0].pid().unwrap(), listener_pid);
    assert_eq!(sink.pid().unwrap(), sink_pid);
    assert!(stack.processes[0].is_alive() && sink.is_alive());
    eprintln!(
        "receiver {old_receiver}->{new_receiver}; listener={listener_pid}, sink={sink_pid} \
         unchanged/alive"
    );
}
