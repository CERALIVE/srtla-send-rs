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
chmod +x "${STUB}/cargo"

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
# made its namespaces, it kills only cargo and exits with FAKE_STATUS. Stub `ip`
# and `sudo` keep the namespace list in a file and log every privileged call.
export TSTUB
mkdir "${TSTUB}/bin"
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
# What a hung test leaves: two namespaces it made, a third name it recorded but
# never made (the run died between the two steps), a process inside one of them
# that lacks the run's variable, as everything started under sudo does, and itself.
cat > "${TSTUB}/netns_basic-0123456789abcdef" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' basic_r_1_1 basic_s_1_0 basic_g_1_2 >> "${NETNS_GATE_RECORD}"
printf '%s\n' basic_r_1_1 basic_s_1_0 >> "${TSTUB}/netns"
env -u NETNS_GATE_RECORD sleep infinity &
echo "$!" > "${TSTUB}/pids.basic_s_1_0"
echo ready > "${TSTUB}/ready"
exec sleep infinity
EOF
cat > "${TSTUB}/bin/ip" <<'EOF'
#!/usr/bin/env bash
[[ "$*" == "netns list" ]] || exit 1
sed 's/$/ (id: 0)/' "${TSTUB}/netns"
EOF
cat > "${TSTUB}/bin/sudo" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "${TSTUB}/sudo.calls"
[[ "$1" == -n ]] || exit 1
shift
case "$1 $2 $3" in
  "ip netns pids")
    [[ "$4" != "${FAKE_PIDS_FAIL}" ]] || exit 1
    cat "${TSTUB}/pids.$4" 2>/dev/null || true
    ;;
  "ip netns delete")
    grep -vxF -- "$4" "${TSTUB}/netns" > "${TSTUB}/netns.next" || true
    mv "${TSTUB}/netns.next" "${TSTUB}/netns"
    ;;
  kill\ -KILL\ *) "$@" ;;
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
# survive: only its NETNS_GATE_RECORD differs.
NETNS_GATE_RECORD="${TSTUB}/other-run" \
  bash -c 'exec -a "$0" sleep infinity' "${TSTUB}/netns_basic-0123456789abcdef" &
OTHER_PID=$!

# $1: the status the stub timeout reports (124 for a timeout, 137 for SIGKILL).
check_failed_run() {
  local reader status resident
  # Another run's namespace, named like this run's.
  printf 'basic_s_1_3\n' > "${TSTUB}/netns"
  rm -f "${TSTUB}/"{sudo.calls,timeout.calls,ready,alive} "${TSTUB}/pids."*
  mkfifo "${TSTUB}/ready" "${TSTUB}/alive"
  # Reaches EOF once every process holding the gate's fd 9, so every process
  # of the run, has exited.
  timeout 30 cat "${TSTUB}/alive" >/dev/null &
  reader=$!
  set +e
  # The process lookup in basic_r_1_1 fails; cleanup must go on past it.
  PATH="${TSTUB}/bin:${PATH}" FAKE_STATUS="$1" FAKE_PIDS_FAIL=basic_r_1_1 \
    bash "${REPO_ROOT}/scripts/netns_test_gate.sh" --only netns_basic \
    9>"${TSTUB}/alive" >/dev/null 2>"${TSTUB}/gate.err"
  status=$?
  set -e
  [[ "${status}" -eq "$1" ]] || fail "the gate exited ${status}, expected $1"
  wait "${reader}" || fail "a process of the failed run is still running"
  [[ "$(cat "${TSTUB}/netns")" == basic_s_1_3 ]] \
    || fail "the namespaces left are: $(tr '\n' ' ' < "${TSTUB}/netns")"
  resident="$(cat "${TSTUB}/pids.basic_s_1_0")"
  grep -qx -- "-n kill -KILL ${resident}" "${TSTUB}/sudo.calls" \
    || fail "the gate did not kill ${resident}, the process inside basic_s_1_0, with sudo -n"
  ! grep -E -- 'basic_s_1_3|basic_g_1_2' "${TSTUB}/sudo.calls" >&2 \
    || fail "the gate ran the calls above on a namespace this run did not make"
  grep -qx -- "--foreground --kill-after=10s 90s cargo test --all-features --test netns_basic -- --nocapture --test-threads=1" \
    "${TSTUB}/timeout.calls" || fail "the gate did not run cargo under timeout --foreground with its budget"
}
check_failed_run 124
check_failed_run 137

kill -TERM "${OTHER_PID}"
OTHER_STATUS=0
wait "${OTHER_PID}" || OTHER_STATUS=$?
[[ "${OTHER_STATUS}" -eq 143 ]] \
  || fail "another run's process of the same target did not survive (status ${OTHER_STATUS})"

echo "netns-test-gate-contract: OK targets=$(grep -c . <<<"${TARGETS}") timeout-cleanup=scoped"
