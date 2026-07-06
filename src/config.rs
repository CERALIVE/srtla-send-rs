//! Runtime configuration for SRTLA sender.
//!
//! Manages dynamic settings that can be changed at runtime via stdin or Unix socket.

#[cfg(unix)]
use std::io::Write;
use std::io::{BufRead, BufReader};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

#[cfg(unix)]
use tracing::debug;
use tracing::{info, warn};

use crate::mode::SchedulingMode;
use crate::stats::SharedStats;
use crate::subscription::SubscriptionManager;

/// Default RTT delta threshold in milliseconds.
/// Links within min_rtt + delta are considered "fast" and preferred.
pub const DEFAULT_RTT_DELTA_MS: u32 = 30;

/// Min interval (ms) between rate-limited PROBE window `+1` steps on a
/// non-earning link under the EXPERIMENTAL `earned_ack_window` valve
/// (`SrtlaConnection::handle_srtla_ack_earned`). Inert while the flag is off
/// (default). Hypothesis-only; NOT validated on real bond hardware.
pub const PROBE_GROWTH_INTERVAL_MS: u64 = 1000;

/// Snapshot of configuration for efficient hot-path access.
/// Call `DynamicConfig::snapshot()` once per select iteration to avoid
/// multiple atomic loads per packet in the hot path.
#[derive(Clone, Copy, Debug)]
pub struct ConfigSnapshot {
    pub mode: SchedulingMode,
    pub quality_enabled: bool,
    pub exploration_enabled: bool,
    pub rtt_delta_ms: u32,
    /// EXPERIMENTAL earned-ACK window valve (default OFF). Selects
    /// `handle_srtla_ack_earned` over `handle_srtla_ack_global` at the SRTLA-ACK
    /// call site; off ⇒ baseline behavior unchanged.
    pub earned_ack_window: bool,
}

impl ConfigSnapshot {
    /// Check if quality scoring is effective for the current mode.
    /// Quality scoring only applies to enhanced and rtt-threshold modes.
    #[inline]
    pub fn effective_quality_enabled(&self) -> bool {
        self.quality_enabled && !self.mode.is_classic()
    }

    /// Check if exploration is effective for the current mode.
    /// Exploration only applies to enhanced mode.
    #[inline]
    pub fn effective_exploration_enabled(&self) -> bool {
        self.exploration_enabled && self.mode.is_enhanced()
    }
}

/// Dynamic configuration that can be modified at runtime.
/// Uses atomic types for lock-free concurrent access.
#[derive(Clone)]
pub struct DynamicConfig {
    mode: Arc<AtomicU8>,
    quality_enabled: Arc<AtomicBool>,
    exploration_enabled: Arc<AtomicBool>,
    rtt_delta_ms: Arc<AtomicU32>,
    earned_ack_window: Arc<AtomicBool>,
}

impl Default for DynamicConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicConfig {
    pub fn new() -> Self {
        Self {
            mode: Arc::new(AtomicU8::new(SchedulingMode::Enhanced.as_u8())),
            quality_enabled: Arc::new(AtomicBool::new(true)),
            exploration_enabled: Arc::new(AtomicBool::new(false)),
            rtt_delta_ms: Arc::new(AtomicU32::new(DEFAULT_RTT_DELTA_MS)),
            earned_ack_window: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Create config from CLI arguments.
    pub fn from_cli(
        mode: SchedulingMode,
        no_quality: bool,
        exploration: bool,
        rtt_delta_ms: u32,
        earned_ack_window: bool,
    ) -> Self {
        Self {
            mode: Arc::new(AtomicU8::new(mode.as_u8())),
            quality_enabled: Arc::new(AtomicBool::new(!no_quality)),
            exploration_enabled: Arc::new(AtomicBool::new(exploration)),
            rtt_delta_ms: Arc::new(AtomicU32::new(rtt_delta_ms)),
            earned_ack_window: Arc::new(AtomicBool::new(earned_ack_window)),
        }
    }

    /// Create a snapshot of current configuration.
    /// Call this once at the start of each select iteration to avoid
    /// multiple atomic loads per packet in the hot path.
    #[inline]
    pub fn snapshot(&self) -> ConfigSnapshot {
        ConfigSnapshot {
            mode: SchedulingMode::from_u8(self.mode.load(Ordering::Acquire)),
            quality_enabled: self.quality_enabled.load(Ordering::Acquire),
            exploration_enabled: self.exploration_enabled.load(Ordering::Acquire),
            rtt_delta_ms: self.rtt_delta_ms.load(Ordering::Acquire),
            earned_ack_window: self.earned_ack_window.load(Ordering::Acquire),
        }
    }

    /// Get the current scheduling mode.
    #[inline]
    pub fn mode(&self) -> SchedulingMode {
        SchedulingMode::from_u8(self.mode.load(Ordering::Acquire))
    }

    /// Set the scheduling mode.
    pub fn set_mode(&self, mode: SchedulingMode) {
        self.mode.store(mode.as_u8(), Ordering::Release);
    }

    /// Set whether quality scoring is enabled.
    pub fn set_quality_enabled(&self, enabled: bool) {
        self.quality_enabled.store(enabled, Ordering::Release);
    }

    /// Set whether exploration is enabled.
    pub fn set_exploration_enabled(&self, enabled: bool) {
        self.exploration_enabled.store(enabled, Ordering::Release);
    }

    /// Set the RTT delta threshold in milliseconds.
    pub fn set_rtt_delta_ms(&self, delta: u32) {
        self.rtt_delta_ms.store(delta, Ordering::Release);
    }

    /// Toggle the EXPERIMENTAL earned-ACK window valve (default OFF).
    pub fn set_earned_ack_window(&self, enabled: bool) {
        self.earned_ack_window.store(enabled, Ordering::Release);
    }
}

pub fn spawn_config_listener(
    config: DynamicConfig,
    socket_path: Option<String>,
    stats: SharedStats,
    subscriptions: SubscriptionManager,
) {
    if let Some(sock_path) = socket_path {
        // Socket path specified: use Unix socket on Unix, fallback to stdin on other platforms
        #[cfg(unix)]
        {
            let config_clone = config.clone();
            let stats_clone = stats.clone();
            let subscriptions_clone = subscriptions.clone();
            std::thread::spawn(move || {
                unix_socket_loop(
                    &config_clone,
                    &sock_path,
                    &stats_clone,
                    &subscriptions_clone,
                );
            });
        }
        #[cfg(not(unix))]
        {
            // Unix sockets not available; fall back to stdin listener
            let _ = sock_path; // suppress unused warning
            let _ = stats; // suppress unused warning
            let _ = subscriptions; // event subscription is Unix-socket only
            let config_clone = config.clone();
            std::thread::spawn(move || {
                let stdin = std::io::stdin();
                let reader = BufReader::new(stdin);
                for cmd in reader.lines().map_while(Result::ok) {
                    apply_cmd(&config_clone, cmd.trim(), None);
                }
            });
        }
    } else {
        let _ = &subscriptions; // stdin control path has no event subscribers
        // No socket path: use stdin listener (backward compatibility)
        let config_clone = config.clone();
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let reader = BufReader::new(stdin);
            for cmd in reader.lines().map_while(Result::ok) {
                apply_cmd(&config_clone, cmd.trim(), None);
            }
        });
    }
}

/// Response from apply_cmd that can be sent back to the client.
#[allow(dead_code)] // Json variant's inner value is read in #[cfg(unix)] code
pub enum CmdResponse {
    /// No response needed (command logged via tracing)
    None,
    /// JSON response to send back
    Json(String),
}

/// Apply a runtime command to the configuration.
///
/// Commands:
/// - `mode classic|enhanced|rtt-threshold|edpf` - switch scheduling mode
/// - `quality on|off` - toggle quality scoring
/// - `explore on|off` - toggle exploration
/// - `rtt-delta <ms>` - set RTT delta threshold
/// - `earned-ack on|off` - toggle the EXPERIMENTAL earned-ACK window valve
/// - `status` - show current configuration
/// - `stats` - get per-link telemetry as JSON
pub fn apply_cmd(config: &DynamicConfig, cmd: &str, stats: Option<&SharedStats>) -> CmdResponse {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return CmdResponse::None;
    }

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return CmdResponse::None;
    }

    match parts[0] {
        "mode" => {
            if parts.len() != 2 {
                warn!("usage: mode classic|enhanced|rtt-threshold|edpf");
                return CmdResponse::None;
            }
            match parts[1] {
                "classic" => {
                    config.set_mode(SchedulingMode::Classic);
                    info!("mode: classic");
                }
                "enhanced" => {
                    config.set_mode(SchedulingMode::Enhanced);
                    info!("mode: enhanced");
                }
                "rtt-threshold" => {
                    config.set_mode(SchedulingMode::RttThreshold);
                    info!("mode: rtt-threshold");
                }
                "edpf" => {
                    config.set_mode(SchedulingMode::Edpf);
                    info!("mode: edpf");
                }
                other => {
                    warn!(
                        "unknown mode '{}': use classic, enhanced, rtt-threshold, or edpf",
                        other
                    );
                }
            }
        }

        "quality" => {
            if parts.len() != 2 {
                warn!("usage: quality on|off");
                return CmdResponse::None;
            }
            match parts[1] {
                "on" => {
                    config.set_quality_enabled(true);
                    info!("quality: on");
                }
                "off" => {
                    config.set_quality_enabled(false);
                    info!("quality: off");
                }
                other => {
                    warn!("invalid value '{}': use on or off", other);
                }
            }
        }

        "explore" => {
            if parts.len() != 2 {
                warn!("usage: explore on|off");
                return CmdResponse::None;
            }
            match parts[1] {
                "on" => {
                    config.set_exploration_enabled(true);
                    info!("explore: on");
                }
                "off" => {
                    config.set_exploration_enabled(false);
                    info!("explore: off");
                }
                other => {
                    warn!("invalid value '{}': use on or off", other);
                }
            }
        }

        "rtt-delta" => {
            if parts.len() != 2 {
                warn!("usage: rtt-delta <ms>");
                return CmdResponse::None;
            }
            match parts[1].parse::<u32>() {
                Ok(delta) => {
                    config.set_rtt_delta_ms(delta);
                    info!("rtt-delta: {}ms", delta);
                }
                Err(_) => {
                    warn!("invalid rtt-delta value: {}", parts[1]);
                }
            }
        }

        "earned-ack" => {
            if parts.len() != 2 {
                warn!("usage: earned-ack on|off");
                return CmdResponse::None;
            }
            match parts[1] {
                "on" => {
                    config.set_earned_ack_window(true);
                    info!("earned-ack: on (EXPERIMENTAL)");
                }
                "off" => {
                    config.set_earned_ack_window(false);
                    info!("earned-ack: off");
                }
                other => {
                    warn!("invalid value '{}': use on or off", other);
                }
            }
        }

        "status" => {
            let snap = config.snapshot();
            info!("mode: {}", snap.mode);
            info!(
                "  quality: {}",
                if snap.quality_enabled { "on" } else { "off" }
            );
            info!(
                "  explore: {}",
                if snap.exploration_enabled {
                    "on"
                } else {
                    "off"
                }
            );
            info!("  rtt-delta: {}ms", snap.rtt_delta_ms);
            info!(
                "  earned-ack: {}",
                if snap.earned_ack_window { "on" } else { "off" }
            );
        }

        "stats" => {
            if let Some(stats) = stats {
                let json = stats.to_json();
                info!("stats requested, returning {} bytes", json.len());
                return CmdResponse::Json(json);
            } else {
                warn!("stats not available (no stats provider)");
            }
        }

        other => {
            warn!("unknown command: {}", other);
        }
    }

    CmdResponse::None
}

/// Prepare the control-socket path before binding, refusing to clobber anything
/// we must not delete.
///
/// Returns `true` when the path is clear to bind — it did not exist, or held a
/// *stale* socket (no live server answered a connect probe) that was safely
/// unlinked. Returns `false`, **without deleting anything**, when the path holds
/// a **live** socket (a server is already listening) or a **non-socket** entry
/// (regular file, directory, …). The caller must not bind when this is `false`.
#[cfg(unix)]
pub(crate) fn prepare_control_socket_path(socket_path: &str) -> bool {
    use std::os::unix::fs::FileTypeExt;

    let Ok(meta) = std::fs::metadata(socket_path) else {
        // Path does not exist (or is unreadable): nothing to unlink — bind will
        // create it, or fail loudly with its own error.
        return true;
    };

    if !meta.file_type().is_socket() {
        warn!("control socket path {socket_path} exists but is not a socket; refusing to unlink");
        return false;
    }

    // It is a socket. Probe it: a successful connect means a live server owns it.
    if UnixStream::connect(socket_path).is_ok() {
        warn!(
            "control socket {socket_path} is already in use by a live server; refusing to unlink"
        );
        return false;
    }

    // Stale socket (connect failed) — safe to unlink so we can rebind.
    let _ = std::fs::remove_file(socket_path);
    true
}

#[cfg(unix)]
fn unix_socket_loop(
    config: &DynamicConfig,
    socket_path: &str,
    stats: &SharedStats,
    subscriptions: &SubscriptionManager,
) {
    // Only unlink a stale socket; never clobber a live server or a non-socket.
    if !prepare_control_socket_path(socket_path) {
        return;
    }

    let listener = match UnixListener::bind(socket_path) {
        Ok(l) => l,
        Err(e) => {
            warn!("failed to bind unix socket {}: {}", socket_path, e);
            return;
        }
    };

    info!("unix socket listening at: {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let config_clone = config.clone();
                let stats_clone = stats.clone();
                let subscriptions_clone = subscriptions.clone();
                std::thread::spawn(move || {
                    handle_unix_client(config_clone, stream, stats_clone, subscriptions_clone);
                });
            }
            Err(e) => {
                debug!("unix socket accept error: {}", e);
            }
        }
    }
}

#[cfg(unix)]
fn handle_unix_client(
    config: DynamicConfig,
    mut stream: UnixStream,
    stats: SharedStats,
    subscriptions: SubscriptionManager,
) {
    // Clone stream for reading (we need separate read/write handles)
    let read_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let reader = BufReader::new(read_stream);

    for line in reader.lines() {
        let Ok(cmd) = line else { break };
        let trimmed = cmd.trim();

        // Frame discriminator (ADR-001): a line beginning with `{` is a JSON-RPC
        // frame; anything else is the legacy text protocol. A malformed JSON-RPC
        // frame returns a structured error — it never falls through to text.
        let response = if trimmed.starts_with('{') {
            // `subscribe-events` turns the connection into a one-way event
            // stream: it has no single response, so it is handled before the
            // request/response dispatch and consumes the connection until EOF.
            if is_subscribe_events(trimmed) {
                run_subscription_loop(&subscriptions, &mut stream);
                return;
            }
            Some(crate::jsonrpc::dispatch_jsonrpc(trimmed, &config))
        } else {
            match apply_cmd(&config, trimmed, Some(&stats)) {
                CmdResponse::Json(json) => Some(json),
                CmdResponse::None => None,
            }
        };

        if let Some(body) = response {
            if let Err(e) = writeln!(stream, "{body}") {
                debug!("failed to write response: {e}");
                break;
            }
            if let Err(e) = stream.flush() {
                debug!("failed to flush response: {e}");
                break;
            }
        }
    }
}

/// True only when `frame` is a JSON-RPC request whose `method` is exactly
/// `subscribe-events`. A malformed frame returns `false` and falls through to
/// `dispatch_jsonrpc`, which emits the proper structured parse error.
#[cfg(unix)]
fn is_subscribe_events(frame: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(frame)
        .ok()
        .and_then(|v| {
            v.get("method")
                .and_then(serde_json::Value::as_str)
                .map(|m| m == "subscribe-events")
        })
        .unwrap_or(false)
}

/// Stream telemetry `"event"` notifications to a subscribed client until the
/// peer hangs up. Each frame from the per-subscriber channel is written as one
/// newline-delimited line; a write/flush error or a closed channel ends the
/// loop, dropping the receiver so the manager prunes this subscriber on its
/// next broadcast.
#[cfg(unix)]
fn run_subscription_loop(subscriptions: &SubscriptionManager, stream: &mut UnixStream) {
    let rx = subscriptions.subscribe();
    while let Ok(frame) = rx.recv() {
        if writeln!(stream, "{frame}").is_err() || stream.flush().is_err() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = DynamicConfig::new();
        let snap = config.snapshot();
        assert_eq!(snap.mode, SchedulingMode::Enhanced);
        assert!(snap.quality_enabled);
        assert!(!snap.exploration_enabled);
        assert_eq!(snap.rtt_delta_ms, DEFAULT_RTT_DELTA_MS);
        assert!(
            !snap.earned_ack_window,
            "earned-ack window valve defaults OFF"
        );
    }

    #[test]
    fn test_config_from_cli() {
        let config = DynamicConfig::from_cli(SchedulingMode::Classic, true, true, 50, false);
        let snap = config.snapshot();
        assert_eq!(snap.mode, SchedulingMode::Classic);
        assert!(!snap.quality_enabled); // no_quality=true means disabled
        assert!(snap.exploration_enabled);
        assert_eq!(snap.rtt_delta_ms, 50);
        assert!(!snap.earned_ack_window);
    }

    #[test]
    fn test_earned_ack_commands() {
        let config = DynamicConfig::new();
        assert!(!config.snapshot().earned_ack_window);

        apply_cmd(&config, "earned-ack on", None);
        assert!(config.snapshot().earned_ack_window);

        apply_cmd(&config, "earned-ack off", None);
        assert!(!config.snapshot().earned_ack_window);
    }

    #[test]
    fn test_earned_ack_from_cli_on() {
        let config = DynamicConfig::from_cli(SchedulingMode::Enhanced, false, false, 30, true);
        assert!(config.snapshot().earned_ack_window);
    }

    #[test]
    fn test_mode_commands() {
        let config = DynamicConfig::new();

        apply_cmd(&config, "mode classic", None);
        assert_eq!(config.mode(), SchedulingMode::Classic);

        apply_cmd(&config, "mode enhanced", None);
        assert_eq!(config.mode(), SchedulingMode::Enhanced);

        apply_cmd(&config, "mode rtt-threshold", None);
        assert_eq!(config.mode(), SchedulingMode::RttThreshold);
    }

    #[test]
    fn test_quality_commands() {
        let config = DynamicConfig::new();

        apply_cmd(&config, "quality off", None);
        assert!(!config.snapshot().quality_enabled);

        apply_cmd(&config, "quality on", None);
        assert!(config.snapshot().quality_enabled);
    }

    #[test]
    fn test_exploration_commands() {
        let config = DynamicConfig::new();

        apply_cmd(&config, "explore on", None);
        assert!(config.snapshot().exploration_enabled);

        apply_cmd(&config, "explore off", None);
        assert!(!config.snapshot().exploration_enabled);
    }

    #[test]
    fn test_rtt_delta_commands() {
        let config = DynamicConfig::new();

        apply_cmd(&config, "rtt-delta 50", None);
        assert_eq!(config.snapshot().rtt_delta_ms, 50);

        apply_cmd(&config, "rtt-delta 100", None);
        assert_eq!(config.snapshot().rtt_delta_ms, 100);
    }

    #[test]
    fn test_effective_quality() {
        // Classic mode - quality never effective
        let snap = ConfigSnapshot {
            mode: SchedulingMode::Classic,
            quality_enabled: true,
            exploration_enabled: true,
            rtt_delta_ms: 30,
            earned_ack_window: false,
        };
        assert!(!snap.effective_quality_enabled());
        assert!(!snap.effective_exploration_enabled());

        // Enhanced mode - both can be effective
        let snap = ConfigSnapshot {
            mode: SchedulingMode::Enhanced,
            quality_enabled: true,
            exploration_enabled: true,
            rtt_delta_ms: 30,
            earned_ack_window: false,
        };
        assert!(snap.effective_quality_enabled());
        assert!(snap.effective_exploration_enabled());

        // RTT-threshold mode - quality effective, exploration not
        let snap = ConfigSnapshot {
            mode: SchedulingMode::RttThreshold,
            quality_enabled: true,
            exploration_enabled: true,
            rtt_delta_ms: 30,
            earned_ack_window: false,
        };
        assert!(snap.effective_quality_enabled());
        assert!(!snap.effective_exploration_enabled());
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let config = DynamicConfig::new();
        let config_clone = config.clone();

        let writer = thread::spawn(move || {
            for _ in 0..100 {
                config_clone.set_mode(SchedulingMode::Classic);
                config_clone.set_mode(SchedulingMode::Enhanced);
                config_clone.set_mode(SchedulingMode::RttThreshold);
            }
        });

        let config_clone2 = config.clone();
        let reader = thread::spawn(move || {
            for _ in 0..100 {
                let _ = config_clone2.snapshot();
            }
        });

        writer.join().unwrap();
        reader.join().unwrap();
    }
}
