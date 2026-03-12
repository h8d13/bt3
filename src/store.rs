use crate::concepts::DigitOperate;
use crate::{Digit, Ternary};
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::Display;
use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Not, Shl, Shr, Sub};

/// Compute the 5-trit balanced ternary expansion of `val` ∈ [-121, 121].
///
/// Used at compile time to build `EXPAND_DIGITS_LUT`.
const fn expand_digits_compute(val: i8) -> [Digit; 5] {
    let negative = val < 0;
    let mut unsigned = if negative { (-(val as i16)) as u16 } else { val as u16 };
    let mut buf = [Digit::Zero; 5];
    let mut i = 5usize;
    while i > 0 {
        i -= 1;
        let rem = unsigned % 3;
        unsigned /= 3;
        if rem == 2 {
            buf[i] = Digit::Neg;
            unsigned += 1;
        } else {
            // SAFETY: rem ∈ {0, 1} ⊂ {-1, 0, 1}.
            buf[i] = unsafe { core::mem::transmute::<i8, Digit>(rem as i8) };
        }
    }
    if negative {
        let mut j = 0usize;
        while j < 5 {
            // SAFETY: negating a valid Digit i8 value yields another valid Digit value.
            buf[j] = unsafe { core::mem::transmute::<i8, Digit>(-(buf[j] as i8)) };
            j += 1;
        }
    }
    buf
}

/// Precomputed 5-trit expansion for all valid `TritsChunk` values [-121, 121].
///
/// Indexed by `val + 121` (i.e., index 0 = value -121, index 121 = value 0,
/// index 242 = value 121). Replaces the per-call division loop in `expand_digits`
/// with a single indexed copy of 5 bytes.
const EXPAND_DIGITS_LUT: [[Digit; 5]; 243] = {
    let mut lut = [[Digit::Zero; 5]; 243];
    let mut i: i32 = -121;
    while i <= 121 {
        lut[(i + 121) as usize] = expand_digits_compute(i as i8);
        i += 1;
    }
    lut
};

/// A struct to store 5 ternary digits (~7.8 bits) value into one byte.
///
/// `TritsChunks` helps store ternary numbers into a compact memory structure.
///
/// From `0` to `± 121`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct TritsChunk(i8);

impl TritsChunk {
    /// Creates a `TritsChunk` from a given decimal value.
    ///
    /// # Arguments
    ///
    /// * `from` - An `i8` value representing the decimal value to be converted into a `TritsChunk`.
    ///
    /// # Panics
    ///
    /// This function panics if the input value is out of the valid range `-121..=121`.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::TritsChunk;
    ///
    /// let chunk = TritsChunk::from_dec(42);
    /// assert_eq!(chunk.to_dec(), 42);
    /// ```
    pub fn from_dec(from: i8) -> Self {
        if !(-121..=121).contains(&from) {
            panic!("TritsChunk::from_dec(): Invalid value: {}", from);
        }
        Self(from)
    }

    /// Converts the `TritsChunk` into its decimal representation.
    ///
    /// # Returns
    ///
    /// An `i8` value representing the decimal form of the `TritsChunk`.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::TritsChunk;
    ///
    /// let chunk = TritsChunk::from_dec(42);
    /// assert_eq!(chunk.to_dec(), 42);
    /// ```
    pub fn to_dec(&self) -> i8 {
        self.0
    }

    /// Converts the `TritsChunk` into its ternary representation.
    ///
    /// # Returns
    ///
    /// A `Ternary` type representing the ternary form of the `TritsChunk`.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{TritsChunk, Ternary};
    ///
    /// let chunk = TritsChunk::from_dec(42);
    /// let ternary = chunk.to_ternary();
    /// assert_eq!(ternary.to_dec(), 42);
    /// ```
    /// # Optimization: direct 5-digit balanced ternary expansion
    ///
    /// The previous implementation called `Ternary::from_dec(self.0 as i64)`
    /// which allocates a `Vec`, computes digits via divmod, and reverses.
    /// We expand the i8 value into a fixed `[Digit; 5]` buffer directly,
    /// then wrap it in `Ternary` — avoiding the generic from_dec overhead
    /// for this known-small range.
    pub fn to_ternary(&self) -> Ternary {
        Ternary::new(Self::expand_digits(self.0).to_vec()).trim()
    }

    /// Converts the `TritsChunk` into its fixed-length ternary representation.
    ///
    /// # Returns
    ///
    /// A `Ternary` type representing the 5-digit fixed-length ternary form of the `TritsChunk`.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{TritsChunk, Ternary};
    ///
    /// let chunk = TritsChunk::from_dec(42);
    /// let fixed_ternary = chunk.to_fixed_ternary();
    /// assert_eq!(fixed_ternary.to_dec(), 42);
    /// assert_eq!(fixed_ternary.to_digit_slice().len(), 5);
    /// ```
    pub fn to_fixed_ternary(&self) -> Ternary {
        Ternary::new(Self::expand_digits(self.0).to_vec())
    }

    /// Converts the `TritsChunk` into a vector of its individual ternary digits.
    ///
    /// # Returns
    ///
    /// A `Vec<Digit>` representing the individual ternary digits of the `TritsChunk`.
    ///
    /// The resulting vector will always contain 5 digits since the `TritsChunk` is
    /// represented in a fixed-length ternary form.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{TritsChunk, Digit};
    ///
    /// let chunk = TritsChunk::from_dec(42);
    /// let digits: Vec<Digit> = chunk.to_digits();
    /// assert_eq!(digits.len(), 5);
    /// ```
    /// # Optimization: direct expansion without Ternary intermediary
    ///
    /// The previous implementation went through `to_fixed_ternary()` →
    /// `Ternary::from_dec` (alloc + divmod + reverse) → `with_length(5)`
    /// (second alloc) → `to_digit_slice().to_vec()` (third alloc).
    /// We now expand directly into a `[Digit; 5]` stack buffer and copy
    /// once to `Vec`.
    pub fn to_digits(&self) -> Vec<Digit> {
        Self::expand_digits(self.0).to_vec()
    }

    /// Creates a `TritsChunk` from a given `Ternary` value.
    ///
    /// # Arguments
    ///
    /// * `ternary` - A `Ternary` value to be converted into a `TritsChunk`.
    ///
    /// # Panics
    ///
    /// This function panics if the provided `ternary` value has a logarithmic length greater than 5,
    /// indicating that it cannot be represented by a single `TritsChunk`.
    ///
    /// Expand an i8 chunk value into 5 balanced ternary digits.
    ///
    /// Shared helper used by `to_ternary`, `to_fixed_ternary`, and
    /// `to_digits` to avoid going through `Ternary::from_dec`. The value
    /// range [-121, 121] always fits in exactly 5 trits.
    /// Expands `val` ∈ [-121, 121] into its 5-trit balanced ternary representation.
    ///
    /// # Optimization: precomputed LUT
    ///
    /// The previous implementation ran a 5-iteration division loop plus a conditional
    /// negation pass. Since the domain is exactly 243 values, we precompute all
    /// expansions at compile time in `EXPAND_DIGITS_LUT` (1215 bytes) and replace
    /// the entire computation with a single indexed 5-byte copy.
    #[inline]
    fn expand_digits(val: i8) -> [Digit; 5] {
        EXPAND_DIGITS_LUT[(val as i16 + 121) as usize]
    }

    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{TritsChunk, Ternary};
    ///
    /// let ternary = Ternary::from_dec(42);
    /// let chunk = TritsChunk::from_ternary(ternary);
    /// assert_eq!(chunk.to_dec(), 42);
    /// ```
    pub fn from_ternary(ternary: Ternary) -> Self {
        if ternary.log() > 5 {
            panic!(
                "TritsChunk::from_ternary(): Ternary is too long: {}",
                ternary.to_string()
            );
        }
        Self(ternary.to_dec() as i8)
    }
}

/// Offers a compact structure to store a ternary number.
///
/// - A [Ternary] is 1 byte long per [Digit]. An 8 (16, 32, 64) digits ternary number is 8 (16, 32, 64) bytes long.
/// - A [DataTernary] is stored into [TritsChunk]. An 8 (16, 32, 64) digits ternary number with this structure is 2 (4, 7, 13) bytes long (1 byte for 5 digits).
///
/// Use the [Ternary] type to execute operations on numbers and [DataTernary] to store numbers.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DataTernary {
    chunks: Vec<TritsChunk>,
}

impl DataTernary {
    /// Creates a new instance of `DataTernary` from a given `Ternary` value.
    ///
    /// This method ensures that the total number of ternary digits is a multiple of 5
    /// by padding as necessary. It then divides the ternary number into chunks of
    /// 5 digits each, which are stored in the `DataTernary` structure.
    ///
    /// # Arguments
    ///
    /// * `ternary` - A `Ternary` value to be converted into a `DataTernary`.
    ///
    /// # Returns
    ///
    /// A new `DataTernary` instance containing the converted chunks.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{DataTernary, Ternary};
    ///
    /// let ternary = Ternary::from_dec(42);
    /// let data_ternary = DataTernary::from_ternary(ternary);
    /// assert_eq!(data_ternary.to_dec(), 42);
    /// ```
    /// # Optimization: direct slice→i8 without intermediate Ternary
    ///
    /// The previous implementation created a `Ternary::new(digits)` per
    /// 5-digit slice and then called `from_ternary → to_dec()` on it.
    /// Each of those allocated a `Vec` and computed powers of 3.
    ///
    /// We now compute each chunk's `i8` value directly from the digit
    /// slice using positional weights `[81, 27, 9, 3, 1]` (powers of 3
    /// for a 5-trit chunk). No intermediate allocations.
    pub fn from_ternary(ternary: Ternary) -> Self {
        let len = ternary.log();
        let partial = len % 5; // digits in the leading partial chunk (0 = full)
        let slice = ternary.to_digit_slice();
        let num_chunks = (len + 4) / 5;

        const WEIGHTS: [i8; 5] = [81, 27, 9, 3, 1]; // used only for partial leading chunk

        let mut chunks = Vec::with_capacity(num_chunks);

        // Leading partial chunk: use only the last `partial` weights so that
        // the high-order trit positions are implicitly zero-padded. Eliminates
        // the `with_length` clone that the previous code required.
        let start = if partial != 0 {
            let first = &slice[..partial];
            let val: i8 = first
                .iter()
                .zip(WEIGHTS[5 - partial..].iter())
                .map(|(d, &w)| d.to_i8() * w)
                .sum();
            chunks.push(TritsChunk::from_dec(val));
            partial
        } else {
            0
        };

        // Remaining full 5-digit chunks: direct indexing lets LLVM eliminate bounds
        // checks (chunks_exact guarantees len=5) and skip the from_dec range check.
        let tail = &slice[start..];
        for c in tail.chunks_exact(5) {
            let val = c[0].to_i8() * 81
                    + c[1].to_i8() * 27
                    + c[2].to_i8() * 9
                    + c[3].to_i8() * 3
                    + c[4].to_i8();
            // SAFETY: val ∈ [-121, 121] since each trit ∈ {-1,0,1} and
            // max(|81+27+9+3+1|) = 121.
            chunks.push(TritsChunk(val));
        }

        Self { chunks }
    }

    /// Converts a `DataTernary` into its equivalent `Ternary` representation.
    ///
    /// This function iterates over all the `TritsChunk` instances in the `DataTernary`,
    /// extracts their ternary representations, and reconstructs them into the full
    /// `Ternary` value. The resulting `Ternary` value may be trimmed to remove
    /// any leading zeroes in its ternary digit representation.
    ///
    /// # Returns
    ///
    /// A `Ternary` value that represents the combined ternary digits of the
    /// `DataTernary`.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{DataTernary, Ternary};
    ///
    /// let ternary = Ternary::from_dec(42);
    /// let data_ternary = DataTernary::from_ternary(ternary.clone());
    /// assert_eq!(data_ternary.to_ternary(), ternary);
    /// ```
    /// # Optimization: inline chunk→digit expansion
    ///
    /// The previous implementation called `chunk.to_ternary()` per chunk,
    /// which went through `Ternary::from_dec` (allocates a `Vec`, builds
    /// digits) and then `.to_digit_slice()` (borrows the vec). For `n`
    /// chunks that's `n` temporary `Ternary` allocations.
    ///
    /// We now expand each chunk's `i8` value directly into 5 balanced
    /// ternary digits using the same modular arithmetic as `from_dec`,
    /// writing straight into the final output `Vec`. The chunk's value
    /// range `[-121, 121]` fits in 5 trits, so 5 iterations per chunk
    /// always suffice.
    /// Uses `TritsChunk::expand_digits` to convert each chunk directly
    /// into 5 digits on the stack, avoiding per-chunk heap allocations.
    pub fn to_ternary(&self) -> Ternary {
        let mut digits = Vec::with_capacity(self.chunks.len() * 5);
        for chunk in &self.chunks {
            digits.extend_from_slice(&TritsChunk::expand_digits(chunk.to_dec()));
        }
        // Trim leading zeros in-place — avoids the second Vec allocation that
        // `Ternary::new(digits).trim()` would incur.
        let first_nz = digits.iter().position(|d| *d != Digit::Zero);
        match first_nz {
            None => { digits.truncate(0); digits.push(Digit::Zero); }
            Some(0) => {}
            Some(pos) => { digits.drain(0..pos); }
        }
        Ternary::new(digits)
    }

    /// Converts the `DataTernary` into its fixed-length `Ternary` representation.
    ///
    /// This method iterates over all the `TritsChunk` instances in the `DataTernary` and
    /// extracts and combines their ternary digits into a single `Ternary` value.
    /// The resulting `Ternary` value will contain a fixed number of digits without trimming
    /// or removing leading zeroes.
    ///
    /// # Returns
    ///
    /// A `Ternary` value representing the combined fixed-length ternary digits of the `DataTernary`.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{DataTernary, Ternary};
    ///
    /// let ternary = Ternary::from_dec(42);
    /// let data_ternary = DataTernary::from_ternary(ternary);
    /// let fixed_ternary = data_ternary.to_fixed_ternary();
    /// assert_eq!(fixed_ternary.to_dec(), 42); // When properly encoded
    /// ```
    /// Reuses `to_ternary()` which already builds digits directly from chunks.
    pub fn to_fixed_ternary(&self) -> Ternary {
        self.to_ternary()
    }

    /// Converts the `DataTernary` into a vector of ternary digits.
    ///
    /// This method first converts the `DataTernary` structure into its `Ternary` representation,
    /// trims any leading zeroes, and then returns the sequence of ternary digits as a `Vec<Digit>`.
    ///
    /// # Returns
    ///
    /// A `Vec<Digit>` containing the ternary digits that represent the `DataTernary` value.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{DataTernary, Digit, Ternary};
    ///
    /// let ternary = Ternary::from_dec(42);
    /// let data_ternary = DataTernary::from_ternary(ternary);
    /// let digits = data_ternary.to_digits();
    /// assert_eq!(digits, vec![Digit::Pos, Digit::Neg, Digit::Neg, Digit::Neg, Digit::Zero]);
    /// ```
    pub fn to_digits(&self) -> Vec<Digit> {
        self.to_ternary().to_digit_slice().to_vec()
    }

    /// Converts a decimal number into a `DataTernary` structure.
    ///
    /// This method takes a signed 64-bit integer as input and converts it into a
    /// `Ternary` representation, which is then stored in the compact `DataTernary`
    /// structure. The conversion ensures that the ternary representation uses
    /// fixed-length chunks for efficient storage.
    ///
    /// # Arguments
    ///
    /// * `from` - A signed 64-bit integer value to be converted into `DataTernary`.
    ///
    /// # Returns
    ///
    /// A `DataTernary` instance that represents the given decimal number.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{DataTernary};
    ///
    /// let data_ternary = DataTernary::from_dec(42);
    /// assert_eq!(data_ternary.to_dec(), 42);
    /// ```
    pub fn from_dec(from: i64) -> Self {
        if from == 0 {
            return Self { chunks: alloc::vec![TritsChunk::from_dec(0)] };
        }
        // Decompose into balanced base-243 (3^5) chunks directly, no Ternary alloc.
        let mut n = from;
        let mut chunks = alloc::vec::Vec::with_capacity(8);
        while n != 0 {
            let rem = ((n % 243) + 243) % 243;
            let val: i8 = if rem <= 121 { rem as i8 } else { (rem - 243) as i8 };
            n = (n - val as i64) / 243;
            chunks.push(TritsChunk::from_dec(val));
        }
        chunks.reverse(); // built LSB-first, store MSB-first
        Self { chunks }
    }

    /// Converts a `DataTernary` into its decimal representation.
    ///
    /// This method reconstructs the ternary value represented by the `DataTernary`
    /// structure and converts it into the corresponding signed 64-bit decimal integer.
    ///
    /// # Returns
    ///
    /// A signed 64-bit integer (`i64`) representing the decimal equivalent of the
    /// `DataTernary` structure.
    ///
    /// # Example
    ///
    /// ```
    /// use balanced_ternary::{DataTernary};
    ///
    /// let data_ternary = DataTernary::from_dec(42);
    /// let decimal = data_ternary.to_dec();
    /// assert_eq!(decimal, 42);
    /// ```
    /// # Optimization: direct chunk arithmetic
    ///
    /// The previous implementation routed through `to_ternary()` which
    /// expanded every chunk into a 5-digit `Ternary`, concatenated them,
    /// trimmed leading zeros, and *then* called `to_dec()` with per-digit
    /// power-of-3 accumulation.
    ///
    /// Each `TritsChunk` already stores its decimal value as an `i8`.
    /// A `DataTernary` with `n` chunks represents:
    ///
    /// ```text
    /// value = sum_{k=0}^{n-1} chunk[n-1-k].to_dec() * 243^k
    /// ```
    ///
    /// where `243 = 3^5` (the weight of 5 balanced ternary digits).
    /// We evaluate this directly with Horner's method — a single loop of
    /// multiply-accumulate on `i64` — avoiding all Ternary allocations.
    pub fn to_dec(&self) -> i64 {
        // Horner's method: ((c0 * 243 + c1) * 243 + c2) * 243 + ...
        // Chunks are stored MSB-first (chunk[0] is the highest-weight chunk).
        let mut acc: i64 = 0;
        for chunk in &self.chunks {
            acc = acc * 243 + chunk.to_dec() as i64;
        }
        acc
    }
}

impl Display for DataTernary {
    /// Uses `TritsChunk::expand_digits` → `Digit::to_char` per digit,
    /// avoiding any Ternary allocation.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for chunk in &self.chunks {
            for d in &TritsChunk::expand_digits(chunk.to_dec()) {
                f.write_fmt(format_args!("{}", d.to_char()))?;
            }
        }
        Ok(())
    }
}

impl From<Ternary> for DataTernary {
    fn from(value: Ternary) -> Self {
        Self::from_ternary(value)
    }
}

impl From<DataTernary> for Ternary {
    fn from(value: DataTernary) -> Self {
        value.to_ternary()
    }
}

// ---------------------------------------------------------------------------
// Ter40 IL-u128 fast-path constants and helpers
// ---------------------------------------------------------------------------

/// Low bits of each 2-bit pair for 40 trits (bits 0,2,4,...,78 in u128).
const MASK_L40: u128 = 0x0000_0000_0000_5555_5555_5555_5555_5555;

/// Bias so balanced ternary v maps to unsigned 40-trit n = v + BIAS40 ≥ 0.
/// BIAS40 = (3^40 − 1) / 2 = 6_078_832_729_528_464_400.
const TER40_BIAS: i64 = 6_078_832_729_528_464_400;

/// 5-trit IL encode table: CHUNK5[k] = IL word for unsigned 5-trit value k (k ∈ 0..243).
/// Each trit digit d ∈ {0,1,2} maps directly to the same 2-bit IL code (0→00,1→01,2→10).
const CHUNK5: [u16; 243] = {
    let mut t = [0u16; 243];
    let mut k = 0usize;
    while k < 243 {
        let d0 = (k % 3) as u16;
        let d1 = ((k / 3) % 3) as u16;
        let d2 = ((k / 9) % 3) as u16;
        let d3 = ((k / 27) % 3) as u16;
        let d4 = (k / 81) as u16;
        t[k] = d0 | (d1 << 2) | (d2 << 4) | (d3 << 6) | (d4 << 8);
        k += 1;
    }
    t
};

/// 10-bit IL decode table: CHUNK5_DEC[code] = balanced decimal value of 5 trits
/// encoded in the 10-bit IL `code`.  Invalid 2-bit pairs (0b11) → value 0.
const CHUNK5_DEC: [i8; 1024] = {
    let mut t = [0i8; 1024];
    let mut k = 0usize;
    while k < 1024 {
        let c0 = (k & 3) as i8;
        let c1 = ((k >> 2) & 3) as i8;
        let c2 = ((k >> 4) & 3) as i8;
        let c3 = ((k >> 6) & 3) as i8;
        let c4 = ((k >> 8) & 3) as i8;
        if c0 < 3 && c1 < 3 && c2 < 3 && c3 < 3 && c4 < 3 {
            t[k] = (c0 - 1) + (c1 - 1) * 3 + (c2 - 1) * 9 + (c3 - 1) * 27 + (c4 - 1) * 81;
        }
        k += 1;
    }
    t
};

/// Split-point for the 20/20 trit encoding: 3^20 = 3_486_784_401.
const TER40_B20: u64 = 3_486_784_401;

/// Encode a balanced-ternary i64 as a 40-trit IL u128.
///
/// Splits `n` into two u32 halves (20 trits each), then runs 4 iterations of
/// divmod-by-243 on each half — the two chains are independent within the loop.
#[inline]
fn i64_to_il40(v: i64) -> u128 {
    let n  = v.wrapping_add(TER40_BIAS) as u64;
    let lo = (n % TER40_B20) as u32;
    let hi = (n / TER40_B20) as u32;
    // Explicit unroll: each divisor (1, 243, 59049, 14348907) is an independent
    // reciprocal-multiply on lo/hi — no serial chain, OOO can schedule all 8 in parallel.
    let wl = (CHUNK5[(lo % 243)               as usize] as u64)
           | ((CHUNK5[((lo /       243) % 243) as usize] as u64) << 10)
           | ((CHUNK5[((lo /    59_049) % 243) as usize] as u64) << 20)
           | ((CHUNK5[( lo / 14_348_907)       as usize] as u64) << 30);
    let wh = (CHUNK5[(hi % 243)               as usize] as u64)
           | ((CHUNK5[((hi /       243) % 243) as usize] as u64) << 10)
           | ((CHUNK5[((hi /    59_049) % 243) as usize] as u64) << 20)
           | ((CHUNK5[( hi / 14_348_907)       as usize] as u64) << 30);
    (wl as u128) | ((wh as u128) << 40)
}

/// Encode two balanced-ternary i64 values as 40-trit IL u128s simultaneously.
///
/// Each 20-trit half is split into four 5-trit groups by dividing the base value
/// by 1, 243, 59049, 14348907 directly — all independent reciprocal-multiplies on
/// the same register.  An OOO CPU can schedule all 16 divisions in parallel across
/// the four quarter-values (alo, ahi, blo, bhi) instead of the former depth-4
/// serial chain per quarter.
#[inline(always)]
fn i64_pair_to_il40(va: i64, vb: i64) -> (u128, u128) {
    let na  = va.wrapping_add(TER40_BIAS) as u64;
    let nb  = vb.wrapping_add(TER40_BIAS) as u64;
    let alo = (na % TER40_B20) as u32;
    let ahi = (na / TER40_B20) as u32;
    let blo = (nb % TER40_B20) as u32;
    let bhi = (nb / TER40_B20) as u32;
    macro_rules! enc {
        ($x:expr) => {
            (CHUNK5[($x % 243)               as usize] as u64)
          | ((CHUNK5[(($x /       243) % 243) as usize] as u64) << 10)
          | ((CHUNK5[(($x /    59_049) % 243) as usize] as u64) << 20)
          | ((CHUNK5[( $x / 14_348_907)       as usize] as u64) << 30)
        }
    }
    ((enc!(alo) as u128) | ((enc!(ahi) as u128) << 40),
     (enc!(blo) as u128) | ((enc!(bhi) as u128) << 40))
}

/// Decode a 40-trit IL u128 back to a balanced-ternary i64.
///
/// Uses Estrin's scheme: reduces the sequential multiply chain from depth 8 (Horner)
/// to depth 3 (log₂ 8).  All 8 table lookups are independent and execute in parallel
/// before the 3-level accumulation.
#[inline]
fn il40_to_i64(w: u128) -> i64 {
    // All 8 chunk extractions are independent.
    let d0 = CHUNK5_DEC[((w      ) & 0x3FF) as usize] as i64;
    let d1 = CHUNK5_DEC[((w >> 10) & 0x3FF) as usize] as i64;
    let d2 = CHUNK5_DEC[((w >> 20) & 0x3FF) as usize] as i64;
    let d3 = CHUNK5_DEC[((w >> 30) & 0x3FF) as usize] as i64;
    let d4 = CHUNK5_DEC[((w >> 40) & 0x3FF) as usize] as i64;
    let d5 = CHUNK5_DEC[((w >> 50) & 0x3FF) as usize] as i64;
    let d6 = CHUNK5_DEC[((w >> 60) & 0x3FF) as usize] as i64;
    let d7 = CHUNK5_DEC[((w >> 70) & 0x3FF) as usize] as i64;
    // Estrin's scheme: 3 levels of depth instead of 8.
    // Level 1 — 4 independent pairs:
    let v0 = d0 + d1 * 243_i64;
    let v1 = d2 + d3 * 243_i64;
    let v2 = d4 + d5 * 243_i64;
    let v3 = d6 + d7 * 243_i64;
    // Level 2 — 2 independent pairs:
    let w0 = v0 + v1 * 59_049_i64;       // 59_049 = 243²
    let w1 = v2 + v3 * 59_049_i64;
    // Level 3:
    w0 + w1 * 3_486_784_401_i64          // 3_486_784_401 = 243⁴
}

/// O(1) trit-wise negation on 40-trit IL u128.
#[inline(always)]
const fn il40_neg(a: u128) -> u128 {
    let h = (a >> 1) & MASK_L40;
    let l =  a       & MASK_L40;
    let new_h = !(h | l) & MASK_L40;
    let new_l = !h & l & MASK_L40;
    (new_h << 1) | new_l
}

/// O(1) trit-wise AND (min) on 40-trit IL u128.
#[inline(always)]
const fn il40_and(a: u128, b: u128) -> u128 {
    let ha = (a >> 1) & MASK_L40; let la = a & MASK_L40;
    let hb = (b >> 1) & MASK_L40; let lb = b & MASK_L40;
    let hr = ha & hb;
    let lr = (la & (hb | lb)) | (ha & lb);
    (hr << 1) | (lr & MASK_L40)
}

/// O(1) trit-wise OR (max) on 40-trit IL u128.
#[inline(always)]
const fn il40_or(a: u128, b: u128) -> u128 {
    let ha = (a >> 1) & MASK_L40; let la = a & MASK_L40;
    let hb = (b >> 1) & MASK_L40; let lb = b & MASK_L40;
    let hr = ha | hb;
    let lr = !(ha | hb) & (la | lb) & MASK_L40;
    (hr << 1) | lr
}

/// O(1) trit-wise XOR (−a·b) on 40-trit IL u128.
#[inline(always)]
const fn il40_xor(a: u128, b: u128) -> u128 {
    let ha = (a >> 1) & MASK_L40; let la = a & MASK_L40;
    let hb = (b >> 1) & MASK_L40; let lb = b & MASK_L40;
    let neg_a = MASK_L40 & !ha & !la;
    let neg_b = MASK_L40 & !hb & !lb;
    let new_h = (ha & neg_b) | (neg_a & hb);
    let is_neg = (ha & hb) | (neg_a & neg_b);
    let new_l = MASK_L40 & !new_h & !is_neg;
    (new_h << 1) | new_l
}

/// O(1) trit-wise consensus on 40-trit IL u128.
#[inline(always)]
const fn il40_consensus(a: u128, b: u128) -> u128 {
    let ha = (a >> 1) & MASK_L40; let la = a & MASK_L40;
    let hb = (b >> 1) & MASK_L40; let lb = b & MASK_L40;
    let neg_a = MASK_L40 & !ha & !la;
    let neg_b = MASK_L40 & !hb & !lb;
    let new_h = ha & hb;
    let is_neg = neg_a & neg_b;
    let new_l = MASK_L40 & !new_h & !is_neg;
    (new_h << 1) | new_l
}

/// O(1) trit-wise accept-anything on 40-trit IL u128.
#[inline(always)]
const fn il40_accept_anything(a: u128, b: u128) -> u128 {
    let ha = (a >> 1) & MASK_L40; let la = a & MASK_L40;
    let hb = (b >> 1) & MASK_L40; let lb = b & MASK_L40;
    let neg_a = MASK_L40 & !ha & !la;
    let neg_b = MASK_L40 & !hb & !lb;
    let not_neg_a = ha | la;
    let not_neg_b = hb | lb;
    let not_pos_a = (MASK_L40 & !ha) | la;
    let not_pos_b = (MASK_L40 & !hb) | lb;
    let new_h = (ha & not_neg_b) | (hb & not_neg_a);
    let is_neg = (neg_a & not_pos_b) | (neg_b & not_pos_a);
    let new_l = MASK_L40 & !new_h & !is_neg;
    (new_h << 1) | new_l
}

/// A struct to store 40 ternary digits (~63.398 bits) value into one `i64`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Ter40(i64);

impl Ter40 {
    pub fn from_dec(from: i64) -> Self {
        Self(from)
    }
    pub fn to_dec(&self) -> i64 {
        self.0
    }
    pub fn from_ternary(ternary: Ternary) -> Self {
        Self(ternary.to_dec())
    }
    pub fn to_ternary(&self) -> Ternary {
        Ternary::from_dec(self.0).with_length(40)
    }

    /// Decomposes the stored i64 into a stack `[Digit; 40]` — zero allocations.
    ///
    /// Uses the same balanced-ternary decomposition as `Tryte::from_i64`:
    /// normalize remainder to {0,1,2} then map to balanced {0,1,−1}.
    #[inline]
    fn to_raw(self) -> [Digit; 40] {
        let mut raw = [Digit::Zero; 40];
        if self.0 == 0 { return raw; }
        let negative = self.0 < 0;
        let mut x = self.0.unsigned_abs();
        // SAFETY: trit ∈ {-1, 0, 1} in both branches.
        if !negative {
            for i in (0..40).rev() {
                let rem = (x % 3) as u8;
                raw[i] = unsafe { core::mem::transmute::<i8, Digit>(
                    if rem == 2 { -1i8 } else { rem as i8 }
                )};
                x = (x - rem as u64) / 3 + (rem == 2) as u64;
            }
        } else {
            for i in (0..40).rev() {
                let rem = (x % 3) as u8;
                raw[i] = unsafe { core::mem::transmute::<i8, Digit>(
                    if rem == 2 { 1i8 } else { -(rem as i8) }
                )};
                x = (x - rem as u64) / 3 + (rem == 2) as u64;
            }
        }
        raw
    }

    /// Recomposes a `[Digit; 40]` into an i64 via Horner's method — zero allocations.
    #[inline]
    fn from_raw(raw: [Digit; 40]) -> Self {
        let mut val: i64 = 0;
        for d in &raw {
            val = val * 3 + d.to_i8() as i64;
        }
        Self(val)
    }

    /// Trit-wise consensus: agrees when both nonzero and equal, else Zero.
    #[inline]
    pub fn consensus(self, other: Self) -> Self {
        let (ia, ib) = i64_pair_to_il40(self.0, other.0);
        Self(il40_to_i64(il40_consensus(ia, ib)))
    }

    /// Trit-wise accept-anything: nonzero wins over zero; conflict → zero.
    #[inline]
    pub fn accept_anything(self, other: Self) -> Self {
        let (ia, ib) = i64_pair_to_il40(self.0, other.0);
        Self(il40_to_i64(il40_accept_anything(ia, ib)))
    }
}

impl DigitOperate for Ter40 {
    fn to_digits(&self) -> Vec<Digit> {
        self.to_raw().to_vec()
    }

    fn digit(&self, index: usize) -> Option<Digit> {
        if index < 40 { Some(self.to_raw()[39 - index]) } else { None }
    }

    /// # Optimization: zero-allocation stack path
    ///
    /// The previous implementation round-tripped through `to_ternary()` (heap
    /// alloc) → `each()` → `to_dec()`. We now decompose to a `[Digit; 40]`
    /// stack array, apply `f` in-place, and recompose via Horner's method —
    /// zero heap allocations.
    fn each(&self, f: impl Fn(Digit) -> Digit) -> Self
    where
        Self: Sized,
    {
        let mut raw = self.to_raw();
        for d in raw.iter_mut() {
            *d = f(*d);
        }
        Self::from_raw(raw)
    }

    fn each_with(&self, f: impl Fn(Digit, Digit) -> Digit, other: Digit) -> Self
    where
        Self: Sized,
    {
        let mut raw = self.to_raw();
        for d in raw.iter_mut() {
            *d = f(*d, other);
        }
        Self::from_raw(raw)
    }

    fn each_zip(&self, f: impl Fn(Digit, Digit) -> Digit, other: Self) -> Self
    where
        Self: Sized,
    {
        let a = self.to_raw();
        let b = other.to_raw();
        let mut raw = [Digit::Zero; 40];
        for i in 0..40 {
            raw[i] = f(a[i], b[i]);
        }
        Self::from_raw(raw)
    }

    fn each_zip_carry(&self, f: impl Fn(Digit, Digit, Digit) -> (Digit, Digit), other: Self) -> Self
    where
        Self: Sized,
    {
        let a = self.to_raw();
        let b = other.to_raw();
        let mut raw = [Digit::Zero; 40];
        let mut carry = Digit::Zero;
        for i in (0..40).rev() {
            let (c, res) = f(a[i], b[i], carry);
            carry = c;
            raw[i] = res;
        }
        Self::from_raw(raw)
    }
}

impl Display for Ter40 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_ternary())
    }
}

impl Add for Ter40 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self(self.0 + other.0)
    }
}
impl Sub for Ter40 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Self(self.0 - other.0)
    }
}
impl Mul for Ter40 {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self(self.0 * other.0)
    }
}
impl Div for Ter40 {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        Self(self.0 / other.0)
    }
}

impl Neg for Ter40 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl BitAnd for Ter40 {
    type Output = Self;
    #[inline]
    fn bitand(self, other: Self) -> Self::Output {
        // Fused pair encode: one loop processes both operands simultaneously so
        // the CPU can overlap the two independent divmod chains.
        let (ia, ib) = i64_pair_to_il40(self.0, other.0);
        Self(il40_to_i64(il40_and(ia, ib)))
    }
}

impl BitOr for Ter40 {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self::Output {
        let (ia, ib) = i64_pair_to_il40(self.0, other.0);
        Self(il40_to_i64(il40_or(ia, ib)))
    }
}

impl BitXor for Ter40 {
    type Output = Self;
    #[inline]
    fn bitxor(self, other: Self) -> Self::Output {
        let (ia, ib) = i64_pair_to_il40(self.0, other.0);
        Self(il40_to_i64(il40_xor(ia, ib)))
    }
}

impl From<i64> for Ter40 {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<Ter40> for i64 {
    fn from(value: Ter40) -> Self {
        value.0
    }
}

impl From<Ternary> for Ter40 {
    fn from(value: Ternary) -> Self {
        Self::from_ternary(value)
    }
}

impl From<Ter40> for Ternary {
    fn from(value: Ter40) -> Self {
        value.to_ternary()
    }
}

#[cfg(test)]
#[test]
fn ter40_each_roundtrip() {
    use crate::concepts::DigitOperate;
    // each(neg) should negate every trit — value must flip sign.
    let t = Ter40::from_dec(42);
    assert_eq!(t.each(|d| -d).to_dec(), -42);
    assert_eq!(Ter40::from_dec(0).each(|d| -d).to_dec(), 0);
}

#[cfg(test)]
#[test]
fn ter40_each_zip_bitwise() {
    use crate::Ternary;
    // a = "++-" = 9+3-1 = 11,  b = "+-+" = 9-3+1 = 7
    let a = Ter40::from_ternary(Ternary::parse("++-"));
    let b = Ter40::from_ternary(Ternary::parse("+-+"));

    // AND = min: min(+,+)=+, min(+,-)=-, min(-,+)=-  → "+--" = 5
    assert_eq!((a & b).to_ternary().trim().to_string(), "+--");
    // OR  = max: max(+,+)=+, max(+,-)=+, max(-,+)=+  → "+++" = 13
    assert_eq!((a | b).to_ternary().trim().to_string(), "+++");
    // XOR = -(a·b): -(+·+)=-, -(+·-)=+, -(-·+)=+    → "-++" = -5
    assert_eq!((a ^ b).to_ternary().trim().to_string(), "-++");
}

#[cfg(test)]
#[test]
fn ter40_fast_logical_ops() {
    // Verify the IL-u128 fast path gives the same results as the digit-loop path.
    let values: &[i64] = &[0, 1, -1, 13, -5, 12_345_678, -9_876_543, 999_999_999, -1_000_000_000];
    for &a_v in values {
        for &b_v in values {
            let a = Ter40::from_dec(a_v);
            let b = Ter40::from_dec(b_v);
            use crate::concepts::DigitOperate;
            use crate::Digit;
            // AND
            let expected_and = a.each_zip(Digit::bitand, b).to_dec();
            assert_eq!((a & b).to_dec(), expected_and, "AND({a_v},{b_v})");
            // OR
            let expected_or = a.each_zip(Digit::bitor, b).to_dec();
            assert_eq!((a | b).to_dec(), expected_or, "OR({a_v},{b_v})");
            // XOR
            let expected_xor = a.each_zip(Digit::bitxor, b).to_dec();
            assert_eq!((a ^ b).to_dec(), expected_xor, "XOR({a_v},{b_v})");
            // consensus
            let expected_con = a.each_zip(Digit::consensus, b).to_dec();
            assert_eq!(a.consensus(b).to_dec(), expected_con, "consensus({a_v},{b_v})");
            // accept_anything
            let expected_aa = a.each_zip(Digit::accept_anything, b).to_dec();
            assert_eq!(a.accept_anything(b).to_dec(), expected_aa, "accept_anything({a_v},{b_v})");
        }
    }
}

#[cfg(test)]
#[test]
fn ter40_to_digits_length() {
    use crate::concepts::DigitOperate;
    // to_digits always returns exactly 40 elements.
    assert_eq!(Ter40::from_dec(0).to_digits().len(), 40);
    assert_eq!(Ter40::from_dec(1).to_digits().len(), 40);
    assert_eq!(Ter40::from_dec(-1).to_digits().len(), 40);
}

#[cfg(test)]
#[test]
fn ter40_digit_index() {
    use crate::concepts::DigitOperate;
    use crate::Digit::{Neg, Pos};
    // +00-  stored as Ter40 — LSB (index 0) is '-', MSB (index 3) is '+'
    use crate::Ternary;
    let t = Ter40::from_ternary(Ternary::parse("+00-"));
    assert_eq!(t.digit(0).unwrap(), Neg);  // least significant
    assert_eq!(t.digit(3).unwrap(), Pos);  // most significant (of non-zero part)
    assert!(t.digit(40).is_none());
}

#[cfg(test)]
#[test]
fn single_chunk_creation() {
    use crate::Ternary;

    let ternary = Ternary::parse("+-0-+");
    let data = DataTernary::from_ternary(ternary.clone());

    assert_eq!(data.chunks.len(), 1);
    assert_eq!(data.to_ternary(), ternary);
}

#[cfg(test)]
#[test]
fn round_trip() {
    use crate::Ternary;

    let ternary = Ternary::parse("+0-0++-");
    let data = DataTernary::from_ternary(ternary.clone());

    assert_eq!(data.to_ternary(), ternary);
}

// ============================================================================
// IlTer40 — 40 balanced ternary trits in one IL u128
// ============================================================================

/// A struct to store 40 balanced ternary trits in a single interleaved u128.
///
/// ## Storage format
///
/// Each trit occupies a 2-bit pair (same IL encoding as [`IlBctTer32`]):
///
/// | Bit pair | Trit value |
/// |----------|------------|
/// | `00`     | −1 (`Neg`) |
/// | `01`     | 0 (`Zero`) |
/// | `10`     | +1 (`Pos`) |
///
/// Trit 0 (LST) at bits (1,0); trit 39 (MST) at bits (79,78).  Bits 80–127 are
/// always zero and must not be set.
///
/// ## Trade-offs vs [`Ter40`]
///
/// | Operation            | `Ter40` (i64 storage) | `IlTer40` (IL u128 storage) |
/// |----------------------|-----------------------|-----------------------------|
/// | Logical AND/OR/XOR   | ~15 ns (CHUNK5+Estrin)| ~0.5 ns (pure bitwise)      |
/// | Arithmetic +/−/×/÷   | ~0.4 ns (direct i64)  | ~15 ns (decode+op+encode)   |
/// | Construction         | O(1)                  | O(1) encode                 |
///
/// Choose `IlTer40` when trit-logical operations dominate; choose `Ter40` when
/// arithmetic dominates.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct IlTer40(u128);

impl IlTer40 {
    /// All 40 trits = Zero.  Bit pattern: `01` × 40.
    pub const ZERO: Self = Self(MASK_L40);
    /// All 40 trits = +1 (Pos).  Bit pattern: `10` × 40.
    pub const MAX:  Self = Self(MASK_L40 << 1);
    /// All 40 trits = −1 (Neg).  Bit pattern: `00` × 40.
    pub const MIN:  Self = Self(0);

    /// Construct directly from the raw IL u128 word.
    #[inline(always)]
    pub const fn from_raw(word: u128) -> Self { Self(word) }

    /// Return the raw IL u128 word.
    #[inline(always)]
    pub const fn raw(self) -> u128 { self.0 }

    /// Encode a balanced-ternary i64 as 40 IL trits.
    ///
    /// Uses the CHUNK5 split-u32 trick: O(1) — 4 iterations of u32 divmod-by-243.
    #[inline]
    pub fn from_dec(v: i64) -> Self { Self(i64_to_il40(v)) }

    /// Decode the 40 IL trits back to a balanced-ternary i64.
    ///
    /// Uses Estrin's scheme for the Horner accumulation: O(1) — 3 multiply levels.
    #[inline]
    pub fn to_dec(self) -> i64 { il40_to_i64(self.0) }

    // -------------------------------------------------------------------------
    // O(1) trit-wise logical operations
    // -------------------------------------------------------------------------

    /// Trit-wise negation: `Pos↔Neg`, `Zero→Zero`.
    #[inline(always)] pub const fn il_neg(self)              -> Self { Self(il40_neg(self.0)) }
    /// Trit-wise AND (min).
    #[inline(always)] pub const fn il_and(self, o: Self)     -> Self { Self(il40_and(self.0, o.0)) }
    /// Trit-wise OR (max).
    #[inline(always)] pub const fn il_or (self, o: Self)     -> Self { Self(il40_or (self.0, o.0)) }
    /// Trit-wise XOR (−a·b).
    #[inline(always)] pub const fn il_xor(self, o: Self)     -> Self { Self(il40_xor(self.0, o.0)) }
    /// Trit-wise consensus: agrees when both nonzero and equal, else Zero.
    #[inline(always)] pub const fn il_consensus(self, o: Self) -> Self { Self(il40_consensus(self.0, o.0)) }
    /// Trit-wise accept-anything: nonzero wins over zero; conflict → zero.
    #[inline(always)] pub const fn il_accept_anything(self, o: Self) -> Self { Self(il40_accept_anything(self.0, o.0)) }
}

impl Neg for IlTer40 {
    type Output = Self;
    #[inline(always)] fn neg(self) -> Self { self.il_neg() }
}
impl Not for IlTer40 {
    type Output = Self;
    #[inline(always)] fn not(self) -> Self { self.il_neg() }
}
impl BitAnd for IlTer40 {
    type Output = Self;
    #[inline(always)] fn bitand(self, o: Self) -> Self { self.il_and(o) }
}
impl BitOr for IlTer40 {
    type Output = Self;
    #[inline(always)] fn bitor(self, o: Self) -> Self { self.il_or(o) }
}
impl BitXor for IlTer40 {
    type Output = Self;
    #[inline(always)] fn bitxor(self, o: Self) -> Self { self.il_xor(o) }
}

impl Add for IlTer40 {
    type Output = Self;
    #[inline] fn add(self, o: Self) -> Self { Self::from_dec(self.to_dec() + o.to_dec()) }
}
impl Sub for IlTer40 {
    type Output = Self;
    #[inline] fn sub(self, o: Self) -> Self { Self::from_dec(self.to_dec() - o.to_dec()) }
}
impl Mul for IlTer40 {
    type Output = Self;
    #[inline] fn mul(self, o: Self) -> Self { Self::from_dec(self.to_dec() * o.to_dec()) }
}
impl Div for IlTer40 {
    type Output = Self;
    #[inline] fn div(self, o: Self) -> Self { Self::from_dec(self.to_dec() / o.to_dec()) }
}

impl From<i64>   for IlTer40 { fn from(v: i64)   -> Self { Self::from_dec(v) } }
impl From<IlTer40> for i64   { fn from(t: IlTer40) -> Self { t.to_dec() } }
impl From<Ter40> for IlTer40 { fn from(t: Ter40)  -> Self { Self::from_dec(t.to_dec()) } }
impl From<IlTer40> for Ter40 { fn from(t: IlTer40)-> Self { Ter40::from_dec(t.to_dec()) } }

#[cfg(test)]
#[test]
fn ilter40_logical_and_arith() {
    let values: &[i64] = &[0, 1, -1, 13, -5, 12_345_678, -9_876_543, 999_999_999];
    for &a_v in values {
        for &b_v in values {
            let a = IlTer40::from_dec(a_v);
            let b = IlTer40::from_dec(b_v);
            let ta = Ter40::from_dec(a_v);
            let tb = Ter40::from_dec(b_v);
            // Logical ops must match Ter40 (which is verified against digit-loop)
            assert_eq!((a & b).to_dec(), (ta & tb).to_dec(), "AND({a_v},{b_v})");
            assert_eq!((a | b).to_dec(), (ta | tb).to_dec(), "OR({a_v},{b_v})");
            assert_eq!((a ^ b).to_dec(), (ta ^ tb).to_dec(), "XOR({a_v},{b_v})");
            assert_eq!(a.il_consensus(b).to_dec(), ta.consensus(tb).to_dec(), "consensus");
            assert_eq!(a.il_accept_anything(b).to_dec(), ta.accept_anything(tb).to_dec(), "aa");
            // Arithmetic
            assert_eq!((a + b).to_dec(), a_v + b_v, "ADD({a_v},{b_v})");
            assert_eq!((a - b).to_dec(), a_v - b_v, "SUB({a_v},{b_v})");
            // Negation
            assert_eq!((-a).to_dec(), -a_v, "NEG({a_v})");
        }
    }
}

// ============================================================================
// BctTer32 — 32 balanced ternary trits in two u32 bitmasks
// ============================================================================

/// 32-trit balanced ternary number encoded as two `u32` bitmasks.
///
/// ## BCT (Binary-Coded Ternary) Encoding
///
/// Each of the 32 trits is represented by **two bits** spread across two
/// separate `u32` fields:
///
/// - `pos`: bit k is `1` iff trit k is `Pos` (+1)
/// - `neg`: bit k is `1` iff trit k is `Neg` (−1)
/// - Both zero ⟹ trit k is `Zero`
/// - Invariant: `pos & neg == 0`
///
/// Bit 0 = trit index 0 (least significant / rightmost), bit 31 = MSB.
///
/// ## Jones Bitwise Identities — O(1) trit-parallel ops
///
/// The `(pos, neg)` bitmask representation turns every trit-wise logical
/// operation into plain integer bitwise instructions:
///
/// | Ternary op       | CPU instruction count |
/// |------------------|-----------------------|
/// | NOT / neg        | 1 swap                |
/// | AND / min        | 2 (`&`, `\|`)          |
/// | OR / max         | 2 (`\|`, `&`)          |
/// | XOR / −(a·b)     | 4                     |
/// | Consensus        | 2                     |
/// | Accept-anything  | 4                     |
/// | Shl / Shr        | 2 (one shift each)    |
///
/// All these are **O(1)** regardless of word width, compared to O(32) for
/// the `Ter40` iterate-over-trits approach.
///
/// ## Arithmetic
///
/// Addition, subtraction, multiplication and division go through `i64` (same
/// as `Tryte<32>` and `Ter40`). Only trit-logical operations are O(1).
///
/// ```
/// use balanced_ternary::BctTer32;
///
/// let a = BctTer32::from_dec(13);   // "+++" in 3 trits
/// let b = BctTer32::from_dec(-5);   // "-++"
/// assert_eq!((a & b).to_dec(), -5); // AND = min
/// assert_eq!((a | b).to_dec(), 13); // OR  = max
/// assert_eq!((-a).to_dec(), -13);   // NOT = neg
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct BctTer32 {
    pos: u32,
    neg: u32,
}

impl BctTer32 {
    /// All trits zero.
    pub const ZERO: Self = Self { pos: 0, neg: 0 };
    /// All 32 trits = Pos (+1).
    pub const MAX: Self = Self { pos: u32::MAX, neg: 0 };
    /// All 32 trits = Neg (−1).
    pub const MIN: Self = Self { pos: 0, neg: u32::MAX };

    /// Construct directly from raw bitmasks.
    ///
    /// # Panics (debug builds only)
    /// Panics if `pos & neg != 0` (invariant violation).
    #[inline]
    pub const fn new(pos: u32, neg: u32) -> Self {
        debug_assert!(pos & neg == 0, "BctTer32: pos and neg masks overlap");
        Self { pos, neg }
    }

    /// Convert a signed 64-bit integer to 32 balanced ternary trits (wraps on overflow).
    ///
    /// ```
    /// use balanced_ternary::BctTer32;
    ///
    /// assert_eq!(BctTer32::from_dec(0).to_dec(), 0);
    /// assert_eq!(BctTer32::from_dec(13).to_dec(), 13);
    /// assert_eq!(BctTer32::from_dec(-13).to_dec(), -13);
    /// ```
    pub fn from_dec(v: i64) -> Self {
        if v == 0 { return Self::ZERO; }
        let negative = v < 0;
        let mut x = v.unsigned_abs();
        let mut pos = 0u32;
        let mut neg = 0u32;
        // SAFETY: unsigned single mod avoids ((n%3)+3)%3 double-mod pattern.
        if !negative {
            for bit in 0..32 {
                let rem = (x % 3) as u8;
                if rem == 1 { pos |= 1u32 << bit; }
                else if rem == 2 { neg |= 1u32 << bit; }
                x = (x - rem as u64) / 3 + (rem == 2) as u64;
            }
        } else {
            for bit in 0..32 {
                let rem = (x % 3) as u8;
                if rem == 1 { neg |= 1u32 << bit; }
                else if rem == 2 { pos |= 1u32 << bit; }
                x = (x - rem as u64) / 3 + (rem == 2) as u64;
            }
        }
        Self { pos, neg }
    }

    /// Convert the 32-trit BCT word back to a signed 64-bit integer.
    ///
    /// ```
    /// use balanced_ternary::BctTer32;
    ///
    /// let t = BctTer32::from_dec(12345);
    /// assert_eq!(t.to_dec(), 12345);
    /// ```
    pub fn to_dec(&self) -> i64 {
        let mut val: i64 = 0;
        for bit in (0..32u32).rev() {
            let trit = if self.pos & (1 << bit) != 0 { 1i64 }
                       else if self.neg & (1 << bit) != 0 { -1i64 }
                       else { 0i64 };
            val = val * 3 + trit;
        }
        val
    }

    /// Build from a `Ternary` (truncates to 32 trits if longer).
    pub fn from_ternary(t: &Ternary) -> Self {
        let digits = t.to_digit_slice(); // MSB-first
        let len = digits.len().min(32);
        let mut pos = 0u32;
        let mut neg = 0u32;
        // digits[len-1] is LSB → bit 0; digits[0] is near-MSB → bit len-1
        for (bit, &d) in digits.iter().rev().take(len).enumerate() {
            match d {
                Digit::Pos => pos |= 1u32 << bit,
                Digit::Neg => neg |= 1u32 << bit,
                Digit::Zero => {}
            }
        }
        Self { pos, neg }
    }

    /// Convert to `Ternary` (always 32 digits wide; call `.trim()` to strip leading zeros).
    pub fn to_ternary(&self) -> Ternary {
        Ternary::from_dec(self.to_dec()).with_length(32)
    }

    // -------------------------------------------------------------------------
    // O(1) Jones BCT bitwise operations
    // -------------------------------------------------------------------------

    /// Trit-wise negation: `Pos↔Neg`, `Zero→Zero`. O(1): one field swap.
    ///
    /// ```
    /// use balanced_ternary::BctTer32;
    ///
    /// let a = BctTer32::from_dec(42);
    /// assert_eq!(a.bct_neg().to_dec(), -42);
    /// assert_eq!(a.bct_neg().bct_neg(), a);
    /// ```
    #[inline(always)]
    pub const fn bct_neg(self) -> Self { Self { pos: self.neg, neg: self.pos } }

    /// Trit-wise AND (min). O(1): two bitwise instructions.
    ///
    /// ```
    /// use balanced_ternary::BctTer32;
    ///
    /// let a = BctTer32::from_dec(13);
    /// let b = BctTer32::from_dec(-5);
    /// assert_eq!(a.bct_and(b).to_dec(), -5);
    /// ```
    #[inline(always)]
    pub const fn bct_and(self, other: Self) -> Self {
        Self { pos: self.pos & other.pos, neg: self.neg | other.neg }
    }

    /// Trit-wise OR (max). O(1): two bitwise instructions.
    ///
    /// ```
    /// use balanced_ternary::BctTer32;
    ///
    /// let a = BctTer32::from_dec(13);
    /// let b = BctTer32::from_dec(-5);
    /// assert_eq!(a.bct_or(b).to_dec(), 13);
    /// ```
    #[inline(always)]
    pub const fn bct_or(self, other: Self) -> Self {
        Self { pos: self.pos | other.pos, neg: self.neg & other.neg }
    }

    /// Trit-wise XOR = −(a·b). O(1): four bitwise instructions.
    ///
    /// ```
    /// use balanced_ternary::BctTer32;
    ///
    /// let a = BctTer32::from_dec(13);
    /// let b = BctTer32::from_dec(-5);
    /// assert_eq!(a.bct_xor(b).to_dec(), 5);
    /// ```
    #[inline(always)]
    pub const fn bct_xor(self, other: Self) -> Self {
        Self {
            pos: (self.pos & other.neg) | (self.neg & other.pos),
            neg: (self.pos & other.pos) | (self.neg & other.neg),
        }
    }

    /// Trit-wise consensus: `+` where both `+`, `−` where both `−`, `0` elsewhere. O(1).
    ///
    /// ```
    /// use balanced_ternary::BctTer32;
    ///
    /// let a = BctTer32::from_dec(13);
    /// let b = BctTer32::from_dec(5);    // "+--"
    /// // consensus(+,+)=+, consensus(+,−)=0, consensus(+,−)=0 → "+00" = 9
    /// assert_eq!(a.bct_consensus(b).to_dec(), 9);
    /// ```
    #[inline(always)]
    pub const fn bct_consensus(self, other: Self) -> Self {
        Self { pos: self.pos & other.pos, neg: self.neg & other.neg }
    }

    /// Trit-wise accept-anything (`sign(a+b)`): non-zero wins, conflict → zero. O(1).
    ///
    /// ```
    /// use balanced_ternary::BctTer32;
    ///
    /// let a = BctTer32::from_dec(9);
    /// assert_eq!(a.bct_accept_anything(BctTer32::ZERO), a);
    /// assert_eq!(a.bct_accept_anything(a.bct_neg()), BctTer32::ZERO);
    /// ```
    #[inline(always)]
    pub const fn bct_accept_anything(self, other: Self) -> Self {
        Self {
            pos: (self.pos & !other.neg) | (other.pos & !self.neg),
            neg: (self.neg & !other.pos) | (other.neg & !self.pos),
        }
    }
}

impl DigitOperate for BctTer32 {
    fn to_digits(&self) -> Vec<Digit> {
        let mut digits = Vec::with_capacity(32);
        for bit in (0..32u32).rev() {
            digits.push(if self.pos & (1 << bit) != 0 {
                Digit::Pos
            } else if self.neg & (1 << bit) != 0 {
                Digit::Neg
            } else {
                Digit::Zero
            });
        }
        digits
    }

    fn digit(&self, index: usize) -> Option<Digit> {
        if index >= 32 { return None; }
        let bit = index as u32;
        Some(if self.pos & (1 << bit) != 0 { Digit::Pos }
             else if self.neg & (1 << bit) != 0 { Digit::Neg }
             else { Digit::Zero })
    }

    fn each(&self, f: impl Fn(Digit) -> Digit) -> Self {
        let mut pos = 0u32;
        let mut neg = 0u32;
        for bit in 0..32u32 {
            let d = if self.pos & (1 << bit) != 0 { Digit::Pos }
                    else if self.neg & (1 << bit) != 0 { Digit::Neg }
                    else { Digit::Zero };
            match f(d) {
                Digit::Pos  => pos |= 1 << bit,
                Digit::Neg  => neg |= 1 << bit,
                Digit::Zero => {}
            }
        }
        Self { pos, neg }
    }

    fn each_with(&self, f: impl Fn(Digit, Digit) -> Digit, with: Digit) -> Self {
        let mut pos = 0u32;
        let mut neg = 0u32;
        for bit in 0..32u32 {
            let d = if self.pos & (1 << bit) != 0 { Digit::Pos }
                    else if self.neg & (1 << bit) != 0 { Digit::Neg }
                    else { Digit::Zero };
            match f(d, with) {
                Digit::Pos  => pos |= 1 << bit,
                Digit::Neg  => neg |= 1 << bit,
                Digit::Zero => {}
            }
        }
        Self { pos, neg }
    }

    fn each_zip(&self, f: impl Fn(Digit, Digit) -> Digit, other: Self) -> Self {
        let mut pos = 0u32;
        let mut neg = 0u32;
        for bit in 0..32u32 {
            let da = if self.pos  & (1 << bit) != 0 { Digit::Pos }
                     else if self.neg  & (1 << bit) != 0 { Digit::Neg }
                     else { Digit::Zero };
            let db = if other.pos & (1 << bit) != 0 { Digit::Pos }
                     else if other.neg & (1 << bit) != 0 { Digit::Neg }
                     else { Digit::Zero };
            match f(da, db) {
                Digit::Pos  => pos |= 1 << bit,
                Digit::Neg  => neg |= 1 << bit,
                Digit::Zero => {}
            }
        }
        Self { pos, neg }
    }

    fn each_zip_carry(
        &self,
        f: impl Fn(Digit, Digit, Digit) -> (Digit, Digit),
        other: Self,
    ) -> Self {
        let mut pos = 0u32;
        let mut neg = 0u32;
        let mut carry = Digit::Zero;
        for bit in 0..32u32 {
            let da = if self.pos  & (1 << bit) != 0 { Digit::Pos }
                     else if self.neg  & (1 << bit) != 0 { Digit::Neg }
                     else { Digit::Zero };
            let db = if other.pos & (1 << bit) != 0 { Digit::Pos }
                     else if other.neg & (1 << bit) != 0 { Digit::Neg }
                     else { Digit::Zero };
            let (c, res) = f(da, db, carry);
            carry = c;
            match res {
                Digit::Pos  => pos |= 1 << bit,
                Digit::Neg  => neg |= 1 << bit,
                Digit::Zero => {}
            }
        }
        Self { pos, neg }
    }
}

impl Add for BctTer32 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        // Route through IlBctTer32 (Morton interleave) to use the fast biased uter trick.
        let a: IlBctTer32 = self.into();
        let b: IlBctTer32 = rhs.into();
        (a + b).into()
    }
}
impl Sub for BctTer32 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let a: IlBctTer32 = self.into();
        let b: IlBctTer32 = rhs.into();
        (a - b).into()
    }
}
impl Mul for BctTer32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self { Self::from_dec(self.to_dec() * rhs.to_dec()) }
}
impl Div for BctTer32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self { Self::from_dec(self.to_dec() / rhs.to_dec()) }
}
impl Neg for BctTer32 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self { self.bct_neg() }
}
impl Not for BctTer32 {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self { self.bct_neg() }
}
impl BitAnd for BctTer32 {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self { self.bct_and(rhs) }
}
impl BitOr for BctTer32 {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self { self.bct_or(rhs) }
}
impl BitXor for BctTer32 {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self { self.bct_xor(rhs) }
}

/// O(1) logical left shift: moves trits toward MSB, fills LSBs with Zero.
impl Shl<usize> for BctTer32 {
    type Output = Self;
    /// ```
    /// use balanced_ternary::BctTer32;
    /// assert_eq!((BctTer32::from_dec(1) << 1usize).to_dec(), 3);
    /// ```
    #[inline(always)]
    fn shl(self, rhs: usize) -> Self {
        if rhs >= 32 { Self::ZERO }
        else { Self { pos: self.pos << rhs, neg: self.neg << rhs } }
    }
}

/// O(1) logical right shift: moves trits toward LSB, fills MSBs with Zero.
impl Shr<usize> for BctTer32 {
    type Output = Self;
    /// ```
    /// use balanced_ternary::BctTer32;
    /// assert_eq!((BctTer32::from_dec(9) >> 1usize).to_dec(), 3);
    /// ```
    #[inline(always)]
    fn shr(self, rhs: usize) -> Self {
        if rhs >= 32 { Self::ZERO }
        else { Self { pos: self.pos >> rhs, neg: self.neg >> rhs } }
    }
}

impl From<i64> for BctTer32 {
    fn from(v: i64) -> Self { Self::from_dec(v) }
}
impl From<BctTer32> for i64 {
    fn from(t: BctTer32) -> Self { t.to_dec() }
}
impl From<Ternary> for BctTer32 {
    fn from(t: Ternary) -> Self { Self::from_ternary(&t) }
}
impl From<BctTer32> for Ternary {
    fn from(t: BctTer32) -> Self { t.to_ternary() }
}
impl core::fmt::Display for BctTer32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_ternary())
    }
}

// ============================================================================
// IlBctTer32 — Jones interleaved BCT: 32 balanced trits in one u64
// ============================================================================

/// Low bits of each 2-bit trit pair (bits 0,2,4,...,62).
const MASK_L: u64 = 0x5555_5555_5555_5555;
/// High bits of each 2-bit trit pair (bits 1,3,5,...,63).
const MASK_H: u64 = 0xAAAA_AAAA_AAAA_AAAA;

/// Spread the 32 bits of `x` to the even-indexed bit positions (0,2,4,...,62) of a u64.
///
/// On x86-64 with BMI2, this compiles to a single `PDEP` instruction.
/// Otherwise falls back to the standard 5-step Morton code interleave.
/// The inverse is `compact_u64`.
#[inline(always)]
fn spread_u32(x: u32) -> u64 {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    // SAFETY: bmi2 is verified at compile time via target_feature.
    return unsafe { core::arch::x86_64::_pdep_u64(x as u64, 0x5555_5555_5555_5555) };

    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
    {
        let mut x = x as u64;
        x = (x | (x << 16)) & 0x0000_FFFF_0000_FFFF;
        x = (x | (x <<  8)) & 0x00FF_00FF_00FF_00FF;
        x = (x | (x <<  4)) & 0x0F0F_0F0F_0F0F_0F0F;
        x = (x | (x <<  2)) & 0x3333_3333_3333_3333;
        x = (x | (x <<  1)) & 0x5555_5555_5555_5555;
        x
    }
}

/// Compact the even-indexed bits (0,2,4,...,62) of `x` into a u32.
///
/// On x86-64 with BMI2, this compiles to a single `PEXT` instruction.
/// Otherwise falls back to the standard 6-step Morton code compact.
/// Inverse of `spread_u32`.
#[inline(always)]
fn compact_u64(x: u64) -> u32 {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    // SAFETY: bmi2 is verified at compile time via target_feature.
    return unsafe { core::arch::x86_64::_pext_u64(x, 0x5555_5555_5555_5555) as u32 };

    #[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
    {
        let mut x = x & 0x5555_5555_5555_5555;
        x = (x | (x >>  1)) & 0x3333_3333_3333_3333;
        x = (x | (x >>  2)) & 0x0F0F_0F0F_0F0F_0F0F;
        x = (x | (x >>  4)) & 0x00FF_00FF_00FF_00FF;
        x = (x | (x >>  8)) & 0x0000_FFFF_0000_FFFF;
        x = (x | (x >> 16)) & 0x0000_0000_FFFF_FFFF;
        x as u32
    }
}

/// 32 balanced ternary trits packed into a **single `u64`** using 2 bits per trit.
///
/// ## Jones Interleaved BCT Encoding
///
/// Trit k occupies bits `(2k+1, 2k)`:
///
/// | Bit pair | Value |
/// |----------|-------|
/// | `00`     | −1 (`Neg`) |
/// | `01`     | 0 (`Zero`) |
/// | `10`     | +1 (`Pos`) |
/// | `11`     | invalid |
///
/// Trit 0 (LST) at bits (1,0); trit 31 (MST) at bits (63,62).
///
/// This matches the Jones `bter27_t` convention from his `libtern` library,
/// extended to 32 trits (2 × 32 = 64 bits, an exact u64 fit).
///
/// ## Comparison with `BctTer32`
///
/// | | `IlBctTer32` | `BctTer32` |
/// |-|-------------|------------|
/// | Storage | 1 × `u64` (single word) | 2 × `u32` (two words) |
/// | Logical ops | O(1) (slightly more instructions) | O(1) (cleaner formulas) |
/// | `from/to BctTer32` | O(1) via spread/compact | — |
/// | `to_dec` / arithmetic | O(32) loop | O(32) loop |
///
/// ## Quick Example
///
/// ```
/// use balanced_ternary::IlBctTer32;
///
/// let a = IlBctTer32::from_dec(13);   // "+++"
/// let b = IlBctTer32::from_dec(-5);   // "-++"
/// assert_eq!((a & b).to_dec(), -5);   // AND = min
/// assert_eq!((a | b).to_dec(), 13);   // OR  = max
/// assert_eq!((-a).to_dec(), -13);     // neg
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IlBctTer32(u64);

impl Default for IlBctTer32 {
    fn default() -> Self { Self::ZERO }
}

impl IlBctTer32 {
    /// All 32 trits = Zero.  Bit pattern: `01` × 32.
    pub const ZERO: Self = Self(MASK_L);
    /// All 32 trits = +1 (Pos).  Bit pattern: `10` × 32.
    pub const MAX: Self = Self(MASK_H);
    /// All 32 trits = −1 (Neg).  Bit pattern: `00` × 32.
    pub const MIN: Self = Self(0);

    /// Construct directly from the raw interleaved u64 word.
    ///
    /// # Safety (debug builds only)
    /// Each 2-bit pair must be `00`, `01`, or `10` (never `11`).
    #[inline]
    pub const fn from_raw(word: u64) -> Self {
        debug_assert!(word & (word >> 1) & MASK_L == 0, "IlBctTer32: invalid trit code 11");
        Self(word)
    }

    /// Return the raw interleaved u64 word.
    #[inline(always)]
    pub const fn raw(self) -> u64 { self.0 }

    /// Encode a signed 64-bit integer as 32 balanced ternary trits (wraps on overflow).
    ///
    /// ```
    /// use balanced_ternary::IlBctTer32;
    ///
    /// assert_eq!(IlBctTer32::from_dec(0).to_dec(), 0);
    /// assert_eq!(IlBctTer32::from_dec(13).to_dec(), 13);
    /// assert_eq!(IlBctTer32::from_dec(-13).to_dec(), -13);
    /// ```
    pub fn from_dec(v: i64) -> Self {
        let mut word = 0u64;
        let mut n = v;
        for k in 0..32u32 {
            let rem = ((n % 3) + 3) % 3;
            let trit: i8 = if rem <= 1 { rem as i8 } else { -1 };
            // -1 → 0b00 (0), 0 → 0b01 (1), +1 → 0b10 (2)
            let code = (trit as i64 + 1) as u64;
            word |= code << (2 * k);
            n = (n - trit as i64) / 3;
        }
        Self(word)
    }

    /// Decode the 32 interleaved trits back to a signed 64-bit integer.
    ///
    /// ```
    /// use balanced_ternary::IlBctTer32;
    ///
    /// let t = IlBctTer32::from_dec(12345);
    /// assert_eq!(t.to_dec(), 12345);
    /// ```
    pub fn to_dec(&self) -> i64 {
        let mut val: i64 = 0;
        for k in (0..32u32).rev() {
            let code = (self.0 >> (2 * k)) & 3;
            let trit = code as i64 - 1; // 0→-1, 1→0, 2→+1
            val = val * 3 + trit;
        }
        val
    }

    /// Convert to a `Ternary` (32 digits wide; call `.trim()` to strip leading zeros).
    pub fn to_ternary(&self) -> Ternary {
        Ternary::from_dec(self.to_dec()).with_length(32)
    }

    // -------------------------------------------------------------------------
    // Conversion to/from BctTer32 (O(1) via spread/compact)
    // -------------------------------------------------------------------------

    /// Convert from the split `(pos, neg)` `BctTer32` representation.
    ///
    /// ```
    /// use balanced_ternary::{BctTer32, IlBctTer32};
    ///
    /// let b = BctTer32::from_dec(42);
    /// let il = IlBctTer32::from_bct(b);
    /// assert_eq!(il.to_dec(), 42);
    /// ```
    #[inline(always)]
    pub fn from_bct(bct: BctTer32) -> Self {
        // Spread pos bits to odd positions (2k+1) and neg bits to even (2k).
        // Start from all-Zero (MASK_L = all 01).
        // Pos trits: set high bit, clear low bit → 10.
        // Neg trits: clear low bit → 00 (high bit already 0).
        let pos_e = spread_u32(bct.pos); // pos bits at even positions
        let neg_e = spread_u32(bct.neg); // neg bits at even positions
        let pos_o = pos_e << 1;           // pos bits at odd positions
        Self((MASK_L | pos_o) & !(pos_e | neg_e))
    }

    /// Convert to the split `(pos, neg)` `BctTer32` representation.
    ///
    /// ```
    /// use balanced_ternary::{BctTer32, IlBctTer32};
    ///
    /// let il = IlBctTer32::from_dec(-99);
    /// let b  = il.to_bct();
    /// assert_eq!(b.to_dec(), -99);
    /// ```
    #[inline(always)]
    pub fn to_bct(&self) -> BctTer32 {
        // For valid codes: 10=Pos (high=1,low=0), 01=Zero, 00=Neg (high=0,low=0).
        let high = (self.0 >> 1) & MASK_L; // high bits at even positions
        let low  =  self.0       & MASK_L; // low bits at even positions
        // pos: high=1 (low=0 for valid code)
        // neg: neither high nor low set
        let pos = compact_u64(high);
        let neg = compact_u64(MASK_L & !high & !low);
        BctTer32 { pos, neg }
    }

    // -------------------------------------------------------------------------
    // O(1) Jones interleaved logical operations
    // -------------------------------------------------------------------------

    /// Trit-wise negation: `Pos↔Neg`, `Zero→Zero`.
    ///
    /// ```
    /// use balanced_ternary::IlBctTer32;
    ///
    /// let a = IlBctTer32::from_dec(42);
    /// assert_eq!(a.il_neg().to_dec(), -42);
    /// assert_eq!(a.il_neg().il_neg(), a);
    /// ```
    #[inline(always)]
    pub const fn il_neg(self) -> Self {
        // For pair (h,l): 00→10, 01→01, 10→00.
        // new_h = NOR(h,l) = !(h|l);  new_l = ANDN(l,h) = !h & l
        let h = (self.0 >> 1) & MASK_L;
        let l =  self.0       & MASK_L;
        let new_h = !(h | l) & MASK_L;
        let new_l = !h & l & MASK_L;
        Self((new_h << 1) | new_l)
    }

    /// Trit-wise AND (min).
    ///
    /// ```
    /// use balanced_ternary::IlBctTer32;
    ///
    /// let a = IlBctTer32::from_dec(13);  // "+++"
    /// let b = IlBctTer32::from_dec(-5);  // "-++"
    /// assert_eq!(a.il_and(b).to_dec(), -5); // min(+,+)=+, min(+,-)=-, min(+,+)=+... wait: AND(13,-5)=-5
    /// ```
    #[inline(always)]
    pub const fn il_and(self, other: Self) -> Self {
        let ha = (self.0  >> 1) & MASK_L;
        let la =  self.0        & MASK_L;
        let hb = (other.0 >> 1) & MASK_L;
        let lb =  other.0       & MASK_L;
        // new_h = ha & hb  (result=+1 only when both +1)
        // new_l = la & (hb|lb) | ha & lb  (result=0 when both ≥0 but not both +1)
        let hr = ha & hb;
        let lr = (la & (hb | lb)) | (ha & lb);
        Self((hr << 1) | (lr & MASK_L))
    }

    /// Trit-wise OR (max).
    ///
    /// ```
    /// use balanced_ternary::IlBctTer32;
    ///
    /// let a = IlBctTer32::from_dec(13);
    /// let b = IlBctTer32::from_dec(-5);
    /// assert_eq!(a.il_or(b).to_dec(), 13);
    /// ```
    #[inline(always)]
    pub const fn il_or(self, other: Self) -> Self {
        let ha = (self.0  >> 1) & MASK_L;
        let la =  self.0        & MASK_L;
        let hb = (other.0 >> 1) & MASK_L;
        let lb =  other.0       & MASK_L;
        // new_h = ha | hb  (result=+1 when either is +1)
        // new_l = !(ha|hb) & (la|lb)  (result=0 when neither +1 but at least one ≥0)
        let hr = ha | hb;
        let lr = !(ha | hb) & (la | lb) & MASK_L;
        Self((hr << 1) | lr)
    }

    /// Trit-wise XOR = −(a·b).
    ///
    /// ```
    /// use balanced_ternary::IlBctTer32;
    ///
    /// let a = IlBctTer32::from_dec(13);
    /// let b = IlBctTer32::from_dec(-5);
    /// assert_eq!(a.il_xor(b).to_dec(), 5);
    /// ```
    #[inline(always)]
    pub const fn il_xor(self, other: Self) -> Self {
        let ha = (self.0  >> 1) & MASK_L;
        let la =  self.0        & MASK_L;
        let hb = (other.0 >> 1) & MASK_L;
        let lb =  other.0       & MASK_L;
        let isneg_a = MASK_L & !ha & !la; // a = -1
        let isneg_b = MASK_L & !hb & !lb; // b = -1
        // result=+1: opposite nonzero signs
        let new_h = (ha & isneg_b) | (isneg_a & hb);
        // result=-1: same nonzero sign
        let is_neg = (ha & hb) | (isneg_a & isneg_b);
        // result=0: otherwise
        let new_l = MASK_L & !new_h & !is_neg;
        Self((new_h << 1) | new_l)
    }

    /// Trit-wise consensus: agrees when both nonzero and equal, else Zero.
    ///
    /// ```
    /// use balanced_ternary::IlBctTer32;
    ///
    /// let a = IlBctTer32::from_dec(13); // "+++"
    /// let b = IlBctTer32::from_dec(5);  // "+--"
    /// // consensus(+,+)=+, consensus(+,-)=0, consensus(+,-)=0 → "+00" = 9
    /// assert_eq!(a.il_consensus(b).to_dec(), 9);
    /// ```
    #[inline(always)]
    pub const fn il_consensus(self, other: Self) -> Self {
        let ha = (self.0  >> 1) & MASK_L;
        let la =  self.0        & MASK_L;
        let hb = (other.0 >> 1) & MASK_L;
        let lb =  other.0       & MASK_L;
        let isneg_a = MASK_L & !ha & !la;
        let isneg_b = MASK_L & !hb & !lb;
        let new_h = ha & hb;                  // +1 only when both +1
        let is_neg = isneg_a & isneg_b;       // -1 only when both -1
        let new_l = MASK_L & !new_h & !is_neg; // 0 otherwise
        Self((new_h << 1) | new_l)
    }

    /// Trit-wise accept-anything: nonzero wins over zero; conflict → zero.
    ///
    /// ```
    /// use balanced_ternary::IlBctTer32;
    ///
    /// let a = IlBctTer32::from_dec(9);
    /// assert_eq!(a.il_accept_anything(IlBctTer32::ZERO), a);
    /// assert_eq!(a.il_accept_anything(a.il_neg()), IlBctTer32::ZERO);
    /// ```
    #[inline(always)]
    pub const fn il_accept_anything(self, other: Self) -> Self {
        let ha = (self.0  >> 1) & MASK_L;
        let la =  self.0        & MASK_L;
        let hb = (other.0 >> 1) & MASK_L;
        let lb =  other.0       & MASK_L;
        let isneg_a = MASK_L & !ha & !la;
        let isneg_b = MASK_L & !hb & !lb;
        let not_neg_a = ha | la;
        let not_neg_b = hb | lb;
        let not_pos_a = (MASK_L & !ha) | la;
        let not_pos_b = (MASK_L & !hb) | lb;
        // +1: (a=+1 and b≠-1) or (b=+1 and a≠-1)
        let new_h = (ha & not_neg_b) | (hb & not_neg_a);
        // -1: (a=-1 and b≠+1) or (b=-1 and a≠+1)
        let is_neg = (isneg_a & not_pos_b) | (isneg_b & not_pos_a);
        let new_l = MASK_L & !new_h & !is_neg;
        Self((new_h << 1) | new_l)
    }

    /// Logical left shift by `n` trits (fill with Zero).
    #[inline(always)]
    pub const fn il_shl(self, n: usize) -> Self {
        if n >= 32 { return Self::ZERO; }
        let shift = 2 * n;
        // Fill the lower `n` trit-positions with Zero (01 pattern).
        let fill = MASK_L & ((1u64 << shift).wrapping_sub(1));
        Self((self.0 << shift) | fill)
    }

    /// Logical right shift by `n` trits (fill with Zero).
    #[inline(always)]
    pub const fn il_shr(self, n: usize) -> Self {
        if n >= 32 { return Self::ZERO; }
        let shift = 2 * n;
        // Fill the upper `n` trit-positions with Zero (01 pattern).
        let fill = MASK_L & (u64::MAX << (64 - shift));
        Self((self.0 >> shift) | fill)
    }
}

impl DigitOperate for IlBctTer32 {
    fn to_digits(&self) -> Vec<Digit> {
        (0..32u32).rev().map(|k| {
            match (self.0 >> (2 * k)) & 3 {
                2 => Digit::Pos,
                0 => Digit::Neg,
                _ => Digit::Zero,
            }
        }).collect()
    }

    fn digit(&self, index: usize) -> Option<Digit> {
        if index >= 32 { return None; }
        Some(match (self.0 >> (2 * index as u32)) & 3 {
            2 => Digit::Pos,
            0 => Digit::Neg,
            _ => Digit::Zero,
        })
    }

    fn each(&self, f: impl Fn(Digit) -> Digit) -> Self {
        let mut word = 0u64;
        for k in 0..32u32 {
            let d = match (self.0 >> (2 * k)) & 3 {
                2 => Digit::Pos,
                0 => Digit::Neg,
                _ => Digit::Zero,
            };
            let code = match f(d) {
                Digit::Pos  => 2u64,
                Digit::Zero => 1u64,
                Digit::Neg  => 0u64,
            };
            word |= code << (2 * k);
        }
        Self(word)
    }

    fn each_with(&self, f: impl Fn(Digit, Digit) -> Digit, with: Digit) -> Self {
        let mut word = 0u64;
        for k in 0..32u32 {
            let d = match (self.0 >> (2 * k)) & 3 {
                2 => Digit::Pos,
                0 => Digit::Neg,
                _ => Digit::Zero,
            };
            let code = match f(d, with) {
                Digit::Pos  => 2u64,
                Digit::Zero => 1u64,
                Digit::Neg  => 0u64,
            };
            word |= code << (2 * k);
        }
        Self(word)
    }

    fn each_zip(&self, f: impl Fn(Digit, Digit) -> Digit, other: Self) -> Self {
        let mut word = 0u64;
        for k in 0..32u32 {
            let da = match (self.0  >> (2 * k)) & 3 { 2 => Digit::Pos, 0 => Digit::Neg, _ => Digit::Zero };
            let db = match (other.0 >> (2 * k)) & 3 { 2 => Digit::Pos, 0 => Digit::Neg, _ => Digit::Zero };
            let code = match f(da, db) { Digit::Pos => 2u64, Digit::Zero => 1u64, Digit::Neg => 0u64 };
            word |= code << (2 * k);
        }
        Self(word)
    }

    fn each_zip_carry(&self, f: impl Fn(Digit, Digit, Digit) -> (Digit, Digit), other: Self) -> Self {
        let mut word = 0u64;
        let mut carry = Digit::Zero;
        for k in 0..32u32 {
            let da = match (self.0  >> (2 * k)) & 3 { 2 => Digit::Pos, 0 => Digit::Neg, _ => Digit::Zero };
            let db = match (other.0 >> (2 * k)) & 3 { 2 => Digit::Pos, 0 => Digit::Neg, _ => Digit::Zero };
            let (c, res) = f(da, db, carry);
            carry = c;
            let code = match res { Digit::Pos => 2u64, Digit::Zero => 1u64, Digit::Neg => 0u64 };
            word |= code << (2 * k);
        }
        Self(word)
    }
}

impl Add for IlBctTer32 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        // Biased UTer32 trick via u128 (avoids MASK_L<<2 overflow for pair 31).
        let s = uter_add_u128(self.0, rhs.0, MASK_L);
        Self(uter_add_u128(s, ILBCT32_REMAP, MASK_L))
    }
}
impl Sub for IlBctTer32 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        let neg_r = il_neg_u64(rhs.0, MASK_L);
        let s = uter_add_u128(self.0, neg_r, MASK_L);
        Self(uter_add_u128(s, ILBCT32_REMAP, MASK_L))
    }
}
impl Mul for IlBctTer32 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self { Self::from_dec(self.to_dec() * rhs.to_dec()) }
}
impl Div for IlBctTer32 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self { Self::from_dec(self.to_dec() / rhs.to_dec()) }
}
impl Neg for IlBctTer32 {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self { self.il_neg() }
}
impl Not for IlBctTer32 {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self { self.il_neg() }
}
impl BitAnd for IlBctTer32 {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self { self.il_and(rhs) }
}
impl BitOr for IlBctTer32 {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self { self.il_or(rhs) }
}
impl BitXor for IlBctTer32 {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self { self.il_xor(rhs) }
}
impl Shl<usize> for IlBctTer32 {
    type Output = Self;
    /// ```
    /// use balanced_ternary::IlBctTer32;
    /// assert_eq!((IlBctTer32::from_dec(1) << 1usize).to_dec(), 3);
    /// ```
    #[inline(always)]
    fn shl(self, rhs: usize) -> Self { self.il_shl(rhs) }
}
impl Shr<usize> for IlBctTer32 {
    type Output = Self;
    /// ```
    /// use balanced_ternary::IlBctTer32;
    /// assert_eq!((IlBctTer32::from_dec(9) >> 1usize).to_dec(), 3);
    /// ```
    #[inline(always)]
    fn shr(self, rhs: usize) -> Self { self.il_shr(rhs) }
}
impl From<i64> for IlBctTer32 {
    fn from(v: i64) -> Self { Self::from_dec(v) }
}
impl From<IlBctTer32> for i64 {
    fn from(t: IlBctTer32) -> Self { t.to_dec() }
}
impl From<Ternary> for IlBctTer32 {
    fn from(t: Ternary) -> Self { Self::from_dec(t.to_dec()) }
}
impl From<IlBctTer32> for Ternary {
    fn from(t: IlBctTer32) -> Self { t.to_ternary() }
}
impl From<BctTer32> for IlBctTer32 {
    #[inline]
    fn from(b: BctTer32) -> Self { Self::from_bct(b) }
}
impl From<IlBctTer32> for BctTer32 {
    #[inline]
    fn from(t: IlBctTer32) -> Self { t.to_bct() }
}
impl core::fmt::Display for IlBctTer32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_ternary())
    }
}

// ---- IlBctTer32 unit tests --------------------------------------------------

#[cfg(test)]
#[test]
fn il_roundtrip() {
    for v in [-13i64, -5, -1, 0, 1, 5, 13, 42, 12345, -12345] {
        assert_eq!(IlBctTer32::from_dec(v).to_dec(), v, "roundtrip failed for {v}");
    }
}

#[cfg(test)]
#[test]
fn il_constants() {
    assert_eq!(IlBctTer32::ZERO.to_dec(), 0);
    // MAX: all trits +1 → value = (3^32-1)/2
    let max_val = IlBctTer32::MAX.to_dec();
    assert!(max_val > 0);
    assert_eq!(IlBctTer32::MIN.to_dec(), -max_val);
    assert_eq!(IlBctTer32::ZERO.raw(), MASK_L);
    assert_eq!(IlBctTer32::MAX.raw(), MASK_H);
    assert_eq!(IlBctTer32::MIN.raw(), 0);
}

#[cfg(test)]
#[test]
fn il_bitwise_ops() {
    // a = 13 = "+++", b = -5 = "-++"
    let a = IlBctTer32::from_dec(13);
    let b = IlBctTer32::from_dec(-5);
    assert_eq!((a & b).to_dec(), -5);             // AND = min
    assert_eq!((a | b).to_dec(), 13);              // OR  = max
    assert_eq!((a ^ b).to_dec(), 5);               // XOR = -(a*b)
    assert_eq!((-a).to_dec(), -13);                // neg
    assert_eq!(a.il_consensus(b).to_dec(), 4);     // consensus
    assert_eq!(a.il_accept_anything(IlBctTer32::ZERO), a);   // zero transparent
    assert_eq!(a.il_accept_anything(-a), IlBctTer32::ZERO);  // conflict → zero
}

#[cfg(test)]
#[test]
fn il_shifts() {
    assert_eq!((IlBctTer32::from_dec(1) << 1usize).to_dec(), 3);
    assert_eq!((IlBctTer32::from_dec(1) << 2usize).to_dec(), 9);
    assert_eq!((IlBctTer32::from_dec(9) >> 1usize).to_dec(), 3);
    assert_eq!((IlBctTer32::from_dec(9) >> 2usize).to_dec(), 1);
    assert_eq!((IlBctTer32::from_dec(1) << 32usize), IlBctTer32::ZERO);
    assert_eq!((IlBctTer32::from_dec(1) >> 32usize), IlBctTer32::ZERO);
}

#[cfg(test)]
#[test]
fn il_bct_conversion_roundtrip() {
    for v in [-99i64, -13, -5, 0, 5, 13, 42, 99, 12345] {
        let il = IlBctTer32::from_dec(v);
        let bct: BctTer32 = il.into();
        let back: IlBctTer32 = bct.into();
        assert_eq!(bct.to_dec(), v, "to_bct failed for {v}");
        assert_eq!(back.to_dec(), v, "from_bct roundtrip failed for {v}");
        assert_eq!(il, back, "interleaved bits differ for {v}");
    }
}

#[cfg(test)]
#[test]
fn il_matches_bct_ops() {
    // Every IlBctTer32 logical op must agree with BctTer32's equivalent.
    let vals = [-13i64, -5, -1, 0, 1, 5, 13];
    for &a in &vals {
        for &b in &vals {
            let ia = IlBctTer32::from_dec(a);
            let ib = IlBctTer32::from_dec(b);
            let ba = BctTer32::from_dec(a);
            let bb = BctTer32::from_dec(b);
            assert_eq!((ia & ib).to_dec(), (ba & bb).to_dec(), "AND mismatch a={a} b={b}");
            assert_eq!((ia | ib).to_dec(), (ba | bb).to_dec(), "OR  mismatch a={a} b={b}");
            assert_eq!((ia ^ ib).to_dec(), (ba ^ bb).to_dec(), "XOR mismatch a={a} b={b}");
            assert_eq!((-ia).to_dec(), (-ba).to_dec(), "NEG mismatch a={a}");
            assert_eq!(ia.il_consensus(ib).to_dec(), ba.bct_consensus(bb).to_dec(), "consensus mismatch a={a} b={b}");
            assert_eq!(ia.il_accept_anything(ib).to_dec(), ba.bct_accept_anything(bb).to_dec(), "accept_anything mismatch a={a} b={b}");
        }
    }
}

// ============================================================================
// Jones libtern types: UTer9, UTer27, BTer9, BTer27
// ============================================================================
//
// All four types use the Jones interleaved BCT encoding (2 bits/trit):
//   unsigned BCT: 00=0, 01=1, 10=2
//   balanced BCT: 00=−1, 01=0, 10=+1
//
// Matches Jones's C typedefs:
//   typedef uint32_t uter9_t;    /* 9-trit unsigned ternary  */
//   typedef uint64_t uter27_t;   /* 27-trit unsigned ternary */
//   typedef uint32_t bter9_t;    /* 9-trit balanced ternary  */
//   typedef uint64_t bter27_t;   /* 27-trit balanced ternary */

// --- 9-trit constants (u32, bits 0-17) ---
const MASK9: u32   = 0x0003_FFFF; // 18-bit mask (9 trit-pairs)
const MASK9_L: u32 = 0x0001_5555; // low  bits of 9 pairs (bits 0,2,4,...,16)
const MASK9_H: u32 = 0x0002_AAAA; // high bits of 9 pairs (bits 1,3,5,...,17)

// --- 27-trit constants (u64, bits 0-53) ---
const MASK27: u64   = 0x003F_FFFF_FFFF_FFFF; // 54-bit mask (27 trit-pairs)
const MASK27_L: u64 = 0x0015_5555_5555_5555; // low  bits of 27 pairs
const MASK27_H: u64 = 0x002A_AAAA_AAAA_AAAA; // high bits of 27 pairs

// -------------------------------------------------------------------------
// Shared bit-manipulation helpers (same logical formulas, different masks)
// -------------------------------------------------------------------------

/// Interleaved BCT trit-wise negation for a u32 word with `mask_l` as the
/// low-bit mask.  Maps 00↔10, preserves 01.
#[inline(always)]
const fn il_neg_u32(w: u32, mask_l: u32) -> u32 {
    let h = (w >> 1) & mask_l;
    let l =  w       & mask_l;
    ((!(h | l) & mask_l) << 1) | (!h & l & mask_l)
}

/// Same for u64.
#[inline(always)]
const fn il_neg_u64(w: u64, mask_l: u64) -> u64 {
    let h = (w >> 1) & mask_l;
    let l =  w       & mask_l;
    ((!(h | l) & mask_l) << 1) | (!h & l & mask_l)
}

/// AND (min) for interleaved BCT u32.
#[inline(always)]
const fn il_and_u32(a: u32, b: u32, mask_l: u32) -> u32 {
    let ha = (a >> 1) & mask_l; let la = a & mask_l;
    let hb = (b >> 1) & mask_l; let lb = b & mask_l;
    let hr = ha & hb;
    let lr = (la & (hb | lb)) | (ha & lb);
    (hr << 1) | (lr & mask_l)
}

/// AND (min) for interleaved BCT u64.
#[inline(always)]
const fn il_and_u64(a: u64, b: u64, mask_l: u64) -> u64 {
    let ha = (a >> 1) & mask_l; let la = a & mask_l;
    let hb = (b >> 1) & mask_l; let lb = b & mask_l;
    let hr = ha & hb;
    let lr = (la & (hb | lb)) | (ha & lb);
    (hr << 1) | (lr & mask_l)
}

/// OR (max) for interleaved BCT u32.
#[inline(always)]
const fn il_or_u32(a: u32, b: u32, mask_l: u32) -> u32 {
    let ha = (a >> 1) & mask_l; let la = a & mask_l;
    let hb = (b >> 1) & mask_l; let lb = b & mask_l;
    let hr = ha | hb;
    let lr = !(ha | hb) & (la | lb) & mask_l;
    (hr << 1) | lr
}

/// OR (max) for interleaved BCT u64.
#[inline(always)]
const fn il_or_u64(a: u64, b: u64, mask_l: u64) -> u64 {
    let ha = (a >> 1) & mask_l; let la = a & mask_l;
    let hb = (b >> 1) & mask_l; let lb = b & mask_l;
    let hr = ha | hb;
    let lr = !(ha | hb) & (la | lb) & mask_l;
    (hr << 1) | lr
}

/// XOR = −(a·b) for interleaved BCT u32.
#[inline(always)]
const fn il_xor_u32(a: u32, b: u32, mask_l: u32) -> u32 {
    let ha = (a >> 1) & mask_l; let la = a & mask_l;
    let hb = (b >> 1) & mask_l; let lb = b & mask_l;
    let na = mask_l & !ha & !la;
    let nb = mask_l & !hb & !lb;
    let new_h = (ha & nb) | (na & hb);
    let is_n  = (ha & hb) | (na & nb);
    let new_l = mask_l & !new_h & !is_n;
    (new_h << 1) | new_l
}

/// XOR for interleaved BCT u64.
#[inline(always)]
const fn il_xor_u64(a: u64, b: u64, mask_l: u64) -> u64 {
    let ha = (a >> 1) & mask_l; let la = a & mask_l;
    let hb = (b >> 1) & mask_l; let lb = b & mask_l;
    let na = mask_l & !ha & !la;
    let nb = mask_l & !hb & !lb;
    let new_h = (ha & nb) | (na & hb);
    let is_n  = (ha & hb) | (na & nb);
    let new_l = mask_l & !new_h & !is_n;
    (new_h << 1) | new_l
}

// ---- Parameterized Jones uter_add (shared by UTer9/UTer27/BTer9/BTer27) ----
//
// bter_add(a, b) = uter_add(uter_add(a.raw, b.raw), REMAP)
// where REMAP = MASK_L + 1 is the BCT complement of MASK_L.
// This avoids the ~91 ns divmod encode by staying in the IL domain.

#[inline(always)]
const fn uter_add_u32_raw(a: u32, b: u32, mask_l: u32) -> u32 {
    let c = a.wrapping_add(mask_l);
    let d = b.wrapping_add(c);
    let e = (b ^ c) ^ d;
    let e = !e & (mask_l << 2);
    let e = e >> 2;
    d.wrapping_sub(e) & (mask_l | (mask_l << 1))
}

#[inline(always)]
const fn uter_add_u64_raw(a: u64, b: u64, mask_l: u64) -> u64 {
    let c = a.wrapping_add(mask_l);
    let d = b.wrapping_add(c);
    let e = (b ^ c) ^ d;
    let e = !e & (mask_l << 2);
    let e = e >> 2;
    d.wrapping_sub(e) & (mask_l | (mask_l << 1))
}

/// BTer add remap constant for 27-trit (complement of MASK27_L in 27-trit modular arithmetic).
const BTER27_REMAP: u64 = MASK27_L + 1;
/// BTer add remap constant for 9-trit.
const BTER9_REMAP: u32  = MASK9_L + 1;

// For IlBctTer32 (32 balanced trits in exactly 64 bits):
// MASK_L << 2 overflows u64 for pair 31, so use u128 intermediate.
// MASK_L = 0x5555_5555_5555_5555; ILBCT32_REMAP = MASK_L + 1 = compl(MASK_L) in 32-trit arithmetic.
const ILBCT32_REMAP: u64 = MASK_L + 1;

/// Jones uter_add via u128 intermediate — needed when mask_l uses all 64 bits (32 trit-pairs).
#[inline(always)]
fn uter_add_u128(a: u64, b: u64, mask_l: u64) -> u64 {
    let ml = mask_l as u128;
    let a  = a  as u128;
    let b  = b  as u128;
    let c  = a.wrapping_add(ml);
    let d  = b.wrapping_add(c);
    let e  = (b ^ c) ^ d;
    let e  = !e & (ml << 2);  // no overflow in u128
    let e  = e >> 2;
    (d.wrapping_sub(e) & (ml | (ml << 1))) as u64
}

/// Precomputed 5-trit BCT encoding: `UTER5_LUT[v]` = 10-bit BCT word for `v < 243`.
/// Trit k occupies bits `2k..2k+2`: `00=0, 01=1, 10=2`.
const UTER5_LUT: [u16; 243] = {
    let mut lut = [0u16; 243];
    let mut v = 0usize;
    while v < 243 {
        let mut word = 0u16;
        let mut n = v;
        let mut i = 0;
        while i < 5 {
            word |= ((n % 3) as u16) << (2 * i);
            n /= 3;
            i += 1;
        }
        lut[v] = word;
        v += 1;
    }
    lut
};

/// Precomputed 3-trit BCT decoding: `UTER3_DEC_LUT[c]` = decimal value (0..26)
/// of the 3-trit BCT word `c` (6 bits, trit k occupies bits `2k..2k+2`).
///
/// Invalid codes (any 2-bit pair == 11) are never indexed by valid `UTer9`/`UTer27` values.
const UTER3_DEC_LUT: [u8; 64] = {
    let mut lut = [0u8; 64];
    let mut c = 0usize;
    while c < 64 {
        lut[c] = (((c >> 4) & 3) * 9 + ((c >> 2) & 3) * 3 + (c & 3)) as u8;
        c += 1;
    }
    lut
};

// =========================================================================
// UTer9 — 9-trit unsigned ternary (uter9_t), range 0..19682
// =========================================================================

/// 9-trit unsigned ternary packed in a `u32` (18 bits used).
///
/// Matches Jones's `uter9_t`.  Encoding: `00=0, 01=1, 10=2`.
///
/// ```
/// use balanced_ternary::UTer9;
///
/// let a = UTer9::from_dec(42);
/// assert_eq!(a.to_dec(), 42);
/// assert_eq!((a + UTer9::from_dec(1)).to_dec(), 43);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct UTer9(u32);

impl UTer9 {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(MASK9_H); // all 10 = 2 per trit → 3^9 - 1 = 19682

    #[inline] pub fn from_dec(v: u32) -> Self {
        let mut n = (v % 19683) as usize; // 3^9
        let lo = UTER5_LUT[n % 243] as u32;
        n /= 243;
        let hi = UTER5_LUT[n] as u32; // n < 82 (19683/243 = 81)
        Self(lo | (hi << 10))
    }

    #[inline] pub fn to_dec(&self) -> u32 {
        // Three 3-trit LUT lookups (6 bits each) + two multiplies instead of
        // a 9-step serial Horner chain. LLVM can schedule all three loads in parallel.
        let lo  = UTER3_DEC_LUT[(self.0 & 0x3F) as usize] as u32;
        let mid = UTER3_DEC_LUT[((self.0 >> 6) & 0x3F) as usize] as u32;
        let hi  = UTER3_DEC_LUT[(self.0 >> 12) as usize] as u32;
        lo + mid * 27 + hi * 729
    }

    #[inline(always)] pub const fn raw(self) -> u32 { self.0 }

    /// Jones `uter9_add`: O(1) BCT addition without going through decimal.
    ///
    /// ```
    /// use balanced_ternary::UTer9;
    /// assert_eq!((UTer9::from_dec(100) + UTer9::from_dec(200)).to_dec(), 300);
    /// ```
    #[inline]
    pub const fn uter_add(self, other: Self) -> Self {
        // Jones 1960s BCD-style trick adapted for base-3.
        let a = self.0;
        let b = other.0;
        let c = a.wrapping_add(MASK9_L);   // add 1 to each trit-pair
        let d = b.wrapping_add(c);          // tentative sum
        let e = (b ^ c) ^ d;               // carry bits
        let e = !e & 0x0005_5554u32;       // positions with no carry overflow
        let e = e >> 2;
        Self(d.wrapping_sub(e) & MASK9)
    }

    #[inline] pub const fn uter_sub(self, other: Self) -> Self {
        // 2's complement style: negate via digit-flip then +1.
        // digit-wise complement: 00↔10, 01↔01 (i.e. 2−d per trit).
        let comp2 = (MASK9_H.wrapping_sub(other.0)) & MASK9;
        let comp3 = Self(comp2).uter_add(Self(0x01)); // +1 (BCT code 01 at trit 0)
        self.uter_add(comp3)
    }

    // Trit-wise logical ops (min/max/xor)
    #[inline(always)] pub const fn trit_and(self, o: Self) -> Self { Self(il_and_u32(self.0, o.0, MASK9_L)) }
    #[inline(always)] pub const fn trit_or (self, o: Self) -> Self { Self(il_or_u32 (self.0, o.0, MASK9_L)) }
    #[inline(always)] pub const fn trit_xor(self, o: Self) -> Self { Self(il_xor_u32(self.0, o.0, MASK9_L)) }
}

impl Add for UTer9 { type Output = Self; fn add(self, r: Self) -> Self { self.uter_add(r) } }
impl Sub for UTer9 { type Output = Self; fn sub(self, r: Self) -> Self { self.uter_sub(r) } }
impl Mul for UTer9 { type Output = Self; fn mul(self, r: Self) -> Self { Self::from_dec(self.to_dec() * r.to_dec()) } }
impl Div for UTer9 { type Output = Self; fn div(self, r: Self) -> Self { Self::from_dec(self.to_dec() / r.to_dec()) } }
impl BitAnd for UTer9 { type Output = Self; fn bitand(self, r: Self) -> Self { self.trit_and(r) } }
impl BitOr  for UTer9 { type Output = Self; fn bitor (self, r: Self) -> Self { self.trit_or(r)  } }
impl BitXor for UTer9 { type Output = Self; fn bitxor(self, r: Self) -> Self { self.trit_xor(r) } }
impl Shl<usize> for UTer9 {
    type Output = Self;
    fn shl(self, n: usize) -> Self {
        if n >= 9 { return Self::ZERO; }
        // fill lower n trits with ZERO (code 00 for unsigned = 0)
        Self((self.0 << (2 * n)) & MASK9)
    }
}
impl Shr<usize> for UTer9 {
    type Output = Self;
    fn shr(self, n: usize) -> Self {
        if n >= 9 { return Self::ZERO; }
        Self((self.0 >> (2 * n)) & MASK9)
    }
}
impl From<u32>  for UTer9 { fn from(v: u32)  -> Self { Self::from_dec(v) } }
impl From<UTer9> for u32  { fn from(t: UTer9) -> u32  { t.to_dec() } }
impl core::fmt::Display for UTer9 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dec())
    }
}

// =========================================================================
// UTer27 — 27-trit unsigned ternary (uter27_t), range 0..7625597484986
// =========================================================================

/// 27-trit unsigned ternary packed in a `u64` (54 bits used).
///
/// Matches Jones's `uter27_t`.  Encoding: `00=0, 01=1, 10=2`.
///
/// ```
/// use balanced_ternary::UTer27;
///
/// let a = UTer27::from_dec(1_000_000);
/// assert_eq!(a.to_dec(), 1_000_000);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct UTer27(u64);

impl UTer27 {
    pub const ZERO: Self = Self(0);
    pub const MAX: Self = Self(MASK27_H); // all 10 = 2 → 3^27 - 1

    #[inline] pub fn from_dec(v: u64) -> Self {
        let mut n = v % 7_625_597_484_987; // 3^27
        let mut word = 0u64;
        let mut shift = 0u32;
        while n > 0 {
            word |= (UTER5_LUT[(n % 243) as usize] as u64) << shift;
            n /= 243;
            shift += 10;
        }
        Self(word)
    }

    #[inline] pub fn to_dec(&self) -> u64 {
        // Split into three independent 9-trit groups — same ILP trick as BTer27::to_dec.
        // Each group calls the LUT-based UTer9::to_dec (3 lookups + 2 muls).
        let lo  = UTer9((self.0 & MASK9 as u64) as u32).to_dec() as u64;
        let mid = UTer9(((self.0 >> 18) & MASK9 as u64) as u32).to_dec() as u64;
        let hi  = UTer9((self.0 >> 36) as u32).to_dec() as u64;
        lo + mid * 19_683u64 + hi * 387_420_489u64
    }

    #[inline(always)] pub const fn raw(self) -> u64 { self.0 }

    /// Jones `uter27_add`: O(1) BCT addition.
    ///
    /// ```
    /// use balanced_ternary::UTer27;
    /// assert_eq!((UTer27::from_dec(500_000) + UTer27::from_dec(500_000)).to_dec(), 1_000_000);
    /// ```
    #[inline]
    pub const fn uter_add(self, other: Self) -> Self {
        let a = self.0;
        let b = other.0;
        let c = a.wrapping_add(MASK27_L);
        let d = b.wrapping_add(c);
        let e = (b ^ c) ^ d;
        let e = !e & 0x0055_5555_5555_5554u64;
        let e = e >> 2;
        Self(d.wrapping_sub(e) & MASK27)
    }

    #[inline] pub const fn uter_sub(self, other: Self) -> Self {
        let comp2 = (MASK27_H.wrapping_sub(other.0)) & MASK27;
        let comp3 = Self(comp2).uter_add(Self(0x01)); // +1
        self.uter_add(comp3)
    }

    #[inline(always)] pub const fn trit_and(self, o: Self) -> Self { Self(il_and_u64(self.0, o.0, MASK27_L)) }
    #[inline(always)] pub const fn trit_or (self, o: Self) -> Self { Self(il_or_u64 (self.0, o.0, MASK27_L)) }
    #[inline(always)] pub const fn trit_xor(self, o: Self) -> Self { Self(il_xor_u64(self.0, o.0, MASK27_L)) }
}

impl Add for UTer27 { type Output = Self; fn add(self, r: Self) -> Self { self.uter_add(r) } }
impl Sub for UTer27 { type Output = Self; fn sub(self, r: Self) -> Self { self.uter_sub(r) } }
impl Mul for UTer27 { type Output = Self; fn mul(self, r: Self) -> Self { Self::from_dec(self.to_dec() * r.to_dec()) } }
impl Div for UTer27 { type Output = Self; fn div(self, r: Self) -> Self { Self::from_dec(self.to_dec() / r.to_dec()) } }
impl BitAnd for UTer27 { type Output = Self; fn bitand(self, r: Self) -> Self { self.trit_and(r) } }
impl BitOr  for UTer27 { type Output = Self; fn bitor (self, r: Self) -> Self { self.trit_or(r)  } }
impl BitXor for UTer27 { type Output = Self; fn bitxor(self, r: Self) -> Self { self.trit_xor(r) } }
impl Shl<usize> for UTer27 {
    type Output = Self;
    fn shl(self, n: usize) -> Self {
        if n >= 27 { return Self::ZERO; }
        Self((self.0 << (2 * n)) & MASK27)
    }
}
impl Shr<usize> for UTer27 {
    type Output = Self;
    fn shr(self, n: usize) -> Self {
        if n >= 27 { return Self::ZERO; }
        Self((self.0 >> (2 * n)) & MASK27)
    }
}
impl From<u64>   for UTer27 { fn from(v: u64)   -> Self { Self::from_dec(v) } }
impl From<UTer27> for u64   { fn from(t: UTer27) -> u64  { t.to_dec() } }
impl core::fmt::Display for UTer27 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dec())
    }
}

// =========================================================================
// BTer9 — 9-trit balanced ternary (bter9_t), range −9841..+9841
// =========================================================================

/// 9-trit balanced ternary packed in a `u32` (18 bits used).
///
/// Matches Jones's `bter9_t`.  Encoding: `00=−1, 01=0, 10=+1`.
///
/// ```
/// use balanced_ternary::BTer9;
///
/// let a = BTer9::from_dec(13);   // "+++"
/// let b = BTer9::from_dec(-5);   // "-++"
/// assert_eq!((a + b).to_dec(), 8);
/// assert_eq!((a & b).to_dec(), -5);  // AND = min
/// assert_eq!((a | b).to_dec(), 13);  // OR  = max
/// assert_eq!((-a).to_dec(), -13);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct BTer9(u32);

/// Bias for 9-trit balanced ternary: (3^9 − 1)/2 = 9841.
const BTER9_BIAS: u32 = 9841;

/// Precomputed 6-bit IL code for each 3-trit unsigned value 0..26.
/// CHUNK3[k] = (trit0_code) | (trit1_code << 2) | (trit2_code << 4)
/// where trit_i_code ∈ {0,1,2} (0→00, 1→01, 2→10 in BCT).
/// Used by BTer9::from_dec and BTer27::from_dec to process 3 trits per divmod.
const CHUNK3: [u8; 27] = {
    let mut t = [0u8; 27];
    let mut k = 0usize;
    while k < 27 {
        let c0 = (k % 3) as u8;
        let c1 = ((k / 3) % 3) as u8;
        let c2 = (k / 9) as u8;
        t[k] = c0 | (c1 << 2) | (c2 << 4);
        k += 1;
    }
    t
};

impl BTer9 {
    pub const ZERO: Self = Self(MASK9_L);  // all 01 = 0
    pub const MAX:  Self = Self(MASK9_H);  // all 10 = +1 → +9841
    pub const MIN:  Self = Self(0);        // all 00 = −1 → −9841

    #[inline] pub fn from_dec(v: i32) -> Self {
        // 3-trit chunk lookup: 3 iterations of divmod-by-27 instead of 9 of divmod-by-3.
        let mut word = 0u32;
        let mut n = (v + BTER9_BIAS as i32) as u32; // bias to unsigned 9-trit range
        for k in 0..3u32 {
            let chunk = (n % 27) as usize;
            word |= (CHUNK3[chunk] as u32) << (6 * k);
            n /= 27;
        }
        Self(word)
    }

    #[inline] pub fn to_dec(&self) -> i32 {
        // Reuse UTer9's LUT-based decoder: BTer9 is UTer9 with a fixed bias.
        UTer9(self.0).to_dec() as i32 - BTER9_BIAS as i32
    }

    #[inline(always)] pub const fn raw(self) -> u32 { self.0 }

    /// Convert to the unsigned BCT representation (adds 1 per trit via the
    /// Jones biased encoding: balanced 0 = unsigned `BTER9_BIAS`).
    #[inline] pub fn to_uter9(self) -> UTer9 { UTer9::from(self) }

    #[inline] pub const fn il_neg(self) -> Self { Self(il_neg_u32(self.0, MASK9_L)) }
    #[inline(always)] pub const fn il_and(self, o: Self) -> Self { Self(il_and_u32(self.0, o.0, MASK9_L)) }
    #[inline(always)] pub const fn il_or (self, o: Self) -> Self { Self(il_or_u32 (self.0, o.0, MASK9_L)) }
    #[inline(always)] pub const fn il_xor(self, o: Self) -> Self { Self(il_xor_u32(self.0, o.0, MASK9_L)) }

    /// Trit-wise logical left shift, fill with Zero.
    #[inline(always)] pub const fn il_shl(self, n: usize) -> Self {
        if n >= 9 { return Self::ZERO; }
        let fill = MASK9_L & ((1u32 << (2 * n)).wrapping_sub(1));
        Self((self.0 << (2 * n)) | fill)
    }
    /// Trit-wise logical right shift, fill with Zero.
    #[inline(always)] pub const fn il_shr(self, n: usize) -> Self {
        if n >= 9 { return Self::ZERO; }
        let fill = MASK9_L & (u32::MAX << (18 - 2 * n));
        Self(((self.0 >> (2 * n)) | fill) & MASK9)
    }
}

impl Add for BTer9 {
    type Output = Self;
    #[inline]
    fn add(self, r: Self) -> Self {
        // Biased UTer9 trick: bter_add(a,b) = uter_add(uter_add(a,b), REMAP)
        let s = uter_add_u32_raw(self.0, r.0, MASK9_L);
        Self(uter_add_u32_raw(s, BTER9_REMAP, MASK9_L))
    }
}
impl Sub for BTer9 {
    type Output = Self;
    #[inline]
    fn sub(self, r: Self) -> Self {
        // sub(a,b) = add(a, -b); il_neg is O(1)
        let neg_r = il_neg_u32(r.0, MASK9_L);
        let s = uter_add_u32_raw(self.0, neg_r, MASK9_L);
        Self(uter_add_u32_raw(s, BTER9_REMAP, MASK9_L))
    }
}
impl Mul for BTer9 { type Output = Self; fn mul(self, r: Self) -> Self { Self::from_dec(self.to_dec() * r.to_dec()) } }
impl Div for BTer9 { type Output = Self; fn div(self, r: Self) -> Self { Self::from_dec(self.to_dec() / r.to_dec()) } }
impl Neg for BTer9 { type Output = Self; fn neg(self) -> Self { self.il_neg() } }
impl Not for BTer9 { type Output = Self; fn not(self) -> Self { self.il_neg() } }
impl BitAnd for BTer9 { type Output = Self; fn bitand(self, r: Self) -> Self { self.il_and(r) } }
impl BitOr  for BTer9 { type Output = Self; fn bitor (self, r: Self) -> Self { self.il_or(r)  } }
impl BitXor for BTer9 { type Output = Self; fn bitxor(self, r: Self) -> Self { self.il_xor(r) } }
impl Shl<usize> for BTer9 { type Output = Self; fn shl(self, n: usize) -> Self { self.il_shl(n) } }
impl Shr<usize> for BTer9 { type Output = Self; fn shr(self, n: usize) -> Self { self.il_shr(n) } }
impl From<i32>  for BTer9 { fn from(v: i32)  -> Self { Self::from_dec(v) } }
impl From<BTer9> for i32  { fn from(t: BTer9) -> i32  { t.to_dec() } }
impl core::fmt::Display for BTer9 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dec())
    }
}

// =========================================================================
// BTer27 — 27-trit balanced ternary (bter27_t), range ±3_762_798_742_493
// =========================================================================

/// 27-trit balanced ternary packed in a `u64` (54 bits used).
///
/// Matches Jones's `bter27_t`.  Encoding: `00=−1, 01=0, 10=+1`.
///
/// ```
/// use balanced_ternary::BTer27;
///
/// let a = BTer27::from_dec(13);   // "+++"
/// let b = BTer27::from_dec(-5);   // "-++"
/// assert_eq!((a + b).to_dec(), 8);
/// assert_eq!((a & b).to_dec(), -5);  // AND = min
/// assert_eq!((a | b).to_dec(), 13);  // OR  = max
/// assert_eq!((-a).to_dec(), -13);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct BTer27(u64);

impl BTer27 {
    pub const ZERO: Self = Self(MASK27_L);
    pub const MAX:  Self = Self(MASK27_H);
    pub const MIN:  Self = Self(0);

    #[inline] pub fn from_dec(v: i64) -> Self {
        // 3-trit chunk lookup: 9 iterations of divmod-by-27 instead of 27 of divmod-by-3.
        const BIAS: i64 = 3_812_798_742_493; // (3^27 - 1) / 2
        let mut word = 0u64;
        let mut n = (v + BIAS) as u64; // bias to unsigned 27-trit range
        for k in 0..9u32 {
            let chunk = (n % 27) as usize;
            word |= (CHUNK3[chunk] as u64) << (6 * k);
            n /= 27;
        }
        Self(word)
    }

    #[inline] pub fn to_dec(&self) -> i64 {
        // Three independent 9-trit groups — LLVM schedules all three Horner
        // chains in parallel via ILP instead of one serial 27-step chain.
        let w = self.0;
        let lo  = BTer9((w         & MASK9 as u64) as u32).to_dec() as i64;
        let mid = BTer9(((w >> 18) & MASK9 as u64) as u32).to_dec() as i64;
        let hi  = BTer9(((w >> 36) & MASK9 as u64) as u32).to_dec() as i64;
        lo + mid * 19683 + hi * (19683 * 19683)
    }

    #[inline(always)] pub const fn raw(self) -> u64 { self.0 }

    #[inline(always)] pub const fn il_neg(self) -> Self { Self(il_neg_u64(self.0, MASK27_L)) }
    #[inline(always)] pub const fn il_and(self, o: Self) -> Self { Self(il_and_u64(self.0, o.0, MASK27_L)) }
    #[inline(always)] pub const fn il_or (self, o: Self) -> Self { Self(il_or_u64 (self.0, o.0, MASK27_L)) }
    #[inline(always)] pub const fn il_xor(self, o: Self) -> Self { Self(il_xor_u64(self.0, o.0, MASK27_L)) }

    #[inline(always)] pub const fn il_shl(self, n: usize) -> Self {
        if n >= 27 { return Self::ZERO; }
        let fill = MASK27_L & ((1u64 << (2 * n)).wrapping_sub(1));
        Self((self.0 << (2 * n)) | fill)
    }
    #[inline(always)] pub const fn il_shr(self, n: usize) -> Self {
        if n >= 27 { return Self::ZERO; }
        let fill = MASK27_L & (u64::MAX << (54 - 2 * n));
        Self(((self.0 >> (2 * n)) | fill) & MASK27)
    }
}

impl Add for BTer27 {
    type Output = Self;
    #[inline]
    fn add(self, r: Self) -> Self {
        // Biased UTer27 trick: bter_add(a,b) = uter_add(uter_add(a,b), REMAP)
        let s = uter_add_u64_raw(self.0, r.0, MASK27_L);
        Self(uter_add_u64_raw(s, BTER27_REMAP, MASK27_L))
    }
}
impl Sub for BTer27 {
    type Output = Self;
    #[inline]
    fn sub(self, r: Self) -> Self {
        // sub(a,b) = add(a, -b); il_neg is O(1)
        let neg_r = il_neg_u64(r.0, MASK27_L);
        let s = uter_add_u64_raw(self.0, neg_r, MASK27_L);
        Self(uter_add_u64_raw(s, BTER27_REMAP, MASK27_L))
    }
}
impl Mul for BTer27 { type Output = Self; fn mul(self, r: Self) -> Self { Self::from_dec(self.to_dec() * r.to_dec()) } }
impl Div for BTer27 { type Output = Self; fn div(self, r: Self) -> Self { Self::from_dec(self.to_dec() / r.to_dec()) } }
impl Neg for BTer27 { type Output = Self; fn neg(self) -> Self { self.il_neg() } }
impl Not for BTer27 { type Output = Self; fn not(self) -> Self { self.il_neg() } }
impl BitAnd for BTer27 { type Output = Self; fn bitand(self, r: Self) -> Self { self.il_and(r) } }
impl BitOr  for BTer27 { type Output = Self; fn bitor (self, r: Self) -> Self { self.il_or(r)  } }
impl BitXor for BTer27 { type Output = Self; fn bitxor(self, r: Self) -> Self { self.il_xor(r) } }
impl Shl<usize> for BTer27 { type Output = Self; fn shl(self, n: usize) -> Self { self.il_shl(n) } }
impl Shr<usize> for BTer27 { type Output = Self; fn shr(self, n: usize) -> Self { self.il_shr(n) } }
impl From<i64>   for BTer27 { fn from(v: i64)   -> Self { Self::from_dec(v) } }
impl From<BTer27> for i64   { fn from(t: BTer27) -> i64  { t.to_dec() } }
impl core::fmt::Display for BTer27 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_dec())
    }
}

// --- Cross-type conversions ---

impl From<UTer9> for BTer9 {
    /// Convert unsigned BCT to balanced by subtracting bias (1 per trit).
    fn from(u: UTer9) -> Self { Self::from_dec(u.to_dec() as i32 - BTER9_BIAS as i32) }
}
impl From<BTer9> for UTer9 {
    /// Convert balanced BCT to unsigned by adding bias (1 per trit).
    fn from(b: BTer9) -> Self { UTer9::from_dec((b.to_dec() + BTER9_BIAS as i32) as u32) }
}
impl From<UTer27> for BTer27 {
    fn from(u: UTer27) -> Self { Self::from_dec(u.to_dec() as i64 - 3_812_798_742_493i64) }
}
impl From<BTer27> for UTer27 {
    fn from(b: BTer27) -> Self { UTer27::from_dec((b.to_dec() + 3_812_798_742_493i64) as u64) }
}
impl From<BTer9> for BTer27 {
    fn from(b: BTer9) -> Self { Self::from_dec(b.to_dec() as i64) }
}
impl From<BTer27> for BTer9 {
    fn from(b: BTer27) -> Self { Self::from_dec(b.to_dec() as i32) }
}

// --- Unit tests ---

#[cfg(test)]
#[test]
fn uter9_roundtrip() {
    for v in [0u32, 1, 2, 9, 42, 100, 9841, 19682] {
        assert_eq!(UTer9::from_dec(v).to_dec(), v, "UTer9 roundtrip {v}");
    }
}

#[cfg(test)]
#[test]
fn uter9_add_matches_naive() {
    for a in [0u32, 1, 5, 42, 100, 9840] {
        for b in [0u32, 1, 5, 42, 100] {
            let expected = a + b;
            let got = (UTer9::from_dec(a) + UTer9::from_dec(b)).to_dec();
            assert_eq!(got, expected, "UTer9 add {a}+{b}");
        }
    }
}

#[cfg(test)]
#[test]
fn uter27_roundtrip() {
    for v in [0u64, 1, 42, 1_000_000, 7_625_597_484_986] {
        assert_eq!(UTer27::from_dec(v).to_dec(), v, "UTer27 roundtrip {v}");
    }
}

#[cfg(test)]
#[test]
fn uter27_add_matches_naive() {
    for a in [0u64, 1, 5, 42, 1_000_000] {
        for b in [0u64, 1, 5, 42, 1_000_000] {
            assert_eq!((UTer27::from_dec(a) + UTer27::from_dec(b)).to_dec(), a + b);
        }
    }
}

#[cfg(test)]
#[test]
fn bter9_roundtrip() {
    for v in [-9841i32, -100, -1, 0, 1, 42, 100, 9841] {
        assert_eq!(BTer9::from_dec(v).to_dec(), v, "BTer9 roundtrip {v}");
    }
}

#[cfg(test)]
#[test]
fn bter9_logical_ops() {
    let a = BTer9::from_dec(13);  // "+++"
    let b = BTer9::from_dec(-5);  // "-++"
    assert_eq!((a & b).to_dec(), -5);
    assert_eq!((a | b).to_dec(), 13);
    assert_eq!((a ^ b).to_dec(), 5);
    assert_eq!((-a).to_dec(), -13);
}

#[cfg(test)]
#[test]
fn bter27_roundtrip() {
    for v in [-3_762_798_742_493i64, -100, -1, 0, 1, 42, 1_000_000] {
        assert_eq!(BTer27::from_dec(v).to_dec(), v, "BTer27 roundtrip {v}");
    }
}

#[cfg(test)]
#[test]
fn bter27_logical_ops() {
    let a = BTer27::from_dec(13);
    let b = BTer27::from_dec(-5);
    assert_eq!((a & b).to_dec(), -5);
    assert_eq!((a | b).to_dec(), 13);
    assert_eq!((-a).to_dec(), -13);
}

#[cfg(test)]
#[test]
fn bter9_bter27_match_bct32() {
    // All logical ops on BTer9/27 must agree with IlBctTer32 for same values.
    let vals: &[i32] = &[-13, -5, -1, 0, 1, 5, 13];
    for &a in vals {
        for &b in vals {
            let b9a = BTer9::from_dec(a); let b9b = BTer9::from_dec(b);
            let ila = IlBctTer32::from_dec(a as i64);
            let ilb = IlBctTer32::from_dec(b as i64);
            assert_eq!((b9a & b9b).to_dec() as i64, (ila & ilb).to_dec(), "AND b9 vs il32 a={a} b={b}");
            assert_eq!((b9a | b9b).to_dec() as i64, (ila | ilb).to_dec(), "OR  b9 vs il32 a={a} b={b}");
            assert_eq!((b9a ^ b9b).to_dec() as i64, (ila ^ ilb).to_dec(), "XOR b9 vs il32 a={a} b={b}");
        }
    }
}

// ---- BctTer32 unit tests ----------------------------------------------------

#[cfg(test)]
#[test]
fn bct_roundtrip() {
    for v in [-13i64, -5, -1, 0, 1, 5, 13, 42, 12345, -12345] {
        assert_eq!(BctTer32::from_dec(v).to_dec(), v, "roundtrip failed for {v}");
    }
}

#[cfg(test)]
#[test]
fn bct_bitwise_ops() {
    // a = 13 = "+++", b = -5 = "-++"
    let a = BctTer32::from_dec(13);
    let b = BctTer32::from_dec(-5);

    assert_eq!((a & b).to_dec(), -5);          // AND = min
    assert_eq!((a | b).to_dec(), 13);           // OR  = max
    assert_eq!((a ^ b).to_dec(), 5);            // XOR = -(a*b)
    assert_eq!((-a).to_dec(), -13);             // NOT
    assert_eq!(a.bct_consensus(b).to_dec(), 4); // consensus: trits 0 and 1 agree (+,+); trit 2 differs (+,−)→0
    assert_eq!(a.bct_accept_anything(BctTer32::ZERO), a);    // zero transparent
    assert_eq!(a.bct_accept_anything(-a), BctTer32::ZERO);   // conflict = zero
}

#[cfg(test)]
#[test]
fn bct_shifts() {
    assert_eq!((BctTer32::from_dec(1) << 1usize).to_dec(), 3);
    assert_eq!((BctTer32::from_dec(1) << 2usize).to_dec(), 9);
    assert_eq!((BctTer32::from_dec(9) >> 1usize).to_dec(), 3);
    assert_eq!((BctTer32::from_dec(9) >> 2usize).to_dec(), 1);
    assert_eq!((BctTer32::from_dec(1) << 32usize), BctTer32::ZERO);
    assert_eq!((BctTer32::from_dec(1) >> 32usize), BctTer32::ZERO);
}

#[cfg(test)]
#[test]
fn bct_from_ternary_roundtrip() {
    let t = Ternary::parse("+-0+-0+-0+");
    let bct = BctTer32::from_ternary(&t);
    assert_eq!(bct.to_ternary().trim().to_string(), t.to_string());
}

#[cfg(test)]
#[test]
fn bct_shift_matches_expected() {
    let v = 81i64; // 3^4
    let bct = BctTer32::from_dec(v);
    assert_eq!((bct >> 2usize).to_dec(), 9);   // 81 / 9 = 9
    assert_eq!((bct << 1usize).to_dec(), 243); // 81 * 3 = 243
}

#[cfg(test)]
#[test]
fn bct32_fast_add_sub() {
    let vals: &[i64] = &[-1_000_000, -100, -13, -5, -1, 0, 1, 5, 13, 42, 100, 1_000_000];
    for &a in vals {
        for &b in vals {
            let ba = BctTer32::from_dec(a);
            let bb = BctTer32::from_dec(b);
            assert_eq!((ba + bb).to_dec(), a + b, "BctTer32 add a={a} b={b}");
            assert_eq!((ba - bb).to_dec(), a - b, "BctTer32 sub a={a} b={b}");
        }
    }
}

#[cfg(test)]
#[test]
fn ilbct32_fast_add_sub() {
    let vals: &[i64] = &[-1_000_000, -100, -13, -5, -1, 0, 1, 5, 13, 42, 100, 1_000_000];
    for &a in vals {
        for &b in vals {
            let ia = IlBctTer32::from_dec(a);
            let ib = IlBctTer32::from_dec(b);
            assert_eq!((ia + ib).to_dec(), a + b, "IlBctTer32 add a={a} b={b}");
            assert_eq!((ia - ib).to_dec(), a - b, "IlBctTer32 sub a={a} b={b}");
        }
    }
}
