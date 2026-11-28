#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

TIMEOUT_SECONDS="${NETNS_TEST_TIMEOUT_SECONDS:-90}"
# The twin scenarios wait out real sender timers the other targets never touch —
# the 15s ACK liveness timeout and the 30s status-log interval — so they cannot
# share the default budget.
TWIN_TIMEOUT_SECONDS="${NETNS_TWIN_TEST_TIMEOUT_SECONDS:-420}"
for BUDGET in "${TIMEOUT_SECONDS}" "${TWIN_TIMEOUT_SECONDS}"; do
  [[ "${BUDGET}" =~ ^[1-9][0-9]*$ ]] || {
    echo "netns-test-gate: timeout budgets must be positive integers" >&2
    exit 2
  }
done

# Every tests/netns_*.rs target, so a new one cannot be left out of the gate.
ALL_TARGETS=()
for FILE in tests/netns_*.rs; do
  ALL_TARGETS+=("$(basename "${FILE}" .rs)")
done

ONLY=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --only)
      ONLY="${2:-}"
      [[ -n "${ONLY}" ]] || {
        echo "netns-test-gate: --only requires a target name" >&2
        exit 2
      }
      shift 2
      ;;
    *)
      echo "netns-test-gate: unknown argument '$1'" >&2
      exit 2
      ;;
  esac
done

TARGETS=()
if [[ -n "${ONLY}" ]]; then
  for TARGET in "${ALL_TARGETS[@]}"; do
    [[ "${TARGET}" == "${ONLY}" ]] && TARGETS=("${TARGET}")
  done
  [[ "${#TARGETS[@]}" -gt 0 ]] || {
    echo "netns-test-gate: '${ONLY}' is not a registered target" >&2
    exit 2
  }
else
  TARGETS=("${ALL_TARGETS[@]}")
fi

budget_for() {
  case "$1" in
    netns_twin) echo "${TWIN_TIMEOUT_SECONDS}" ;;
    *) echo "${TIMEOUT_SECONDS}" ;;
  esac
}

# `timeout --foreground` stops only cargo, so a timed-out target leaves its test
# process and everything it started under sudo running. Remove exactly what that
# process created: it runs from this checkout's target directory, and the harness
# names every namespace after its pid (`_<pid % 0xffff in hex>_<n>`,
# crates/network-sim/src/test_util.rs), so no other checkout's run is touched.
clean_up_after_timeout() {
  local target_dir="${CARGO_TARGET_DIR:-${PWD}/target}" pid tag ns
  for pid in $(pgrep -u "$(id -u)" -f -- "${target_dir}/debug/deps/$1-[0-9a-f]+( |$)" || true); do
    kill -KILL "${pid}" 2>/dev/null || true
    tag="$(printf '_%x_' $((pid % 65535)))"
    for ns in $(ip netns list | awk '{print $1}' | grep -E -- "${tag}[0-9]+$" || true); do
      sudo -n ip netns pids "${ns}" | xargs -r sudo -n kill -KILL
      if sudo -n ip netns delete "${ns}"; then
        echo "netns-test-gate: removed namespace ${ns} left by $1 (pid ${pid})" >&2
      else
        echo "netns-test-gate: could not remove namespace ${ns}; run: sudo ip netns delete ${ns}" >&2
      fi
    done
  done
}

for TARGET in "${TARGETS[@]}"; do
  BUDGET="$(budget_for "${TARGET}")"
  set +e
  # One test at a time, as upstream CI runs them: the harness names veth pairs
  # after the process id, so two tests of one target collide on `ip link add`.
  timeout --foreground --kill-after=10s "${BUDGET}s" \
    cargo test --all-features --test "${TARGET}" -- --nocapture --test-threads=1
  STATUS=$?
  set -e
  # 137: the target ignored SIGTERM and `--kill-after` had to send SIGKILL.
  if [[ "${STATUS}" -eq 124 || "${STATUS}" -eq 137 ]]; then
    echo "netns-test-gate: ${TARGET} exceeded ${BUDGET}s" >&2
    clean_up_after_timeout "${TARGET}"
    exit 124
  fi
  [[ "${STATUS}" -eq 0 ]] || exit "${STATUS}"
done

echo "netns-test-gate: OK timeout=${TIMEOUT_SECONDS}s twin-timeout=${TWIN_TIMEOUT_SECONDS}s targets=${#TARGETS[@]}"
