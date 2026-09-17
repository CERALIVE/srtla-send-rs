use serde::{Deserialize, Serialize};

use super::sink::SinkSeries;
use super::{MetricError, Window};
use crate::profile::{Action, EventLog, Profile};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub event_index: usize,
    pub onset_ms: i64,
    pub restore_ms: Option<i64>,
    pub baseline_bps: f64,
    pub horizon_ms: u64,
    pub end_ms: i64,
    pub graded: bool,
    pub impacted: bool,
    pub failover_ms: Option<u64>,
    pub recovery_ms: Option<u64>,
    pub outage_ms: u64,
    pub recovered: bool,
    pub complete: bool,
}

pub fn evaluate(
    profile: &Profile,
    log: &EventLog,
    sink: &SinkSeries,
) -> Result<Vec<Episode>, MetricError> {
    let mut states = profile.initial_states();
    let mut link_count = profile.links.len();
    let mut episodes = Vec::new();
    for (i, record) in log.entries.iter().enumerate() {
        let event = &record.event;
        crate::profile::observe_creation(&mut states, &mut link_count, event)
            .map_err(|error| MetricError::InvalidField(error.to_string()))?;
        let tracks_restore = match &event.action {
            Action::OfferedRate { .. } => {
                states.push(event.clone());
                continue;
            }
            Action::Periodic { .. } => {
                return Err(MetricError::InvalidField(
                    "unexpanded Periodic event".into(),
                ));
            }
            Action::SetImpairment(_)
            | Action::DataBlackhole { .. }
            | Action::LinkUp(_)
            | Action::DefaultRoute(_)
            | Action::SighupReorder(_) => !event.horizon.is_zero(),
            Action::AddLink(_) | Action::Replug | Action::ReceiverRestart => false,
            Action::CrossTraffic { on, .. } => *on,
        };
        let onset = i64::try_from(record.t_actual_ms).map_err(|_| MetricError::InvalidWindow)?;
        let prior = states
            .iter()
            .rev()
            .find(|old| old.link == event.link && old.action.same_channel(&event.action));
        let restore = prior
            .filter(|old| tracks_restore && old.action != event.action)
            .and_then(|old| {
                log.entries[i + 1..]
                    .iter()
                    .find(|next| next.event.link == event.link && next.event.action == old.action)
            })
            .map(|r| i64::try_from(r.t_actual_ms))
            .transpose()
            .map_err(|_| MetricError::InvalidWindow)?;
        let horizon =
            u64::try_from(event.horizon.as_millis()).map_err(|_| MetricError::InvalidWindow)?;
        let end = restore
            .unwrap_or(onset)
            .checked_add(i64::try_from(horizon).map_err(|_| MetricError::InvalidWindow)?)
            .ok_or(MetricError::InvalidWindow)?;
        let baseline_window = Window::new(
            onset.checked_sub(10000).ok_or(MetricError::InvalidWindow)?,
            onset,
        )?;
        let baseline = if sink.covers(baseline_window) {
            sink.useful_goodput_bps(baseline_window)?
        } else {
            0.0
        };
        let window = if end > onset {
            Some(Window::new(onset, end)?)
        } else {
            None
        };
        let complete = window.is_some_and(|w| sink.covers(w)) && sink.covers(baseline_window);
        let buckets = window
            .map(|w| sink.seconds(w))
            .transpose()?
            .unwrap_or_default();
        let raw_buckets: Vec<_> = sink
            .buckets()
            .iter()
            .filter(|b| b.start_ms() >= onset && b.t_ms <= end)
            .collect();
        let impact = raw_buckets.iter().find(|b| b.bps() < 0.5 * baseline);
        let recovered_at = impact.and_then(|impact| {
            buckets
                .iter()
                .find(|b| b.start_ms() >= impact.t_ms && b.bps() >= 0.9 * baseline)
        });
        let failover_ms = match (impact, recovered_at) {
            (Some(_), Some(bucket)) => Some(bucket.t_ms.abs_diff(onset)),
            (None, _) if complete && baseline > 0.0 => Some(0),
            (Some(_), None) | (None, _) => None,
        };
        let recovery_ms = restore.filter(|_| baseline > 0.0).and_then(|restore| {
            buckets
                .iter()
                .find(|b| {
                    b.start_ms() >= restore
                        && b.bps() >= 0.9 * baseline
                        && impact.is_none_or(|impact| b.start_ms() >= impact.t_ms)
                })
                .map(|b| b.t_ms.abs_diff(restore))
        });
        let outage_ms = raw_buckets
            .iter()
            .filter(|b| b.bps() < 0.1 * baseline)
            .map(|b| u64::from(b.duration_ms))
            .sum();
        episodes.push(Episode {
            event_index: i,
            onset_ms: onset,
            restore_ms: restore,
            baseline_bps: baseline,
            horizon_ms: horizon,
            end_ms: end,
            graded: event.graded,
            impacted: impact.is_some(),
            failover_ms,
            recovery_ms,
            outage_ms,
            recovered: failover_ms.is_some() && restore.is_none_or(|_| recovery_ms.is_some()),
            complete,
        });
        states.push(event.clone());
    }
    Ok(episodes)
}
