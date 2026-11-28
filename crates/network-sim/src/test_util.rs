use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

static NS_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Returns `true` if the environment supports namespace-based tests
/// (requires `ip` tool and passwordless `sudo`).
pub fn check_privileges() -> bool {
    let has_ip = Command::new("ip")
        .arg("netns")
        .output()
        .is_ok_and(|o| o.status.success());

    has_ip
        && Command::new("sudo")
            .args(["-n", "ip", "netns", "list"])
            .output()
            .is_ok_and(|o| o.status.success())
}

/// Generate a unique namespace/interface name safe for parallel tests.
///
/// Combines prefix + PID + atomic counter. The uniqueness suffix
/// (`_{pid:x}_{seq}`) is always preserved; the prefix is truncated
/// if the total would exceed 15 chars (Linux netdev name limit).
///
/// Under `scripts/netns_test_gate.sh` the suffix is `-{pid:x}-{seq}`. A name
/// always ends in its separator and the counter's digits, so no run outside the
/// gate can make a name a gated run made, and the gate runs one at a time: its
/// cleanup can delete such a name without hitting another run's namespace.
pub fn unique_ns_name(prefix: &str) -> String {
    let seq = NS_COUNTER.fetch_add(1, Ordering::Relaxed);
    let gated = std::env::var_os(crate::topology::NETNS_RECORD_ENV).is_some();
    ns_name(prefix, std::process::id() % 0xffff, seq, gated)
}

fn ns_name(prefix: &str, pid: u32, seq: u32, gated: bool) -> String {
    let sep = if gated { '-' } else { '_' };
    let suffix = format!("{sep}{pid:x}{sep}{seq}");
    let max_prefix = 15_usize.saturating_sub(suffix.len());
    let truncated_prefix = &prefix[..prefix.len().min(max_prefix)];
    format!("{truncated_prefix}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::ns_name;

    /// A name ends in `_<digits>` outside the gate and `-<digits>` under it, so
    /// the two can never be equal, whatever the prefix, pid or counter.
    #[test]
    fn a_gated_name_never_matches_an_ungated_one() {
        for prefix in ["", "ts0", "fwd_s", "basic_r", "twin_reroute_g1"] {
            for (pid, seq) in [(0, 0), (0x1a2b, 7), (0xfffe, 12345)] {
                let ungated = ns_name(prefix, pid, seq, false);
                let gated = ns_name(prefix, pid, seq, true);
                assert!(
                    ungated.len() <= 15 && gated.len() <= 15,
                    "{ungated} / {gated}"
                );
                assert!(ungated.ends_with(&format!("_{pid:x}_{seq}")), "{ungated}");
                assert!(gated.ends_with(&format!("-{pid:x}-{seq}")), "{gated}");
            }
        }
    }
}
