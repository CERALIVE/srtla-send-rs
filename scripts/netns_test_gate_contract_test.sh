#!/usr/bin/env bash
# Runs scripts/netns_test_gate.sh against stubs, with no sudo needed, and checks that
# it runs every tests/netns_*.rs target exactly once, one test at a time, and that
# after a failed run it removes only what that run left behind.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STUB="$(mktemp -d)"
TSTUB="$(mktemp -d)"
# Every fake process below inherits TSTUB, so this stops exactly those, even
# when an assertion fails.
stop_fakes() {
  local -a pids
  export -n TSTUB
  mapfile -t pids < <(grep -lsxzF -- "TSTUB=${TSTUB}" /proc/[0-9]*/environ | cut -d/ -f3)
  [[ "${#pids[@]}" -eq 0 ]] || kill -KILL "${pids[@]}" 2>/dev/null || true
  rm -rf "${STUB}" "${TSTUB}"
}
trap stop_fakes EXIT
printf '#!/usr/bin/env bash\nprintf "%%s\\n" "$*" >> "%s/calls"\n' "${STUB}" > "${STUB}/cargo"
cat > "${STUB}/systemd-run" <<'EOF'
#!/usr/bin/env bash
while [[ "$1" != -- ]]; do shift; done
shift
exec "$@"
EOF
chmod +x "${STUB}/cargo" "${STUB}/systemd-run"

# Fails unless the recorded calls ran exactly the targets on stdin, each once, serially.
check_calls() {
  local expected actual
  expected="$(sort)"
  actual="$(sed -nE 's/.* --test ([^ ]+).*/\1/p' "${STUB}/calls" | sort)"
  [[ "${actual}" == "${expected}" ]] || {
    echo "netns-test-gate-contract: ran these targets instead of each expected one once:" >&2
    diff <(echo "${expected}") <(echo "${actual}") >&2 || true
    exit 1
  }
  if grep -v -E -- '(^| )--test-threads=1( |$)' "${STUB}/calls" >&2; then
    echo "netns-test-gate-contract: the calls above do not run one test at a time" >&2
    exit 1
  fi
  rm "${STUB}/calls"
}

PATH="${STUB}:${PATH}" bash "${REPO_ROOT}/scripts/netns_test_gate.sh" >/dev/null
TARGETS="$(for FILE in "${REPO_ROOT}"/tests/netns_*.rs; do basename "${FILE}" .rs; done)"
check_calls <<<"${TARGETS}"

PATH="${STUB}:${PATH}" bash "${REPO_ROOT}/scripts/netns_test_gate.sh" --only netns_arrival_reports >/dev/null
check_calls <<<"netns_arrival_reports"

# A failed run, with no clock involved. The stub `timeout` does what
# `timeout --foreground` does when the budget runs out: once the fake test has
# made its namespaces, it kills only cargo and exits with FAKE_STATUS. Stub
# `systemd-run` and `systemctl` give the run a fake scope, and stub `ip` and
# `sudo` keep the namespace list in a file and log every privileged call.
export TSTUB
mkdir -p "${TSTUB}/bin" "${TSTUB}/cgroup/run"
cat > "${TSTUB}/bin/systemd-run" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "${TSTUB}/systemd-run.calls"
unit=""
while [[ "$1" != -- ]]; do
  [[ "$1" != --unit=* ]] || unit="${1#--unit=}"
  shift
done
shift
[[ -n "${unit}" ]] || exec "$@"
# A fake scope that acts as the kernel's does: every member holds fd 8 on
# `members`, and a write to cgroup.kill SIGKILLs each pid in cgroup.procs, then
# reports the scope empty once all of them have exited.
scope="${TSTUB}/cgroup/run/${unit}.scope"
mkdir "${scope}"
mkfifo "${scope}/members" "${scope}/cgroup.kill"
echo 'populated 1' > "${scope}/cgroup.events"
: > "${scope}/cgroup.procs"
(
  shopt -s nullglob
  exec 9>&- 7<"${scope}/members"
  read -r value < "${scope}/cgroup.kill"
  [[ "${value:-}" == 1 ]] || exit 0
  mapfile -t pids < "${scope}/cgroup.procs"
  kill -KILL "${pids[@]}"
  cat <&7 >/dev/null
  for f in "${TSTUB}"/pids.*; do
    grep -vxF -f "${scope}/cgroup.procs" "${f}" > "${f}.next"
    mv "${f}.next" "${f}"
  done
  echo 'populated 0' > "${scope}/cgroup.events"
) &
exec 8>"${scope}/members"
export FAKE_SCOPE="${scope}"
exec "$@"
EOF
cat > "${TSTUB}/bin/systemctl" <<'EOF'
#!/usr/bin/env bash
[[ "$1 $2 $3 $4" == "--user show --property=ControlGroup --value" ]] || exit 1
if [[ -n "${FAKE_CGROUP}" ]]; then
  echo "${FAKE_CGROUP}"
elif [[ -d "${TSTUB}/cgroup/run/$5" ]]; then
  echo "/run/$5"
fi
EOF
cat > "${TSTUB}/bin/timeout" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "${TSTUB}/timeout.calls"
exec 3<>"${TSTUB}/ready"
while [[ "$1" == -* ]]; do shift; done
shift
"$@" 3<&- &
read -r -t 30 _ <&3 || exit 99
kill -KILL "$!"
exit "${FAKE_STATUS}"
EOF
cat > "${TSTUB}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
"${TSTUB}/netns_basic-0123456789abcdef" &
wait
EOF
# What a hung test leaves: three namespaces it made, a name whose creation it
# never saw finish, a name whose add failed because another run holds exactly
# that name, a process of its own inside basic_s_1_0, and itself. basic_t_1_4
# also holds a process that is not part of this run.
cat > "${TSTUB}/netns_basic-0123456789abcdef" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' 'pending basic_r_1_1' 'created basic_r_1_1' 'pending basic_s_1_0' \
  'created basic_s_1_0' 'pending basic_t_1_4' 'created basic_t_1_4' \
  'pending basic_g_1_2' 'pending basic_s_1_3' >> "${NETNS_GATE_RECORD}"
printf '%s\n' basic_r_1_1 basic_s_1_0 basic_t_1_4 >> "${TSTUB}/netns"
sleep infinity &
printf '%s\n' "$$" "$!" >> "${FAKE_SCOPE}/cgroup.procs"
echo "$!" > "${TSTUB}/pids.basic_s_1_0"
echo "${OTHER_PID}" > "${TSTUB}/pids.basic_t_1_4"
echo ready > "${TSTUB}/ready"
exec sleep infinity
EOF
cat > "${TSTUB}/bin/ip" <<'EOF'
#!/usr/bin/env bash
[[ "$*" == "netns list" ]] || exit 1
# As on a host whose /run/netns is root-only: listed only through sudo.
[[ -z "${FAKE_ROOT:-}" ]] || sed 's/$/ (id: 0)/' "${TSTUB}/netns"
EOF
cat > "${TSTUB}/bin/sudo" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "${TSTUB}/sudo.calls"
[[ "$1" == -n ]] || exit 1
shift
case "$1 $2 $3" in
  "ip netns list") FAKE_ROOT=1 exec ip netns list ;;
  "ip netns pids")
    [[ "$4" != "${FAKE_PIDS_FAIL}" ]] || exit 1
    cat "${TSTUB}/pids.$4" 2>/dev/null || true
    ;;
  "ip netns delete")
    grep -vxF -- "$4" "${TSTUB}/netns" > "${TSTUB}/netns.next" || true
    mv "${TSTUB}/netns.next" "${TSTUB}/netns"
    ;;
  *) exit 1 ;;
esac
EOF
chmod +x "${TSTUB}/bin/"* "${TSTUB}/netns_basic-0123456789abcdef"

fail() {
  echo "netns-test-gate-contract: after a failed run, $*" >&2
  [[ ! -s "${TSTUB}/gate.err" ]] || sed 's/^/  gate: /' "${TSTUB}/gate.err" >&2
  exit 1
}

# Another run of the same target, started from the same path, which must
# survive: it is not in this run's scope, though it runs inside basic_t_1_4.
NETNS_GATE_RECORD="${TSTUB}/other-run" \
  bash -c 'exec -a "$0" sleep infinity' "${TSTUB}/netns_basic-0123456789abcdef" &
export OTHER_PID=$!

# $1: the status the stub timeout reports (124 for a timeout, 137 for SIGKILL);
# $2: a cgroup for systemctl to report instead of the scope's own. Leaves the
# gate's status in STATUS and the pid of the fd-9 reader in READER.
run_failed_gate() {
  # Another run's namespace, with exactly the name this run's add failed on.
  printf 'basic_s_1_3\n' > "${TSTUB}/netns"
  rm -rf "${TSTUB}/"{sudo.calls,timeout.calls,systemd-run.calls,ready,alive} \
    "${TSTUB}/pids."* "${TSTUB}/cgroup/run/"*
  mkfifo "${TSTUB}/ready" "${TSTUB}/alive"
  # Reaches EOF once every process holding the gate's fd 9, so every process
  # of the run, has exited.
  timeout 30 cat "${TSTUB}/alive" >/dev/null &
  READER=$!
  set +e
  # The process lookup in basic_r_1_1 fails: cleanup must leave that namespace
  # in place and go on to the next.
  PATH="${TSTUB}/bin:${PATH}" FAKE_STATUS="$1" FAKE_CGROUP="${2:-}" FAKE_PIDS_FAIL=basic_r_1_1 \
    NETNS_GATE_CGROUP_ROOT="${TSTUB}/cgroup" \
    bash "${REPO_ROOT}/scripts/netns_test_gate.sh" --only netns_basic \
    9>"${TSTUB}/alive" >/dev/null 2>"${TSTUB}/gate.err"
  STATUS=$?
  set -e
}

check_failed_run() {
  run_failed_gate "$1"
  [[ "${STATUS}" -eq "$1" ]] || fail "the gate exited ${STATUS}, expected $1"
  wait "${READER}" || fail "a process of the failed run is still running"
  [[ "$(tr '\n' ' ' < "${TSTUB}/netns")" == "basic_s_1_3 basic_r_1_1 basic_t_1_4 " ]] \
    || fail "the namespaces left are: $(tr '\n' ' ' < "${TSTUB}/netns")"
  ! grep -E -- 'basic_s_1_3|basic_g_1_2' "${TSTUB}/sudo.calls" >&2 \
    || fail "the gate ran the calls above on a namespace this run did not make"
  ! grep -E -- '^-n kill' "${TSTUB}/sudo.calls" >&2 \
    || fail "the gate signalled the processes above by id"
  grep -qE -- '^--user --scope --quiet --collect --unit=netns-gate-netns_basic-[0-9a-f-]{36} -- timeout ' \
    "${TSTUB}/systemd-run.calls" || fail "the gate did not run the target in its own systemd user scope"
  grep -qx -- "--foreground --kill-after=10s 90s cargo test --all-features --test netns_basic -- --nocapture --test-threads=1" \
    "${TSTUB}/timeout.calls" || fail "the gate did not run cargo under timeout --foreground with its budget"
}
check_failed_run 124
check_failed_run 137

# A scope that reports any cgroup but its own is never killed: here systemctl
# reports the root, whose cgroup.kill would stop every process on the host. The
# run's fakes are left running here, and stop_fakes ends them.
: > "${TSTUB}/cgroup/cgroup.kill"
run_failed_gate 124 /
kill "${READER}" 2>/dev/null || true
[[ "${STATUS}" -eq 124 ]] || fail "the gate exited ${STATUS}, expected 124"
[[ ! -s "${TSTUB}/cgroup/cgroup.kill" ]] || fail "the gate wrote to the root cgroup's cgroup.kill"

kill -TERM "${OTHER_PID}"
OTHER_STATUS=0
wait "${OTHER_PID}" || OTHER_STATUS=$?
[[ "${OTHER_STATUS}" -eq 143 ]] \
  || fail "another run's process of the same target did not survive (status ${OTHER_STATUS})"

echo "netns-test-gate-contract: OK targets=$(grep -c . <<<"${TARGETS}") timeout-cleanup=scoped"
