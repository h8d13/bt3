#!/usr/bin/env python3
"""
Compare per-function instruction counts between two rustc asm dumps.

Deterministic counterpart to bench-compare.py: timing noise on a laptop can
exceed ±50 %, instruction diffs do not.

Usage:
  cargo rustc --release --lib --all-features -- --emit asm -C codegen-units=1
  cp target/release/deps/balanced_ternary-*.s base.s
  # ... change code, emit again into new.s ...
  python3 asm-compare.py base.s new.s            # functions whose count moved
  python3 asm-compare.py base.s new.s --all      # include same-count changes
"""

import argparse
import re


def parse_asm(path: str) -> dict[str, list[str]]:
	funcs, cur = {}, None
	for line in open(path):
		m = re.match(r"^(_R\S+):$", line.rstrip())
		if m:
			cur = m.group(1)
			funcs[cur] = []
			continue
		if cur is None:
			continue
		s = line.strip()
		if s.startswith(".cfi_endproc"):
			cur = None
		elif s and not s.startswith((".", "#")) and not s.endswith(":"):
			funcs[cur].append(s)
	return funcs


def demangle(sym: str) -> str:
	# v0 mangling: identifiers are <len>[_]<name>; keep the readable ones.
	# Crate roots carry a base-62 hash (`Cs<hash>_<len><name>`): drop it.
	sym = re.sub(r"Cs[0-9A-Za-z]+_", "C", sym)
	names, i = [], 0
	while i < len(sym):
		m = re.match(r"(\d+)_?", sym[i:])
		if not m:
			i += 1
			continue
		n, start = int(m.group(1)), i + m.end()
		word = sym[start:start + n]
		if n and re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", word):
			names.append(word)
			i = start + n
		else:
			i += m.end()
	skip = {"balanced_ternary", "core", "ops", "arith", "bit", "fmt", "convert"}
	return "::".join(w for w in names if w not in skip) or sym


def main():
	ap = argparse.ArgumentParser(description=__doc__.splitlines()[1])
	ap.add_argument("base")
	ap.add_argument("new")
	ap.add_argument("--all", action="store_true",
		help="also list functions whose body changed at equal count")
	args = ap.parse_args()

	a, b = parse_asm(args.base), parse_asm(args.new)
	rows = []
	for sym in set(a) | set(b):
		na, nb = len(a.get(sym, [])), len(b.get(sym, []))
		if na != nb or (args.all and a.get(sym) != b.get(sym)):
			rows.append((nb - na, na, nb, demangle(sym)))
	rows.sort(key=lambda r: (-r[0], r[3]))
	for delta, na, nb, name in rows:
		print(f"{delta:+6d}  {na:5} -> {nb:5}  {name}")
	total_a = sum(map(len, a.values()))
	total_b = sum(map(len, b.values()))
	print(f"funcs {len(a)} -> {len(b)}  instrs {total_a} -> {total_b}")


if __name__ == "__main__":
	main()
