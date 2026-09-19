use std::time::{Duration, Instant};

use network_sim::bond::{BondTopology, MappingMode};
use network_sim::harness::ProcessControlError;
use network_sim::profile::traffic::CrossTraffic;
use network_sim::profile::{
    Action, BondRuntime, LinkProfile, ProcessEndpoints, Profile, TimedEvent,
};
use network_sim::{ImpairmentConfig, NamespaceProcess};

use super::{NAT, mapping, stack};

#[test]
fn profile_cross_traffic_uses_the_selected_bottleneck_and_stops_only_its_processes() {
    if !stack::available() {
        return;
    }
    for python in [false, true] {
        // Given: two NAT paths, with a 1Mbit shaped target and unrelated processes.
        let topo = BondTopology::new("profilext", &[NAT, NAT], MappingMode::None).unwrap();
        topo.apply_impairment(
            1,
            &ImpairmentConfig {
                rate_kbit: Some(1000),
                tbf_shaping: true,
                queue_limit: Some(100),
                ..Default::default()
            },
        )
        .unwrap();
        let mut sender =
            NamespaceProcess::spawn_process_only(&topo.sender_ns, "sleep", &["30"]).unwrap();
        let mut receiver =
            NamespaceProcess::spawn_process_only(&topo.receiver_ns, "sleep", &["30"]).unwrap();
        let start = topo.tx_bytes(1).unwrap();
        // When: iperf or the forced Python fallback offers 10Mbit on link 1.
        let mut load = if python {
            CrossTraffic::start_python(&topo, 1, 10)
        } else {
            CrossTraffic::start(&topo, 1, 10)
        }
        .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while topo.tx_bytes(1).unwrap() - start < 100_000 {
            assert!(
                Instant::now() < deadline,
                "cross-traffic did not carry bytes"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        let begin = topo.tx_bytes(1).unwrap();
        std::thread::sleep(Duration::from_secs(1));
        let carried = topo.tx_bytes(1).unwrap() - begin;
        load.stop().unwrap();
        drop(load);
        // Then: the actual qdisc caps carriage; the other path and processes are untouched.
        assert!(
            (50_000..200_000).contains(&carried),
            "1Mbit bottleneck carried {carried} B/s"
        );
        assert!(topo.tx_bytes(0).unwrap() < 2048);
        assert!(sender.is_alive() && receiver.is_alive());
        eprintln!(
            "cross-traffic python={python}, shaped carriage={carried} B/s, other processes alive"
        );
    }
}

#[test]
fn profile_reorder_replug_route_link_and_offered_rate_dispatch() {
    if !stack::available() {
        return;
    }
    // Given: a mapped bond with stable link identities and explicit offered-load ownership.
    let topo = BondTopology::new("profilectl", &[NAT, NAT], mapping(2)).unwrap();
    let mut stack = stack::Stack::start(topo).unwrap();
    stack.registered(2).unwrap();
    let config = ImpairmentConfig {
        delay_ms: Some(25),
        ..Default::default()
    };
    let profile = Profile {
        links: vec![
            LinkProfile {
                base: config,
                carrier: NAT.carrier
            };
            2
        ],
        events: vec![],
        duration: Duration::from_secs(30),
    };
    let mut rates = Vec::new();
    let mut offered = |bps| {
        rates.push(bps);
        Ok(())
    };
    let endpoints = ProcessEndpoints {
        sender: &mut stack.sender,
        receiver: &mut stack.processes[1],
        offered_rate: &mut offered,
    };
    let mut runtime = BondRuntime::new(&profile, &stack.topo, endpoints).unwrap();
    let index_path = format!("/sys/class/net/{}/ifindex", stack.topo.sender_iface(1));
    let old_index = stack
        .topo
        .sender_ns
        .exec_checked("cat", &[&index_path])
        .unwrap()
        .stdout;
    // When: dispatching every topology/control action on the real stack.
    for (link, action) in [
        (Some(1), Action::LinkUp(false)),
        (Some(1), Action::LinkUp(true)),
        (Some(1), Action::DefaultRoute(false)),
        (Some(1), Action::DefaultRoute(true)),
        (Some(1), Action::Replug),
        (None, Action::SighupReorder(vec![1, 0])),
        (None, Action::OfferedRate { bps: 4_000_000 }),
        (Some(1), Action::CrossTraffic { mbit: 1, on: true }),
        (Some(1), Action::CrossTraffic { mbit: 1, on: false }),
    ] {
        runtime
            .apply(&TimedEvent::new(Duration::ZERO, link, action))
            .unwrap();
    }
    drop(runtime);
    // Then: physical identity changed, publication stayed coherent, source callback was invoked.
    assert_ne!(
        old_index,
        stack
            .topo
            .sender_ns
            .exec_checked("cat", &[&index_path])
            .unwrap()
            .stdout
    );
    assert_eq!(rates, [4_000_000]);
    assert_eq!(
        std::fs::read_to_string(stack.topo.ips_path()).unwrap(),
        format!("{}\n{}\n", stack.topo.sender_ip(1), stack.topo.sender_ip(0))
    );
    let sidecar: serde_json::Value =
        serde_json::from_slice(&std::fs::read(stack.topo.sidecar_path().unwrap()).unwrap())
            .unwrap();
    assert_eq!(sidecar["generation"], 2);
    assert_eq!(sidecar["links"][0]["link_id"], "link-1");
    assert_eq!(sidecar["links"][1]["link_id"], "link-0");
    let deadline = Instant::now() + Duration::from_secs(5);
    while !stack
        .sender
        .log_snapshot()
        .iter()
        .any(|line| line.contains("SIGHUP"))
    {
        assert!(Instant::now() < deadline, "sender did not receive SIGHUP");
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(stack.sender.is_alive() && stack.processes[0].is_alive());
}

#[test]
fn profile_restart_of_exited_namespace_pid_is_typed_and_isolated() {
    if !stack::available() {
        return;
    }
    // Given: an exited exact inner PID beside a live process in the same namespace.
    let topo = BondTopology::new("profileexit", &[NAT], MappingMode::None).unwrap();
    let mut other =
        NamespaceProcess::spawn_process_only(&topo.receiver_ns, "sleep", &["30"]).unwrap();
    let mut target =
        NamespaceProcess::spawn_process_only(&topo.receiver_ns, "sleep", &["30"]).unwrap();
    target.stop_process_only().unwrap();
    // When: restart is attempted after the target has exited and been reaped.
    let error = target.restart_process_only().unwrap_err();
    // Then: the API fails closed with no namespace-wide signaling.
    assert!(matches!(
        error.downcast_ref(),
        Some(ProcessControlError::AlreadyExited)
    ));
    assert!(other.is_alive());
}
