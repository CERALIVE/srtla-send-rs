#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${REPO_ROOT}/Cargo.toml"
[[ -f "${CARGO_TOML}" ]] || {
  echo "deb-version-ordering: Cargo.toml is missing" >&2
  exit 1
}
command -v dpkg >/dev/null 2>&1 || {
  echo "deb-version-ordering: dpkg is required" >&2
  exit 1
}

CURRENT_VERSION="$(awk -F\" '/^version = /{print $2; exit}' "${CARGO_TOML}")"
[[ -n "${CURRENT_VERSION}" ]] || {
  echo "deb-version-ordering: Cargo.toml version is empty" >&2
  exit 1
}
[[ "${CURRENT_VERSION}" != *:* ]] || {
  echo "deb-version-ordering: epochs are forbidden (version=${CURRENT_VERSION})" >&2
  exit 1
}

if [[ "${CURRENT_VERSION}" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)([-+].*)?$ ]]; then
  PATCH_BUMP="${BASH_REMATCH[1]}.${BASH_REMATCH[2]}.$((BASH_REMATCH[3] + 1))"
else
  echo "deb-version-ordering: expected SemVer source version, got ${CURRENT_VERSION}" >&2
  exit 1
fi

dpkg --compare-versions "${PATCH_BUMP}" gt "${CURRENT_VERSION}" || {
  echo "deb-version-ordering: patch bump ${PATCH_BUMP} does not outrank ${CURRENT_VERSION}" >&2
  exit 1
}

STALE_CALVER="2026.6.1"
if dpkg --compare-versions "${STALE_CALVER}" gt "${CURRENT_VERSION}"; then
  STALE_VERDICT="stale CalVer ${STALE_CALVER} outranks current SemVer ${CURRENT_VERSION}; removal required"
else
  STALE_VERDICT="stale CalVer ${STALE_CALVER} does not outrank current SemVer ${CURRENT_VERSION}"
fi

printf 'deb-version-ordering: OK current=%s patch-bump=%s; %s\n' \
  "${CURRENT_VERSION}" "${PATCH_BUMP}" "${STALE_VERDICT}"
