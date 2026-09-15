#!/usr/bin/env bash
set -euo pipefail

records_only=false
if [[ ${1:-} == --records-only ]]; then records_only=true; shift; fi
if [[ $# -gt 3 ]]; then
  printf 'usage: %s [--records-only] [summary.json [manifest.json [results-directory]]]\n' "$0" >&2
  exit 2
fi
summary=${1:-test-results/bench/smoke/summary.json}
manifest=${2:-scripts/bench/manifests/smoke.json}
results=${3:-$(dirname "$summary")}
shopt -s nullglob
records=()
for path in "$results"/*/run-*.json; do
  if [[ $(basename "$path") =~ ^run-[0-9]+\.json$ ]]; then
    jq -e '.status == "ok"' "$path" > /dev/null
    records+=("$path")
  fi
done
for directory in "$results"/*/; do
  for index in 0 1; do
    if [[ ! -f "$directory/run-$index.json" && -f "$directory/run-$index.exhausted.json" ]]; then
      jq -e '.status == "failed"' "$directory/run-$index.exhausted.json" > /dev/null
      records+=("$directory/run-$index.exhausted.json")
    fi
  done
done
if [[ ${#records[@]} != 12 ]]; then
  printf 'assert_smoke: expected 12 completed index outcomes (success or exhaustion), found %s\n' "${#records[@]}" >&2
  exit 1
fi

binary=$(jq -er '.candidates | map(.bin) | unique | if length == 1 then .[0] else error("one binary required") end' "$manifest")
digest=$(sha256sum "$binary")
digest=${digest%% *}
receiver=$(jq -er '.receivers | map(select(.name == "ceralive")) | if length == 1 then .[0].bin else error("one CeraLive receiver required") end' "$manifest")
receiver_digest=$(sha256sum "$receiver")
receiver_digest=${receiver_digest%% *}
summary_input=$summary
if [[ $records_only == true ]]; then summary_input=/dev/null; fi
# Summary v1 has aggregates only; raw-only obligations must inspect its source records.
# Owner-calibrated coverage, not a majority/performance test: see the known-limitation note.
# docs/notes/scheduler-evaluation-2026-09.md records first-cell failures and isolated success.
jq -e -s --arg digest "$digest" --arg receiver_digest "$receiver_digest" --argjson records_only "$records_only" --slurpfile manifest "$manifest" --slurpfile summary "$summary_input" '
  def require($ok; $message): if $ok then . else error($message) end;
  def finite_nonnegative: type == "number" and isfinite and . >= 0;
  def cell_id: [.candidate, .scenario, .receiver, .srt_profile] | join("--");
  def exempt: .candidate == "adaptive" or .scenario == "D";
  . as $all | $manifest[0] as $m | $summary[0] as $s
  | [$all[] | select(.candidate.label != "adaptive" and .scenario.id == "A" and .status == "ok")] as $runs
  | require(($manifest|length) == 1 and ($records_only or ($summary|length) == 1); "one manifest and summary required")
  | require($m.campaign == "smoke" and $m.seed == 1; "smoke campaign/seed mismatch")
  | require(($m.candidates|map(.label)|sort) == ["adaptive","classic","enhanced"];
      "expected exactly three smoke candidates")
  | require(($m.candidates|map(.bin)|unique|length) == 1; "candidates must share one binary")
  | require(($m.cells|length) == 6 and all($m.cells[];
      .runs == 2 and .receiver == "ceralive" and .srt_profile == "production"
      and (.scenario == "A" or .scenario == "D")); "invalid smoke cells")
  | require(($m.cells|map(cell_id)|unique|length) == 6; "duplicate manifest cells")
  | require(all($all[]; .schema_version==1 and .campaign==$m.campaign and .seed==$m.seed);
      "record provenance mismatch")
  | require(all($all[];
      .receiver.kind=="ceralive" and .receiver.sha256==$receiver_digest
      and .srt_profile=={name:"production",latency_ms:2000,lossmaxttl:40}
      and .window=={start_ms:0,end_ms:(if .scenario.id=="A" then 45000 else 75000 end)}
      and (.status=="ok" or (.status=="failed" and (.reason|type)=="string"
        and (.candidate.label=="adaptive" or .scenario.id=="D" or .reason=="settle_timeout"))));
      "invalid outcome/provenance or non-settling failure in a required cell")
  | require(all($m.cells[]; . as $cell |
      [$all[] | select(.cell_id == ($cell|cell_id)) | .run_index] | sort == [0,1]);
      "missing, extra, or duplicate paired run")
  | [$m.cells[] | select(exempt|not) | . as $cell
      | select([$runs[] | select(.cell_id == ($cell|cell_id))] | length == 0)
      | cell_id] as $missing
  | require(($missing|length)==0;
      "required cells have zero successful runs (need 1 of 2): " + ($missing|join(", ")))
  | require(all($all[]; . as $r |
      any($m.cells[]; (. | cell_id) == $r.cell_id and .candidate == $r.candidate.label
        and .scenario == $r.scenario.id)); "record/cell identity mismatch")
  | require(all($runs[]; (.useful_goodput_bps | finite_nonnegative) and .useful_goodput_bps > 1000000
      and .no_traffic == false); "useful goodput must exceed 1 Mbps in every run")
  | require(all($runs[]; . as $r |
      [.raw.stats_csv.rows[] | select(.t_ms >= $r.window.start_ms and .t_ms <= $r.window.end_ms)] as $rows
      | ($rows|length) >= 2 and ($rows[-1].pkt_recv_total - $rows[0].pkt_recv_total) > 0);
      "missing positive measured receiver packet count")
  | require(all($runs[]; ((.per_link|length) == 2 or (.per_link|length) == 3)
      and .receiver.kind == "ceralive"
      and .srt_profile == {name:"production", latency_ms:2000, lossmaxttl:40}
      and .window == {start_ms:0, end_ms:(if .scenario.id == "A" then 45000 else 75000 end)});
      "link count, receiver, profile, or full window mismatch")
  | require(all($all[]; . as $r | any($m.candidates[];
      .label == $r.candidate.label and .args == ["--mode", .label]
      and .args == $r.candidate.args
      and ({PATH:"/usr/bin:/bin", RUST_LOG:"info"} + .env) == $r.candidate.env
      and .effective_config != null
      and ($r.status != "ok" or .effective_config == $r.sender.effective_config)));
      "effective configuration, arguments, or environment mismatch")
  | require(all($all[]; .candidate.bin_sha256 == $digest); "binary fingerprint mismatch")
  | if $records_only then
      "PASS: at least 1 of 2 successes in classic/A and enhanced/A; all 12 outcomes retained; all D and adaptive cells informational"
    else
    require($s.schema_version == 1 and ($s.groups|length) == 1
      and ([$s.groups[].cells[].n]|add) == ($runs|length); "summary must match actual required successes, never planned or failed counts")
  | require(($s.groups|map(.scenario)|sort) == ["A"]; "summary scenario mismatch")
  | require(all($s.groups[]; . as $g |
      .campaign == $m.campaign and .receiver == "ceralive" and .profile == "production"
      and (.cells|keys) == ["classic","enhanced"]
      and all(.cells|to_entries[]; . as $cell |
        [$runs[] | select(.scenario.id == $g.scenario and .candidate.label == $cell.key)] as $raw
      | .value.n == ($raw|length) and .value.run_indices == ($raw|map(.run_index)|sort) and .value.integrity_errors == []
        and .value.fingerprints == ($raw|map(.fingerprint)|unique)
        and .value.metrics.useful_goodput_bps.n == ($raw|length)
        and .value.metrics.useful_goodput_bps.median == (($raw|map(.useful_goodput_bps)|add) / ($raw|length))));
      "summary does not match its raw records")
  | "PASS: final one-of-two classic/A and enhanced/A smoke coverage; goodput, receiver packets, config and full windows verified; original indices preserved; all D and adaptive cells informational"
  end
' "${records[@]}"
