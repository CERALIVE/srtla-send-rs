# Agent contracts

Verbatim sections relocated from the repository manifest. Read the applicable contract before changing its subsystem.

| File | Original heading | Governed paths / scope |
|---|---|---|
| [overview.md](overview.md) | Overview | Fork policy and integration |
| [upstream-relationship.md](upstream-relationship.md) | UPSTREAM RELATIONSHIP | `docs/notes/upstream-hardfork-2026-09.md` |
| [pinned-toolchain.md](pinned-toolchain.md) | PINNED TOOLCHAIN | `rust-toolchain.toml` |
| [parity-contract-ceraui-depends-on-every-bullet-change-only-w.md](parity-contract-ceraui-depends-on-every-bullet-change-only-w.md) | PARITY CONTRACT (CeraUI depends on every bullet; change only with a versioned decision) | `crates/srtla-protocol/src/constants.rs`, `docs/CONTROL_PROTOCOL.md`, `docs/adr/ADR-003-bind-map-contract.md`, `docs/adr/ADR-004-control-dialect.md`, `src/bind_map/report.rs`, `src/capabilities.rs`, `src/control.rs`, `src/net/egress.rs`, `src/net/route.rs`, `src/net/socket.rs`, `src/net/spec.rs`, `src/sender/connections.rs`, `src/sender/egress_tick.rs`, `src/sender/links.rs`, `src/sender/reload.rs`, `src/sender/uplink.rs`, `src/stats.rs`, `src/subscriptions.rs`, `src/telemetry_doc.rs`, `src/telemetry_file.rs`, `src/version.rs`, `tests/bind_map_contract.rs`, `tests/capabilities_probe.rs`, `tests/cli_surface.rs`, `tests/fixtures/*.json`, `tests/signal_parity.rs`, `tests/startup_bind_ordering.rs`, `tests/telemetry_fixtures.rs` |
| [build-gate.md](build-gate.md) | BUILD / GATE | `README.md`, `crates/network-sim/src/twin/`, `rust-toolchain.toml`, `scripts/check-doc-refs.sh`, `scripts/deb_version_ordering_test.sh`, `scripts/netns_test_gate.sh`, `scripts/release_version_contract_test.sh`, `scripts/release_workflow_contract_test.py`, `scripts/rust_cache_contract_test.py`, `scripts/workflow_authority_contract_test.py`, `src/net/batch_recv.rs`, `src/subscriptions.rs`, `tests/netns_*.rs`, `tests/netns_twin.rs`, `tests/subscription_loom.rs` |
| [ci-packaging.md](ci-packaging.md) | CI / PACKAGING | `Cargo.toml`, `ci/build-deb.sh`, `docs/notes/mimalloc-decision.md` |
| [codebase-upstream-layout-the-ceralive-additions.md](codebase-upstream-layout-the-ceralive-additions.md) | CODEBASE (upstream layout + the CERALIVE additions) | `ci/build-deb.sh`, `crates/network-sim/`, `crates/srtla-core/`, `crates/srtla-protocol/`, `docs/adr/`, `src/bind_map/`, `src/capabilities.rs`, `src/control.rs`, `src/main.rs`, `src/net/`, `src/sender/`, `src/stats.rs`, `src/subscriptions.rs`, `src/telemetry_doc.rs`, `src/telemetry_file.rs`, `src/version.rs` |
| [anti-patterns.md](anti-patterns.md) | ANTI-PATTERNS | Fork policy and integration |
| [docs-discipline-rule-a.md](docs-discipline-rule-a.md) | DOCS DISCIPLINE (Rule A) | `README.md`, `scripts/check-doc-refs.sh` |
