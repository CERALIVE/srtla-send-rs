# Keepalive Interop & Compatibility Note

This note documents a deliberate, known divergence between this fork's keepalive
wire format and the original **BELABOX** SRTLA sender, why the two are expected
to interoperate, and the one assumption that interop rests on. It is the
authoritative reference for the keepalive-interop conformance tests
(`src/tests/keepalive_interop_tests.rs`).

## The divergence

There are two keepalive lineages in the SRTLA ecosystem:

| Lineage | Keepalive frame | Size | Carries a timestamp? |
|---|---|---|---|
| **BELABOX** (`BELABOX/srtla`) | type only — `htobe16(SRTLA_TYPE_KEEPALIVE)` | **2 bytes** | No |
| **This fork** (from `irlserver/srtla_send`) | standard: type + `u64` ms timestamp | **10 bytes** | Yes |
| **This fork** (extended) | standard 10 B + `0xC01F`-tagged `ConnectionInfo` trailer | **38 bytes** | Yes (+ telemetry) |

- BELABOX sends a **bare 2-byte keepalive** — the packet type and nothing else.
  See `BELABOX/srtla` `srtla_send.c`
  ([L589-L593](https://github.com/BELABOX/srtla/blob/37862da3d0c13b46956efd3f88877053293d97d6/srtla_send.c#L589-L593)):
  it packs `htobe16(SRTLA_TYPE_KEEPALIVE)` and sends those 2 bytes.
- This fork inherited irlserver's **timestamped** keepalive. The sender always
  emits the backwards-compatible **extended 38-byte** form
  (`create_keepalive_packet_ext`), which layers a `ConnectionInfo` telemetry
  trailer (magic `0xC01F`, version `0x0001`) behind the standard 10-byte
  `type + timestamp` header. The plain 10-byte form (`create_keepalive_packet`)
  remains as the minimal timestamped frame. The full extended layout is
  specified in [`EXTENDED_KEEPALIVE.md`](./EXTENDED_KEEPALIVE.md).

The timestamp is what lets this fork measure RTT directly off keepalives: the
sender stamps `now_ms()` into bytes 2-9, the receiver echoes the frame back, and
the sender computes `RTT = now - echoed_timestamp`
(`RttTracker::handle_keepalive_response`). A bare BELABOX keepalive carries no
timestamp, so no RTT can be derived from a bare echo.

## The receiver-echo interop ASSUMPTION

SRTLA receivers echo received keepalive bytes back to the sender; that echo is
the round-trip signal this fork times. The assumption that makes our extended
keepalive safe against a BELABOX-style receiver is:

> **ASSUMPTION:** an SRTLA receiver treats a keepalive as an opaque,
> echo-on-receipt control frame and tolerates extra trailing bytes — i.e. it
> reads the 2-byte type, does not length-validate the keepalive, and echoes
> what it received (or replies with its own keepalive) without rejecting the
> 10-/38-byte frame.

This holds for the irlserver and CeraLive receivers (the extended format is
designed to be length-agnostic — old receivers read bytes 0-9 and ignore the
rest; see the compatibility matrix in `EXTENDED_KEEPALIVE.md`). It is
**believed** to hold for BELABOX receivers but is **NOT proven here** — verifying
it requires a live BELABOX receiver, which the in-process test suite does not
have. Until that live interop test exists, treat "a BELABOX receiver accepts our
38-byte keepalive" as an assumption, not a guarantee.

What the tests in this repo *do* prove, in-process and deterministically:

1. **`keepalive_extended_round_trip`** — our extended keepalive builds → parses
   with every `ConnectionInfo` field (incl. `rtt_ms`) preserved, and the
   timestamp at bytes 2-9 yields a correct RTT measurement through the real
   receive path even with the 28-byte extended trailer.
2. **`keepalive_bare_2byte_accepted`** — a bare 2-byte BELABOX-style keepalive
   echo is recognised as a keepalive and handled **defensively**: no timestamp,
   no telemetry, no RTT sample, no panic, and the keepalive-wait flag is cleared
   so the cycle is not wedged.
3. **`keepalive_truncated_graceful`** — every truncated length (0–64 B) and an
   oversized (MTU-sized) frame parse without panicking, returning `None`/empty
   per the length contract.

These cover the *sender's* side of interop: we correctly produce/consume our own
extended keepalive, and we never choke on a bare or malformed echo from the
other lineage.

## Deferred follow-up: a 2-byte-compat keepalive mode

This task **intentionally does not** add a BELABOX-compatible 2-byte keepalive
*send* mode (i.e. an option to emit bare 2-byte keepalives for receivers that
might reject the extended form). That is a **deferred follow-up**, to be picked
up only if live interop testing against a real BELABOX receiver shows the
receiver-echo assumption above does **not** hold for the extended frame.

Constraints that a future 2-byte-compat mode must respect (and that this task
preserves unchanged):

- **The keepalive wire format is unchanged.** The sender still emits the
  extended 38-byte keepalive; the standard 10-byte and extended 38-byte builders
  are untouched.
- **The 32-byte NAT-padding floor (`MIN_CONTROL_PKT_LEN`) is unchanged.** A
  hypothetical bare 2-byte keepalive would still be zero-padded to 32 bytes on
  the wire by `send_control_padded` (parity with the C `pad_sendto`), so it would
  not be a literal 2-byte datagram on the wire regardless.
- **RTT measurement is not weakened.** A bare keepalive carries no timestamp and
  thus yields no RTT; any compat mode would trade RTT-from-keepalive for wire
  compatibility, and must make that trade explicit rather than silently
  degrading the existing timestamped path.

## References

- [`EXTENDED_KEEPALIVE.md`](./EXTENDED_KEEPALIVE.md) — full extended keepalive layout & compatibility matrix
- `src/protocol/builders.rs` — `create_keepalive_packet` (10 B), `create_keepalive_packet_ext` (38 B)
- `src/protocol/parsers.rs` — `extract_keepalive_timestamp`, `extract_keepalive_conn_info`
- `src/connection/rtt.rs` — `RttTracker::handle_keepalive_response` (RTT from echoed timestamp)
- `src/tests/keepalive_interop_tests.rs` — the three conformance tests
- BELABOX bare keepalive: `srtla_send.c` [L589-L593](https://github.com/BELABOX/srtla/blob/37862da3d0c13b46956efd3f88877053293d97d6/srtla_send.c#L589-L593)
