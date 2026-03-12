[![Rust](https://github.com/Trehinos/balanced-ternary/actions/workflows/rust.yml/badge.svg)](https://github.com/Trehinos/balanced-ternary/actions/workflows/rust.yml)

# BT3 BrainTruck3

**BT3** is a Rust library for manipulating **[balanced ternary](https://en.wikipedia.org/wiki/Balanced_ternary)** a numeral system with digits `-1`, `0`, and `+1`.

This system is particularly useful in specialized computing applications such as reversible computing, digital signal processing, and three-valued logic modeling. 

This fork extends the original [`balanced-ternary`](https://github.com/Trehinos/balanced-ternary) with additional fixed-width types, unsigned ternary types following [Jones's design](https://homepage.divms.uiowa.edu/~jones/ternary/), TERSCII encoding, and significant performance improvements throughout.

## Features

- **No Standard Library:** Suitable for `#![no_std]` environments.
- **Number Conversions:** Convert between decimal and balanced ternary representations.
- **Arithmetic Operations:** Support for addition, subtraction, multiplication, division, and bit shifting (`<<`, `>>`).
- **[Three-value Logic Operations](https://en.wikipedia.org/wiki/Three-valued_logic):**
    - Support for bitwise and, or, xor, and not (in Kleene algebra (K3)).
    - **Advanced logic**: Implementation of
      [K3](https://en.wikipedia.org/wiki/De_Morgan_algebra#Kleene_algebra),
      [BI3](https://en.wikipedia.org/wiki/Many-valued_logic#Bochvar's_internal_three-valued_logic),
      [L3](https://en.wikipedia.org/wiki/%C5%81ukasiewicz_logic),
      [RM3](https://en.wikipedia.org/wiki/Paraconsistent_logic#An_ideal_three-valued_paraconsistent_logic),
      [paraconsistent-logic](https://en.wikipedia.org/wiki/Paraconsistent_logic#An_ideal_three-valued_paraconsistent_logic)
      and [HT](https://en.wikipedia.org/wiki/Intermediate_logic) imply operations,
      and more HT, BI3, L3 and post-logic operations.
    - **Consensus** (`a · (a == b)`): extracts positions where both trits agree; zero elsewhere.
    - **Accept-anything** (`sign(a + b)`): the non-zero trit wins; conflicting non-zero trits → `Zero`.
- **Custom Representation:** Parse and display numbers using `+`, `0`, and `-` symbols by default, or custom ones.
- **Multiple storage formats** for different performance/memory trade-offs (see [Storage types](#storage-types)).
- **TERSCII:** Jones's 81-character ternary ASCII encoding for ternary text processing.

## Library features

All features are enabled by default.

To enable only some features, use the `default-features` option
in your [dependency declaration](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features):

```toml
[dependencies.balanced-ternary]
version = "*.*"
default-features = false
# Choose which ones to enable
features = ["ternary-string", "tryte", "ternary-store", "terscii"]
```

### `ternary-string`

Provides the `Ternary` type: a heap-allocated `Vec<Digit>` for arbitrary-precision balanced ternary.
Implements `DigitOperate`.

### `tryte`

> Requires `ternary-string`.

Provides `Tryte<N>`: a stack-allocated, `Copy`, fixed-width ternary number of exactly `N` trits.
Implements `DigitOperate`.

### `ternary-store`

> Requires `ternary-string`.

Provides fixed-width storage types optimised for performance and memory:

| Type | Storage | Trits | Range | Notes |
|------|---------|-------|-------|-------|
| `TritsChunk` | `i8` | 5 | −121..121 | 5 balanced trits in one byte |
| `DataTernary` | `Vec<TritsChunk>` | variable | arbitrary | 5 trits/byte compact storage |
| `Ter40` | `i64` | 40 | ±(3⁴⁰−1)/2 | Fastest arithmetic — native i64 ops |
| `IlTer40` | `u128` | 40 | ±(3⁴⁰−1)/2 | Interleaved BCT; O(1) trit logic on all 40 trits |
| `BctTer32` | `(u32, u32)` | 32 | ±(3³²−1)/2 | Split BCT; O(1) trit-logical ops |
| `IlBctTer32` | `u64` | 32 | ±(3³²−1)/2 | Jones interleaved BCT; O(1) logic + arithmetic |
| `UTer9` | `u32` | 9 | 0..19682 | Unsigned BCT, Jones `uter9_t`; O(1) `uter_add` |
| `UTer27` | `u64` | 27 | 0..7625597484986 | Unsigned BCT, Jones `uter27_t`; O(1) `uter_add` |
| `BTer9` | `u32` | 9 | −9841..9841 | Balanced BCT, Jones `bter9_t` |
| `BTer27` | `u64` | 27 | ±(3²⁷−1)/2 | Balanced BCT, Jones `bter27_t` |

### `terscii`

> Requires `ternary-string` and `tryte`.

Provides Jones's [TERSCII](https://homepage.divms.uiowa.edu/~jones/ternary/terscii.shtml) encoding:
an 81-character table arranged in a 9×9 grid, where each character maps to a 4-trit ternary value.
Covers printable ASCII, common controls (space, tab, newline), and punctuation.

## Storage types

Choosing the right type for your use case matters:

- **Arbitrary precision / unknown size** → `Ternary` (heap, O(n) ops)
- **Fixed size, stack-allocated, `Copy`** → `Tryte<N>` (stack, SIMD-vectorizable bitwise)
- **Compact storage of variable data** → `DataTernary` (5 trits/byte)
- **Fastest arithmetic on ~40-trit values** → `Ter40` (pure i64, no decomposition)
- **O(1) bitwise over all trits at once** → `IlTer40` or `IlBctTer32`
- **O(1) arithmetic + logic combined** → `IlBctTer32` / `UTer9` / `UTer27`
- **Unsigned base-3 counting** → `UTer9` / `UTer27` (BCT addition via Jones BCD trick)

## Three-valued logic

The library supports numerous three-valued logic operations, each of them having its own specificities:

- **K3** (Kleene logic)
  A three-valued logic that introduces an "unknown" (0) state, useful for dealing with partial information.
- **BI3** (Bochvar logic)
  A logic designed to handle "nonsense" or meaningless statements, where 0 represents an invalid or undefined value.
- **L3** (Łukasiewicz logic)
  A non-classical logic allowing for degrees of truth, often used in fuzzy logic and multi-valued reasoning.
- **RM3** (Routley-Meyer paraconsistent logic)
  A logic that tolerates contradictions without collapsing into triviality, useful in paraconsistent reasoning.
- **HT** (Heyting logic-inspired ternary system)
  A variant of intermediate logic, often related to intuitionistic logic and constructive reasoning.
- **Paraconsistent logic**
  A logic framework that avoids the principle of explosion, allowing systems to work with contradictory information.
- **Post logic**
  A logical system that extends classical logic with additional operators to handle uncertainty in a structured way.
- **Consensus** (`a · (a == b)`)
  Keeps the trit value where both operands agree, `Zero` elsewhere. Useful for extracting shared information from two ternary words.
- **Accept-anything** (`sign(a + b)`)
  The non-zero trit wins; two conflicting non-zero trits resolve to `Zero`. Lets two ternary words with non-overlapping fields be merged losslessly.
- **Decoders** (`is_neg`, `is_zero`, `is_pos`)
  Return `Pos` when the trit matches the named value, `Neg` otherwise. Jones NTI / K / PTI indicator functions.
- **Clamps** (`clamp_down`, `clamp_up`)
  Suppress one polarity: `clamp_down` is min with Zero, `clamp_up` is max with Zero. Branchless via arithmetic right shift.
- **Equality**
  Returns `Pos` if both trits are equal, `Neg` otherwise: a per-trit equality test.

## License

Copyright (c) 2025 [Sébastien GELDREICH](mailto:dev@trehinos.eu)
`Balanced Ternary` is licensed under the [MIT License](LICENSE).
