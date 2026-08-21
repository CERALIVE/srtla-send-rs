#!/usr/bin/env bash
# Runs in ci.yml's Bun-only `bindings` job, so this script's own runtime calls
# are `bun`. The script UNDER TEST — ci/verify-bindings-release-ref.sh — is the
# sanctioned Node island (publish-bindings.yml's Node-26 OIDC publish job) and
# keeps its `node` invocation verbatim, so it is exercised here through a
# Bun-backed `node` shim.
#
# The shim is prepended UNCONDITIONALLY, and that is the whole point: a GitHub
# runner ships an ambient Node, so a "only when node is missing" guard would
# never fire on the machine that matters and this test would silently go on
# proving the island against a runtime its job does not declare. Prepending
# always means every run exercises the Bun path. The export is scoped to this
# process — the production publish job resolves its own real Node 26 and is
# unaffected.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SHIM_DIR="$(mktemp -d)"
trap 'rm -rf "${SHIM_DIR}"' EXIT
printf '#!/usr/bin/env bash\nexec bun "$@"\n' >"${SHIM_DIR}/node"
chmod +x "${SHIM_DIR}/node"
export PATH="${SHIM_DIR}:${PATH}"

# Behavioral proof, not a path match: process.versions.bun exists only under Bun.
SHIM_RUNTIME="$(node -e "console.log(process.versions.bun ?? 'node')")"
[[ "$(command -v node)" == "${SHIM_DIR}/node" && "${SHIM_RUNTIME}" != "node" ]] || {
  echo "bindings-release-ref-contract: node must resolve to the bun shim, got $(command -v node) (runtime ${SHIM_RUNTIME})" >&2
  exit 1
}

PACKAGE_VERSION="$(bun -e "console.log(require('${REPO_ROOT}/bindings/typescript/package.json').version)")"
RELEASE_TAG="bindings-v${PACKAGE_VERSION}"
HEAD_SHA="$(git -C "${REPO_ROOT}" rev-parse 'HEAD^{commit}')"

run_verifier() {
  GITHUB_EVENT_NAME="${1}" \
  GITHUB_REF_TYPE="${2}" \
  GITHUB_REF="${3}" \
  GITHUB_REF_NAME="${4}" \
  GITHUB_SHA="${5}" \
    bash "${REPO_ROOT}/ci/verify-bindings-release-ref.sh"
}

expect_rejection() {
  if run_verifier "$@" >/dev/null 2>&1; then
    echo "bindings-release-ref-contract: unauthorized provenance was accepted: $*" >&2
    exit 1
  fi
}

run_verifier push tag "refs/tags/${RELEASE_TAG}" "${RELEASE_TAG}" "${HEAD_SHA}"
expect_rejection workflow_dispatch branch refs/heads/main main "${HEAD_SHA}"
expect_rejection push branch refs/heads/main main "${HEAD_SHA}"
expect_rejection push tag refs/tags/bindings-v0.0.0 bindings-v0.0.0 "${HEAD_SHA}"
expect_rejection push tag "refs/tags/${RELEASE_TAG}" "${RELEASE_TAG}" "${HEAD_SHA}^"

echo "bindings-release-ref-contract: OK tag=${RELEASE_TAG} sha=${HEAD_SHA} node-runtime=bun@${SHIM_RUNTIME}"
