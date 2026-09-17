#!/usr/bin/env bash
set -euo pipefail
lock=${1:-"$(dirname "${BASH_SOURCE[0]}")/receivers.lock.json"}
jq -e '
  def require($condition; $message):
    if $condition then . else error($message) end;
  def natural: type == "number" and . >= 0 and . == floor;
  def cpu_set:
    type == "array" and length > 0 and all(.[]; natural)
    and length == (unique | length);
  def disjoint($a; $b): ($a - $b | length) == ($a | length);

  .host | require(type == "object"; "missing host object") |
  require((.parallel_lanes | natural) and .parallel_lanes >= 1
    and (.core_map | type == "array")
    and (.core_map | length) == .parallel_lanes; "invalid lane count") |
  require((.online_cpus | cpu_set) and (.reserved_cpus | cpu_set)
    and (.reserved_cpus | length) == 4; "invalid online/reserved CPUs") |
  require((.smt_siblings | type == "array")
    and all(.smt_siblings[]; cpu_set)
    and ([.smt_siblings[][]] | sort) == (.online_cpus | sort);
    "incomplete or duplicate SMT topology") |
  require((.port_base | natural) and .port_base > 0; "invalid port base") |
  . as $h |
  reduce range(.parallel_lanes) as $i (.;
    .core_map[$i] as $lane |
    require($lane.lane == $i; "lane IDs must be contiguous from zero") |
    require($lane.cpus | cpu_set; "lane \($i): invalid CPU set") |
    require(disjoint($lane.cpus; $h.reserved_cpus); "lane \($i): uses reserved CPUs") |
    require($lane.netns_prefix == "bpc\($i)-"; "lane \($i): namespace prefix") |
    require($lane.artifact_directory == "lane-\($i)"; "lane \($i): artifact directory") |
    require(($lane.port_range | type == "array" and length == 2)
      and all($lane.port_range[]; natural and . > 0 and . <= 65535)
      and $lane.port_range[0] <= $lane.port_range[1]; "lane \($i): invalid port range")
  ) |
  reduce range(.parallel_lanes) as $i (.;
    reduce range($i + 1; .parallel_lanes) as $j (.;
      require(disjoint(.core_map[$i].cpus; .core_map[$j].cpus);
        "lanes \($i) and \($j): overlapping CPUs") |
      require(.core_map[$i].port_range[1] < .core_map[$j].port_range[0]
        or .core_map[$j].port_range[1] < .core_map[$i].port_range[0];
        "lanes \($i) and \($j): overlapping ports")
    )
  ) |
  require(([.core_map[].cpus[]] + .reserved_cpus | sort) == (.online_cpus | sort);
    "incomplete or invalid CPU coverage") |
  ([.core_map[].cpus] + [.reserved_cpus]) as $owners |
  require(all(.smt_siblings[]; . as $siblings |
    any($owners[]; ($siblings - . | length) == 0)); "split SMT siblings") |
  require(all(.core_map[];
    .port_range == [$h.port_base + .lane * 100, $h.port_base + .lane * 100 + 99]);
    "invalid port stride (expected base + lane * 100)") |
  true
' "$lock"
