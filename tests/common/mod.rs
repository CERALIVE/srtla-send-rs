//! Shared utilities for integration tests.
#![allow(dead_code)]

use std::time::Duration;

use network_sim::{
    SrtlaTestStack, check_impairment_deps, check_integration_deps, wait_for_udp_listener,
};

/// Bounded readiness gate replacing a fixed "sleep N seconds for registration".
/// Returns once srtla_send's local SRT listener is up and every uplink has
/// completed registration, so callers wait on observed state, not a timer.
pub fn wait_until_ready(stack: &SrtlaTestStack) {
    wait_for_udp_listener(
        &stack.topo.sender_ns,
        stack.sender_srt_port(),
        Duration::from_secs(15),
    )
    .expect("srtla_send local SRT listener up");
    stack
        .wait_for_registered_uplinks(Duration::from_secs(15))
        .expect("srtla_send uplinks registered with receiver");
}

/// Check all integration test dependencies. Returns `true` if tests should
/// be skipped (prints the reason to stderr). Use at the top of every test.
pub fn skip_without_deps() -> bool {
    match check_integration_deps() {
        Ok(()) => false,
        Err(reason) => {
            eprintln!("Skipping: {reason}");
            true
        }
    }
}

/// Like `skip_without_deps` but also requires netem for impairment tests.
pub fn skip_without_impairment_deps() -> bool {
    match check_impairment_deps() {
        Ok(()) => false,
        Err(reason) => {
            eprintln!("Skipping: {reason}");
            true
        }
    }
}

/// Build the srtla_send binary (debug mode). Call once before tests that
/// need the binary. Panics if the build fails.
pub fn build_srtla_send() {
    let status = std::process::Command::new("cargo")
        .args(["build", "--bin", "srtla_send"])
        .status()
        .expect("failed to run cargo build");
    assert!(status.success(), "cargo build failed");
}

/// Inject `count` UDP packets to srtla_send's local SRT port from within
/// the sender namespace. Each packet is 188 bytes of zeroes.
pub fn inject_packets(stack: &SrtlaTestStack, count: usize) -> anyhow::Result<()> {
    network_sim::inject_udp_packets(
        &stack.topo.sender_ns,
        "127.0.0.1",
        stack.sender_srt_port(),
        count,
    )
}

/// Inject sequenced SRT DATA into the bond for `duration`, returning the next
/// sequence so a later phase cannot reuse an already acknowledged packet.
/// Synthetic DATA isolates the SRTLA guard from SRT retransmission feedback.
#[allow(dead_code)]
pub fn inject_stream(
    stack: &SrtlaTestStack,
    first_seq: u32,
    packets_per_sec: u32,
    duration: Duration,
) -> anyhow::Result<u32> {
    let port = stack.sender_srt_port();
    let seconds = duration.as_secs_f64();
    let script = format!(
        "import socket,time\ns=socket.socket(socket.AF_INET,socket.SOCK_DGRAM)\nstart=time.\
         monotonic(); i=0\nwhile time.monotonic()-start < {seconds}:\n\x20   \
         data=({first_seq}+i).to_bytes(4,'big')+bytes(184)\n\x20   \
         s.sendto(data,('127.0.0.1',{port}))\n\x20   i+=1\n\x20   \
         delay=start+i/{packets_per_sec}-time.monotonic()\n\x20   if delay>0: \
         time.sleep(delay)\ns.close()\nprint({first_seq}+i)"
    );
    let output = stack
        .topo
        .sender_ns
        .exec_checked("python3", &["-c", &script])?;
    Ok(String::from_utf8(output.stdout)?.trim().parse()?)
}

/// Collect and print all process output for debugging failed tests.
pub fn dump_output(output: &network_sim::StackOutput) {
    eprintln!("--- srtla_send stdout ---");
    for line in &output.srtla_send_stdout {
        eprintln!("  {line}");
    }
    eprintln!("--- srtla_send stderr ---");
    for line in &output.srtla_send_stderr {
        eprintln!("  {line}");
    }
    eprintln!("--- srtla_rec stdout ---");
    for line in &output.srtla_rec_stdout {
        eprintln!("  {line}");
    }
    eprintln!("--- srtla_rec stderr ---");
    for line in &output.srtla_rec_stderr {
        eprintln!("  {line}");
    }
    eprintln!("--- srt-live-transmit stdout ---");
    for line in &output.srt_server_stdout {
        eprintln!("  {line}");
    }
    eprintln!("--- srt-live-transmit stderr ---");
    for line in &output.srt_server_stderr {
        eprintln!("  {line}");
    }
}
