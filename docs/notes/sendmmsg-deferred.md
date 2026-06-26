# DEFERRED: `sendmmsg(2)` batch send

**Status:** Deferred (tracked, not planned). Do **not** implement without profiling
evidence and a deliberate PR.

**Location of the live note:** `src/connection/batch_send.rs` `flush()` (the
`// DEFERRED: sendmmsg(2) batch send` comment block) and the
AGENTS.md "TEST HARDENING (Tasks 7-8, 25) → Task 25 — sendmmsg triage" section.

## What it would do

`sendmmsg(2)` is a Linux-only syscall that submits **multiple UDP datagrams in a
single kernel entry**, amortizing the per-`send` syscall transition over a whole
batch. Today `BatchSender::flush()` loops `socket.send(packet).await` once per
queued packet, so a flush of N packets is N syscalls.

## Why it is deferred

- **Marginal gain at current rates.** Packet batching (16-packet / 15 ms flush,
  Moblin-inspired) already cuts syscalls ~15x — from ~960 syscalls/s per
  connection to **~60-67 batch flushes/s at 10 Mbps**. `sendmmsg` would collapse
  each flush's inner loop, but the flush rate itself is already low.
- **Linux-only `unsafe` FFI.** `sendmmsg` is not in the stable Tokio surface; using
  it means OS-specific `unsafe` code and a non-portable code path, raising the
  maintenance and soundness cost (cf. the `recvmmsg` Miri lane in `ci.yml`).
- **No profiling justification.** There is no measurement on the **Jetson Nano**
  device target showing the per-packet `send` syscall is a bottleneck. Optimizing
  without that evidence is speculative.

## When to revisit

- Profiling on the device target shows syscall overhead in `flush()` is a real
  bottleneck, **or**
- Tokio gains native, safe `sendmmsg` support that removes the `unsafe`/portability
  cost.

Any future work must keep the existing **DATA-is-never-padded** invariant (only
control frames route through `send_control_padded`; padding DATA would corrupt the
SRT byte stream) and the partial-failure drain semantics in the current `flush()`.
