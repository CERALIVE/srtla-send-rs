<!-- Moved verbatim from AGENTS.md on 2026-10-05 by lean-rules-docs-landing-latam -->

# srtla-send-rs

CERALIVE's hard fork of [`irlserver/srtla_send`](https://github.com/irlserver/srtla_send),
the Rust SRTLA bonding sender. It reads local SRT (UDP) on a listen port and forwards it
over one bound UDP socket per source IP to an SRTLA receiver. On the device it is spawned
by CeraUI and feeds the bonded path into `irl-srt-server`.

The rule of this fork is **track upstream, differentiate on top**. The tree is upstream's
code plus a thin, individually committed CERALIVE layer that CeraUI actually consumes.
Everything upstream ships (scheduler, control socket, metrics, re-home, stall deselect)
is used as-is. Do not reintroduce anything from the pre-hard-fork `legacy` branch that is
not listed under PARITY CONTRACT below.

Credits: upstream builds on ideas from [Moblin](https://github.com/eerimoq/moblin) and the
[BELABOX SRTLA reference](https://github.com/BELABOX/srtla). Keep upstream's `LICENSE`
(MIT, Thomas Lekanger) and its credits intact; CERALIVE layers AGPLv3 at distribution.

