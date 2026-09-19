#!/usr/bin/env bash
set -euo pipefail

export BENCH_HISTORICAL_FROM_LOCK=0
case "${1:-}" in
  --historical-from-lock) export BENCH_HISTORICAL_FROM_LOCK=1; shift ;;
esac
if [[ $# != 1 ]]; then
  printf 'usage: bash scripts/bench/run_campaign.sh [--historical-from-lock] MANIFEST\n' >&2
  exit 2
fi
export BENCH_MANIFEST="$1"
exec cargo test --features test-internals --test bench_scheduler campaign -- --ignored --exact --nocapture
