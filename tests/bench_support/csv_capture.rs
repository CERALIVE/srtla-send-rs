use std::collections::BTreeMap;
use std::path::Path;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use anyhow::Result;
use network_sim::metrics::MetricError;
use network_sim::metrics::srt_stats::SrtStats;
use serde::Serialize;

use super::clock::{Clock, CsvClock, complete_csv};

#[derive(Debug, Serialize)]
pub struct Calibration {
    offset_ms: i64,
    uncertainty_ms: i64,
}

pub struct Capture {
    pub stats: SrtStats,
    pub clocks: BTreeMap<u64, Calibration>,
}

struct Clocks {
    initial_offset_ms: i64,
    sockets: BTreeMap<u64, Calibration>,
}

impl Capture {
    pub fn run(path: &Path, timing: (&Clock, &CsvClock), stop: Receiver<()>) -> Result<Self> {
        let (clock, initial) = timing;
        let mut clocks = Clocks {
            initial_offset_ms: initial.offset_ms,
            sockets: BTreeMap::from([(
                initial.socket_id,
                Calibration {
                    offset_ms: initial.offset_ms,
                    uncertainty_ms: initial.uncertainty_ms,
                },
            )]),
        };
        let mut before = clock.now_ms();
        loop {
            let csv = complete_csv(path)?;
            let after = clock.now_ms();
            let stats = clocks.align(&csv, (before, after))?;
            before = after;
            match stop.recv_timeout(Duration::from_millis(10)) {
                Ok(()) | Err(RecvTimeoutError::Disconnected) => {
                    return Ok(Self {
                        stats,
                        clocks: clocks.sockets,
                    });
                }
                Err(RecvTimeoutError::Timeout) => {}
            }
        }
    }
}

impl Clocks {
    fn align(&mut self, csv: &str, bracket: (i64, i64)) -> Result<SrtStats, MetricError> {
        let (before, after) = bracket;
        SrtStats::parse_intervals_with_clock(csv, self.initial_offset_ms, |socket, time| {
            let calibration = self.sockets.entry(socket).or_insert_with(|| Calibration {
                offset_ms: before + (after - before) / 2 - time,
                uncertainty_ms: after - before,
            });
            time.checked_add(calibration.offset_ms)
                .ok_or_else(|| MetricError::InvalidField("Time offset".into()))
        })
    }
}

#[test]
fn reconnect_clock_is_bracketed_once_without_compressing_the_gap() {
    // Given one calibrated socket and a first flushed row from its replacement.
    let header = "Time,SocketID,Time,pktRecv,pktRecvUnique,pktRcvLoss,pktRcvDrop,pktRcvRetrans,\
                  pktRcvBelated,byteRecv,msRTT,mbpsRecvRate\n";
    let csv = format!(
        "{header}1000,11,1000,10,10,0,0,0,0,13160,60,1\n500,12,500,20,20,0,0,0,0,26320,60,1\n"
    );
    let mut clocks = Clocks {
        initial_offset_ms: 0,
        sockets: BTreeMap::from([(
            11,
            Calibration {
                offset_ms: 0,
                uncertainty_ms: 10,
            },
        )]),
    };
    // When the replacement row arrives inside a monotonic bracket 18 seconds later.
    let stats = clocks.align(&csv, (19000, 19020)).unwrap();
    // Then the original socket stays fixed and the new socket retains its real outage gap.
    assert_eq!(
        stats.rows.iter().map(|r| r.t_ms).collect::<Vec<_>>(),
        [1000, 19010]
    );
    assert_eq!(clocks.sockets[&12].uncertainty_ms, 20);
    assert_eq!(clocks.sockets[&12].offset_ms, 18510);
    assert_eq!(clocks.align(&csv, (20000, 20020)).unwrap(), stats);
}
