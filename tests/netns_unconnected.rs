//! Privileged supplements for the unconnected-uplink-socket change.
//!
//! (a) A receiver that replies from a SECOND address on the same host keeps the
//!     link alive. This is the interop point unconnected sockets buy: a
//!     `connect(2)`ed socket would silently stop receiving and time out.
//! (b) A send that hard-fails (route removed / netfilter REJECT) drives the link
//!     into recovery promptly, rather than waiting on the liveness timeout.
//!
//! Silent loss is deliberately NOT covered here — it produces no send error and
//! is demoted by liveness + quality scoring instead (see `netns_failure.rs`).
//!
//! Self-skips without netns privileges, `srtla_rec`, `srt-live-transmit`, or
//! `iptables`.

mod common;

use std::thread::sleep;
use std::time::Duration;

use network_sim::SrtlaTestStack;

const SRTLA_REC_PORT: &str = "5000";
const SECOND_RECEIVER_IP: &str = "10.10.1.22";
/// srtla_rec already owns 0.0.0.0:5000, so the alternate source uses its own port.
const SECOND_RECEIVER_PORT: u16 = 5001;

fn skip_without_iptables(stack: &SrtlaTestStack) -> bool {
    match stack
        .topo
        .receiver_ns
        .exec("iptables", &["-t", "nat", "-L"])
    {
        Ok(out) if out.status.success() => false,
        _ => {
            eprintln!("Skipping: iptables unavailable inside the test namespace");
            true
        }
    }
}

fn sender_log(stack: &SrtlaTestStack) -> String {
    stack.sender_log_snapshot().join("\n")
}

#[test]
fn receiver_replying_from_a_second_address_keeps_the_link_alive() {
    if common::skip_without_deps() {
        return;
    }
    common::build_srtla_send();

    let mut stack = SrtlaTestStack::start("unconn_altsrc", 1, &[]).expect("start stack");
    if skip_without_iptables(&stack) {
        return;
    }
    common::wait_until_ready(&stack);

    let receiver_iface = stack.topo.receiver_ifaces[0].clone();
    let sender_ip = stack.topo.sender_ips[0].clone();

    stack
        .topo
        .receiver_ns
        .exec_checked(
            "ip",
            &[
                "addr",
                "add",
                &format!("{SECOND_RECEIVER_IP}/24"),
                "dev",
                &receiver_iface,
            ],
        )
        .expect("add a second receiver address");

    let uplink_port = *network_sim::bound_udp_ports(&stack.topo.sender_ns, &sender_ip)
        .expect("read the uplink's bound port")
        .first()
        .expect("the uplink socket must be bound");

    // Speak to the established uplink from the receiver's SECOND address —
    // exactly what a multi-homed or NAT'd receiver looks like. A `connect(2)`ed
    // socket would never see this datagram at all.
    network_sim::inject_udp_packets_from(
        &stack.topo.receiver_ns,
        SECOND_RECEIVER_IP,
        SECOND_RECEIVER_PORT,
        &sender_ip,
        uplink_port,
        188,
        5,
    )
    .expect("inject datagrams from the second receiver address");

    // Several keepalive rounds: enough for the link to have died if the foreign
    // source had displaced or blocked the real one.
    sleep(Duration::from_secs(4));
    let log = sender_log(&stack);

    let output = stack.stop();
    common::dump_output(&output);

    assert!(
        log.contains("datagram from unexpected source"),
        "the sender must accept and count datagrams from the receiver's other address:\n{log}"
    );
    assert!(
        !log.contains("attempting full socket reconnection"),
        "a datagram from another address on the same receiver must NOT kill the link:\n{log}"
    );
}

#[test]
fn send_error_marks_the_link_for_recovery_promptly() {
    if common::skip_without_deps() {
        return;
    }
    common::build_srtla_send();

    let mut stack = SrtlaTestStack::start("unconn_senderr", 1, &[]).expect("start stack");
    if skip_without_iptables(&stack) {
        return;
    }
    common::wait_until_ready(&stack);

    // A netfilter REJECT on locally generated output makes the sendto syscall
    // itself fail, which is the point: a silent blackhole would produce no send
    // error at all (that path is netns_failure.rs's liveness/quality story).
    stack
        .topo
        .sender_ns
        .exec_checked(
            "iptables",
            &[
                "-A",
                "OUTPUT",
                "-p",
                "udp",
                "--dport",
                SRTLA_REC_PORT,
                "-j",
                "REJECT",
                "--reject-with",
                "icmp-host-unreachable",
            ],
        )
        .expect("reject the uplink's egress");

    common::inject_packets(&stack, 200).expect("inject packets");
    sleep(Duration::from_secs(4));
    let log = sender_log(&stack);

    let output = stack.stop();
    common::dump_output(&output);

    assert!(
        log.contains("marking for recovery"),
        "a hard send error must drive the link into recovery within 4s:\n{log}"
    );
}
