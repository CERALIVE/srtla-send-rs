#!/usr/bin/env bash
set -euo pipefail
checker=$(realpath "$(dirname "$0")/assert_smoke.sh")
manifest=${1:-scripts/bench/manifests/smoke.json}
scratch=$(mktemp -d)
trap 'rm -rf -- "$scratch"' EXIT
cp "$manifest" "$scratch/manifest.json"
binary=$(jq -r '.candidates[0].bin' "$manifest")
digest=$(sha256sum "$binary")
digest=${digest%% *}
receiver=$(jq -r '.receivers[0].bin' "$manifest")
receiver_digest=$(sha256sum "$receiver")
receiver_digest=${receiver_digest%% *}

# Given synthetic checker fixtures, not measured campaign evidence.
while IFS= read -r cell; do
  id=$(jq -r '[.candidate,.scenario,.receiver,.srt_profile]|join("--")' <<< "$cell")
  mkdir "$scratch/$id"
  for index in 0 1; do
    jq -n --argjson cell "$cell" --argjson index "$index" --arg digest "$digest" --arg receiver_digest "$receiver_digest" --arg id "$id" --slurpfile manifest "$manifest" '
      ($manifest[0].candidates[] | select(.label == $cell.candidate)) as $candidate
      | {schema_version:1,campaign:"smoke",seed:1,cell_id:$id,run_index:$index,
         status:"ok",useful_goodput_bps:2000000,no_traffic:false,
         scenario:{id:$cell.scenario,hash:$digest},receiver:{kind:"ceralive",sha256:$receiver_digest},
         candidate:{label:$candidate.label,args:$candidate.args,
           env:({PATH:"/usr/bin:/bin",RUST_LOG:"info"}+$candidate.env),bin_sha256:$digest},
         sender:{cpu_ms:100,effective_config:$candidate.effective_config},fingerprint:$digest,
         viewer_loss_ratio:0,diagnostics:{pkt_belated_sum:0,reorder_distance_max:null},
         load_intervals:[],loadavg_1m:0.5,warnings:[],
         srt_profile:{name:"production",latency_ms:2000,lossmaxttl:40},
         window:{start_ms:0,end_ms:(if $cell.scenario=="A" then 45000 else 75000 end)},
         per_link:[{iface:"link0",share:0.5},{iface:"link1",share:0.5}],events:[{},{}],
         episodes:[{event_index:0,horizon_ms:30000,restore_ms:1000,graded:true,
                    recovered:true,complete:true,impacted:true,failover_ms:0,recovery_ms:1000}],
         raw:{stats_csv:{rows:[{t_ms:0,pkt_recv_total:0},{t_ms:1000,pkt_recv_total:100}]}}}
      | if $cell.candidate=="adaptive" or $cell.scenario=="D"
          or ($cell.candidate=="classic" and $index==0) then
          .status="failed" | .reason="settle_timeout" | .no_traffic=true | .useful_goodput_bps=0
        else . end
    ' > "$scratch/current.json"
    suffix=json
    if [[ $(jq -r '.status' "$scratch/current.json") == failed ]]; then suffix=exhausted.json; fi
    mv "$scratch/current.json" "$scratch/$id/run-$index.$suffix"
  done
done < <(jq -c '.cells[]' "$manifest")
jq -s '
  {schema_version:1,groups:(group_by(.scenario.id)|map(. as $group|
    {campaign:"smoke",scenario:.[0].scenario.id,receiver:"ceralive",profile:"production",
     cells:(group_by(.candidate.label)|map({key:.[0].candidate.label,value:{
       n:length,run_indices:map(.run_index)|sort,integrity_errors:[],
       fingerprints:map(.fingerprint)|unique,
       metrics:{useful_goodput_bps:{n:length,median:(map(.useful_goodput_bps)|add)/length}}
     }})|from_entries)}))}
' "$scratch"/*/run-[01].json > "$scratch/summary.json"

# When each required cell has coverage, then original indices and all failures survive.
bash "$checker" "$scratch/summary.json" "$scratch/manifest.json" "$scratch"
cp "$scratch/classic--A--ceralive--production/run-1.json" "$scratch/original-record.json"
bash "$(dirname "$checker")/report_smoke.sh" "$scratch/manifest.json" "$scratch" "$scratch/generated"
cmp "$scratch/original-record.json" "$scratch/classic--A--ceralive--production/run-1.json"
shopt -s nullglob
for id in classic--A enhanced--A; do
  directory="$scratch/$id--ceralive--production"
  for record in "$directory"/run-[01].json; do
    name=$(basename "$record" .json)
    mv "$record" "$scratch/$name.saved"
    jq '.status="failed" | .reason="settle_timeout"' "$scratch/$name.saved" > "$directory/$name.exhausted.json"
  done
  if bash "$checker" "$scratch/summary.json" "$scratch/manifest.json" "$scratch" > /dev/null 2>&1; then
    printf 'FAIL: exempted required cell %s\n' "$id" >&2
    exit 1
  fi
  for saved in "$scratch"/run-*.saved; do
    name=$(basename "$saved" .saved)
    mv "$saved" "$directory/$name.json"
    rm "$directory/$name.exhausted.json"
  done
  printf 'PASS: zero coverage stays blocking: %s\n' "$id"
done
optional="$scratch/adaptive--A--ceralive--production/run-0.exhausted.json"
cp "$optional" "$scratch/saved.json"
for mutation in '.candidate.bin_sha256="wrong"' '.receiver.sha256="wrong"' '.run_index=1'; do
  jq "$mutation" "$scratch/saved.json" > "$optional"
  if bash "$checker" "$scratch/summary.json" "$scratch/manifest.json" "$scratch" > /dev/null 2>&1; then
    printf 'FAIL: optional identity/setup corruption accepted: %s\n' "$mutation" >&2
    exit 1
  fi
done
cp "$scratch/saved.json" "$optional"
for id in classic--D enhanced--D adaptive--A adaptive--D; do
  optional="$scratch/$id--ceralive--production/run-0.exhausted.json"
  jq '.reason="execution_error"' "$optional" > "$scratch/optional.json"
  mv "$scratch/optional.json" "$optional"
done
bash "$checker" "$scratch/summary.json" "$scratch/manifest.json" "$scratch"
informational="$scratch/adaptive--D--ceralive--production/run-1.json"
jq '.cell_id="adaptive--D--ceralive--production" | .scenario.id="D" | .window.end_ms=75000 | .candidate.label="adaptive"
    | .candidate.args=["--mode","adaptive"]' \
  "$scratch/classic--A--ceralive--production/run-1.json" > "$scratch/informational-base.json"
for mutation in '.events=[]' '.episodes=[]' '.episodes=[{impacted:true,recovery_ms:null}]' '.episodes=[{impacted:true,recovery_ms:"+inf"}]'; do
  jq "$mutation" "$scratch/informational-base.json" > "$informational"
  bash "$checker" "$scratch/summary.json" "$scratch/manifest.json" "$scratch" > /dev/null
done
bash "$checker" --records-only /dev/null "$scratch/manifest.json" "$scratch" > /dev/null
bash "$(dirname "$checker")/assert_smoke_test.sh" "$scratch/summary.json" "$scratch/manifest.json" "$scratch"
printf 'PASS: one-of-two classic/enhanced A coverage; all D and adaptive cells informational; synthetic fixtures are not campaign evidence\n'
