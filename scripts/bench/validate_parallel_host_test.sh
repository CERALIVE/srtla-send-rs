#!/usr/bin/env bash
set -euo pipefail
here=$(dirname "$(realpath "${BASH_SOURCE[0]}")")
validator="$here/validate_parallel_host.sh"

# Given synthetic allocation fixtures, NOT benchmark observations.
valid=$(jq -n '{host: {
  parallel_lanes: 2, online_cpus: [range(8)], reserved_cpus: [0,1,2,3],
  smt_siblings: [[0,1],[2,3],[4,5],[6,7]], port_base: 20000,
  core_map: [
    {lane: 0, cpus: [4,5], port_range: [20000,20099],
     netns_prefix: "bpc0-", artifact_directory: "lane-0"},
    {lane: 1, cpus: [6,7], port_range: [20100,20199],
     netns_prefix: "bpc1-", artifact_directory: "lane-1"}
  ]}}')

# When valid one- and two-lane maps cross the CLI, then both pass.
bash "$validator" <(printf '%s' "$valid")
bash "$validator" <(printf '%s' "$valid" | jq '
  .host.parallel_lanes = 1 | .host.core_map = [.host.core_map[0]] |
  .host.core_map[0].cpus = [4,5,6,7]')

reject() {
  local name=$1 mutation=$2 expected=$3 output status
  # Given one broken invariant; when validated, then fail for that invariant.
  set +e
  output=$(bash "$validator" <(printf '%s' "$valid" | jq "$mutation") 2>&1)
  status=$?
  set -e
  if ((status == 0)) || [[ "$output" != *"$expected"* ]]; then
    printf 'FAIL %s (exit=%s): %s\n' "$name" "$status" "$output" >&2
    exit 1
  fi
  printf 'PASS %s: exit=%s %s\n' "$name" "$status" "$output"
}

reject cpu_overlap '.host.core_map[1].cpus = [4,5,6,7]' 'lanes 0 and 1: overlapping CPUs'
reject port_overlap '.host.core_map[1].port_range = [20099,20198]' 'lanes 0 and 1: overlapping ports'
reject split_siblings '.host.core_map[0].cpus = [4,6] | .host.core_map[1].cpus = [5,7]' 'split SMT siblings'
reject reserved_cpu '.host.core_map[0].cpus += [0]' 'reserved CPUs'
reject empty_cores '.host.core_map[0].cpus = []' 'CPU set'
reject duplicate_cpu '.host.core_map[0].cpus = [4,4,5]' 'CPU set'
reject lane_count '.host.parallel_lanes = 1' 'lane count'
reject empty_map '.host.parallel_lanes = 0 | .host.core_map = []' 'lane count'
reject fractional_cpu '.host.core_map[0].cpus = [4.5,5]' 'CPU set'
reject missing_cpu '.host.core_map[1].cpus = [6]' 'CPU coverage'
reject missing_host 'del(.host)' 'host object'
reject duplicate_prefix '.host.core_map[1].netns_prefix = "bpc0-"' 'lane 1: namespace prefix'
reject duplicate_artifacts '.host.core_map[1].artifact_directory = "lane-0"' 'lane 1: artifact directory'
reject invalid_port '.host.core_map[1].port_range = [65500,65599]' 'port range'
reject wrong_stride '.host.core_map[1].port_range = [20200,20299]' 'port stride'
reject absent_smt '.host.smt_siblings = []' 'SMT topology'
printf 'PASS allocation validator: two valid maps, sixteen independent rejection cases\n'
