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
  ARTIFACT="${TMPDIR}/out/srtla-send-rs_${MANIFEST_VERSION}_${ARCH}.deb"
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
  fi
done

if GITHUB_REF_TYPE=tag GITHUB_REF_NAME="${EXPECTED_TAG}-mismatch" \
  "${REPO_ROOT}/ci/build-deb.sh" amd64 "${TMPDIR}/srtla_send" "${TMPDIR}/mismatch" \
  >/dev/null 2>&1; then
  echo "release-version-contract: mismatched tag was accepted" >&2
  exit 1
fi

echo "release-version-contract: OK tag=${EXPECTED_TAG} artifacts=${ARTIFACT_NAMES[*]} version=${MANIFEST_VERSION}"
