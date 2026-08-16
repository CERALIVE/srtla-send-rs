# ADOPTED: `sendmmsg(2)` batch send (was: DEFERRED)

**Status:** ADOPTED. This note is retained as an adoption record so existing links
stay valid; it no longer tracks a deferred item.

- **Adopted in:** the `merge/upstream-2026-08` upstream-sync PR, todo 9
  ("sendmmsg batch flush + unconnected uplink sockets"), porting upstream
  `673138d` *feat(srtla_send): flush batches with sendmmsg* with fork fixes.
- **Triage row:** `docs/notes/upstream-sync-2026-08-evaluation.md` → `673138d`
  (`ADOPT-WITH-FORK-FIX`). That row, not this file, is the authority.

## What shipped

`BatchSender::flush` no longer loops one `send` per queued packet. It submits up
to `BATCH_SEND_SIZE = 32` datagrams per kernel entry via `sendmmsg(2)` on Linux
(`src/connection/batch_recv.rs`), with a sequential fallback capped identically on
other platforms. Uplink sockets are unconnected: `BatchUdpSocket` owns the
resolved peer and names it on every send.

The port also fixed the upstream defects the deferred note's "partial-failure
drain semantics" warning was about:

- **Prefix commit.** `flush` returns a `FlushOutcome { accepted, error }` carrying
  the tracking records for exactly the kernel-accepted datagrams, in queue order.
  Candidates are peeked, transmitted, and only then removed; the unsent suffix is
  retained. A partial send can neither duplicate nor drop a datagram.
- **Short `msg_len` is a hard error.** A datagram is all-or-nothing, so a
  kernel-reported length that differs from the packet length fails at that index
  and the usable prefix is the messages before it. The remainder is never re-sent
  as a fresh datagram.
- **EINTR / WouldBlock.** EINTR retries without clearing readiness; WouldBlock
  clears readiness and awaits writability (returning a short count after partial
  progress rather than blocking).
- **Recovery.** All three flush call sites (connection switch, batch threshold,
  periodic timer) route a hard error into `mark_for_recovery()` plus
  `SequenceTracker::remove_connection()`.

## Invariants that still hold

- **DATA is never padded.** Only control frames route through
  `send_control_padded` (`MIN_CONTROL_PKT_LEN = 32`); padding DATA would corrupt
  the SRT byte stream. The batch path deliberately bypasses it.
- **ADR-002 byte accounting is unchanged.** `bitrate_bps` and `bytes_sent_total`
  are still counted at `queue_data_packet`, not at transmit time.
- **Miri cannot execute the syscall.** The `miri` lane vets only the pure pointer
  logic around `recvmmsg`/`sendmmsg` (`sendmmsg_pointers_rebuilt_after_move`,
  `sendmmsg_prefix_extraction_bounded`), never the live kernel transition.
