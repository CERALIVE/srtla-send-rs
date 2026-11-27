#!/usr/bin/env bash
# Runs scripts/netns_test_gate.sh against a stub cargo and checks that it runs
# every tests/netns_*.rs target exactly once, one test at a time, with no sudo needed.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STUB="$(mktemp -d)"
trap 'rm -rf "${STUB}"' EXIT
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
  if grep -v -- ' --test-threads=1' "${STUB}/calls" >&2; then
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

echo "netns-test-gate-contract: OK targets=$(grep -c . <<<"${TARGETS}")"
