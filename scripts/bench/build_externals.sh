#!/usr/bin/env bash
set -euo pipefail

readonly ART="${BENCH_ARTIFACT_DIR:-${TMPDIR:-/tmp}/srtla-bench}"
readonly EXTERNALS="$ART/externals"
readonly UPSTREAM_SHA="df0b3938791ff24eced4aed8b29e3d49d0efb639"
readonly BELABOX_SHA="37862da3d0c13b46956efd3f88877053293d97d6"
readonly UPSTREAM_SOURCE="https://github.com/irlserver/srtla_send.git"
readonly BELABOX_SOURCE="https://github.com/BELABOX/srtla.git"
readonly IRLSERVER_SOURCE="https://github.com/irlserver/srtla.git"

REPO_ROOT="$(GIT_MASTER=1 git rev-parse --show-toplevel)"
PINNED_TOOLCHAIN="$(grep '^channel[[:space:]]*=' "$REPO_ROOT/rust-toolchain.toml" | cut -d'"' -f2)"

if [[ -z "$PINNED_TOOLCHAIN" ]]; then
  printf 'could not resolve the pinned Rust toolchain\n' >&2
  exit 1
fi

if [[ -z "${SRTLA_REPO:-}" ]]; then
  printf 'SRTLA_REPO must name the CeraLive receiver checkout\n' >&2
  exit 1
fi

if [[ ! -f "$SRTLA_REPO/src/receiver_main.cpp" ]]; then
  printf 'SRTLA_REPO does not contain src/receiver_main.cpp: %s\n' "$SRTLA_REPO" >&2
  exit 1
fi

mkdir -p "$EXTERNALS"
readonly SMOKE_LOG="$EXTERNALS/smoke.log"
readonly MANIFEST="$EXTERNALS/externals.json"
readonly SMOKE_IPS="$EXTERNALS/upstream-smoke-ips.txt"
: > "$SMOKE_LOG"
printf '127.0.0.1\n' > "$SMOKE_IPS"

clone_repo() {
  local name="$1"
  local source="$2"
  local destination="$3"

  if [[ -e "$destination" && ! -d "$destination/.git" ]]; then
    printf '%s exists but is not a git checkout: %s\n' "$name" "$destination" >&2
    return 1
  fi

  if [[ -d "$destination/.git" ]]; then
    GIT_MASTER=1 git -C "$destination" fetch --tags origin
  else
    GIT_MASTER=1 git clone "$source" "$destination"
  fi
}

checkout_exact() {
  local checkout="$1"
  local expected_sha="$2"
  local actual_sha

  GIT_MASTER=1 git -C "$checkout" checkout --detach "$expected_sha"
  actual_sha="$(GIT_MASTER=1 git -C "$checkout" rev-parse HEAD)"
  if [[ "$actual_sha" != "$expected_sha" ]]; then
    printf 'pinned checkout mismatch: expected %s, got %s\n' "$expected_sha" "$actual_sha" >&2
    return 1
  fi
}

copy_executable() {
  local source="$1"
  local destination="$2"

  if [[ ! -x "$source" ]]; then
    printf 'expected executable is missing: %s\n' "$source" >&2
    return 1
  fi
  install -m 0755 "$source" "$destination"
}

find_built_receiver() {
  local build_directory="$1"
  local matches=()
  local candidate

  shopt -s globstar nullglob
  for candidate in "$build_directory"/**/srtla_rec; do
    if [[ -x "$candidate" ]]; then
      matches+=("$candidate")
    fi
  done
  shopt -u globstar nullglob

  if [[ ${#matches[@]} -ne 1 ]]; then
    printf 'expected one srtla_rec under %s, found %s\n' "$build_directory" "${#matches[@]}" >&2
    return 1
  fi
  printf '%s\n' "${matches[0]}"
}

smoke_bind() {
  local name="$1"
  local binary="$2"
  shift 2
  local pid
  local bound=false
  local deadline
  local exit_code

  if ss -lun | grep -q '15000'; then
    printf '%s: UDP port 15000 is already in use\n' "$name" | tee -a "$SMOKE_LOG" >&2
    return 1
  fi

  timeout --foreground --kill-after=1s 2s "$binary" "$@" >> "$SMOKE_LOG" 2>&1 &
  pid=$!
  deadline=$((SECONDS + 2))
  while ((SECONDS < deadline)); do
    if ss -lun | grep '15000' >> "$SMOKE_LOG"; then
      printf '%s: bound UDP port 15000\n' "$name" | tee -a "$SMOKE_LOG"
      bound=true
      break
    fi
    sleep 0.1
  done

  if wait "$pid"; then
    exit_code=0
  else
    exit_code=$?
  fi

  if [[ "$bound" != true ]]; then
    printf '%s: did not bind UDP port 15000\n' "$name" | tee -a "$SMOKE_LOG" >&2
    return 1
  fi
  if [[ "$exit_code" -ne 124 && "$exit_code" -ne 137 ]]; then
    printf '%s: exited before the 2s smoke timeout (status %s)\n' "$name" "$exit_code" | tee -a "$SMOKE_LOG" >&2
    return 1
  fi
}

write_manifest() {
  python3 - "$MANIFEST" \
    'upstream-srtla_send' 'upstream' "$UPSTREAM_BINARY" "$(sha256sum "$UPSTREAM_BINARY" | cut -d' ' -f1)" "$UPSTREAM_SOURCE" "$UPSTREAM_SHA" 'positional' 'success' "$UPSTREAM_TOOLCHAIN" \
    'belabox-srtla_rec' 'belabox' "$BELABOX_BINARY" "$(sha256sum "$BELABOX_BINARY" | cut -d' ' -f1)" "$BELABOX_SOURCE" "$BELABOX_SHA" 'positional' 'success' '' \
    'irlserver-srtla_rec' 'irlserver' "$IRLSERVER_BINARY" "$(sha256sum "$IRLSERVER_BINARY" | cut -d' ' -f1)" "$IRLSERVER_SOURCE" "$IRLSERVER_SHA" 'flags' 'success' '' \
    'ceralive-srtla_rec' 'ceralive' "$CERALIVE_BINARY" "$(sha256sum "$CERALIVE_BINARY" | cut -d' ' -f1)" "$SRTLA_REPO" "$CERALIVE_SHA" 'flags' 'success' '' <<'PY'
import json
import sys

keys = (
    'name', 'kind', 'path', 'sha256', 'source', 'sha', 'argv_form', 'build_status',
    'toolchain',
)
values = sys.argv[2:]
entries = []
for offset in range(0, len(values), len(keys)):
    entry = dict(zip(keys, values[offset : offset + len(keys)], strict=True))
    if not entry['toolchain']:
        del entry['toolchain']
    entries.append(entry)

with open(sys.argv[1], 'w', encoding='utf-8') as manifest:
    json.dump(entries, manifest, indent=2)
    manifest.write('\n')
PY
  cat "$MANIFEST"
}

UPSTREAM_CHECKOUT="$EXTERNALS/upstream-srtla-send-source"
UPSTREAM_TARGET="$EXTERNALS/upstream-target"
UPSTREAM_BUILD_RECORD="$EXTERNALS/upstream-build.txt"
clone_repo upstream "$UPSTREAM_SOURCE" "$UPSTREAM_CHECKOUT"
checkout_exact "$UPSTREAM_CHECKOUT" "$UPSTREAM_SHA"
UPSTREAM_TOOLCHAIN="$(RUSTUP_TOOLCHAIN="$PINNED_TOOLCHAIN" rustup show active-toolchain)"
{
  printf 'pinned_toolchain=%s\n' "$PINNED_TOOLCHAIN"
  printf 'active_toolchain=%s\n' "$UPSTREAM_TOOLCHAIN"
} > "$UPSTREAM_BUILD_RECORD"
if RUSTUP_TOOLCHAIN="$PINNED_TOOLCHAIN" CARGO_TARGET_DIR="$UPSTREAM_TARGET" cargo build --release --manifest-path "$UPSTREAM_CHECKOUT/Cargo.toml"; then
  printf 'status=success\n' >> "$UPSTREAM_BUILD_RECORD"
else
  printf 'status=failure\n' >> "$UPSTREAM_BUILD_RECORD"
  exit 1
fi
UPSTREAM_BINARY="$UPSTREAM_TARGET/release/srtla_send"
copy_executable "$UPSTREAM_BINARY" "$EXTERNALS/upstream-srtla_send"
UPSTREAM_BINARY="$EXTERNALS/upstream-srtla_send"

BELABOX_CHECKOUT="$EXTERNALS/belabox-srtla"
clone_repo belabox "$BELABOX_SOURCE" "$BELABOX_CHECKOUT"
checkout_exact "$BELABOX_CHECKOUT" "$BELABOX_SHA"
make -C "$BELABOX_CHECKOUT"
BELABOX_BINARY="$EXTERNALS/belabox-srtla_rec"
copy_executable "$BELABOX_CHECKOUT/srtla_rec" "$BELABOX_BINARY"

IRLSERVER_CHECKOUT="$EXTERNALS/irlserver-srtla"
IRLSERVER_BUILD="$EXTERNALS/irlserver-build"
clone_repo irlserver "$IRLSERVER_SOURCE" "$IRLSERVER_CHECKOUT"
GIT_MASTER=1 git -C "$IRLSERVER_CHECKOUT" fetch origin main
GIT_MASTER=1 git -C "$IRLSERVER_CHECKOUT" checkout --detach origin/main
IRLSERVER_SHA="$(GIT_MASTER=1 git -C "$IRLSERVER_CHECKOUT" rev-parse HEAD)"
GIT_MASTER=1 git -C "$IRLSERVER_CHECKOUT" submodule update --init --recursive
cmake -B "$IRLSERVER_BUILD" -S "$IRLSERVER_CHECKOUT"
cmake --build "$IRLSERVER_BUILD" -j
IRLSERVER_BINARY="$EXTERNALS/irlserver-srtla_rec"
copy_executable "$(find_built_receiver "$IRLSERVER_BUILD")" "$IRLSERVER_BINARY"

CERALIVE_BUILD="$EXTERNALS/ceralive-build"
CERALIVE_SHA="$(GIT_MASTER=1 git -C "$SRTLA_REPO" rev-parse HEAD)"
cmake -B "$CERALIVE_BUILD" -S "$SRTLA_REPO"
cmake --build "$CERALIVE_BUILD" -j
CERALIVE_BINARY="$EXTERNALS/ceralive-srtla_rec"
copy_executable "$(find_built_receiver "$CERALIVE_BUILD")" "$CERALIVE_BINARY"

smoke_bind upstream-srtla_send "$UPSTREAM_BINARY" 15000 127.0.0.1 15001 "$SMOKE_IPS"
smoke_bind belabox-srtla_rec "$BELABOX_BINARY" 15000 127.0.0.1 15001
smoke_bind irlserver-srtla_rec "$IRLSERVER_BINARY" --srtla_port 15000 --srt_hostname 127.0.0.1 --srt_port 15001
smoke_bind ceralive-srtla_rec "$CERALIVE_BINARY" --srtla_port 15000 --srt_hostname 127.0.0.1 --srt_port 15001

write_manifest
