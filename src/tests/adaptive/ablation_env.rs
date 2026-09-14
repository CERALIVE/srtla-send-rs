//! `SRTLA_ADAPTIVE_FEATURES` / `SRTLA_ADAPTIVE_TUNING` boundary parsing.
//!
//! Pure parser assertions: nothing here mutates the process environment, so
//! these run beside the trace tests without ordering constraints.

use crate::adaptive_env::{
    AdaptiveTuning, feature_names, parse_features, parse_tuning, with_effective_config,
};
use crate::sender::selection::adaptive::AdaptiveFeatures;

#[test]
fn absent_env_equals_default_not_all() {
    // Given no `SRTLA_ADAPTIVE_FEATURES`, when parsed, then the SHIPPED set.
    assert_eq!(
        parse_features(None).expect("absent is valid"),
        AdaptiveFeatures::default()
    );
    // The release binary and an unset-env `test-internals` binary must agree,
    // which is what resolving through `Default` (never through a second
    // literal) guarantees. TODAY the shipped default is still every bit, so
    // this equality holds; once the default narrows, flip this assertion to
    // `assert_ne!` — the parse target above does not change.
    assert_eq!(AdaptiveFeatures::default(), AdaptiveFeatures::ALL);
    assert_eq!(
        parse_features(None).expect("absent is valid"),
        parse_features(Some("default")).expect("word is valid")
    );
}

#[test]
fn whole_value_words_resolve_to_whole_sets() {
    assert_eq!(
        parse_features(Some("all")).expect("valid"),
        AdaptiveFeatures::ALL
    );
    assert_eq!(
        parse_features(Some("none")).expect("valid"),
        AdaptiveFeatures::NONE
    );
    assert_eq!(
        parse_features(Some(" all ")).expect("surrounding space is trimmed"),
        AdaptiveFeatures::ALL
    );
}

#[test]
fn a_comma_list_selects_exactly_its_tokens() {
    let parsed = parse_features(Some("stall, ratecap")).expect("valid");
    assert_eq!(feature_names(parsed), vec!["stall", "ratecap"]);
    assert_eq!(
        parse_features(Some("stall,stall")).expect("idempotent"),
        AdaptiveFeatures::STALL
    );
    assert_eq!(
        feature_names(AdaptiveFeatures::NONE),
        Vec::<&'static str>::new()
    );
}

#[test]
fn an_unusable_feature_value_is_an_error() {
    for raw in [
        "bogus",
        "",
        " ",
        "stall,bogus",
        "STALL",
        "stall,all",
        "stall;loss",
    ] {
        assert!(
            parse_features(Some(raw)).is_err(),
            "'{raw}' must be refused"
        );
    }
}

#[test]
fn absent_tuning_is_the_shipped_constants() {
    assert_eq!(
        parse_tuning(None).expect("absent is valid"),
        AdaptiveTuning::SHIPPED
    );
    assert_eq!(AdaptiveTuning::default(), AdaptiveTuning::SHIPPED);
    assert!((AdaptiveTuning::SHIPPED.deadline_release_fraction() - 0.4).abs() < f64::EPSILON);
}

#[test]
fn listed_keys_override_and_the_rest_keep_shipped_values() {
    let parsed = parse_tuning(Some(
        "stall_attempts=8, loss_enter=0.25,deadline_hold_fraction=0.75,ratecap_loss_backoff=0.5",
    ))
    .expect("valid");
    assert_eq!(parsed.stall_attempts, 8);
    assert!((parsed.loss_enter - 0.25).abs() < f64::EPSILON);
    assert!((parsed.deadline_hold_fraction - 0.75).abs() < f64::EPSILON);
    assert!((parsed.ratecap_loss_backoff - 0.5).abs() < f64::EPSILON);

    let partial = parse_tuning(Some("stall_attempts=4")).expect("valid");
    assert_eq!(partial.stall_attempts, 4);
    assert!((partial.loss_enter - AdaptiveTuning::SHIPPED.loss_enter).abs() < f64::EPSILON);
}

#[test]
fn an_unusable_tuning_value_is_an_error() {
    for raw in [
        "stall_attempts=abc",
        "stall_attempts=0",
        "stall_attempts=10001",
        "stall_attempts=-1",
        "unknown_key=1",
        "stall_attempts",
        "",
        "loss_enter=0",
        "loss_enter=1.5",
        "loss_enter=nan",
        "deadline_hold_fraction=0",
        "deadline_hold_fraction=2",
        "ratecap_loss_backoff=1",
        "ratecap_loss_backoff=0",
        "stall_attempts=4,unknown_key=1",
    ] {
        assert!(parse_tuning(Some(raw)).is_err(), "'{raw}' must be refused");
    }
}

#[test]
fn effective_config_is_additive_on_the_metrics_reply() {
    let base = crate::ab_metrics::metrics().to_json();
    let merged = with_effective_config(&base);
    let parsed: serde_json::Value = serde_json::from_str(&merged).expect("valid JSON");
    assert!(parsed["switch_count"].is_u64(), "{merged}");
    assert!(parsed["adaptive_features"].is_array(), "{merged}");
    assert!(parsed["adaptive_tuning"].is_object(), "{merged}");
    assert_eq!(
        parsed["adaptive_tuning"]["stall_attempts"].as_u64(),
        Some(u64::from(AdaptiveTuning::SHIPPED.stall_attempts))
    );
    assert!(!merged.contains('\n'), "reply must stay one line: {merged}");
}

#[test]
fn a_non_object_body_is_returned_untouched() {
    for body in ["[1,2]", "not json", "\"text\""] {
        assert_eq!(with_effective_config(body), body);
    }
}
