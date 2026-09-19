#!/usr/bin/env bash
set -euo pipefail

[[ $# == 0 || ( $# == 1 && $1 == --sls ) ]] || {
  printf 'usage: %s [--sls]\n' "$0" >&2; exit 2;
}

# Host-local artifacts only; no installs into /usr, source patches, or repo remotes.
ROOT="$(GIT_MASTER=1 git rev-parse --show-toplevel)"
ART="${BENCH_ARTIFACT_DIR:-/home/andres/.cache/opencode/tmp/srtla-bench}"
LOCK="${BENCH_RECEIVERS_LOCK:-$ROOT/scripts/bench/receivers.lock.json}"
SRT_WORKTREE="${SRT_WORKTREE:-$(dirname "$ROOT")/srt-bonded-path-convergence}"
JOBS="${BENCH_BUILD_JOBS:-4}"
SINK="$ROOT/crates/network-sim/tools/srt-sink-min.cpp"
readonly OLD=b06fdb6b85937f3f5cf5452b150a6bb7e35b0226
readonly PROD=f2297192ce9ab572464e84228efbc46f8c1eabf4
readonly NEXT=164d51bb24e258285d1a99c881b55af7e9a94894
readonly BELABOX=37862da3d0c13b46956efd3f88877053293d97d6
readonly UPSTREAM=df0b3938791ff24eced4aed8b29e3d49d0efb639
for tool in git cmake make g++ jq sha256sum flock; do command -v "$tool" >/dev/null; done
[[ "$ART" = /* && "$JOBS" =~ ^[1-9][0-9]*$ ]] || { printf 'absolute ART and positive JOBS required\n' >&2; exit 1; }
mkdir -p "$ART"
ART="$(realpath "$ART")"
case "$ART/" in "$ROOT/"*) printf 'artifact root must be outside the checkout\n' >&2; exit 1;; esac
mkdir -p "$ART/src" "$ART/lineage"
exec 9>"$ART/build-receivers.lock"
flock 9
[[ -f "$LOCK" ]] || { printf 'missing lock skeleton: %s\n' "$LOCK" >&2; exit 1; }
jq -e '.schema_version == 1 and (.receivers | type == "object") and (.candidates | type == "object")' "$LOCK" >/dev/null
TMP="$(mktemp "$LOCK.XXXXXX")"
trap 'rm -f "$TMP"' EXIT

digest() { sha256sum "$1" | cut -d ' ' -f 1; }
publish() {
  if ! cmp -s "$TMP" "$LOCK"; then mv "$TMP" "$LOCK"; fi
}
checkout() {
  if [[ ! -e "$checkout" ]]; then
    GIT_MASTER=1 git clone --no-checkout "$source" "$checkout"
  fi
  [[ -d "$checkout/.git" && ! -L "$checkout" ]] || { printf 'not a scratch clone: %s\n' "$checkout" >&2; exit 1; }
  [[ "$(GIT_MASTER=1 git -C "$checkout" remote get-url origin)" == "$source" ]]
  # Old production commits may be reachable only by explicit SHA fetch.
  if ! GIT_MASTER=1 git -C "$checkout" cat-file -e "$sha^{commit}" 2>/dev/null; then
    GIT_MASTER=1 git -C "$checkout" fetch origin "$sha"
  fi
  # Never reset an existing checkout; detach only if it has no index yet.
  if [[ ! -e "$checkout/.git/index" ]]; then
    GIT_MASTER=1 git -C "$checkout" checkout --detach "$sha"
  fi
  [[ "$(GIT_MASTER=1 git -C "$checkout" rev-parse HEAD)" == "$sha" ]]
  GIT_MASTER=1 git -C "$checkout" diff --quiet HEAD --
}
current() {
  jq -e --arg n "$name" --arg sha "$sha" --arg p "$binary" --arg f "$fingerprint" '
    .artifacts[$n] | .source_sha == $sha and .path == $p and
    .fingerprint == $f and .emulated == false' "$LOCK" >/dev/null || return 1
  local checks
  checks="$(jq -r --arg n "$name" '.artifacts[$n] | ., .dependencies[]? | "\(.sha256)  \(.path)"' "$LOCK")"
  sha256sum --check --status <<< "$checks" || return 1
  printf '%s: up-to-date (locked sha256 verified)\n' "$name"
}
record() {
  local dependencies='[]' file
  for file in "$@"; do
    dependencies="$(jq --arg p "$file" --arg h "$(digest "$file")" --arg s "$sha" \
      '. + [{path:$p,sha256:$h,source_sha:$s,emulated:false}]' <<< "$dependencies")"
  done
  jq --arg n "$name" --arg p "$binary" --arg h "$(digest "$binary")" \
    --arg s "$sha" --arg repo "$source" --arg f "$fingerprint" --arg method "$method" \
    --argjson elapsed "$((SECONDS - started))" --argjson deps "$dependencies" '
    .artifacts[$n] = {path:$p,sha256:$h,source_sha:$s,source:$repo,emulated:false,
      fingerprint:$f,build_method:$method,build_seconds:$elapsed,dependencies:$deps}' "$LOCK" > "$TMP"
  publish
  printf '%s: built sha256=%s (%ss)\n' "$name" "$(digest "$binary")" "$((SECONDS - started))"
}

# SLS is deliberately NOT a metric receiver entry: the campaign only accepts slt.
# SLS_WORKTREE names the caller's server checkout; no sibling is required in CI.
if [[ ${1:-} == --sls ]]; then
  : "${SLS_WORKTREE:?set SLS_WORKTREE to the irl-srt-server checkout}"
  checkout="$(realpath "$SLS_WORKTREE")"
  GIT_MASTER=1 git -C "$checkout" diff --quiet HEAD --
  sha="$(GIT_MASTER=1 git -C "$checkout" rev-parse HEAD)"
  build="$checkout/build"
  if [[ ! -f "$build/CMakeCache.txt" ]]; then
    : "${SLS_SRT_PREFIX:?fresh configure requires a gate-capable installed SLS_SRT_PREFIX}"
    prefix="$(realpath "$SLS_SRT_PREFIX")"
    [[ -f "$prefix/include/srt/srt.h" && -f "$prefix/lib/libsrt.so" ]]
    cmake -S "$checkout" -B "$build" -DCMAKE_BUILD_TYPE=Release \
      "-DCMAKE_CXX_FLAGS=-I$prefix/include" \
      "-DCMAKE_EXE_LINKER_FLAGS=-L$prefix/lib -Wl,-rpath,$prefix/lib"
  fi
  cmake --build "$build" --target srt_server --parallel "$JOBS"
  grep -qx 'SLS_HAVE_SRTO_PERIODICNAKGATE:INTERNAL=1' "$build/CMakeCache.txt" || {
    printf 'SLS build lacks the required periodic NAK gate\n' >&2; exit 1;
  }
  binary="$build/bin/srt_server"
  [[ -x "$binary" ]]
  dependencies='[]'
  while read -r library arrow path rest; do
    if [[ "$library" == libsrt.so* && "$arrow" == '=>' ]]; then
      [[ -f "$path" ]]
      dependencies="$(jq --arg p "$(realpath "$path")" --arg h "$(digest "$path")" \
        '. + [{path:$p,sha256:$h}]' <<< "$dependencies")"
    fi
  done < <(ldd "$binary")
  [[ "$(jq length <<< "$dependencies")" == 1 ]] || {
    printf 'SLS must resolve exactly one auditable shared libsrt\n' >&2; exit 1;
  }
  jq --arg p "$binary" --arg h "$(digest "$binary")" --arg s "$sha" \
    --arg conf "$(digest "$build/CMakeCache.txt")" --argjson deps "$dependencies" '
    .conformance_sinks.sls = {sink:"sls",metrics:"none",sls_bin:$p,sha256:$h,
      source:"https://github.com/CERALIVE/irl-srt-server.git",source_sha:$s,
      cmake_cache_sha256:$conf,dependencies:$deps,emulated:false,
      conf_template:"tests/bench_support/sls-conformance.conf.tmpl"}
  ' "$LOCK" > "$TMP"
  publish
  printf 'SLS_BIN=%s\nSLS sha256=%s (conformance only)\n' "$binary" "$(digest "$binary")"
  exit 0
fi

for name in ours-old ours-new irlserver-prod irlserver-next; do
  started=$SECONDS
  source=https://github.com/onsmith/srt.git
  checkout="$ART/src/$name"
  target=srt-sink-min
  case "$name" in
    ours-old) sha=$OLD; source=https://github.com/CERALIVE/srt.git; target=srt-live-transmit;;
    ours-new) checkout="$SRT_WORKTREE"; sha="$(GIT_MASTER=1 git -C "$checkout" rev-parse HEAD)";
      source=https://github.com/CERALIVE/srt.git; target=srt-live-transmit;
      GIT_MASTER=1 git -C "$checkout" diff --quiet HEAD --;;
    irlserver-prod) sha=$PROD;;
    irlserver-next) sha=$NEXT;;
  esac
  out="$ART/lineage/$name"
  binary="$out/$target"
  fingerprint="$(digest "$0"):$(digest "$SINK"):$sha"
  if current; then continue; fi
  if [[ "$name" != ours-new ]]; then checkout; fi
  build="$ART/lineage/build-$name"
  method="cmake Release shared OpenSSL; unmodified source; RUNPATH=\$ORIGIN/lib"
  cmake -S "$checkout" -B "$build" -DCMAKE_BUILD_TYPE=Release \
    -DENABLE_APPS=ON -DENABLE_SHARED=ON -DENABLE_STATIC=OFF \
    -DENABLE_UNITTESTS=OFF -DENABLE_TESTING=OFF -DENABLE_ENCRYPTION=ON \
    -DCMAKE_BUILD_WITH_INSTALL_RPATH=ON "-DCMAKE_INSTALL_RPATH=\$ORIGIN/lib"
  cmake --build "$build" --target srt-live-transmit --parallel "$JOBS"
  mkdir -p "$out/lib"
  cp -a "$build"/libsrt.so* "$out/lib/"
  if [[ "$target" == srt-live-transmit ]]; then
    install -m 0755 "$build/$target" "$binary"
  else
    printf '#include "srt.h"\nstatic_assert(SRTO_SRTLAPATCHES == 120);\nstatic_assert(SRTO_LOSSMAXTTL == 42);\n' |
      g++ -std=c++17 -x c++ -fsyntax-only -I "$checkout/srtcore" -I "$build" -
    g++ -std=c++17 -O2 -Wall -Wextra -Werror -I "$checkout/srtcore" -I "$build" \
      -I "$checkout/apps" -I "$checkout/common" "$SINK" "$checkout/apps/statswriter.cpp" \
      -L "$out/lib" "-Wl,-rpath,\$ORIGIN/lib" -lsrt -pthread -o "$binary"
  fi
  record "$out"/lib/libsrt.so*
done

source=https://github.com/BELABOX/srtla.git
sha=$BELABOX
checkout="$ART/src/belabox"
for name in belabox-srtla_rec belabox-srtla_send; do
  started=$SECONDS
  binary="$ART/lineage/$name"
  fingerprint="$(digest "$0"):$sha"
  if current; then continue; fi
  checkout
  GIT_MASTER=1 make -C "$checkout" -j "$JOBS" "${name#belabox-}"
  install -m 0755 "$checkout/${name#belabox-}" "$binary"
  method='make at pinned BELABOX revision'
  record
done

# Reuse the earlier builder's receipts only after checking source and binary hash.
for name in irlserver-srtla_send ceralive-srtla_rec irlserver-srtla_rec; do
  started=$SECONDS
  oldname=$name
  [[ "$name" != irlserver-srtla_send ]] || oldname=upstream-srtla_send
  receipt="$(jq -c --arg n "$oldname" '.[] | select(.name == $n and .build_status == "success")' \
    "$ART/externals/externals.json" 2>/dev/null || true)"
  if [[ "$name" == irlserver-srtla_send ]]; then
    sha=$UPSTREAM; source=https://github.com/irlserver/srtla_send.git
  else
    [[ -n "$receipt" ]] || { printf 'BLOCKER: run build_externals.sh for %s first\n' "$name" >&2; exit 1; }
    sha="$(jq -r '.sha' <<< "$receipt")"; source="$(jq -r '.source' <<< "$receipt")"
  fi
  binary="$ART/lineage/$name"
  fingerprint="$(digest "$0"):$sha"
  if current; then continue; fi
  if [[ -n "$receipt" ]] && jq -e --arg sha "$sha" --arg source "$source" \
      '.sha == $sha and .source == $source' <<< "$receipt" >/dev/null &&
      sha256sum --check --status <<< "$(jq -r '"\(.sha256)  \(.path)"' <<< "$receipt")"; then
    install -m 0755 "$(jq -r '.path' <<< "$receipt")" "$binary"
    method='reused hash-verified build_externals.sh artifact'
  elif [[ "$name" == irlserver-srtla_send ]]; then
    checkout="$ART/src/irlserver-srtla_send"
    checkout
    toolchain="$(sed -n 's/^channel = "\([^"]*\)"/\1/p' "$ROOT/rust-toolchain.toml")"
    [[ -n "$toolchain" ]]
    RUSTUP_TOOLCHAIN="$toolchain" CARGO_TARGET_DIR="$ART/lineage/build-upstream" \
      cargo build --release --locked --manifest-path "$checkout/Cargo.toml"
    install -m 0755 "$ART/lineage/build-upstream/release/srtla_send" "$binary"
    method="cargo --release --locked $toolchain"
  else
    printf 'BLOCKER: invalid external receiver receipt for %s\n' "$name" >&2; exit 1
  fi
  record
done

jq '
  def receiver($sink; $rec; $kind):
    {srt_live_transmit_bin:.artifacts[$sink].path, srt_live_transmit_sha256:.artifacts[$sink].sha256,
     srtla_rec_bin:.artifacts[$rec].path, srtla_rec_sha256:.artifacts[$rec].sha256,
     kind:$kind,emulated:false};
  .receivers["ours-old"] = (receiver("ours-old";"ceralive-srtla_rec";"ceralive") + {
    socket_options:{"42":"SRTO_LOSSMAXTTL","120":"SRTO_REORDERFREEZE"},
    listener_uri_template:"srt://:<port>?mode=listener&latency=<preset>&lossmaxttl=40&reorderfreeze=1",
    reorderfreeze_uri_supported:false,
    blocker:"Pinned production source lacks the reorderfreeze URI row; this unmodified binary does not apply it. Do not grade as freeze-on."}) |
  .receivers["ours-new"] = (receiver("ours-new";"ceralive-srtla_rec";"ceralive") + {
    socket_options:{"42":"SRTO_LOSSMAXTTL","120":"SRTO_REORDERFREEZE","121":"SRTO_PERIODICNAKGATE"},
    listener_uri_template:"srt://:<port>?mode=listener&latency=<preset>&reorderfreeze=1&periodicnakgate=1&lossmaxttl=<TTL>"}) |
  reduce ["irlserver-prod","irlserver-next"][] as $n (.;
    .receivers[$n] = (receiver($n;"irlserver-srtla_rec";"irlserver") + {
      sink_cli:"srt-sink-min",capture_semantics:"interval",stats_every_messages:1000,
      socket_options:{"42":"SRTO_LOSSMAXTTL","120":"SRTO_SRTLAPATCHES"},
      sink_args:["--port","<port>","--latency","<preset>","--sockopt","120=1","--sockopt","42=200","--statsout","<CSV>","--out","<FILE>"],
      harness_adapter_required:true})) |
  .receivers.belabox = (.receivers["irlserver-next"] + receiver("irlserver-next";"belabox-srtla_rec";"belabox") + {alias_of:"irlserver-next"}) |
  .candidates["reverse-interop"] = {
    belabox:{srtla_send_bin:.artifacts["belabox-srtla_send"].path,srtla_send_sha256:.artifacts["belabox-srtla_send"].sha256,emulated:false},
    irlserver:{srtla_send_bin:.artifacts["irlserver-srtla_send"].path,srtla_send_sha256:.artifacts["irlserver-srtla_send"].sha256,emulated:false}}
' "$LOCK" > "$TMP"
publish
printf 'Receiver artifacts locked: %s (build availability, not campaign readiness)\n' "$LOCK"
