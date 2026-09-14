use super::observe::Run;

#[derive(Default)]
pub struct Checks(Vec<String>);

impl Checks {
    pub fn require(&mut self, condition: bool, message: impl Into<String>) {
        if !condition {
            self.0.push(message.into());
        }
    }
    pub fn finish(self) {
        assert!(self.0.is_empty(), "{}", self.0.join("\n"));
    }

    pub fn obstruction(&mut self, run: &Run, id: &str) {
        let onset = run
            .events
            .iter()
            .find(|(_, e)| {
                matches!(
                    e.action,
                    network_sim::profile::Action::DataBlackhole { on: true }
                )
            })
            .expect("blackhole onset")
            .0;
        let restore = run
            .events
            .iter()
            .find(|(_, e)| {
                matches!(
                    e.action,
                    network_sim::profile::Action::DataBlackhole { on: false }
                )
            })
            .expect("blackhole restore")
            .0;
        let tau = (4.0 * f64::from(run.at(onset).snapshot.link(id).rtt_ms)).clamp(1000.0, 3000.0)
            / 1000.0;
        let stalled = run.samples.iter().find(|s| {
            s.t >= onset
                && s.t <= onset + tau + 2.0
                && s.snapshot.link(id).health == "stalled"
                && s.snapshot.link(id).weight_percent == 0
        });
        self.require(
            stalled.is_some(),
            format!("{id}: no stalled/0 snapshot within tau+2={:.3}s", tau + 2.0),
        );
        let rejoining = run
            .samples
            .iter()
            .find(|s| s.t >= restore && s.snapshot.link(id).health == "rejoining");
        let healthy = rejoining.and_then(|r| {
            run.samples
                .iter()
                .find(|s| s.t > r.t && s.snapshot.link(id).health == "healthy")
        });
        self.require(
            healthy.is_some_and(|s| s.t <= restore + 4.0 * tau + 5.0),
            format!(
                "{id}: no rejoining -> healthy within {:.3}s",
                4.0 * tau + 5.0
            ),
        );
        let full_rate = run
            .events
            .iter()
            .find(|(t, e)| {
                *t > restore && matches!(e.action, network_sim::profile::Action::OfferedRate { .. })
            })
            .expect("separate full-rate restoration")
            .0;
        self.require(
            full_rate >= restore + 4.0 * tau + 5.0,
            "full-rate restoration preceded recovery deadline",
        );
        self.require(
            rejoining.is_some_and(|r| {
                r.t < full_rate
                    && run
                        .samples
                        .iter()
                        .filter(|s| s.t >= r.t && s.t < full_rate)
                        .all(|s| {
                            matches!(s.snapshot.link(id).health.as_str(), "rejoining" | "healthy")
                        })
            }),
            format!("{id}: missing or relapsed feasible-load recovery"),
        );
        self.require(
            run.samples
                .iter()
                .filter(|s| s.t >= full_rate + 1.0)
                .all(|s| s.snapshot.connections.iter().all(|l| l.health == "healthy")),
            "both links must remain Healthy after full-rate restoration",
        );
        let start = onset + tau + 2.0;
        let end = restore - 0.25;
        let bytes = run.phase(start, end);
        self.require(
            share(&bytes, 1) >= 0.9,
            format!("{id}: surviving DATA share <90%: {bytes:?}"),
        );
        self.require(
            bytes[1] >= 100_000,
            "survivor carries real DATA, not just control",
        );
        self.require(
            run.at(end).keepalives[0] > run.at(start).keepalives[0],
            format!("{id}: no continuing stalled-link keepalive RTT echoes"),
        );
        eprintln!(
            "adaptive obstruction {id}: tau={tau:.3}s stalled={:?} rejoining={:?} healthy={:?} \
             survival={bytes:?} keepalives={} -> {}",
            stalled.map(|s| s.t - onset),
            rejoining.map(|s| s.t - restore),
            healthy.map(|s| s.t - restore),
            run.at(start).keepalives[0],
            run.at(end).keepalives[0]
        );
    }
}

pub fn share(bytes: &[u64], index: usize) -> f64 {
    let total: u64 = bytes.iter().sum();
    assert!(total > 100_000, "measurement must contain DATA: {bytes:?}");
    let fraction = f64::from(u32::try_from(bytes[index]).expect("bounded phase bytes"))
        / f64::from(u32::try_from(total).expect("bounded phase total"));
    eprintln!("adaptive phase bytes={bytes:?} link={index} share={fraction:.6}");
    fraction
}
