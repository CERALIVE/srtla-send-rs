#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKAGE_VERSION="$(node -p "require('${REPO_ROOT}/bindings/typescript/package.json').version")"
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

echo "bindings-release-ref-contract: OK tag=${RELEASE_TAG} sha=${HEAD_SHA}"
