# Next binding release — BREAKING: sender 4.0.0 scheduling surface

`SchedulingMode` and spawn-option validation now accept only `enhanced`.
`classic`, `rtt-threshold`, `edpf`, and `adaptive` are no longer valid spawn
options. Omit `mode` to use the binary's Enhanced default.

Control responses are not spawn inputs: `get-status` must remain forward- and
backward-compatible with mode strings from other binary versions. The existing
`rawRequest('get-status')` returns `unknown`, not the narrowed spawn union.
The additive `controlStatusSchema` parser and its `GetStatusResult` type expose
`mode: string`, preserving historical and future spellings plus additive fields.
Use `controlStatusSchema.parse(await client.rawRequest('get-status'))`.

Retired quality/exploration/RTT controls remain discoverable but succeed with
additive `deprecated: true, effect: 'none'` fields and do not change configuration.
The binding's independent CalVer release/tag is not cut by the Rust 4.0.0 bump.

## Additive: typed 4.0.0 `get-status` receiver observation and per-link counters

`controlStatusSchema` now declares the additive fields the 4.0.0 sender returns
from `get-status`, all optional so a 3.3.0 binary still parses:

- `receiver?: { nak_report?: boolean; srt_version?: string; rexmit_flag?: boolean }`
  — the bond-wide SRT handshake (HSRSP) observation. `{}` before any handshake;
  each key omitted when unknown. Read `nak_report ?? true` for policy (fail-safe
  NAK-on) but render an absent value as unknown, not on.
- `links?: Array<{ conn_id; iface?; link_id?; health?; priority?; rexmit_forwarded? }>`
  — `rexmit_forwarded` counts SRT DATA forwarded on that link with the
  retransmission bit set; diagnostic only, never in the telemetry file.
- `negotiated_latency_ms?: number`.

`controlReceiverSchema`, `controlLinkSchema`, `ControlReceiver` and `ControlLink`
are exported. Unknown keys still pass through; `mode` stays an open string.
The telemetry reader's optional top-level `receiver_nak_report` (declared in
producer order after `disposition`) is unchanged from the previous release.
See the sender's `docs/adr/ADR-004-bonded-path-convergence.md`.
