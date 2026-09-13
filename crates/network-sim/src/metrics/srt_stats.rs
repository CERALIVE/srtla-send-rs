use serde::{Deserialize, Serialize};

use super::MetricError;
use super::error::number;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureSemantics {
    #[default]
    CumulativePacketsIntervalBelated,
    Interval,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SrtRow {
    pub t_ms: i64,
    pub socket_id: u64,
    pub pkt_recv_total: u64,
    pub pkt_recv_unique: u64,
    pub pkt_loss_total: u64,
    pub pkt_drop_total: u64,
    pub pkt_retrans_total: u64,
    pub pkt_belated: u64,
    pub byte_recv: u64,
    pub reorder_distance: Option<u64>,
    pub ms_rtt: f64,
    pub mbps_recv_rate: f64,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SrtStats {
    pub capture_semantics: CaptureSemantics,
    pub header: Vec<String>,
    pub clock_offset_ms: i64,
    pub rows: Vec<SrtRow>,
}

impl SrtStats {
    /// Parses libsrt's unquoted numeric CSV. Offset aligns socket-relative Time to the run clock.
    /// Packet counters must be cumulative and belated interval-valued for `window`.
    pub fn parse(csv: &str, clock_offset_ms: i64) -> Result<Self, MetricError> {
        let mut lines = csv.lines().filter(|line| !line.trim().is_empty());
        let header: Vec<String> = lines
            .next()
            .ok_or_else(|| MetricError::MissingColumn("Time".into()))?
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect();
        let position = |name: &str| {
            header
                .iter()
                .position(|h| h == name)
                .ok_or_else(|| MetricError::MissingColumn(name.into()))
        };
        let time = header
            .iter()
            .enumerate()
            .filter(|(_, h)| h.as_str() == "Time")
            .nth(1)
            .map(|(i, _)| i)
            .ok_or_else(|| MetricError::MissingColumn("second Time".into()))?;
        let names = [
            "SocketID",
            "pktRecv",
            "pktRecvUnique",
            "pktRcvLoss",
            "pktRcvDrop",
            "pktRcvRetrans",
            "pktRcvBelated",
            "byteRecv",
            "msRTT",
            "mbpsRecvRate",
        ];
        let positions = names
            .iter()
            .map(|name| position(name))
            .collect::<Result<Vec<_>, _>>()?;
        let reorder = header.iter().position(|h| h == "pktReorderDistance");
        let mut rows: Vec<SrtRow> = Vec::new();
        for raw in lines {
            let fields: Vec<_> = raw.split(',').collect();
            if fields.len() != header.len() {
                return Err(MetricError::InvalidField("CSV row width".into()));
            }
            let count = |n: usize| number::<u64>(fields[positions[n]], names[n]);
            let gauge = |n: usize| -> Result<f64, MetricError> {
                let value = number::<f64>(fields[positions[n]], names[n])?;
                if value.is_finite() && value >= 0.0 {
                    Ok(value)
                } else {
                    Err(MetricError::InvalidField(names[n].into()))
                }
            };
            let t_ms = number::<i64>(fields[time], "Time")?
                .checked_add(clock_offset_ms)
                .ok_or_else(|| MetricError::InvalidField("Time offset".into()))?;
            if rows.last().is_some_and(|row| row.t_ms > t_ms) {
                return Err(MetricError::InvalidField("Time ordering".into()));
            }
            rows.push(SrtRow {
                t_ms,
                socket_id: count(0)?,
                pkt_recv_total: count(1)?,
                pkt_recv_unique: count(2)?,
                pkt_loss_total: count(3)?,
                pkt_drop_total: count(4)?,
                pkt_retrans_total: count(5)?,
                pkt_belated: count(6)?,
                byte_recv: count(7)?,
                reorder_distance: reorder
                    .map(|i| number(fields[i], "pktReorderDistance"))
                    .transpose()?,
                ms_rtt: gauge(8)?,
                mbps_recv_rate: gauge(9)?,
                raw: raw.into(),
            });
        }
        Ok(Self {
            capture_semantics: CaptureSemantics::CumulativePacketsIntervalBelated,
            header,
            clock_offset_ms,
            rows,
        })
    }

    pub fn parse_with_semantics(
        csv: &str,
        clock_offset_ms: i64,
        semantics: CaptureSemantics,
    ) -> Result<Self, MetricError> {
        let mut stats = Self::parse(csv, clock_offset_ms)?;
        match semantics {
            CaptureSemantics::CumulativePacketsIntervalBelated => {}
            CaptureSemantics::Interval => {
                let mut totals = [0_u64; 6];
                for row in &mut stats.rows {
                    for (total, count) in totals.iter_mut().zip([
                        &mut row.pkt_recv_total,
                        &mut row.pkt_recv_unique,
                        &mut row.pkt_loss_total,
                        &mut row.pkt_drop_total,
                        &mut row.pkt_retrans_total,
                        &mut row.byte_recv,
                    ]) {
                        *total = total.checked_add(*count).ok_or_else(|| {
                            MetricError::InvalidField("interval normalization overflow".into())
                        })?;
                        *count = *total;
                    }
                }
            }
        }
        stats.capture_semantics = semantics;
        Ok(stats)
    }
}
