use anyhow::Result;

use crate::{Namespace, SrtlaTestTopology};

pub(super) const fn source_table(index: usize) -> usize {
    let table = 101 + index;
    if table < 253 { table } else { table + 3 }
}

pub(super) struct SourceRoute<'a> {
    pub iface: &'a str,
    pub subnet: &'a str,
    pub gateway: &'a str,
    pub ip: &'a str,
    pub table: usize,
}

impl SourceRoute<'_> {
    pub fn commands(&self) -> [String; 3] {
        let Self {
            iface,
            subnet,
            gateway,
            ip,
            table,
        } = self;
        [
            format!("route add {subnet} dev {iface} table {table}"),
            format!("route add default via {gateway} dev {iface} table {table}"),
            format!("rule add from {ip}/32 table {table}"),
        ]
    }
}

pub(super) fn ip(ns: &Namespace, command: &str) -> Result<()> {
    ns.exec_checked("ip", &command.split_whitespace().collect::<Vec<_>>())?;
    Ok(())
}

pub(super) fn relax(ns: &Namespace, iface: &str) -> Result<()> {
    ns.exec_checked(
        "sysctl",
        &["-qw", &format!("net.ipv4.conf.{iface}.rp_filter=0")],
    )?;
    Ok(())
}

/// Source-routing setup extracted from the A/B runner, preserving its original topology.
pub fn configure_bond_routing(topo: &SrtlaTestTopology) -> Result<()> {
    for (ns, ifaces) in [
        (&topo.sender_ns, &topo.sender_ifaces),
        (&topo.receiver_ns, &topo.receiver_ifaces),
    ] {
        for iface in ["all", "default"]
            .into_iter()
            .chain(ifaces.iter().map(String::as_str))
        {
            relax(ns, iface)?;
        }
    }
    for idx in 1..topo.sender_ips.len() {
        let table = 101 + idx;
        let src = &topo.sender_ips[idx];
        let sif = &topo.sender_ifaces[idx];
        let rif = &topo.receiver_ifaces[idx];
        let recv = &topo.receiver_ip;
        ip(
            &topo.sender_ns,
            &format!("route add {recv} dev {sif} table {table}"),
        )?;
        ip(
            &topo.sender_ns,
            &format!("rule add from {src} lookup {table}"),
        )?;
        ip(
            &topo.receiver_ns,
            &format!("route add {src} dev {rif} src {recv}"),
        )?;
    }
    Ok(())
}
