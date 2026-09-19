use serde::{Deserialize, Serialize};

use super::error::{delta, number};
use super::{MetricError, Window, rate_number};
use crate::Namespace;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkCounterSample {
    pub t_ms: i64,
    pub iface: String,
    pub ifindex: u32,
    pub tx_bytes: u64,
    pub tx_packets: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkShare {
    pub iface: String,
    pub tx_bytes: u64,
    pub tx_packets: u64,
    pub share: f64,
}

pub fn sample(ns: Option<&Namespace>, iface: &str, t_ms: i64) -> anyhow::Result<LinkCounterSample> {
    anyhow::ensure!(
        !iface.is_empty()
            && iface.len() < 16
            && iface
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"_.:-".contains(&c))
            && iface != "."
            && iface != "..",
        MetricError::InvalidField("iface".into())
    );
    let read = |field: &str| -> anyhow::Result<String> {
        let path = format!("/sys/class/net/{iface}/{field}");
        match ns {
            Some(ns) => Ok(String::from_utf8(ns.exec_checked("cat", &[&path])?.stdout)?),
            None => Ok(std::fs::read_to_string(path)?),
        }
    };
    let ifindex = number(&read("ifindex")?, "ifindex")?;
    let tx_bytes = number(&read("statistics/tx_bytes")?, "tx_bytes")?;
    let tx_packets = number(&read("statistics/tx_packets")?, "tx_packets")?;
    anyhow::ensure!(
        ifindex == number::<u32>(&read("ifindex")?, "ifindex")?,
        "interface replaced during sample"
    );
    Ok(LinkCounterSample {
        t_ms,
        iface: iface.into(),
        ifindex,
        tx_bytes,
        tx_packets,
    })
}

/// A recreated netdev starts at zero. Bank each new generation instead of subtracting it.
pub fn shares(
    samples: &[LinkCounterSample],
    window: Window,
) -> Result<Vec<LinkShare>, MetricError> {
    let mut by_iface = std::collections::BTreeMap::<&str, Vec<&LinkCounterSample>>::new();
    for sample in samples {
        by_iface.entry(&sample.iface).or_default().push(sample);
    }
    let mut result = Vec::new();
    for (iface, rows) in by_iface {
        if rows.windows(2).any(|pair| pair[0].t_ms >= pair[1].t_ms) {
            return Err(MetricError::IncompleteSeries);
        }
        let start = rows
            .iter()
            .rposition(|s| s.t_ms <= window.start_ms())
            .ok_or(MetricError::MissingBaseline)?;
        let mut bytes = 0_u64;
        let mut packets = 0_u64;
        for pair in rows[start..]
            .windows(2)
            .take_while(|pair| pair[1].t_ms <= window.end_ms())
        {
            let (a, b) = (pair[0], pair[1]);
            let (db, dp) = if a.ifindex == b.ifindex {
                (
                    delta(b.tx_bytes, a.tx_bytes, "tx_bytes")?,
                    delta(b.tx_packets, a.tx_packets, "tx_packets")?,
                )
            } else {
                (b.tx_bytes, b.tx_packets)
            };
            bytes = bytes
                .checked_add(db)
                .ok_or_else(|| MetricError::InvalidField("tx_bytes sum".into()))?;
            packets = packets
                .checked_add(dp)
                .ok_or_else(|| MetricError::InvalidField("tx_packets sum".into()))?;
        }
        result.push(LinkShare {
            iface: iface.into(),
            tx_bytes: bytes,
            tx_packets: packets,
            share: 0.0,
        });
    }
    let total: f64 = result.iter().map(|r| rate_number(r.tx_bytes)).sum();
    for link in &mut result {
        link.share = if total > 0.0 {
            rate_number(link.tx_bytes) / total
        } else {
            0.0
        };
    }
    Ok(result)
}
