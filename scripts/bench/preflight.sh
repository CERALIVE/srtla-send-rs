#!/usr/bin/env bash
set -euo pipefail

# Preflight checks for the adaptive scheduler benchmark harness.
# Exits 0 if all checks pass, non-zero on first blocking failure.
# Prints one line per check item.

REPO_ROOT="$(GIT_MASTER=1 git rev-parse --show-toplevel)"

# Item (a): Privileged command checks via sudo -n
check_privileged_commands() {
  local commands=(
    "ip netns list"
    "tc qdisc show"
    "tcpdump --version"
    "nsenter --version"
    "modprobe -n sch_netem"
    "kill -0 $$"
  )
  
  for cmd in "${commands[@]}"; do
    local command_parts=()
    read -r -a command_parts <<< "$cmd"
    if ! sudo -n "${command_parts[@]}" > /dev/null 2>&1; then
      echo "BLOCKING (a): unattended sudo required for ip/tc/tcpdump/nsenter/modprobe/kill: grant NOPASSWD for exactly those commands, run the campaign in a root session, or pre-authenticate with a long timestamp_timeout"
      return 1
    fi
  done
  
  echo "OK (a): privileged commands available via sudo -n"
  return 0
}

# Item (b): Required tools on PATH
check_required_tools() {
  local required=(
    "ip"
    "tc"
    "tcpdump"
    "ss"
    "python3"
    "uv"
    "git"
    "cmake"
    "g++"
    "make"
    "jq"
    "sha256sum"
    "srt-live-transmit"
  )
  
  local optional=(
    "iperf3"
  )
  
  local missing_required=()
  local missing_optional=()
  
  for tool in "${required[@]}"; do
    if ! command -v "${tool}" > /dev/null 2>&1; then
      missing_required+=("${tool}")
    fi
  done
  
  for tool in "${optional[@]}"; do
    if ! command -v "${tool}" > /dev/null 2>&1; then
      missing_optional+=("${tool}")
    fi
  done
  
  if [[ ${#missing_required[@]} -gt 0 ]]; then
    echo "BLOCKING (b): missing required tools: ${missing_required[*]}"
    return 1
  fi
  
  if [[ ${#missing_optional[@]} -gt 0 ]]; then
    echo "WARN (b): missing optional tools: ${missing_optional[*]}"
  else
    echo "OK (b): all required and optional tools found"
  fi
  
  return 0
}

# Item (c): Kernel modules loadable via modprobe -n
check_kernel_modules() {
  local modules=(
    "sch_netem"
    "sch_tbf"
    "cls_u32"
  )
  
  local unloadable=()
  
  for mod in "${modules[@]}"; do
    if ! modprobe -n "${mod}" > /dev/null 2>&1; then
      unloadable+=("${mod}")
    fi
  done
  
  if [[ ${#unloadable[@]} -gt 0 ]]; then
    echo "WARN (c): kernel modules may not be loadable: ${unloadable[*]} (best-effort check)"
  else
    echo "OK (c): kernel modules appear loadable"
  fi
  
  return 0
}

# Item (d): BENCH_ARTIFACT_DIR exists or is creatable, check filesystem and free space
check_artifact_dir() {
  local bench_dir="${BENCH_ARTIFACT_DIR:-/home/andres/.cache/opencode/tmp/srtla-bench}"
  
  # Ensure directory exists or can be created
  if [[ ! -d "${bench_dir}" ]]; then
    if ! mkdir -p "${bench_dir}" 2>/dev/null; then
      echo "BLOCKING (d): cannot create BENCH_ARTIFACT_DIR: ${bench_dir}"
      return 1
    fi
  fi
  
  # Check filesystem type and free space
  local fs_type
  fs_type=$(df "${bench_dir}" | tail -1 | awk '{print $1}')
  
  local free_space_kb
  free_space_kb=$(df "${bench_dir}" | tail -1 | awk '{print $4}')
  
  local free_space_gb=$((free_space_kb / 1024 / 1024))
  
  # BLOCKING if free space < 6 GB
  if [[ ${free_space_gb} -lt 6 ]]; then
    echo "BLOCKING (d): insufficient free space in BENCH_ARTIFACT_DIR (${bench_dir}): ${free_space_gb} GB < 6 GB required"
    return 1
  fi
  
  # WARN if tmpfs and free space < 12 GB
  if [[ "${fs_type}" == *"tmpfs"* ]] && [[ ${free_space_gb} -lt 12 ]]; then
    echo "WARN (d): tmpfs BENCH_ARTIFACT_DIR (${bench_dir}) has only ${free_space_gb} GB free (< 12 GB recommended)"
  else
    echo "OK (d): BENCH_ARTIFACT_DIR (${bench_dir}) has ${free_space_gb} GB free"
  fi
  
  return 0
}

# Item (e): Repo filesystem free space
check_repo_filesystem() {
  local repo_free_gb
  repo_free_gb=$(df "${REPO_ROOT}" | tail -1 | awk '{print $4 / 1024 / 1024}' | xargs printf "%.0f")
  
  if [[ ${repo_free_gb} -lt 5 ]]; then
    echo "BLOCKING (e): insufficient free space on repo filesystem (${REPO_ROOT}): ${repo_free_gb} GB < 5 GB required"
    return 1
  fi
  
  echo "OK (e): repo filesystem (${REPO_ROOT}) has ${repo_free_gb} GB free"
  return 0
}

# Item (f): SRTLA_REPO env var and receiver_main.cpp
check_srtla_repo() {
  if [[ -z "${SRTLA_REPO:-}" ]]; then
    echo "BLOCKING (f): SRTLA_REPO environment variable not set"
    return 1
  fi
  
  if [[ ! -f "${SRTLA_REPO}/src/receiver_main.cpp" ]]; then
    echo "BLOCKING (f): SRTLA_REPO is set to '${SRTLA_REPO}' but '${SRTLA_REPO}/src/receiver_main.cpp' does not exist"
    return 1
  fi
  
  echo "OK (f): SRTLA_REPO (${SRTLA_REPO}) contains receiver_main.cpp"
  return 0
}

check_receiver_lock() {
  local lock="${BENCH_RECEIVERS_LOCK:-$REPO_ROOT/scripts/bench/receivers.lock.json}"
  local checks
  if ! jq -e '
    .schema_version == 1 and (.artifacts | type == "object" and length >= 7) and
    (["ours-old","ours-new","irlserver-prod","irlserver-next","belabox-srtla_rec",
      "belabox-srtla_send","irlserver-srtla_send"] - (.artifacts | keys) | length == 0) and
    all(.artifacts[] | ., .dependencies[]?;
      .emulated == false and (.source_sha | test("^[0-9a-f]{40}$")) and
      (.path | type == "string") and (.sha256 | test("^[0-9a-f]{64}$")))
    ' "$lock" >/dev/null; then
    printf 'BLOCKING (g): missing or invalid receiver lock: %s\n' "$lock" >&2
    return 1
  fi
  # Include historical candidate/receiver hashes, not just the new artifact table.
  checks="$(jq -er '[.. | objects | . as $o |
    (if has("path") and has("sha256") then {path,sha256} else empty end),
    (keys[] | select(endswith("_sha256")) | . as $k |
      {path:$o[($k | rtrimstr("_sha256")) + "_bin"],sha256:$o[$k]})] |
    unique | .[] | if (.path | type != "string") or
      (.path | test("[\\n\\r\\\\]")) or (.sha256 | test("^[0-9a-f]{64}$") | not)
      then error("invalid hash/path pair") else "\(.sha256)  \(.path)" end' "$lock")" || return 1
  if ! sha256sum --check <<< "$checks"; then
    printf 'BLOCKING (g): locked artifact drift; rebuild or recover the named artifact\n' >&2
    return 1
  fi
  jq -r '.receivers | to_entries[] | select(.value.blocker or .value.harness_adapter_required) |
    "WARN (g): \(.key): \(.value.blocker // "srt-sink-min requires a harness CLI/output adapter")"' "$lock"
  printf 'OK (g): every locked artifact SHA-256 verified (not campaign readiness)\n'
}

# Main execution
main() {
  # Run all checks, stopping on first blocking failure
  if ! check_privileged_commands; then
    return 1
  fi
  
  if ! check_required_tools; then
    return 1
  fi

  if ! check_receiver_lock; then
    return 1
  fi
  
  if ! check_kernel_modules; then
    # (c) is best-effort, continue even on warn
    :
  fi
  
  if ! check_artifact_dir; then
    return 1
  fi
  
  if ! check_repo_filesystem; then
    return 1
  fi
  
  if ! check_srtla_repo; then
    return 1
  fi
  
  echo "preflight: all checks passed"
  return 0
}

main "$@"
