use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Stats {
    #[serde(rename = "pktRcvDropTotal")]
    pub dropped: u64,
    #[serde(rename = "pktRcvBelated")]
    pub belated: u64,
    #[serde(rename = "pktRcvRetransTotal")]
    pub retransmitted: u64,
    #[serde(rename = "pktRecvUniqueTotal")]
    pub unique: u64,
}

impl Stats {
    pub fn read(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let mut lines = text.lines();
        let header: Vec<_> = lines
            .next()
            .context("stats CSV header")?
            .split(',')
            .collect();
        let rows: Vec<_> = lines.filter(|line| !line.is_empty()).collect();
        let last: Vec<_> = rows.last().context("stats CSV rows")?.split(',').collect();
        let field = |name: &str| -> Result<u64> {
            let idx = header
                .iter()
                .position(|v| *v == name)
                .with_context(|| format!("CSV field {name}"))?;
            last.get(idx)
                .context("truncated CSV row")?
                .parse()
                .context("CSV counter")
        };
        // -fullstats uses srt_bstats(clear=false): even local counters accumulate.
        // Every run has a new socket (zero baseline); CSV omits the Total suffix.
        Ok(Self {
            dropped: field("pktRcvDrop")?,
            belated: field("pktRcvBelated")?,
            retransmitted: field("pktRcvRetrans")?,
            unique: field("pktRecvUnique")?,
        })
    }
}

#[derive(Serialize)]
pub struct RunResult {
    pub variant: String,
    pub duplicate_sequences: usize,
    pub unique_sequences: usize,
    pub retransmit_flags: usize,
    pub unregistered_ingress: usize,
    pub bytes_equal: bool,
    pub source_bytes: u64,
    pub sink_bytes: u64,
    pub stats: Stats,
}

pub const fn wire_form(clear: &Stats, set: &Stats) -> &'static str {
    if (set.belated as u128 + set.retransmitted as u128)
        < (clear.belated as u128 + clear.retransmitted as u128)
    {
        "set"
    } else {
        "clear"
    }
}

#[test]
fn wire_form_minimizes_total_side_effect_events_and_breaks_ties_clear() {
    // Given counters in which fewer belated events cost many retransmit events.
    let clear = Stats {
        dropped: 0,
        belated: 104,
        retransmitted: 0,
        unique: 3000,
    };
    let set = Stats {
        dropped: 0,
        belated: 81,
        retransmitted: 300,
        unique: 3000,
    };
    // When choosing the smaller receiver-side effect, then count both kinds equally.
    assert_eq!(wire_form(&clear, &set), "clear");
    assert_eq!(wire_form(&set, &clear), "set");
    assert_eq!(wire_form(&clear, &clear), "clear");
}

pub fn packets(path: &Path) -> Result<Vec<Vec<u8>>> {
    let bytes = std::fs::read(path)?;
    ensure!(bytes.len() >= 24, "missing pcap header: {}", path.display());
    let little = match &bytes[..4] {
        [0xd4, 0xc3, 0xb2, 0xa1] => true,
        [0xa1, 0xb2, 0xc3, 0xd4] => false,
        _ => anyhow::bail!("unsupported pcap magic"),
    };
    let word = |raw: &[u8]| -> u32 {
        let raw = raw.try_into().unwrap();
        if little {
            u32::from_le_bytes(raw)
        } else {
            u32::from_be_bytes(raw)
        }
    };
    ensure!(
        word(&bytes[20..24]) == 1,
        "capture must be Ethernet (lo or veth, not any)"
    );
    let mut result = Vec::new();
    let mut offset = 24;
    while offset + 16 <= bytes.len() {
        let size = usize::try_from(word(&bytes[offset + 8..offset + 12]))?;
        offset += 16;
        // A live -U capture may be read between its record header and payload.
        if offset + size > bytes.len() {
            break;
        }
        let frame = &bytes[offset..offset + size];
        offset += size;
        ensure!(
            frame.len() >= 42 && frame[12..14] == [8, 0],
            "expected IPv4 Ethernet"
        );
        let ip = &frame[14..];
        let ihl = usize::from(ip[0] & 15) * 4;
        ensure!(ip[9] == 17 && ip.len() >= ihl + 8, "expected UDP");
        let udp = &ip[ihl..];
        let len = usize::from(u16::from_be_bytes([udp[4], udp[5]]));
        ensure!(len >= 24 && udp.len() >= len, "truncated SRT datagram");
        let payload = &udp[8..len];
        ensure!(payload[0] & 0x80 == 0, "capture filter admitted a control");
        result.push(payload.to_vec());
    }
    Ok(result)
}
