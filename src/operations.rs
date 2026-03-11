//! This module provides implementations for arithmetic operations on the `Ternary` type
//! such as addition, subtraction, multiplication, and division.
//! Using `Ternary` arithmetic:
//!
//! ```rust
//! use balanced_ternary::Ternary;
//!
//! let repr9 = Ternary::parse("+00"); // Represents decimal 9 in balanced ternary
//! let repr4 = Ternary::parse("++");  // Represents decimal 4 in balanced ternary
//! let sum = &repr9 + &repr4;         // Results in Ternary::parse("+++"), decimal 13
//! assert_eq!(sum.to_dec(), 13);
//! let difference = &sum - &repr4;   // Results in Ternary::parse("+00"), decimal 9
//! assert_eq!(difference.to_dec(), 9);
//! ```
//!
//! # Implementations
//!
//! The following arithmetic operations are implemented for the `Ternary` :
//!
//! ## `Ternary` type
//!
//! - `Neg` and `Not` for `&Ternary`: Negates the `Ternary` by negating each digit in its balanced ternary representation.
//! - `Add<&Ternary>` for `&Ternary`: Adds two `Ternary` values and returns a new `Ternary`. Panics on overflow.
//! - `Sub<&Ternary>` for `&Ternary`: Subtracts one `Ternary` from another and returns a new `Ternary`. Panics on overflow.
//! - `Mul<&Ternary>` for `&Ternary`: Multiplies two `Ternary` values and returns a new `Ternary`. Panics on overflow.
//! - `Div<&Ternary>` for `&Ternary`: Divides one `Ternary` by another and returns a new `Ternary`. Panics on overflow or division by zero.
//! - `BitAnd<&Ternary>` for `&Ternary`: Computes the bitwise AND operation on two `Ternary` operands.
//! - `BitOr<&Ternary>` for `&Ternary`: Computes the bitwise OR operation on two `Ternary` operands.
//! - `BitXor<&Ternary>` for `&Ternary`: Computes the bitwise XOR operation on two `Ternary` operands.

use crate::concepts::DigitOperate;
use crate::{Digit, Ternary};
use alloc::vec;
use alloc::vec::Vec;
use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Sub, Shl, Shr};

/// Balanced ternary carry decomposition.
///
/// Given a `sum` in the range `[-3, 3]` (the result of adding up to three
/// trits), decomposes it into `(carry, digit)` where `sum = carry * 3 + digit`
/// and both `carry` and `digit` are in `{-1, 0, +1}`.
///
/// # How it works
///
/// Balanced ternary digits must lie in `{-1, 0, +1}`. When a column sum falls
/// outside that range, we "rebalance" by shifting excess into a carry:
///
/// - `sum ∈ {-1, 0, +1}` → no carry needed, digit = sum.
/// - `sum ∈ {+2, +3}` → carry = +1, digit = sum − 3 (maps 2→−1, 3→0).
/// - `sum ∈ {−2, −3}` → carry = −1, digit = sum + 3 (maps −2→+1, −3→0).
///
/// This is the ternary analogue of binary's "half-adder carry" and is the
/// key primitive that makes native ternary addition possible without
/// converting to/from decimal.
#[inline]
fn balanced_carry(sum: i8) -> (i8, i8) {
    // Branchless: carry is +1 when sum > 1, -1 when sum < -1, else 0.
    // Same signum pattern as `Digit::accept_anything`.
    // digit = sum - carry * 3 maps {2→-1, 3→0, -2→1, -3→0}, identity elsewhere.
    let carry = (sum > 1) as i8 - (sum < -1) as i8;
    (carry, sum - carry * 3)
}

/// Perform native balanced ternary addition/subtraction on two digit slices.
///
/// When `negate_b` is false this computes `a + b`; when true, `a - b`.
///
/// # Optimization: native ternary ripple-carry
///
/// The previous `Add`/`Sub` implementations converted both operands to `i64`
/// via `to_dec()` (O(n) multiplications each), performed a single i64
/// operation, then converted back via `from_dec()` (string allocation +
/// per-digit parsing). This tripled the work and also imposed an artificial
/// `i64::MAX` overflow limit.
///
/// This implementation operates directly on the digit arrays using
/// `balanced_carry()` as a ternary full-adder, processing one trit per
/// iteration in a single right-to-left pass. The result:
///
/// - **No intermediate conversions** — digits in, digits out.
/// - **No allocations** beyond the result `Vec` (pre-sized with `with_capacity`).
/// - **No i64 overflow limit** — the output simply grows by one trit if needed.
/// - **Cache-friendly** — sequential array access, no pointer chasing.
fn ternary_add_sub(a: &[Digit], b: &[Digit], negate_b: bool) -> Ternary {
    let len = a.len().max(b.len());

    let mut digits = Vec::with_capacity(len + 1);
    let mut carry: i8 = 0;

    // Fast path: equal-length operands — eliminates per-iteration branch.
    if a.len() == b.len() {
        for (&da, &db) in a.iter().rev().zip(b.iter().rev()) {
            let db_val = if negate_b { -db.to_i8() } else { db.to_i8() };
            let (c, d) = balanced_carry(da.to_i8() + db_val + carry);
            carry = c;
            // SAFETY: balanced_carry output is in {-1, 0, 1}.
            digits.push(unsafe { core::mem::transmute::<i8, Digit>(d) });
        }
    } else {
        let (oa, ob) = (len - a.len(), len - b.len());
        for i in (0..len).rev() {
            let da = if i < oa { 0 } else { a[i - oa].to_i8() };
            let mut db = if i < ob { 0 } else { b[i - ob].to_i8() };
            if negate_b {
                db = -db;
            }
            let (c, d) = balanced_carry(da + db + carry);
            carry = c;
            // SAFETY: balanced_carry output is in {-1, 0, 1}.
            digits.push(unsafe { core::mem::transmute::<i8, Digit>(d) });
        }
    }

    if carry != 0 {
        // SAFETY: balanced_carry carry is in {-1, 0, 1}.
        digits.push(unsafe { core::mem::transmute::<i8, Digit>(carry) });
    }

    digits.reverse();

    // Trim leading zeros in-place — `drain` avoids a second allocation.
    let first_nonzero = digits.iter().position(|d| *d != Digit::Zero);
    match first_nonzero {
        None => { digits.truncate(0); digits.push(Digit::Zero); }
        Some(0) => {}
        Some(pos) => { digits.drain(0..pos); }
    }
    Ternary::new(digits)
}

impl Neg for &Ternary {
    type Output = Ternary;

    fn neg(self) -> Self::Output {
        self.each(|d| -d)
    }
}

impl Add<&Ternary> for &Ternary {
    type Output = Ternary;

    fn add(self, rhs: &Ternary) -> Self::Output {
        ternary_add_sub(self.to_digit_slice(), rhs.to_digit_slice(), false)
    }
}

impl Add<Digit> for &Ternary {
    type Output = Ternary;

    /// Single-digit add: ripple a carry from the LSB upward.
    ///
    /// # Optimization: in-place carry propagation
    ///
    /// Instead of converting the whole number to `i64` and back, we clone
    /// the digit array and propagate a single carry from the least-significant
    /// trit upward. Early-exit when carry becomes zero means best-case is O(1)
    /// and worst-case (all `Pos` + 1, full ripple) is O(n).
    fn add(self, rhs: Digit) -> Self::Output {
        let mut digits: Vec<Digit> = self.to_digit_slice().to_vec();
        let mut carry = rhs.to_i8();
        for d in digits.iter_mut().rev() {
            if carry == 0 {
                break;
            }
            let (c, r) = balanced_carry(d.to_i8() + carry);
            // SAFETY: balanced_carry output is in {-1, 0, 1}.
            *d = unsafe { core::mem::transmute::<i8, Digit>(r) };
            carry = c;
        }
        if carry != 0 {
            // SAFETY: balanced_carry carry is in {-1, 0, 1}.
            digits.insert(0, unsafe { core::mem::transmute::<i8, Digit>(carry) });
        }
        Ternary::new(digits)
    }
}

impl Sub<&Ternary> for &Ternary {
    type Output = Ternary;

    fn sub(self, rhs: &Ternary) -> Self::Output {
        ternary_add_sub(self.to_digit_slice(), rhs.to_digit_slice(), true)
    }
}

impl Sub<Digit> for &Ternary {
    type Output = Ternary;

    fn sub(self, rhs: Digit) -> Self::Output {
        // Subtraction of a digit is addition of its negation.
        self + (-rhs)
    }
}

impl Mul<&Ternary> for &Ternary {
    type Output = Ternary;

    /// # Optimization: i64 fast path + stack-buffer schoolbook fallback
    ///
    /// **Fast path (≤ 20 digits each):** The maximum absolute value of a
    /// 20-digit balanced-ternary number is `(3²⁰ − 1) / 2 = 1 743 392 200`,
    /// so the maximum product magnitude is `1 743 392 200² ≈ 3.04 × 10¹⁸`,
    /// which is below `i64::MAX ≈ 9.22 × 10¹⁸`. We therefore delegate to
    /// `to_dec() * rhs.to_dec()` + `from_dec()` — two O(n) passes and one
    /// machine multiply — instead of an O(n²) digit loop. This covers every
    /// size benchmarked (4, 10, 16, 20 digits) with no overflow risk.
    ///
    /// **Schoolbook fallback (> 20 digits on either side):** a stack-resident
    /// `i8` accumulator (128 bytes) avoids heap allocation for products up to
    /// 64 × 64 digits. Larger inputs fall back to a heap buffer.
    fn mul(self, rhs: &Ternary) -> Self::Output {
        let a = self.to_digit_slice();
        let b = rhs.to_digit_slice();

        if a.len() <= 20 && b.len() <= 20 {
            return Ternary::from_dec(self.to_dec() * rhs.to_dec());
        }

        let rlen = a.len() + b.len();

        // Stack accumulator — 128 covers products up to 64×64 digit numbers.
        // Falls back to heap for larger inputs.
        let mut stack_buf = [0i8; 128];
        let mut heap_buf;
        let acc: &mut [i8] = if rlen <= 128 {
            &mut stack_buf[..rlen]
        } else {
            heap_buf = vec![0i8; rlen];
            &mut heap_buf
        };

        for j in 0..b.len() {
            let bd = b[j].to_i8();
            if bd == 0 { continue; }
            for i in 0..a.len() {
                acc[i + j + 1] += a[i].to_i8() * bd;
            }
        }

        // Single right-to-left carry-rebalancing pass.
        for k in (1..rlen).rev() {
            let v = acc[k];
            let mut rem = v % 3;
            let mut carry = v / 3;
            // Branchless adjustment: same signum pattern as `balanced_carry`.
            let adj = (rem > 1) as i8 - (rem < -1) as i8;
            rem -= adj * 3;
            carry += adj;
            acc[k] = rem;
            acc[k - 1] += carry;
        }

        // Rebalance position 0 — may produce one extra carry digit.
        let v0 = acc[0];
        let extra = if v0 < -1 || v0 > 1 {
            let mut rem = v0 % 3;
            let mut carry = v0 / 3;
            if rem > 1 { rem -= 3; carry += 1; }
            else if rem < -1 { rem += 3; carry -= 1; }
            acc[0] = rem;
            if carry != 0 { Some(carry) } else { None }
        } else {
            None
        };

        // Build result. Since Digit is #[repr(i8)] with Neg=-1, Zero=0,
        // Pos=1 and all rebalanced values are in {-1,0,1}, we transmute
        // each i8 directly to Digit — no match/branch per element.
        #[inline(always)]
        unsafe fn i8_to_digit(v: i8) -> Digit {
            debug_assert!(
                v >= -1 && v <= 1,
                "balanced_carry rebalancing produced invalid trit value: {v}"
            );
            core::mem::transmute::<i8, Digit>(v)
        }

        let first_nz = acc.iter().position(|&v| v != 0);
        match (extra, first_nz) {
            (Some(c), _) => {
                let mut digits = Vec::with_capacity(1 + rlen);
                digits.push(unsafe { i8_to_digit(c) });
                for &v in acc.iter() {
                    digits.push(unsafe { i8_to_digit(v) });
                }
                Ternary::new(digits)
            }
            (None, None) => Ternary::new(vec![Digit::Zero]),
            (None, Some(pos)) => {
                let mut digits = Vec::with_capacity(rlen - pos);
                for &v in &acc[pos..] {
                    digits.push(unsafe { i8_to_digit(v) });
                }
                Ternary::new(digits)
            }
        }
    }
}

impl Div<&Ternary> for &Ternary {
    type Output = Ternary;

    fn div(self, rhs: &Ternary) -> Self::Output {
        Ternary::from_dec(
            self.to_dec()
                .checked_div(rhs.to_dec())
                .expect("Overflow in division or division by zero."),
        )
    }
}

impl BitAnd<&Ternary> for &Ternary {
    type Output = Ternary;

    fn bitand(self, rhs: &Ternary) -> Self::Output {
        self.each_zip(Digit::bitand, rhs.clone())
    }
}

impl BitOr<&Ternary> for &Ternary {
    type Output = Ternary;

    fn bitor(self, rhs: &Ternary) -> Self::Output {
        self.each_zip(Digit::bitor, rhs.clone())
    }
}

impl BitXor<&Ternary> for &Ternary {
    type Output = Ternary;

    fn bitxor(self, rhs: &Ternary) -> Self::Output {
        self.each_zip(Digit::bitxor, rhs.clone())
    }
}

impl Shl<usize> for &Ternary {
    type Output = Ternary;

    /// # Optimization: pre-allocated vec
    ///
    /// Output length = input length + shift amount. Single allocation via
    /// `with_capacity`, then `extend_from_slice` (one `memcpy` for `Copy`
    /// types) followed by zero-fill.
    fn shl(self, rhs: usize) -> Self::Output {
        let mut digits = Vec::with_capacity(self.to_digit_slice().len() + rhs);
        digits.extend_from_slice(self.to_digit_slice());
        digits.extend(core::iter::repeat(Digit::Zero).take(rhs));
        Ternary::new(digits)
    }
}

impl Shr<usize> for &Ternary {
    type Output = Ternary;

    fn shr(self, rhs: usize) -> Self::Output {
        if rhs >= self.to_digit_slice().len() {
            return Ternary::new(vec![Digit::Zero]);
        }
        let len = self.to_digit_slice().len() - rhs;
        Ternary::new(self.to_digit_slice()[..len].to_vec())
    }
}

impl Not for &Ternary {
    type Output = Ternary;
    fn not(self) -> Self::Output {
        -self
    }
}

#[cfg(test)]
#[test]
fn test_ternary_ops() {
    use alloc::string::ToString;

    let repr9 = Ternary::parse("+00");
    let repr4 = Ternary::parse("++");
    let repr13 = &repr9 + &repr4;
    let repr17 = &repr13 + &repr4;
    let repr34 = &repr17 + &repr17;

    assert_eq!(repr13.to_string(), "+++");
    assert_eq!(repr17.to_string(), "+-0-");
    assert_eq!(repr34.to_string(), "++-+");

    let repr30 = &repr34 - &repr4;
    assert_eq!(repr30.to_dec(), 30);
    assert_eq!(repr30.to_string(), "+0+0");

    let repr120 = &repr30 * &repr4;
    assert_eq!(repr120.to_dec(), 120);
    assert_eq!(repr120.to_string(), "++++0");

    let repr_neg120 = -&repr120;
    assert_eq!(repr_neg120.to_dec(), -120);
    assert_eq!(repr_neg120.to_string(), "----0");

    let bitwise = &Ternary::parse("++00") & &Ternary::parse("0000");
    assert_eq!(bitwise.to_string(), "0000");

    let bitwise = &Ternary::parse("++00") & &Ternary::parse("0+00");
    assert_eq!(bitwise.to_string(), "0+00");

    let bitwise = &Ternary::parse("+000") | &Ternary::parse("000-");
    assert_eq!(bitwise.to_string(), "+000");

    let bitwise = &Ternary::parse("+000") & &Ternary::parse("000-");
    assert_eq!(bitwise.to_string(), "000-");

    let bitwise = &Ternary::parse("+000") | &Ternary::parse("000+");
    assert_eq!(bitwise.to_string(), "+00+");
}

#[cfg(test)]
#[test]
fn test_shift_ops() {
    use alloc::string::ToString;
    let t = Ternary::parse("+0-");
    assert_eq!((&t << 2).to_string(), "+0-00");
    let back = &(&t << 2) >> 2;
    assert_eq!(back.to_string(), "+0-");
    let zero = &t >> 5;
    assert_eq!(zero.to_string(), "0");
}
