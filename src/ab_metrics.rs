//! Process-lifetime monotonic counters for the switch-cooldown / flush-on-switch
//! A/B evaluation (`tests/ab_switch_eval.rs`).
//!
//! These are DELIBERATELY separate from every other counter in the tree:
//!
//! - **not** `SrtlaConnection::nak_count` — that one is reset by the congestion
//!   controller (`src/connection/congestion/mod.rs`), so a window-delta taken
//!   across it is not a count of NAKs observed;
//! - **not** `SharedStats` — that is a housekeeping-cadence *snapshot*, refreshed
//!   on a timer, so a query does not read query-time truth.
//!
//! Every counter here is incremented on the production path, NEVER reset, and
//! read atomically by the `metrics` runtime command. The A/B runner queries at
//! the start and end of its measurement window and gates on the deltas, so
//! pre-window traffic (registration, warm-up) is excluded by construction.
//!
//! The whole facility is behind the `test-internals` feature. With the feature
//! off, every `record_*` entry point is an `#[inline(always)]` empty function,
//! so the shipped binary carries no counters and no atomics on the packet path.

#[cfg(feature = "test-internals")]
mod imp {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, LazyLock};

    /// The three A/B evaluation counters, shared by `Arc` across the sender
    /// tasks and the control-plane thread that answers `metrics`.
    #[derive(Debug, Default)]
    pub struct AbMetrics {
        switch_count: AtomicU64,
        nak_count: AtomicU64,
        cooldown_hold_count: AtomicU64,
    }

    impl AbMetrics {
        /// A `Some(prev) -> Some(new)` connection switch with `prev != new`.
        /// The initial selection (`None -> Some(new)`) is deliberately excluded:
        /// it is not a switch and would make an idle run look "activated".
        #[inline]
        pub fn record_switch(&self) {
            self.switch_count.fetch_add(1, Ordering::Relaxed);
        }

        /// One NAKed sequence number observed on the uplink receive path.
        #[inline]
        pub fn record_nak(&self) {
            self.nak_count.fetch_add(1, Ordering::Relaxed);
        }

        /// The switch cooldown suppressed a proposed switch (the selector wanted
        /// a different link and was forced to keep the current one).
        #[inline]
        pub fn record_cooldown_hold(&self) {
            self.cooldown_hold_count.fetch_add(1, Ordering::Relaxed);
        }

        pub fn switch_count(&self) -> u64 {
            self.switch_count.load(Ordering::Relaxed)
        }

        pub fn nak_count(&self) -> u64 {
            self.nak_count.load(Ordering::Relaxed)
        }

        pub fn cooldown_hold_count(&self) -> u64 {
            self.cooldown_hold_count.load(Ordering::Relaxed)
        }

        /// One newline-free JSON object, the reply body of the `metrics`
        /// runtime command.
        pub fn to_json(&self) -> String {
            format!(
                "{{\"switch_count\":{},\"nak_count\":{},\"cooldown_hold_count\":{}}}",
                self.switch_count(),
                self.nak_count(),
                self.cooldown_hold_count()
            )
        }
    }

    static METRICS: LazyLock<Arc<AbMetrics>> = LazyLock::new(|| Arc::new(AbMetrics::default()));

    /// The process-wide `Arc<AbMetrics>` handle.
    pub fn metrics() -> Arc<AbMetrics> {
        Arc::clone(&METRICS)
    }

    #[inline]
    pub fn record_switch() {
        METRICS.record_switch();
    }

    #[inline]
    pub fn record_nak() {
        METRICS.record_nak();
    }

    #[inline]
    pub fn record_cooldown_hold() {
        METRICS.record_cooldown_hold();
    }
}

#[cfg(not(feature = "test-internals"))]
mod imp {
    #[inline(always)]
    pub fn record_switch() {}

    #[inline(always)]
    pub fn record_nak() {}

    #[inline(always)]
    pub fn record_cooldown_hold() {}
}

pub use imp::*;

#[cfg(all(test, feature = "test-internals"))]
mod tests {
    use super::*;

    #[test]
    fn counters_are_monotonic_and_shared() {
        let m = metrics();
        let switches = m.switch_count();
        let naks = m.nak_count();
        let holds = m.cooldown_hold_count();

        record_switch();
        record_nak();
        record_nak();
        record_cooldown_hold();

        // A second handle observes the same counters (Arc-shared, not a copy).
        let m2 = metrics();
        assert_eq!(m2.switch_count(), switches + 1);
        assert_eq!(m2.nak_count(), naks + 2);
        assert_eq!(m2.cooldown_hold_count(), holds + 1);
    }

    #[test]
    fn json_shape_is_the_three_gated_fields() {
        let json = metrics().to_json();
        assert!(json.starts_with("{\"switch_count\":"), "{json}");
        assert!(json.contains("\"nak_count\":"), "{json}");
        assert!(json.ends_with("}"), "{json}");
        assert!(!json.contains('\n'), "reply must be a single line: {json}");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
        assert!(parsed["switch_count"].is_u64());
        assert!(parsed["nak_count"].is_u64());
        assert!(parsed["cooldown_hold_count"].is_u64());
    }
}
