//! Ablation switches for the adaptive-scheduler evaluation campaign.
//!
//! Two environment variables, read ONCE at startup and ONLY in a
//! `test-internals` build:
//!
//! - `SRTLA_ADAPTIVE_FEATURES` — which `SchedulerFeatures` bits every selector runs.
//! - `SRTLA_ADAPTIVE_TUNING` — overrides for the four sweepable constants.
//!
//! The gate is COMPILE-time, not runtime: without the feature this module never
//! names either variable, so the shipped binary contains neither string and every
//! accessor const-folds to the shipped value.
//!
//! **ABSENT means `SchedulerFeatures::default()` and `AdaptiveTuning::SHIPPED`,
//! never the all-bits set by construction.** A `test-internals` binary with no
//! env set must run exactly what the release binary runs; otherwise a campaign
//! measures a configuration that never ships. When a later change narrows the
//! shipped default, the unset-env binary narrows with it — automatically,
//! because it resolves through `Default`, not through a second literal.

/// The four sweepable scheduler constants, resolved once at startup.
///
/// Every field carries the SHIPPED value unless `SRTLA_ADAPTIVE_TUNING` overrode
/// it in a `test-internals` build.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdaptiveTuning {
    /// `HealthConstants::stall_attempts` — sends without proof before a stall.
    pub stall_attempts: u32,
    /// `HealthConstants::loss_enter` — loss EWMA that latches Degraded.
    pub loss_enter: f64,
    /// Fraction of the negotiated SRT latency above which `DeadlineGate` holds.
    pub deadline_hold_fraction: f64,
    /// Multiplicative decrease applied to the rate-cap target on a loss backoff.
    pub ratecap_loss_backoff: f64,
}

impl AdaptiveTuning {
    /// What the release binary runs. These literals are the single source of
    /// truth for all four constants; their former call sites read them here.
    pub const SHIPPED: Self = Self {
        stall_attempts: 32,
        loss_enter: 0.10,
        deadline_hold_fraction: 0.5,
        ratecap_loss_backoff: 0.85,
    };

    /// The `DeadlineGate` release threshold, kept at the shipped 0.4/0.5 ratio so
    /// a swept hold fraction keeps a hysteresis band instead of inverting it
    /// against a second, un-swept literal.
    #[must_use]
    pub fn deadline_release_fraction(self) -> f64 {
        0.8 * self.deadline_hold_fraction
    }
}

impl Default for AdaptiveTuning {
    fn default() -> Self {
        Self::SHIPPED
    }
}

#[cfg(feature = "test-internals")]
mod imp {
    use std::sync::OnceLock;

    use anyhow::{Result, bail};

    use super::AdaptiveTuning;
    use crate::sender::SchedulerFeatures;

    pub const FEATURES_ENV: &str = "SRTLA_ADAPTIVE_FEATURES";
    pub const TUNING_ENV: &str = "SRTLA_ADAPTIVE_TUNING";

    /// Token spelling and emission order for every feature bit. One table backs
    /// the parser AND the `effective_config` projection, so a campaign manifest
    /// can round-trip what it asked for.
    pub const FEATURE_TOKENS: [(&str, SchedulerFeatures); 9] = [
        ("stall", SchedulerFeatures::STALL),
        ("loss", SchedulerFeatures::LOSS),
        ("queue", SchedulerFeatures::QUEUE),
        ("deadline", SchedulerFeatures::DEADLINE),
        ("rejoin", SchedulerFeatures::REJOIN),
        ("sole", SchedulerFeatures::SOLE),
        ("pref", SchedulerFeatures::PREF),
        ("ratecap", SchedulerFeatures::RATECAP),
        ("quality", SchedulerFeatures::QUALITY),
    ];

    pub const TUNING_KEYS: [&str; 4] = [
        "stall_attempts",
        "loss_enter",
        "deadline_hold_fraction",
        "ratecap_loss_backoff",
    ];

    #[derive(Clone, Copy)]
    struct Resolved {
        features: SchedulerFeatures,
        tuning: AdaptiveTuning,
    }

    static RESOLVED: OnceLock<Resolved> = OnceLock::new();

    /// Read both variables once, before anything consults a feature bit or a
    /// constant. An unparseable value is a STARTUP ERROR: a silently ignored
    /// ablation request would publish an `effective_config` the run did not use.
    ///
    /// An uninitialized process (unit tests, library embedders) keeps the shipped
    /// values, so nothing behaves differently for never calling this.
    pub fn init_from_env() -> Result<()> {
        let resolved = Resolved {
            features: parse_features(std::env::var(FEATURES_ENV).ok().as_deref())?,
            tuning: parse_tuning(std::env::var(TUNING_ENV).ok().as_deref())?,
        };
        let _ = RESOLVED.set(resolved);
        Ok(())
    }

    pub fn features() -> SchedulerFeatures {
        RESOLVED
            .get()
            .map_or_else(SchedulerFeatures::default, |resolved| resolved.features)
    }

    pub fn tuning() -> AdaptiveTuning {
        RESOLVED
            .get()
            .map_or(AdaptiveTuning::SHIPPED, |resolved| resolved.tuning)
    }

    /// `None` (absent) is the shipped default. `all`, `none` and `default` are
    /// whole-value words: inside a comma list they are unknown tokens, because
    /// `stall,all` has no single honest meaning.
    pub fn parse_features(raw: Option<&str>) -> Result<SchedulerFeatures> {
        let Some(raw) = raw else {
            return Ok(SchedulerFeatures::default());
        };
        let raw = raw.trim();
        match raw {
            "all" => return Ok(SchedulerFeatures::ALL),
            "none" => return Ok(SchedulerFeatures::NONE),
            "default" => return Ok(SchedulerFeatures::default()),
            "" => bail!(
                "{FEATURES_ENV} is empty: use `default`, `all`, `none`, or a comma list of {}",
                feature_tokens()
            ),
            _ => {}
        }
        let mut features = SchedulerFeatures::NONE;
        for token in raw.split(',') {
            let token = token.trim();
            let Some(&(_, bit)) = FEATURE_TOKENS.iter().find(|(name, _)| *name == token) else {
                bail!(
                    "{FEATURES_ENV}: unknown feature '{token}'; expected one of {} (or the whole \
                     value `all`, `none`, `default`)",
                    feature_tokens()
                );
            };
            features = features | bit;
        }
        Ok(features)
    }

    /// `None` (absent) is `AdaptiveTuning::SHIPPED`. Listed keys override; every
    /// unlisted key keeps its shipped value.
    pub fn parse_tuning(raw: Option<&str>) -> Result<AdaptiveTuning> {
        let mut tuning = AdaptiveTuning::SHIPPED;
        let Some(raw) = raw else {
            return Ok(tuning);
        };
        let raw = raw.trim();
        if raw.is_empty() {
            bail!(
                "{TUNING_ENV} is empty: use `key=value` pairs from {}",
                tuning_keys()
            );
        }
        for entry in raw.split(',') {
            let entry = entry.trim();
            let Some((key, value)) = entry.split_once('=') else {
                bail!(
                    "{TUNING_ENV}: '{entry}' is not `key=value`; expected keys {}",
                    tuning_keys()
                );
            };
            apply(&mut tuning, key.trim(), value.trim())?;
        }
        Ok(tuning)
    }

    fn apply(tuning: &mut AdaptiveTuning, key: &str, value: &str) -> Result<()> {
        match key {
            "stall_attempts" => tuning.stall_attempts = parse_attempts(key, value)?,
            "loss_enter" => tuning.loss_enter = parse_unit_fraction(key, value)?,
            "deadline_hold_fraction" => {
                tuning.deadline_hold_fraction = parse_unit_fraction(key, value)?;
            }
            "ratecap_loss_backoff" => {
                tuning.ratecap_loss_backoff = parse_backoff_factor(key, value)?;
            }
            other => bail!(
                "{TUNING_ENV}: unknown key '{other}'; expected one of {}",
                tuning_keys()
            ),
        }
        Ok(())
    }

    /// Zero attempts would latch every link stalled on its first send; the upper
    /// bound keeps a typo from disabling the detector outright.
    fn parse_attempts(key: &str, value: &str) -> Result<u32> {
        let parsed: u32 = value
            .parse()
            .map_err(|_| anyhow::anyhow!("{TUNING_ENV}: {key}='{value}' is not an integer"))?;
        if !(1..=10_000).contains(&parsed) {
            bail!("{TUNING_ENV}: {key}={parsed} is outside 1..=10000");
        }
        Ok(parsed)
    }

    /// A fraction of a measured quantity: `(0.0, 1.0]`.
    fn parse_unit_fraction(key: &str, value: &str) -> Result<f64> {
        let parsed = parse_finite(key, value)?;
        if parsed <= 0.0 || parsed > 1.0 {
            bail!("{TUNING_ENV}: {key}={parsed} is outside (0.0, 1.0]");
        }
        Ok(parsed)
    }

    /// A multiplicative DECREASE: `1.0` or above would grow the target on loss.
    fn parse_backoff_factor(key: &str, value: &str) -> Result<f64> {
        let parsed = parse_finite(key, value)?;
        if parsed <= 0.0 || parsed >= 1.0 {
            bail!("{TUNING_ENV}: {key}={parsed} is outside (0.0, 1.0)");
        }
        Ok(parsed)
    }

    fn parse_finite(key: &str, value: &str) -> Result<f64> {
        let parsed: f64 = value
            .parse()
            .map_err(|_| anyhow::anyhow!("{TUNING_ENV}: {key}='{value}' is not a number"))?;
        if !parsed.is_finite() {
            bail!("{TUNING_ENV}: {key}='{value}' is not finite");
        }
        Ok(parsed)
    }

    fn feature_tokens() -> String {
        FEATURE_TOKENS
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(",")
    }

    fn tuning_keys() -> String {
        TUNING_KEYS.join(",")
    }

    /// Active bits in table order. `none` is an empty array, never a missing key.
    pub fn feature_names(features: SchedulerFeatures) -> Vec<&'static str> {
        FEATURE_TOKENS
            .iter()
            .filter(|(_, bit)| features.contains(*bit))
            .map(|(name, _)| *name)
            .collect()
    }

    fn tuning_value(tuning: AdaptiveTuning) -> serde_json::Value {
        serde_json::json!({
            "stall_attempts": tuning.stall_attempts,
            "loss_enter": tuning.loss_enter,
            "deadline_hold_fraction": tuning.deadline_hold_fraction,
            "ratecap_loss_backoff": tuning.ratecap_loss_backoff,
        })
    }

    /// Append this build's `effective_config` to a single-line JSON object body.
    /// Both members are ADDITIVE; a reader that does not know them is unaffected.
    /// A body that is not a JSON object is returned untouched rather than
    /// replaced, so a malformed base can never become a fabricated config.
    pub fn with_effective_config(object: &str) -> String {
        let Ok(mut doc) = serde_json::from_str::<serde_json::Value>(object) else {
            return object.to_string();
        };
        let Some(map) = doc.as_object_mut() else {
            return object.to_string();
        };
        map.insert(
            "adaptive_features".to_string(),
            serde_json::json!(feature_names(features())),
        );
        map.insert("adaptive_tuning".to_string(), tuning_value(tuning()));
        doc.to_string()
    }

    /// Operator-readable form of the active bits for the `status` command.
    pub fn features_csv() -> String {
        let names = feature_names(features());
        if names.is_empty() {
            "none".to_string()
        } else {
            names.join(",")
        }
    }

    /// Operator-readable form of the effective constants for the `status` command.
    pub fn tuning_csv() -> String {
        let tuning = tuning();
        format!(
            "stall_attempts={},loss_enter={},deadline_hold_fraction={},ratecap_loss_backoff={}",
            tuning.stall_attempts,
            tuning.loss_enter,
            tuning.deadline_hold_fraction,
            tuning.ratecap_loss_backoff
        )
    }
}

#[cfg(not(feature = "test-internals"))]
mod imp {
    use anyhow::Result;

    use super::AdaptiveTuning;
    use crate::sender::SchedulerFeatures;

    /// Without `test-internals` no environment is read at all — this module does
    /// not name either variable, which is what the release `strings` check pins.
    #[inline(always)]
    pub fn init_from_env() -> Result<()> {
        Ok(())
    }

    #[inline(always)]
    pub fn features() -> SchedulerFeatures {
        SchedulerFeatures::default()
    }

    #[inline(always)]
    pub fn tuning() -> AdaptiveTuning {
        AdaptiveTuning::SHIPPED
    }
}

pub use imp::*;
