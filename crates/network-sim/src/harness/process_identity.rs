use std::process::Command;

use anyhow::{Context, Result, ensure};

#[derive(Debug)]
pub(crate) struct Identity {
    pub pid: u32,
    pub start_time: u64,
    parent: u32,
}

pub(super) fn read(pid: u32) -> Result<Identity> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let fields = stat
        .rsplit_once(')')
        .context("process stat comm terminator")?
        .1
        .split_whitespace()
        .collect::<Vec<_>>();
    Ok(Identity {
        pid,
        parent: fields.get(1).context("process parent field")?.parse()?,
        start_time: fields
            .get(19)
            .context("process start-time field")?
            .parse()?,
    })
}

pub(super) fn namespace_pids(namespace: &str) -> Result<Vec<u32>> {
    let out = Command::new("sudo")
        .args(["-n", "ip", "netns", "pids", namespace])
        .output()?;
    ensure!(
        out.status.success(),
        "list namespace PIDs: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)?
        .split_whitespace()
        .map(|p| p.parse().context("parse namespace PID"))
        .collect()
}

pub(super) fn discover(namespace: &str, before: &[u32], wrapper: u32) -> Result<Option<Identity>> {
    let after = namespace_pids(namespace)?;
    for pid in after.iter().filter(|pid| !before.contains(pid)) {
        let Ok(candidate) = read(*pid) else {
            continue;
        };
        let mut parent = candidate.parent;
        let mut nested = false;
        for _ in 0..32 {
            if *pid == wrapper || parent == wrapper {
                return Ok(Some(candidate));
            }
            if after.contains(&parent) {
                nested = true;
                break;
            }
            match read(parent) {
                Ok(ancestor) if ancestor.parent != parent => parent = ancestor.parent,
                Ok(_) | Err(_) => break,
            }
        }
        if nested {
            continue;
        }
    }
    Ok(None)
}
