//! JSON-RPC 2.0 control-server tests (ADR-001).
//!
//! Covers the dual-support control surface on the Unix `--control-socket`:
//! - loopback Unix-socket roundtrips for `hello` / `set-mode` / `set-rtt-delta`,
//! - the legacy line-oriented text protocol still working on the same socket,
//! - parse/dispatch of whitespace + key-order JSON-RPC variants,
//! - structured `-32700` / `-32600` / `-32601` errors for malformed/invalid
//!   frames and unknown methods,
//! - the socket-unlink guard refusing to delete a non-socket file.
//!
//! Gated to `unix` because the JSON-RPC control socket is Unix-only (the
//! non-unix build serves stdin text only — see `config::spawn_config_listener`).

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use smallvec::SmallVec;

use crate::bind_map::{LinkId, Priority};
use crate::config::{DynamicConfig, prepare_control_socket_path, spawn_config_listener};
use crate::connection::SrtlaConnection;
use crate::jsonrpc::dispatch_jsonrpc;
use crate::mode::SchedulingMode;
use crate::sender::pool_control::PoolControlReceiver;
use crate::stats::SharedStats;
use crate::subscription::SubscriptionManager;

// ---- Test harness ---------------------------------------------------------

/// Spawn a control-socket listener bound to a fresh temp path, sharing the
/// supplied `config` (atomic-backed, so socket-driven mutations are visible
/// through the returned handle). The `TempDir` guard must outlive the test so
/// the socket path survives.
fn spawn_listener(config: &DynamicConfig) -> (String, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create tempdir");
    let path = dir.path().join("control.sock");
    let path_str = path.to_str().expect("utf-8 socket path").to_string();
    spawn_config_listener(
        config.clone(),
        Some(path_str.clone()),
        SharedStats::new(),
        SubscriptionManager::new(),
    );
    (path_str, dir)
}

/// A blocking line-oriented client over the control socket.
struct Client {
    write: UnixStream,
    read: BufReader<UnixStream>,
}

impl Client {
    /// Connect, retrying briefly because the listener binds on a background
    /// thread spawned by `spawn_config_listener`.
    fn connect(path: &str) -> Self {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match UnixStream::connect(path) {
                Ok(write) => {
                    write
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .unwrap();
                    let read = BufReader::new(write.try_clone().expect("clone stream"));
                    return Self { write, read };
                }
                Err(e) => {
                    assert!(
                        Instant::now() < deadline,
                        "could not connect to control socket {path}: {e}"
                    );
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    /// Send one frame and read exactly one newline-delimited response line.
    fn call(&mut self, frame: &str) -> Value {
        writeln!(self.write, "{frame}").expect("write frame");
        self.write.flush().expect("flush frame");
        let mut line = String::new();
        self.read.read_line(&mut line).expect("read response line");
        serde_json::from_str(line.trim_end()).expect("response is valid JSON")
    }

    /// Send a frame that produces no response (legacy text command).
    fn send_no_reply(&mut self, frame: &str) {
        writeln!(self.write, "{frame}").expect("write frame");
        self.write.flush().expect("flush frame");
    }
}

/// Poll `cond` until true or the deadline elapses (socket mutations land on the
/// listener thread, asynchronously to the client write).
fn wait_for(mut cond: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if cond() {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("condition not met within timeout");
}

fn capabilities(result: &Value) -> Vec<String> {
    result["capabilities"]
        .as_array()
        .expect("capabilities array")
        .iter()
        .map(|c| c.as_str().expect("capability string").to_string())
        .collect()
}

const EXPECTED_CAPABILITIES: [&str; 6] = [
    "stats-subscription",
    "set-mode",
    "set-quality",
    "set-exploration",
    "set-rtt-delta",
    "get-status",
];

// ---- 1. hello over a loopback Unix socket ---------------------------------

#[test]
fn hello_returns_version_and_capabilities() {
    let config = DynamicConfig::new();
    let (path, _dir) = spawn_listener(&config);
    let mut client = Client::connect(&path);

    let resp = client.call(r#"{"jsonrpc":"2.0","method":"hello","id":1}"#);

    assert_eq!(resp["jsonrpc"], Value::from("2.0"));
    assert_eq!(resp["id"], Value::from(1));
    let result = &resp["result"];
    assert_eq!(result["schema_version"], Value::from(1));
    assert_eq!(result["engine"], Value::from("srtla_send"));
    let caps = capabilities(result);
    for expected in EXPECTED_CAPABILITIES {
        assert!(
            caps.contains(&expected.to_string()),
            "missing capability {expected} in {caps:?}"
        );
    }
}

// ---- 2. set-mode mutates config; legacy text works on the same socket -----

#[test]
fn set_mode_changes_dynamic_config() {
    let config = DynamicConfig::new();
    let (path, _dir) = spawn_listener(&config);
    let mut client = Client::connect(&path);

    let resp =
        client.call(r#"{"jsonrpc":"2.0","method":"set-mode","params":{"mode":"classic"},"id":2}"#);
    assert_eq!(resp["result"]["ok"], Value::Bool(true));
    assert_eq!(resp["id"], Value::from(2));
    wait_for(|| config.mode() == SchedulingMode::Classic);

    // Legacy text `mode enhanced` on the SAME socket still applies.
    client.send_no_reply("mode enhanced");
    wait_for(|| config.mode() == SchedulingMode::Enhanced);
}

// ---- 3. set-rtt-delta mutates config (dispatch-level) ----------------------

#[test]
fn set_rtt_delta_changes_config() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"set-rtt-delta","params":{"ms":75},"id":3}"#,
        &config,
        &SharedStats::new(),
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
    assert_eq!(resp["result"]["ok"], Value::Bool(true));
    assert_eq!(resp["id"], Value::from(3));
    assert_eq!(config.snapshot().rtt_delta_ms, 75);
}

// ---- The full loopback roundtrip the checklist requires --------------------

#[test]
fn loopback_roundtrip_hello_setmode_setrttdelta() {
    let config = DynamicConfig::new();
    let (path, _dir) = spawn_listener(&config);
    let mut client = Client::connect(&path);

    let hello = client.call(r#"{"jsonrpc":"2.0","method":"hello","id":1}"#);
    assert_eq!(hello["result"]["engine"], Value::from("srtla_send"));

    let set_mode =
        client.call(r#"{"jsonrpc":"2.0","method":"set-mode","params":{"mode":"classic"},"id":2}"#);
    assert_eq!(set_mode["result"]["ok"], Value::Bool(true));
    wait_for(|| config.mode() == SchedulingMode::Classic);

    let set_rtt =
        client.call(r#"{"jsonrpc":"2.0","method":"set-rtt-delta","params":{"ms":75},"id":3}"#);
    assert_eq!(set_rtt["result"]["ok"], Value::Bool(true));
    wait_for(|| config.snapshot().rtt_delta_ms == 75);
}

// ---- 4. whitespace + key-order variants parse to a valid response ----------

#[test]
fn whitespace_and_key_order_variants_parse() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(
        r#"{ "id" : 1, "method" : "hello", "jsonrpc" : "2.0" }"#,
        &config,
        &SharedStats::new(),
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");

    assert!(
        resp.get("error").is_none(),
        "expected success, got {resp_str}"
    );
    assert_eq!(resp["id"], Value::from(1));
    assert_eq!(resp["result"]["engine"], Value::from("srtla_send"));
}

// ---- 5. malformed JSON -> -32700 parse error, id null ----------------------

#[test]
fn malformed_json_returns_parse_error() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc("{not valid json", &config, &SharedStats::new());
    let resp: Value = serde_json::from_str(&resp_str).expect("error envelope is valid JSON");

    assert_eq!(resp["jsonrpc"], Value::from("2.0"));
    assert_eq!(resp["error"]["code"], Value::from(-32700));
    assert_eq!(resp["id"], Value::Null);
}

// ---- 6. valid JSON without `method` -> -32600 invalid request --------------

#[test]
fn invalid_request_returns_error() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","foo":"bar","id":4}"#,
        &config,
        &SharedStats::new(),
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");

    assert_eq!(resp["error"]["code"], Value::from(-32600));
    assert_eq!(resp["id"], Value::from(4));
}

// ---- 7. unknown method -> -32601 method not found --------------------------

#[test]
fn unknown_method_returns_error() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"nonexistent","id":5}"#,
        &config,
        &SharedStats::new(),
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");

    assert_eq!(resp["error"]["code"], Value::from(-32601));
    assert_eq!(resp["id"], Value::from(5));
}

// ---- 8. socket-unlink guard refuses to delete a non-socket file ------------

#[test]
fn socket_unlink_guard_refuses_regular_file() {
    let dir = tempfile::tempdir().expect("create tempdir");
    let path = dir.path().join("not-a-socket.sock");
    std::fs::write(&path, b"i am a regular file").expect("write regular file");
    let path_str = path.to_str().expect("utf-8 path");

    // The guard must refuse (return false) and must NOT delete the file.
    assert!(
        !prepare_control_socket_path(path_str),
        "guard must refuse a non-socket path"
    );
    assert!(path.exists(), "guard must not delete a non-socket file");
    assert_eq!(
        std::fs::read(&path).expect("file still readable"),
        b"i am a regular file"
    );
}

// ---- 9. legacy text protocol still works on the JSON-RPC socket ------------

#[test]
fn legacy_text_protocol_still_works() {
    let config = DynamicConfig::new();
    let (path, _dir) = spawn_listener(&config);
    let mut client = Client::connect(&path);

    // A plain-text command (no leading `{`) on the same socket that serves
    // JSON-RPC routes to the legacy parser and mutates config.
    client.send_no_reply("mode classic");
    wait_for(|| config.mode() == SchedulingMode::Classic);
}

// ---- 10. hello matches ADR-001 §Methods (additive superset) ----------------

#[test]
fn hello_matches_adr() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"hello","id":1}"#,
        &config,
        &SharedStats::new(),
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
    let result = &resp["result"];

    // ADR-001 mandates the protocol/engine_version/schema_version triple.
    assert!(
        result.get("protocol").is_some(),
        "missing protocol: {result}"
    );
    assert!(
        result.get("engine_version").is_some(),
        "missing engine_version: {result}"
    );
    assert!(
        result.get("schema_version").is_some(),
        "missing schema_version: {result}"
    );

    // The additive superset (engine + capabilities) must remain present.
    assert!(result.get("engine").is_some(), "missing engine: {result}");
    assert!(
        result.get("capabilities").is_some(),
        "missing capabilities: {result}"
    );
}

// ---- 11. set-rtt-delta accepts the ADR canonical `delta_ms` -----------------

#[test]
fn set_rtt_delta_accepts_delta_ms() {
    let config = DynamicConfig::new();
    let set_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"set-rtt-delta","params":{"delta_ms":50},"id":1}"#,
        &config,
        &SharedStats::new(),
    );
    let set: Value = serde_json::from_str(&set_str).expect("valid JSON");
    assert_eq!(set["result"]["ok"], Value::Bool(true));

    let status_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"get-status","id":2}"#,
        &config,
        &SharedStats::new(),
    );
    let status: Value = serde_json::from_str(&status_str).expect("valid JSON");
    assert_eq!(status["result"]["rtt_delta_ms"], Value::from(50));
}

// ---- 12. set-rtt-delta accepts the `ms` back-compat alias -------------------

#[test]
fn set_rtt_delta_accepts_ms_alias() {
    let config = DynamicConfig::new();
    let set_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"set-rtt-delta","params":{"ms":50},"id":1}"#,
        &config,
        &SharedStats::new(),
    );
    let set: Value = serde_json::from_str(&set_str).expect("valid JSON");
    assert_eq!(set["result"]["ok"], Value::Bool(true));

    let status_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"get-status","id":2}"#,
        &config,
        &SharedStats::new(),
    );
    let status: Value = serde_json::from_str(&status_str).expect("valid JSON");
    assert_eq!(status["result"]["rtt_delta_ms"], Value::from(50));
}

// ---- 13. set-rtt-delta with no usable param -> -32602 invalid params --------

#[test]
fn set_rtt_delta_missing_param_is_invalid_params() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"set-rtt-delta","params":{},"id":1}"#,
        &config,
        &SharedStats::new(),
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
    assert_eq!(resp["error"]["code"], Value::from(-32602));
    assert_eq!(resp["id"], Value::from(1));
}

// ---- 14. set-mode accepts `edpf` (parity with the 4-mode enum) --------------

#[test]
fn set_mode_accepts_edpf() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"set-mode","params":{"mode":"edpf"},"id":1}"#,
        &config,
        &SharedStats::new(),
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
    assert_eq!(resp["result"]["ok"], Value::Bool(true));
    assert_eq!(resp["id"], Value::from(1));
    assert_eq!(config.mode(), SchedulingMode::Edpf);
}

// ---- 15. a valid-JSON array is an Invalid Request, NOT a panic --------------
//
// `dispatch_jsonrpc` parses any well-formed JSON value, including a top-level
// array (a JSON-RPC 2.0 *batch*). Such a value has no string `method`, so it
// routes to the `-32600 Invalid Request` arm rather than panicking — batch
// dispatch is deliberately not implemented here.
//
// NOTE ON SOCKET ROUTING (existing behavior, NOT changed here): the Unix
// control socket's frame discriminator (`config.rs:433`) routes a frame to
// `dispatch_jsonrpc` ONLY when the trimmed line begins with `{`. A `[`-prefixed
// frame (a batch array) therefore falls through to the legacy TEXT parser and
// never reaches `dispatch_jsonrpc` over the socket today. This test exercises
// the dispatcher directly to prove it degrades gracefully if a batch array ever
// does reach it.
#[test]
fn batch_array_is_invalid_request() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc("[1,2,3]", &config, &SharedStats::new());
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON envelope");

    assert_eq!(resp["jsonrpc"], Value::from("2.0"));
    assert_eq!(resp["error"]["code"], Value::from(-32600));
    // An array has no `id` member, so the echoed id is null.
    assert_eq!(resp["id"], Value::Null);
}

// ---- 16. `id` is echoed verbatim for string / number / absent(null) ---------

#[test]
fn id_echoed_verbatim_for_each_type() {
    let config = DynamicConfig::new();

    // (frame, expected echoed id)
    let cases: [(&str, Value); 4] = [
        (
            r#"{"jsonrpc":"2.0","method":"hello","id":7}"#,
            Value::from(7),
        ),
        (
            r#"{"jsonrpc":"2.0","method":"hello","id":"abc"}"#,
            Value::from("abc"),
        ),
        (
            // Absent id → echoed as null.
            r#"{"jsonrpc":"2.0","method":"hello"}"#,
            Value::Null,
        ),
        (
            // Explicit null id → echoed as null.
            r#"{"jsonrpc":"2.0","method":"hello","id":null}"#,
            Value::Null,
        ),
    ];

    for (frame, expected_id) in cases {
        let resp_str = dispatch_jsonrpc(frame, &config, &SharedStats::new());
        let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
        assert_eq!(resp["id"], expected_id, "id mismatch for frame {frame}");
        // The id echo holds even on an error envelope.
        let unknown_str = dispatch_jsonrpc(
            &frame.replace("hello", "no-such-method"),
            &config,
            &SharedStats::new(),
        );
        let unknown: Value = serde_json::from_str(&unknown_str).expect("valid JSON");
        assert_eq!(
            unknown["id"], expected_id,
            "id mismatch on error for frame {frame}"
        );
    }
}

// ---- 17. `method` present but not a string -> -32600 Invalid Request --------

#[test]
fn non_string_method_is_invalid_request() {
    let config = DynamicConfig::new();

    // method as a number, an array, and an object — all are Invalid Request.
    for frame in [
        r#"{"jsonrpc":"2.0","method":123,"id":1}"#,
        r#"{"jsonrpc":"2.0","method":["hello"],"id":1}"#,
        r#"{"jsonrpc":"2.0","method":{"name":"hello"},"id":1}"#,
    ] {
        let resp_str = dispatch_jsonrpc(frame, &config, &SharedStats::new());
        let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
        assert_eq!(
            resp["error"]["code"],
            Value::from(-32600),
            "expected Invalid Request for {frame}"
        );
        assert_eq!(resp["id"], Value::from(1));
    }
}

// ---- 18. malformed JSON variants all -> -32700 parse error, id null ---------

#[test]
fn malformed_json_variants_return_parse_error() {
    let config = DynamicConfig::new();

    for frame in [
        "{not valid json",
        "",
        "   ",
        r#"{"jsonrpc":"2.0","method":"hello""#, // truncated (unterminated object)
        "}{",
    ] {
        let resp_str = dispatch_jsonrpc(frame, &config, &SharedStats::new());
        let resp: Value = serde_json::from_str(&resp_str).expect("error envelope is valid JSON");
        assert_eq!(
            resp["error"]["code"],
            Value::from(-32700),
            "expected parse error for {frame:?}"
        );
        assert_eq!(resp["id"], Value::Null, "parse-error id must be null");
    }
}

// ---- 19. set-mode wrong/missing params -> -32602 invalid params -------------

#[test]
fn set_mode_bad_params_are_invalid_params() {
    let config = DynamicConfig::new();

    for frame in [
        // missing params entirely
        r#"{"jsonrpc":"2.0","method":"set-mode","id":1}"#,
        // empty params object
        r#"{"jsonrpc":"2.0","method":"set-mode","params":{},"id":1}"#,
        // wrong-typed mode (number, not string)
        r#"{"jsonrpc":"2.0","method":"set-mode","params":{"mode":42},"id":1}"#,
        // unknown mode string
        r#"{"jsonrpc":"2.0","method":"set-mode","params":{"mode":"bogus"},"id":1}"#,
    ] {
        let resp_str = dispatch_jsonrpc(frame, &config, &SharedStats::new());
        let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
        assert_eq!(
            resp["error"]["code"],
            Value::from(-32602),
            "expected invalid params for {frame}"
        );
        assert_eq!(resp["id"], Value::from(1));
    }
}

// ---- 20. set-quality wrong/missing params -> -32602 invalid params ----------

#[test]
fn set_quality_bad_params_are_invalid_params() {
    let config = DynamicConfig::new();

    for frame in [
        // missing params entirely
        r#"{"jsonrpc":"2.0","method":"set-quality","id":1}"#,
        // empty params object
        r#"{"jsonrpc":"2.0","method":"set-quality","params":{},"id":1}"#,
        // wrong-typed enabled (string, not boolean)
        r#"{"jsonrpc":"2.0","method":"set-quality","params":{"enabled":"yes"},"id":1}"#,
    ] {
        let resp_str = dispatch_jsonrpc(frame, &config, &SharedStats::new());
        let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
        assert_eq!(
            resp["error"]["code"],
            Value::from(-32602),
            "expected invalid params for {frame}"
        );
        assert_eq!(resp["id"], Value::from(1));
    }
}

// ---- 21. set-rtt-delta with a non-u32 (negative) value -> -32602 -----------

#[test]
fn set_rtt_delta_negative_is_invalid_params() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"set-rtt-delta","params":{"delta_ms":-5},"id":1}"#,
        &config,
        &SharedStats::new(),
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
    assert_eq!(resp["error"]["code"], Value::from(-32602));
    assert_eq!(resp["id"], Value::from(1));
}

// ---- 16. get-capabilities is the SAME document --capabilities-json prints ---

#[test]
fn get_capabilities_matches_the_pre_spawn_probe_document() {
    // Given: a supervisor that probed the binary before spawning it.
    let probed: Value =
        serde_json::from_str(&crate::capabilities::capability_json()).expect("probe emits JSON");

    // When: a consumer asks the live control socket the same question.
    let resp: Value = serde_json::from_str(&dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"get-capabilities","id":1}"#,
        &DynamicConfig::new(),
        &SharedStats::new(),
    ))
    .expect("valid JSON");
    let result = &resp["result"];

    // Then: every key of the probe document is present and IDENTICAL. Two
    // answers to "what can this build do?" that disagree would send a
    // supervisor down the wrong spawn path.
    for (key, value) in probed.as_object().expect("the probe is an object") {
        assert_eq!(result[key], *value, "get-capabilities drifted on `{key}`");
    }
    assert_eq!(
        result["capabilities"]["bind_map_schema_version"],
        Value::from(1)
    );
}

#[test]
fn get_capabilities_still_enumerates_the_control_methods() {
    // ADR-001 requires the method/topic enumeration; it moved to `methods` when
    // `capabilities` became the probe document's capability object.
    let resp: Value = serde_json::from_str(&dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"get-capabilities","id":1}"#,
        &DynamicConfig::new(),
        &SharedStats::new(),
    ))
    .expect("valid JSON");

    let methods = resp["result"]["methods"]
        .as_array()
        .expect("methods must be enumerated");
    assert!(methods.iter().any(|m| m == "get-status"));
    assert!(methods.iter().any(|m| m == "stats-subscription"));
    assert!(methods.iter().any(|m| m == "set-link-priority"));
    assert_eq!(resp["result"]["capabilities"]["adaptive_scheduler"], true);
    assert_eq!(resp["result"]["capabilities"]["link_priority"], true);
}

#[test]
fn hello_keeps_capabilities_as_the_frozen_method_array() {
    // The TS control binding feature-detects with `hello.capabilities.includes`,
    // so this field must stay a string array even though `get-capabilities`
    // now answers with the richer probe document.
    let resp: Value = serde_json::from_str(&dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"hello","id":1}"#,
        &DynamicConfig::new(),
        &SharedStats::new(),
    ))
    .expect("valid JSON");

    let caps = resp["result"]["capabilities"]
        .as_array()
        .expect("hello.capabilities must remain an array");
    assert_eq!(caps, EXPECTED_CAPABILITIES.map(Value::from).as_slice());
}

#[test]
fn adaptive_jsonrpc_mode_round_trips() {
    // Given a real control socket, When setting adaptive, Then get-status echoes it.
    let config = DynamicConfig::new();
    let (path, _dir) = spawn_listener(&config);
    let mut client = Client::connect(&path);
    let response = client.call(r#"{"method":"set-mode","params":{"mode":"adaptive"},"id":1}"#);
    assert_eq!(response["result"]["ok"], true);
    let status = client.call(r#"{"method":"get-status","id":2}"#);
    assert_eq!(status["result"]["mode"], "adaptive");
}

#[test]
fn set_link_priority_bad_params_are_invalid_params() {
    // Given malformed key/value combinations, When dispatched, Then reject before enqueue.
    for params in [
        serde_json::json!({"link_id":"modem-a","conn_id":"0","priority":0.1}),
        serde_json::json!({"link_id":null,"conn_id":"0","priority":0.1}),
        serde_json::json!({"priority":0.1}),
        serde_json::json!({"conn_id":"0"}),
        serde_json::json!({"conn_id":"0","priority":0.20001}),
        serde_json::json!({"conn_id":"0","priority":-0.20001}),
        serde_json::json!({"conn_id":"0","priority":"0.1"}),
        serde_json::json!({"conn_id":0,"priority":null}),
        serde_json::json!({"conn_id":"-1","priority":null}),
        serde_json::json!({"conn_id":"184467440737095516160","priority":null}),
        serde_json::json!({"link_id":"","priority":null}),
        serde_json::json!(null),
    ] {
        let frame = serde_json::json!({"method":"set-link-priority","params":params,"id":"bad"});
        let response: Value = serde_json::from_str(&dispatch_jsonrpc(
            &frame.to_string(),
            &DynamicConfig::new(),
            &SharedStats::new(),
        ))
        .unwrap();
        assert_eq!(response["error"]["code"], -32602, "{frame}: {response}");
        assert_eq!(response["id"], "bad");
    }
}

#[test]
fn set_link_priority_without_live_sender_returns_unavailable() {
    // Given no sender owner, When priority is requested, Then never report applied.
    let response: Value = serde_json::from_str(&dispatch_jsonrpc(
        r#"{"method":"set-link-priority","params":{"conn_id":"0","priority":0.2},"id":1}"#,
        &DynamicConfig::new(),
        &SharedStats::new(),
    ))
    .unwrap();
    assert_eq!(response["error"]["code"], -32002);
    assert!(response.get("result").is_none());
}

struct PriorityPool {
    stats: SharedStats,
    receiver: PoolControlReceiver,
    connections: SmallVec<SrtlaConnection, 4>,
}

impl PriorityPool {
    async fn new() -> Self {
        let stats = SharedStats::new();
        let receiver = stats.attach_pool_control();
        let mut conn = crate::test_helpers::create_test_connection().await;
        conn.link_id = Some(LinkId::parse("modem-a").unwrap());
        conn.priority_baseline = Some(Priority::try_from(-0.1).unwrap());
        Self {
            stats,
            receiver,
            connections: smallvec::smallvec![conn],
        }
    }

    async fn call(&mut self, params: Value) -> Value {
        let stats = self.stats.clone();
        let mut response = tokio::task::spawn_blocking(move || {
            dispatch_jsonrpc(
                &json!({"method":"set-link-priority","params":params,"id":24}).to_string(),
                &DynamicConfig::new(),
                &stats,
            )
        });
        let body = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::select! {
                body = &mut response => body.unwrap(),
                message = self.receiver.recv() => {
                    message.unwrap().apply(&mut self.connections);
                    response.await.unwrap()
                }
            }
        })
        .await
        .unwrap();
        serde_json::from_str(&body).unwrap()
    }

    async fn reload(&mut self) {
        self.receiver.close_for_reload();
        let specs: Vec<_> = self.connections.iter().map(SrtlaConnection::spec).collect();
        crate::sender::apply_link_changes(
            &mut self.connections,
            &specs,
            "127.0.0.1",
            5000,
            &mut None,
            &mut crate::sender::SequenceTracker::new(),
            &mut crate::registration::SrtlaRegistrationManager::new(),
        )
        .await;
        self.receiver = self.stats.attach_pool_control();
    }
}

#[tokio::test]
async fn set_link_priority_replies_after_live_application() {
    // Given real sockets and the production consumer, When addressed, Then echo actual state.
    let mut pool = PriorityPool::new().await;
    let reply = pool.call(json!({"link_id":"modem-a","priority":0.2})).await;
    assert_eq!(
        reply["result"],
        json!({
            "applied":true,"key":"link_id","link_id":"modem-a","conn_id":"0",
            "priority":0.2,"effective_priority":0.2,
        })
    );
    assert_eq!(pool.connections[0].effective_priority().unwrap().get(), 0.2);
}

#[tokio::test]
async fn unknown_link_returns_domain_error_without_mutation() {
    // Given an existing pool, When either key misses, Then return typed errors without mutation.
    let mut pool = PriorityPool::new().await;
    for params in [
        json!({"link_id":"absent","priority":0.1}),
        json!({"conn_id":"9","priority":0.1}),
    ] {
        let reply = pool.call(params).await;
        assert_eq!(reply["error"]["code"], -32001);
        assert_eq!(reply["error"]["data"]["kind"], "unknown_link");
        assert!(reply.get("result").is_none());
        assert_eq!(
            pool.connections[0].effective_priority().unwrap().get(),
            -0.1
        );
    }
}

#[tokio::test]
async fn null_clears_only_the_addressed_layer() {
    // Given distinct baseline/link/conn values, When cleared in order, Then expose the next layer.
    let mut pool = PriorityPool::new().await;
    for (params, expected) in [
        (json!({"link_id":"modem-a","priority":0.2}), 0.2),
        (json!({"conn_id":"0","priority":-0.2}), -0.2),
        (json!({"conn_id":"0","priority":null}), 0.2),
        (json!({"link_id":"modem-a","priority":null}), -0.1),
    ] {
        let reply = pool.call(params.clone()).await;
        assert_eq!(reply["result"]["priority"], params["priority"]);
        assert_eq!(reply["result"]["effective_priority"], expected);
        assert_eq!(
            pool.connections[0].effective_priority().unwrap().get(),
            expected
        );
    }
}

#[tokio::test]
async fn link_id_layer_survives_sighup_pool_rebuild() {
    // Given an RPC link override, When the SIGHUP rebuild runs, Then it survives by identity.
    let mut pool = PriorityPool::new().await;
    assert_eq!(
        pool.call(json!({"link_id":"modem-a","priority":0.2})).await["result"]["applied"],
        true
    );
    pool.reload().await;
    let reply = pool.call(json!({"conn_id":"0","priority":null})).await;
    assert_eq!(reply["result"]["effective_priority"], 0.2);
}

#[tokio::test]
async fn conn_id_layer_dropped_on_sighup_pool_rebuild() {
    // Given an RPC positional override, When SIGHUP keeps the same pool, Then baseline returns.
    let mut pool = PriorityPool::new().await;
    assert_eq!(
        pool.call(json!({"conn_id":"0","priority":-0.2})).await["result"]["applied"],
        true
    );
    pool.reload().await;
    let reply = pool
        .call(json!({"link_id":"modem-a","priority":null}))
        .await;
    assert_eq!(reply["result"]["effective_priority"], -0.1);
    assert!(pool.connections[0].priority_override_conn.is_none());
}

#[tokio::test]
async fn real_control_socket_changes_running_sender_priority() {
    // Given the real sender loop and control transport, When RPC sets priority, Then the owner replies.
    let dir = tempfile::tempdir().unwrap();
    let ips = dir.path().join("ips");
    std::fs::write(&ips, "127.0.0.1\n").unwrap();
    let path = dir.path().join("control.sock").to_str().unwrap().to_owned();
    let stats = SharedStats::new();
    let config = DynamicConfig::new();
    let subscriptions = SubscriptionManager::new();
    let snapshots = subscriptions.subscribe();
    spawn_config_listener(
        config.clone(),
        Some(path.clone()),
        stats.clone(),
        subscriptions.clone(),
    );
    let peer = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let sender = crate::sender::run_sender_with_config(
        0,
        "127.0.0.1",
        peer.local_addr().unwrap().port(),
        crate::sender::SenderPaths {
            ips_file: ips.to_str().unwrap(),
            bind_map: None,
        },
        config,
        stats,
        crate::sender::TelemetrySinks {
            file: None,
            subscriptions,
        },
    );
    let control = tokio::task::spawn_blocking(move || {
        snapshots.recv_timeout(Duration::from_secs(3)).unwrap();
        let mut client = Client::connect(&path);
        let reply = client.call(
            r#"{"method":"set-link-priority","params":{"conn_id":"0","priority":0.15},"id":24}"#,
        );
        assert_eq!(
            reply["result"],
            json!({"applied":true,"key":"conn_id","conn_id":"0","priority":0.15,"effective_priority":0.15})
        );
        let failed = client.call(r#"{"method":"set-link-priority","params":{"conn_id":"0","link_id":"both","priority":0.2},"id":25}"#);
        assert_eq!(failed["error"]["code"], -32602);
        eprintln!("real sender socket priority reply: {reply}; both-keys reply: {failed}");
    });
    tokio::select! {
        result = sender => panic!("sender exited before control: {result:?}"),
        result = control => result.unwrap(),
    }
}

#[tokio::test]
async fn priority_reply_deadline_cancels_unapplied_work() {
    // Given a stalled owner, When the real reply deadline expires, Then no late write is applied.
    let mut pool = PriorityPool::new().await;
    let stats = pool.stats.clone();
    let response = tokio::task::spawn_blocking(move || {
        dispatch_jsonrpc(
            r#"{"method":"set-link-priority","params":{"conn_id":"0","priority":0.2},"id":24}"#,
            &DynamicConfig::new(),
            &stats,
        )
    });
    let response: Value = serde_json::from_str(
        &tokio::time::timeout(Duration::from_secs(5), response)
            .await
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(response["error"]["code"], -32005);
    assert!(response.get("result").is_none());
    pool.receiver
        .recv()
        .await
        .unwrap()
        .apply(&mut pool.connections);
    assert_eq!(
        pool.connections[0].effective_priority().unwrap().get(),
        -0.1
    );
}

// ---- 17. get-status carries the ADR-003 operating mode + identity -----------

#[test]
fn get_status_reports_the_legacy_operating_mode_by_default() {
    // Given: a sender that never resolved a bind-map.
    let resp: Value = serde_json::from_str(&dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"get-status","id":1}"#,
        &DynamicConfig::new(),
        &SharedStats::new(),
    ))
    .expect("valid JSON");
    let result = &resp["result"];

    // Then: it says so in typed data rather than leaving the caller to infer it
    // from the absence of a field.
    assert_eq!(result["bind_map_status"]["state"], "absent");
    assert_eq!(result["disposition"]["state"], "legacy_unique_only");
    assert_eq!(result["links"], serde_json::json!([]));
}

#[test]
fn get_status_echoes_the_link_identity_and_the_degraded_mode() {
    // Given: a live sender whose reload degraded while a mapped pool runs.
    let stats = SharedStats::new();
    stats.set_bind_map(&crate::bind_map::BindMapReport::degraded(
        crate::bind_map::DegradedReason::HashMismatch,
        crate::bind_map::BindMapDisposition::RetainedLastValid,
    ));

    // When: get-status is asked.
    let resp: Value = serde_json::from_str(&dispatch_jsonrpc(
        r#"{"jsonrpc":"2.0","method":"get-status","id":1}"#,
        &DynamicConfig::new(),
        &stats,
    ))
    .expect("valid JSON");
    let result = &resp["result"];

    // Then: both halves of the operating mode are visible — degraded AND still
    // interface-pinned, which one field could not express.
    assert_eq!(result["bind_map_status"]["state"], "degraded");
    assert_eq!(result["bind_map_status"]["reason"], "hash_mismatch");
    assert_eq!(result["disposition"]["state"], "retained_last_valid");
}
