//! A [balanced ternary](https://en.wikipedia.org/wiki/Balanced_ternary) data structure.
//!
//! A `Ternary` object in this module represents a number in the balanced ternary numeral system.
//! Balanced ternary is a non-standard positional numeral system that uses three digits: {-1, 0, +1}
//! represented here as `Neg` for -1, `Zero` for 0, and `Pos` for +1. It is useful in some domains
//! of computer science and mathematics due to its arithmetic properties and representation
//! symmetry.
//!
//! # Data Structures
//!
//! - **`Digit` Enum**:
//!     Represents a single digit for balanced ternary values, with possible values:
//!     - `Neg` for -1
//!     - `Zero` for 0
//!     - `Pos` for +1
//!
//! ## Features
//!
//! All features are enabled by default.
//!
//! To enable only some features, use the `default-features` option
//! in your [dependency declaration](https://doc.rust-lang.org/cargo/reference/features.html#dependency-features):
//!
//! ```toml
//! [dependencies.balanced-ternary]
//! version = "*.*"
//! default-features = false
//! 
//! # Choose which one to enable
//! features = ["ternary-string", "tryte", "ternary-store"]
//! ```
//!
//! ### Featureless
//!
//! Without any feature, this library provide the type `Digit` and all its operations and the trait `DigitOperate`.
//!
//! ### `ternary-string`
//!
//! Add the structure [Ternary] which is a vector of [Digit]s and a lot of utilities
//! to manipulate digits into the ternary number. Implements [DigitOperate].
//!
//! ### `tryte`
//!
//! > Needs the feature `ternary-string`.
//!
//! Add the type [Tryte]`<N>` which is a fixed size copy-type ternary number. Implements [DigitOperate].
//!
//! ### `ternary-store`
//!
//! > Needs the feature `ternary-string`.
//!
//! Add structures to store ternaries efficiently. These types are provided:
//! - [DataTernary]: a variable length ternary number stored into [TritsChunk]s,
//! - [TritsChunk]: a fixed size copy-type 5 digits stored into one byte,
//! - [Ter40]: a fixed size copy-type 40 digits stored into one 64 bits integer. Implements [DigitOperate].
//! - [BctTer32]: 32-trit split BCT `(pos:u32, neg:u32)` — O(1) trit-logical ops.
//! - [IlBctTer32]: 32-trit Jones interleaved BCT in a single `u64` (`00=−1, 01=0, 10=+1`).
//!

#![no_std]
extern crate alloc;

pub mod concepts;

#[cfg(feature = "ternary-string")]
use alloc::{format, string::String, vec, vec::Vec};

use crate::concepts::DigitOperate;
#[cfg(feature = "ternary-string")]
use core::{
    fmt::{Display, Formatter, Write as FmtWrite},
    str::FromStr,
    error::Error,
    cmp::Ordering,
};

#[cfg(feature = "ternary-string")]
/// Error returned when parsing a string into a [`Ternary`] fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTernaryError;

#[cfg(feature = "ternary-string")]
impl Display for ParseTernaryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "invalid character in balanced ternary string")
    }
}

#[cfg(feature = "ternary-string")]
impl Error for ParseTernaryError {}

/// Provides helper functions for formatting integers in a given radix.
///
/// Used internally to convert decimal numbers into their ternary representation.
/// - `x`: The number to be formatted.
/// - `radix`: The base of the numeral system.
///
/// Returns a string representation of the number in the specified base.
#[cfg(feature = "ternary-string")]
fn format_radix(x: i64, radix: u32) -> String {
    let mut result = vec![];
    let sign = x.signum();
    let mut x = x.unsigned_abs();
    loop {
        let m = (x % radix as u64) as u32;
        x /= radix as u64;
        result.push(core::char::from_digit(m, radix).unwrap());
        if x == 0 {
            break;
        }
    }
    format!(
        "{}{}",
        if sign == -1 { "-" } else { "" },
        result.into_iter().rev().collect::<String>()
    )
}

mod digit;

pub use crate::digit::{
    Digit,
    Digit::{Neg, Pos, Zero},
};

/// Converts a character into a `Digit`.
///
/// # Arguments
/// * `from` - A single character (`+`, `0`, or `-`).
/// * **Panics** if the input character is invalid.
///
/// # Returns
/// * A `Digit` enum corresponding to the character.
///
/// # Example
/// ```
/// use balanced_ternary::{trit, Digit};
///
/// let digit = trit('+');
/// assert_eq!(digit, Digit::Pos);
/// ```
pub const fn trit(from: char) -> Digit {
    Digit::from_char(from)
}

/// Converts a string representation of a balanced ternary number into a `Ternary` object.
///
/// This function is a convenient shorthand for creating `Ternary` numbers
/// from string representations. The input string must consist of balanced
/// ternary characters: `+`, `0`, and `-`.
///
/// # Arguments
///
/// * `from` - A string slice representing the balanced ternary number.
/// * **Panics** if an input character is invalid.
///
/// # Returns
///
/// A `Ternary` object created from the provided string representation.
///
/// # Example
/// ```
/// use balanced_ternary::{ter, Ternary};
///
/// let ternary = ter("+-0+");
/// assert_eq!(ternary.to_string(), "+-0+");
/// let ternary = "+-0+".parse::<Ternary>().unwrap();
/// assert_eq!(ternary.to_string(), "+-0+");
/// ```
#[cfg(feature = "ternary-string")]
pub fn ter(from: &str) -> Ternary {
    Ternary::parse(from)
}

#[cfg(feature = "tryte")]
/// Creates a `Tryte` object from a string representation of a balanced ternary number.
/// It contains approximately 9.5 bits of information.
///
/// This function first converts the input string representation into a `Ternary` object
/// using the `ter` function, and then constructs a `Tryte` from that `Ternary`.
///
/// # Panics
///
/// This function panics if the `Ternary` contains more than 6 digits or if an input character is invalid.
///
/// # Arguments
///
/// * `from` - A string slice representing the balanced ternary number. It must contain
///   valid balanced ternary characters (`+`, `0`, or `-`) only.
/// * Panics if an input character is invalid.
///
/// # Returns
///
/// A `Tryte` object constructed from the provided balanced ternary string.
///
/// # Example
/// ```
/// use balanced_ternary::{tryte, Tryte};
///
/// let tryte_value = tryte("+0+0");
/// assert_eq!(tryte_value.to_string(), "00+0+0");
/// ```
pub fn tryte(from: &str) -> Tryte {
    // Direct parse: branchless byte→Digit, right-aligned into [Digit; 6].
    // Avoids Ternary::parse() heap allocation + from_ternary() copy.
    // Same branchless formula as Ternary::parse: (b=='+') as i8 - (b=='-') as i8.
    let bytes = from.as_bytes();
    let n = bytes.len().min(6);
    let mut raw = [Digit::Zero; 6];
    let offset = 6 - n;
    // SAFETY: (b==b'+') as i8 - (b==b'-') as i8 ∈ {-1, 0, 1} = valid Digit repr.
    unsafe {
        let dst = raw.as_mut_ptr() as *mut i8;
        for (j, &b) in bytes[..n].iter().enumerate() {
            dst.add(offset + j).write((b == b'+') as i8 - (b == b'-') as i8);
        }
    }
    Tryte::new(raw)
}

/// Creates a `DataTernary` object from a string representation of a balanced ternary number.
///
/// This function converts the provided string representation of a balanced ternary number
/// into a `DataTernary` object. It first parses the input string into a `Ternary`
/// using the `ter` function, and then constructs the `DataTernary`.
///
/// # Arguments
///
/// * `from` - A string slice that contains a valid balanced ternary number.
///   Valid characters are `+`, `0`, and `-`.
///
/// # Panics
///
/// * Panics if the input contains invalid balanced ternary characters.
///
/// # Returns
///
/// A `DataTernary` object constructed from the input string.
///
/// # Example
/// ```
/// use balanced_ternary::{dter, DataTernary};
///
/// let data_ternary = dter("+-0-");
/// assert_eq!(data_ternary.to_string(), "0+-0-");
/// ```
#[cfg(feature = "ternary-store")]
pub fn dter(from: &str) -> DataTernary {
    DataTernary::from_ternary(ter(from))
}

/// Represents a balanced ternary number using a sequence of `Digit`s.
///
/// Provides functions for creating, parsing, converting, and manipulating balanced ternary numbers.
/// Compute one entry of `FROM_DEC_TRIT3_LUT`: expand `v` ∈ 0..=27 into
/// `(carry_out, [d0, d1, d2])` (digits LSB-first) for the 3-trit-at-a-time
/// `from_dec` fast path.
///
/// `v = (x % 243) + carry_in`. carry_in ∈ {0,1}, `x % 243` ∈ 0..242,
/// so `v` ∈ 0..=243. `v` ≥ 122 requires carry_out=1 (5 balanced trits hold
/// −121..121; values ≥ 122 exceed that range and must borrow 243 = 3^5).
#[cfg(feature = "ternary-string")]
const fn from_dec_trit5_entry(v: u8) -> (u8, [Digit; 5]) {
    let carry = if v >= 122 { 1u8 } else { 0u8 };
    let n = if v >= 122 { v as i64 - 243 } else { v as i64 };
    let mut digits = [Digit::Zero; 5];
    let mut rem_n = n;
    let mut i = 0usize;
    while i < 5 {
        // Normalize rem to {0, 1, 2} then map to balanced {0, 1, -1}.
        let rem = ((rem_n % 3) + 3) % 3;
        let trit: i8 = if rem <= 1 { rem as i8 } else { -1 };
        // SAFETY: trit ∈ {-1, 0, 1}.
        digits[i] = unsafe { core::mem::transmute::<i8, Digit>(trit) };
        rem_n = (rem_n - trit as i64) / 3;
        i += 1;
    }
    (carry, digits)
}

/// Precomputed 5-trit balanced ternary expansion for `(x % 243) + carry_in` ∈ 0..=243.
///
/// Indexed by `v = x % 243 + carry_in`. Each entry is `(carry_out, [d0..d4])`
/// where digits are LSB-first. Reduces loop count by ~5× vs a single-trit loop
/// (8 iterations for i64::MAX vs 40), or ~1.6× vs the former 3-trit LUT.
/// 244 entries × 6 bytes = 1464 bytes — fits comfortably in L1 cache.
#[cfg(feature = "ternary-string")]
const FROM_DEC_TRIT5_LUT: [(u8, [Digit; 5]); 244] = {
    let mut lut = [(0u8, [Digit::Zero; 5]); 244];
    let mut v = 0u8;
    loop {
        lut[v as usize] = from_dec_trit5_entry(v);
        if v == 243 { break; }
        v += 1;
    }
    lut
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg(feature = "ternary-string")]
pub struct Ternary {
    digits: Vec<Digit>,
}

#[cfg(feature = "ternary-string")]
impl Ternary {
    /// Creates a new balanced ternary number from a vector of `Digit`s.
    pub fn new(digits: Vec<Digit>) -> Ternary {
        Ternary { digits }
    }

/// Returns the number of digits (length) of the balanced ternary number.
    pub fn log(&self) -> usize {
        self.digits.len()
    }

    /// Retrieves a slice containing the digits of the `Ternary`.
    ///
    /// # Returns
    ///
    /// A slice referencing the digits vec of the `Ternary`.
    ///
    /// This function allows access to the raw representation of the
    /// balanced ternary number as a slice of `Digit` values.
    pub fn to_digit_slice(&self) -> &[Digit] {
        self.digits.as_slice()
    }

    /// Returns an iterator over the digits from most significant to least significant.
    pub fn iter(&self) -> core::slice::Iter<'_, Digit> {
        self.digits.iter()
    }

    /// Returns a reference to the [Digit] indexed by `index` if it exists.
    ///
    /// Digits are indexed **from the right**:
    /// ```
    /// use balanced_ternary::Ternary;
    ///
    /// // Indexes :
    /// //                              32
    /// //                             4||1
    /// //                            5||||0
    /// //                            ||||||
    /// //                            vvvvvv
    /// let ternary = Ternary::parse("+++--+");
    /// assert_eq!(ternary.get_digit(1).unwrap().to_char(), '-')
    /// ```
    pub fn get_digit(&self, index: usize) -> Option<&Digit> {
        self.digits.iter().rev().nth(index)
    }

    /// Parses a string representation of a balanced ternary number into a `Ternary` object.
    ///
    /// Each character in the string must be one of `+`, `0`, or `-`.
    ///
    /// # Example
    /// ```
    /// use balanced_ternary::Ternary;
    ///
    /// let ternary = "+-0".parse::<Ternary>().unwrap();
    /// assert_eq!(ternary.to_string(), "+-0");
    /// ```
    /// # Optimization: pre-allocated vec
    ///
    /// We know the exact digit count from the string length (all valid chars
    /// are single-byte ASCII: `+`, `0`, `-`), so we allocate once upfront.
    pub fn parse(str: &str) -> Self {
        // All valid ternary chars ('+', '-', '0') are single-byte ASCII.
        //
        // # Optimization: branchless byte→Digit + unsafe ptr write
        //
        // `(b == b'+') as i8 - (b == b'-') as i8` maps:
        //   b'+' (43) → 1-0 = 1  = Digit::Pos
        //   b'-' (45) → 0-1 = -1 = Digit::Neg
        //   any other → 0-0 = 0  = Digit::Zero
        // No branches → LLVM auto-vectorizes the entire loop (SIMD byte compares).
        // Unsafe write bypasses Vec::push bounds check to expose the pattern.
        let bytes = str.as_bytes();
        let n = bytes.len();
        let mut digits: Vec<Digit> = Vec::with_capacity(n);
        // SAFETY: Digit is #[repr(i8)] with discriminants {-1, 0, 1}.
        // The expression yields exactly those values, so every write is valid.
        // We write all n elements before set_len(n).
        unsafe {
            let dst = digits.as_mut_ptr() as *mut i8;
            for (i, &b) in bytes.iter().enumerate() {
                dst.add(i).write((b == b'+') as i8 - (b == b'-') as i8);
            }
            digits.set_len(n);
        }
        Ternary::new(digits)
    }

    /// Converts the `Ternary` object to its integer (decimal) representation.
    ///
    /// # Optimization: Horner's method
    ///
    /// The previous implementation computed `digit * 3^rank` per digit,
    /// calling `i64::pow(rank)` which internally uses exponentiation by
    /// squaring — O(log rank) multiplications per digit, O(n log n) total.
    /// It also iterated in reverse (`.rev()`), adding iterator overhead.
    ///
    /// Horner's method evaluates the polynomial MSB-first:
    ///
    /// ```text
    /// value = (((d[0] * 3 + d[1]) * 3 + d[2]) * 3 + ... + d[n-1])
    /// ```
    ///
    /// This is exactly **one multiply + one add per digit**, O(n) total,
    /// and iterates in natural (MSB-first) storage order — no `.rev()`.
    pub fn to_dec(&self) -> i64 {
        let mut dec: i64 = 0;
        for digit in self.digits.iter() {
            dec = dec * 3 + digit.to_i8() as i64;
        }
        dec
    }

    /// Creates a balanced ternary number from a decimal integer.
    ///
    /// The input number is converted into its balanced ternary representation,
    /// with digits represented as `Digit`s.
    ///
    /// # Optimization: direct modular arithmetic
    ///
    /// Instead of converting to a base-3 string via `format_radix` and then
    /// parsing each character back (2 allocations + per-digit char→u8 parse),
    /// we compute balanced ternary digits directly using modular arithmetic:
    ///
    /// - Extract `remainder = x % 3` (values 0, 1, or 2).
    /// - Remainders 0 and 1 map directly to digits `Zero` and `Pos`.
    /// - Remainder 2 is "rebalanced" to digit `Neg` (-1) with a carry of +1
    ///   into the next higher position (since 2 = 3·1 + (-1)).
    /// - For negative inputs, we compute the positive form and negate each
    ///   digit in-place (avoiding a second pass via `-&repr`).
    ///
    /// This eliminates all intermediate string/char allocations and reduces
    /// `from_dec` to a tight loop of integer divmod operations.
    pub fn from_dec(dec: i64) -> Self {
        if dec == 0 {
            return Ternary::new(vec![Zero]);
        }

        let negative = dec < 0;
        let mut x = dec.unsigned_abs();

        // Write quintets into a stack buffer MSB-first (from the end, working
        // backwards), avoiding a post-loop reverse() and drain()-based trim.
        // 45 = 9 groups × 5 trits; i64::MAX needs at most 8 groups (40 trits).
        let mut scratch = [Digit::Zero; 45];
        let mut end = 45usize;
        let mut carry: u64 = 0;
        while x > 0 || carry > 0 {
            let v = (x % 243 + carry) as u8;
            x /= 243;
            let (new_carry, quintet) = FROM_DEC_TRIT5_LUT[v as usize];
            carry = new_carry as u64;
            // quintet is LSB-first; write reversed into scratch for MSB-first.
            // Negate inline for negative inputs — eliminates a second O(n) pass.
            end -= 5;
            if negative {
                scratch[end]     = -quintet[4];
                scratch[end + 1] = -quintet[3];
                scratch[end + 2] = -quintet[2];
                scratch[end + 3] = -quintet[1];
                scratch[end + 4] = -quintet[0];
            } else {
                scratch[end]     = quintet[4];
                scratch[end + 1] = quintet[3];
                scratch[end + 2] = quintet[2];
                scratch[end + 3] = quintet[1];
                scratch[end + 4] = quintet[0];
            }
        }

        // Trim leading zeros introduced by fixed 5-digit groups.
        let start = scratch[end..45].iter().position(|d| *d != Digit::Zero)
            .map(|p| end + p)
            .unwrap_or(44);

        Ternary::new(scratch[start..45].to_vec())
    }

    /// Converts the balanced ternary number to its unbalanced representation as a string.
    ///
    /// The unbalanced representation treats the digits as standard ternary (0, 1, 2),
    /// instead of balanced ternary (-1, 0, +1). Negative digits are handled by
    /// calculating the decimal value of the balanced ternary number and converting
    /// it back to an unbalanced ternary string.
    ///
    /// Returns:
    /// * `String` - The unbalanced ternary representation of the number, where each
    /// digit is one of `0`, `1`, or `2`.
    ///
    /// Example:
    /// ```
    /// use balanced_ternary::Ternary;
    ///
    /// let repr = Ternary::parse("+--");
    /// assert_eq!(repr.to_unbalanced(), "12");
    /// assert_eq!(repr.to_dec(), 5);
    /// let repr = Ternary::parse("-++");
    /// assert_eq!(repr.to_unbalanced(), "-12");
    /// assert_eq!(repr.to_dec(), -5);
    /// ```
    pub fn to_unbalanced(&self) -> String {
        format_radix(self.to_dec(), 3)
    }

    /// Parses a string representation of an unbalanced ternary number into a `Ternary` object.
    ///
    /// The string must only contain characters valid in the unbalanced ternary numeral system (`0`, `1`, or `2`).
    /// Each character is directly converted into its decimal value and then interpreted as a balanced ternary number.
    ///
    /// # Arguments
    ///
    /// * `unbalanced` - A string slice representing the unbalanced ternary number.
    ///
    /// # Returns
    ///
    /// A `Ternary` object representing the same value as the input string in balanced ternary form.
    ///
    /// # Panics
    ///
    /// This function will panic if the string is not a valid unbalanced ternary number.
    /// For instance, if it contains characters other than `0`, `1`, or `2`.
    ///
    /// # Examples
    ///
    /// ```
    /// use balanced_ternary::Ternary;
    ///
    /// let ternary = Ternary::from_unbalanced("-12");
    /// assert_eq!(ternary.to_string(), "-++");
    /// assert_eq!(ternary.to_dec(), -5);
    /// ```
    pub fn from_unbalanced(unbalanced: &str) -> Self {
        Self::from_dec(i64::from_str_radix(unbalanced, 3).unwrap())
    }

    /// Removes leading `Zero` digits from the `Ternary` number, effectively trimming
    /// it down to its simplest representation. The resulting `Ternary` number
    /// will still represent the same value.
    ///
    /// # Returns
    ///
    /// * `Self` - A new `Ternary` object, trimmed of leading zeros.
    ///
    /// # Examples
    ///
    /// ```
    /// use balanced_ternary::{ Neg, Pos, Ternary, Zero};
    ///
    /// let ternary = Ternary::new(vec![Zero, Zero, Pos, Neg]);
    /// let trimmed = ternary.trim();
    /// assert_eq!(trimmed.to_string(), "+-");
    /// ```
    ///
    /// # Notes
    ///
    /// This method does not mutate the original `Ternary` object but returns a new representation.
    /// # Optimization: digit-scan instead of `to_dec()`
    ///
    /// The previous implementation called `self.to_dec() == 0` to detect the
    /// all-zeros case. That performed O(n) multiplications and power-of-3
    /// accumulations just to check a condition that is trivially visible from
    /// the digit array itself. We now use `Iterator::position` to find the
    /// first non-zero digit and slice from there — pure O(n) comparison with
    /// no arithmetic.
    pub fn trim(&self) -> Self {
        match self.digits.iter().position(|d| *d != Zero) {
            None => Ternary::new(vec![Zero]),
            Some(pos) => Ternary::new(self.digits[pos..].to_vec()),
        }
    }

    /// Adjusts the representation of the `Ternary` number to have a fixed number of digits.
    ///
    /// If the current `Ternary` has fewer digits than the specified `length`, leading zero digits
    /// will be added to the `Ternary` to match the desired length. If the current `Ternary` has
    /// more digits than the specified `length`, it will be returned unmodified.
    ///
    /// # Arguments
    ///
    /// * `length` - The desired length of the `Ternary` number.
    ///
    /// # Returns
    ///
    /// * `Self` - A new `Ternary` object with the specified fixed length.
    ///
    /// # Notes
    ///
    /// If `length` is smaller than the existing number of digits, the function does not truncate
    /// the number but instead returns the original `Ternary` unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use balanced_ternary::{Ternary, Zero, Pos};
    ///
    /// let ternary = Ternary::new(vec![Pos]);
    /// let fixed = ternary.with_length(5);
    /// assert_eq!(fixed.to_string(), "0000+");
    ///
    /// let fixed = ternary.with_length(1);
    /// assert_eq!(fixed.to_string(), "+");
    /// ```
    /// # Optimization: single allocation
    ///
    /// The previous implementation allocated a temporary `vec![Zero; diff]`,
    /// then created an empty `Ternary` and extended it twice (zeroes + digits).
    /// We now do one `Vec::with_capacity` and fill it in order — zero padding
    /// followed by the existing digits — avoiding intermediate allocations and
    /// redundant memcpys.
    pub fn with_length(&self, length: usize) -> Self {
        if length <= self.log() {
            return self.clone();
        }
        let pad = length - self.log();
        let mut digits = Vec::with_capacity(length);
        digits.extend(core::iter::repeat(Zero).take(pad));
        digits.extend_from_slice(&self.digits);
        Ternary::new(digits)
    }

    /// Converts the `Ternary` number into a string representation by applying a given
    /// transformation function to each digit of the ternary number.
    ///
    /// # Arguments
    ///
    /// * `transform` - A function or closure that takes a `Digit` and returns a `char`, representing the digit.
    ///
    /// # Returns
    ///
    /// A `String`-based representation of the `Ternary` number resulting from
    /// applying the transformation to its digits.
    ///
    /// # Examples
    ///
    /// ```
    /// use balanced_ternary::{Digit, Pos, Neg, Zero, Ternary};
    ///
    /// let ternary = Ternary::new(vec![Pos, Zero, Neg]);
    ///
    /// let custom_repr = ternary.to_string_repr(Digit::to_char_t);
    /// assert_eq!(custom_repr, "10T");
    /// let custom_repr = ternary.to_string_repr(Digit::to_char_theta);
    /// assert_eq!(custom_repr, "10Θ");
    /// let custom_repr = ternary.to_string_repr(Digit::to_char);
    /// assert_eq!(custom_repr, "+0-");
    /// ```
    ///
    /// # Notes
    ///
    /// * The function provides flexibility to define custom string representations
    ///   for the ternary number digits.
    /// * Call to `Ternary::to_string()` is equivalent to `Ternary::to_string_repr(Digit::to_char)`.
    /// # Optimization: pre-allocated string
    ///
    /// Each digit maps to exactly one ASCII character, so the output length
    /// equals the digit count. Pre-allocating avoids `String` reallocation
    /// during the push loop.
    pub fn to_string_repr<F: Fn(&Digit) -> char>(&self, transform: F) -> String {
        let mut s = String::with_capacity(self.digits.len());
        for digit in self.digits.iter() {
            s.push(transform(digit));
        }
        s
    }

    /// Shifts every trit's **value** one step up the cycle: `-→0`, `0→+`, `+→-`.
    ///
    /// This is the ternary-native SHU↑ operation (no binary equivalent).
    /// It applies [`Digit::post`] to every position. Three consecutive calls
    /// return the original number (period-3 cycle).
    ///
    /// ```
    /// use balanced_ternary::{ter, Ternary};
    ///
    /// let t = ter("-0+");
    /// assert_eq!(t.shu_up().to_string(), "0+-");
    /// assert_eq!(t.shu_up().shu_up().to_string(), "+-0");
    /// assert_eq!(t.shu_up().shu_up().shu_up().to_string(), "-0+");
    /// ```
    /// Returns the sign of this `Ternary` as a single `Digit`.
    ///
    /// Balanced ternary's symmetric digit set means the sign equals the
    /// most significant non-zero trit — no arithmetic required, just a
    /// left-to-right scan.
    ///
    /// - Returns `Pos` if the number is positive.
    /// - Returns `Neg` if the number is negative.
    /// - Returns `Zero` if the number is zero.
    ///
    /// ```
    /// use balanced_ternary::{ter, Digit::{Pos, Neg, Zero}};
    ///
    /// assert_eq!(ter("+0-").signum(), Pos);
    /// assert_eq!(ter("-0+").signum(), Neg);
    /// assert_eq!(ter("000").signum(), Zero);
    /// ```
    pub fn signum(&self) -> Digit {
        self.digits.iter().copied().find(|&d| d != Zero).unwrap_or(Zero)
    }

    /// Returns the absolute value of this `Ternary`.
    ///
    /// In balanced ternary, `abs` is free: if negative, negate (flip every
    /// trit); otherwise clone. No magnitude computation needed.
    ///
    /// ```
    /// use balanced_ternary::{ter, Ternary};
    ///
    /// assert_eq!(ter("-++").abs().to_dec(), 5);
    /// assert_eq!(ter("+--").abs().to_dec(), 5);
    /// assert_eq!(ter("0").abs().to_dec(), 0);
    /// ```
    pub fn abs(&self) -> Self {
        if self.signum() == Neg { -self } else { self.clone() }
    }

    /// Arithmetic (signed) right shift by `n` trit positions: computes `floor(self / 3^n)`.
    ///
    /// Unlike the logical [`Shr`](core::ops::Shr) operator (which truncates toward zero),
    /// this always rounds toward negative infinity — matching the Trillium ISA `srs`
    /// instruction semantics.
    ///
    /// ```
    /// use balanced_ternary::{ter, Ternary};
    ///
    /// assert_eq!(ter("+--").shr_signed(1).to_dec(),  1);  // floor(5/3)  =  1
    /// assert_eq!(ter("-++").shr_signed(1).to_dec(), -2);  // floor(-5/3) = -2
    /// assert_eq!(ter("-+-").shr_signed(1).to_dec(), -3);  // floor(-7/3) = -3
    /// assert_eq!(ter("+--").shr_signed(3).to_dec(),  0);  // floor(5/27) =  0
    /// assert_eq!(ter("-++").shr_signed(3).to_dec(), -1);  // floor(-5/27)= -1
    /// ```
    pub fn shr_signed(&self, n: usize) -> Self {
        if n == 0 {
            return self.clone();
        }
        let digits = self.to_digit_slice();
        if n >= digits.len() {
            return if self.signum() == Neg {
                Ternary::from_dec(-1)
            } else {
                Ternary::new(vec![Zero])
            };
        }
        let split = digits.len() - n;
        let result = Ternary::new(digits[..split].to_vec());
        // Signum of the dropped (lower) digits determines whether we need to
        // subtract 1 to convert from balanced-ternary truncation to floor.
        let dropped_signum = digits[split..]
            .iter()
            .copied()
            .find(|&d| d != Zero)
            .unwrap_or(Zero);
        if dropped_signum == Neg {
            (&result - &Ternary::from_dec(1)).trim()
        } else {
            result.trim()
        }
    }

    /// Clamp every trit toward negative: `min(trit, Zero)`.
    ///
    /// Applies [`Digit::clamp_down`] to every position: `Neg→Neg`, `Zero→Zero`, `Pos→Zero`.
    ///
    /// ```
    /// use balanced_ternary::ter;
    ///
    /// assert_eq!(ter("+-0").clamp_down().to_string(), "0-0");
    /// ```
    pub fn clamp_down(&self) -> Self { self.each(Digit::clamp_down) }

    /// Clamp every trit toward positive: `max(trit, Zero)`.
    ///
    /// Applies [`Digit::clamp_up`] to every position: `Neg→Zero`, `Zero→Zero`, `Pos→Pos`.
    ///
    /// ```
    /// use balanced_ternary::ter;
    ///
    /// assert_eq!(ter("+-0").clamp_up().to_string(), "+00");
    /// ```
    pub fn clamp_up(&self) -> Self { self.each(Digit::clamp_up) }

    /// Indicator map: `Pos` where trit is `Neg`, `Neg` elsewhere.
    ///
    /// Applies [`Digit::is_neg`] to every position.
    ///
    /// ```
    /// use balanced_ternary::ter;
    ///
    /// assert_eq!(ter("+-0").map_neg().to_string(), "-+-");
    /// ```
    pub fn map_neg(&self) -> Self { self.each(Digit::is_neg) }

    /// Indicator map: `Pos` where trit is `Zero`, `Neg` elsewhere.
    ///
    /// Applies [`Digit::is_zero`] to every position.
    ///
    /// ```
    /// use balanced_ternary::ter;
    ///
    /// assert_eq!(ter("+-0").map_zero().to_string(), "--+");
    /// ```
    pub fn map_zero(&self) -> Self { self.each(Digit::is_zero) }

    /// Indicator map: `Pos` where trit is `Pos`, `Neg` elsewhere.
    ///
    /// Applies [`Digit::is_pos`] to every position.
    ///
    /// ```
    /// use balanced_ternary::ter;
    ///
    /// assert_eq!(ter("+-0").map_pos().to_string(), "+--");
    /// ```
    pub fn map_pos(&self) -> Self { self.each(Digit::is_pos) }

    pub fn shu_up(&self) -> Self {
        self.each(Digit::post)
    }

    /// Shifts every trit's **value** one step down the cycle: `+→0`, `0→-`, `-→+`.
    ///
    /// Inverse of [`shu_up`](Self::shu_up). Applies [`Digit::pre`] to every position.
    ///
    /// ```
    /// use balanced_ternary::{ter, Ternary};
    ///
    /// let t = ter("-0+");
    /// assert_eq!(t.shu_down().to_string(), "+-0");
    /// assert_eq!(t.shu_up().shu_down().to_string(), "-0+");
    /// ```
    pub fn shu_down(&self) -> Self {
        self.each(Digit::pre)
    }

    /// Tritwise **consensus**: keeps the value where both numbers agree, `Zero` elsewhere.
    ///
    /// Useful for extracting the positions where two ternary numbers carry the
    /// same information — e.g. range checks or agreement masks.
    ///
    /// ```
    /// use balanced_ternary::{ter, Ternary};
    ///
    /// // a = [+,+,0,0,-,-], b = [+,0,+,0,-,0]
    /// // Agreement at positions 0 (+,+) and 4 (-,-) only
    /// let a = ter("++00--");
    /// let b = ter("+0+0-0");
    /// assert_eq!(a.consensus(&b).to_string(), "+000-0");
    /// ```
    pub fn consensus(&self, other: &Self) -> Self {
        self.each_zip(Digit::consensus, other.clone())
    }

    /// Tritwise **accept-anything** (ANY): non-zero trit wins; conflicting non-zero trits → `Zero`.
    ///
    /// `Zero` positions are transparent (pass-through). This allows lossless
    /// merging of two trytes whose non-zero fields don't overlap:
    ///
    /// ```text
    /// 000--- ANY +++000 = +++---
    /// ```
    ///
    /// ```
    /// use balanced_ternary::{ter, Ternary};
    ///
    /// let a = ter("000---");
    /// let b = ter("+++000");
    /// assert_eq!(a.accept_anything(&b).to_string(), "+++---");
    ///
    /// // Conflict: opposing non-zero trits cancel to zero
    /// let c = ter("+");
    /// let d = ter("-");
    /// assert_eq!(c.accept_anything(&d).to_string(), "0");
    /// ```
    pub fn accept_anything(&self, other: &Self) -> Self {
        self.each_zip(Digit::accept_anything, other.clone())
    }

    /// Encodes this `Ternary` as a **heptavintimal** (base-27) string.
    ///
    /// Heptavintimal groups trits in threes (trybbles), encoding each group as a
    /// single character from the 27-symbol alphabet `0–9 A–H K M N P R T V X Z`
    /// (letters I, J, L, O, Q, S, U, W, Y are omitted to avoid ambiguity with digits).
    ///
    /// Each character represents a value from 0 (`---`) to 26 (`+++`) in balanced
    /// ternary. The number is zero-padded on the left to a multiple of 3 trits first.
    ///
    /// ```
    /// use balanced_ternary::{ter, Ternary};
    ///
    /// // "000" = 0 → unbiased index 0+13=13 → 'D'
    /// assert_eq!(ter("000").to_heptavintimal(), "D");
    /// // "---" = -13 → unbiased index -13+13=0 → '0'
    /// assert_eq!(ter("---").to_heptavintimal(), "0");
    /// // "+++" = +13 → unbiased index 13+13=26 → 'Z'
    /// assert_eq!(ter("+++").to_heptavintimal(), "Z");
    /// // Multi-trybble: "++-" = 9+3-1=11 → 11+13=24 → 'V'; "-++" = -9+3+1=-5 → -5+13=8 → '8'
    /// assert_eq!(ter("++-").concat(&ter("-++")).to_heptavintimal(), "V8");
    /// ```
    pub fn to_heptavintimal(&self) -> String {
        const CHARS: &[u8] = b"0123456789ABCDEFGHKMNPRTVXZ";
        let digits = self.to_digit_slice();
        let pad = (3 - digits.len() % 3) % 3;
        let mut out = String::with_capacity((digits.len() + pad) / 3);
        let mut iter = core::iter::repeat(&Zero).take(pad).chain(digits.iter());
        loop {
            let d0 = match iter.next() { Some(d) => d.to_i8(), None => break };
            let d1 = iter.next().unwrap().to_i8();
            let d2 = iter.next().unwrap().to_i8();
            // Convert balanced trybble to unbalanced 0..26: bias = 13 = 1*9+1*3+1
            let val = (d0 * 9 + d1 * 3 + d2 + 13) as usize;
            out.push(CHARS[val] as char);
        }
        out
    }

    /// Parses a **heptavintimal** (base-27) string into a `Ternary`.
    ///
    /// Inverse of [`to_heptavintimal`](Self::to_heptavintimal).
    /// Each character maps to 3 trits; the result is trimmed of leading zeros.
    ///
    /// Returns `None` if any character is not in the heptavintimal alphabet.
    ///
    /// ```
    /// use balanced_ternary::{ter, Ternary};
    ///
    /// assert_eq!(Ternary::from_heptavintimal("D").unwrap().to_dec(), 0);
    /// assert_eq!(Ternary::from_heptavintimal("0").unwrap().to_dec(), -13);
    /// assert_eq!(Ternary::from_heptavintimal("Z").unwrap().to_dec(), 13);
    /// // Round-trip
    /// let t = ter("+-0+-");
    /// assert_eq!(Ternary::from_heptavintimal(&t.to_heptavintimal()).unwrap().trim().to_string(),
    ///            t.trim().to_string());
    /// ```
    pub fn from_heptavintimal(s: &str) -> Option<Self> {
        // Map each hept character to its 0..26 value
        let char_val = |c: char| -> Option<i8> {
            match c {
                '0'..='9' => Some(c as i8 - '0' as i8),
                'A'..='H' => Some(c as i8 - 'A' as i8 + 10),
                'K' => Some(19), 'M' => Some(20), 'N' => Some(21),
                'P' => Some(22), 'R' => Some(23), 'T' => Some(24),
                'V' => Some(25), 'X' => Some(26), 'Z' => Some(27 - 1),
                // Accept lowercase input too
                'a'..='h' => Some(c as i8 - 'a' as i8 + 10),
                'k' => Some(19), 'm' => Some(20), 'n' => Some(21),
                'p' => Some(22), 'r' => Some(23), 't' => Some(24),
                'v' => Some(25), 'x' => Some(26), 'z' => Some(26),
                _ => None,
            }
        };
        let mut digits = Vec::with_capacity(s.len() * 3);
        for c in s.chars() {
            let v = char_val(c)? as i8 - 13; // unbiased: -13..=13
            // Decompose balanced trybble (v = d0*9 + d1*3 + d2)
            let r2 = ((v % 3) + 3) % 3;
            let t2 = if r2 <= 1 { r2 } else { r2 - 3 };
            let v1 = (v - t2) / 3;
            let r1 = ((v1 % 3) + 3) % 3;
            let t1 = if r1 <= 1 { r1 } else { r1 - 3 };
            let t0 = (v1 - t1) / 3;
            // SAFETY: t0, t1, t2 ∈ {-1, 0, 1}
            digits.push(unsafe { core::mem::transmute::<i8, Digit>(t0) });
            digits.push(unsafe { core::mem::transmute::<i8, Digit>(t1) });
            digits.push(unsafe { core::mem::transmute::<i8, Digit>(t2) });
        }
        Some(Ternary::new(digits).trim())
    }

    /// Concatenates the current `Ternary` number with another `Ternary` number.
    ///
    /// This function appends the digits of the provided `Ternary` object to the digits
    /// of the current `Ternary` object, creating a new `Ternary` number as the result.
    ///
    /// # Arguments
    ///
    /// * `other` - A reference to the `Ternary` number to be concatenated to the current one.
    ///
    /// # Returns
    ///
    /// * `Ternary` - A new `Ternary` object formed by concatenating the digits.
    ///
    /// # Examples
    ///
    /// ```
    /// use balanced_ternary::{Ternary, Pos, Zero, Neg};
    ///
    /// let ternary1 = Ternary::new(vec![Pos, Zero]);
    /// let ternary2 = Ternary::new(vec![Neg, Pos]);
    ///
    /// let concatenated = ternary1.concat(&ternary2);
    /// assert_eq!(concatenated.to_string(), "+0-+");
    /// ```
    /// # Optimization: pre-allocated vec
    ///
    /// We know the exact output length (sum of both digit counts), so we
    /// allocate once and copy both slices. `extend_from_slice` is a single
    /// `memcpy` for `Copy` types like `Digit`.
    pub fn concat(&self, other: &Ternary) -> Ternary {
        let mut digits = Vec::with_capacity(self.digits.len() + other.digits.len());
        digits.extend_from_slice(&self.digits);
        digits.extend_from_slice(&other.digits);
        Ternary::new(digits)
    }
}

#[cfg(feature = "ternary-string")]
impl DigitOperate for Ternary {
    fn to_digits(&self) -> Vec<Digit> {
        self.to_digit_slice().to_vec()
    }

    fn digit(&self, index: usize) -> Option<Digit> {
        self.get_digit(index).cloned()
    }

    /// # Optimization: pre-allocated vec + unsafe ptr write
    ///
    /// Output length is known upfront (same as input). Single allocation.
    /// Bypasses `Vec::push` bounds check so LLVM can auto-vectorize when
    /// `f` compiles to branchless arithmetic (e.g. `post`, `pre`, `not`).
    fn each(&self, f: impl Fn(Digit) -> Digit) -> Self {
        let n = self.digits.len();
        let mut out: Vec<Digit> = Vec::with_capacity(n);
        // SAFETY: we write exactly `n` elements before set_len(n).
        unsafe {
            let out_ptr = out.as_mut_ptr();
            for (i, &d) in self.digits.iter().enumerate() {
                out_ptr.add(i).write(f(d));
            }
            out.set_len(n);
        }
        Ternary::new(out)
    }

    /// # Optimization: pre-allocated vec
    fn each_with(&self, f: impl Fn(Digit, Digit) -> Digit, other: Digit) -> Self {
        Ternary::new(self.digits.iter().map(|&d| f(d, other)).collect())
    }

    /// # Optimization: O(n²) → O(n), zero allocations beyond the result
    ///
    /// The previous implementation had three performance problems:
    ///
    /// 1. **`get_digit(i)` is O(i)** — it calls `.iter().rev().nth(i)` which
    ///    walks `i` elements each time. Inside a loop of `n` iterations this
    ///    gives O(n²) total work.
    /// 2. **`with_length` allocation** — pads the shorter operand by allocating
    ///    a new `Ternary` with leading zeros.
    /// 3. **Result reversal** — digits were pushed in reverse order then
    ///    `reverse()`d at the end.
    ///
    /// We now iterate MSB-first using offset arithmetic to virtually pad the
    /// shorter operand with leading zeros (same trick as `Ord::cmp`). This
    /// gives a single O(n) forward pass with one pre-sized allocation and
    /// no reversal.
    fn each_zip(&self, f: impl Fn(Digit, Digit) -> Digit, other: Self) -> Self {
        let a = &self.digits;
        // Fast path: equal-length operands (the common case).
        //
        // # Optimization: reuse `other`'s allocation
        //
        // `other` is owned (callers pass `rhs.clone()`), so we mutate it
        // in-place rather than collecting into a fresh Vec. This eliminates
        // one heap allocation + one deallocation per call.
        if a.len() == other.digits.len() {
            let mut result = other;
            for (rb, &da) in result.digits.iter_mut().zip(a.iter()) {
                *rb = f(da, *rb);
            }
            return result;
        }
        let b = &other.digits;
        let len = a.len().max(b.len());
        let (oa, ob) = (len - a.len(), len - b.len());
        let mut digits = Vec::with_capacity(len);
        for i in 0..len {
            let da = if i < oa { Zero } else { a[i - oa] };
            let db = if i < ob { Zero } else { b[i - ob] };
            digits.push(f(da, db));
        }
        Ternary::new(digits)
    }

    /// # Optimization: O(n²) → O(n), same approach as `each_zip`
    ///
    /// Same three problems as `each_zip` (see above), plus the carry must
    /// propagate right-to-left so we still iterate in reverse — but we use
    /// direct indexing instead of `get_digit(i)`, eliminating the O(i)
    /// inner walk. One `reverse()` remains (unavoidable for carry direction).
    fn each_zip_carry(
        &self,
        f: impl Fn(Digit, Digit, Digit) -> (Digit, Digit),
        other: Self,
    ) -> Self {
        let a = &self.digits;
        // Fast path: equal-length — reuse `other`'s allocation, write in-place
        // right-to-left. No reverse() needed: `i` is already the MSB-first index.
        if a.len() == other.digits.len() {
            let n = a.len();
            let mut result = other;
            let mut carry = Zero;
            for i in (0..n).rev() {
                let (c, res) = f(a[i], result.digits[i], carry);
                carry = c;
                result.digits[i] = res;
            }
            return result;
        }
        let b = &other.digits;
        let len = a.len().max(b.len());
        let (oa, ob) = (len - a.len(), len - b.len());
        // Pre-size the Vec and write directly at each MSB-first index,
        // eliminating the push-then-reverse pass.
        // SAFETY: Digit is Copy + !Drop; all `len` elements are written below.
        let mut digits = Vec::with_capacity(len);
        unsafe { digits.set_len(len); }
        let mut carry = Zero;
        for i in (0..len).rev() {
            let da = if i < oa { Zero } else { a[i - oa] };
            let db = if i < ob { Zero } else { b[i - ob] };
            let (c, res) = f(da, db, carry);
            carry = c;
            digits[i] = res;
        }
        Ternary::new(digits)
    }
}

#[cfg(feature = "ternary-string")]
impl Display for Ternary {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // For typical sizes (≤ 64 trits) write the chars via a stack byte
        // buffer + single write_str call — avoids a heap `String` allocation
        // that `to_string_repr` would introduce.
        const MAX_STACK: usize = 64;
        let n = self.digits.len();
        if n <= MAX_STACK {
            // MaybeUninit avoids zero-initializing the 64-byte buffer.
            let mut buf = core::mem::MaybeUninit::<[u8; MAX_STACK]>::uninit();
            let ptr = buf.as_mut_ptr() as *mut u8;
            for (i, d) in self.digits.iter().enumerate() {
                // SAFETY: i < n ≤ MAX_STACK; to_byte() returns valid ASCII.
                unsafe { ptr.add(i).write(d.to_byte()); }
            }
            // SAFETY: buf[..n] initialised above; all bytes are valid ASCII.
            let s = unsafe { core::str::from_utf8_unchecked(
                core::slice::from_raw_parts(ptr, n)
            )};
            f.write_str(s)
        } else {
            for d in self.digits.iter() {
                FmtWrite::write_char(f, d.to_char())?;
            }
            Ok(())
        }
    }
}

#[cfg(feature = "ternary-string")]
impl FromStr for Ternary {
    type Err = ParseTernaryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.bytes().all(|b| matches!(b, b'+' | b'0' | b'-')) {
            Ok(Ternary::parse(s))
        } else {
            Err(ParseTernaryError)
        }
    }
}

/// Cold path for [`Ternary::cmp`] when operands have unequal length.
///
/// Isolated into a `#[cold] #[inline(never)]` function so LLVM places it
/// outside the hot code region, keeping `cmp`'s hot equal-length path
/// compact and consistently cache-line-aligned regardless of surrounding
/// code changes.
#[cold]
#[inline(never)]
fn cmp_unequal_len(a: &[Digit], b: &[Digit]) -> Ordering {
    let len = a.len().max(b.len());
    let (oa, ob) = (len - a.len(), len - b.len());
    for i in 0..len {
        let da = if i < oa { 0i8 } else { a[i - oa].to_i8() };
        let db = if i < ob { 0i8 } else { b[i - ob].to_i8() };
        match da.cmp(&db) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    Ordering::Equal
}

#[cfg(feature = "ternary-string")]
impl Ord for Ternary {
    /// # Optimization: lexicographic digit comparison
    ///
    /// Balanced ternary has a useful property: lexicographic ordering of
    /// digits (MSB-first, with `Neg < Zero < Pos`) equals numerical ordering.
    ///
    /// **Why this works:** each trit position has weight `3^k`. The maximum
    /// contribution of all lower-order trits combined is
    /// `sum_{i=0}^{k-1} 3^i = (3^k - 1) / 2`, which is strictly less than
    /// `3^k`. So a difference in a higher-order trit always dominates,
    /// making MSB-first lexicographic comparison correct.
    ///
    /// The previous implementation called `to_dec()` on both operands,
    /// performing O(n) multiplications each. This version does a single
    /// O(n) pass of integer comparisons with no arithmetic beyond subtraction.
    fn cmp(&self, other: &Self) -> Ordering {
        let (a, b) = (&self.digits, &other.digits);
        // Hot path: equal-length operands.
        // `Digit` is `#[repr(i8)]` with Neg=-1 < Zero=0 < Pos=1, so
        // `[Digit]::cmp` is lexicographic ternary order — LLVM can
        // lower this to a vectorized signed byte comparison.
        if a.len() == b.len() {
            return a.as_slice().cmp(b.as_slice());
        }
        cmp_unequal_len(a, b)
    }
}

#[cfg(feature = "ternary-string")]
impl PartialOrd for Ternary {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(feature = "ternary-string")]
impl IntoIterator for Ternary {
    type Item = Digit;
    type IntoIter = alloc::vec::IntoIter<Digit>;

    fn into_iter(self) -> Self::IntoIter {
        self.digits.into_iter()
    }
}

#[cfg(feature = "ternary-string")]
mod operations;

mod conversions;

#[cfg(feature = "ternary-store")]
mod store;

#[cfg(feature = "ternary-store")]
pub use crate::store::{BctTer32, IlBctTer32, Ter40, IlTer40, DataTernary, TritsChunk,
                        UTer9, UTer27, BTer9, BTer27};

#[cfg(feature = "tryte")]
mod tryte;

#[cfg(feature = "tryte")]
pub use crate::tryte::Tryte;

#[cfg(feature = "terscii")]
pub mod terscii;


#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_ternary() {
    use alloc::string::ToString;
    use crate::*;

    let repr5 = Ternary::new(vec![Pos, Neg, Neg]);
    assert_eq!(repr5.to_dec(), 5);
    let repr5 = Ternary::from_dec(5);
    assert_eq!(repr5.to_dec(), 5);

    let repr13 = Ternary::new(vec![Pos, Pos, Pos]);
    assert_eq!(repr13.to_dec(), 13);

    let repr14 = Ternary::parse("+---");
    let repr15 = Ternary::parse("+--0");
    assert_eq!(repr14.to_dec(), 14);
    assert_eq!(repr15.to_dec(), 15);
    assert_eq!(repr14.to_string(), "+---");
    assert_eq!(repr15.to_string(), "+--0");

    let repr120 = Ternary::from_dec(120);
    assert_eq!(repr120.to_dec(), 120);
    assert_eq!(repr120.to_string(), "++++0");
    let repr121 = Ternary::from_dec(121);
    assert_eq!(repr121.to_dec(), 121);
    assert_eq!(repr121.to_string(), "+++++");

    let repr_neg_5 = Ternary::parse("-++");
    assert_eq!(repr_neg_5.to_dec(), -5);
    assert_eq!(repr_neg_5.to_string(), "-++");

    let repr_neg_5 = Ternary::from_dec(-5);
    assert_eq!(repr_neg_5.to_dec(), -5);
    assert_eq!(repr_neg_5.to_string(), "-++");

    let repr_neg_121 = Ternary::from_dec(-121);
    assert_eq!(repr_neg_121.to_dec(), -121);
    assert_eq!(repr_neg_121.to_string(), "-----");

    let test = Ternary::from_dec(18887455);
    assert_eq!(test.to_dec(), 18887455);
    assert_eq!(test.to_string(), "++00--0--+-0++0+");

    let unbalanced = Ternary::from_unbalanced("12");
    assert_eq!(unbalanced.to_dec(), 5);
    assert_eq!(unbalanced.to_string(), "+--");

    let unbalanced = Ternary::from_unbalanced("-12");
    assert_eq!(unbalanced.to_dec(), -5);
    assert_eq!(unbalanced.to_string(), "-++");

    let unbalanced = Ternary::from_dec(121);
    assert_eq!(unbalanced.to_unbalanced(), "11111");
    assert_eq!(unbalanced.to_string(), "+++++");
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_each() {
    use alloc::string::ToString;
    use crate::*;
    let ternary = Ternary::parse("+0-");
    assert_eq!(ternary.each(Digit::possibly).to_string(), "++-");
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_operations() {
    use alloc::string::ToString;
    fn test_ternary_eq(a: Ternary, b: &str) {
        let repr = Ternary::parse(b);
        assert_eq!(a.to_string(), repr.to_string());
    }
    fn test_binary_op(a: &Ternary, f: impl Fn(Digit, Digit) -> Digit, b: &Ternary, c: &str) {
        test_ternary_eq(a.each_zip(f, b.clone()), c);
    }

    use core::ops::{BitAnd, BitOr, BitXor, Mul, Not};

    let short = Ternary::parse("-0+");
    let long = Ternary::parse("---000+++");
    let other = Ternary::parse("-0+-0+-0+");

    // K3
    test_ternary_eq(short.each(Digit::not), "+0-");
    test_binary_op(&long, Digit::bitand, &other, "----00-0+");
    test_binary_op(&long, Digit::bitor, &other, "-0+00++++");
    test_binary_op(&long, Digit::bitxor, &other, "-0+000+0-");
    test_binary_op(&long, Digit::k3_equiv, &other, "+0-000-0+");
    test_binary_op(&long, Digit::k3_imply, &other, "+++00+-0+");

    // HT
    test_ternary_eq(short.each(Digit::ht_not), "+--");
    test_binary_op(&long, Digit::ht_imply, &other, "+++-++-0+");

    // BI3
    test_binary_op(&long, Digit::bi3_and, &other, "-0-000-0+");
    test_binary_op(&long, Digit::bi3_or, &other, "-0+000+0+");
    test_binary_op(&long, Digit::bi3_imply, &other, "+0+000-0+");

    // L3
    test_ternary_eq(short.each(Digit::possibly), "-++");
    test_ternary_eq(short.each(Digit::necessary), "--+");
    test_ternary_eq(short.each(Digit::contingently), "-+-");
    test_binary_op(&long, Digit::l3_imply, &other, "+++0++-0+");

    // PARA / RM3
    test_binary_op(&long, Digit::rm3_imply, &other, "+++-0+--+");
    test_binary_op(&long, Digit::para_imply, &other, "+++-0+-0+");

    // Other operations
    test_ternary_eq(short.each(Digit::post), "0+-");
    test_ternary_eq(short.each(Digit::pre), "+-0");
    test_ternary_eq(short.each(Digit::absolute_positive), "+0+");
    test_ternary_eq(short.each(Digit::positive), "00+");
    test_ternary_eq(short.each(Digit::not_negative), "0++");
    test_ternary_eq(short.each(Digit::not_positive), "--0");
    test_ternary_eq(short.each(Digit::negative), "-00");
    test_ternary_eq(short.each(Digit::absolute_negative), "-0-");

    test_binary_op(&long, Digit::mul, &other, "+0-000-0+");
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_from_str() {
    use alloc::string::ToString;
    use core::str::FromStr;

    let ternary = Ternary::from_str("+-0").unwrap();
    assert_eq!(ternary.to_string(), "+-0");

    assert!(Ternary::from_str("+-x").is_err());

    #[cfg(feature = "tryte")]
    {
        let tryte = <crate::Tryte>::from_str("+-0").unwrap();
        assert_eq!(tryte.to_string(), "000+-0");
        assert!(<crate::Tryte>::from_str("+-x").is_err());
    }
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_ordering() {
    use crate::ter;

    assert!(ter("-+") < ter("0"));
    assert!(ter("0") < ter("++"));
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_ordering_additional() {
    use crate::ter;

    // Validate comparisons across a range of values
    assert!(ter("--") < ter("-0"));
    assert!(ter("-0") < ter("-"));
    assert!(ter("+") < ter("+-"));
    assert!(ter("+-") < ter("++"));

    // Sorting should arrange values by their decimal value
    let mut values = vec![ter("+"), ter("--"), ter("+-"), ter("-"), ter("0"), ter("-0"), ter("++")];
    values.sort();
    let expected = vec![ter("--"), ter("-0"), ter("-"), ter("0"), ter("+"), ter("+-"), ter("++")];
    assert_eq!(values, expected);
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_iterators() {
    use crate::*;

    let ternary = Ternary::parse("+0-");
    let expected = vec![Pos, Zero, Neg];
    assert_eq!(ternary.iter().cloned().collect::<Vec<_>>(), expected);
    let collected: Vec<Digit> = Ternary::parse("+0-").into_iter().collect();
    assert_eq!(collected, expected);
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_shu_up_down() {
    use alloc::string::ToString;
    use crate::ter;

    // SHU↑: each trit cycles -→0→+→- (Digit::post)
    let t = ter("-0+");
    assert_eq!(t.shu_up().to_string(), "0+-");
    assert_eq!(t.shu_up().shu_up().to_string(), "+-0");
    // Three shu_up steps = identity
    assert_eq!(t.shu_up().shu_up().shu_up().to_string(), "-0+");

    // SHU↓ is the inverse of SHU↑
    assert_eq!(t.shu_down().to_string(), "+-0");
    assert_eq!(t.shu_up().shu_down().to_string(), "-0+");

    // Two shu_up = one shu_down (period 3)
    assert_eq!(t.shu_up().shu_up().to_string(), t.shu_down().to_string());
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_consensus() {
    use alloc::string::ToString;
    use crate::ter;

    // Positions that agree survive; all others become 0
    // a = [+,+,0,0,-,-], b = [+,0,+,0,-,0]
    // Agreement at pos 0 (+,+) and pos 4 (-,-) only
    let a = ter("++00--");
    let b = ter("+0+0-0");
    assert_eq!(a.consensus(&b).to_string(), "+000-0");

    // Full agreement = identity
    assert_eq!(a.consensus(&a).to_string(), a.to_string());

    // Full disagreement = all zeros (opposite signs at every non-zero position)
    let neg_a = ter("--00++");
    assert_eq!(a.consensus(&neg_a).to_string(), "000000");
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_accept_anything() {
    use alloc::string::ToString;
    use crate::ter;

    // Non-overlapping fields merge losslessly (vine ANY trick)
    let a = ter("000---");
    let b = ter("+++000");
    assert_eq!(a.accept_anything(&b).to_string(), "+++---");

    // Zero is transparent (pass-through); full width preserved
    let z = ter("000000");
    assert_eq!(a.accept_anything(&z).to_string(), a.to_string());

    // Conflict: opposing non-zero trits cancel
    let pos = ter("+");
    let neg = ter("-");
    assert_eq!(pos.accept_anything(&neg).to_string(), "0");

    // Self-merge = identity
    assert_eq!(a.accept_anything(&a).to_string(), a.to_string());
}

#[cfg(test)]
#[cfg(feature = "ternary-string")]
#[test]
fn test_shr_signed() {
    use crate::ter;

    // Positive: floor(5/3) = 1
    assert_eq!(ter("+--").shr_signed(1).to_dec(), 1);
    // Negative: floor(-5/3) = -2
    assert_eq!(ter("-++").shr_signed(1).to_dec(), -2);
    // Negative with negative remainder: floor(-7/3) = -3
    assert_eq!(ter("-+-").shr_signed(1).to_dec(), -3);
    // Exact division: floor(6/3) = 2
    assert_eq!(ter("+-0").shr_signed(1).to_dec(), 2);
    // Shift of zero
    assert_eq!(ter("0").shr_signed(1).to_dec(), 0);
    // Large shift of positive: floor(5/27) = 0
    assert_eq!(ter("+--").shr_signed(3).to_dec(), 0);
    // Large shift of negative: floor(-5/27) = -1
    assert_eq!(ter("-++").shr_signed(3).to_dec(), -1);
    // Zero shift is identity
    assert_eq!(ter("+--").shr_signed(0).to_dec(), 5);
    // Multi-trit shift: floor(22/9) = 2
    assert_eq!(ter("+-++").shr_signed(2).to_dec(), 2);
}
