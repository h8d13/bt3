//! Performance benchmarks using only std::time (no extra dependencies).
//!
//! Run with: cargo test --release --test bench -- --nocapture --test-threads=1
//!
//! ## Baseline comparison
//!
//! Save a baseline with:
//!   cargo test --release --test bench -- --nocapture --test-threads=1 2>&1 | tee bench-baseline.txt
//!
//! After changes, compare with the Python script:
//!   cargo test --release --test bench -- --nocapture --test-threads=1 2>&1 | tee bench-new.txt

use balanced_ternary::concepts::DigitOperate;
use balanced_ternary::terscii;
use balanced_ternary::*;
use std::ops::{BitAnd, Not};
use std::time::Instant;

const ITERS: u32 = 500_000;

/// Runs `f` for `iters` iterations across `PASSES` timed samples (after warmup).
///
/// Reports best (fastest sample) and median.  The compare script keys on
/// `(X.X ns/op)` for best and `M ops/s` for the label — both kept stable.
fn bench<F: FnMut()>(label: &str, iters: u32, mut f: F) {
    const PASSES: usize = 9;

    // Warmup: ~10% of iters so CPU boost and I-cache are hot before timing.
    for _ in 0..iters / 10 {
        f();
    }

    let mut samples = [0f64; PASSES];
    for s in &mut samples {
        let start = Instant::now();
        for _ in 0..iters {
            f();
        }
        *s = start.elapsed().as_secs_f64() * 1e9 / iters as f64;
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let best_ns   = samples[0];
    let median_ns = samples[PASSES / 2]; // index 4 of 9

    let ops_sec = 1e9 / best_ns;
    let (ops_val, ops_unit) = if ops_sec >= 1_000_000.0 {
        (ops_sec / 1_000_000.0, "M ops/s")
    } else if ops_sec >= 1_000.0 {
        (ops_sec / 1_000.0, "K ops/s")
    } else {
        (ops_sec, "  ops/s")
    };

    // `(X.X ns/op)` is parsed by bench-compare.py for best-latency tracking.
    println!(
        "  {:<52}  {:>8.2} {}  ({:>6.1} ns/op)  med {:>6.1} ns  [{} × {}]",
        label, ops_val, ops_unit, best_ns, median_ns, PASSES, iters
    );
}

// === Helpers to build large ternary values ===

/// Build a Ternary with `n` digits by repeating "+-0"
fn make_ternary(n: usize) -> Ternary {
    let pattern = "+-0";
    let s: String = pattern.chars().cycle().take(n).collect();
    Ternary::parse(&s)
}

/// Build a second Ternary with `n` digits by repeating "-0+"
fn make_ternary_b(n: usize) -> Ternary {
    let pattern = "-0+";
    let s: String = pattern.chars().cycle().take(n).collect();
    Ternary::parse(&s)
}

// ---------------------------------------------------------------------------
#[test]
fn bench_conversions() {
    println!("\n============================================================");
    println!("  CONVERSIONS");
    println!("============================================================");

    println!("\n--- from_dec ---");
    for &(label, val) in &[
        ("from_dec(0)", 0i64),
        ("from_dec(42)", 42),
        ("from_dec(18_887_455)", 18_887_455),
        ("from_dec(1_000_000_000)", 1_000_000_000),
        ("from_dec(i64::MAX/4)", i64::MAX / 4),
        ("from_dec(i64::MAX/2)", i64::MAX / 2),
        ("from_dec(-i64::MAX/2)", -(i64::MAX / 2)),
    ] {
        bench(label, ITERS, || {
            std::hint::black_box(Ternary::from_dec(std::hint::black_box(val)));
        });
    }

    println!("\n--- to_dec ---");
    let sizes = [4, 10, 20, 30, 40];
    for &n in &sizes {
        let t = make_ternary(n);
        bench(&format!("to_dec ({n}-digit)"), ITERS, || {
            std::hint::black_box(t.to_dec());
        });
    }

    println!("\n--- roundtrip ---");
    for &(label, val) in &[
        ("roundtrip(42)", 42i64),
        ("roundtrip(18_887_455)", 18_887_455),
        ("roundtrip(1_000_000_000)", 1_000_000_000),
        ("roundtrip(i64::MAX/2)", i64::MAX / 2),
    ] {
        bench(label, ITERS, || {
            let t = Ternary::from_dec(std::hint::black_box(val));
            std::hint::black_box(t.to_dec());
        });
    }
}

// ---------------------------------------------------------------------------
#[test]
fn bench_arithmetic() {
    println!("\n============================================================");
    println!("  ARITHMETIC");
    println!("============================================================");

    let sizes = [4, 10, 16, 24, 32, 40];

    println!("\n--- add ---");
    for &n in &sizes {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        bench(&format!("add ({n}-digit)"), ITERS, || {
            std::hint::black_box(&a + &b);
        });
    }

    println!("\n--- sub ---");
    for &n in &sizes {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        bench(&format!("sub ({n}-digit)"), ITERS, || {
            std::hint::black_box(&a - &b);
        });
    }

    println!("\n--- mul ---");
    for &n in &[4, 10, 16, 20] {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        // skip sizes where product overflows i64
        let ad = a.to_dec();
        let bd = b.to_dec();
        if ad.checked_mul(bd).is_some() {
            bench(&format!("mul ({n}-digit)"), ITERS, || {
                std::hint::black_box(&a * &b);
            });
        }
    }

    println!("\n--- div ---");
    for &n in &[4, 10, 16, 24] {
        let a = make_ternary(n);
        let b = make_ternary_b(4.max(n / 3)); // smaller divisor
        if b.to_dec() != 0 {
            bench(&format!("div ({n}/{}-digit)", 4.max(n / 3)), ITERS, || {
                std::hint::black_box(&a / &b);
            });
        }
    }

    println!("\n--- neg ---");
    for &n in &sizes {
        let a = make_ternary(n);
        bench(&format!("neg ({n}-digit)"), ITERS, || {
            std::hint::black_box(-&a);
        });
    }
}

// ---------------------------------------------------------------------------
#[test]
fn bench_bitwise() {
    println!("\n============================================================");
    println!("  BITWISE");
    println!("============================================================");

    let sizes = [4, 10, 16, 24, 32, 40];

    for &n in &sizes {
        let a = make_ternary(n);
        let b = make_ternary_b(n);

        bench(&format!("and ({n}-digit)"), ITERS, || {
            std::hint::black_box(&a & &b);
        });
        bench(&format!("or  ({n}-digit)"), ITERS, || {
            std::hint::black_box(&a | &b);
        });
        bench(&format!("xor ({n}-digit)"), ITERS, || {
            std::hint::black_box(&a ^ &b);
        });
    }
}

// ---------------------------------------------------------------------------
#[test]
fn bench_shift() {
    println!("\n============================================================");
    println!("  SHIFT");
    println!("============================================================");

    let sizes = [10, 20, 40];
    let shifts = [1, 5, 10, 20];

    for &n in &sizes {
        let a = make_ternary(n);
        for &s in &shifts {
            bench(&format!("shl {s} ({n}-digit)"), ITERS, || {
                std::hint::black_box(&a << s);
            });
        }
        for &s in &shifts {
            bench(&format!("shr {s} ({n}-digit)"), ITERS, || {
                std::hint::black_box(&a >> s);
            });
        }
    }
}

// ---------------------------------------------------------------------------
#[test]
fn bench_digit_operate() {
    println!("\n============================================================");
    println!("  DIGIT OPERATE");
    println!("============================================================");

    let sizes = [4, 10, 16, 24, 32, 40];

    println!("\n--- each(not) ---");
    for &n in &sizes {
        let a = make_ternary(n);
        bench(&format!("each(not) ({n}-digit)"), ITERS, || {
            std::hint::black_box(a.each(Digit::not));
        });
    }

    println!("\n--- each_zip(bitand) ---");
    for &n in &sizes {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        bench(&format!("each_zip(bitand) ({n}-digit)"), ITERS, || {
            std::hint::black_box(a.each_zip(Digit::bitand, b.clone()));
        });
    }

    println!("\n--- each_zip_carry (ternary add) ---");
    for &n in &sizes {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        bench(
            &format!("each_zip_carry(add) ({n}-digit)"),
            ITERS,
            || {
                std::hint::black_box(a.each_zip_carry(
                    |a, b, c| {
                        let sum = a.to_i8() + b.to_i8() + c.to_i8();
                        match sum {
                            -3 => (Digit::Neg, Digit::Zero),
                            -2 => (Digit::Neg, Digit::Pos),
                            -1 => (Digit::Zero, Digit::Neg),
                            0 => (Digit::Zero, Digit::Zero),
                            1 => (Digit::Zero, Digit::Pos),
                            2 => (Digit::Pos, Digit::Neg),
                            3 => (Digit::Pos, Digit::Zero),
                            _ => unreachable!(),
                        }
                    },
                    b.clone(),
                ));
            },
        );
    }
}

// ---------------------------------------------------------------------------
#[test]
fn bench_utilities() {
    println!("\n============================================================");
    println!("  UTILITIES");
    println!("============================================================");

    println!("\n--- parse ---");
    for &n in &[4, 10, 16, 24, 32, 40] {
        let s: String = "+-0".chars().cycle().take(n).collect();
        bench(&format!("parse ({n}-char)"), ITERS, || {
            std::hint::black_box(Ternary::parse(std::hint::black_box(&s)));
        });
    }

    println!("\n--- to_string ---");
    for &n in &[4, 10, 16, 24, 32, 40] {
        let t = make_ternary(n);
        bench(&format!("to_string ({n}-digit)"), ITERS, || {
            std::hint::black_box(t.to_string());
        });
    }

    println!("\n--- trim ---");
    for &zeros in &[4, 10, 20, 30] {
        let s = format!("{}{}", "0".repeat(zeros), "+-0+-0+-0+");
        let t = Ternary::parse(&s);
        bench(
            &format!("trim ({} leading zeros + 10 digits)", zeros),
            ITERS,
            || {
                std::hint::black_box(t.trim());
            },
        );
    }

    println!("\n--- with_length ---");
    let short = Ternary::parse("+-0+");
    for &target in &[10, 20, 40, 80] {
        bench(&format!("with_length({target})"), ITERS, || {
            std::hint::black_box(short.with_length(target));
        });
    }

    println!("\n--- concat ---");
    for &n in &[4, 10, 20, 40] {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        bench(&format!("concat ({n}+{n} digits)"), ITERS, || {
            std::hint::black_box(a.concat(&b));
        });
    }
}

// ---------------------------------------------------------------------------
#[test]
fn bench_ordering() {
    println!("\n============================================================");
    println!("  ORDERING");
    println!("============================================================");

    for &n in &[4, 10, 16, 24, 32, 40] {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        let c = a.clone();

        bench(&format!("cmp equal ({n}-digit)"), ITERS, || {
            std::hint::black_box(a.cmp(&c));
        });
        bench(&format!("cmp not-equal ({n}-digit)"), ITERS, || {
            std::hint::black_box(a.cmp(&b));
        });
    }
}

// ---------------------------------------------------------------------------
#[test]
fn bench_tryte() {
    println!("\n============================================================");
    println!("  TRYTE");
    println!("============================================================");

    bench("tryte() from str (public helper)", ITERS, || {
        std::hint::black_box(tryte(std::hint::black_box("+0-+0-")));
    });
    bench("Tryte::parse (FromStr trait, direct no-alloc)", ITERS, || {
        std::hint::black_box(std::hint::black_box("+0-+0-").parse::<Tryte>().unwrap());
    });

    let t = tryte("+0-+0-");

    bench("Tryte::to_string", ITERS, || {
        std::hint::black_box(t.to_string());
    });

    bench("Tryte::to_digits", ITERS, || {
        std::hint::black_box(std::hint::black_box(t).to_digits());
    });

    bench("Tryte::each(not)", ITERS, || {
        std::hint::black_box(std::hint::black_box(t).each(Digit::not));
    });

    // Arithmetic — previously went through Ternary heap alloc (3-5 allocs each)
    println!("\n--- Tryte arithmetic (direct i64 path) ---");
    let ta = Tryte::<6>::from("+-0+-0");
    let tb = Tryte::<6>::from("-0+-0+");
    bench("Tryte<6> add", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta) + std::hint::black_box(tb));
    });
    bench("Tryte<6> sub", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta) - std::hint::black_box(tb));
    });
    bench("Tryte<6> mul", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta) * std::hint::black_box(tb));
    });

    // Bitwise — previously went through Ternary heap alloc (3 allocs each)
    println!("\n--- Tryte bitwise (direct each_zip path) ---");
    bench("Tryte<6> & (bitand)", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta) & std::hint::black_box(tb));
    });
    bench("Tryte<6> | (bitor)", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta) | std::hint::black_box(tb));
    });
    bench("Tryte<6> ^ (bitxor)", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta) ^ std::hint::black_box(tb));
    });

    // New ternary-native ops
    println!("\n--- Tryte ternary-native ops ---");
    bench("Tryte<6> shu_up", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta).shu_up());
    });
    bench("Tryte<6> shu_down", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta).shu_down());
    });
    bench("Tryte<6> consensus", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta).consensus(std::hint::black_box(tb)));
    });
    bench("Tryte<6> accept_anything", ITERS, || {
        std::hint::black_box(std::hint::black_box(ta).accept_anything(std::hint::black_box(tb)));
    });
}

// ---------------------------------------------------------------------------
#[test]
fn bench_new_ops() {
    println!("\n============================================================");
    println!("  NEW TERNARY-NATIVE OPS (shu, consensus, accept_anything)");
    println!("============================================================");

    let sizes = [4, 10, 16, 24, 40];

    println!("\n--- shu_up / shu_down ---");
    for &n in &sizes {
        let a = make_ternary(n);
        bench(&format!("shu_up  ({n}-digit)"), ITERS, || {
            std::hint::black_box(a.shu_up());
        });
        bench(&format!("shu_down ({n}-digit)"), ITERS, || {
            std::hint::black_box(a.shu_down());
        });
    }

    println!("\n--- consensus ---");
    for &n in &sizes {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        bench(&format!("consensus ({n}-digit)"), ITERS, || {
            std::hint::black_box(a.consensus(&b));
        });
    }

    println!("\n--- accept_anything ---");
    for &n in &sizes {
        let a = make_ternary(n);
        let b = make_ternary_b(n);
        bench(&format!("accept_anything ({n}-digit)"), ITERS, || {
            std::hint::black_box(a.accept_anything(&b));
        });
    }
}

// ---------------------------------------------------------------------------
#[test]
fn bench_store() {
    println!("\n============================================================");
    println!("  DATA TERNARY / STORE");
    println!("============================================================");

    println!("\n--- TritsChunk ---");
    bench("TritsChunk::from_dec(100)", ITERS, || {
        std::hint::black_box(TritsChunk::from_dec(std::hint::black_box(100)));
    });
    let chunk = TritsChunk::from_dec(100);
    bench("TritsChunk::to_dec", ITERS, || {
        std::hint::black_box(chunk.to_dec());
    });

    println!("\n--- DataTernary ---");
    for &n in &[5, 10, 20, 40] {
        let t = make_ternary(n);
        bench(&format!("DT::from_ternary ({n}-digit)"), ITERS, || {
            std::hint::black_box(DataTernary::from_ternary(t.clone()));
        });
    }
    for &n in &[5, 10, 20, 40] {
        let dt = DataTernary::from_ternary(make_ternary(n));
        bench(&format!("DT::to_ternary ({n}-digit)"), ITERS, || {
            std::hint::black_box(dt.to_ternary());
        });
    }
    for &(label, val) in &[
        ("DT::from_dec(42)", 42i64),
        ("DT::from_dec(18_887_455)", 18_887_455),
        ("DT::from_dec(1_000_000_000)", 1_000_000_000),
    ] {
        bench(label, ITERS, || {
            std::hint::black_box(DataTernary::from_dec(std::hint::black_box(val)));
        });
    }
    for &n in &[5, 10, 20, 40] {
        let dt = DataTernary::from_ternary(make_ternary(n));
        bench(&format!("DT::to_dec ({n}-digit)"), ITERS, || {
            std::hint::black_box(dt.to_dec());
        });
    }
}

// ---------------------------------------------------------------------------
/// BCT vs Ter40 head-to-head: shows O(1) vs O(32) for trit-logical ops.
#[test]
fn bench_bct() {
    println!("\n============================================================");
    println!("  BCT vs TER40 — bitwise op comparison");
    println!("============================================================");
    println!("  (BctTer32 uses O(1) Jones bitmask formulas;");
    println!("   Ter40    uses O(1) IL u128 via CHUNK5 encode/decode)");

    // Build 32-trit test values that are comparable
    let a_ter = Ter40::from_dec(12345678);
    let b_ter = Ter40::from_dec(-9876543);
    let a_bct = BctTer32::from_dec(12345678);
    let b_bct = BctTer32::from_dec(-9876543);

    println!("\n--- AND (trit-wise min) ---");
    bench("Ter40    & (O(1) IL u128)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_ter) & std::hint::black_box(b_ter));
    });
    bench("BctTer32 & (32-trit, O(1) bitmask)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) & std::hint::black_box(b_bct));
    });

    println!("\n--- OR (trit-wise max) ---");
    bench("Ter40    | (O(1) IL u128)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_ter) | std::hint::black_box(b_ter));
    });
    bench("BctTer32 | (32-trit, O(1) bitmask)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) | std::hint::black_box(b_bct));
    });

    println!("\n--- XOR = -(a·b) ---");
    bench("Ter40    ^ (O(1) IL u128)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_ter) ^ std::hint::black_box(b_ter));
    });
    bench("BctTer32 ^ (32-trit, O(1) bitmask)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) ^ std::hint::black_box(b_bct));
    });

    println!("\n--- NOT (trit negation) ---");
    bench("Ter40    neg (direct i64 negate)", ITERS, || {
        std::hint::black_box(-std::hint::black_box(a_ter));
    });
    bench("BctTer32 ! (32-trit, O(1) swap)", ITERS, || {
        std::hint::black_box(!std::hint::black_box(a_bct));
    });

    println!("\n--- Consensus ---");
    bench("Ter40    consensus (O(1) IL u128)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_ter).consensus(std::hint::black_box(b_ter)));
    });
    bench("BctTer32 consensus (O(1) bitmask)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct).bct_consensus(std::hint::black_box(b_bct)));
    });

    println!("\n--- Accept-anything ---");
    bench("Ter40    accept_anything (O(1) IL u128)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_ter).accept_anything(std::hint::black_box(b_ter)));
    });
    bench("BctTer32 accept_anything (O(1) bitmask)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct).bct_accept_anything(std::hint::black_box(b_bct)));
    });

    println!("\n--- Shift (Ternary vs BctTer32 >> 4) ---");
    let a_tern32 = make_ternary(32);
    bench("Ternary  >> 4 (heap slice, 32-digit)", ITERS, || {
        std::hint::black_box(&a_tern32 >> 4usize);
    });
    bench("BctTer32 >> 4 (O(1) single shift)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) >> 4usize);
    });

    println!("\n--- Arithmetic (both via i64) ---");
    bench("Ter40    + (i64 add, O(1))", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_ter) + std::hint::black_box(b_ter));
    });
    bench("BctTer32 + (i64 add + decompose)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) + std::hint::black_box(b_bct));
    });
}

// ---------------------------------------------------------------------------
/// Jones libtern types: UTer9, UTer27, BTer9, BTer27, IlBctTer32.
/// Showcases the key tricks:
///   1. UTer9/27 uter_add  — O(1) Jones BCD-style parallel BCT addition
///   2. BTer9/27 il_and/or — O(1) interleaved-BCT trit-logical ops
///   3. IlBctTer32 ↔ BctTer32 — O(1) spread/compact conversion
///   4. BTer9/27 arithmetic  — goes through i32/i64 (baseline)
#[test]
fn bench_libtern() {
    println!("\n============================================================");
    println!("  JONES LIBTERN TYPES");
    println!("============================================================");

    // ---- UTer9: unsigned 9-trit (Jones uter9_t) -------------------------
    println!("\n--- UTer9 (9-trit unsigned BCT, range 0..19682) ---");

    let u9a = UTer9::from_dec(12345);
    let u9b = UTer9::from_dec(6789);

    bench("UTer9  uter_add       (Jones O(1) BCT trick)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a).uter_add(std::hint::black_box(u9b)));
    });
    bench("UTer9  +  (via uter_add)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a) + std::hint::black_box(u9b));
    });
    bench("UTer9  uter_add_carry (Jones O(1) + carry-out)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a).uter_add_carry(std::hint::black_box(u9b), UTer9::ZERO));
    });
    bench("UTer9  uter_sub  (Jones 3s-complement)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a).uter_sub(std::hint::black_box(u9b)));
    });
    bench("UTer9  trit_and  (O(1) il formula, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a).trit_and(std::hint::black_box(u9b)));
    });
    bench("UTer9  trit_or   (O(1) il formula, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a).trit_or(std::hint::black_box(u9b)));
    });
    bench("UTer9  >> 3      (O(1) shift, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a) >> 3usize);
    });
    bench("UTer9  from_dec  (encode loop, 9 trits)", ITERS, || {
        std::hint::black_box(UTer9::from_dec(std::hint::black_box(12345u32)));
    });
    bench("UTer9  to_dec    (decode loop, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a).to_dec());
    });

    // ---- UTer27: unsigned 27-trit (Jones uter27_t) ----------------------
    println!("\n--- UTer27 (27-trit unsigned BCT, range 0..7.6×10¹²) ---");

    let u27a = UTer27::from_dec(1_234_567_890);
    let u27b = UTer27::from_dec(987_654_321);

    bench("UTer27 uter_add       (Jones O(1) BCT trick)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a).uter_add(std::hint::black_box(u27b)));
    });
    bench("UTer27 +  (via uter_add)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a) + std::hint::black_box(u27b));
    });
    bench("UTer27 uter_add_carry (Jones O(1) + carry-out)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a).uter_add_carry(std::hint::black_box(u27b), UTer27::ZERO));
    });
    bench("UTer27 uter_sub  (Jones 3s-complement)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a).uter_sub(std::hint::black_box(u27b)));
    });
    bench("UTer27 trit_and  (O(1) il formula, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a).trit_and(std::hint::black_box(u27b)));
    });
    bench("UTer27 trit_or   (O(1) il formula, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a).trit_or(std::hint::black_box(u27b)));
    });
    bench("UTer27 >> 9      (O(1) shift, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a) >> 9usize);
    });
    bench("UTer27 from_dec  (encode loop, 27 trits)", ITERS, || {
        std::hint::black_box(UTer27::from_dec(std::hint::black_box(1_234_567_890u64)));
    });
    bench("UTer27 to_dec    (decode loop, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a).to_dec());
    });

    // ---- BTer9: balanced 9-trit (Jones bter9_t) -------------------------
    println!("\n--- BTer9 (9-trit balanced BCT, range ±9841) ---");

    let b9a = BTer9::from_dec(4000);
    let b9b = BTer9::from_dec(-2000);

    bench("BTer9  il_and    (O(1) interleaved, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b9a).il_and(std::hint::black_box(b9b)));
    });
    bench("BTer9  il_or     (O(1) interleaved, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b9a).il_or(std::hint::black_box(b9b)));
    });
    bench("BTer9  il_xor    (O(1) interleaved, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b9a).il_xor(std::hint::black_box(b9b)));
    });
    bench("BTer9  il_neg    (O(1) bit-swap, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b9a).il_neg());
    });
    bench("BTer9  <<  3     (O(1) shift, fill Zero)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b9a) << 3usize);
    });
    bench("BTer9  +   (i32 add + encode, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b9a) + std::hint::black_box(b9b));
    });
    bench("BTer9  from_dec  (encode loop, 9 trits)", ITERS, || {
        std::hint::black_box(BTer9::from_dec(std::hint::black_box(4000i32)));
    });
    bench("BTer9  to_dec    (decode loop, 9 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b9a).to_dec());
    });

    // ---- BTer27: balanced 27-trit (Jones bter27_t) ----------------------
    println!("\n--- BTer27 (27-trit balanced BCT, range ±3.8×10¹²) ---");

    let b27a = BTer27::from_dec(1_234_567_890);
    let b27b = BTer27::from_dec(-987_654_321);

    bench("BTer27 il_and    (O(1) interleaved, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b27a).il_and(std::hint::black_box(b27b)));
    });
    bench("BTer27 il_or     (O(1) interleaved, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b27a).il_or(std::hint::black_box(b27b)));
    });
    bench("BTer27 il_xor    (O(1) interleaved, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b27a).il_xor(std::hint::black_box(b27b)));
    });
    bench("BTer27 il_neg    (O(1) bit-swap, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b27a).il_neg());
    });
    bench("BTer27 <<  9     (O(1) shift, fill Zero)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b27a) << 9usize);
    });
    bench("BTer27 +   (i64 add + encode, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b27a) + std::hint::black_box(b27b));
    });
    bench("BTer27 from_dec  (encode loop, 27 trits)", ITERS, || {
        std::hint::black_box(BTer27::from_dec(std::hint::black_box(1_234_567_890i64)));
    });
    bench("BTer27 to_dec    (decode loop, 27 trits)", ITERS, || {
        std::hint::black_box(std::hint::black_box(b27a).to_dec());
    });

    // ---- IlBctTer32 ↔ BctTer32 spread/compact conversion ----------------
    println!("\n--- IlBctTer32 ↔ BctTer32 (O(1) spread/compact) ---");

    let il = IlBctTer32::from_dec(12_345_678);
    let bct = BctTer32::from_dec(12_345_678);

    bench("IlBctTer32 → BctTer32  (compact, O(1))", ITERS, || {
        std::hint::black_box(std::hint::black_box(il).to_bct());
    });
    bench("BctTer32   → IlBctTer32 (spread,  O(1))", ITERS, || {
        std::hint::black_box(IlBctTer32::from_bct(std::hint::black_box(bct)));
    });
    bench("IlBctTer32 from_dec (7-group UTER5_LUT, 32-trit)", ITERS, || {
        std::hint::black_box(IlBctTer32::from_dec(std::hint::black_box(12_345_678i64)));
    });
    bench("BctTer32   from_dec (via IlBctTer32 + PEXT)", ITERS, || {
        std::hint::black_box(BctTer32::from_dec(std::hint::black_box(12_345_678i64)));
    });
    bench("IlBctTer32 il_neg (Jones MASK_H−self, 32-trit)", ITERS, || {
        std::hint::black_box(std::hint::black_box(il).il_neg());
    });
    bench("IlBctTer32 to_dec (Jones parallel reduction)", ITERS, || {
        std::hint::black_box(std::hint::black_box(il).to_dec());
    });
    bench("BctTer32   to_dec (via IlBctTer32 Jones reduction)", ITERS, || {
        std::hint::black_box(std::hint::black_box(bct).to_dec());
    });

    // ---- Head-to-head: IlBctTer32 vs BctTer32 vs Ter40 -----------------
    println!("\n--- Head-to-head: IlBctTer32 vs BctTer32 vs Ter40 (32-trit AND) ---");

    let a_ter = Ter40::from_dec(12_345_678);
    let b_ter = Ter40::from_dec(-9_876_543);
    let a_bct = BctTer32::from_dec(12_345_678);
    let b_bct = BctTer32::from_dec(-9_876_543);
    let a_il  = IlBctTer32::from_dec(12_345_678);
    let b_il  = IlBctTer32::from_dec(-9_876_543);

    bench("Ter40      & (O(1) IL u128)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_ter) & std::hint::black_box(b_ter));
    });
    bench("BctTer32   & (O(1) split bitmask)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) & std::hint::black_box(b_bct));
    });
    bench("IlBctTer32 & (O(1) interleaved)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_il) & std::hint::black_box(b_il));
    });

    println!("\n--- Head-to-head: arithmetic (add) ---");
    bench("Ter40      + (direct i64, O(1))", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_ter) + std::hint::black_box(b_ter));
    });
    bench("BctTer32   + (i64 add + decompose)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) + std::hint::black_box(b_bct));
    });
    bench("IlBctTer32 + (Jones biased-uter O(1), 32-trit)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_il) + std::hint::black_box(b_il));
    });
    bench("UTer9      + (Jones uter_add O(1), 9-trit)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u9a) + std::hint::black_box(u9b));
    });
    bench("UTer27     + (Jones uter_add O(1), 27-trit)", ITERS, || {
        std::hint::black_box(std::hint::black_box(u27a) + std::hint::black_box(u27b));
    });

    // ---- Trick: use il_neg instead of from_dec(-x) ----------------------
    println!("\n--- Trick: O(1) il_neg vs encode-negate-encode ---");
    bench("BTer27 -a  (il_neg, O(1) bit-swap)", ITERS, || {
        std::hint::black_box(-std::hint::black_box(b27a));
    });
    bench("BTer27 from_dec(-x) (decode, negate, encode)", ITERS, || {
        std::hint::black_box(BTer27::from_dec(-std::hint::black_box(b27a).to_dec()));
    });

    // ---- Trick: chained logical ops stay O(1) per op --------------------
    println!("\n--- Trick: chained trit-logical ops (each O(1)) ---");
    bench("BTer27 (a & b) | c  (2 ops, all O(1))", ITERS, || {
        let c = std::hint::black_box(BTer27::from_dec(999_999));
        let ab = std::hint::black_box(b27a).il_and(std::hint::black_box(b27b));
        std::hint::black_box(ab.il_or(c));
    });
    bench("Ter40  (a & b) | c  (2 ops, each O(1) IL u128)", ITERS, || {
        let c = std::hint::black_box(Ter40::from_dec(999_999));
        let ab = std::hint::black_box(a_ter) & std::hint::black_box(b_ter);
        std::hint::black_box(ab | c);
    });
}

// ---------------------------------------------------------------------------
/// IlTer40 vs Ter40 vs BctTer32: storage trade-off benchmark.
///
/// IlTer40 stores IL u128 → logical ops O(1), arithmetic pays encode/decode.
/// Ter40   stores i64     → arithmetic O(1), logical ops pay encode/decode.
#[test]
fn bench_ilter40() {
    println!("\n============================================================");
    println!("  IlTer40 vs Ter40 vs BctTer32 — storage trade-off");
    println!("============================================================");
    println!("  IlTer40 = IL u128 storage: O(1) logical, ~2 ns add/sub (Jones biased-uter)");
    println!("  Ter40   = i64    storage: O(1) arithmetic, ~15 ns logical");
    println!("  BctTer32= split BCT u32s: O(1) both (32-trit range only)");

    let a_t40  = Ter40::from_dec(12_345_678);
    let b_t40  = Ter40::from_dec(-9_876_543);
    let a_il40 = IlTer40::from_dec(12_345_678);
    let b_il40 = IlTer40::from_dec(-9_876_543);
    let a_bct  = BctTer32::from_dec(12_345_678);
    let b_bct  = BctTer32::from_dec(-9_876_543);

    println!("\n--- AND (trit-wise min) ---");
    bench("IlTer40  & (O(1) pure bitwise, IL u128)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_il40) & std::hint::black_box(b_il40));
    });
    bench("Ter40    & (O(1) CHUNK5+Estrin encode/decode)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_t40) & std::hint::black_box(b_t40));
    });
    bench("BctTer32 & (O(1) pure bitwise, split u32)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) & std::hint::black_box(b_bct));
    });

    println!("\n--- OR ---");
    bench("IlTer40  | (O(1) pure bitwise)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_il40) | std::hint::black_box(b_il40));
    });
    bench("Ter40    | (O(1) CHUNK5+Estrin)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_t40) | std::hint::black_box(b_t40));
    });
    bench("BctTer32 | (O(1) pure bitwise)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) | std::hint::black_box(b_bct));
    });

    println!("\n--- XOR ---");
    bench("IlTer40  ^ (O(1) pure bitwise)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_il40) ^ std::hint::black_box(b_il40));
    });
    bench("Ter40    ^ (O(1) CHUNK5+Estrin)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_t40) ^ std::hint::black_box(b_t40));
    });

    println!("\n--- Chained: (a & b) | c ---");
    bench("IlTer40  (a & b) | c  (2 pure O(1) ops)", ITERS, || {
        let c = std::hint::black_box(IlTer40::from_dec(999_999));
        let ab = std::hint::black_box(a_il40) & std::hint::black_box(b_il40);
        std::hint::black_box(ab | c);
    });
    bench("Ter40    (a & b) | c  (2 encode/decode ops)", ITERS, || {
        let c = std::hint::black_box(Ter40::from_dec(999_999));
        let ab = std::hint::black_box(a_t40) & std::hint::black_box(b_t40);
        std::hint::black_box(ab | c);
    });
    bench("BTer27   (a & b) | c  (O(1), 27-trit range)", ITERS, || {
        let a = BTer27::from_dec(12_345_678);
        let b = BTer27::from_dec(-9_876_543);
        let c = std::hint::black_box(BTer27::from_dec(999_999));
        let ab = std::hint::black_box(a) & std::hint::black_box(b);
        std::hint::black_box(ab | c);
    });

    println!("\n--- Arithmetic: add / sub ---");
    bench("IlTer40  + (Jones biased-uter, O(1))", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_il40) + std::hint::black_box(b_il40));
    });
    bench("IlTer40  - (neg + Jones biased-uter, O(1))", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_il40) - std::hint::black_box(b_il40));
    });
    bench("Ter40    + (direct i64, O(1))", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_t40) + std::hint::black_box(b_t40));
    });
    bench("BctTer32 + (via IlBctTer32 O(1) PDEP/PEXT)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct) + std::hint::black_box(b_bct));
    });

    println!("\n--- Negation ---");
    bench("IlTer40  neg (O(1) pure bitwise)", ITERS, || {
        std::hint::black_box(-std::hint::black_box(a_il40));
    });
    bench("Ter40    neg (O(1) i64 negate)", ITERS, || {
        std::hint::black_box(-std::hint::black_box(a_t40));
    });

    println!("\n--- Consensus ---");
    bench("IlTer40  consensus (O(1) pure bitwise)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_il40).il_consensus(std::hint::black_box(b_il40)));
    });
    bench("Ter40    consensus (O(1) CHUNK5+Estrin)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_t40).consensus(std::hint::black_box(b_t40)));
    });
    bench("BctTer32 consensus (O(1) pure bitwise)", ITERS, || {
        std::hint::black_box(std::hint::black_box(a_bct).bct_consensus(std::hint::black_box(b_bct)));
    });
}

// ---------------------------------------------------------------------------
#[test]
fn bench_terscii() {
    println!("\n============================================================");
    println!("  TERSCII — ternary character encoding");
    println!("============================================================");

    let h = terscii::encode('H').unwrap();
    let space = terscii::encode(' ').unwrap();

    println!("\n--- encode (char → Ternary) ---");
    bench("terscii::encode('H')  [end of table]", ITERS, || {
        std::hint::black_box(terscii::encode(std::hint::black_box('H')));
    });
    bench("terscii::encode(' ')  [near start]", ITERS, || {
        std::hint::black_box(terscii::encode(std::hint::black_box(' ')));
    });

    println!("\n--- decode (Ternary → char) ---");
    bench("terscii::decode(75='H')", ITERS, || {
        std::hint::black_box(terscii::decode(std::hint::black_box(&h)));
    });
    bench("terscii::decode(1=' ')", ITERS, || {
        std::hint::black_box(terscii::decode(std::hint::black_box(&space)));
    });

    println!("\n--- encode_code (TersciiCode BCT4, O(1) LUT) ---");
    bench("terscii::encode_code('H')  [end of table]", ITERS, || {
        std::hint::black_box(terscii::encode_code(std::hint::black_box('H')));
    });
    bench("terscii::encode_code(' ')  [near start]", ITERS, || {
        std::hint::black_box(terscii::encode_code(std::hint::black_box(' ')));
    });

    println!("\n--- decode_code (TersciiCode BCT4, O(1) LUT) ---");
    let ch = terscii::encode_code('H').unwrap();
    let cs = terscii::encode_code(' ').unwrap();
    bench("terscii::decode_code(75='H')", ITERS, || {
        std::hint::black_box(terscii::decode_code(std::hint::black_box(ch)));
    });
    bench("terscii::decode_code(1=' ')", ITERS, || {
        std::hint::black_box(terscii::decode_code(std::hint::black_box(cs)));
    });

    println!("\n--- roundtrip ---");
    bench("Ternary      encode+decode single char", ITERS, || {
        let t = terscii::encode(std::hint::black_box('H')).unwrap();
        std::hint::black_box(terscii::decode(&t));
    });
    bench("TersciiCode  encode+decode single char", ITERS, || {
        let t = terscii::encode_code(std::hint::black_box('H')).unwrap();
        std::hint::black_box(terscii::decode_code(std::hint::black_box(t)));
    });
    bench("TersciiCode  encode+decode \"Hello, World!\" (2 allocs)", ITERS / 10, || {
        let encoded: Vec<terscii::TersciiCode> = "Hello, World!".chars()
            .map(|c| terscii::encode_code(c).unwrap())
            .collect();
        let decoded: String = encoded.iter()
            .map(|&c| terscii::decode_code(c).unwrap())
            .collect();
        std::hint::black_box(decoded);
    });
    bench("TersciiCode  encode_str+decode_codes    (2 allocs, direct write)", ITERS / 10, || {
        let encoded = terscii::encode_str(std::hint::black_box("Hello, World!")).unwrap();
        let decoded = terscii::decode_codes(std::hint::black_box(&encoded)).unwrap();
        std::hint::black_box(decoded);
    });
    bench("TersciiCode  encode+decode no-alloc (stack [u8;13])", ITERS, || {
        let src = std::hint::black_box(b"Hello, World!");
        let mut buf = [0u8; 13];
        for i in 0..13 {
            let code = terscii::encode_code(src[i] as char).unwrap();
            buf[i] = terscii::decode_code(code).unwrap() as u8;
        }
        std::hint::black_box(buf);
    });

    println!("\n--- TersciiCode helpers ---");
    bench("TersciiCode::to_ternary('H')  (BCT4 → Ternary)", ITERS, || {
        std::hint::black_box(std::hint::black_box(ch).to_ternary());
    });
    {
        let hello = terscii::encode_str("Hello, World!").unwrap();
        bench("terscii::unbalanced_str (13 codes → String)", ITERS / 10, || {
            std::hint::black_box(terscii::unbalanced_str(std::hint::black_box(&hello)));
        });
        bench("terscii::balanced_str   (13 codes → String)", ITERS / 10, || {
            std::hint::black_box(terscii::balanced_str(std::hint::black_box(&hello)));
        });
    }
}

// ---------------------------------------------------------------------------
/// Getrandom: syscall-backed vs SplitMix64 PRNG random ternary values.
#[test]
fn bench_getrandom() {
    use balanced_ternary::getrandom::*;

    println!("\n============================================================");
    println!("  GETRANDOM — syscall vs SplitMix64 PRNG");
    println!("============================================================");
    println!("  Syscall variants: ~180-200 ns each (one getrandom(2) per call)");
    println!("  SplitMix64 variants: ~2-4 ns (seeded once, pure arithmetic)");

    println!("\n--- Syscall-backed (one kernel call per value) ---");
    bench("rand_digit()         (1 syscall, rejection loop)", ITERS / 100, || {
        std::hint::black_box(rand_digit());
    });
    bench("rand_bter9()         (1 syscall, rejection loop)", ITERS / 100, || {
        std::hint::black_box(rand_bter9());
    });
    bench("rand_bter27()        (1 syscall, rejection loop)", ITERS / 100, || {
        std::hint::black_box(rand_bter27());
    });
    bench("rand_uter27()        (1 syscall, rejection loop)", ITERS / 100, || {
        std::hint::black_box(rand_uter27());
    });

    println!("\n--- SplitMix64 PRNG (seeded once, no syscall per call) ---");
    let mut rng = SplitMix64::new();
    bench("SplitMix64::next_u64 (pure arithmetic, no syscall)", ITERS, || {
        std::hint::black_box(rng.next_u64());
    });
    let mut rng = SplitMix64::new();
    bench("SplitMix64::rand_bter9()  (~0 ns PRNG + from_dec)", ITERS, || {
        std::hint::black_box(rng.rand_bter9());
    });
    let mut rng = SplitMix64::new();
    bench("SplitMix64::rand_bter27() (~0 ns PRNG + from_dec)", ITERS, || {
        std::hint::black_box(rng.rand_bter27());
    });
    let mut rng = SplitMix64::new();
    bench("SplitMix64::rand_uter27() (~0 ns PRNG + from_dec)", ITERS, || {
        std::hint::black_box(rng.rand_uter27());
    });
}

#[test]
#[cfg(feature = "ternary-matrix")]
fn bench_matrix() {
    use balanced_ternary::matrix::{TernaryMatrix, TernaryVec};

    println!("\n============================================================");
    println!(" TernaryMatrix / TernaryVec  (BctTer64, AND+POPCNT dot)");
    println!("============================================================\n");

    // -- word-level primitive -------------------------------------------------
    println!("--- BctTer64 word dot product ---");
    let a64 = BctTer64::new(0x5555_5555_5555_5555, 0x2222_2222_2222_2222);
    let b64 = BctTer64::new(0x3333_3333_3333_3333, 0x4444_4444_4444_4444);
    bench("BctTer64 bct_dot_word (64 trits, 4AND+2OR+2POPCNT)", ITERS, || {
        std::hint::black_box(a64.bct_dot_word(std::hint::black_box(b64)));
    });

    // -- TernaryVec dot -------------------------------------------------------
    println!("\n--- TernaryVec dot product ---");

    let make_vec = |len: usize, stride: i8| -> TernaryVec {
        let mut v = TernaryVec::zeros(len);
        for i in 0..len {
            v.set(i, ((i as i8 % 3) - 1) * stride);
        }
        v
    };

    let v64a = make_vec(64, 1);
    let v64b = make_vec(64, -1);
    bench("TernaryVec dot  64-trit (1 word)", ITERS, || {
        std::hint::black_box(v64a.dot(&v64b));
    });

    let v128a = make_vec(128, 1);
    let v128b = make_vec(128, -1);
    bench("TernaryVec dot 128-trit (2 words)", ITERS, || {
        std::hint::black_box(v128a.dot(&v128b));
    });

    let v768a = make_vec(768, 1);
    let v768b = make_vec(768, -1);
    bench("TernaryVec dot 768-trit (12 words, BitNet hidden dim)", ITERS, || {
        std::hint::black_box(v768a.dot(&v768b));
    });

    // -- TernaryMatrix matvec -------------------------------------------------
    println!("\n--- TernaryMatrix matvec ---");

    let make_mat = |rows: usize, cols: usize| -> TernaryMatrix {
        let mut m = TernaryMatrix::zeros(rows, cols);
        for r in 0..rows {
            for c in 0..cols {
                m.set(r, c, ((r + c) % 3) as i8 - 1);
            }
        }
        m
    };

    let m64  = make_mat(64, 64);
    let x64  = make_vec(64, 1);
    bench("TernaryMatrix matvec   64×64  (64 rows × 1 word)", ITERS / 10, || {
        std::hint::black_box(m64.matvec(&x64));
    });

    let m128 = make_mat(128, 128);
    let x128 = make_vec(128, 1);
    bench("TernaryMatrix matvec  128×128 (128 rows × 2 words)", ITERS / 100, || {
        std::hint::black_box(m128.matvec(&x128));
    });
    let m768 = make_mat(768, 768);
    let x768 = make_vec(768, 1);
    bench("TernaryMatrix matvec  768×768 (768 rows × 12 words)", ITERS / 1000, || {
        std::hint::black_box(m768.matvec(&x768));
    });
}
