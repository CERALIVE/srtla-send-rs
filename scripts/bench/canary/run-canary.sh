#!/usr/bin/env bash
# Owner-run only: compares this worktree's enhanced build to the locked 3.3.0 binary.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
readonly REPO_ROOT
readonly LOCK_FILE="${REPO_ROOT}/scripts/bench/receivers.lock.json"
readonly SESSION_SECONDS=1800
readonly RECEIVER_PORT=9000

dry_run=false
output_dir="${CANARY_OUTPUT_DIR:-${REPO_ROOT}/docs/evidence/bpc/canary/runs/$(date -u +%Y%m%dT%H%M%SZ)}"
survivor_bin="${CANARY_SURVIVOR_BIN:-${REPO_ROOT}/target/release/srtla_send}"
receiver_host="${CANARY_RECEIVER_HOST:-}"
bind_ips_file="${CANARY_BIND_IPS_FILE:-}"
stream_command="${CANARY_STREAM_COMMAND:-}"
stats_url="${CANARY_STATS_URL:-}"
listen_port="${CANARY_LISTEN_PORT:-10000}"
expected_links="${CANARY_EXPECTED_LINKS:-}"
goodput_jq="${CANARY_GOODPUT_JQ:-(.mbpsRecvRate // .mbps_recv_rate // .goodput_mbps // .goodputMbps) * 1000000}"
drop_jq="${CANARY_PKT_RCV_DROP_JQ:-(.pktRcvDrop // .pkt_rcv_drop // .pktRcvDropTotal // .pkt_rcv_drop_total)}"

usage() {
  printf '%s\n' \
    'Usage: run-canary.sh [--dry-run] [--output DIR] [--survivor-bin PATH]' \
    'Runs two sequential, 30-minute arms: survivor and the locked ours-3.3.0.' \
    '' \
    'The real-rig invocation requires CANARY_RECEIVER_HOST, CANARY_BIND_IPS_FILE,' \
    'CANARY_STREAM_COMMAND, CANARY_STATS_URL, and CANARY_EXPECTED_LINKS. See README.md.'
}

while (($#)); do
  case "$1" in
    --dry-run) dry_run=true ;;
    --output)
      output_dir="${2:?--output requires a directory}"
      shift
      ;;
    --survivor-bin)
      survivor_bin="${2:?--survivor-bin requires a path}"
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      printf 'error: unknown argument: %s\n' "$1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

locked_old_bin() {
  jq -er '.candidates["m3-interop"]["ours-3.3.0"].srtla_send_bin' "$LOCK_FILE"
}

locked_old_sha() {
  jq -er '.candidates["m3-interop"]["ours-3.3.0"].srtla_send_sha256' "$LOCK_FILE"
}

validate_static_inputs() {
  [[ -f "$LOCK_FILE" ]] || { printf 'error: missing lock: %s\n' "$LOCK_FILE" >&2; return 1; }
  jq -e '.candidates["m3-interop"]["ours-3.3.0"] | has("srtla_send_bin") and has("srtla_send_sha256")' "$LOCK_FILE" >/dev/null
  if [[ ! "$listen_port" =~ ^[0-9]+$ ]] || ((listen_port <= 0 || listen_port >= 65536)); then
    printf 'error: CANARY_LISTEN_PORT must be a UDP port\n' >&2
    return 1
  fi
}

validate_real_rig_inputs() {
  local required
  for required in receiver_host bind_ips_file stream_command stats_url expected_links; do
    [[ -n "${!required}" ]] || { printf 'error: CANARY_%s is required for a real-rig run\n' "${required^^}" >&2; return 1; }
  done
  [[ -r "$bind_ips_file" ]] || { printf 'error: BIND_IPS_FILE is unreadable: %s\n' "$bind_ips_file" >&2; return 1; }
  [[ "$expected_links" =~ ^[1-9][0-9]*$ ]] || { printf 'error: CANARY_EXPECTED_LINKS must be positive\n' >&2; return 1; }
  [[ -x "$survivor_bin" ]] || { printf 'error: survivor binary is not executable: %s\n' "$survivor_bin" >&2; return 1; }
  [[ -x "$(locked_old_bin)" ]] || { printf 'error: locked ours-3.3.0 binary is not executable\n' >&2; return 1; }
  [[ "$(sha256sum "$(locked_old_bin)" | cut -d' ' -f1)" == "$(locked_old_sha)" ]] || {
    printf 'error: locked ours-3.3.0 binary hash drifted\n' >&2
    return 1
  }
}

copy_telemetry() {
  local source="$1"
  local destination="$2"
  [[ -s "$source" ]] || return 0
  cp "$source" "$destination/$(date -u +%s%3N).json"
}

collect_arm() {
  local arm="$1"
  local binary="$2"
  local arm_dir="${output_dir}/${arm}"
  local telemetry_file="${arm_dir}/sender-stats.json"
  local sender_pid collector_pid producer_pid producer_status
  mkdir -p "${arm_dir}/telemetry" "${arm_dir}/receiver-stats" "${arm_dir}/resources"

  printf '%s\n' "$binary" > "${arm_dir}/binary-path.txt"
  sha256sum "$binary" > "${arm_dir}/binary.sha256"
  printf '%s\n' "$stream_command" > "${arm_dir}/stream-command.txt"

  "$binary" "$listen_port" "$receiver_host" "$RECEIVER_PORT" "$bind_ips_file" \
    --mode enhanced --stats-file "$telemetry_file" --stats-file-interval 1000 \
    > "${arm_dir}/sender.log" 2>&1 &
  sender_pid=$!

  (
    while kill -0 "$sender_pid" 2>/dev/null; do
      curl --fail --silent --show-error --max-time 5 "$stats_url" > "${arm_dir}/receiver-stats/$(date -u +%s%3N).json" || true
      copy_telemetry "$telemetry_file" "${arm_dir}/telemetry"
      ps -o pid=,pcpu=,rss= -p "$sender_pid" >> "${arm_dir}/resources/sender.csv" || true
      sleep 1
    done
  ) &
  collector_pid=$!

  CANARY_LISTEN_PORT="$listen_port" CANARY_SESSION_SECONDS="$SESSION_SECONDS" \
    timeout --foreground --signal=TERM --kill-after=30s "${SESSION_SECONDS}s" bash -c "$stream_command" \
    > "${arm_dir}/stream.log" 2>&1 &
  producer_pid=$!
  wait "$producer_pid" || producer_status=$?
  producer_status="${producer_status:-0}"

  copy_telemetry "$telemetry_file" "${arm_dir}/telemetry"
  kill -TERM "$sender_pid" 2>/dev/null || true
  wait "$sender_pid" || true
  wait "$collector_pid" || true

  if [[ "$producer_status" -ne 124 ]]; then
    printf 'error: %s producer exited %s before the required 30-minute arm completed\n' "$arm" "$producer_status" >&2
    return 1
  fi
  summarize_arm "$arm"
}

numeric_samples() {
  local filter="$1"
  local stats_dir="$2"
  local sample value
  local -a samples=()
  for sample in "${stats_dir}"/*.json; do
    [[ -f "$sample" ]] || continue
    value="$(jq -er "$filter" "$sample" 2>/dev/null)" || continue
    jq -en --argjson value "$value" '$value | numbers and select(isfinite)' >/dev/null || continue
    samples+=("$value")
  done
  ((${#samples[@]} > 1)) || return 1
  printf '%s\n' "${samples[@]}"
}

summarize_arm() {
  local arm="$1"
  local arm_dir="${output_dir}/${arm}"
  local goodputs drops goodput_bps first_drop last_drop drop_delta telemetry unrecovered
  goodputs="$(numeric_samples "$goodput_jq" "${arm_dir}/receiver-stats")" || {
    printf 'error: %s has fewer than two numeric goodput samples from /stats\n' "$arm" >&2
    return 1
  }
  drops="$(numeric_samples "$drop_jq" "${arm_dir}/receiver-stats")" || {
    printf 'error: %s has fewer than two numeric pktRcvDrop samples from /stats\n' "$arm" >&2
    return 1
  }
  goodput_bps="$(printf '%s\n' "$goodputs" | jq -s 'add / length')"
  first_drop="$(printf '%s\n' "$drops" | jq -s 'first')"
  last_drop="$(printf '%s\n' "$drops" | jq -s 'last')"
  drop_delta="$(jq -en --argjson first "$first_drop" --argjson last "$last_drop" '$last - $first')"
  telemetry="$(printf '%s\n' "${arm_dir}/telemetry"/*.json | sort | tail -n 1)"
  [[ -f "$telemetry" ]] || { printf 'error: %s emitted no sender telemetry\n' "$arm" >&2; return 1; }
  unrecovered="$(jq -er --argjson expected "$expected_links" '
    if (.connections | length) != $expected then ($expected - (.connections | length) | if . > 0 then . else 0 end)
    elif all(.connections[]; has("health")) then [.connections[] | select(.health != "healthy")] | length
    else 0
    end
  ' "$telemetry")"
  jq -n \
    --arg arm "$arm" \
    --arg telemetry "$telemetry" \
    --argjson goodput_bps "$goodput_bps" \
    --argjson pktRcvDrop_delta "$drop_delta" \
    --argjson unrecovered_links "$unrecovered" \
    --argjson cpu_rss_gated false \
    '{arm:$arm, goodput_bps:$goodput_bps, pktRcvDrop_delta:$pktRcvDrop_delta,
      unrecovered_links:$unrecovered_links, telemetry_final:$telemetry,
      cpu_rss_gated:$cpu_rss_gated}' > "${arm_dir}/summary.json"
}

decide() {
  local survivor_summary="${output_dir}/survivor/summary.json"
  local baseline_summary="${output_dir}/ours-3.3.0/summary.json"
  jq -n --slurpfile survivor "$survivor_summary" --slurpfile baseline "$baseline_summary" '
    ($survivor[0]) as $s | ($baseline[0]) as $b |
    ($s.unrecovered_links == 0 and $b.unrecovered_links == 0) as $links |
    ($s.pktRcvDrop_delta == 0 and $b.pktRcvDrop_delta == 0) as $stream_drops |
    ($s.goodput_bps >= (0.95 * $b.goodput_bps)) as $goodput |
    ($s.pktRcvDrop_delta <= $b.pktRcvDrop_delta) as $drop_not_worse |
    {arms:[$s, $b], criteria:{zero_unrecovered_links:$links, zero_stream_drops:$stream_drops,
      survivor_goodput_at_least_95_percent_of_3_3_0:$goodput, pktRcvDrop_not_worse:$drop_not_worse,
      cpu_rss_recorded_not_gated:true},
      outcome:(if ($links and $stream_drops and $goodput and $drop_not_worse) then "pass" else "fail" end)}
  ' > "${output_dir}/canary-result.json"
  [[ "$(jq -r '.outcome' "${output_dir}/canary-result.json")" == "pass" ]]
}

validate_static_inputs
if "$dry_run"; then
  printf 'dry-run: validated two 1800-second arms (survivor, ours-3.3.0); no hardware, sockets, or network calls were attempted\n'
  printf 'dry-run: ours-3.3.0 lock = %s (%s)\n' "$(locked_old_bin)" "$(locked_old_sha)"
  exit 0
fi

validate_real_rig_inputs
mkdir -p "$output_dir"
collect_arm survivor "$survivor_bin"
collect_arm ours-3.3.0 "$(locked_old_bin)"
decide
