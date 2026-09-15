#!/usr/bin/env bash
set -euo pipefail

# Run from the checkout to build. stdout is one TSV row: artifact path, SHA-256.
root=$(GIT_MASTER=1 git rev-parse --show-toplevel)
cd "$root"
revision=$(GIT_MASTER=1 git rev-parse HEAD)
require_clean() {
  if [[ -n $(GIT_MASTER=1 git status --porcelain --untracked-files=all) ]]; then
    printf 'build_candidate: refusing dirty worktree (including untracked files)\n' >&2
    exit 1
  fi
  if [[ $(GIT_MASTER=1 git rev-parse HEAD) != "$revision" ]]; then
    printf 'build_candidate: source revision changed during build\n' >&2
    exit 1
  fi
}
require_clean

BENCH_ARTIFACT_DIR=${BENCH_ARTIFACT_DIR:-${TMPDIR:-/tmp}/srtla-bench}
mkdir -p "$BENCH_ARTIFACT_DIR"
BENCH_ARTIFACT_DIR=$(realpath "$BENCH_ARTIFACT_DIR")
mkdir -p "$BENCH_ARTIFACT_DIR/bins"
# Serialize builds sharing target-ti; never publish a hard link to Cargo's output.
exec 9>"$BENCH_ARTIFACT_DIR/build-candidate.lock"
flock 9
require_clean
CARGO_TARGET_DIR="$BENCH_ARTIFACT_DIR/target-ti" cargo build --release --features test-internals >&2
require_clean

pending=$(mktemp "$BENCH_ARTIFACT_DIR/bins/.candidate-XXXXXX")
trap 'rm -f -- "$pending"' EXIT
install -m 0555 "$BENCH_ARTIFACT_DIR/target-ti/release/srtla_send" "$pending"
digest=$(sha256sum "$pending")
digest=${digest%% *}
artifact="$BENCH_ARTIFACT_DIR/bins/srtla_send-$revision-${digest:0:16}"
# link(2) publishes without replacing an existing artifact, even in a race.
if ! ln "$pending" "$artifact" 2>/dev/null; then
  existing=$(sha256sum "$artifact")
  if [[ ${existing%% *} != "$digest" || -L "$artifact" || ! -x "$artifact" ]]; then
    printf 'build_candidate: artifact collision: %s\n' "$artifact" >&2
    exit 1
  fi
fi
printf '%s\t%s\n' "$artifact" "$digest"
