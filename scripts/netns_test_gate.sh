#!/usr/bin/env bash
set -euo pipefail

TIMEOUT_SECONDS="${NETNS_TEST_TIMEOUT_SECONDS:-90}"
[[ "${TIMEOUT_SECONDS}" =~ ^[1-9][0-9]*$ ]] || {
  echo "netns-test-gate: NETNS_TEST_TIMEOUT_SECONDS must be a positive integer" >&2
  exit 2
}

TARGETS=(
  netns_basic
  netns_edpf
  netns_failure
  netns_impairment
  netns_pr19_parity
  netns_scenario
)

for TARGET in "${TARGETS[@]}"; do
  set +e
  timeout --foreground --kill-after=10s "${TIMEOUT_SECONDS}s" \
    cargo test --all-features --test "${TARGET}" -- --nocapture
  STATUS=$?
  set -e
  if [[ "${STATUS}" -eq 124 ]]; then
    echo "netns-test-gate: ${TARGET} exceeded ${TIMEOUT_SECONDS}s" >&2
    exit 124
  fi
  [[ "${STATUS}" -eq 0 ]] || exit "${STATUS}"
done

echo "netns-test-gate: OK timeout=${TIMEOUT_SECONDS}s targets=${#TARGETS[@]}"
