use crate::{
    Digit,
    Digit::{Neg, Pos, Zero},
    Ternary,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg as StdNeg, Not, Shl, Shr, Sub};
use core::str::FromStr;
use crate::concepts::DigitOperate;

/// The `Tryte<S>` struct represents a Copy type balanced ternary number with exactly S digits (6 by default).
/// Each digit in a balanced ternary system can have one of three values: -1, 0, or 1.
///
/// A [Tryte<6>] can holds value between `-364` and `+364`.
///
/// The underlying representation of the number is an array of SIZE `Digit` values.
/// This struct provides conversion methods to and from other formats.
///
/// # Default SIZE
///
/// `SIZE` is 6 by default (the size of a tryte in a Setun computer).
///
/// > **6 trits ~= 9.505 bits**
///
/// > `-364` to `364`
///
/// # Warning
///
/// Because arithmetic operations are performed in with 64 bits integers, `SIZE` cannot be > 40.
///
/// > **40 trits ~= 63.398 bits**
/// >
/// > `-6 078 832 729 528 464 400` to `6 078 832 729 528 464 400`
///
#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub struct Tryte<const SIZE: usize = 6> {
    /// The raw representation of the `Tryte` as SIZE ternary digits.
    raw: [Digit; SIZE],
}

impl<const SIZE: usize> Tryte<SIZE> {
    /// `++...++`
    pub const MAX: Self = Self::new([Pos; SIZE]);
    /// `--...--`
    pub const MIN: Self = Self::new([Neg; SIZE]);
    /// `00...00`
    pub const ZERO: Self = Self::new([Zero; SIZE]);

    /// Creates a new `Tryte` instance from a given array of `Digit`s.
    ///
    /// # Arguments
    ///
    /// * `raw` - An array of exactly SIZE `Digit` values representing the balanced ternary digits.
    ///
    /// # Returns
    ///
    /// A new `Tryte` instance with the specified balanced ternary digits.
    ///
    /// # Panics
    ///
    /// Panic if `SIZE > 40` as 41 trits would be too much information for 64 bits.
    ///
    /// # Examples
    ///
    /// ```
    /// use balanced_ternary::{Tryte, Digit::{Pos, Zero, Neg}};
    ///
    /// let digits = [Pos, Zero, Neg, Zero, Pos, Neg];
    /// let tryte = Tryte::new(digits);
    /// assert_eq!(tryte.to_digit_slice(), &digits);
    /// ```
    pub const fn new(digits: [Digit; SIZE]) -> Self {
        if SIZE > 40 {
            panic!("Cannot construct a Tryte with more than 40 digits (~63.5 bits).")
        }
        Self { raw: digits }
    }

    /// Converts the `Tryte` into its `Ternary` representation.
    ///
    /// # Returns
    ///
    /// A `Ternary` object representing the same balanced ternary number.
    pub fn to_ternary(&self) -> Ternary {
        Ternary::new(self.raw.to_vec())
    }

    /// Retrieves a slice containing the digits of the `Tryte`.
    ///
    /// # Returns
    ///
    /// A slice referencing the six-digit array of the `Tryte`.
    ///
    /// This function allows access to the raw representation of the
    /// balanced ternary number as a slice of `Digit` values.
    pub const fn to_digit_slice(&self) -> &[Digit] {
        &self.raw
    }

    /// Creates a `Tryte` from the given `Ternary`.
    ///
    /// # Arguments
    ///
    /// * `v` - A reference to a `Ternary` object.
    ///
    /// # Panics
    ///
    /// This function panics if the `Ternary` contains more than SIZE digits.
    pub fn from_ternary(v: &Ternary) -> Self {
        if v.log() > SIZE {
            panic!(
                "Cannot convert a Ternary with more than {} digits to a Tryte<{}>.",
                SIZE, SIZE
            );
        }
        let mut digits = [Zero; SIZE];
        for (i, d) in v.digits.iter().rev().enumerate() {
            digits[SIZE - 1 - i] = *d;
        }
        Self::new(digits)
    }

    /// Converts the `Tryte` into a signed 64-bit integer.
    ///
    /// # Returns
    ///
    /// A `i64` representing the decimal value of the `Tryte`.
    ///
    /// # Optimization: direct Horner's method on fixed array
    ///
    /// The previous implementation called `to_ternary()` which heap-allocates
    /// a `Vec<Digit>`, then `to_dec()` on the result. We now apply Horner's
    /// method directly to the stack-resident `[Digit; SIZE]` array — zero
    /// allocations, pure arithmetic.
    pub fn to_i64(&self) -> i64 {
        let mut dec: i64 = 0;
        for d in &self.raw {
            dec = dec * 3 + d.to_i8() as i64;
        }
        dec
    }

    /// Creates a `Tryte` from a signed 64-bit integer.
    ///
    /// # Arguments
    ///
    /// * `v` - A signed 64-bit integer.
    ///
    /// # Returns
    ///
    /// A `Tryte` representing the equivalent ternary number.
    pub const fn from_i64(v: i64) -> Self {
        let mut raw = [Digit::Zero; SIZE];
        if v == 0 {
            return Self::new(raw);
        }
        // Work unsigned: avoids ((n%3)+3)%3 (two signed mod-3 per iteration).
        // Hoist the sign branch outside the loop so each hot path is branch-free.
        // SAFETY: trit ∈ {-1, 0, 1} in both branches.
        let negative = v < 0;
        let mut x = v.unsigned_abs();
        let mut i = SIZE;
        if !negative {
            while i > 0 {
                i -= 1;
                let rem = (x % 3) as u8; // rem ∈ {0, 1, 2}
                // Map: 0→Zero(0), 1→Pos(1), 2→Neg(-1)
                raw[i] = unsafe { core::mem::transmute::<i8, Digit>(
                    if rem == 2 { -1i8 } else { rem as i8 }
                )};
                // Advance: x = (x - trit) / 3
                //   rem=0: x/3  rem=1: (x-1)/3  rem=2: (x-2)/3+1 = (x+1)/3
                x = (x - rem as u64) / 3 + (rem == 2) as u64;
            }
        } else {
            while i > 0 {
                i -= 1;
                let rem = (x % 3) as u8;
                // Negate: 0→Zero(0), 1→Neg(-1), 2→Pos(+1)
                raw[i] = unsafe { core::mem::transmute::<i8, Digit>(
                    if rem == 2 { 1i8 } else { -(rem as i8) }
                )};
                x = (x - rem as u64) / 3 + (rem == 2) as u64;
            }
        }
        Self::new(raw)
    }

    /// Shifts every trit's value one step up the cycle: `-→0`, `0→+`, `+→-`.
    ///
    /// See [`Ternary::shu_up`] for details.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// let t = Tryte::<3>::from("-0+");
    /// assert_eq!(t.shu_up().to_string(), "0+-");
    /// assert_eq!(t.shu_up().shu_up().shu_up().to_string(), "-0+");
    /// ```
    /// # Optimization: 3-element LUT (LLVM emits PSHUFB shuffle)
    ///
    /// Instead of the branchless arithmetic form of `Digit::post` (which is better
    /// for large `Ternary` arrays but generates more instructions for small `Tryte`
    /// sizes), we use a 3-entry compile-time LUT indexed by `d+1`. LLVM recognizes
    /// this pattern and emits a single `PSHUFB` byte-shuffle, processing all SIZE
    /// digits in one vector instruction.
    pub fn shu_up(self) -> Self {
        const MAP: [Digit; 3] = [Digit::Zero, Digit::Pos, Digit::Neg];
        let mut raw = self.raw;
        for d in raw.iter_mut() {
            *d = MAP[(*d as i8 + 1) as usize];
        }
        Self::new(raw)
    }

    /// Shifts every trit's value one step down the cycle: `+→0`, `0→-`, `-→+`.
    ///
    /// See [`Ternary::shu_down`] for details.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// let t = Tryte::<3>::from("-0+");
    /// assert_eq!(t.shu_down().to_string(), "+-0");
    /// ```
    ///
    /// # Optimization: 3-element LUT (LLVM emits PSHUFB shuffle)
    ///
    /// Same reasoning as `shu_up`: 3-entry LUT → PSHUFB for small fixed-size arrays.
    pub fn shu_down(self) -> Self {
        const MAP: [Digit; 3] = [Digit::Pos, Digit::Neg, Digit::Zero];
        let mut raw = self.raw;
        for d in raw.iter_mut() {
            *d = MAP[(*d as i8 + 1) as usize];
        }
        Self::new(raw)
    }

    /// Tritwise consensus: keeps the value where both trytes agree, `Zero` elsewhere.
    ///
    /// See [`Ternary::consensus`] for details.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// // a = [+,+,0,0,-,-], b = [+,0,+,0,-,0]
    /// // Agreement at positions 0 (+,+) and 4 (-,-) only
    /// let a = Tryte::<6>::from("++00--");
    /// let b = Tryte::<6>::from("+0+0-0");
    /// assert_eq!(a.consensus(b).to_string(), "+000-0");
    /// ```
    pub fn consensus(self, other: Self) -> Self {
        self.each_zip(Digit::consensus, other)
    }

    /// Tritwise accept-anything (ANY): non-zero trit wins; conflicting non-zero trits → `Zero`.
    ///
    /// See [`Ternary::accept_anything`] for details.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// let a = Tryte::<6>::from("000---");
    /// let b = Tryte::<6>::from("+++000");
    /// assert_eq!(a.accept_anything(b).to_string(), "+++---");
    /// ```
    pub fn accept_anything(self, other: Self) -> Self {
        self.each_zip(Digit::accept_anything, other)
    }

    /// Clamp every trit toward negative: `Pos→Zero`, others unchanged.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    /// assert_eq!(Tryte::<3>::from("+-0").clamp_down().to_string(), "0-0");
    /// ```
    pub fn clamp_down(self) -> Self { self.each(Digit::clamp_down) }

    /// Clamp every trit toward positive: `Neg→Zero`, others unchanged.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    /// assert_eq!(Tryte::<3>::from("+-0").clamp_up().to_string(), "+00");
    /// ```
    pub fn clamp_up(self) -> Self { self.each(Digit::clamp_up) }

    /// Returns the maximum representable value as an `i64`: `(3^SIZE - 1) / 2`.
    ///
    /// This equals `Self::MAX.to_i64()` but is computed without borrowing `self`.
    const fn max_i64() -> i64 {
        let mut v = 0i64;
        let mut i = 0;
        while i < SIZE {
            v = v * 3 + 1;
            i += 1;
        }
        v
    }

    /// Arithmetic (signed) right shift by `rhs` trit positions: computes `floor(self / 3^rhs)`.
    ///
    /// This matches the Trillium ISA `srs` instruction semantics — unlike the logical
    /// [`Shr`] operator, this rounds toward negative infinity for negative values.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// let t = Tryte::<6>::from_i64(5);   // "+--" in 6-trit form
    /// assert_eq!(t.shr_signed(1).to_i64(),  1); // floor(5/3)   = 1
    ///
    /// let t = Tryte::<6>::from_i64(-5);
    /// assert_eq!(t.shr_signed(1).to_i64(), -2); // floor(-5/3)  = -2
    ///
    /// let t = Tryte::<6>::from_i64(-7);
    /// assert_eq!(t.shr_signed(1).to_i64(), -3); // floor(-7/3)  = -3
    /// ```
    pub fn shr_signed(self, rhs: usize) -> Self {
        if rhs == 0 {
            return self;
        }
        let v = self.to_i64();
        if rhs >= SIZE {
            // |v| < 3^SIZE so |v / 3^rhs| < 1; floor is -1 or 0.
            return if v < 0 { Self::from_i64(-1) } else { Self::ZERO };
        }
        // SAFETY: rhs < SIZE ≤ 40 so rhs ≤ 39 and 3^39 < i64::MAX.
        Self::from_i64(v.div_euclid(3_i64.pow(rhs as u32)))
    }

    /// Checked addition: returns `None` if the result exceeds `[MIN, MAX]`.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// assert_eq!(Tryte::<6>::from_i64(300).checked_add(Tryte::<6>::from_i64(100)), None);
    /// assert_eq!(Tryte::<6>::from_i64(100).checked_add(Tryte::<6>::from_i64(100)),
    ///            Some(Tryte::<6>::from_i64(200)));
    /// ```
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let result = self.to_i64() + rhs.to_i64();
        let max = Self::max_i64();
        if result >= -max && result <= max { Some(Self::from_i64(result)) } else { None }
    }

    /// Checked subtraction: returns `None` if the result exceeds `[MIN, MAX]`.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// assert_eq!(Tryte::<6>::from_i64(-300).checked_sub(Tryte::<6>::from_i64(100)), None);
    /// assert_eq!(Tryte::<6>::from_i64(100).checked_sub(Tryte::<6>::from_i64(50)),
    ///            Some(Tryte::<6>::from_i64(50)));
    /// ```
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        let result = self.to_i64() - rhs.to_i64();
        let max = Self::max_i64();
        if result >= -max && result <= max { Some(Self::from_i64(result)) } else { None }
    }

    /// Checked multiplication: returns `None` if the result exceeds `[MIN, MAX]`.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// assert_eq!(Tryte::<6>::from_i64(100).checked_mul(Tryte::<6>::from_i64(100)), None);
    /// assert_eq!(Tryte::<6>::from_i64(10).checked_mul(Tryte::<6>::from_i64(10)),
    ///            Some(Tryte::<6>::from_i64(100)));
    /// ```
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let result = self.to_i64() * rhs.to_i64();
        let max = Self::max_i64();
        if result >= -max && result <= max { Some(Self::from_i64(result)) } else { None }
    }

    /// Saturating addition: clamps the result to `[MIN, MAX]` on overflow.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// assert_eq!(Tryte::<6>::from_i64(300).saturating_add(Tryte::<6>::from_i64(100)),
    ///            Tryte::<6>::MAX);
    /// assert_eq!(Tryte::<6>::from_i64(-300).saturating_add(Tryte::<6>::from_i64(-100)),
    ///            Tryte::<6>::MIN);
    /// ```
    pub fn saturating_add(self, rhs: Self) -> Self {
        let result = self.to_i64() + rhs.to_i64();
        let max = Self::max_i64();
        if result > max { Self::MAX } else if result < -max { Self::MIN } else { Self::from_i64(result) }
    }

    /// Saturating subtraction: clamps the result to `[MIN, MAX]` on overflow.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// assert_eq!(Tryte::<6>::from_i64(-300).saturating_sub(Tryte::<6>::from_i64(100)),
    ///            Tryte::<6>::MIN);
    /// ```
    pub fn saturating_sub(self, rhs: Self) -> Self {
        let result = self.to_i64() - rhs.to_i64();
        let max = Self::max_i64();
        if result > max { Self::MAX } else if result < -max { Self::MIN } else { Self::from_i64(result) }
    }

    /// Saturating multiplication: clamps the result to `[MIN, MAX]` on overflow.
    ///
    /// ```
    /// use balanced_ternary::Tryte;
    ///
    /// assert_eq!(Tryte::<6>::from_i64(100).saturating_mul(Tryte::<6>::from_i64(100)),
    ///            Tryte::<6>::MAX);
    /// ```
    pub fn saturating_mul(self, rhs: Self) -> Self {
        let result = self.to_i64() * rhs.to_i64();
        let max = Self::max_i64();
        if result > max { Self::MAX } else if result < -max { Self::MIN } else { Self::from_i64(result) }
    }

}

impl<const SIZE: usize> DigitOperate for Tryte<SIZE> {
    fn to_digits(&self) -> Vec<Digit> {
        self.to_digit_slice().to_vec()
    }

    /// Retrieves the digit at the specified index in the `Tryte`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the digit to retrieve (0-based, right-to-left).
    ///
    /// # Returns
    ///
    /// The `Digit` at the specified index or None.
    fn digit(&self, index: usize) -> Option<Digit> {
        if index > SIZE - 1 {
            None
        } else {
            Some(*self.raw.iter().rev().nth(index).unwrap())
        }
    }

    /// # Optimization: operate directly on the fixed-size array
    ///
    /// The previous implementation converted to a heap-allocated `Ternary`,
    /// applied the transform, then converted back. Since `Tryte` is a
    /// fixed-size `[Digit; SIZE]` on the stack, we can transform in-place
    /// with zero allocations.
    fn each(&self, f: impl Fn(Digit) -> Digit) -> Self {
        let mut raw = self.raw;
        for d in raw.iter_mut() {
            *d = f(*d);
        }
        Self::new(raw)
    }

    /// # Optimization: direct array transform (see `each` above)
    fn each_with(&self, f: impl Fn(Digit, Digit) -> Digit, with: Digit) -> Self {
        let mut raw = self.raw;
        for d in raw.iter_mut() {
            *d = f(*d, with);
        }
        Self::new(raw)
    }

    /// Direct array zip — no heap allocation.
    /// Previous impl converted to `Ternary` twice (2 allocs), operated, then
    /// converted back (1 alloc). Now operates in-place on the fixed stack array.
    fn each_zip(&self, f: impl Fn(Digit, Digit) -> Digit, other: Self) -> Self {
        let mut raw = self.raw;
        for i in 0..SIZE {
            raw[i] = f(self.raw[i], other.raw[i]);
        }
        Self::new(raw)
    }

    /// Direct array carry-zip — no heap allocation.
    /// Same motivation as `each_zip`: eliminates 2 intermediate `Ternary` allocs.
    fn each_zip_carry(
        &self,
        f: impl Fn(Digit, Digit, Digit) -> (Digit, Digit),
        other: Self,
    ) -> Self {
        let mut raw = self.raw;
        let mut carry = Digit::Zero;
        for i in (0..SIZE).rev() {
            let (c, res) = f(self.raw[i], other.raw[i], carry);
            carry = c;
            raw[i] = res;
        }
        Self::new(raw)
    }
}


impl<const SIZE: usize> Display for Tryte<SIZE> {
    /// Formats the `Tryte` for display.
    ///
    /// # Optimization: direct digit→char without Ternary allocation
    ///
    /// The previous implementation called `self.to_ternary()` (heap alloc)
    /// then `.to_string()` (second heap alloc) then formatted with width
    /// padding. We now write each digit's char directly from the fixed-size
    /// array — zero allocations.
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Build on the stack (SIZE bytes), write once — zero heap, one write_str call.
        let mut buf = [0u8; SIZE];
        for (i, d) in self.raw.iter().enumerate() {
            buf[i] = d.to_char() as u8;
        }
        // SAFETY: '+', '-', '0' are all valid single-byte UTF-8.
        f.write_str(unsafe { core::str::from_utf8_unchecked(&buf) })
    }
}

impl<const SIZE: usize> StdNeg for Tryte<SIZE> {
    type Output = Tryte<SIZE>;

    /// # Optimization: direct array negation
    ///
    /// Negating each element of a fixed-size stack array is far cheaper
    /// than converting to a heap-allocated `Ternary`, negating, and
    /// converting back.
    fn neg(self) -> Self::Output {
        let mut raw = self.raw;
        for d in raw.iter_mut() {
            *d = -*d;
        }
        Self::new(raw)
    }
}

/// Arithmetic ops route through `to_i64()`/`from_i64()` — zero heap allocations.
/// Previous impl converted each operand to a heap `Ternary`, performed Ternary
/// arithmetic (which itself may alloc for add/sub), then converted back: 3-5 allocs.
/// Since SIZE ≤ 40 is enforced by `new()`, i64 arithmetic never overflows.
impl<const SIZE: usize> Add for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from_i64(self.to_i64() + rhs.to_i64())
    }
}

impl<const SIZE: usize> Sub for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_i64(self.to_i64() - rhs.to_i64())
    }
}

impl<const SIZE: usize> Mul for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_i64(self.to_i64() * rhs.to_i64())
    }
}

impl<const SIZE: usize> Div for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn div(self, rhs: Self) -> Self::Output {
        Self::from_i64(self.to_i64() / rhs.to_i64())
    }
}

/// Bitwise ops route through `each_zip` — zero heap allocations.
/// Previous impl called `to_ternary()` twice + `from_ternary()`: 3 allocs each.
impl<const SIZE: usize> BitAnd for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn bitand(self, rhs: Self) -> Self::Output {
        self.each_zip(Digit::bitand, rhs)
    }
}

impl<const SIZE: usize> BitOr for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn bitor(self, rhs: Self) -> Self::Output {
        self.each_zip(Digit::bitor, rhs)
    }
}

impl<const SIZE: usize> BitXor for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.each_zip(Digit::bitxor, rhs)
    }
}

impl<const SIZE: usize> Not for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn not(self) -> Self::Output {
        -self
    }
}

/// Logical left shift: moves all trits toward the MSB, fills vacated LSBs with `Zero`.
///
/// Equivalent to multiplying by `3^rhs` and truncating to `SIZE` trits.
///
/// ```
/// use balanced_ternary::Tryte;
///
/// let t = Tryte::<6>::from("-0+");   // "000-0+"
/// assert_eq!((t << 2usize).to_string(), "0-0+00");
/// assert_eq!((t << 6usize).to_string(), "000000"); // shift ≥ SIZE → zero
/// ```
impl<const SIZE: usize> Shl<usize> for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn shl(self, rhs: usize) -> Self::Output {
        if rhs >= SIZE {
            return Self::ZERO;
        }
        let mut raw = [Zero; SIZE];
        raw[..SIZE - rhs].copy_from_slice(&self.raw[rhs..]);
        Self::new(raw)
    }
}

/// Logical right shift: moves all trits toward the LSB, fills vacated MSBs with `Zero`.
///
/// Equivalent to integer division by `3^rhs`, truncating toward zero (i.e. `to_i64() / 3^rhs`).
/// For floor-toward-negative-infinity, use [`Tryte::shr_signed`].
///
/// ```
/// use balanced_ternary::Tryte;
///
/// let t = Tryte::<6>::from_i64(5);   // "+--" sign-extended to "000+--"
/// assert_eq!((t >> 1usize).to_i64(), 2);  // (5 - (-1)) / 3 = 2
/// assert_eq!((t >> 6usize).to_i64(), 0);  // shift ≥ SIZE → zero
/// ```
impl<const SIZE: usize> Shr<usize> for Tryte<SIZE> {
    type Output = Tryte<SIZE>;
    fn shr(self, rhs: usize) -> Self::Output {
        if rhs >= SIZE {
            return Self::ZERO;
        }
        let mut raw = [Zero; SIZE];
        raw[rhs..].copy_from_slice(&self.raw[..SIZE - rhs]);
        Self::new(raw)
    }
}

impl<const SIZE: usize> From<Ternary> for Tryte<SIZE> {
    fn from(value: Ternary) -> Self {
        Tryte::from_ternary(&value)
    }
}

impl<const SIZE: usize> From<Tryte<SIZE>> for Ternary {
    fn from(value: Tryte<SIZE>) -> Self {
        value.to_ternary()
    }
}

impl<const SIZE: usize> From<&str> for Tryte<SIZE> {
    fn from(value: &str) -> Self {
        Self::from_ternary(&Ternary::parse(value))
    }
}

impl<const SIZE: usize> From<String> for Tryte<SIZE> {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl<const SIZE: usize> From<Tryte<SIZE>> for String {
    fn from(value: Tryte<SIZE>) -> Self {
        value.to_string()
    }
}

impl<const SIZE: usize> From<i64> for Tryte<SIZE> {
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}

impl<const SIZE: usize> From<Tryte<SIZE>> for i64 {
    fn from(value: Tryte<SIZE>) -> Self {
        value.to_i64()
    }
}

impl<const SIZE: usize> FromStr for Tryte<SIZE> {
    type Err = crate::ParseTernaryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tryte::from_ternary(&Ternary::from_str(s)?))
    }
}

#[cfg(test)]
#[test]
pub fn test_tryte() {
    let tryte = Tryte::<6>::from_i64(255);
    assert_eq!(tryte.to_i64(), 255);
    assert_eq!(tryte.to_string(), "+00++0");

    let tryte = Tryte::<6>::from_i64(16);
    assert_eq!(tryte.to_i64(), 16);
    assert_eq!(tryte.to_string(), "00+--+");

    assert_eq!(Tryte::<6>::MAX.to_string(), "++++++");
    assert_eq!(Tryte::<6>::MAX.to_i64(), 364);
    assert_eq!(Tryte::<6>::MIN.to_string(), "------");
    assert_eq!(Tryte::<6>::MIN.to_i64(), -364);
    assert_eq!(Tryte::<6>::ZERO.to_string(), "000000");
    assert_eq!(Tryte::<6>::ZERO.to_i64(), 0);
}

#[cfg(test)]
#[test]
pub fn test_tryte_from_str() {
    use core::str::FromStr;

    let tryte = Tryte::<6>::from_str("+-0").unwrap();
    assert_eq!(tryte.to_string(), "000+-0");

    assert!(Tryte::<6>::from_str("+-x").is_err());
}

#[cfg(test)]
#[test]
pub fn test_tryte_shu() {
    let t = Tryte::<3>::from("-0+");

    // SHU↑: -→0, 0→+, +→-
    assert_eq!(t.shu_up().to_string(), "0+-");
    assert_eq!(t.shu_up().shu_up().to_string(), "+-0");
    // Period-3 identity
    assert_eq!(t.shu_up().shu_up().shu_up().to_string(), "-0+");

    // SHU↓ is inverse of SHU↑
    assert_eq!(t.shu_down().to_string(), "+-0");
    assert_eq!(t.shu_up().shu_down(), t);
    assert_eq!(t.shu_down().shu_up(), t);
}

#[cfg(test)]
#[test]
pub fn test_tryte_consensus() {
    // a = [+,+,0,0,-,-], b = [+,0,+,0,-,0]
    // Agreement only at pos 0 (+,+) and pos 4 (-,-)
    let a = Tryte::<6>::from("++00--");
    let b = Tryte::<6>::from("+0+0-0");
    assert_eq!(a.consensus(b).to_string(), "+000-0");

    // Self-consensus = identity
    assert_eq!(a.consensus(a), a);

    // Opposing trytes → all zeros
    let neg_a = Tryte::<6>::from("--00++");
    assert_eq!(a.consensus(neg_a), Tryte::<6>::ZERO);
}

#[cfg(test)]
#[test]
pub fn test_tryte_accept_anything() {
    // Non-overlapping fields merge losslessly
    let a = Tryte::<6>::from("000---");
    let b = Tryte::<6>::from("+++000");
    assert_eq!(a.accept_anything(b).to_string(), "+++---");

    // Zero is transparent
    assert_eq!(a.accept_anything(Tryte::<6>::ZERO), a);
    assert_eq!(Tryte::<6>::ZERO.accept_anything(a), a);

    // Conflict cancels
    let pos = Tryte::<3>::from("00+");
    let neg = Tryte::<3>::from("00-");
    assert_eq!(pos.accept_anything(neg), Tryte::<3>::ZERO);
}

#[cfg(test)]
#[test]
pub fn test_tryte_shl_shr() {
    // Logical left shift (toward MSB): vacate LSBs with Zero
    let t = Tryte::<6>::from_i64(5); // "000+--"
    assert_eq!((t << 1usize).to_string(), "00+--0");
    assert_eq!((t << 3usize).to_string(), "+--000");
    // Shift ≥ SIZE → zero
    assert_eq!((t << 6usize), Tryte::<6>::ZERO);
    assert_eq!((t << 7usize), Tryte::<6>::ZERO);

    // Logical right shift (toward LSB): vacate MSBs with Zero
    // shr gives balanced-ternary quotient (n - LSB) / 3, not floor division
    assert_eq!((t >> 1usize).to_string(), "0000+-");
    assert_eq!((t >> 6usize), Tryte::<6>::ZERO);

    // Round-trip: shl then shr restores high bits only
    let t2 = Tryte::<6>::from_i64(9); // "000+00"
    assert_eq!((t2 << 1usize >> 1usize), t2);
}

#[cfg(test)]
#[test]
pub fn test_tryte_shr_signed() {
    // floor(5/3) = 1 (positive, negative LSB requires +1 quotient adjustment)
    assert_eq!(Tryte::<6>::from_i64(5).shr_signed(1).to_i64(), 1);
    // floor(-5/3) = -2
    assert_eq!(Tryte::<6>::from_i64(-5).shr_signed(1).to_i64(), -2);
    // floor(-7/3) = -3 (negative, negative LSB trit)
    assert_eq!(Tryte::<6>::from_i64(-7).shr_signed(1).to_i64(), -3);
    // Exact: floor(9/3) = 3
    assert_eq!(Tryte::<6>::from_i64(9).shr_signed(1).to_i64(), 3);
    // Exact negative: floor(-9/3) = -3
    assert_eq!(Tryte::<6>::from_i64(-9).shr_signed(1).to_i64(), -3);
    // Large shift of small value: floor(5/27) = 0
    assert_eq!(Tryte::<6>::from_i64(5).shr_signed(3).to_i64(), 0);
    // Large shift of negative: floor(-1/3^6) = -1 (rhs >= SIZE path)
    assert_eq!(Tryte::<6>::from_i64(-1).shr_signed(6).to_i64(), -1);
    // Zero shift is identity
    assert_eq!(Tryte::<6>::from_i64(42).shr_signed(0).to_i64(), 42);
}

#[cfg(test)]
#[test]
pub fn test_tryte_checked_saturating() {
    // checked_add: overflow returns None
    assert_eq!(Tryte::<6>::from_i64(300).checked_add(Tryte::<6>::from_i64(100)), None);
    assert_eq!(Tryte::<6>::from_i64(-300).checked_add(Tryte::<6>::from_i64(-100)), None);
    assert_eq!(
        Tryte::<6>::from_i64(100).checked_add(Tryte::<6>::from_i64(100)),
        Some(Tryte::<6>::from_i64(200))
    );

    // checked_sub
    assert_eq!(Tryte::<6>::from_i64(-300).checked_sub(Tryte::<6>::from_i64(100)), None);
    assert_eq!(
        Tryte::<6>::from_i64(100).checked_sub(Tryte::<6>::from_i64(50)),
        Some(Tryte::<6>::from_i64(50))
    );

    // checked_mul
    assert_eq!(Tryte::<6>::from_i64(100).checked_mul(Tryte::<6>::from_i64(100)), None);
    assert_eq!(
        Tryte::<6>::from_i64(10).checked_mul(Tryte::<6>::from_i64(10)),
        Some(Tryte::<6>::from_i64(100))
    );

    // saturating_add clamps
    assert_eq!(Tryte::<6>::from_i64(300).saturating_add(Tryte::<6>::from_i64(100)), Tryte::<6>::MAX);
    assert_eq!(Tryte::<6>::from_i64(-300).saturating_add(Tryte::<6>::from_i64(-100)), Tryte::<6>::MIN);
    assert_eq!(Tryte::<6>::from_i64(100).saturating_add(Tryte::<6>::from_i64(100)), Tryte::<6>::from_i64(200));

    // saturating_sub
    assert_eq!(Tryte::<6>::from_i64(-300).saturating_sub(Tryte::<6>::from_i64(100)), Tryte::<6>::MIN);

    // saturating_mul clamps
    assert_eq!(Tryte::<6>::from_i64(100).saturating_mul(Tryte::<6>::from_i64(100)), Tryte::<6>::MAX);
    assert_eq!(Tryte::<6>::from_i64(-100).saturating_mul(Tryte::<6>::from_i64(100)), Tryte::<6>::MIN);
}
