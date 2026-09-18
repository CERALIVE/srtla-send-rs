#!/usr/bin/env bash
set -euo pipefail

root=${1:?absolute durable artifact directory}
export PATH=/home/andres/.cargo/bin:/usr/local/bin:/usr/bin:/bin
export GIT_MASTER=1 BENCH_MAX_RETRIES=1 BENCH_PCAP_ON_FAIL=0
unset SLS_BONDED_PROFILE_OVERRIDE BENCH_SOAK
export SLS_BIN
SLS_BIN=$(jq -er '.conformance_sinks.sls.sls_bin' scripts/bench/receivers.lock.json)
test -x "$SLS_BIN"
printf '%s  %s\n' "$(jq -er '.conformance_sinks.sls.sha256' scripts/bench/receivers.lock.json)" "$SLS_BIN" | sha256sum --check
export BENCH_MANIFEST=scripts/bench/manifests/m4b-lineages.json
export BENCH_OUT_DIR="$root/results" BENCH_ARTIFACT_DIR="$root/artifacts"
mkdir -p "$BENCH_OUT_DIR" "$BENCH_ARTIFACT_DIR"
test ! -e "$root/start.txt"
date --iso-8601=seconds >"$root/start.txt"
date +%s >"$root/start-epoch.txt"
trap 'code=$?; printf "%s\n" "$code" >"$root/controller-exit.txt"; date --iso-8601=seconds >"$root/end.txt"; date +%s >"$root/end-epoch.txt"' EXIT
sha256sum --check "$root/frozen.sha256"
uv run scripts/bench/m4b_manifest.py --verdict "$root/base-verdict.json" --validate "$BENCH_MANIFEST"
printf '%s\n' 'taskset -c 4-27 timeout --foreground --kill-after=30s 21600s target/m4b-bench_scheduler --ignored --exact --nocapture campaign' >"$root/command.txt"
set +e
taskset -c 4-27 timeout --foreground --kill-after=30s 21600s \
    target/m4b-bench_scheduler --ignored --exact --nocapture campaign >"$root/campaign.log" 2>&1
code=$?
set -e
printf '%s\n' "$code" >"$root/campaign-exit.txt"
date --iso-8601=seconds >"$root/campaign-end.txt"
[[ $code == 0 || $code == 101 ]]
sha256sum --check "$root/frozen.sha256"
for output in "$root/verdict-final.json" "$root/verdict-repeat.json"; do
    uv run scripts/bench/m4b_gate.py --verdict "$root/base-verdict.json" \
        --manifest "$BENCH_MANIFEST" --results "$BENCH_OUT_DIR" --out "$output"
done
cmp "$root/verdict-final.json" "$root/verdict-repeat.json"
sha256sum "$root/verdict-final.json"
