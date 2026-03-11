#!/usr/bin/env bash
# bench-compare.sh — save a baseline or compare against one.
#
# Usage:
#   Save baseline:   ./bench-compare.sh save
#   Compare:         ./bench-compare.sh compare
#   Save + compare:  ./bench-compare.sh both

set -euo pipefail

BASELINE="bench-baseline.txt"
NEW="bench-new.txt"

run_bench() {
    cargo test --release --test bench -- --nocapture --test-threads=1 2>&1
}

case "${1:-compare}" in
  save)
    echo "Running benchmarks and saving baseline to $BASELINE..."
    run_bench | tee "$BASELINE"
    echo ""
    echo "Baseline saved to $BASELINE"
    ;;

  compare)
    if [[ ! -f "$BASELINE" ]]; then
        echo "No baseline found at $BASELINE. Run './bench-compare.sh save' first."
        exit 1
    fi
    echo "Running benchmarks..."
    run_bench | tee "$NEW"
    echo ""
    echo "============================================================"
    echo "  COMPARISON  (baseline vs new, sorted by improvement)"
    echo "============================================================"
    printf "  %-45s  %8s  %8s  %8s\n" "label" "before" "after" "delta"
    echo "  ------------------------------------------------------------"
    # Extract matching ns/op lines from both files and diff them
    paste \
        <(grep 'ns/op' "$BASELINE" | sed 's/.*(\(.*\) ns\/op).*/\1/' ) \
        <(grep 'ns/op' "$NEW"      | sed 's/.*(\(.*\) ns\/op).*/\1/' ) \
        <(grep 'ns/op' "$BASELINE" | sed 's/^  \(.\{45\}\).*/\1/'   ) \
    | awk '{
        before=$1; after=$2;
        label=substr($0, index($0,$3));
        if (before > 0) {
            pct = (before - after) / before * 100;
            printf "  %-45s  %6.1f ns  %6.1f ns  %+6.1f%%\n", label, before, after, pct
        }
    }' | sort -t'%' -k1 -rn
    ;;

  both)
    echo "Running benchmarks..."
    run_bench | tee "$NEW"
    if [[ -f "$BASELINE" ]]; then
        echo ""
        echo "============================================================"
        echo "  COMPARISON  (baseline vs new)"
        echo "============================================================"
        printf "  %-45s  %8s  %8s  %8s\n" "label" "before" "after" "delta"
        echo "  ------------------------------------------------------------"
        paste \
            <(grep 'ns/op' "$BASELINE" | sed 's/.*(\(.*\) ns\/op).*/\1/' ) \
            <(grep 'ns/op' "$NEW"      | sed 's/.*(\(.*\) ns\/op).*/\1/' ) \
            <(grep 'ns/op' "$BASELINE" | sed 's/^  \(.\{45\}\).*/\1/'   ) \
        | awk '{
            before=$1; after=$2;
            label=substr($0, index($0,$3));
            if (before > 0) {
                pct = (before - after) / before * 100;
                printf "  %-45s  %6.1f ns  %6.1f ns  %+6.1f%%\n", label, before, after, pct
            }
        }' | sort -t'%' -k1 -rn
    else
        echo "No baseline found. Run './bench-compare.sh save' to create one."
        cp "$NEW" "$BASELINE"
        echo "Saved current run as baseline."
    fi
    ;;

  *)
    echo "Usage: $0 [save|compare|both]"
    exit 1
    ;;
esac
