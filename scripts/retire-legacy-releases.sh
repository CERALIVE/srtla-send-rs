#!/usr/bin/env bash
# Temporary retirement tool. Delete after the retirement ledger is complete.
# Target selection is literal; inventory can veto a plan, never enlarge it.
set -euo pipefail

refuse() {
  printf 'refused: %s\n' "$*" >&2
  exit 1
}

usage() {
  printf '%s\n' \
    'Usage: bash scripts/retire-legacy-releases.sh --repo {srtla-send-rs|srtla} [--dry-run|--apply]' \
    'Default: --dry-run. Apply requires RETIRE_CONFIRM equal to the selected short repo name.' \
    'GH_MOCK=1 GH_MOCK_FIXTURE=<json> reads a fixture and ONLY prints, even with --apply.' \
    'Fixture: {"repo":"CERALIVE/<repo>","tags":["tag"],"releases":[{"tag":"tag","assets":["name"]}]}' \
    '--also-tag <tag> is a refusal-test probe, never permission to extend the literal targets.' \
    'npm commands are printed for the owner only; this script never executes them.'
}

contains() {
  local needle=$1 candidate
  shift
  for candidate in "$@"; do
    [[ "$candidate" != "$needle" ]] || return 0
  done
  return 1
}

repo=''
mode=dry-run
mode_seen=false
also_tags=()
while (($#)); do
  case "$1" in
    --repo)
      [[ $# -ge 2 && -n "$2" && -z "$repo" ]] || refuse 'one --repo value is required'
      repo=$2
      shift 2
      ;;
    --dry-run|--apply)
      [[ "$mode_seen" == false ]] || refuse 'choose only one execution mode'
      mode=${1#--}
      mode_seen=true
      shift
      ;;
    --also-tag)
      [[ $# -ge 2 && -n "$2" ]] || refuse '--also-tag requires a value'
      also_tags+=("$2")
      shift 2
      ;;
    --help|-h) usage; exit 0 ;;
    *) refuse "unknown argument: $1" ;;
  esac
done

# The selected API repository is assigned from literals, never from checkout state.
case "$repo" in
  srtla-send-rs)
    full_repo=CERALIVE/srtla-send-rs
    target_tags=(
      v1.0.0 v1.0.1 v2.1.0 v2.1.1 v2.2.0 v2.2.1 v2.3.0 v.2.5.0
      v2.6.0 v2.6.1 v2.7.0 v2.7.1 v2.8.0 v3.0.0 v3.0.1 v3.1.0 v3.2.0 v3.3.0
      bindings-v2026.6.1 bindings-v2026.6.2 bindings-v2026.8.0
    )
    target_releases=(v1.0.0 v1.0.1 v3.0.1 v3.1.0 v3.2.0 v3.3.0)
    ;;
  srtla)
    full_repo=CERALIVE/srtla
    target_tags=(v2026.6.0 v2026.6.1 v2026.6.2)
    target_releases=(v2026.6.0 v2026.6.1 v2026.6.2)
    ;;
  modem-stack|modem-control|cerastream|CeraUI|srt|irl-srt-server)
    refuse "repository is on the refusal list: $repo" ;;
  *) refuse "repository is not allowlisted: $repo" ;;
esac
readonly repo full_repo mode
readonly -a target_tags target_releases

# Repository qualification matters: the sender's v3.1.0 is NOT the SLS release.
declare -Ar PROTECTED=(
  [CERALIVE/srtla-send-rs]=v4.1.0
  [CERALIVE/srtla]=''
  [CERALIVE/irl-srt-server]=v3.1.0
  [CERALIVE/srt]=srt-v1.5.7+ceralive.2
)
protected=()
[[ -z "${PROTECTED[$full_repo]}" ]] || protected+=("${PROTECTED[$full_repo]}")
readonly -a protected

# Validate the WHOLE candidate set before inventory reads or any mutation.
for tag in "${target_tags[@]}" "${target_releases[@]}" "${also_tags[@]}"; do
  if contains "$tag" "${protected[@]}"; then
    refuse "protected target $full_repo:$tag"
  fi
done
for tag in "${also_tags[@]}"; do
  contains "$tag" "${target_tags[@]}" || refuse "extra target is not allowlisted: $full_repo:$tag"
done
if [[ "$mode" == apply && "${RETIRE_CONFIRM:-}" != "$repo" ]]; then
  refuse "--apply requires RETIRE_CONFIRM=$repo"
fi
mock=${GH_MOCK:-0}
[[ "$mock" == 0 || "$mock" == 1 ]] || refuse 'GH_MOCK must be 0 or 1'
readonly mock
command -v jq >/dev/null || refuse 'jq is required'
if [[ "$mock" == 0 ]]; then
  command -v gh >/dev/null || refuse 'gh is required'
fi
export GH_HOST=github.com GH_PROMPT_DISABLED=1

read_inventory() {
  local tag_pages release_pages
  if [[ "$mock" == 1 ]]; then
    [[ -n "${GH_MOCK_FIXTURE:-}" && -f "$GH_MOCK_FIXTURE" && -r "$GH_MOCK_FIXTURE" ]] \
      || refuse 'GH_MOCK_FIXTURE must name a readable JSON file'
    inventory=$(jq -s 'if length == 1 then .[0] else error("one document required") end' \
      "$GH_MOCK_FIXTURE") || refuse 'invalid mock inventory JSON'
  else
    # Explicit GETs, no checkout-relative placeholders, no truncated release-list limit.
    tag_pages=$(gh api -X GET --paginate --slurp "repos/$full_repo/git/matching-refs/tags/?per_page=100") \
      || refuse "cannot read tag inventory for $full_repo"
    release_pages=$(gh api -X GET --paginate --slurp "repos/$full_repo/releases?per_page=100") \
      || refuse "cannot read release inventory for $full_repo"
    inventory=$(jq -n --arg repo "$full_repo" --argjson tags "$tag_pages" --argjson releases "$release_pages" '
      if (all($tags[]; type == "array") and all($releases[]; type == "array")
          and all($tags[][]; .ref | startswith("refs/tags/"))) then
        {repo: $repo, tags: [$tags[][] | .ref | ltrimstr("refs/tags/")],
         releases: [$releases[][] | {tag: .tag_name, assets: [.assets[].name]}]}
      else error("invalid API inventory") end
    ') || refuse "invalid API inventory for $full_repo"
  fi
  jq -e --arg repo "$full_repo" '
    def name: type == "string" and length > 0 and (explode | all(. >= 32 and . != 127));
    type == "object" and .repo == $repo
    and (.tags | type == "array" and all(.[]; name))
    and (.releases | type == "array" and all(.[];
      type == "object" and (.tag | name) and (.assets | type == "array" and all(.[]; name))))
    and ((.tags | length) == (.tags | unique | length))
    and ((.releases | length) == ([.releases[].tag] | unique | length))
  ' <<<"$inventory" >/dev/null || refuse "malformed or wrong-repository inventory for $full_repo"
  local names
  names=$(jq -r '.tags[]' <<<"$inventory") || refuse 'cannot parse tags'
  live_tags=()
  [[ -z "$names" ]] || mapfile -t live_tags <<<"$names"
  names=$(jq -r '.releases[].tag' <<<"$inventory") || refuse 'cannot parse releases'
  live_releases=()
  [[ -z "$names" ]] || mapfile -t live_releases <<<"$names"
}

validate_inventory() {
  local tag
  # Unknown names refuse even outside the version namespaces. Never infer new targets.
  # Tags and releases have distinct allowlists: a new release for an old tag is drift.
  for tag in "${live_tags[@]}"; do
    contains "$tag" "${target_tags[@]}" "${protected[@]}" \
      || refuse "unknown live tag $full_repo:$tag"
  done
  for tag in "${live_releases[@]}"; do
    contains "$tag" "${target_releases[@]}" "${protected[@]}" \
      || refuse "unknown live release $full_repo:$tag"
  done
  for tag in "${protected[@]}"; do
    contains "$tag" "${live_tags[@]}" || refuse "missing protected tag $full_repo:$tag"
    contains "$tag" "${live_releases[@]}" || refuse "missing protected release $full_repo:$tag"
  done
}

print_list() {
  local label=$1
  shift
  printf '%s (%d):' "$label" "$#"
  if (($#)); then printf ' %s' "$@"; else printf ' (none)'; fi
  printf '\n'
}

print_inventory() {
  print_list 'LIVE TAGS' "${live_tags[@]}"
  print_list 'LIVE RELEASES' "${live_releases[@]}"
  jq -r '.releases[] | "LIVE ASSETS " + .tag + ": " + (.assets | tojson)' <<<"$inventory"
}

print_npm() {
  printf '%s\n' \
    'NPM: OWNER ACTION ONLY; commands below are printed, never executed.' \
    'npm view @ceralive/srtla-send versions --json' \
    'npm unpublish @ceralive/srtla-send --force' \
    "npm deprecate @ceralive/srtla-send@'*' \"retired: absorbed into CeraUI (packages/srtla-send); no further releases\"" \
    'npm view @ceralive/srtla versions --json' \
    'Only if @ceralive/srtla exists, the owner may run:' \
    'npm unpublish @ceralive/srtla --force' \
    "npm deprecate @ceralive/srtla@'*' \"retired: absorbed into CeraUI (packages/srtla-send); no further releases\""
}

read_inventory
printf 'REPO: %s\nMODE: %s\nINVENTORY SOURCE: %s\n' "$full_repo" "$mode" "$([[ "$mock" == 1 ]] && printf fixture || printf github.com)"
print_inventory
print_list 'TARGETS TAGS' "${target_tags[@]}"
print_list 'TARGETS RELEASES' "${target_releases[@]}"
print_list 'PROTECTED' "${protected[@]}"
validate_inventory
printf 'PREFLIGHT: PASS\nCOMMANDS:\n'

# Release objects first; tag refs separately. No combined cleanup option.
# Arrays preserve the exact argv printed; there is no eval or shell command expansion.
for tag in "${target_releases[@]}"; do
  cmd=(gh release delete "$tag" --repo "$full_repo" --yes)
  printf '%s\n' "${cmd[*]}"
  if ! contains "$tag" "${live_releases[@]}"; then
    printf 'SKIP absent release: %s\n' "$tag"
  elif [[ "$mode" == apply && "$mock" == 0 ]]; then
    "${cmd[@]}" || refuse "release deletion failed for $full_repo:$tag; inspect live state before retry"
  fi
done
for tag in "${target_tags[@]}"; do
  cmd=(gh api -X DELETE "repos/$full_repo/git/refs/tags/$tag")
  printf '%s\n' "${cmd[*]}"
  if ! contains "$tag" "${live_tags[@]}"; then
    printf 'SKIP absent tag: %s\n' "$tag"
  elif [[ "$mode" == apply && "$mock" == 0 ]]; then
    "${cmd[@]}" || refuse "tag deletion failed for $full_repo:$tag; inspect live state before retry"
  fi
done
print_npm

if [[ "$mode" == apply && "$mock" == 0 ]]; then
  read_inventory
  printf 'POST-APPLY INVENTORY:\n'
  print_inventory
  validate_inventory
  for tag in "${live_tags[@]}" "${live_releases[@]}"; do
    contains "$tag" "${protected[@]}" || refuse "retirement incomplete: $full_repo:$tag remains"
  done
  printf 'POST-APPLY: PASS; only this repository\x27s protected names remain.\n'
else
  printf 'NO MUTATIONS: dry-run or fixture mode; no deletion or npm command executed.\n'
fi
