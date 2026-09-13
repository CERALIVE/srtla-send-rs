use serde::{Deserialize, Serialize};

use super::error::delta;
use super::srt_stats::SrtStats;
use super::{MetricError, Window, rate_number};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SrtWindow {
    pub pkt_recv_total: u64,
    pub pkt_recv_unique: u64,
    pub pkt_loss_total: u64,
    pub pkt_drop_total: u64,
    pub pkt_retrans_total: u64,
    pub byte_recv: u64,
    pub pkt_belated_sum: u64,
    pub viewer_loss_ratio: f64,
    pub no_traffic: bool,
    pub loss_ratio: f64,
    pub retrans_ratio: f64,
    pub reorder_distance_max: Option<u64>,
    pub ms_rtt_median: Option<f64>,
    pub mbps_recv_rate_mean: Option<f64>,
}

impl SrtStats {
    /// Uses (start, end] for interval rows, and last <= edge for cumulative counters.
    pub fn window(&self, window: Window, sink_bytes: u64) -> Result<SrtWindow, MetricError> {
        let first = self
            .rows
            .iter()
            .rposition(|row| row.t_ms <= window.start_ms())
            .ok_or(MetricError::MissingBaseline)?;
        let last = self
            .rows
            .iter()
            .rposition(|row| row.t_ms <= window.end_ms())
            .ok_or(MetricError::MissingBaseline)?;
        let start = &self.rows[first];
        let end = &self.rows[last];
        for pair in self.rows[first..=last].windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            if a.socket_id != b.socket_id {
                return Err(MetricError::CounterReset("SocketID".into()));
            }
            for (before, after, name) in [
                (a.pkt_recv_total, b.pkt_recv_total, "pktRecv"),
                (a.pkt_recv_unique, b.pkt_recv_unique, "pktRecvUnique"),
                (a.pkt_loss_total, b.pkt_loss_total, "pktRcvLoss"),
                (a.pkt_drop_total, b.pkt_drop_total, "pktRcvDrop"),
                (a.pkt_retrans_total, b.pkt_retrans_total, "pktRcvRetrans"),
                (a.byte_recv, b.byte_recv, "byteRecv"),
            ] {
                delta(after, before, name)?;
            }
        }
        let received = delta(end.pkt_recv_total, start.pkt_recv_total, "pktRecv")?;
        let unique = delta(end.pkt_recv_unique, start.pkt_recv_unique, "pktRecvUnique")?;
        let dropped = delta(end.pkt_drop_total, start.pkt_drop_total, "pktRcvDrop")?;
        let loss = delta(end.pkt_loss_total, start.pkt_loss_total, "pktRcvLoss")?;
        let retrans = delta(
            end.pkt_retrans_total,
            start.pkt_retrans_total,
            "pktRcvRetrans",
        )?;
        let rows: Vec<_> = self
            .rows
            .iter()
            .filter(|row| window.contains(row.t_ms))
            .collect();
        let mut rtts: Vec<_> = rows.iter().map(|row| row.ms_rtt).collect();
        rtts.sort_by(f64::total_cmp);
        let median = match rtts.len() {
            0 => None,
            n if n % 2 == 0 => Some(rtts[n / 2 - 1] / 2.0 + rtts[n / 2] / 2.0),
            n => Some(rtts[n / 2]),
        };
        let denominator = rate_number(unique) + rate_number(dropped);
        let ratio = |n, d: f64| if d > 0.0 { rate_number(n) / d } else { 0.0 };
        let count =
            u64::try_from(rows.len()).map_err(|_| MetricError::InvalidField("row count".into()))?;
        let belated = rows
            .iter()
            .try_fold(0_u64, |sum, row| sum.checked_add(row.pkt_belated))
            .ok_or_else(|| MetricError::InvalidField("pktRcvBelated sum".into()))?;
        Ok(SrtWindow {
            pkt_recv_total: received,
            pkt_recv_unique: unique,
            pkt_loss_total: loss,
            pkt_drop_total: dropped,
            pkt_retrans_total: retrans,
            byte_recv: delta(end.byte_recv, start.byte_recv, "byteRecv")?,
            pkt_belated_sum: belated,
            viewer_loss_ratio: ratio(dropped, denominator),
            no_traffic: denominator == 0.0 && sink_bytes == 0,
            loss_ratio: ratio(loss, rate_number(received) + rate_number(loss)),
            retrans_ratio: ratio(retrans, rate_number(received)),
            reorder_distance_max: rows.iter().filter_map(|row| row.reorder_distance).max(),
            ms_rtt_median: median,
            mbps_recv_rate_mean: (count > 0).then(|| {
                rows.iter()
                    .map(|row| row.mbps_recv_rate / rate_number(count))
                    .sum()
            }),
        })
    }
}
