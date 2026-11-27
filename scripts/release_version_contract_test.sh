#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST_VERSION="$(awk -F\" '/^version = /{print $2; exit}' "${REPO_ROOT}/Cargo.toml")"
[[ -n "${MANIFEST_VERSION}" ]] || { echo "release-version-contract: Cargo.toml version is empty" >&2; exit 1; }
EXPECTED_TAG="v${MANIFEST_VERSION}"

LOCK_VERSION="$(awk '
  /^\[\[package\]\]$/ { in_package = 1; package_name = ""; next }
  in_package && /^name = "srtla_send"$/ { package_name = "srtla_send"; next }
  package_name == "srtla_send" && /^version = / {
    gsub(/^version = "|"$/, "")
    print
    exit
  }
' "${REPO_ROOT}/Cargo.lock")"
[[ "${LOCK_VERSION}" == "${MANIFEST_VERSION}" ]] || {
  echo "release-version-contract: Cargo.lock has ${LOCK_VERSION}, expected ${MANIFEST_VERSION}" >&2
  exit 1
}

TMPDIR="$(mktemp -d)"
trap 'rm -rf "${TMPDIR}"' EXIT
install -D -m 0755 /bin/true "${TMPDIR}/srtla_send"

ARTIFACT_NAMES=()
for ARCH in arm64 amd64; do
  BUILD_OUTPUT="$(GITHUB_REF_TYPE=tag GITHUB_REF_NAME="${EXPECTED_TAG}" \
    "${REPO_ROOT}/ci/build-deb.sh" "${ARCH}" "${TMPDIR}/srtla_send" "${TMPDIR}/out" 2>&1)"
  ARTIFACT="${TMPDIR}/out/srtla_${MANIFEST_VERSION}_${ARCH}.deb"
  ARTIFACT_NAMES+=("$(basename "${ARTIFACT}")")

  grep -Fqx "build-deb: output=${ARTIFACT}" <<<"${BUILD_OUTPUT}" || {
    echo "release-version-contract: build did not select ${ARTIFACT}" >&2
    exit 1
  }

  grep -Fq "| Version: ${MANIFEST_VERSION}" <<<"${BUILD_OUTPUT}" || {
    echo "release-version-contract: control metadata is not Version: ${MANIFEST_VERSION}" >&2
    exit 1
  }

  if command -v dpkg-deb >/dev/null; then
    [[ -f "${ARTIFACT}" ]] || {
      echo "release-version-contract: missing ${ARTIFACT}" >&2
      exit 1
    }
    DEB_VERSION="$(dpkg-deb -f "${ARTIFACT}" Version)"
    [[ "${DEB_VERSION}" == "${MANIFEST_VERSION}" ]] || {
      echo "release-version-contract: deb has ${DEB_VERSION}, expected ${MANIFEST_VERSION}" >&2
      exit 1
    }
    DEB_PACKAGE="$(dpkg-deb -f "${ARTIFACT}" Package)"
    [[ "${DEB_PACKAGE}" == "srtla" ]] || {
      echo "release-version-contract: deb package is ${DEB_PACKAGE}, expected srtla" >&2
      exit 1
    }
  fi
done

# The caller's umask must not reach the package (hosts often run with 0077).
STRICT_DEB="${TMPDIR}/strict/srtla_${MANIFEST_VERSION}_amd64.deb"
STRICT_OUTPUT="$( (umask 0077; "${REPO_ROOT}/ci/build-deb.sh" amd64 "${TMPDIR}/srtla_send" "${TMPDIR}/strict") 2>&1)" || {
  echo "release-version-contract: build failed under umask 0077:" >&2
  echo "${STRICT_OUTPUT}" >&2
  exit 1
}
if command -v dpkg-deb >/dev/null; then
  STRICT_MODES="$(dpkg-deb --contents "${STRICT_DEB}" | awk '{print $1, $NF}')"
  [[ "${STRICT_MODES}" == $'drwxr-xr-x ./\ndrwxr-xr-x ./usr/\ndrwxr-xr-x ./usr/bin/\n-rwxr-xr-x ./usr/bin/srtla_send' ]] || {
    echo "release-version-contract: modes under umask 0077 differ:" >&2
    echo "${STRICT_MODES}" >&2
    exit 1
  }
fi

if GITHUB_REF_TYPE=tag GITHUB_REF_NAME="${EXPECTED_TAG}-mismatch" \
  "${REPO_ROOT}/ci/build-deb.sh" amd64 "${TMPDIR}/srtla_send" "${TMPDIR}/mismatch" \
  >/dev/null 2>&1; then
  echo "release-version-contract: mismatched tag was accepted" >&2
  exit 1
fi

echo "release-version-contract: OK tag=${EXPECTED_TAG} artifacts=${ARTIFACT_NAMES[*]} version=${MANIFEST_VERSION}"
