#!/usr/bin/env bash
# Runs scripts/netns_test_gate.sh against stubs, with no sudo needed, and checks that
# it runs every tests/netns_*.rs target exactly once, one test at a time, and that
# after a timeout it removes only what the timed-out test process left behind.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STUB="$(mktemp -d)"
TSTUB="$(mktemp -d)"
OTHER_PID=""
# Stop both fake test processes even when an assertion fails, so a red run leaks nothing.
trap 'pkill -f -- "${TSTUB}/" 2>/dev/null; rm -rf "${STUB}" "${TSTUB}"; [[ -z "${OTHER_PID}" ]] || kill "${OTHER_PID}" 2>/dev/null || true' EXIT
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

# A timed-out target. The stub cargo starts a fake test process from the fake
# target directory and then hangs past the 1 s budget, as a hung test would.
mkdir -p "${TSTUB}/bin" "${TSTUB}/target/debug/deps" "${TSTUB}/other/debug/deps"
fake_test() {
  printf '#!/usr/bin/env bash\necho $$ > "%s"\nwhile :; do sleep 1; done\n' "$2" > "$1"
  chmod +x "$1"
}
fake_test "${TSTUB}/target/debug/deps/netns_basic-0123456789abcdef" "${TSTUB}/test.pid"
fake_test "${TSTUB}/other/debug/deps/netns_basic-0123456789abcdef" "${TSTUB}/other.pid"
printf '#!/usr/bin/env bash\n"%s/target/debug/deps/netns_basic-0123456789abcdef" &\nwhile :; do sleep 1; done\n' \
  "${TSTUB}" > "${TSTUB}/bin/cargo"
# shellcheck disable=SC2016  # the stub's body must expand when the stub runs
printf '#!/usr/bin/env bash\n[[ "$*" == "netns list" ]] || exit 1\nprintf "basic_s_%%x_0\\nother_s_ffff_0\\n" $(( $(cat "%s/test.pid") %% 65535 ))\n' \
  "${TSTUB}" > "${TSTUB}/bin/ip"
printf '#!/usr/bin/env bash\nprintf "%%s\\n" "$*" >> "%s/sudo"\n' "${TSTUB}" > "${TSTUB}/bin/sudo"
chmod +x "${TSTUB}/bin/cargo" "${TSTUB}/bin/ip" "${TSTUB}/bin/sudo"
# Another checkout's test process of the same target, which must survive.
"${TSTUB}/other/debug/deps/netns_basic-0123456789abcdef" &
OTHER_PID=$!

set +e
PATH="${TSTUB}/bin:${PATH}" CARGO_TARGET_DIR="${TSTUB}/target" NETNS_TEST_TIMEOUT_SECONDS=1 \
  bash "${REPO_ROOT}/scripts/netns_test_gate.sh" --only netns_basic >/dev/null 2>&1
STATUS=$?
set -e
fail() {
  echo "netns-test-gate-contract: after a timeout, $*" >&2
  exit 1
}
[[ "${STATUS}" -eq 124 ]] || fail "the gate exited ${STATUS}, expected 124"
TEST_PID="$(cat "${TSTUB}/test.pid")"
TAG="$(printf '_%x_' $((TEST_PID % 65535)))"
if kill -0 "${TEST_PID}" 2>/dev/null && [[ "$(ps -o stat= -p "${TEST_PID}")" != Z* ]]; then
  fail "the timed-out test process ${TEST_PID} is still running"
fi
grep -qx -- "-n ip netns delete basic_s${TAG}0" "${TSTUB}/sudo" \
  || fail "the gate did not remove basic_s${TAG}0 with sudo -n"
! grep -q -- "other_s_ffff_0" "${TSTUB}/sudo" || fail "the gate touched another process's namespace"
kill -0 "${OTHER_PID}" 2>/dev/null || fail "the gate killed another checkout's test process"

echo "netns-test-gate-contract: OK targets=$(grep -c . <<<"${TARGETS}") timeout-cleanup=scoped"
