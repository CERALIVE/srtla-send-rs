use serde_json::Value;

use crate::config::DynamicConfig;
#[cfg(all(unix, not(loom)))]
use crate::jsonrpc::dispatch_jsonrpc;
use crate::mode::SchedulingMode;
use crate::sender::SchedulerShared;
use crate::stats::SharedStats;
use crate::telemetry_doc::build_telemetry_json_from_stats;
use crate::tests::adaptive_tests::pool_of;

fn assert_private_keys_absent(value: &Value) {
    match value {
        Value::Object(fields) => {
            for (key, child) in fields {
                assert!(
                    ![
                        "SchedulerFeatures",
                        "features",
                        "scheduler_features",
                        "adaptive_features"
                    ]
                    .contains(&key.as_str()),
                    "leaked {key}"
                );
                assert_private_keys_absent(child);
            }
        }
        Value::Array(values) => values.iter().for_each(assert_private_keys_absent),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}

#[tokio::test]
#[cfg(all(unix, not(loom)))]
async fn scheduler_features_remain_private_on_all_serialized_surfaces() {
    // Given real admitted links and each mode with quality enabled and disabled.
    let config = DynamicConfig::new();
    let stats = SharedStats::new();
    let mut shared = SchedulerShared::new(stats.clone());
    let mut conns = pool_of(2).await;
    for mode in [SchedulingMode::Enhanced] {
        config.set_mode(mode);
        for quality in [false, true] {
            config.set_quality_enabled(quality);
            shared.update_stats(&mut conns, &config.snapshot());
            // When the actual RPC dispatcher and telemetry/capability serializers run.
            let mut documents = vec![
                crate::capabilities::capability_json(),
                build_telemetry_json_from_stats(10_000, &stats.get()),
            ];
            for method in ["get-status", "hello", "get-capabilities"] {
                let frame = serde_json::json!({"jsonrpc":"2.0", "id":1, "method":method});
                let response = dispatch_jsonrpc(&frame.to_string(), &config, &stats);
                let parsed: Value = serde_json::from_str(&response).unwrap();
                assert!(parsed["result"].is_object());
                documents.push(response);
            }
            // Then no private feature key appears, including nested per-link objects.
            for document in documents {
                assert_private_keys_absent(&serde_json::from_str(&document).unwrap());
            }
        }
    }
}

#[test]
#[should_panic(expected = "leaked features")]
fn serialization_key_guard_detects_nested_leaks() {
    // Given a nested future leak, when inspected, then the guard must fail.
    assert_private_keys_absent(&serde_json::json!({"result":{"links":[{"features":511}]}}));
}
