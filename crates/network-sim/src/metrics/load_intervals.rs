use serde::{Deserialize, Serialize};

use super::sink::SinkSeries;
use super::{MetricError, Window, rate_number};
use crate::profile::{Action, EventLog, Profile};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadInterval {
    pub start_ms: i64,
    pub end_ms: i64,
    pub offered_bps: u64,
    pub target_bps: f64,
    pub graded: bool,
    pub no_collapse: bool,
    pub reached_ms: Option<u64>,
    pub recovered: bool,
}

pub fn evaluate(
    profile: &Profile,
    log: &EventLog,
    sink_capacity: (&SinkSeries, u64),
) -> Result<Vec<LoadInterval>, MetricError> {
    let (sink, capacity) = sink_capacity;
    let episodes = super::episodes::evaluate(profile, log, sink)?;
    let mut exclusions: Vec<Window> = episodes
        .iter()
        .filter(|episode| !episode.graded)
        .filter_map(|episode| {
            episode
                .restore_ms
                .and_then(|restore| Window::new(episode.onset_ms, restore).ok())
        })
        .collect();
    for record in &log.entries {
        if !record.event.graded {
            let start = i64::try_from(record.event.at.as_millis())
                .map_err(|_| MetricError::InvalidWindow)?;
            let end = i64::try_from(record.t_actual_ms).map_err(|_| MetricError::InvalidWindow)?;
            if end > start {
                exclusions.push(Window::new(start, end)?);
            }
        }
    }
    let mut boundaries = Vec::new();
    for record in &log.entries {
        match record.event.action {
            Action::OfferedRate { bps } => boundaries.push((record, bps)),
            Action::Periodic { .. } => {
                return Err(MetricError::InvalidField(
                    "unexpanded Periodic event".into(),
                ));
            }
            Action::SetImpairment(_)
            | Action::DataBlackhole { .. }
            | Action::LinkUp(_)
            | Action::DefaultRoute(_)
            | Action::Replug
            | Action::ReceiverRestart
            | Action::SighupReorder(_)
            | Action::CrossTraffic { .. } => {}
        }
    }
    let duration =
        u64::try_from(profile.duration.as_millis()).map_err(|_| MetricError::InvalidWindow)?;
    boundaries
        .iter()
        .enumerate()
        .map(|(i, (record, offered))| {
            let start =
                i64::try_from(record.t_actual_ms).map_err(|_| MetricError::InvalidWindow)?;
            let end = i64::try_from(
                boundaries
                    .get(i + 1)
                    .map_or(duration, |(r, _)| r.t_actual_ms),
            )
            .map_err(|_| MetricError::InvalidWindow)?;
            let window = Window::new(start, end)?;
            let buckets: Vec<_> = sink
                .seconds(window)?
                .into_iter()
                .filter(|bucket| {
                    !exclusions.iter().any(|excluded| {
                        bucket.start_ms() < excluded.end_ms() && bucket.t_ms > excluded.start_ms()
                    })
                })
                .collect();
            let feasible = rate_number((*offered).min(capacity));
            let graded = record.event.graded && *offered > 0;
            let horizon = i64::try_from(record.event.horizon.as_millis())
                .map_err(|_| MetricError::InvalidWindow)?;
            let reach_end = if horizon > 0 {
                end.min(
                    start
                        .checked_add(horizon)
                        .ok_or(MetricError::InvalidWindow)?,
                )
            } else {
                end
            };
            let reached_ms = graded
                .then(|| {
                    buckets
                        .iter()
                        .find(|b| b.t_ms <= reach_end && b.bps() > 0.0 && b.bps() >= 0.9 * feasible)
                        .map(|b| b.t_ms.abs_diff(start))
                })
                .flatten();
            let healthy = buckets
                .iter()
                .filter(|b| b.bps() > 0.0 && b.bps() >= 0.7 * feasible)
                .count();
            Ok(LoadInterval {
                start_ms: start,
                end_ms: end,
                offered_bps: *offered,
                target_bps: 0.9 * feasible,
                graded,
                no_collapse: graded
                    && sink.covers(window)
                    && !buckets.is_empty()
                    && healthy * 10 >= buckets.len() * 9,
                reached_ms,
                recovered: reached_ms.is_some(),
            })
        })
        .collect()
}
