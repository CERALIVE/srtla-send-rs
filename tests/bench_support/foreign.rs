use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use anyhow::{Result, ensure};
use network_sim::NamespaceProcess;

pub fn wait_registered(process: &NamespaceProcess, count: usize) -> Result<()> {
    let start = Instant::now();
    loop {
        let logs = process.log_snapshot();
        let registered: BTreeSet<_> = logs
            .iter()
            .filter_map(|line| {
                line.split_once(": connection established")
                    .map(|(peer, _)| peer)
            })
            .collect();
        if registered.len() >= count {
            return Ok(());
        }
        ensure!(
            start.elapsed() < Duration::from_secs(30),
            "foreign sender registration timeout: {}",
            logs.join("\n")
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}
