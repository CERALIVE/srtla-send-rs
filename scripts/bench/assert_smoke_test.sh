#!/usr/bin/env bash
set -euo pipefail
summary=${1:-test-results/bench/smoke/summary.json}
manifest=${2:-scripts/bench/manifests/smoke.json}
results=${3:-$(dirname "$summary")}
checker=$(realpath "$(dirname "$0")/assert_smoke.sh")
bash "$checker" "$summary" "$manifest" "$results"
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
shopt -s nullglob
cp "$manifest" "$scratch/manifest.json"
cp "$summary" "$scratch/summary.json"
for directory in "$results"/*--*; do
  mkdir "$scratch/$(basename "$directory")"
  for run in "$directory"/run-*.json; do
    cp "$run" "$scratch/$(basename "$directory")/"
  done
done

# Given real passing evidence, when one obligation is falsified, then reject it.
record=
for path in "$results/classic--A--ceralive--production"/run-[01].json; do
  record="classic--A--ceralive--production/$(basename "$path")"
  break
done
[[ -n $record ]]
while IFS='|' read -r name expression; do
  jq "$expression" "$results/$record" > "$scratch/$record"
  if bash "$checker" "$scratch/summary.json" "$scratch/manifest.json" "$scratch" > "$scratch/output" 2>&1; then
    printf 'FAIL: accepted %s\n' "$name" >&2
    exit 1
  fi
  printf 'PASS: rejects %s\n' "$name"
done <<'CASES'
failed run|.status = "failed"
low goodput|.useful_goodput_bps = 1000000
missing SRT packets|.raw.stats_csv.rows |= map(.pkt_recv_total = 0)
wrong link count|.per_link = []
missing effective config|del(.sender.effective_config)
wrong effective config|.sender.effective_config.adaptive_tuning.stall_attempts = 16
wrong mode|.candidate.args = ["--mode", "enhanced"]
wrong binary|.candidate.bin_sha256 = ("0" * 64)
truncated A window|.window.end_ms = 44000
duplicate paired index|.run_index = (1 - .run_index)
CASES
cp "$results/$record" "$scratch/$record"
while IFS='|' read -r name expression; do
  jq "$expression" "$summary" > "$scratch/summary.json"
  if bash "$checker" "$scratch/summary.json" "$scratch/manifest.json" "$scratch" > "$scratch/output" 2>&1; then
    printf 'FAIL: accepted %s\n' "$name" >&2
    exit 1
  fi
  printf 'PASS: rejects %s\n' "$name"
done <<'CASES'
inflated summary count|.groups[0].cells.classic.n = 3
summary integrity error|.groups[0].cells.classic.integrity_errors = ["config mismatch"]
stale summary fingerprint|.groups[0].cells.classic.fingerprints = []
altered summary goodput|.groups[0].cells.classic.metrics.useful_goodput_bps.median = 1
CASES
