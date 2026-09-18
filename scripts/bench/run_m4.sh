#!/usr/bin/env bash
set -uo pipefail

phase=${1:?smoke or measure}
root=${2:?absolute durable artifact directory}
export PATH=/home/andres/.cargo/bin:/usr/local/bin:/usr/bin:/bin
export GIT_MASTER=1
export BENCH_MAX_RETRIES=1
export BENCH_PCAP_ON_FAIL=0

run_campaign() {
    local name=$1 manifest=$2 seconds=$3 code
    export BENCH_MANIFEST="$manifest"
    export BENCH_OUT_DIR="$root/$name/results"
    export BENCH_ARTIFACT_DIR="$root/$name/artifacts"
    mkdir -p "$BENCH_OUT_DIR" "$BENCH_ARTIFACT_DIR"
    local launch
    launch=$(mktemp -d "$root/$name/launch-XXXXXX")
    date --iso-8601=seconds >"$launch/start.txt"
    set -x
    taskset -c 4-27 timeout --foreground --kill-after=30s "$seconds" \
        target/m4-bench_scheduler --ignored --exact --nocapture campaign \
        >"$launch/campaign.log" 2>&1
    code=$?
    set +x
    printf '%s\n' "$code" >"$launch/exit-code.txt"
    date --iso-8601=seconds >"$launch/end.txt"
    [[ $code == 0 || $code == 101 ]]
}

date --iso-8601=seconds >"$root/$phase-start.txt"
case "$phase" in
    smoke)
        run_campaign smoke "$root/smoke-manifest.json" 1200s || exit 1
        uv run scripts/bench/m4_resume_check.py snapshot "$root/smoke" || exit 1
        run_campaign smoke "$root/smoke-manifest.json" 120s || exit 1
        uv run scripts/bench/m4_resume_check.py compare "$root/smoke" || exit 1
        ;;
    measure)
        sha256sum --check "$root/frozen.sha256" || exit 1
        uv run scripts/bench/m4_manifest.py --validate scripts/bench/manifests/m4a-ours-new.json || exit 1
        run_campaign a scripts/bench/manifests/m4a-ours-new.json 108000s || exit 1
        export BENCH_SOAK=1
        run_campaign soak scripts/bench/manifests/m4-soak.json 1800s || exit 1
        sha256sum --check "$root/frozen.sha256" || exit 1
        uv run scripts/bench/report.py --manifest scripts/bench/manifests/m4a-ours-new.json \
            --results "$root/a/results" --out docs/evidence/bpc/m4/a/report.md \
            --json docs/evidence/bpc/m4/a/summary.json || exit 1
        for suffix in provisional repeat; do
            uv run scripts/bench/decide.py --rule lineage-d1 \
                --summaries docs/evidence/bpc/m4/a/summary.json \
                --out "docs/evidence/bpc/m4/verdict-$suffix.json" || exit 1
        done
        cmp docs/evidence/bpc/m4/verdict-provisional.json docs/evidence/bpc/m4/verdict-repeat.json || exit 1
        sha256sum docs/evidence/bpc/m4/verdict-{provisional,repeat}.json
        uv run scripts/bench/m4_supplements.py "$root" docs/evidence/bpc/m4/supplements.json || exit 1
        ;;
    *) exit 2 ;;
esac
date --iso-8601=seconds >"$root/$phase-end.txt"
