#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
SCRIPT=$ROOT/scripts/retire-legacy-releases.sh
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
export GH_MOCK=1 GH_MOCK_FIXTURE=$scratch/sender.json
unset RETIRE_CONFIRM

# Fail closed if any test accidentally invokes gh, npm, or git, including reads.
mkdir "$scratch/bin"
for tool in gh npm git; do
  printf '#!/usr/bin/env bash\nprintf "unexpected external command\\n" >> "%s/called"\nexit 99\n' "$scratch" >"$scratch/bin/$tool"
  chmod +x "$scratch/bin/$tool"
done
export PATH="$scratch/bin:$PATH"

sender_tags='v1.0.0 v1.0.1 v2.1.0 v2.1.1 v2.2.0 v2.2.1 v2.3.0 v.2.5.0 v2.6.0 v2.6.1 v2.7.0 v2.7.1 v2.8.0 v3.0.0 v3.0.1 v3.1.0 v3.2.0 v3.3.0 bindings-v2026.6.1 bindings-v2026.6.2 bindings-v2026.8.0'
sender_releases='v1.0.0 v1.0.1 v3.0.1 v3.1.0 v3.2.0 v3.3.0'
jq -n --arg tags "$sender_tags" --arg releases "$sender_releases" '
  {repo: "CERALIVE/srtla-send-rs", tags: (($tags | split(" ")) + ["v4.1.0"]),
   releases: (($releases | split(" ")) + ["v4.1.0"] | map({tag: ., assets: ["fixture.deb"]}))}
' >"$GH_MOCK_FIXTURE"

fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
expect_refused() {
  local output rc=0
  printf '$'; printf ' %q' "$@"; printf '\n'
  output=$("$@" 2>&1) || rc=$?
  printf '%s\nexit=%d\n' "$output" "$rc"
  [[ $rc -ne 0 && "$output" == *refused* ]] || fail 'expected nonzero refusal'
  [[ "$output" != *'COMMANDS:'* ]] || fail 'refusal happened after command planning'
}

expect_refused bash "$SCRIPT" --dry-run --repo modem-stack
expect_refused env -u RETIRE_CONFIRM bash "$SCRIPT" --apply --repo srtla-send-rs
expect_refused env RETIRE_CONFIRM=srtla-send-rs bash "$SCRIPT" --apply --repo srtla-send-rs --also-tag v4.1.0

output=$(bash "$SCRIPT" --dry-run --repo srtla-send-rs)
printf 'POSITIVE MOCK: GH_MOCK=1 --dry-run --repo srtla-send-rs\n%s\nexit=0\n' "$output"
grep -Fx "TARGETS TAGS (21): $sender_tags" <<<"$output" >/dev/null || fail 'wrong sender tag targets'
grep -Fx "TARGETS RELEASES (6): $sender_releases" <<<"$output" >/dev/null || fail 'wrong sender release targets'
grep -Fx 'PROTECTED (1): v4.1.0' <<<"$output" >/dev/null || fail 'missing sender protection'
[[ $(grep -c '^gh api -X DELETE ' <<<"$output") == 21 ]] || fail 'wrong tag command count'
[[ $(grep -c '^gh release delete ' <<<"$output") == 6 ]] || fail 'wrong release command count'
if grep -E '^gh .*v4\.1\.0' <<<"$output"; then fail 'protected command emitted'; fi
[[ $(bash "$SCRIPT" --repo srtla-send-rs) == "$output" ]] || fail 'default is not dry-run'
mock_apply=$(RETIRE_CONFIRM=srtla-send-rs bash "$SCRIPT" --apply --repo srtla-send-rs)
[[ "$mock_apply" == *'NO MUTATIONS:'* ]] || fail 'mock apply escaped print-only mode'

for repo in modem-control cerastream CeraUI srt irl-srt-server unexpected CERALIVE/srtla; do
  expect_refused bash "$SCRIPT" --repo "$repo"
done
expect_refused env RETIRE_CONFIRM=srtla bash "$SCRIPT" --apply --repo srtla-send-rs
expect_refused bash "$SCRIPT" --repo srtla-send-rs --also-tag v99.0.0
expect_refused bash "$SCRIPT" --repo srtla-send-rs --apply --dry-run
expect_refused bash "$SCRIPT" --repo srtla-send-rs --repo srtla
expect_refused bash "$SCRIPT" --repo

for mutation in \
  '.tags += ["v99.0.0"]' \
  '.tags += ["bindings-v2099.1.0"]' \
  '.releases += [{tag:"v2.1.0",assets:[]}]' \
  '.releases += [{tag:"bindings-v2099.1.0",assets:[]}]' \
  '.tags -= ["v4.1.0"]' \
  '.releases |= map(select(.tag != "v4.1.0"))' \
  '.repo = "CERALIVE/srtla"' \
  '.tags += ["v3.1.0"]' \
  '.tags = null'; do
  jq "$mutation" "$GH_MOCK_FIXTURE" >"$scratch/mutated.json"
  expect_refused env GH_MOCK_FIXTURE="$scratch/mutated.json" bash "$SCRIPT" --repo srtla-send-rs
done
printf '{broken\n' >"$scratch/invalid.json"
expect_refused env GH_MOCK_FIXTURE="$scratch/invalid.json" bash "$SCRIPT" --repo srtla-send-rs
expect_refused env GH_MOCK_FIXTURE="$scratch/absent.json" bash "$SCRIPT" --repo srtla-send-rs

jq -n '{repo:"CERALIVE/srtla",tags:["v2026.6.0","v2026.6.1","v2026.6.2"],
  releases: ["v2026.6.0","v2026.6.1","v2026.6.2"] | map({tag:.,assets:[]})}' >"$scratch/receiver.json"
output=$(GH_MOCK_FIXTURE="$scratch/receiver.json" bash "$SCRIPT" --repo srtla)
grep -Fx 'TARGETS TAGS (3): v2026.6.0 v2026.6.1 v2026.6.2' <<<"$output" >/dev/null || fail 'wrong receiver tags'
grep -Fx 'TARGETS RELEASES (3): v2026.6.0 v2026.6.1 v2026.6.2' <<<"$output" >/dev/null || fail 'wrong receiver releases'
grep -Fx 'PROTECTED (0): (none)' <<<"$output" >/dev/null || fail 'global protection leaked to receiver'
[[ $(grep -c '^gh api -X DELETE repos/CERALIVE/srtla/git/refs/tags/' <<<"$output") == 3 ]] || fail 'wrong receiver tag commands'
[[ $(grep -c '^gh release delete .* --repo CERALIVE/srtla --yes$' <<<"$output") == 3 ]] || fail 'wrong receiver release commands'
jq '.tags = [] | .releases = []' "$scratch/receiver.json" >"$scratch/empty.json"
GH_MOCK_FIXTURE="$scratch/empty.json" bash "$SCRIPT" --repo srtla >/dev/null

[[ ! -e "$scratch/called" ]] || fail 'gh/npm/git was invoked'
printf 'PASS: retirement contract; exact lists, repo-qualified protection, refusals, default dry-run, empty inventory, mock apply; zero gh/npm/git calls.\n'
