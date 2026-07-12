#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BINDINGS_ROOT="${REPO_ROOT}/bindings/typescript"
PACKAGE_MANAGER="$(node -p "require('${BINDINGS_ROOT}/package.json').packageManager")"

[[ "${PACKAGE_MANAGER}" == pnpm@* ]] || {
  echo "bindings-package-manager-contract: packageManager must pin pnpm" >&2
  exit 1
}
[[ -f "${BINDINGS_ROOT}/pnpm-lock.yaml" ]] || {
  echo "bindings-package-manager-contract: pnpm-lock.yaml is missing" >&2
  exit 1
}
[[ ! -e "${BINDINGS_ROOT}/bun.lock" ]] || {
  echo "bindings-package-manager-contract: bun.lock is forbidden by root policy" >&2
  exit 1
}

echo "bindings-package-manager-contract: OK manager=${PACKAGE_MANAGER}"
