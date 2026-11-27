#!/usr/bin/env bash
# Runs scripts/netns_test_gate.sh against a stub cargo and checks that it runs
# every tests/netns_*.rs target, one test at a time, with no sudo needed.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STUB="$(mktemp -d)"
trap 'rm -rf "${STUB}"' EXIT
printf '#!/usr/bin/env bash\nprintf "%%s\\n" "$*" >> "%s/calls"\n' "${STUB}" > "${STUB}/cargo"
chmod +x "${STUB}/cargo"

expected_call() {
  echo "test --all-features --test $1 -- --nocapture --test-threads=1"
}

PATH="${STUB}:${PATH}" bash "${REPO_ROOT}/scripts/netns_test_gate.sh" >/dev/null
EXPECTED=""
for FILE in "${REPO_ROOT}"/tests/netns_*.rs; do
  EXPECTED+="$(expected_call "$(basename "${FILE}" .rs)")"$'\n'
done
ACTUAL="$(cat "${STUB}/calls")"$'\n'
[[ "${ACTUAL}" == "${EXPECTED}" ]] || {
  echo "netns-test-gate-contract: the gate did not run every target serially" >&2
  diff <(echo "${EXPECTED}") <(echo "${ACTUAL}") >&2 || true
  exit 1
}

rm "${STUB}/calls"
PATH="${STUB}:${PATH}" bash "${REPO_ROOT}/scripts/netns_test_gate.sh" --only netns_arrival_reports >/dev/null
[[ "$(cat "${STUB}/calls")" == "$(expected_call netns_arrival_reports)" ]] || {
  echo "netns-test-gate-contract: --only netns_arrival_reports ran: $(cat "${STUB}/calls")" >&2
  exit 1
}

echo "netns-test-gate-contract: OK targets=$(grep -c . <<<"${EXPECTED}")"
