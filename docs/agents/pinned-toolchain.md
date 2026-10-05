<!-- Moved verbatim from AGENTS.md on 2026-10-05 by lean-rules-docs-landing-latam -->

## PINNED TOOLCHAIN

`rust-toolchain.toml` pins an exact nightly, `nightly-2026-06-12`, with `rustfmt`,
`clippy`, `rust-src`, and the `aarch64-unknown-linux-gnu` target. Nightly is mandatory:
`rustfmt.toml` uses unstable features (edition 2024, `group_imports`, `format_strings`).
Upstream floats on `nightly`; the fork pins a date so the device image is reproducible.
Bump only in a deliberate toolchain or upstream-merge PR, and re-run the full gate after.

Two dependency pins are load-bearing and must survive an upstream merge or a
`cargo upgrade` sweep: `smallvec = "=2.0.0-alpha.12"` (exact; pre-release line) and
`libc = "0.2"` (never a `1.0` pre-release).

