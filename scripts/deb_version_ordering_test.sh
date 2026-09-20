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

# Only the `srtla` stream's own ordering is checked. The retired srtla-send-rs
# package is displaced by Conflicts/Replaces, not outranked by version, so a
# cross-name version comparison here would assert nothing.
PACKAGE_NAME="srtla"

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

printf 'deb-version-ordering: OK package=%s current=%s patch-bump=%s\n' \
  "${PACKAGE_NAME}" "${CURRENT_VERSION}" "${PATCH_BUMP}"
