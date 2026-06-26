//! Clean-room JSON-RPC 2.0 control dispatch over the Unix `--control-socket`.
//!
//! Implements ADR-001 (`docs/adr/ADR-001-control-protocol.md`): the sender's
//! control surface speaks JSON-RPC 2.0 — mirroring cerastream's wire shape — on
//! the same Unix socket that still serves the legacy line-oriented text
//! protocol. A frame is routed by a cheap discriminator in
//! `config::handle_unix_client`: a trimmed line beginning with `{` is dispatched
//! here; anything else falls through to the text parser. A malformed JSON-RPC
//! frame returns a structured error (`-32700`/`-32600`) — it never silently
//! falls through to the text parser.
//!
//! The control methods map onto the existing `DynamicConfig` setters (the same
//! runtime state the text protocol mutates), so the two dialects stay in lock
//! step. This module owns only the JSON-RPC framing/dispatch; the socket
//! transport and the text protocol live in `config.rs`.

use serde_json::{Value, json};

use crate::config::DynamicConfig;
use crate::mode::SchedulingMode;

/// JSON-RPC 2.0 protocol version tag echoed in every response.
const JSONRPC_VERSION: &str = "2.0";

/// Handshake/telemetry schema version (matches the ADR-001 stats schema).
const SCHEMA_VERSION: u32 = 1;

/// Engine identifier returned by `hello` (mirrors cerastream's `engine`).
const ENGINE: &str = "srtla_send";

/// Control-protocol tag returned by `hello` (ADR-001); the wire dialect spoken.
const PROTOCOL: &str = "srtla-send-jsonrpc";

/// Engine build version returned by `hello` (ADR-001), from the crate version.
const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

// Standard JSON-RPC 2.0 error codes (the subset this dispatcher emits).
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// Control methods + event topics this sender build supports, advertised by
/// `hello` and `get-capabilities` so a consumer can feature-detect before
/// driving it.
const CAPABILITIES: [&str; 6] = [
    "stats-subscription",
    "set-mode",
    "set-quality",
    "set-exploration",
    "set-rtt-delta",
    "get-status",
];

/// Dispatch one JSON-RPC frame against the runtime config, returning the
/// serialized JSON-RPC response line (no trailing newline).
///
/// A frame that is not valid JSON returns a `-32700` parse error with a null
/// `id`; valid JSON that is not a request object carrying a string `method`
/// returns `-32600`; an unrecognized method returns `-32601`. Otherwise the
/// matching `DynamicConfig` setter runs and a `{"ok":true}` result is returned.
pub(crate) fn dispatch_jsonrpc(frame: &str, config: &DynamicConfig) -> String {
    // A non-JSON / malformed frame is a parse error with a null id.
    let Ok(value) = serde_json::from_str::<Value>(frame) else {
        return error_response(Value::Null, PARSE_ERROR, "Parse error");
    };

    // The request id is echoed verbatim; absent (or unparseable) → null.
    let id = value.get("id").cloned().unwrap_or(Value::Null);

    // Valid JSON that is not a request object with a string `method` is an
    // Invalid Request.
    let Some(method) = value.get("method").and_then(Value::as_str) else {
        return error_response(id, INVALID_REQUEST, "Invalid Request");
    };

    let params = value.get("params");

    match method {
        "hello" => success_response(id, hello_result()),
        "get-capabilities" => success_response(id, json!({ "capabilities": CAPABILITIES })),
        "set-mode" => set_mode(id, params, config),
        "set-quality" => set_bool(id, params, |enabled| config.set_quality_enabled(enabled)),
        "set-exploration" => set_bool(id, params, |enabled| {
            config.set_exploration_enabled(enabled)
        }),
        "set-rtt-delta" => set_rtt_delta(id, params, config),
        "get-status" => success_response(id, status_result(config)),
        _ => error_response(id, METHOD_NOT_FOUND, "Method not found"),
    }
}

/// The `hello` handshake. ADR-001 requires the `{protocol, engine_version,
/// schema_version}` triple; `engine` + `capabilities` are an additive superset.
fn hello_result() -> Value {
    json!({
        "protocol": PROTOCOL,
        "engine_version": ENGINE_VERSION,
        "schema_version": SCHEMA_VERSION,
        "engine": ENGINE,
        "capabilities": CAPABILITIES,
    })
}

/// The `get-status` result: the current `ConfigSnapshot` as JSON (the same
/// state the text protocol's `status` command prints).
fn status_result(config: &DynamicConfig) -> Value {
    let snap = config.snapshot();
    json!({
        "mode": snap.mode.to_string(),
        "quality_enabled": snap.quality_enabled,
        "exploration_enabled": snap.exploration_enabled,
        "rtt_delta_ms": snap.rtt_delta_ms,
    })
}

fn set_mode(id: Value, params: Option<&Value>, config: &DynamicConfig) -> String {
    let Some(mode_str) = params.and_then(|p| p.get("mode")).and_then(Value::as_str) else {
        return error_response(id, INVALID_PARAMS, "set-mode requires params.mode (string)");
    };
    match mode_str.parse::<SchedulingMode>() {
        Ok(mode) => {
            config.set_mode(mode);
            success_response(id, ok_result())
        }
        Err(_) => error_response(
            id,
            INVALID_PARAMS,
            "invalid mode; use classic, enhanced, rtt-threshold, or edpf",
        ),
    }
}

/// Shared `params.enabled` validation for the boolean toggles; `apply` runs the
/// per-method `DynamicConfig` setter.
fn set_bool(id: Value, params: Option<&Value>, apply: impl FnOnce(bool)) -> String {
    let Some(enabled) = params
        .and_then(|p| p.get("enabled"))
        .and_then(Value::as_bool)
    else {
        return error_response(id, INVALID_PARAMS, "requires params.enabled (boolean)");
    };
    apply(enabled);
    success_response(id, ok_result())
}

fn set_rtt_delta(id: Value, params: Option<&Value>, config: &DynamicConfig) -> String {
    // ADR-001 canonical key is `delta_ms`; `ms` is a back-compat alias.
    // `delta_ms` wins when both are present.
    let Some(ms) = params
        .and_then(|p| p.get("delta_ms").or_else(|| p.get("ms")))
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
    else {
        return error_response(
            id,
            INVALID_PARAMS,
            "set-rtt-delta requires params.delta_ms (u32 milliseconds; `ms` accepted as alias)",
        );
    };
    config.set_rtt_delta_ms(ms);
    success_response(id, ok_result())
}

fn ok_result() -> Value {
    json!({ "ok": true })
}

fn success_response(id: Value, result: Value) -> String {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "result": result,
        "id": id,
    })
    .to_string()
}

fn error_response(id: Value, code: i64, message: &str) -> String {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "error": { "code": code, "message": message },
        "id": id,
    })
    .to_string()
}
