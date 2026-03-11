#!/usr/bin/env python3
"""
Compare two benchmark output files and show improvements/regressions.

Usage:
  # Save a baseline:
  cargo test --release --test bench -- --nocapture --test-threads=1 2>&1 | tee bench-baseline.txt

  # After changes, compare:
  cargo test --release --test bench -- --nocapture --test-threads=1 2>&1 | tee bench-new.txt
  python3 bench-compare.py bench-baseline.txt bench-new.txt
"""

import re
import sys
import argparse


def parse_bench(fname: str) -> dict[str, float]:
    rows = {}
    with open(fname) as f:
        for line in f:
            m = re.search(r"\(\s*([\d.]+)\s*ns/op\)", line)
            nm = re.match(r"\s+(.+?)\s{2,}[\d.]+ M ops", line)
            if m and nm:
                rows[nm.group(1).strip()] = float(m.group(1))
    return rows


def main():
    parser = argparse.ArgumentParser(description="Compare benchmark results")
    parser.add_argument("baseline", help="Baseline benchmark output file")
    parser.add_argument("new", help="New benchmark output file")
    parser.add_argument(
        "--threshold",
        type=float,
        default=5.0,
        help="Min %% change to report (default: 5%%)",
    )
    parser.add_argument(
        "--top",
        type=int,
        default=20,
        help="Number of top improvements/regressions to show (default: 20)",
    )
    parser.add_argument(
        "--min-ns",
        type=float,
        default=1.0,
        help="Ignore ops faster than this (ns) to reduce noise (default: 1.0 ns)",
    )
    parser.add_argument(
        "--fail-on-regression",
        action="store_true",
        help="Exit with code 1 if any regression exceeds --threshold (for CI use)",
    )
    args = parser.parse_args()

    b = parse_bench(args.baseline)
    n = parse_bench(args.new)

    diffs = []
    for name in b:
        if name in n:
            before, after = b[name], n[name]
            # Skip sub-ns ops — too noisy to measure reliably
            if before < args.min_ns and after < args.min_ns:
                continue
            pct = (before - after) / before * 100
            diffs.append((pct, before, after, name))

    diffs.sort(reverse=True)

    common = len(diffs)
    only_base = set(b) - set(n)
    only_new = set(n) - set(b)

    improved = sum(1 for p, _, _, _ in diffs if p > args.threshold)
    regressed = sum(1 for p, _, _, _ in diffs if p < -args.threshold)
    unchanged = common - improved - regressed

    print(f"Comparing: {args.baseline}  →  {args.new}")
    print(f"Common ops: {common}  |  improved: {improved}  unchanged: {unchanged}  regressed: {regressed}")
    if only_base:
        print(f"  Only in baseline ({len(only_base)}): {', '.join(sorted(only_base)[:5])}{'...' if len(only_base) > 5 else ''}")
    if only_new:
        print(f"  Only in new      ({len(only_new)}): {', '.join(sorted(only_new)[:5])}{'...' if len(only_new) > 5 else ''}")
    print()

    improvements = [(p, b, a, nm) for (p, b, a, nm) in diffs if p > args.threshold]
    regressions  = [(p, b, a, nm) for (p, b, a, nm) in diffs if p < -args.threshold]

    if improvements:
        print(f"TOP {min(args.top, len(improvements))} IMPROVEMENTS:")
        for pct, before, after, name in improvements[: args.top]:
            bar = "▓" * int(pct / 5)
            print(f"  {pct:+6.1f}%  {before:6.1f} → {after:6.1f} ns  {bar}  {name}")
        print()

    if regressions:
        print(f"TOP {min(args.top, len(regressions))} REGRESSIONS:")
        for pct, before, after, name in sorted(regressions)[-args.top :]:
            bar = "░" * int(-pct / 5)
            print(f"  {pct:+6.1f}%  {before:6.1f} → {after:6.1f} ns  {bar}  {name}")
        print()

    if not improvements and not regressions:
        print("No significant changes detected.")

    if args.fail_on_regression and regressions:
        sys.exit(1)


if __name__ == "__main__":
    main()
