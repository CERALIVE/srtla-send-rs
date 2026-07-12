#!/usr/bin/env bash
set -euo pipefail

die() { printf 'verify-bindings-release-ref: ERROR: %s\n' "$*" >&2; exit 1; }

[[ "${GITHUB_EVENT_NAME:-}" == "push" ]] || die "real publish requires a tag push event"
[[ "${GITHUB_REF_TYPE:-}" == "tag" ]] || die "real publish requires a tag ref"
[[ "${GITHUB_REF_NAME:-}" =~ ^bindings-v[0-9]{4}\.[0-9]+\.[0-9]+(-rc\.[0-9]+)?$ ]] \
  || die "tag must match bindings-vYYYY.M.P or bindings-vYYYY.M.P-rc.N"
[[ "${GITHUB_REF:-}" == "refs/tags/${GITHUB_REF_NAME}" ]] \
  || die "GITHUB_REF does not match GITHUB_REF_NAME"

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${HERE}/.." && pwd)"
PACKAGE_VERSION="$(node -p "require('${REPO_ROOT}/bindings/typescript/package.json').version")"
TAG_VERSION="${GITHUB_REF_NAME#bindings-v}"
[[ "${TAG_VERSION}" == "${PACKAGE_VERSION}" ]] \
  || die "tag version '${TAG_VERSION}' does not match package version '${PACKAGE_VERSION}'"

HEAD_SHA="$(git -C "${REPO_ROOT}" rev-parse 'HEAD^{commit}')"
EVENT_SHA="$(git -C "${REPO_ROOT}" rev-parse "${GITHUB_SHA:-missing}^{commit}" 2>/dev/null)" \
  || die "GITHUB_SHA does not resolve to a commit"
[[ "${HEAD_SHA}" == "${EVENT_SHA}" ]] \
  || die "checked-out commit '${HEAD_SHA}' does not match event commit '${EVENT_SHA}'"

echo "verify-bindings-release-ref: OK tag=${GITHUB_REF_NAME} version=${PACKAGE_VERSION} sha=${HEAD_SHA}"
