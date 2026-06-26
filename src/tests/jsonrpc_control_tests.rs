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

use serde_json::Value;

use crate::config::{DynamicConfig, prepare_control_socket_path, spawn_config_listener};
use crate::jsonrpc::dispatch_jsonrpc;
use crate::mode::SchedulingMode;
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
    let resp_str = dispatch_jsonrpc("{not valid json", &config);
    let resp: Value = serde_json::from_str(&resp_str).expect("error envelope is valid JSON");

    assert_eq!(resp["jsonrpc"], Value::from("2.0"));
    assert_eq!(resp["error"]["code"], Value::from(-32700));
    assert_eq!(resp["id"], Value::Null);
}

// ---- 6. valid JSON without `method` -> -32600 invalid request --------------

#[test]
fn invalid_request_returns_error() {
    let config = DynamicConfig::new();
    let resp_str = dispatch_jsonrpc(r#"{"jsonrpc":"2.0","foo":"bar","id":4}"#, &config);
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
    let resp_str = dispatch_jsonrpc(r#"{"jsonrpc":"2.0","method":"hello","id":1}"#, &config);
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
    );
    let set: Value = serde_json::from_str(&set_str).expect("valid JSON");
    assert_eq!(set["result"]["ok"], Value::Bool(true));

    let status_str = dispatch_jsonrpc(r#"{"jsonrpc":"2.0","method":"get-status","id":2}"#, &config);
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
    );
    let set: Value = serde_json::from_str(&set_str).expect("valid JSON");
    assert_eq!(set["result"]["ok"], Value::Bool(true));

    let status_str = dispatch_jsonrpc(r#"{"jsonrpc":"2.0","method":"get-status","id":2}"#, &config);
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
    let resp_str = dispatch_jsonrpc("[1,2,3]", &config);
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
        let resp_str = dispatch_jsonrpc(frame, &config);
        let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
        assert_eq!(resp["id"], expected_id, "id mismatch for frame {frame}");
        // The id echo holds even on an error envelope.
        let unknown_str = dispatch_jsonrpc(&frame.replace("hello", "no-such-method"), &config);
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
        let resp_str = dispatch_jsonrpc(frame, &config);
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
        let resp_str = dispatch_jsonrpc(frame, &config);
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
        let resp_str = dispatch_jsonrpc(frame, &config);
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
        let resp_str = dispatch_jsonrpc(frame, &config);
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
    );
    let resp: Value = serde_json::from_str(&resp_str).expect("valid JSON");
    assert_eq!(resp["error"]["code"], Value::from(-32602));
    assert_eq!(resp["id"], Value::from(1));
}
