use balanced_ternary::{DataTernary, IlTer40, Ter40, Ternary, Tryte};
use std::mem::size_of;

fn main() {
    let n = 1_000_000i64;

    // --- Ternary: heap-allocated, arbitrary precision --------------------
    // Stores one Digit (1 byte) per trit. Grows as large as needed.
    let t = Ternary::from_dec(n);
    println!("Ternary       (heap, arbitrary size)");
    println!("  {}  =  {}  ({} trits, {} bytes on heap)", n, t, t.log(), t.log());

    // Can hold numbers that don't fit in any fixed type
    let huge = Ternary::from_dec(i64::MAX);
    println!("  i64::MAX  =  {}  ({} trits)", huge, huge.log());

    // --- Tryte<6>: stack-allocated, Copy, fixed 6 trits ------------------
    // Zero heap allocation. Can be copied like an integer.
    let a = Tryte::<6>::from_i64(42);
    let b = a; // Copy — no clone() needed
    println!("\nTryte<6>      (stack, Copy, {} bytes)", size_of::<Tryte<6>>());
    println!("  a = {}  b = {}  (b is a copy of a, both still valid)", a, b);
    println!("  a + b = {}", (a + b).to_i64());

    // --- DataTernary: 5 trits packed per byte ----------------------------
    // Same value as Ternary but uses ceil(trits/5) bytes instead of trits bytes.
    let d = DataTernary::from_dec(n);
    let trits = t.log();
    let bytes = (trits + 4) / 5;
    println!("\nDataTernary   (5 trits/byte, compact storage)");
    println!("  {}  =  {}", n, d);
    println!("  Ternary:     {} trits = {} bytes", trits, trits);
    println!("  DataTernary: {} trits = {} bytes  ({:.1}× denser)", trits, bytes, trits as f64 / bytes as f64);

    // --- Ter40: i64 wrapper, fastest arithmetic --------------------------
    // Arithmetic is just native i64 ops. No trit decomposition at all.
    let x = Ter40::from_dec(364);
    let y = Ter40::from_dec(91);
    println!("\nTer40         (i64 wrapper, {} bytes, fastest arithmetic)", size_of::<Ter40>());
    println!("  {} + {} = {}", x.to_dec(), y.to_dec(), (x + y).to_dec());
    println!("  {} * {} = {}", x.to_dec(), y.to_dec(), (x * y).to_dec());

    // --- IlTer40: interleaved u128, O(1) logic on all 40 trits at once --
    // Each trit is 2 bits in a u128. AND/OR/XOR/neg operate on all 40 trits
    // in a single instruction — no loop, no decomposition.
    let p = IlTer40::from_dec(364); // ++++++
    let q = IlTer40::from_dec(91);  // +0+0+
    println!("\nIlTer40       (u128 interleaved, {} bytes, O(1) trit logic)", size_of::<IlTer40>());
    println!("  p = {} ({})", p.to_dec(), Ternary::from_dec(p.to_dec()));
    println!("  q = {} ({})", q.to_dec(), Ternary::from_dec(q.to_dec()));
    println!("  p & q = {} ({})  ← all 40 trits in one instruction", (p & q).to_dec(), Ternary::from_dec((p & q).to_dec()));
    println!("  p | q = {} ({})  ← all 40 trits in one instruction", (p | q).to_dec(), Ternary::from_dec((p | q).to_dec()));
    println!("    -p  = {} ({})  ← flip all 40 trits in one instruction", (-p).to_dec(), Ternary::from_dec((-p).to_dec()));
}
