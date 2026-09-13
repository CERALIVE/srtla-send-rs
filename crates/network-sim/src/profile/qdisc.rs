use std::cell::{Cell, RefCell};
use std::process::Command;

use anyhow::{Result, ensure};

use crate::{ImpairmentConfig, Namespace};

/// Owns the composed layout, not the namespace. The namespace must outlive this handle.
pub struct LinkQdisc {
    namespace: String,
    interface: String,
    current: RefCell<Option<ImpairmentConfig>>,
    blackhole: Cell<bool>,
}

impl LinkQdisc {
    pub fn new(namespace: &Namespace, interface: &str) -> Self {
        Self {
            namespace: namespace.name.clone(),
            interface: interface.into(),
            current: RefCell::new(None),
            blackhole: Cell::new(false),
        }
    }

    /// Updates only band one. Kind changes detach that band, never the root/filter.
    pub fn apply(&self, config: &ImpairmentConfig) -> Result<()> {
        let commands = apply_commands(&self.interface, config)?;
        let current = self.current.borrow();
        match current.as_ref() {
            None => {
                for command in installation_commands(&self.interface) {
                    self.execute(&command)?;
                }
            }
            Some(old) if old.tbf_shaping != config.tbf_shaping => {
                self.execute(&words(&format!(
                    "qdisc del dev {} parent 1:1 handle 10:",
                    self.interface
                )))?;
            }
            Some(_) => {}
        }
        for command in commands {
            self.execute(&command)?;
        }
        drop(current);
        *self.current.borrow_mut() = Some(config.clone());
        Ok(())
    }

    pub fn set_blackhole(&self, on: bool) -> Result<()> {
        ensure!(
            self.current.borrow().is_some(),
            "apply base impairment before blackhole"
        );
        if on != self.blackhole.get() {
            self.execute(&filter_command(&self.interface, on))?;
            self.blackhole.set(on);
        }
        Ok(())
    }

    /// Called only after the owning veth has actually been recreated.
    pub fn reset_after_replug(&self) {
        *self.current.borrow_mut() = None;
        self.blackhole.set(false);
    }

    fn execute(&self, args: &[String]) -> Result<()> {
        let output = Command::new("sudo")
            .args(["-n", "ip", "netns", "exec", &self.namespace, "tc"])
            .args(args)
            .output()?;
        ensure!(
            output.status.success(),
            "tc {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}

pub fn installation_commands(iface: &str) -> Vec<Vec<String>> {
    vec![
        words(&format!(
            "qdisc add dev {iface} root handle 1: prio bands 2 priomap 0 0 0 0 0 0 0 0 0 0 0 0 0 \
             0 0 0"
        )),
        words(&format!(
            "qdisc add dev {iface} parent 1:2 handle 20: netem loss 100%"
        )),
    ]
}

pub fn filter_command(iface: &str, on: bool) -> Vec<String> {
    let mut command = words(&format!(
        "filter {} dev {iface} parent 1: protocol ip prio 10",
        if on { "add" } else { "del" }
    ));
    if on {
        command.extend(words("u32 match u16 0x0400 0xfc00 at 2 flowid 1:2"));
    }
    command
}

pub fn apply_commands(iface: &str, config: &ImpairmentConfig) -> Result<Vec<Vec<String>>> {
    ensure!(
        config.queue_limit != Some(0),
        "queue_limit must be positive"
    );
    ensure!(
        config.delay_distribution.is_none()
            || (config.delay_ms.is_some() && config.jitter_ms.is_some_and(|j| j > 0)),
        "delay distribution requires delay and positive jitter"
    );
    let mut commands = Vec::new();
    let (parent, handle) = if config.tbf_shaping {
        let rate = config
            .rate_kbit
            .filter(|r| *r > 0)
            .ok_or_else(|| anyhow::anyhow!("TBF requires positive rate_kbit"))?;
        let burst = rate
            .checked_mul(1000)
            .ok_or_else(|| anyhow::anyhow!("TBF rate overflow"))?
            / 8;
        let burst = burst.max(15400) / 10;
        let latency = config
            .tbf_latency_ms
            .map_or_else(|| "1s".into(), |ms| format!("{ms}ms"));
        commands.push(words(&format!(
            "qdisc replace dev {iface} parent 1:1 handle 10: tbf rate {rate}kbit burst {burst} \
             latency {latency}"
        )));
        ("10:1", "11:")
    } else {
        ("1:1", "10:")
    };
    let mut netem = words(&format!(
        "qdisc replace dev {iface} parent {parent} handle {handle} netem"
    ));
    netem.extend(config.netem_args(!config.tbf_shaping));
    if config.queue_limit.is_none() {
        netem.extend(words("limit 1000"));
    }
    commands.push(netem);
    Ok(commands)
}

fn words(command: &str) -> Vec<String> {
    command.split_whitespace().map(str::to_owned).collect()
}
