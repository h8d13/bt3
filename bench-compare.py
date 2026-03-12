#!/usr/bin/env python3
"""
Compare two benchmark output files.

Usage:
  cargo test --release --test bench -- --nocapture --test-threads=1 2>&1 | tee bench-baseline.txt
  # ... make changes ...
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
    parser.add_argument("baseline", help="Baseline benchmark file")
    parser.add_argument("new", help="New benchmark file")
    parser.add_argument("--threshold", type=float, default=5.0,
                        help="Min %% change to highlight (default: 5%%)")
    parser.add_argument("--min-ns", type=float, default=1.0,
                        help="Skip ops faster than this ns (default: 1.0)")
    parser.add_argument("--fail-on-regression", action="store_true",
                        help="Exit 1 if any regression exceeds --threshold")
    args = parser.parse_args()

    b = parse_bench(args.baseline)
    n = parse_bench(args.new)

    diffs = []
    for name in b:
        if name in n:
            before, after = b[name], n[name]
            if before < args.min_ns and after < args.min_ns:
                continue
            pct = (before - after) / before * 100
            diffs.append((pct, before, after, name))

    diffs.sort(reverse=True)

    only_base = sorted(set(b) - set(n))
    only_new  = sorted(set(n) - set(b))

    improved  = sum(1 for p, *_ in diffs if p >  args.threshold)
    regressed = sum(1 for p, *_ in diffs if p < -args.threshold)
    unchanged = len(diffs) - improved - regressed

    print(f"{args.baseline} -> {args.new}")
    print(f"{len(diffs)} ops: +{improved} improved  -{regressed} regressed  ={unchanged} unchanged")
    if only_base:
        print(f"removed: {', '.join(only_base)}")
    if only_new:
        print(f"added:   {', '.join(only_new)}")
    print()

    col = max((len(name) for _, _, _, name in diffs), default=0)

    print(f"{'name':<{col}}  {'before':>8}  {'after':>8}  {'change':>8}")
    print("-" * (col + 30))
    for pct, before, after, name in diffs:
        flag = "  +" if pct > args.threshold else ("  -" if pct < -args.threshold else "   ")
        print(f"{name:<{col}}  {before:>7.1f}ns  {after:>7.1f}ns  {pct:>+7.1f}%{flag}")

    if args.fail_on_regression and regressed:
        sys.exit(1)


if __name__ == "__main__":
    main()
