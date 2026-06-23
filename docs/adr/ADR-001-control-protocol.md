# ADR-001: JSON-RPC Control Protocol for srtla_send (supersedes srtla ADR-001 transport)

## Status

Accepted

> **Numbering note.** This is the first ADR in `srtla-send-rs` and is therefore
> numbered `ADR-001`. It is **distinct from** `srtla/docs/adr/ADR-001-telemetry-ipc.md`
> (the bonding-receiver/C-sender repo's telemetry-IPC record). Where this document
> says "srtla ADR-001" it means that other file; "this ADR" means the one you are
> reading. This ADR **supersedes the transport decision** of srtla ADR-001 for the
> Rust sender only — it preserves that ADR's JSON schema verbatim.

## Context

`srtla_send` (the Rust fork, this repo) already carries an opt-in telemetry sink
(`--stats-file`, `src/telemetry_file.rs`) and a runtime control surface
(`--control-socket`, `src/config.rs`). The control surface today speaks a
**line-oriented text protocol** (`mode classic`, `quality on|off`, `explore on|off`,
`rtt-delta <ms>`, `status`, `stats`) over stdin or a Unix domain socket; the
telemetry sink publishes an ADR-001 JSON snapshot to a file on a fixed cadence. Two
unrelated dialects, neither of which matches how the rest of the engine talks.

Meanwhile `cerastream` — the **sole** on-device streaming engine — drives every
operation as **JSON-RPC 2.0** over a Unix domain socket: a startup `hello` returns
`{protocol, engine_version, schema_version}`, `get-capabilities` enumerates the
feature set, control methods mutate runtime state, and `subscribe-events` opens a
push stream of `"event"` notifications. CeraUI's backend
(`cerastream-backend.ts`) is built around exactly this shape — it connects, runs
`hello`, validates `schema_version`, then `subscribeEvents(...)` to receive pushed
status. The srtla Rust sender being the **one** first-party engine that does NOT
speak this dialect is now **technical debt**: CeraUI must carry a second, bespoke
text/file integration path purely for the sender, and an operator-facing tool that
understands the engine cannot understand the sender.

**Engine consistency is the driver.** Aligning the sender's control + telemetry
surface with cerastream's JSON-RPC shape lets CeraUI reuse one client mental model,
one capability-probe pattern, and one event-subscription pattern across both
engines. This ADR records the decision to adopt that shape on the sender's
**existing** `--control-socket`, and to supersede srtla ADR-001's *transport*
(Option A — the stats file) for the Rust sender while keeping its JSON *schema*
exactly as-is.

### What this ADR does NOT change

- It does **not** redefine the telemetry JSON schema. The
  `{schema_version, last_updated_ms, connections:[…]}` shape and every field's name,
  type, unit, and staleness rule defined in srtla ADR-001 are **preserved verbatim**.
- It does **not** remove `--stats-file` or the legacy text protocol. Both keep
  working unchanged (see Dual-Support).
- It does **not** introduce any TCP or `0.0.0.0` listener. The control plane stays a
  local Unix domain socket, exactly as `--control-socket` is today.

## Supersession Statement

**This ADR supersedes the *transport* decision in
`srtla/docs/adr/ADR-001-telemetry-ipc.md` (Option A — the periodically-rewritten JSON
stats file) for the Rust sender `srtla_send` (`srtla-send-rs`).**

The supersession is **transport-only and scoped to the Rust sender**:

- **Schema preserved.** The JSON contract defined in srtla ADR-001 — field names,
  types, units (`bitrate_bps` = wire bytes/s × 8, `rtt_ms` in ms, `weight_percent`
  0–100, …), the `connections: []` idle-vs-absent rule, and the 5000 ms staleness
  threshold — remains the **canonical contract**. Each pushed event carries that same
  document (plus the additive `schema_version` field the Rust producer already emits).
- **Transport added, not swapped out.** JSON-RPC over `--control-socket` becomes the
  **preferred** transport for a capable consumer; the stats file remains the
  Option-A transport for the C `srtla_send` (deprecated, receiver-only package) and
  for anyone passing `--stats-file`.
- **C sender unaffected.** srtla ADR-001 Option A continues to govern the C
  `srtla_send` and the `--stats-file` option in full.

## Decision

**Adopt JSON-RPC 2.0 over the existing Unix `--control-socket` as the sender's
control + stats-subscription transport, mirroring cerastream's wire shape.** The
socket path convention is `/tmp/srtla-send-control-<listen_port>.sock` (mirroring the
`--stats-file` convention `/tmp/srtla-send-stats-<listen_port>.json`, keyed by the
same SRT listen port CeraUI already owns).

### Methods (request/response)

- **`hello`** → returns `{protocol, engine_version, schema_version}` **only** — the
  protocol/schema/engine version triple, matching cerastream's `hello`. It does
  **NOT** return the telemetry snapshot. (The last-known snapshot is replayed on
  `subscribe-events`, not on `hello` — see Rationale.)
- **`get-capabilities`** → enumerates the control methods and event topics this
  sender build supports, so a consumer can feature-detect before driving it (e.g.
  whether `set-exploration` / `set-rtt-delta` are honored in the active mode).
- **Control methods** (each maps onto the existing `DynamicConfig` setters in
  `src/config.rs`, the same state the text protocol mutates):
  - `set-mode` → `{mode: "classic"|"enhanced"|"rtt-threshold"}` (DynamicConfig mode)
  - `set-quality` → `{enabled: bool}` (quality scoring on/off)
  - `set-exploration` → `{enabled: bool}` (exploration on/off)
  - `set-rtt-delta` → `{delta_ms: u32}` (`set_rtt_delta_ms`)
  - `get-status` → current `ConfigSnapshot` (the JSON `status` the text protocol prints)

### Stats subscription (push)

- **`subscribe-events`** → a request that registers the caller for pushed
  **JSON-RPC notifications** with method `"event"` (the cerastream wire shape — NOT
  `subscribe`/`unsubscribe`; NOT one notification method per topic).
- Each `"event"` notification carries the **full ADR-001 telemetry snapshot** as its
  params: `{schema_version, last_updated_ms, connections:[ <7-key entries> ]}`, where
  each connection entry is the frozen 7-key set
  (`conn_id, rtt_ms, nak_count, weight_percent, window, in_flight, bitrate_bps`). This
  is the **same document** `build_telemetry_json` produces for `--stats-file` — the
  subscription path reuses that builder, it does not invent a second shape.
- Events are pushed at the `--stats-file-interval` cadence (default 1000 ms), the same
  clock that drives the file writer.
- On `subscribe-events`, the **last-known snapshot is replayed immediately** so a
  late-attaching consumer gets current state without waiting a full interval (this is
  the cure for srtla ADR-001's Option-B "no last-known-state" objection — see
  Rationale).

### Dual-support

The text protocol AND `--stats-file` keep working **unchanged**. A frame is routed by
a cheap discriminator: a trimmed line beginning with `{` is parsed as a JSON-RPC frame
(`serde_json`); anything else falls through to the existing text-command parser. A
malformed JSON-RPC frame returns a structured JSON-RPC error (`-32700`/`-32600`) — it
does **not** silently fall through to the text parser. `--stats-file` continues to
publish the atomic-rename snapshot in parallel with any active subscription
(dual-publish), so a file-based consumer and a subscription-based consumer can coexist.

## Consistency with cerastream

srtla ADR-001 rejected Option B (a streaming Unix-socket push channel) on four
grounds. cerastream's already-shipped JSON-RPC pattern resolves each of them — which
is precisely why adopting that pattern (rather than re-litigating a bespoke socket)
is the right move:

1. **"No last-known-state" (Option-B objection).** srtla ADR-001 noted that a raw push
   socket loses anything emitted before the consumer attaches, weakening the
   empty/stale story. **Resolved:** the **last-known snapshot is replayed on
   `subscribe-events`** (NOT on `hello` — `hello` stays a pure version handshake,
   matching cerastream). A late or reconnecting consumer immediately receives the
   current snapshot, then live deltas. The idle (`connections: []`) and staleness
   (5000 ms) semantics from srtla ADR-001 ride along on the replayed + pushed
   documents unchanged.

2. **"Accept-loop in hot path" (Option-B objection).** srtla ADR-001 worried that a C
   producer running an accept loop + per-client backpressure/`EPIPE`/framing adds
   failure modes inside the sender hot path. **Resolved:** the subscription uses a
   **bounded** model — a fixed-capacity channel per subscriber with a
   **drop-with-log** policy. If a subscriber is slow or its buffer is full, the
   telemetry tick **drops the frame and logs**; it **never blocks the hot path** and
   never applies backpressure to packet forwarding. The control socket already runs
   off the data path (`spawn_config_listener` is its own thread/task), so the push
   loop inherits that isolation.

3. **"Testability" (Option-B objection).** srtla ADR-001 favored Option A because a
   fixture file is trivially testable, whereas a live socket peer is not.
   **Resolved:** JSON-RPC frames are **fixture-testable** — request/response and
   `"event"` notification shapes are plain JSON that unit tests assert against without
   a live socket peer (the same way cerastream's binding tests drive an in-memory fake
   client). The snapshot builder (`build_telemetry_json`) is already unit-tested with a
   fixed timestamp seam; the subscription path reuses it, so the existing golden
   fixtures still pin the wire shape.

4. **"Push not needed at 5s" (Option-B objection).** srtla ADR-001 correctly observed
   that CeraUI's ~5 s broadcast cadence does not *need* sub-second push.
   **Resolved/re-weighed:** that remains true, but the deciding factor is no longer
   latency — it is **engine consistency**. cerastream **already** drives CeraUI over
   exactly this subscribe-events/`"event"` mechanism; making the sender speak the same
   protocol removes a whole bespoke integration path from CeraUI and gives operators
   one mental model for both engines. The consistency value tips the balance that
   srtla ADR-001 — written before cerastream's pattern existed as the house standard —
   left on the "file" side.

## Dual-Support, Capability-Gated Cutover, and Deprecation Timeline

The migration is staged so no consumer breaks at any step:

1. **Add (this ADR).** Ship JSON-RPC `hello`/`get-capabilities`/control-methods/
   `subscribe-events` on `--control-socket`. The text protocol and `--stats-file`
   continue to work unchanged. `get-capabilities` advertises the new methods so CeraUI
   can **feature-detect** the control protocol before using it.
2. **Capability-gated cutover.** CeraUI prefers the JSON-RPC control socket **only when
   `get-capabilities` advertises it**; otherwise it falls back to `--stats-file` +
   text. Fallback also triggers on: connect failure, `hello`/`subscribe-events`
   timeout, mid-stream socket disconnect, a malformed `"event"`, or a
   `schema_version` it does not understand. The stats file stays the safety net for
   the entire dual-support window.
3. **Deprecation (future, versioned).** Only after the JSON-RPC path is the proven
   default across the device fleet may the **text protocol** be considered for
   deprecation — as a deliberate, versioned parity-contract change, announced ahead of
   removal. **`--stats-file` is NOT scheduled for removal** here; it remains the
   Option-A transport for the C sender and for file-based consumers. No transport is
   removed before its replacement is the capability-gated default.

This ADR authorizes step 1 and the step-2 contract. Step 3 is explicitly out of scope
and requires its own versioned change.

## Constraints (carried forward)

- **No TCP / no `0.0.0.0` listener.** The control plane is a **local Unix domain
  socket** only, exactly as `--control-socket` is today. Network exposure of the
  control plane is out of scope and explicitly disallowed.
- **No removal of `--stats-file` or the text protocol before cutover.** Both are
  load-bearing for current consumers and the C sender; they stay through the
  dual-support window (see timeline).
- **Schema is frozen here.** The telemetry JSON schema is srtla ADR-001's; this ADR
  does not add, rename, or retype any telemetry field. `schema_version` remains the
  additive Rust-producer field already defined in the parity contract.
- **Hot path is sacrosanct.** The subscription is bounded + drop-with-log; it must
  never block or backpressure packet forwarding.
