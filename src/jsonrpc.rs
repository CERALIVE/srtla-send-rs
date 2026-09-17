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
//! Config setters share the text protocol's state. Priority writes instead await
//! the sender-owned pool channel; enqueue success alone is never application.
//! Socket transport and text parsing live in `config.rs`.

// allow: SIZE_OK — the scoped control lane cannot add module files; framing and
// the bounded priority adapter remain together without moving the shared transport.

use std::time::Duration;

use serde_json::{Value, json};

use crate::bind_map::{LinkId, Priority};
use crate::capabilities::capability_document;
use crate::config::DynamicConfig;
use crate::mode::SchedulingMode;
use crate::sender::pool_control::{LinkKey, PoolControlError, PoolControlRequest};
use crate::stats::SharedStats;

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

/// Frozen legacy `hello` array. New methods belong only in `get-capabilities`.
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
/// matching operation runs. Priority application waits at most two seconds.
/// Call only on a blocking/control thread, never a Tokio worker: the priority
/// adapter drives its reply future locally while the sender runtime keeps forwarding.
pub(crate) fn dispatch_jsonrpc(frame: &str, config: &DynamicConfig, stats: &SharedStats) -> String {
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
        "get-capabilities" => success_response(id, capabilities_result()),
        "set-mode" => set_mode(id, params, config),
        "set-link-priority" => set_link_priority(id, params, stats),
        "set-quality" => set_bool(id, params, |enabled| config.set_quality_enabled(enabled)),
        "set-exploration" => set_bool(id, params, |enabled| {
            config.set_exploration_enabled(enabled)
        }),
        "set-rtt-delta" => set_rtt_delta(id, params, config),
        "get-status" => success_response(id, status_result(config, stats)),
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

/// The runtime `get-capabilities` result.
///
/// This is the **same document** `--capabilities-json` prints before the process
/// ever binds a socket, so a supervisor that probed the binary pre-spawn and a
/// consumer that asks the live control socket can never be told two different
/// things. `methods` is the additive control-method/event-topic enumeration
/// ADR-001 requires; everything else is the capability document verbatim.
fn capabilities_result() -> Value {
    let mut doc = serde_json::to_value(capability_document())
        .unwrap_or_else(|e| unreachable!("capability document is always serializable: {e}"));
    if let Some(obj) = doc.as_object_mut() {
        let methods: Vec<_> = CAPABILITIES
            .iter()
            .copied()
            .chain(["set-link-priority"])
            .collect();
        obj.insert("methods".to_string(), json!(methods));
    }
    doc
}

/// The `get-status` result: the current `ConfigSnapshot` as JSON (the same state
/// the text protocol's `status` command prints), plus the ADR-003 operating
/// mode and the per-link identity echo.
fn status_result(config: &DynamicConfig, stats: &SharedStats) -> Value {
    let snap = config.snapshot();
    let live = stats.get();
    let mut result = json!({
        "mode": snap.mode.to_string(),
        "quality_enabled": snap.quality_enabled,
        "exploration_enabled": snap.exploration_enabled,
        "rtt_delta_ms": snap.rtt_delta_ms,
        "bind_map_status": live.bind_map.bind_map_status,
        "disposition": live.bind_map.disposition,
        "links": link_identities(&live),
    });
    if let Some(ms) = stats.negotiated_latency_ms() {
        result["negotiated_latency_ms"] = json!(ms);
    }
    result
}

#[cfg(test)]
#[path = "scheduler_serialization_tests.rs"]
mod scheduler_serialization_tests;

#[cfg(test)]
mod negotiated_latency_tests {
    use super::*;

    #[test]
    fn get_status_omits_unknown_negotiated_latency() {
        // Given no handshake observation, When queried, Then the optional key is absent.
        let result = status_result(&DynamicConfig::new(), &SharedStats::new());
        assert!(result.get("negotiated_latency_ms").is_none());
    }

    #[test]
    fn get_status_reports_negotiated_latency_without_housekeeping() {
        // Given a received delay with no snapshot update, When queried, Then expose ms.
        let stats = SharedStats::new();
        stats.set_negotiated_latency_ms(2000);
        let response = dispatch_jsonrpc(
            r#"{"jsonrpc":"2.0","id":1,"method":"get-status"}"#,
            &DynamicConfig::new(),
            &stats,
        );
        let parsed: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(parsed["result"]["negotiated_latency_ms"], 2000);
    }

    #[tokio::test]
    async fn status_link_observations_are_optional_and_preserve_zero_priority() {
        // Given a real link snapshot, When observations appear, Then emit only known values.
        let stats = SharedStats::new();
        let conn = crate::test_helpers::create_test_connection().await;
        stats.update(&[conn], &DynamicConfig::new().snapshot());
        let mut snapshot = stats.get();
        assert_eq!(link_identities(&snapshot)[0]["health"], "down");
        snapshot.links[0].health = None;
        assert!(link_identities(&snapshot)[0].get("health").is_none());
        assert!(link_identities(&snapshot)[0].get("priority").is_none());
        snapshot.links[0].health = Some("healthy");
        snapshot.links[0].priority = Some(0.0);
        let links = link_identities(&snapshot);
        assert_eq!(links[0]["health"], "healthy");
        assert_eq!(links[0]["priority"], 0.0);
    }

    #[test]
    fn pool_errors_have_distinct_typed_wire_codes() {
        // Given each owner error, When projected, Then preserve its retry-relevant distinction.
        for (error, code, kind) in [
            (
                PoolControlError::UnknownLink(LinkKey::ConnId(8)),
                -32001,
                "unknown_link",
            ),
            (PoolControlError::Unavailable, -32002, "unavailable"),
            (PoolControlError::Busy, -32003, "busy"),
            (PoolControlError::PoolReloaded, -32004, "pool_reloaded"),
        ] {
            let response: Value =
                serde_json::from_str(&pool_error_response(json!(24), error)).unwrap();
            assert_eq!(response["error"]["code"], code);
            assert_eq!(response["error"]["data"]["kind"], kind);
            assert_eq!(response["id"], 24);
        }
    }
}

/// Per-link identity, in the same order (and therefore the same `conn_id`
/// numbering) the telemetry document uses.
///
/// `iface` and `link_id` are omitted for an unmapped link rather than sent as
/// null: the sender only ever echoes the sidecar's identity and never invents
/// one, so "absent" is the honest answer.
fn link_identities(stats: &crate::stats::StatsSnapshot) -> Value {
    let links: Vec<Value> = stats
        .links
        .iter()
        .enumerate()
        .map(|(idx, link)| {
            let mut record = json!({ "conn_id": idx.to_string() });
            let obj = record.as_object_mut().expect("just built as an object");
            if let Some(iface) = link.iface.as_ref() {
                obj.insert("iface".to_string(), json!(iface));
            }
            if let Some(link_id) = link.link_id.as_ref() {
                obj.insert("link_id".to_string(), json!(link_id));
            }
            if let Some(health) = link.health {
                obj.insert("health".to_string(), json!(health));
            }
            if let Some(priority) = link.priority {
                obj.insert("priority".to_string(), json!(priority));
            }
            record
        })
        .collect();
    json!(links)
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
            "invalid mode; use classic, enhanced, rtt-threshold, edpf, or adaptive",
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

fn parse_priority_request(params: Option<&Value>) -> Result<PoolControlRequest, &'static str> {
    let params = params
        .and_then(Value::as_object)
        .ok_or("params must be an object")?;
    let key = match (params.get("link_id"), params.get("conn_id")) {
        (Some(Value::String(id)), None) => LinkKey::LinkId(
            LinkId::parse(id)
                .map_err(|_| "link_id must be 1-64 printable ASCII bytes without spaces")?,
        ),
        (None, Some(Value::String(id)))
            if !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()) =>
        {
            LinkKey::ConnId(id.parse().map_err(|_| "conn_id position is too large")?)
        }
        _ => return Err("provide exactly one of link_id or conn_id (string)"),
    };
    let priority = match params.get("priority") {
        Some(Value::Null) => None,
        Some(Value::Number(value)) => Some(Priority::try_from(
            value.as_f64().ok_or("priority must be a finite number")?,
        )?),
        _ => return Err("priority is required: number in -0.20..=0.20 or null"),
    };
    Ok(PoolControlRequest::SetLinkPriority { key, priority })
}

fn set_link_priority(id: Value, params: Option<&Value>, stats: &SharedStats) -> String {
    let request = match parse_priority_request(params) {
        Ok(request) => request,
        Err(message) => return error_response(id, INVALID_PARAMS, message),
    };
    let Some(handle) = stats.pool_control() else {
        return pool_error_response(id, PoolControlError::Unavailable);
    };
    // This timer-only runtime lives on the existing std control thread: no new
    // worker, no polling sleeps, and no wait on the sender's forwarding thread.
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::warn!(%error, "could not initialize priority reply timer");
            return error_response(id, -32603, "could not initialize priority reply timer");
        }
    };
    let reply = match handle.submit(request) {
        Ok(reply) => reply,
        Err(error) => return pool_error_response(id, error),
    };
    let applied =
        runtime.block_on(async { tokio::time::timeout(Duration::from_secs(2), reply).await });
    match applied {
        Ok(Ok(Ok(reply))) => {
            let key = match reply.key {
                LinkKey::LinkId(_) => "link_id",
                LinkKey::ConnId(_) => "conn_id",
            };
            let mut result = json!({
                "applied": reply.applied, "key": key, "conn_id": reply.conn_id.to_string(),
                "priority": reply.priority.map(Priority::get),
                "effective_priority": reply.effective_priority.map(Priority::get),
            });
            if let Some(link_id) = reply.link_id {
                result["link_id"] = json!(link_id.as_str());
            }
            success_response(id, result)
        }
        Ok(Ok(Err(error))) => pool_error_response(id, error),
        Ok(Err(_)) => pool_error_response(id, PoolControlError::Unavailable),
        // Dropping the reply cancels queued work, but cannot undo an application
        // racing the deadline. Report uncertainty; never retry across a pool epoch.
        Err(_) => error_response(
            id,
            -32005,
            "priority reply timed out; application outcome unknown",
        ),
    }
}

fn pool_error_response(id: Value, error: PoolControlError) -> String {
    let (code, kind) = match &error {
        PoolControlError::UnknownLink(_) => (-32001, "unknown_link"),
        PoolControlError::Unavailable => (-32002, "unavailable"),
        PoolControlError::Busy => (-32003, "busy"),
        PoolControlError::PoolReloaded => (-32004, "pool_reloaded"),
    };
    json!({
        "jsonrpc": JSONRPC_VERSION, "id": id,
        "error": {"code": code, "message": error.to_string(), "data": {"kind": kind}},
    })
    .to_string()
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
