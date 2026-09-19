use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};

use super::traffic::CrossTraffic;
use super::{Action, Profile, TimedEvent};
use crate::bond::BondTopology;
use crate::{ImpairmentConfig, NamespaceProcess, wait_for_udp_listener};

/// Borrowed stack handles. Offered-rate changes must reach the caller's actual traffic source.
pub struct ProcessEndpoints<'a> {
    pub sender: &'a mut NamespaceProcess,
    pub receiver: &'a mut NamespaceProcess,
    pub offered_rate: &'a mut dyn FnMut(u64) -> Result<()>,
}

pub struct BondRuntime<'a> {
    pub processes: ProcessEndpoints<'a>,
    receiver_restart_elapsed: Option<Duration>,
    topology: TopologyAccess<'a>,
    impairments: Vec<ImpairmentConfig>,
    blackholes: Vec<bool>,
    routes: Vec<bool>,
    up: Vec<bool>,
    cross_traffic: Vec<Option<CrossTraffic>>,
}

impl<'a> BondRuntime<'a> {
    pub fn new(
        profile: &Profile,
        topology: &'a BondTopology,
        processes: ProcessEndpoints<'a>,
    ) -> Result<Self> {
        Self::build(profile, TopologyAccess::Fixed(topology), processes)
    }

    pub fn with_link_additions(
        profile: &Profile,
        topology: &'a mut BondTopology,
        processes: ProcessEndpoints<'a>,
    ) -> Result<Self> {
        Self::build(profile, TopologyAccess::Growing(topology), processes)
    }

    fn build(
        profile: &Profile,
        topology: TopologyAccess<'a>,
        processes: ProcessEndpoints<'a>,
    ) -> Result<Self> {
        profile.validate()?;
        ensure!(
            profile.links.len() == topology.link_count(),
            "profile/topology link count mismatch"
        );
        for (i, link) in profile.links.iter().enumerate() {
            ensure!(
                link.carrier == topology.carrier_mode(i),
                "profile/topology carrier mismatch at link {i}"
            );
            topology.apply_impairment(i, &link.base)?;
        }
        Ok(Self {
            processes,
            receiver_restart_elapsed: None,
            impairments: profile.links.iter().map(|l| l.base.clone()).collect(),
            blackholes: vec![false; topology.link_count()],
            routes: vec![true; topology.link_count()],
            up: vec![true; topology.link_count()],
            cross_traffic: (0..topology.link_count()).map(|_| None).collect(),
            topology,
        })
    }

    pub fn apply(&mut self, event: &TimedEvent) -> Result<()> {
        ensure!(
            event.link.is_some() == event.action.link_scoped(),
            "action has incorrect link/global scope"
        );
        let link = || -> Result<usize> {
            let index = event.link.context("link-scoped event requires an index")?;
            ensure!(
                index < self.topology.link_count(),
                "event link out of range"
            );
            Ok(index)
        };
        match &event.action {
            Action::AddLink(profile) => {
                match &mut self.topology {
                    TopologyAccess::Growing(topology) => {
                        topology.add_link(profile)?;
                    }
                    TopologyAccess::Fixed(_) => {
                        bail!("AddLink requires BondRuntime::with_link_additions")
                    }
                }
                self.impairments.push(profile.base.clone());
                self.blackholes.push(false);
                self.routes.push(true);
                self.up.push(true);
                self.cross_traffic.push(None);
                self.processes.sender.signal_process_only("-HUP")?;
            }
            Action::SetImpairment(config) => {
                let i = link()?;
                self.topology.apply_impairment(i, config)?;
                self.impairments[i] = config.clone();
            }
            Action::DataBlackhole { on } => {
                let i = link()?;
                self.topology.set_data_blackhole(i, *on)?;
                self.blackholes[i] = *on;
            }
            Action::LinkUp(up) => {
                let i = link()?;
                self.topology.set_link_up(i, *up)?;
                if *up && !self.routes[i] {
                    self.topology.delete_default_route(i)?;
                }
                self.up[i] = *up;
            }
            Action::DefaultRoute(present) => {
                let i = link()?;
                if self.up[i] && *present != self.routes[i] {
                    if *present {
                        self.topology.restore_default_route(i)?;
                    } else {
                        self.topology.delete_default_route(i)?;
                    }
                }
                self.routes[i] = *present;
            }
            Action::Replug => {
                let i = link()?;
                self.topology.replug(i)?;
                self.topology.apply_impairment(i, &self.impairments[i])?;
                self.topology.set_data_blackhole(i, self.blackholes[i])?;
                if !self.routes[i] {
                    self.topology.delete_default_route(i)?;
                }
                if !self.up[i] {
                    self.topology.set_link_up(i, false)?;
                }
            }
            Action::ReceiverRestart => {
                let start = std::time::Instant::now();
                self.processes.receiver.restart_process_only()?;
                self.receiver_restart_elapsed = Some(start.elapsed());
                wait_for_udp_listener(&self.topology.receiver_ns, 5000, Duration::from_secs(5))?;
            }
            Action::SighupReorder(order) => {
                self.topology.reorder(order)?;
                self.processes.sender.signal_process_only("-HUP")?;
            }
            Action::CrossTraffic { mbit, on } => {
                let i = link()?;
                if let Some(mut traffic) = self.cross_traffic[i].take() {
                    traffic.stop()?;
                }
                if *on {
                    self.cross_traffic[i] = Some(CrossTraffic::start(&self.topology, i, *mbit)?);
                }
            }
            Action::OfferedRate { bps } => (self.processes.offered_rate)(*bps)?,
            Action::Periodic { .. } => bail!("expand periodic events before applying them"),
        }
        Ok(())
    }

    /// Kill/respawn duration, excluding the receiver's separate UDP-readiness preflight.
    pub const fn receiver_restart_elapsed(&self) -> Option<Duration> {
        self.receiver_restart_elapsed
    }

    pub fn topology(&self) -> &BondTopology {
        &self.topology
    }
}

enum TopologyAccess<'a> {
    Fixed(&'a BondTopology),
    Growing(&'a mut BondTopology),
}

impl std::ops::Deref for TopologyAccess<'_> {
    type Target = BondTopology;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Fixed(topology) => topology,
            Self::Growing(topology) => topology,
        }
    }
}
