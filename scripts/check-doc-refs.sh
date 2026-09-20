#!/usr/bin/env bash
# check-doc-refs.sh — documentation reference consistency gate.
#
# Verifies that every `openspec/…` and `docs/…` path referenced in this repo's
# top-level docs (AGENTS.md, README.md) actually resolves on disk. Exits non-zero
# on the first dangling reference so a moved/renamed/deleted note is caught in CI
# instead of rotting silently (Rule A — docs stay in sync with the tree).
#
# Cross-repo references such as `CeraUI/docs/…` are intentionally skipped: they
# live in a sibling workspace checkout that does not exist in this standalone repo
# (Rule D). The negative look-behind below only matches `docs/`/`openspec/` when
# they are NOT preceded by a path character.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

doc_sources=(AGENTS.md README.md)

# Collect every in-repo openspec/ and docs/ path, strip trailing sentence
# punctuation, de-duplicate.
mapfile -t refs < <(
  grep -hoP '(?<![\w./-])(?:openspec|docs)/[\w./-]+' "${doc_sources[@]}" 2>/dev/null \
    | sed 's/[.,]*$//' \
    | sort -u
)

missing=0
if [ "${#refs[@]}" -gt 0 ]; then
  for ref in "${refs[@]}"; do
    if [ -e "$ref" ]; then
      echo "ok       $ref"
    else
      echo "DANGLING $ref" >&2
      missing=1
    fi
  done
fi

if [ "$missing" -ne 0 ]; then
  echo "check-doc-refs: FAIL — one or more referenced paths do not resolve" >&2
  exit 1
fi

echo "check-doc-refs: OK — all openspec/ and docs/ refs in ${doc_sources[*]} resolve"
