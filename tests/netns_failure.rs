//! Link failure and recovery integration tests.
//!
//! Validates that srtla_send detects link failure, continues on surviving
//! links, and recovers when a failed link returns.

mod common;

use std::thread::sleep;
use std::time::Duration;

use network_sim::{ImpairmentConfig, SrtlaTestStack};

#[test]
fn test_link_failure_failover() {
    if common::skip_without_impairment_deps() {
        return;
    }
    common::build_srtla_send();

    let mut stack = SrtlaTestStack::start("fail", 2, &[]).expect("start stack");

    common::wait_until_ready(&stack);

    // Inject background data
    common::inject_packets(&stack, 100).expect("inject initial data");
    // Steady-state window: let baseline traffic flow before failing a link.
    sleep(Duration::from_secs(2));

    // Kill link 0 with 100% loss
    stack
        .impair_link(
            0,
            ImpairmentConfig {
                loss_percent: Some(100.0),
                ..Default::default()
            },
        )
        .expect("kill link 0");

    // Continue sending — should survive on link 1
    common::inject_packets(&stack, 200).expect("inject data after link kill");
    // Steady-state window: let failover settle on the surviving link.
    sleep(Duration::from_secs(5));

    let output = stack.stop();
    common::dump_output(&output);

    let all_stderr: String = output.srtla_send_stderr.join("\n");
    assert!(
        !all_stderr.contains("PANIC") && !all_stderr.contains("panic"),
        "srtla_send panicked after link failure"
    );
}

#[test]
fn test_link_recovery() {
    if common::skip_without_impairment_deps() {
        return;
    }
    common::build_srtla_send();

    let mut stack = SrtlaTestStack::start("recv", 2, &[]).expect("start stack");

    common::wait_until_ready(&stack);

    // Kill link 0
    stack
        .impair_link(
            0,
            ImpairmentConfig {
                loss_percent: Some(100.0),
                ..Default::default()
            },
        )
        .expect("kill link 0");

    // Steady-state window: let the sender observe link 0 failing.
    sleep(Duration::from_secs(5));

    // Restore link 0 (clear impairment)
    stack
        .impair_link(0, ImpairmentConfig::default())
        .expect("restore link 0");

    // Steady-state window: let link 0 re-establish after restore.
    sleep(Duration::from_secs(5));

    common::inject_packets(&stack, 100).expect("inject data after recovery");
    // Steady-state window: let injected data flow post-recovery.
    sleep(Duration::from_secs(3));

    let output = stack.stop();
    common::dump_output(&output);

    let all_stderr: String = output.srtla_send_stderr.join("\n");
    assert!(
        !all_stderr.contains("PANIC") && !all_stderr.contains("panic"),
        "srtla_send panicked during link recovery"
    );
}
