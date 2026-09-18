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
