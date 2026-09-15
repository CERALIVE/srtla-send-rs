#!/usr/bin/env bash
set -euo pipefail
if [[ $# -lt 2 || $# -gt 3 ]]; then
  printf 'usage: %s manifest.json results-directory [output-directory]\n' "$0" >&2
  exit 2
fi
manifest=$(realpath "$1")
results=$(realpath "$2")
output=${3:-$results}
here=$(dirname "$(realpath "$0")")
scope=$(mktemp -d)
trap 'rm -rf -- "$scope"' EXIT

# Validate the full original tree before deriving the explicitly scoped report view.
bash "$here/assert_smoke.sh" --records-only /dev/null "$manifest" "$results"
jq '.cells |= map(select(.candidate != "adaptive" and .scenario == "A"))' "$manifest" > "$scope/manifest.json"
mkdir "$scope/results"
shopt -s nullglob
while IFS= read -r id; do
  mkdir "$scope/results/$id"
  for record in "$results/$id"/run-*.json; do
    ln -s "$(realpath "$record")" "$scope/results/$id/$(basename "$record")"
  done
done < <(jq -r '.cells[] | [.candidate,.scenario,.receiver,.srt_profile] | join("--")' "$scope/manifest.json")
mkdir -p "$output"
uv run "$here/report.py" --smoke-coverage --results "$scope/results" --manifest "$scope/manifest.json" \
  --out "$output/report.md" --json "$output/summary.json"
bash "$here/assert_smoke.sh" "$output/summary.json" "$manifest" "$results"
jq -s '{
  scope:"Todo29 smoke-path acceptance only",
  cells:["classic--D--ceralive--production","enhanced--D--ceralive--production","adaptive--A--ceralive--production","adaptive--D--ceralive--production"],
  blocking:false,
  documentation:"docs/notes/scheduler-evaluation-2026-09.md#known-limitation-adaptive-mode-baseline-topology-throughput-instability-scenario-a-discovered-post-todo-28",
  outcomes:map({run_id,run_index,attempt,status,reason,candidate,receiver})
}' "$results/classic--D--ceralive--production"/run-*.json "$results/enhanced--D--ceralive--production"/run-*.json "$results/adaptive--A--ceralive--production"/run-*.json "$results/adaptive--D--ceralive--production"/run-*.json > "$scope/findings.json"
mv "$scope/findings.json" "$output/known-findings.json"
printf '\n## Final owner-approved smoke scope\n\nAcceptance requires at least one successful run out of the two planned indices in classic/A and enhanced/A. Counts and indices are actual successes, not planned or renumbered observations. Failed attempts remain in the unchanged original result tree and report warnings. Classic/D, enhanced/D, adaptive/A and adaptive/D were executed and are informational in known-findings.json. The original full-matrix campaign result remains a failure; this is a retrospective, explicitly scoped path-coverage acceptance, not a statistical majority, a C1/C2 waiver, or a performance pass. See docs/notes/scheduler-evaluation-2026-09.md for the measured variance and owner decision.\n' >> "$output/report.md"
