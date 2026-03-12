fn main() {
    divan::main();
}

use balanced_ternary::concepts::DigitOperate;
use balanced_ternary::terscii;
use balanced_ternary::*;
use std::ops::{BitAnd, Not};

// ---------------------------------------------------------------------------
// Helpers

fn make_ternary(n: usize) -> Ternary {
    let s: String = "+-0".chars().cycle().take(n).collect();
    Ternary::parse(&s)
}

fn make_ternary_b(n: usize) -> Ternary {
    let s: String = "-0+".chars().cycle().take(n).collect();
    Ternary::parse(&s)
}

// ---------------------------------------------------------------------------
// Conversions

mod conversions {
    use super::*;

    #[divan::bench(args = [0i64, 42, 18_887_455, 1_000_000_000, i64::MAX / 4, i64::MAX / 2])]
    fn from_dec(n: i64) -> Ternary {
        Ternary::from_dec(divan::black_box(n))
    }

    #[divan::bench(args = [4usize, 10, 20, 30, 40])]
    fn to_dec(b: divan::Bencher, n: usize) {
        let t = make_ternary(n);
        b.bench(|| t.to_dec())
    }

    #[divan::bench(args = [42i64, 18_887_455, 1_000_000_000, i64::MAX / 2])]
    fn roundtrip(n: i64) -> i64 {
        Ternary::from_dec(divan::black_box(n)).to_dec()
    }
}

// ---------------------------------------------------------------------------
// Arithmetic

mod arithmetic {
    use super::*;

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn add(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| &a + &x)
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn sub(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| &a - &x)
    }

    #[divan::bench(args = [4usize, 10, 16, 20])]
    fn mul(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| &a * &x)
    }

    #[divan::bench(args = [4usize, 10, 16, 24])]
    fn div(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(4.max(n / 3));
        b.bench(|| &a / &x)
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn neg(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        b.bench(|| -&a)
    }
}

// ---------------------------------------------------------------------------
// Bitwise

mod bitwise {
    use super::*;

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn and(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| &a & &x)
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn or(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| &a | &x)
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn xor(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| &a ^ &x)
    }
}

// ---------------------------------------------------------------------------
// Shift

mod shift {
    use super::*;

    #[divan::bench(args = [1usize, 5, 10, 20])]
    fn shl_10(b: divan::Bencher, s: usize) {
        let a = make_ternary(10);
        b.bench(|| &a << s)
    }

    #[divan::bench(args = [1usize, 5, 10, 20])]
    fn shr_10(b: divan::Bencher, s: usize) {
        let a = make_ternary(10);
        b.bench(|| &a >> s)
    }

    #[divan::bench(args = [1usize, 5, 10, 20])]
    fn shl_40(b: divan::Bencher, s: usize) {
        let a = make_ternary(40);
        b.bench(|| &a << s)
    }

    #[divan::bench(args = [1usize, 5, 10, 20])]
    fn shr_40(b: divan::Bencher, s: usize) {
        let a = make_ternary(40);
        b.bench(|| &a >> s)
    }
}

// ---------------------------------------------------------------------------
// Digit operations

mod digit_ops {
    use super::*;

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn each_not(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        b.bench(|| a.each(Digit::not))
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn each_zip_bitand(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench_local(move || a.each_zip(Digit::bitand, x.clone()))
    }
}

// ---------------------------------------------------------------------------
// Utilities

mod utilities {
    use super::*;

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn parse(b: divan::Bencher, n: usize) {
        let s: String = "+-0".chars().cycle().take(n).collect();
        b.bench(|| Ternary::parse(divan::black_box(&s)))
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn to_string(b: divan::Bencher, n: usize) {
        let t = make_ternary(n);
        b.bench(|| t.to_string())
    }

    #[divan::bench(args = [4usize, 10, 20, 30])]
    fn trim(b: divan::Bencher, zeros: usize) {
        let s = format!("{}{}", "0".repeat(zeros), "+-0+-0+-0+");
        let t = Ternary::parse(&s);
        b.bench(|| t.trim())
    }

    #[divan::bench(args = [10usize, 20, 40, 80])]
    fn with_length(b: divan::Bencher, target: usize) {
        let short = Ternary::parse("+-0+");
        b.bench(|| short.with_length(target))
    }

    #[divan::bench(args = [4usize, 10, 20, 40])]
    fn concat(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| a.concat(&x))
    }
}

// ---------------------------------------------------------------------------
// Ordering

mod ordering {
    use super::*;

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn cmp_equal(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let c = a.clone();
        b.bench(|| a.cmp(&c))
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 32, 40])]
    fn cmp_not_equal(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| a.cmp(&x))
    }
}

// ---------------------------------------------------------------------------
// Tryte

mod tryte {
    use super::*;

    #[divan::bench]
    fn from_str() -> Tryte {
        tryte(divan::black_box("+0-+0-"))
    }

    #[divan::bench]
    fn to_string(b: divan::Bencher) {
        let t = tryte("+0-+0-");
        b.bench(|| t.to_string())
    }

    #[divan::bench]
    fn to_digits(b: divan::Bencher) {
        let t = tryte("+0-+0-");
        b.bench(|| divan::black_box(t).to_digits())
    }

    #[divan::bench]
    fn each_not(b: divan::Bencher) {
        let t = tryte("+0-+0-");
        b.bench(|| divan::black_box(t).each(Digit::not))
    }

    // Arithmetic (i64 path)
    #[divan::bench]
    fn add(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        let x = Tryte::<6>::from("-0+-0+");
        b.bench(|| divan::black_box(a) + divan::black_box(x))
    }

    #[divan::bench]
    fn sub(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        let x = Tryte::<6>::from("-0+-0+");
        b.bench(|| divan::black_box(a) - divan::black_box(x))
    }

    #[divan::bench]
    fn mul(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        let x = Tryte::<6>::from("-0+-0+");
        b.bench(|| divan::black_box(a) * divan::black_box(x))
    }

    // Bitwise (each_zip path)
    #[divan::bench]
    fn bitand(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        let x = Tryte::<6>::from("-0+-0+");
        b.bench(|| divan::black_box(a) & divan::black_box(x))
    }

    #[divan::bench]
    fn bitor(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        let x = Tryte::<6>::from("-0+-0+");
        b.bench(|| divan::black_box(a) | divan::black_box(x))
    }

    #[divan::bench]
    fn bitxor(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        let x = Tryte::<6>::from("-0+-0+");
        b.bench(|| divan::black_box(a) ^ divan::black_box(x))
    }

    // Ternary-native ops
    #[divan::bench]
    fn shu_up(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        b.bench(|| divan::black_box(a).shu_up())
    }

    #[divan::bench]
    fn shu_down(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        b.bench(|| divan::black_box(a).shu_down())
    }

    #[divan::bench]
    fn consensus(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        let x = Tryte::<6>::from("-0+-0+");
        b.bench(|| divan::black_box(a).consensus(divan::black_box(x)))
    }

    #[divan::bench]
    fn accept_anything(b: divan::Bencher) {
        let a = Tryte::<6>::from("+-0+-0");
        let x = Tryte::<6>::from("-0+-0+");
        b.bench(|| divan::black_box(a).accept_anything(divan::black_box(x)))
    }
}

// ---------------------------------------------------------------------------
// Ternary-native ops on heap Ternary

mod ternary_ops {
    use super::*;

    #[divan::bench(args = [4usize, 10, 16, 24, 40])]
    fn shu_up(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        b.bench(|| a.shu_up())
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 40])]
    fn shu_down(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        b.bench(|| a.shu_down())
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 40])]
    fn consensus(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| a.consensus(&x))
    }

    #[divan::bench(args = [4usize, 10, 16, 24, 40])]
    fn accept_anything(b: divan::Bencher, n: usize) {
        let a = make_ternary(n);
        let x = make_ternary_b(n);
        b.bench(|| a.accept_anything(&x))
    }
}

// ---------------------------------------------------------------------------
// Store / DataTernary

mod store {
    use super::*;

    #[divan::bench]
    fn trits_chunk_from_dec() -> TritsChunk {
        TritsChunk::from_dec(divan::black_box(100))
    }

    #[divan::bench]
    fn trits_chunk_to_dec(b: divan::Bencher) {
        let c = TritsChunk::from_dec(100);
        b.bench(|| c.to_dec())
    }

    #[divan::bench(args = [5usize, 10, 20, 40])]
    fn dt_from_ternary(b: divan::Bencher, n: usize) {
        let t = make_ternary(n);
        b.bench_local(move || DataTernary::from_ternary(t.clone()))
    }

    #[divan::bench(args = [5usize, 10, 20, 40])]
    fn dt_to_ternary(b: divan::Bencher, n: usize) {
        let dt = DataTernary::from_ternary(make_ternary(n));
        b.bench(|| dt.to_ternary())
    }

    #[divan::bench(args = [42i64, 18_887_455, 1_000_000_000])]
    fn dt_from_dec(n: i64) -> DataTernary {
        DataTernary::from_dec(divan::black_box(n))
    }

    #[divan::bench(args = [5usize, 10, 20, 40])]
    fn dt_to_dec(b: divan::Bencher, n: usize) {
        let dt = DataTernary::from_ternary(make_ternary(n));
        b.bench(|| dt.to_dec())
    }
}

// ---------------------------------------------------------------------------
// Fixed-point types: Ter40, IlTer40, BctTer32

mod fixed_point {
    use super::*;

    // --- Ter40 ---
    #[divan::bench]
    fn ter40_and(b: divan::Bencher) {
        let a = Ter40::from_dec(12_345_678);
        let x = Ter40::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) & divan::black_box(x))
    }

    #[divan::bench]
    fn ter40_or(b: divan::Bencher) {
        let a = Ter40::from_dec(12_345_678);
        let x = Ter40::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) | divan::black_box(x))
    }

    #[divan::bench]
    fn ter40_xor(b: divan::Bencher) {
        let a = Ter40::from_dec(12_345_678);
        let x = Ter40::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) ^ divan::black_box(x))
    }

    #[divan::bench]
    fn ter40_add(b: divan::Bencher) {
        let a = Ter40::from_dec(12_345_678);
        let x = Ter40::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) + divan::black_box(x))
    }

    #[divan::bench]
    fn ter40_neg(b: divan::Bencher) {
        let a = Ter40::from_dec(12_345_678);
        b.bench(|| -divan::black_box(a))
    }

    // --- IlTer40 ---
    #[divan::bench]
    fn ilter40_and(b: divan::Bencher) {
        let a = IlTer40::from_dec(12_345_678);
        let x = IlTer40::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) & divan::black_box(x))
    }

    #[divan::bench]
    fn ilter40_or(b: divan::Bencher) {
        let a = IlTer40::from_dec(12_345_678);
        let x = IlTer40::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) | divan::black_box(x))
    }

    #[divan::bench]
    fn ilter40_add(b: divan::Bencher) {
        let a = IlTer40::from_dec(12_345_678);
        let x = IlTer40::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) + divan::black_box(x))
    }

    #[divan::bench]
    fn ilter40_neg(b: divan::Bencher) {
        let a = IlTer40::from_dec(12_345_678);
        b.bench(|| -divan::black_box(a))
    }

    // --- BctTer32 ---
    #[divan::bench]
    fn bct32_and(b: divan::Bencher) {
        let a = BctTer32::from_dec(12_345_678);
        let x = BctTer32::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) & divan::black_box(x))
    }

    #[divan::bench]
    fn bct32_or(b: divan::Bencher) {
        let a = BctTer32::from_dec(12_345_678);
        let x = BctTer32::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) | divan::black_box(x))
    }

    #[divan::bench]
    fn bct32_add(b: divan::Bencher) {
        let a = BctTer32::from_dec(12_345_678);
        let x = BctTer32::from_dec(-9_876_543);
        b.bench(|| divan::black_box(a) + divan::black_box(x))
    }

    // --- BTer27 / UTer27 ---
    #[divan::bench]
    fn bter27_add(b: divan::Bencher) {
        let a = BTer27::from_dec(1_234_567_890);
        let x = BTer27::from_dec(-987_654_321);
        b.bench(|| divan::black_box(a) + divan::black_box(x))
    }

    #[divan::bench]
    fn bter27_from_dec() -> BTer27 {
        BTer27::from_dec(divan::black_box(1_234_567_890i64))
    }

    #[divan::bench]
    fn bter27_to_dec(b: divan::Bencher) {
        let a = BTer27::from_dec(1_234_567_890);
        b.bench(|| divan::black_box(a).to_dec())
    }

    #[divan::bench]
    fn uter27_from_dec() -> UTer27 {
        UTer27::from_dec(divan::black_box(1_234_567_890u64))
    }

    #[divan::bench]
    fn uter27_to_dec(b: divan::Bencher) {
        let a = UTer27::from_dec(1_234_567_890);
        b.bench(|| divan::black_box(a).to_dec())
    }

    #[divan::bench]
    fn uter27_add(b: divan::Bencher) {
        let a = UTer27::from_dec(1_234_567_890);
        let x = UTer27::from_dec(987_654_321);
        b.bench(|| divan::black_box(a) + divan::black_box(x))
    }
}

// ---------------------------------------------------------------------------
// TERSCII

mod terscii_bench {
    use super::*;

    #[divan::bench]
    fn encode_h() -> Option<Ternary> {
        terscii::encode(divan::black_box('H'))
    }

    #[divan::bench]
    fn encode_space() -> Option<Ternary> {
        terscii::encode(divan::black_box(' '))
    }

    #[divan::bench]
    fn decode_h(b: divan::Bencher) {
        let h = terscii::encode('H').unwrap();
        b.bench(|| terscii::decode(divan::black_box(&h)))
    }

    #[divan::bench]
    fn encode_tryte_h() -> Option<Tryte<5>> {
        terscii::encode_tryte(divan::black_box('H'))
    }

    #[divan::bench]
    fn decode_tryte_h(b: divan::Bencher) {
        let th = terscii::encode_tryte('H').unwrap();
        b.bench(|| terscii::decode_tryte(divan::black_box(th)))
    }

    #[divan::bench]
    fn roundtrip_tryte(b: divan::Bencher) {
        b.bench(|| {
            let t = terscii::encode_tryte(divan::black_box('H')).unwrap();
            terscii::decode_tryte(t)
        })
    }

    #[divan::bench(sample_count = 50000)]
    fn encode_decode_hello_world() -> String {
        let encoded: Vec<Tryte<5>> = "Hello, World!"
            .chars()
            .map(|c| terscii::encode_tryte(c).unwrap())
            .collect();
        encoded
            .iter()
            .map(|&t| terscii::decode_tryte(t).unwrap())
            .collect()
    }
}
