//! Shared statistics for external telemetry consumers.
//!
//! Exports per-link metrics via the Unix socket `stats` command for external
//! consumers like gst-app's bitrate controller.
//!
//! ## Design Principles
//!
//! 1. **Raw metrics only**: Export values directly from `SrtlaConnection` without
//!    transformation or speculation. Let consumers decide how to interpret them.
//!
//! 2. **Match established formats**: Per-link metrics align with `ConnectionInfo`
//!    (the extended keepalive format sent to srtla_rec).
//!
//! 3. **Include quality_multiplier**: This is the exact value used by enhanced/
//!    rtt-threshold selection algorithms, useful for understanding sender behavior.
//!
//! 4. **Simple aggregates**: Only sums and counts, no derived calculations like
//!    "capacity estimation" that would require assumptions about packet sizes.

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use serde::Serialize;

use crate::bind_map::BindMapReport;
use crate::config::ConfigSnapshot;
use crate::connection::SrtlaConnection;
use crate::sender::calculate_quality_multiplier;
use crate::utils::now_ms;

/// Per-link statistics.
///
/// Fields match `ConnectionInfo` (extended keepalive format) where applicable,
/// plus additional context useful for external monitoring.
#[derive(Clone, Debug, Serialize)]
pub struct LinkStats {
    /// Local IP address used for this link
    pub ip: IpAddr,
    /// Human-readable label (e.g., "host:port via ip")
    pub label: String,
    /// Egress interface this link's socket is bound to (`SO_BINDTODEVICE`).
    /// `None` for a legacy source-IP-bound link — ADDITIVE, never required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iface: Option<String>,
    /// The sidecar's writer-assigned identity for this link (ADR-003), echoed
    /// verbatim. `None` for an unmapped link; the sender never invents one.
    /// ADDITIVE, never required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_id: Option<String>,
    /// True if SRTLA registration completed (REG3 received)
    pub connected: bool,
    /// True if no packets received within timeout period
    pub timed_out: bool,

    // --- Core metrics (same as ConnectionInfo in extended keepalives) ---
    /// Congestion window size (packet count). Higher = more capacity.
    /// This is the primary capacity signal used by all selection modes.
    pub window: i32,
    /// Packets sent but not yet ACKed. Higher = more load on this link.
    pub in_flight: i32,
    /// Smoothed RTT in milliseconds. From Kalman filter.
    pub rtt_ms: u32,
    /// Total NAK count since connection established. Indicates packet loss.
    pub nak_count: i32,
    /// Current send bitrate in bytes/sec (measured, not estimated).
    pub bitrate_bps: u32,
    /// Wire bytes this link has sent for the whole process lifetime (ADR-002).
    /// Survives a socket replacement; restarts only when the process does.
    pub bytes_sent_total: u64,

    // --- RTT baseline tracking ---
    /// Dual-window minimum RTT baseline in milliseconds.
    pub rtt_min_ms: f64,
    /// Kalman RTT velocity in ms/sample (positive = rising, negative = falling).
    pub rtt_velocity: f64,

    // --- Selection algorithm context ---
    /// Base score: window / (in_flight + 1). Used by classic mode.
    /// Higher score = more available capacity on this link.
    pub base_score: i32,
    /// Quality multiplier (0.35 to 1.1) used by enhanced/rtt-threshold modes.
    /// - 1.1 = perfect (no NAKs ever)
    /// - 1.0 = normal
    /// - <1.0 = degraded due to recent NAKs
    /// - ~0.35 = heavily penalized (NAK burst)
    ///
    /// This is the EXACT multiplier used in `select_connection_idx()`.
    /// In classic mode, this is always 1.0 (quality scoring disabled).
    pub quality_multiplier: f64,
}

/// Aggregate statistics snapshot.
#[derive(Clone, Debug, Serialize)]
pub struct StatsSnapshot {
    /// Current scheduling mode: "classic", "enhanced", or "rtt-threshold"
    pub mode: String,
    /// Whether quality scoring is enabled (always false for classic mode)
    pub quality_enabled: bool,
    /// RTT delta threshold in ms (only relevant for rtt-threshold mode)
    pub rtt_delta_ms: u32,

    /// Number of links that are connected AND not timed out
    pub active_links: usize,
    /// Total configured links
    pub total_links: usize,

    /// Sum of window across active links
    pub total_window: i32,
    /// Sum of in_flight across active links
    pub total_in_flight: i32,

    /// Wire bytes the whole bond has sent this session (ADR-002), in **bytes**
    /// — not bits, and not a rate. Monotonic for the process lifetime: it does
    /// not regress when a link reconnects or is dropped by a SIGHUP reload.
    ///
    /// Deliberately NOT `links.map(bytes_sent_total).sum()`: a link torn down by
    /// a reload would take its bytes out of that sum and the operator's "total
    /// transferred" would go backwards.
    pub session_bytes_sent: u64,

    /// The sender's bind-map operating mode (ADR-003 §6.4), flattened to the
    /// top-level `bind_map_status` + `disposition` pair so a consumer reads the
    /// ACTUAL mode from typed data instead of inferring it from log text.
    #[serde(flatten)]
    pub bind_map: BindMapReport,

    /// Per-link details
    pub links: Vec<LinkStats>,
}

impl Default for StatsSnapshot {
    fn default() -> Self {
        Self {
            mode: "enhanced".to_string(),
            quality_enabled: true,
            rtt_delta_ms: 30,
            active_links: 0,
            total_links: 0,
            total_window: 0,
            total_in_flight: 0,
            session_bytes_sent: 0,
            bind_map: BindMapReport::default(),
            links: Vec::new(),
        }
    }
}

/// Monotonic bond-level byte accumulator behind [`StatsSnapshot::session_bytes_sent`].
///
/// Each link's own counter is monotonic but *local* — it disappears when the
/// link is torn down by a SIGHUP reload, and a re-added IP comes back as a fresh
/// connection starting at 0. Summing the live links would therefore make the
/// bond total jump backwards on every reload. Instead this banks each link's
/// **delta** since the last observation, keyed by the connection's stable
/// `conn_id`, so bytes are only ever added.
#[derive(Default)]
struct SessionBytes {
    total: u64,
    last_seen: HashMap<u64, u64>,
}

impl SessionBytes {
    fn observe(&mut self, connections: &[SrtlaConnection]) -> u64 {
        self.observe_totals(
            connections
                .iter()
                .map(|c| (c.conn_id, c.session_bytes_sent())),
        )
    }

    /// Bank each live connection's delta since the last observation, then drop
    /// bookkeeping for connections that are gone — their bytes are already in
    /// `total`, so forgetting them is what keeps the accumulator monotonic
    /// instead of subtracting a departed link back out.
    fn observe_totals(&mut self, totals: impl IntoIterator<Item = (u64, u64)>) -> u64 {
        let mut live = HashSet::with_capacity(self.last_seen.len());
        for (conn_id, link_total) in totals {
            let previous = self.last_seen.insert(conn_id, link_total);
            self.total = self
                .total
                .saturating_add(link_total.saturating_sub(previous.unwrap_or(0)));
            live.insert(conn_id);
        }
        self.last_seen.retain(|conn_id, _| live.contains(conn_id));
        self.total
    }
}

/// Thread-safe container for stats export.
///
/// Updated by sender during housekeeping (~1s interval).
/// Read by config handler when `stats` command is received.
#[derive(Clone, Default)]
pub struct SharedStats {
    inner: Arc<RwLock<StatsSnapshot>>,
    session_bytes: Arc<Mutex<SessionBytes>>,
    bind_map: Arc<RwLock<BindMapReport>>,
}

impl SharedStats {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(StatsSnapshot::default())),
            session_bytes: Arc::new(Mutex::new(SessionBytes::default())),
            bind_map: Arc::new(RwLock::new(BindMapReport::default())),
        }
    }

    /// Record the sender's current bind-map operating mode.
    ///
    /// Held separately from the snapshot because `update` rebuilds the snapshot
    /// wholesale on every housekeeping tick, while the mode only changes on a
    /// reload — folding it into the rebuild would drop it on the next tick.
    pub fn set_bind_map(&self, report: &BindMapReport) {
        if let Ok(mut guard) = self.bind_map.write() {
            *guard = report.clone();
        }
    }

    /// Update stats from current connection state.
    pub fn update(&self, connections: &[SrtlaConnection], config: &ConfigSnapshot) {
        let current_time_ms = now_ms();
        let quality_enabled = config.quality_enabled && !config.mode.is_classic();

        let session_bytes_sent = self
            .session_bytes
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .observe(connections);

        let mut snapshot = StatsSnapshot {
            mode: format!("{}", config.mode),
            quality_enabled,
            rtt_delta_ms: config.rtt_delta_ms,
            total_links: connections.len(),
            session_bytes_sent,
            ..Default::default()
        };

        for conn in connections {
            let timed_out = conn.is_timed_out();
            let is_active = conn.connected && !timed_out;

            // Quality multiplier: use actual selection algorithm calculation,
            // or 1.0 if quality scoring is disabled (classic mode)
            let quality_multiplier = if quality_enabled {
                calculate_quality_multiplier(conn, current_time_ms)
            } else {
                1.0
            };

            let link = LinkStats {
                ip: conn.local_ip,
                label: conn.label.clone(),
                iface: conn.egress.iface().map(|i| i.as_str().to_string()),
                link_id: conn.link_id.as_ref().map(|id| id.as_str().to_string()),
                connected: conn.connected,
                timed_out,
                window: conn.window,
                in_flight: conn.in_flight_packets,
                rtt_ms: conn.get_smooth_rtt_ms() as u32,
                nak_count: conn.total_nak_count(),
                bitrate_bps: (conn.current_bitrate_mbps() * 1_000_000.0 / 8.0) as u32,
                bytes_sent_total: conn.session_bytes_sent(),
                rtt_min_ms: conn.get_rtt_min_ms(),
                rtt_velocity: conn.get_rtt_velocity(),
                base_score: conn.get_score(),
                quality_multiplier,
            };

            if is_active {
                snapshot.active_links += 1;
                snapshot.total_window += conn.window;
                snapshot.total_in_flight += conn.in_flight_packets;
            }

            snapshot.links.push(link);
        }

        if let Ok(mut guard) = self.inner.write() {
            *guard = snapshot;
        }
    }

    /// Get current stats snapshot.
    ///
    /// The bind-map operating mode is composed in here rather than baked into
    /// the stored snapshot: `update` rebuilds that snapshot on every
    /// housekeeping tick while the mode changes only on a reload, so this is
    /// the single point where the two cadences meet.
    pub fn get(&self) -> StatsSnapshot {
        let mut snapshot = self
            .inner
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default();
        snapshot.bind_map = self
            .bind_map
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default();
        snapshot
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.get()).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mode::SchedulingMode;

    #[test]
    fn test_shared_stats_new() {
        let stats = SharedStats::new();
        let snapshot = stats.get();
        assert_eq!(snapshot.active_links, 0);
        assert_eq!(snapshot.total_links, 0);
    }

    #[test]
    fn test_shared_stats_empty_update() {
        let stats = SharedStats::new();
        let config = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: true,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        };
        stats.update(&[], &config);
        let snapshot = stats.get();
        assert_eq!(snapshot.mode, "enhanced");
        assert!(snapshot.quality_enabled);
    }

    #[test]
    fn test_to_json_contains_expected_fields() {
        let stats = SharedStats::new();
        let json = stats.to_json();
        assert!(json.contains("\"mode\""));
        assert!(json.contains("\"active_links\""));
        assert!(json.contains("\"total_window\""));
        assert!(json.contains("\"links\""));
        assert!(json.contains("\"session_bytes_sent\""));
    }

    // ---- ADR-002 session-bytes accumulator --------------------------------

    #[test]
    fn session_bytes_sums_live_links() {
        let mut acc = SessionBytes::default();
        assert_eq!(acc.observe_totals([(7, 1_000), (9, 500)]), 1_500);
    }

    #[test]
    fn session_bytes_banks_only_the_delta_between_observations() {
        let mut acc = SessionBytes::default();
        acc.observe_totals([(7, 1_000)]);
        assert_eq!(
            acc.observe_totals([(7, 1_600)]),
            1_600,
            "a link's own counter is cumulative, so re-observing it must add 600, not 1600"
        );
    }

    #[test]
    fn session_bytes_survives_a_link_teardown() {
        // A SIGHUP reload that drops an uplink must not take its bytes with it —
        // this is the regression a naive `links.map(total).sum()` would ship.
        let mut acc = SessionBytes::default();
        acc.observe_totals([(7, 1_000), (9, 500)]);

        assert_eq!(acc.observe_totals([(7, 1_000)]), 1_500);
    }

    #[test]
    fn session_bytes_counts_a_readded_link_from_zero() {
        // A re-added IP comes back as a NEW connection (fresh conn_id, counter at
        // 0). Its bytes must accrue on top of the banked total, never replace it.
        let mut acc = SessionBytes::default();
        acc.observe_totals([(7, 1_000)]);
        acc.observe_totals([]);

        assert_eq!(acc.observe_totals([(11, 300)]), 1_300);
    }

    #[test]
    fn session_bytes_never_regresses_on_a_backwards_link_counter() {
        // Per-link counters are monotonic by construction; if one ever went
        // backwards the bond total must still refuse to shrink.
        let mut acc = SessionBytes::default();
        acc.observe_totals([(7, 1_000)]);

        assert_eq!(acc.observe_totals([(7, 400)]), 1_000);
    }

    #[test]
    fn session_bytes_forgets_departed_links() {
        // Bookkeeping for a link that is gone must not accumulate across a long
        // session of SIGHUP churn.
        let mut acc = SessionBytes::default();
        acc.observe_totals([(1, 10), (2, 10), (3, 10)]);
        acc.observe_totals([(3, 10)]);

        assert_eq!(acc.last_seen.len(), 1);
        assert_eq!(acc.total, 30);
    }

    #[test]
    fn empty_update_reports_zero_session_bytes() {
        let stats = SharedStats::new();
        stats.update(&[], &test_config());
        assert_eq!(stats.get().session_bytes_sent, 0);
    }

    fn test_config() -> ConfigSnapshot {
        ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: true,
            exploration_enabled: false,
            rtt_delta_ms: 30,
            earned_ack_window: false,
            stall_deselect: false,
            stall_min_in_flight: 32,
            stall_ack_stale_ms: 3000,
            stall_reprobe_ms: 1000,
        }
    }
}
