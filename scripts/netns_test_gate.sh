#!/usr/bin/env bash
set -euo pipefail

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

TARGETS=(
  netns_basic
  netns_edpf
  netns_failure
  netns_impairment
  netns_pr19_parity
  netns_scenario
  netns_twin
  netns_unconnected
)

budget_for() {
  case "$1" in
    netns_twin) echo "${TWIN_TIMEOUT_SECONDS}" ;;
    *) echo "${TIMEOUT_SECONDS}" ;;
  esac
}

for TARGET in "${TARGETS[@]}"; do
  BUDGET="$(budget_for "${TARGET}")"
  set +e
  timeout --foreground --kill-after=10s "${BUDGET}s" \
    cargo test --all-features --test "${TARGET}" -- --nocapture
  STATUS=$?
  set -e
  if [[ "${STATUS}" -eq 124 ]]; then
    echo "netns-test-gate: ${TARGET} exceeded ${BUDGET}s" >&2
    exit 124
  fi
  [[ "${STATUS}" -eq 0 ]] || exit "${STATUS}"
done

echo "netns-test-gate: OK timeout=${TIMEOUT_SECONDS}s twin-timeout=${TWIN_TIMEOUT_SECONDS}s targets=${#TARGETS[@]}"
