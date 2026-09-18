use serde_json::{Value, json};

use crate::config::{CmdResponse, DynamicConfig, apply_cmd};
#[cfg(all(unix, not(loom)))]
use crate::jsonrpc::dispatch_jsonrpc;
use crate::stats::SharedStats;

#[test]
#[cfg(all(unix, not(loom)))]
fn retired_rpc_controls_succeed_without_changing_status_or_effective_config() {
    // Given each valid retired control request and a fresh configuration.
    for (method, params) in [
        ("set-quality", json!({"enabled": false})),
        ("set-exploration", json!({"enabled": true})),
        ("set-rtt-delta", json!({"delta_ms": 123})),
    ] {
        let config = DynamicConfig::new();
        let stats = SharedStats::new();
        let status = r#"{"id":1,"method":"get-status"}"#;
        let before = dispatch_jsonrpc(status, &config, &stats);
        let effective = crate::adaptive_env::with_effective_config("{}");
        // When the control request is dispatched.
        let response: Value = serde_json::from_str(&dispatch_jsonrpc(
            &json!({"id":2,"method":method,"params":params}).to_string(),
            &config,
            &stats,
        ))
        .unwrap();
        // Then success is additive and neither visible nor effective config changed.
        assert_eq!(
            response["result"],
            json!({"ok":true,"deprecated":true,"effect":"none"})
        );
        assert_eq!(dispatch_jsonrpc(status, &config, &stats), before);
        assert_eq!(crate::adaptive_env::with_effective_config("{}"), effective);
    }
}

#[test]
fn retired_text_controls_return_deprecation_without_mutation() {
    // Given both boolean values and a distinct RTT delta.
    for command in [
        "quality on",
        "quality off",
        "explore on",
        "explore off",
        "rtt-delta 123",
    ] {
        let config = DynamicConfig::new();
        let before = format!("{:?}", config.snapshot());
        let effective = crate::adaptive_env::with_effective_config("{}");
        // When applying the legacy text command.
        let response = apply_cmd(&config, command, None);
        // Then the text socket receives a success document and config is unchanged.
        let CmdResponse::Json(response) = response else {
            panic!("missing deprecation response: {command}");
        };
        assert_eq!(
            serde_json::from_str::<Value>(&response).unwrap(),
            json!({"ok":true,"deprecated":true,"effect":"none"})
        );
        assert_eq!(format!("{:?}", config.snapshot()), before);
        assert_eq!(crate::adaptive_env::with_effective_config("{}"), effective);
    }
}

#[test]
#[cfg(all(unix, not(loom)))]
fn retired_rpc_modes_return_typed_errors_without_mutation() {
    // Given each removed CLI mode.
    for mode in ["classic", "rtt-threshold", "edpf", "adaptive"] {
        let config = DynamicConfig::new();
        // When the runtime mode method is called.
        let response: Value = serde_json::from_str(&dispatch_jsonrpc(
            &json!({"id":1,"method":"set-mode","params":{"mode":mode}}).to_string(),
            &config,
            &SharedStats::new(),
        ))
        .unwrap();
        // Then rejection has a machine-readable kind and retains the running mode.
        assert_eq!(response["error"]["code"], -32602);
        assert_eq!(response["error"]["data"]["kind"], "retired_mode");
        assert_eq!(config.mode().to_string(), "enhanced");
    }
}

#[test]
#[cfg(all(unix, not(loom)))]
fn control_discovery_arrays_remain_byte_identical() {
    // Given the frozen pre-retirement arrays.
    let hello = r#"["stats-subscription","set-mode","set-quality","set-exploration","set-rtt-delta","get-status"]"#;
    let methods = r#"["stats-subscription","set-mode","set-quality","set-exploration","set-rtt-delta","get-status","set-link-priority"]"#;
    for (method, field, expected) in [
        ("hello", "capabilities", hello),
        ("get-capabilities", "methods", methods),
    ] {
        // When querying discovery through the real dispatcher.
        let response: Value = serde_json::from_str(&dispatch_jsonrpc(
            &json!({"id":1,"method":method}).to_string(),
            &DynamicConfig::new(),
            &SharedStats::new(),
        ))
        .unwrap();
        // Then order and bytes remain frozen despite retiring the setters.
        assert_eq!(response["result"][field].to_string(), expected);
    }
}
