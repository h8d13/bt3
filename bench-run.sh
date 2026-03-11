#!/usr/bin/env bash
# bench-run.sh — compile, benchmark (single + multi-threaded), save with timestamp,
# and auto-compare to previous run.
#
# Usage:
#   ./bench-run.sh                         # default (threshold 15%, min-ns 5)
#   ./bench-run.sh --threshold 10 --top 30 # override bench-compare.py options
#   BENCH_THRESHOLD=10 ./bench-run.sh      # via env var
#
# Files saved:
#   bench-runs/bench-YYYYMMDD-HHMMSS-1t.txt   single-threaded
#   bench-runs/bench-YYYYMMDD-HHMMSS-mt.txt   multi-threaded (default threads)
#
# Comparison:
#   - New 1t vs previous 1t  (primary: cleanest signal for optimization)
#   - New 1t vs new mt       (shows parallelism noise impact)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUNS_DIR="$SCRIPT_DIR/bench-runs"
mkdir -p "$RUNS_DIR"

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
FILE_1T="$RUNS_DIR/bench-${TIMESTAMP}-1t.txt"
FILE_MT="$RUNS_DIR/bench-${TIMESTAMP}-mt.txt"

COMPARE_PY="$SCRIPT_DIR/bench-compare.py"
THRESHOLD="${BENCH_THRESHOLD:-15}"
MIN_NS="${BENCH_MIN_NS:-5}"
# Extra args passed by the user (e.g. --threshold 10 --top 30)
EXTRA_ARGS=("$@")

run_bench() {
    local threads="$1"
    local outfile="$2"
    local label="$3"
    echo ""
    echo "==> Running benchmarks ($label)..."
    if [ "$threads" = "1" ]; then
        cargo test --release --test bench -- --nocapture --test-threads=1 2>&1 | tee "$outfile"
    else
        cargo test --release --test bench -- --nocapture 2>&1 | tee "$outfile"
    fi
    echo "    Saved: $outfile"
}

do_compare() {
    local old="$1"
    local new="$2"
    local label="$3"
    echo ""
    echo "==> $label"
    echo ""
    python3 "$COMPARE_PY" "$old" "$new" \
        --threshold "$THRESHOLD" \
        --min-ns "$MIN_NS" \
        "${EXTRA_ARGS[@]+"${EXTRA_ARGS[@]}"}"
}

# --- Run both variants ---
run_bench 1  "$FILE_1T" "single-threaded, --test-threads=1"
run_bench mt "$FILE_MT" "multi-threaded, default threads"

echo ""
echo "=================================================="

# --- Compare new 1t vs previous 1t ---
PREV_1T=$(ls -t "$RUNS_DIR"/bench-*-1t.txt 2>/dev/null | sed -n '2p')
if [ -n "$PREV_1T" ]; then
    do_compare "$PREV_1T" "$FILE_1T" "Single-threaded: prev → new (optimization signal)"
else
    echo "==> No previous single-threaded run to compare against."
fi

# --- Compare new 1t vs new mt (noise impact of parallel execution) ---
echo ""
echo "=================================================="
do_compare "$FILE_1T" "$FILE_MT" "This run: single-threaded → multi-threaded (noise impact)"
