use balanced_ternary::{terscii, Digit::{Neg as N, Pos as P, Zero as Z}, Ternary};

const MESSAGE: &str = "Hello, World!";

fn print_row(label: &str, items: &[impl std::fmt::Display]) {
    print!("  {label}");
    for item in items { print!("  {item}"); }
    println!();
}

fn main() {
    // TERSCII: encode a string to balanced ternary trytes, decode back
    let encoded = terscii::encode_str(MESSAGE).unwrap();
    println!("\nstring operations:");
    println!("  encoded:  {encoded}");
    println!("  decoded:  {}", terscii::decode_str(&encoded).unwrap());

    // Negation is O(1) — just flip every trit (+ ↔ -, 0 stays 0)
    let neg: Vec<Ternary> = [364i64, 91].map(Ternary::from_dec).into();
    let neg_d: Vec<Ternary> = neg.iter().map(|t| -t).collect();
    println!("\nnegation is free:");
    print_row("ternary:", &neg);
    print_row("negated:", &neg_d);
    print_row("decoded:", &neg.iter().map(|t| format!("{}  →  {}", t.to_dec(), -t.to_dec())).collect::<Vec<_>>());

    // Three-valued logic: N = false, Z = unknown, P = true
    println!("\nthree-valued logic (- = false, 0 = unknown, + = true):");
    for (a, b) in [(N,N),(N,Z),(N,P),(Z,Z),(Z,P),(P,P)] {
        println!("  {} & {} = {}    {} | {} = {}",
            a.to_char(), b.to_char(), (a & b).to_char(),
            a.to_char(), b.to_char(), (a | b).to_char());
    }

    // Fibonacci in balanced ternary
    let mut fibs: Vec<Ternary> = vec![Ternary::from_dec(0), Ternary::from_dec(1)];
    for i in 2..10 { let next = &fibs[i - 2] + &fibs[i - 1]; fibs.push(next); }
    let decoded: Vec<i64> = fibs.iter().map(|t| t.to_dec()).collect();
    println!("\nfibonacci:");
    print_row("ternary:", &fibs);
    print_row("decoded:", &decoded);
}
