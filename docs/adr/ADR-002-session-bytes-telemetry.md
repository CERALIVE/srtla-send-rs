# ADR-002: Cumulative Session-Bytes Telemetry (`bytes_sent_total`)

## Status

Accepted

> **Numbering note.** This is the second ADR in `srtla-send-rs`. It **extends**
> the telemetry JSON schema that `srtla/docs/adr/ADR-001-telemetry-ipc.md`
> defines and that `srtla-send-rs/docs/adr/ADR-001-control-protocol.md` carries
> verbatim onto the JSON-RPC transport. Where this document says "srtla ADR-001"
> it means that other repo's file; "ADR-001" alone means this repo's.

## Context

An operator streaming from a bonded device on metered cellular data has no way
to answer the question "how much data have I transferred?". Everything the
sender publishes today is **instantaneous**: `bitrate_bps` is a rate over a
2-second window, `window`/`in_flight` are point-in-time depths, and `nak_count`
counts events rather than volume. A rate cannot be integrated after the fact by
a consumer with any accuracy — a page reload, a backend restart, a dropped
telemetry tick, or a link that reconnects mid-stream each silently lose an
unknown quantity of bytes from a client-side running sum.

The sender already has the exact number. `BitrateTracker::bytes_sent_total`
(`src/connection/bitrate.rs`) has always accumulated every wire byte handed to
each uplink — it was simply consumed as a *difference* to derive the 2-second
rate and then discarded. Two defects made it unusable as a session total:

1. **It was zeroed on socket replacement.** `BitrateTracker::reset()` is called
   from `SrtlaConnection::reset_state()` on every `reconnect()`. A transient
   radio stall therefore erased the link's entire history — the one event a
   real "total transferred" must survive.
2. **There was no bond-level figure at all.** Only per-link counters existed,
   and the obvious aggregation (sum the live links) is wrong: a SIGHUP IP-list
   reload tears links down, and their bytes would vanish from the sum, making
   the operator's total run **backwards**.

## Decision

**Publish a cumulative wire-byte count at two scopes as an additive
`bytes_sent_total` field, and make it monotonic for the sender process's
lifetime.**

### Schema addition (additive only)

Both scopes use the same name, the same unit, and the same reset rule; the scope
is given by position in the document.

| Field | Type | Units | Notes |
|-------|------|-------|-------|
| `bytes_sent_total` (top level) | integer ≥ 0 | **bytes** | The whole bond's cumulative wire bytes for this session. The authoritative "total transferred". |
| `connections[].bytes_sent_total` | integer ≥ 0 | **bytes** | That uplink's own cumulative wire bytes for this session. |

```json
{"schema_version":1,"last_updated_ms":1749556546000,"connections":[{"conn_id":"0","rtt_ms":42,"nak_count":3,"weight_percent":85,"window":8192,"in_flight":100,"bitrate_bps":2500000,"bytes_sent_total":812000000},{"conn_id":"1","rtt_ms":73,"nak_count":11,"weight_percent":55,"window":4096,"in_flight":240,"bitrate_bps":1200000,"bytes_sent_total":808000000}],"bytes_sent_total":1620000000}
```

**`schema_version` stays `1`.** The change is purely additive: every field srtla
ADR-001 defines keeps its name, type, unit, and meaning, and no consumer of the
old schema is affected. The TS reader validates `schema_version` as
`z.literal(1)`, so bumping it would hard-fail every existing device — a
disproportionate response to adding a field that older consumers ignore.

### Units — the field beside it is NOT the same unit

`bitrate_bps` is **bits per second** and carries a mandatory ×8 conversion from
the producer's internal wire-bytes/s. `bytes_sent_total` is **bytes** and carries
**no multiplication at all**: it is a count, not a rate. The two live next to
each other in the same object, so this is the single most likely place for a
consumer to introduce a factor-of-8 error. The producer's ×8 has exactly one
home (`ConnRecord::from`, `src/telemetry_file.rs`), and the line immediately
below it passes `bytes_sent_total` through verbatim with a comment saying why.

### What is counted

`bytes_sent_total` is incremented at **exactly the same call site** as the input
to `bitrate_bps` — `SrtlaConnection::queue_data_packet()`
(`src/connection/mod.rs`), which calls `BitrateTracker::update_on_send()`. The
two therefore agree by construction rather than by convention:

- **Included:** SRT DATA packets forwarded to an uplink, at their full wire
  length. That length includes the SRT header, so SRT-level **retransmits are
  counted every time they are sent** — a retransmitted packet really does cost
  the operator's data plan twice, and reporting it once would understate the
  bill.
- **Excluded:** SRTLA control frames — keepalives, REG1/REG2/REG3 — which are
  sent via `send_control_padded()` and never touch the bitrate tracker. These
  are a small, roughly constant background cost (a padded keepalive is 38 B at
  1 Hz per link) and excluding them keeps the field consistent with
  `bitrate_bps`, which has always excluded them too.
- Bytes are counted at **queue** time, not at flush time — again matching
  `bitrate_bps`. A packet queued into a batch that then fails to flush is
  counted. The window is at most one 15 ms batch interval, and reporting at
  queue time is what makes the two fields consistent.

### Reset semantics

The rule is one sentence: **the counter's lifetime is the `srtla_send`
process's lifetime.** CeraUI spawns exactly one `srtla_send` per streaming
session, so "process lifetime" *is* "session". Concretely:

| Event | Behaviour | Why |
|-------|-----------|-----|
| **Per-link socket replacement** (`reconnect()` → `reset_state()` → `BitrateTracker::reset()`) | **Does NOT reset.** | A radio stall is a transient link event, not a new session. `reset()` now rebases the rate window to the current total instead of zeroing both, so rate behaviour is unchanged while the cumulative survives. |
| **SIGHUP IP-list reload that drops a link** | **Does NOT reset**, and does not subtract. | The bond total keeps the departed link's bytes. |
| **SIGHUP reload that adds/re-adds a link** | Adds on top. | A re-added IP returns as a *new* connection with a fresh `conn_id` and a counter at 0; its bytes accrue on top of the banked total. |
| **CeraUI backend restart with the stream still up** | **Survives.** | The backend does not own the counter; `srtla_send` does. A backend that re-adopts a running stream re-reads the live stats file and sees the true total, exactly like every other adopted stream field. |
| **Stream stop → new stream start** | **Resets to 0.** | A new session spawns a new `srtla_send` process. |
| **Sender crash / respawn** | Resets to 0. | Honest: the bytes sent by the dead process are genuinely unrecoverable, and reporting a fabricated carry-over would be worse than restarting. |

### Monotonicity mechanism

The bond total is **not** `connections.map(bytes_sent_total).sum()`. It is a
session accumulator (`SessionBytes`, `src/stats.rs`) that banks each link's
**delta** since the previous observation, keyed by the connection's stable
`conn_id`:

```text
for each live connection:
    delta  = link_total - last_seen[conn_id]      (saturating; never negative)
    total += delta
    last_seen[conn_id] = link_total
forget last_seen entries whose conn_id is no longer live
```

Forgetting a departed link — rather than subtracting it out — is precisely what
makes the total monotonic across a reload. Bookkeeping stays bounded because the
map is pruned to the live set on every observation.

The accumulator is advanced in `SharedStats::update()`, which the sender's
housekeeping tick calls **before** it applies any queued SIGHUP connection
changes. A link removed by a reload therefore always has its final delta banked
on the tick that precedes its teardown.

### Consumer contract

`bytes_sent_total` is `z.number().int().min(0).optional()` at both scopes in
`@ceralive/srtla-send`'s `telemetrySchema`. **Absent means UNKNOWN, never
zero** — the same convention CeraUI already applies to `bitrate_bps`. A device
running a sender that predates this ADR omits the field, and its consumer must
render an unknown state rather than "0 B", which would be a lie.

The C `srtla_send` (deprecated, receiver-only package) does not and will not
emit this field; that is why the schema entry is optional rather than required.

## Consequences

- srtla ADR-001 is **not amended**. It is the canonical schema record for the
  **C** producer, which is deprecated and will never emit this field. Amending
  it would assert a contract its own producer does not honour. This ADR is the
  Rust-sender-specific extension, exactly as ADR-001 (control protocol) is the
  Rust-sender-specific transport supersession.
- ADR-001's "Schema is frozen here" constraint stands as written — that ADR adds
  no field. This ADR is the deliberate, versioned change that constraint asks
  for.
- Both golden fixtures (`tests/fixtures/telemetry-golden.json` and
  `bindings/typescript/tests/fixtures/telemetry-golden.json`) grow the new keys
  and must stay byte-identical; `tests/telemetry_fixture_parity.rs` asserts the
  frozen ADR-001 key set as a **subset** so the schema can only ever grow, and
  the full current key set for equality so a new field cannot land silently.
- **A consumer only sees the field once `@ceralive/srtla-send` is republished.**
  The Zod reader strips unknown keys, so a CeraUI pinned to a pre-ADR-002
  binding reads `undefined` — correct, but UNKNOWN rather than live. Shipping
  the operator-visible figure therefore requires a binding release
  (`bindings-vYYYY.M.P`) in addition to a sender release.
