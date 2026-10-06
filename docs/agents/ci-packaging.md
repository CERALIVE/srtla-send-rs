<!-- Moved verbatim from AGENTS.md on 2026-10-05 by lean-rules-docs-landing-latam -->

## CI / PACKAGING

Two workflows. Both build on the pinned nightly; neither publishes a binding.

- **`ci.yml`** (push/PR): the gate above, the `loom` and `miri` lanes, upstream's
  stable/beta/Windows/macOS lanes (they call `cargo +<channel>` explicitly so the pin
  does not shadow them), and a `build-deb` matrix that cross-compiles
  `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-gnu` and packages each `.deb`
  so a packaging break is caught before any tag. The CI `.deb` job runs on
  `ubuntu-latest` on purpose; its artifacts are never shipped.
- **`release.yml`** (tag push `v*`): the full gate plus `loom` and `miri`, then
  `build-deb` inside **`debian:bookworm-slim`** for both arches. Each ELF's versioned
  imports are inspected and anything above **`GLIBC_2.36`** (Debian 12, the device
  image) fails the job. Both `.deb`s and `.sha256`s are attached to the GitHub release
  and the APT reindex is dispatched for component `srtla`. Never move the release build
  back onto the raw runner userspace.

`ci/build-deb.sh` is the single source of truth for the package: **Package `srtla`**,
binary at `/usr/bin/srtla_send`, `Architecture` `arm64`/`amd64`, filename
**`srtla_<ver>_<arch>.deb`** (it re-runs the image pipeline's `*${ARCH}*.deb` fetch glob
as a self-test), `Conflicts`/`Replaces` on the retired transitional package name. The
version comes from `Cargo.toml` (**`4.1.0`**), follows upstream's semver line (not
CalVer), and a tag build must be `v<version>`; the script rejects a mismatch.

aarch64 cross-build: linker `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc`,
apt `gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libc6-dev-arm64-cross binutils-aarch64-linux-gnu pkg-config`,
`PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig`.

`mimalloc` is the global allocator behind a default-on feature; `--no-default-features`
is the system-allocator build for Miri and profiling. `docs/notes/mimalloc-decision.md`.

