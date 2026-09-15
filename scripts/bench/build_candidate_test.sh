#!/usr/bin/env bash
set -euo pipefail
builder=$(realpath "$(dirname "$0")/build_candidate.sh")
source_root=$(GIT_MASTER=1 git rev-parse --show-toplevel)
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
GIT_MASTER=1 git clone --quiet --shared --no-checkout "$source_root" "$scratch/source"
GIT_MASTER=1 git -C "$scratch/source" checkout --quiet --detach "$(GIT_MASTER=1 git rev-parse HEAD)"
cd "$scratch/source"
for state in untracked staged unstaged; do
  case "$state" in
    untracked) printf 'dirty\n' > dirty-candidate-test ;;
    staged) GIT_MASTER=1 git add dirty-candidate-test ;;
    unstaged)
      GIT_MASTER=1 git reset --quiet -- dirty-candidate-test
      rm dirty-candidate-test
      printf '\n' >> Cargo.toml
      ;;
  esac
  # Given a dirty checkout, when building, then refuse before creating artifacts.
  if BENCH_ARTIFACT_DIR="$scratch/artifacts" bash "$builder" > "$scratch/stdout" 2> "$scratch/stderr"; then
    printf 'FAIL: accepted %s worktree\n' "$state" >&2
    exit 1
  fi
  [[ ! -e "$scratch/artifacts" && ! -s "$scratch/stdout" ]]
  printf 'PASS: rejects %s worktree before build\n' "$state"
done
