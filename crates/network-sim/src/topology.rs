use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{Context, Result, bail};
use tracing::debug;

/// A Linux network namespace with RAII cleanup.
///
/// Creates the namespace on construction, brings up loopback, and deletes
/// it on drop. All commands inside the namespace run via `sudo ip netns exec`.
pub struct Namespace {
    pub name: String,
}

/// Names the file `scripts/netns_test_gate.sh` gives each run, so that after a
/// failed run it removes exactly the namespaces that run created.
pub(crate) const NETNS_RECORD_ENV: &str = "NETNS_GATE_RECORD";

/// Append `<state> <entry>` as one line to the gate's record file.
fn record_namespace(record: &Path, state: &str, entry: &str) -> Result<()> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(record)
        .and_then(|mut file| file.write_all(format!("{state} {entry}\n").as_bytes()))
        .with_context(|| format!("record netns '{entry}' in '{}'", record.display()))
}

impl Namespace {
    pub fn new(name: &str) -> Result<Self> {
        let record = std::env::var_os(NETNS_RECORD_ENV);
        // Clean up stale namespace with same name (idempotent). Not under the
        // gate: there the name may be another run's, and the gate removes this
        // run's own leftovers itself.
        if record.is_none() {
            let _ = sudo(&["ip", "netns", "del", name]);
        }

        // `pending` before the add, so the gate still hears of a namespace
        // whose creation a killed run never saw finish.
        if let Some(record) = &record {
            record_namespace(Path::new(record), "pending", name)?;
        }
        sudo_checked(&["ip", "netns", "add", name])
            .with_context(|| format!("create netns '{name}'"))?;
        // Owned from here: Drop deletes it, also if the record below fails.
        let ns = Self {
            name: name.to_string(),
        };
        // `created` only after this add succeeded, so a name another run
        // already holds is never this run's to delete. The inode tells this
        // namespace apart from a later one that reuses the name.
        if let Some(record) = &record {
            let path = format!("/run/netns/{name}");
            let stat = sudo_checked(&["stat", "-c", "%i", &path])
                .with_context(|| format!("identify netns '{name}'"))?;
            let inode = String::from_utf8_lossy(&stat.stdout).trim().to_owned();
            record_namespace(Path::new(record), "created", &format!("{name} {inode}"))?;
        }

        debug!(ns = name, "created network namespace");

        // Loopback — best-effort, failure is non-fatal
        let _ = sudo(&["ip", "netns", "exec", name, "ip", "link", "set", "lo", "up"]);

        Ok(ns)
    }

    /// Run a command inside this namespace, returning raw output.
    pub fn exec(&self, cmd: &str, args: &[&str]) -> Result<Output> {
        let mut full_args = vec!["ip", "netns", "exec", &self.name, cmd];
        full_args.extend_from_slice(args);
        sudo(&full_args).with_context(|| format!("exec '{cmd}' in ns '{}'", self.name))
    }

    /// Run a command inside this namespace, failing if it exits non-zero.
    pub fn exec_checked(&self, cmd: &str, args: &[&str]) -> Result<Output> {
        let mut full_args = vec!["ip", "netns", "exec", &self.name, cmd];
        full_args.extend_from_slice(args);
        sudo_checked(&full_args).with_context(|| format!("exec '{cmd}' in ns '{}'", self.name))
    }

    /// Create a veth pair connecting this namespace to `peer`.
    ///
    /// Each end gets an IP address assigned and is brought up.
    /// Interface names must be <= 15 chars (Linux limit).
    pub fn add_veth_link(
        &self,
        peer: &Namespace,
        local_iface: &str,
        peer_iface: &str,
        local_ip: &str,
        peer_ip: &str,
    ) -> Result<()> {
        // Clean up stale veth (idempotent)
        let _ = sudo(&["ip", "link", "del", local_iface]);

        // Create pair in host namespace
        sudo_checked(&[
            "ip",
            "link",
            "add",
            local_iface,
            "type",
            "veth",
            "peer",
            "name",
            peer_iface,
        ])
        .context("create veth pair")?;

        debug!(local = local_iface, peer = peer_iface, "created veth pair");

        // Move each end into its namespace
        sudo_checked(&["ip", "link", "set", local_iface, "netns", &self.name])
            .context("move local veth")?;
        sudo_checked(&["ip", "link", "set", peer_iface, "netns", &peer.name])
            .context("move peer veth")?;

        // Configure local end
        self.exec_checked("ip", &["addr", "add", local_ip, "dev", local_iface])
            .context("set local IP")?;
        self.exec_checked("ip", &["link", "set", local_iface, "up"])
            .context("bring local link up")?;

        // Configure peer end
        peer.exec_checked("ip", &["addr", "add", peer_ip, "dev", peer_iface])
            .context("set peer IP")?;
        peer.exec_checked("ip", &["link", "set", peer_iface, "up"])
            .context("bring peer link up")?;

        debug!(
            ns_local = self.name,
            ns_peer = peer.name,
            local_ip,
            peer_ip,
            "veth link configured"
        );

        Ok(())
    }
}

impl Drop for Namespace {
    fn drop(&mut self) {
        debug!(ns = self.name, "deleting network namespace");
        let _ = sudo(&["ip", "netns", "del", &self.name]);
    }
}

// -- helpers --

/// Run `sudo <args>`, returning raw output.
fn sudo(args: &[&str]) -> Result<Output> {
    Command::new("sudo")
        .args(args)
        .output()
        .with_context(|| format!("sudo {}", args.join(" ")))
}

/// Run `sudo <args>`, returning output on success or bailing with stderr.
fn sudo_checked(args: &[&str]) -> Result<Output> {
    let output = sudo(args)?;
    if !output.status.success() {
        bail!(
            "command failed: sudo {}\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{check_privileges, unique_ns_name};

    #[test]
    fn record_namespace_appends_one_line_per_entry() {
        let dir = tempfile::tempdir().unwrap();
        let record = dir.path().join("record");
        record_namespace(&record, "pending", "basic_s_1_0").unwrap();
        record_namespace(&record, "created", "basic_s_1_0 4026532861").unwrap();
        assert_eq!(
            std::fs::read_to_string(&record).unwrap(),
            "pending basic_s_1_0\ncreated basic_s_1_0 4026532861\n"
        );
    }

    /// The real constructor, run in a child copy of this test binary with a
    /// stub `sudo` as the only command on its PATH: under the gate it never
    /// deletes a name before creating it, `pending` is on record when
    /// `ip netns add` runs, and only a successful add becomes `created`, with
    /// the inode the stub reports.
    #[cfg(unix)]
    #[test]
    fn record_brackets_ip_netns_add() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let (record, calls) = (dir.path().join("record"), dir.path().join("calls"));
        let sudo = dir.path().join("sudo");
        // Shell builtins only, so it needs no PATH: if it cannot run, no real
        // sudo can be found in its place.
        std::fs::write(
            &sudo,
            format!(
                "#!/bin/sh\nrec=\nif [ -f '{record}' ]; then\nwhile IFS= read -r line; do \
                 rec=\"$rec$line,\"; done < '{record}'\nfi\nprintf '%s | %s\\n' \"$*\" \"$rec\" \
                 >> '{calls}'\ncase \"$*\" in\n'ip netns add taken') exit 1 ;;\n'stat -c %i \
                 /run/netns/mine') echo 4242 ;;\nesac\n",
                record = record.display(),
                calls = calls.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&sudo, std::fs::Permissions::from_mode(0o755)).unwrap();
        let status = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "topology::tests::record_brackets_ip_netns_add_child",
                "--ignored",
            ])
            .env("PATH", dir.path())
            .env(NETNS_RECORD_ENV, &record)
            .env("NETNS_RECORD_TEST_CHILD", "1")
            .status()
            .unwrap();
        assert!(status.success(), "the child constructor run failed");

        let calls = std::fs::read_to_string(&calls).unwrap();
        assert!(
            calls
                .lines()
                .any(|line| line == "ip netns add mine | pending mine,"),
            "`pending` must be on record, and `created` not yet, when the add runs; \
             calls:\n{calls}"
        );
        assert!(
            !calls.contains("ip netns del taken"),
            "under the gate a name that may be another run's is never deleted; calls:\n{calls}"
        );
        assert_eq!(
            std::fs::read_to_string(&record).unwrap(),
            "pending mine\ncreated mine 4242\npending taken\n"
        );
    }

    /// The child half of `record_brackets_ip_netns_add`; it does nothing
    /// unless that test started it.
    #[cfg(unix)]
    #[test]
    #[ignore = "run only by record_brackets_ip_netns_add, with a stub sudo"]
    fn record_brackets_ip_netns_add_child() {
        if std::env::var_os("NETNS_RECORD_TEST_CHILD").is_none() {
            return;
        }
        let mine = Namespace::new("mine").expect("the stub add of 'mine' succeeds");
        assert!(
            Namespace::new("taken").is_err(),
            "the stub add of 'taken' fails"
        );
        drop(mine);
        let name = unique_ns_name("t");
        assert!(
            name.starts_with("t-"),
            "under the gate a name uses `-`: {name}"
        );
    }

    #[test]
    fn test_namespace_has_loopback() {
        if !check_privileges() {
            eprintln!("Skipping: insufficient privileges");
            return;
        }

        let ns = Namespace::new(&unique_ns_name("nst_a")).expect("create ns");
        let out = ns.exec("ip", &["link"]).expect("ip link");
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("lo"), "loopback missing: {stdout}");
    }

    #[test]
    fn test_veth_ping() {
        if !check_privileges() {
            eprintln!("Skipping: insufficient privileges");
            return;
        }

        let ns1 = Namespace::new(&unique_ns_name("nst_a")).expect("create ns1");
        let ns2 = Namespace::new(&unique_ns_name("nst_b")).expect("create ns2");

        let id = std::process::id() % 100_000;
        let v_a = format!("va_{id}");
        let v_b = format!("vb_{id}");

        ns1.add_veth_link(&ns2, &v_a, &v_b, "10.200.1.1/24", "10.200.1.2/24")
            .expect("add veth link");

        let out = ns1
            .exec("ping", &["-c", "1", "-W", "1", "10.200.1.2"])
            .expect("ping");

        assert!(
            out.status.success(),
            "ping failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
