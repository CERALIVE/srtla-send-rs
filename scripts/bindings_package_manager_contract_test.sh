#!/usr/bin/env bash
# Pins the binding package-manager policy: Bun is the ONE package manager for
# bindings/typescript. A second lockfile is the failure this guards against —
# it resolves a different dependency graph than the one CI installs, silently.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINDINGS_ROOT="${REPO_ROOT}/bindings/typescript"
WORKFLOWS_ROOT="${REPO_ROOT}/.github/workflows"
FORBIDDEN_LOCKFILES=(pnpm-lock.yaml package-lock.json yarn.lock)

PACKAGE_MANAGER="$(node -p "require('${BINDINGS_ROOT}/package.json').packageManager")"

[[ "${PACKAGE_MANAGER}" == bun@* ]] || {
  echo "bindings-package-manager-contract: packageManager must pin bun, got ${PACKAGE_MANAGER}" >&2
  exit 1
}
[[ -f "${BINDINGS_ROOT}/bun.lock" ]] || {
  echo "bindings-package-manager-contract: bun.lock is missing" >&2
  exit 1
}
for lockfile in "${FORBIDDEN_LOCKFILES[@]}"; do
  [[ ! -e "${BINDINGS_ROOT}/${lockfile}" ]] || {
    echo "bindings-package-manager-contract: ${lockfile} is forbidden — bun.lock is the only lockfile" >&2
    exit 1
  }
done

if pnpm_usage="$(grep -rn '\bpnpm\b' "${WORKFLOWS_ROOT}")"; then
  echo "bindings-package-manager-contract: workflows must not invoke pnpm" >&2
  echo "${pnpm_usage}" >&2
  exit 1
fi

echo "bindings-package-manager-contract: OK manager=${PACKAGE_MANAGER}"
