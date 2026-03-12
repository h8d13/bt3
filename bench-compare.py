#!/usr/bin/env python3
"""
Compare two Divan benchmark output files.

Usage:
  cargo bench 2>&1 | tee bench-baseline.txt
  # ... make changes ...
  cargo bench 2>&1 | tee bench-new.txt
  python3 bench-compare.py bench-baseline.txt bench-new.txt [--threshold 17] [--min-ns 5]
"""

import re
import sys
import argparse


# Matches a number + unit, e.g. "12.34 ns", "1.23 µs", "4.56 ms", "7.89 s"
_TIME_RE = re.compile(r"([\d.]+)\s*(ns|µs|us|ms|s)\b")

_UNITS = {"ns": 1.0, "µs": 1e3, "us": 1e3, "ms": 1e6, "s": 1e9}


def parse_ns(text: str) -> float | None:
    """Return nanoseconds from a time string like '12.34 ns' or '1.23 µs'."""
    m = _TIME_RE.search(text)
    if not m:
        return None
    return float(m.group(1)) * _UNITS[m.group(2)]


def parse_divan(fname: str) -> dict[str, float]:
    """
    Parse a Divan bench output file into {flat/path: median_ns}.

    Divan tree structure uses Unicode box-drawing characters:
      ╰─  ├─  │   to show depth and siblings.
    We match the full line with a regex so that leading │ characters in the
    tree prefix don't collide with the │ column separators.
    """
    results: dict[str, float] = {}
    # stack of ancestor names indexed by depth (0 = top-level module)
    name_stack: list[str] = []

    # Match: prefix (│  or    groups) + tree char (├─ or ╰─) + rest of line.
    _LINE_RE = re.compile(r"^((?:│\s{2}|\s{3})*)(├─|╰─)\s+(.*)")

    with open(fname, encoding="utf-8") as f:
        for raw in f:
            line = raw.rstrip("\n")

            m = _LINE_RE.match(line)
            if not m:
                continue

            depth_str, _tree_char, rest = m.groups()
            depth = len(depth_str) // 3

            # rest = "<name> [fastest_time]  │ slowest │ median │ ..."
            # Find the first │ in rest to split name+fastest from timing cols.
            pipe = rest.find("│")
            if pipe == -1:
                # Group header with no timing data — just track the name.
                bench_name = rest.strip()
                median_ns = None
            else:
                name_part = rest[:pipe]
                # timing_cols: ["", " slowest", " median", " mean", ...]
                timing_cols = rest[pipe:].split("│")
                median_ns = parse_ns(timing_cols[2]) if len(timing_cols) > 2 else None
                # Strip the "fastest" time that Divan appends to the name field.
                bench_name = re.sub(r'\s+[\d.]+\s*(?:ns|µs|us|ms|s)\s*$', '', name_part).strip()

            if not bench_name:
                continue

            # Trim stack to current depth and push this name.
            name_stack = name_stack[:depth]
            name_stack.append(bench_name)

            # Only record leaf nodes (those with timing data in the median column).
            if median_ns is not None:
                key = "/".join(name_stack)
                results[key] = median_ns

    return results


def main():
    parser = argparse.ArgumentParser(description="Compare Divan benchmark results")
    parser.add_argument("baseline", help="Baseline benchmark file")
    parser.add_argument("new", help="New benchmark file")
    parser.add_argument("--threshold", type=float, default=17.0,
                        help="Min %% change to flag (default: 17%%)")
    parser.add_argument("--min-ns", type=float, default=5.0,
                        help="Skip ops faster than this ns in the report (default: 5.0)")
    parser.add_argument("--fail-min-ns", type=float, default=0.0,
                        help="Only exit 1 for regressions where 'before' >= this ns "
                             "(default: 0 = same as --min-ns). Useful on noisy CI clocks.")
    parser.add_argument("--top", type=int, default=0,
                        help="Show only top N regressions/improvements (0 = all)")
    args = parser.parse_args()
    if args.fail_min_ns == 0.0:
        args.fail_min_ns = args.min_ns

    b = parse_divan(args.baseline)
    n = parse_divan(args.new)

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
    print(f"{len(diffs)} benchmarks: +{improved} improved  -{regressed} regressed  ={unchanged} unchanged")
    if only_base:
        print(f"removed: {', '.join(only_base)}")
    if only_new:
        print(f"added:   {', '.join(only_new)}")
    print()

    rows = diffs
    if args.top:
        # Top N regressions + top N improvements
        regressions = [r for r in diffs if r[0] < -args.threshold][-args.top:]
        improvements = [r for r in diffs if r[0] >  args.threshold][:args.top]
        neutral = [r for r in diffs if abs(r[0]) <= args.threshold]
        rows = improvements + neutral + list(reversed(regressions))

    col = max((len(name) for _, _, _, name in rows), default=0)
    print(f"{'name':<{col}}  {'before':>8}  {'after':>8}  {'change':>8}")
    print("-" * (col + 32))
    for pct, before, after, name in rows:
        flag = "  +" if pct > args.threshold else ("  -" if pct < -args.threshold else "   ")
        print(f"{name:<{col}}  {before:>7.1f}ns  {after:>7.1f}ns  {pct:>+7.1f}%{flag}")

    actionable = sum(1 for p, before, _, _ in diffs if p < -args.threshold and before >= args.fail_min_ns)
    if actionable:
        sys.exit(1)


if __name__ == "__main__":
    main()
