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

# The contract test points these at fakes.
CGROUP_ROOT="${NETNS_GATE_CGROUP_ROOT:-/sys/fs/cgroup}"
LOCK_FILE="${NETNS_GATE_LOCK:-/tmp/netns-test-gate.lock}"

# Namespaces are host-wide and the harness's names repeat across runs, so only
# one gate runs at a time on a host. Whatever the run starts inherits the lock.
exec 5>"${LOCK_FILE}"
flock -n 5 || {
  echo "netns-test-gate: another netns gate, or a process one left, holds ${LOCK_FILE}" >&2
  exit 2
}

# Each target runs in its own systemd user scope, so after a failure one write
# to that scope's cgroup.kill stops every process of the run, root or not and
# in whatever namespace, with no process id read first and signalled later.
# Before any target runs, a scope checks that it could do exactly that.
# shellcheck disable=SC2016  # it must expand inside the probe scope's own shell
systemd-run --user --scope --quiet --collect -- \
  bash -c '[[ -w "$1$(sed -n "s/^0:://p" /proc/self/cgroup)/cgroup.kill" ]]' _ "${CGROUP_ROOT}" \
  >/dev/null 2>&1 || {
  echo "netns-test-gate: needs a systemd user session on cgroup v2 whose scopes it may kill (systemd-run --user --scope, cgroup.kill)" >&2
  exit 2
}

# Each run also gets its own record file, named by NETNS_GATE_RECORD:
# Namespace::new (crates/network-sim/src/topology.rs) writes `pending <name>`
# to it before `ip netns add` and `created <name>` once that add has succeeded.
RUN_DIR="$(mktemp -d)"
trap 'rm -rf "${RUN_DIR}"' EXIT

# cgroup.kill only sends the signals; wait, within a bound, until they landed.
wait_until_empty() {
  local _
  for _ in $(seq 1 100); do
    # A scope with nothing left in it is collected, files and all.
    [[ -e "$1/cgroup.events" ]] || return 0
    grep -qx 'populated 0' "$1/cgroup.events" && return 0
    sleep 0.1
  done
  return 1
}

# `timeout --foreground` stops only cargo, so a failed run can leave its test
# process running, and the namespaces it made under sudo with whatever runs in
# them. A step that fails is reported, and the remaining steps still run.
clean_up_run() {
  local record="$1" scope="$2.scope" cg listed out entry ns quiet=no
  local -a created unconfirmed
  if ! cg="$(systemctl --user show --property=ControlGroup --value "${scope}")"; then
    warn "could not look up ${scope}; check it with: systemctl --user status ${scope}"
  elif [[ -n "${cg}" && "${cg}" != */"${scope}" ]]; then
    # Writing any other cgroup's kill file, the root's above all, would stop
    # processes that are not this run's.
    warn "${scope} reports cgroup '${cg}'; not killing it"
  elif [[ -z "${cg}" ]]; then
    # A scope with nothing left in it is collected.
    quiet=yes
  elif ! echo 1 > "${CGROUP_ROOT}${cg}/cgroup.kill"; then
    warn "could not kill ${scope}; run: systemctl --user kill --signal=KILL ${scope}"
  elif wait_until_empty "${CGROUP_ROOT}${cg}"; then
    warn "killed every process left in ${scope}"
    quiet=yes
  else
    warn "${scope} still has processes: $(tr '\n' ' ' < "${CGROUP_ROOT}${cg}/cgroup.procs")"
  fi
  [[ -s "${record}" ]] || return 0
  # `created <name> <inode>`, one per namespace this run made.
  mapfile -t created < <(awk '$1 == "created" {print $2 " " $3}' "${record}" | sort -u)
  # Pending but never created: the add failed (another run may hold that name)
  # or was cut short, so whose namespace it is cannot be told.
  mapfile -t unconfirmed < <(awk '$1 == "pending" {print $2}' "${record}" | sort -u \
    | comm -23 - <(printf '%s\n' "${created[@]%% *}"))
  for ns in "${unconfirmed[@]}"; do
    warn "this run never saw ${ns} created; if it is this run's, run: sudo ip netns delete ${ns}"
  done
  [[ "${#created[@]}" -gt 0 ]] || return 0
  if [[ "${quiet}" != yes ]]; then
    warn "the run's processes may still be running, so no namespace is removed; check: ${created[*]%% *}"
    return
  fi
  # /run/netns can be root-only, and then a plain `ip netns list` prints nothing.
  if ! listed="$(sudo -n ip netns list)"; then
    warn "could not list namespaces; check these by hand: ${created[*]}"
    return
  fi
  listed="$(awk '{print $1}' <<<"${listed}")"
  for entry in "${created[@]}"; do
    ns="${entry%% *}"
    # Already deleted by the run itself.
    grep -qxF -- "${ns}" <<<"${listed}" || continue
    # A namespace that took the name after this run's was deleted is not this run's.
    if [[ "$(sudo -n stat -c %i "/run/netns/${ns}")" != "${entry#* }" ]]; then
      warn "${ns} is no longer the namespace this run created; left it"
      continue
    fi
    # Deleting a name with processes still inside would hide them, so a
    # namespace is removed only once nothing runs in it.
    if ! out="$(sudo -n ip netns pids "${ns}")"; then
      warn "could not list the processes in ${ns}; left it, check: sudo ip netns pids ${ns}"
      continue
    fi
    if [[ -n "${out}" ]]; then
      warn "${ns} still runs processes: $(tr '\n' ' ' <<<"${out}"); left it, check: sudo ip netns pids ${ns}"
      continue
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
  SCOPE="netns-gate-${TARGET}-$(cat /proc/sys/kernel/random/uuid)"
  # No errexit until this target passes: after a failure, every cleanup step
  # must run and the gate must still exit with the target's own status.
  set +e
  # One test at a time, as upstream CI runs them: the harness names veth pairs
  # after the process id, so two tests of one target collide on `ip link add`.
  NETNS_GATE_RECORD="${RECORD}" systemd-run --user --scope --quiet --collect --unit="${SCOPE}" -- \
    timeout --foreground --kill-after=10s "${BUDGET}s" \
    cargo test --all-features --test "${TARGET}" -- --nocapture --test-threads=1
  STATUS=$?
  if [[ "${STATUS}" -ne 0 ]]; then
    case "${STATUS}" in
      124) warn "${TARGET} exceeded ${BUDGET}s" ;;
      # `--kill-after` ends in 137, but so does a cargo killed from outside.
      137) warn "${TARGET} was killed with SIGKILL (by --kill-after after ${BUDGET}s, or from outside)" ;;
    esac
    clean_up_run "${RECORD}" "${SCOPE}"
    exit "${STATUS}"
  fi
  set -e
done

echo "netns-test-gate: OK timeout=${TIMEOUT_SECONDS}s twin-timeout=${TWIN_TIMEOUT_SECONDS}s targets=${#TARGETS[@]}"
