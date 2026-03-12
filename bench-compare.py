#!/usr/bin/env python3
"""
Compare two Divan benchmark output files showing both fastest and median.

Fastest (minimum sample) reflects best-achievable latency with warm caches.
Median captures typical latency including occasional OS scheduling noise.
Regression detection uses fastest to avoid false positives from jitter.

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


def parse_divan(fname: str) -> dict[str, tuple[float | None, float | None]]:
    """
    Parse a Divan bench output file into {flat/path: (fastest_ns, median_ns)}.

    Divan tree structure uses Unicode box-drawing characters:
      ╰─  ├─  │   to show depth and siblings.
    We match the full line with a regex so that leading │ characters in the
    tree prefix don't collide with the │ column separators.
    """
    results: dict[str, tuple[float | None, float | None]] = {}
    name_stack: list[str] = []

    _LINE_RE = re.compile(r"^((?:│\s{2}|\s{3})*)(├─|╰─)\s+(.*)")
    # Divan appends the fastest sample directly to the name field.
    _FASTEST_RE = re.compile(r'^(.*?)\s+([\d.]+\s*(?:ns|µs|us|ms|s))\s*$')

    with open(fname, encoding="utf-8") as f:
        for raw in f:
            line = raw.rstrip("\n")

            m = _LINE_RE.match(line)
            if not m:
                continue

            depth_str, _tree_char, rest = m.groups()
            depth = len(depth_str) // 3

            # rest = "<name> [fastest_time]  │ slowest │ median │ mean │ ..."
            pipe = rest.find("│")
            if pipe == -1:
                # Group header with no timing data — just track the name.
                bench_name = rest.strip()
                fastest_ns = median_ns = None
            else:
                name_part = rest[:pipe]
                # timing_cols: ["", " slowest", " median", " mean", ...]
                timing_cols = rest[pipe:].split("│")
                median_ns = parse_ns(timing_cols[2]) if len(timing_cols) > 2 else None
                # Extract the fastest time Divan appends to the name field.
                fm = _FASTEST_RE.match(name_part)
                if fm:
                    bench_name = fm.group(1).strip()
                    fastest_ns = parse_ns(fm.group(2))
                else:
                    bench_name = name_part.strip()
                    fastest_ns = None

            if not bench_name:
                continue

            name_stack = name_stack[:depth]
            name_stack.append(bench_name)

            if fastest_ns is not None or median_ns is not None:
                key = "/".join(name_stack)
                results[key] = (fastest_ns, median_ns)

    return results


def pct_change(before: float, after: float) -> float:
    return (before - after) / before * 100


def fmt_pct(p: float | None) -> str:
    if p is None:
        return "    n/a"
    return f"{p:>+7.1f}%"


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

    # diffs: (fastest_pct, bf, af, bm, am, name)
    diffs = []
    for name in b:
        if name not in n:
            continue
        bf, bm = b[name]
        af, am = n[name]
        # Use fastest for filtering and sorting; fall back to median if missing.
        ref_before = bf if bf is not None else bm
        ref_after  = af if af is not None else am
        if ref_before is None or ref_after is None:
            continue
        if ref_before < args.min_ns and ref_after < args.min_ns:
            continue
        fp = pct_change(bf, af) if bf is not None and af is not None else None
        mp = pct_change(bm, am) if bm is not None and am is not None else None
        sort_key = fp if fp is not None else mp
        diffs.append((sort_key, bf, af, bm, am, fp, mp, name))

    diffs.sort(reverse=True)

    only_base = sorted(set(b) - set(n))
    only_new  = sorted(set(n) - set(b))

    improved  = sum(1 for r in diffs if r[0] is not None and r[0] >  args.threshold)
    regressed = sum(1 for r in diffs if r[0] is not None and r[0] < -args.threshold)
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
        regressions  = [r for r in diffs if r[0] is not None and r[0] < -args.threshold][-args.top:]
        improvements = [r for r in diffs if r[0] is not None and r[0] >  args.threshold][:args.top]
        neutral      = [r for r in diffs if r[0] is None or abs(r[0]) <= args.threshold]
        rows = improvements + neutral + list(reversed(regressions))

    col = max((len(r[-1]) for r in rows), default=0)
    hdr = f"{'name':<{col}}  {'bst_b':>7}  {'bst_a':>7}  {'Δbst':>8}  {'med_b':>7}  {'med_a':>7}  {'Δmed':>8}"
    print(hdr)
    print("-" * len(hdr))
    for sort_key, bf, af, bm, am, fp, mp, name in rows:
        flag = "  +" if (sort_key is not None and sort_key > args.threshold) \
          else ("  -" if (sort_key is not None and sort_key < -args.threshold) else "   ")
        bf_s = f"{bf:>6.1f}ns" if bf is not None else "     n/a"
        af_s = f"{af:>6.1f}ns" if af is not None else "     n/a"
        bm_s = f"{bm:>6.1f}ns" if bm is not None else "     n/a"
        am_s = f"{am:>6.1f}ns" if am is not None else "     n/a"
        print(f"{name:<{col}}  {bf_s}  {af_s}  {fmt_pct(fp)}  {bm_s}  {am_s}  {fmt_pct(mp)}{flag}")

    # Regression exit code uses fastest (most reliable signal).
    actionable = sum(
        1 for _, bf, _, _, _, fp, _, _ in diffs
        if fp is not None and fp < -args.threshold
        and bf is not None and bf >= args.fail_min_ns
    )
    if actionable:
        sys.exit(1)


if __name__ == "__main__":
    main()
