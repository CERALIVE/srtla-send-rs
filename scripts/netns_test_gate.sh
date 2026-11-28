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

warn() {
  echo "netns-test-gate: $*" >&2
}

# Each run gets its own record file. Every process of the run inherits
# NETNS_GATE_RECORD naming it, and Namespace::new
# (crates/network-sim/src/topology.rs) appends each namespace's name to it
# before creating the namespace. Cleanup goes by those two alone, so it never
# touches another run's processes or namespaces.
RUN_DIR="$(mktemp -d)"
trap 'rm -rf "${RUN_DIR}"' EXIT

# `timeout --foreground` stops only cargo, so a failed run can leave its test
# process running, and the namespaces it made under sudo with whatever runs in
# them. A step that fails is reported, and the remaining steps still run.
clean_up_run() {
  local record="$1" listed out ns
  local -a pids recorded
  mapfile -t pids < <(grep -lsxzF -- "NETNS_GATE_RECORD=${record}" /proc/[0-9]*/environ | cut -d/ -f3)
  if [[ "${#pids[@]}" -gt 0 ]]; then
    warn "killing this run's leftover processes: ${pids[*]}"
    kill -KILL "${pids[@]}" || warn "some of them had exited already"
  fi
  [[ -s "${record}" ]] || return 0
  mapfile -t recorded < <(sort -u "${record}")
  if ! listed="$(ip netns list)"; then
    warn "could not list namespaces; check these by hand: ${recorded[*]}"
    return
  fi
  listed="$(awk '{print $1}' <<<"${listed}")"
  for ns in "${recorded[@]}"; do
    # Recorded but never created: the run died between the two steps.
    grep -qxF -- "${ns}" <<<"${listed}" || continue
    if out="$(sudo -n ip netns pids "${ns}")"; then
      mapfile -t pids < <(printf '%s' "${out}")
      if [[ "${#pids[@]}" -gt 0 ]] && ! sudo -n kill -KILL "${pids[@]}"; then
        warn "could not kill every process in ${ns}: ${pids[*]}"
      fi
    else
      warn "could not list the processes in ${ns}"
    fi
    if sudo -n ip netns delete "${ns}"; then
      warn "removed namespace ${ns} left by this run"
    else
      warn "could not remove namespace ${ns}; run: sudo ip netns delete ${ns}"
    fi
  done
}

for TARGET in "${TARGETS[@]}"; do
  BUDGET="$(budget_for "${TARGET}")"
  RECORD="${RUN_DIR}/${TARGET}"
  # No errexit until this target passes: after a failure, every cleanup step
  # must run and the gate must still exit with the target's own status.
  set +e
  # One test at a time, as upstream CI runs them: the harness names veth pairs
  # after the process id, so two tests of one target collide on `ip link add`.
  NETNS_GATE_RECORD="${RECORD}" timeout --foreground --kill-after=10s "${BUDGET}s" \
    cargo test --all-features --test "${TARGET}" -- --nocapture --test-threads=1
  STATUS=$?
  if [[ "${STATUS}" -ne 0 ]]; then
    case "${STATUS}" in
      124) warn "${TARGET} exceeded ${BUDGET}s" ;;
      # `--kill-after` ends in 137, but so does a cargo killed from outside.
      137) warn "${TARGET} was killed with SIGKILL (by --kill-after after ${BUDGET}s, or from outside)" ;;
    esac
    clean_up_run "${RECORD}"
    exit "${STATUS}"
  fi
  set -e
done

echo "netns-test-gate: OK timeout=${TIMEOUT_SECONDS}s twin-timeout=${TWIN_TIMEOUT_SECONDS}s targets=${#TARGETS[@]}"
