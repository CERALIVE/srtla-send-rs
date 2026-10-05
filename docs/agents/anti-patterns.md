<!-- Moved verbatim from AGENTS.md on 2026-10-05 by lean-rules-docs-landing-latam -->

## ANTI-PATTERNS

- **No scheduler or mode work.** No new scheduling modes, no alternative selectors, no
  path-prediction or in-order-delivery pipelines, no experimental window-growth or
  link-exclusion flags. Upstream's `classic`/`enhanced` selector is the scheduler. If a
  ported test references a removed mode, delete the reference; never re-add the mode.
- **No bindings in this repo.** There is no `bindings/` directory, no npm package, and no
  binding tag namespace. The TypeScript sender/telemetry/control helper lives in CeraUI
  as a workspace package and is tested against the real binary there.
- **No auto-sync with upstream** and no transient remote left attached at push time.
- **No second control dialect.** No `hello`, no `subscribe-events`, no kebab-case
  methods, no replay-on-subscribe. Extend upstream's JSON-RPC additively only.
- **Don't unpin or silently bump the toolchain**, and don't "modernize" the `smallvec`
  or `libc` pins.
- **Don't break the parity contract** (binary name, positional order, telemetry shape,
  `bitrate_bps` ×8, additive-only telemetry fields, capability key set, SIGHUP reload,
  empty start, bind ordering) without a deliberate versioned change.
- **Don't strip upstream MIT/credits.** Layer AGPLv3 at distribution only.
- **No path above the repo root in any tracked file.** The repo builds and releases
  standalone in CI; the workspace parent does not exist there.
- **Don't bake `CONN_TIMEOUT = 15` into the binary.** CeraUI passes `--conn-timeout-ms`.
- **Don't add silent source filtering to the uplink sockets.** They are unconnected by
  upstream design (NAT and multi-homed receivers reply from other addresses).

