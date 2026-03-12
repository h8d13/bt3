//! Ternary matrix types built on [`BctTer32`] word storage.
//!
//! Each word encodes 32 trits in BCT (Binary-Coded Ternary) format.
//! The dot product uses 4 AND + 2 OR + 2 POPCNT per word — no multiplications.
//!
//! # Example
//! ```
//! use balanced_ternary::matrix::{TernaryMatrix, TernaryVec};
//!
//! let mut m = TernaryMatrix::zeros(2, 4);
//! m.set(0, 0,  1); m.set(0, 1, -1); m.set(0, 2,  1); m.set(0, 3,  0);
//! m.set(1, 0, -1); m.set(1, 1,  1); m.set(1, 2,  0); m.set(1, 3, -1);
//!
//! let mut x = TernaryVec::zeros(4);
//! x.set(0,  1); x.set(1,  1); x.set(2, -1); x.set(3,  1);
//!
//! let y = m.matvec(&x);
//! assert_eq!(y[0],  1 - 1 - 1 + 0);  // -1
//! assert_eq!(y[1], -1 + 1 + 0 - 1);  // -1
//! ```

extern crate alloc;
use alloc::vec::Vec;

use crate::store::BctTer32;

// ── helpers ──────────────────────────────────────────────────────────────────

/// Number of [`BctTer32`] words needed to hold `n` trits.
#[inline(always)]
fn words_for(n: usize) -> usize { (n + 31) / 32 }

/// Set a single trit bit in a BCT word pair.
#[inline(always)]
fn set_trit_in_word(w: BctTer32, bit: u32, v: i8) -> BctTer32 {
    let mask = 1u32 << bit;
    let mut pos = w.pos_mask() & !mask;
    let mut neg = w.neg_mask() & !mask;
    match v {
        1  => pos |= mask,
        -1 => neg |= mask,
        _  => {}
    }
    BctTer32::new(pos, neg)
}

/// Read a single trit from a BCT word.
#[inline(always)]
fn get_trit_from_word(w: BctTer32, bit: u32) -> i8 {
    let mask = 1u32 << bit;
    if w.pos_mask() & mask != 0 { 1 }
    else if w.neg_mask() & mask != 0 { -1 }
    else { 0 }
}

// ── TernaryVec ───────────────────────────────────────────────────────────────

/// A dense ternary vector stored as packed [`BctTer32`] words.
///
/// Words are stored in little-endian trit order: word 0 holds trits 0..31,
/// word 1 holds trits 32..63, etc.
pub struct TernaryVec {
    words: Vec<BctTer32>,
    len: usize,
}

impl TernaryVec {
    /// Create a zero vector of `len` trits.
    pub fn zeros(len: usize) -> Self {
        Self {
            words: (0..words_for(len)).map(|_| BctTer32::ZERO).collect(),
            len,
        }
    }

    /// Number of trits in this vector.
    #[inline(always)]
    pub fn len(&self) -> usize { self.len }

    /// True if the vector has no trits.
    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.len == 0 }

    /// Set trit at position `i` to `v` ∈ {−1, 0, +1}.
    ///
    /// # Panics
    /// Panics if `i >= self.len()`.
    pub fn set(&mut self, i: usize, v: i8) {
        assert!(i < self.len, "trit index out of bounds");
        let (w, b) = (i / 32, (i % 32) as u32);
        self.words[w] = set_trit_in_word(self.words[w], b, v);
    }

    /// Get trit at position `i`: −1, 0, or +1.
    ///
    /// # Panics
    /// Panics if `i >= self.len()`.
    pub fn get(&self, i: usize) -> i8 {
        assert!(i < self.len, "trit index out of bounds");
        let (w, b) = (i / 32, (i % 32) as u32);
        get_trit_from_word(self.words[w], b)
    }

    /// Raw word slice.
    #[inline(always)]
    pub fn words(&self) -> &[BctTer32] { &self.words }

    /// Ternary dot product with another vector of the same length.
    ///
    /// Returns `Σ self[i] * other[i]` as `i32`.
    ///
    /// # Panics
    /// Panics if lengths differ.
    ///
    /// # Example
    /// ```
    /// use balanced_ternary::matrix::TernaryVec;
    ///
    /// let mut a = TernaryVec::zeros(3);
    /// let mut b = TernaryVec::zeros(3);
    /// a.set(0, 1); a.set(1, -1); a.set(2, 1);
    /// b.set(0, 1); b.set(1,  1); b.set(2, 1);
    /// assert_eq!(a.dot(&b), 1); // +1 - 1 + 1
    /// ```
    pub fn dot(&self, other: &TernaryVec) -> i32 {
        dot(self, other)
    }
}

// ── dot ──────────────────────────────────────────────────────────────────────

/// Ternary dot product of two [`TernaryVec`]s.
///
/// Returns `Σ a[i] * b[i]` as `i32`.
///
/// # Panics
/// Panics if the vectors have different lengths.
pub fn dot(a: &TernaryVec, b: &TernaryVec) -> i32 {
    assert_eq!(a.len, b.len, "dot: vector lengths must match");
    a.words.iter().zip(b.words.iter()).map(|(wa, wb)| wa.bct_dot_word(*wb)).sum()
}

// ── TernaryMatrix ─────────────────────────────────────────────────────────────

/// A row-major ternary matrix stored as packed [`BctTer32`] words.
///
/// Row `r` occupies words `data[r*row_stride .. (r+1)*row_stride]`.
pub struct TernaryMatrix {
    data: Vec<BctTer32>,
    rows: usize,
    cols: usize,
    row_stride: usize,
}

impl TernaryMatrix {
    /// Create a zero matrix of shape `rows × cols`.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        let row_stride = words_for(cols);
        Self {
            data: (0..rows * row_stride).map(|_| BctTer32::ZERO).collect(),
            rows,
            cols,
            row_stride,
        }
    }

    /// Number of rows.
    #[inline(always)]
    pub fn rows(&self) -> usize { self.rows }

    /// Number of columns.
    #[inline(always)]
    pub fn cols(&self) -> usize { self.cols }

    /// Set element at `(row, col)` to `v` ∈ {−1, 0, +1}.
    ///
    /// # Panics
    /// Panics if `row >= self.rows()` or `col >= self.cols()`.
    pub fn set(&mut self, row: usize, col: usize, v: i8) {
        assert!(row < self.rows && col < self.cols, "matrix index out of bounds");
        let idx = row * self.row_stride + col / 32;
        self.data[idx] = set_trit_in_word(self.data[idx], (col % 32) as u32, v);
    }

    /// Get element at `(row, col)`: −1, 0, or +1.
    ///
    /// # Panics
    /// Panics if `row >= self.rows()` or `col >= self.cols()`.
    pub fn get(&self, row: usize, col: usize) -> i8 {
        assert!(row < self.rows && col < self.cols, "matrix index out of bounds");
        let idx = row * self.row_stride + col / 32;
        get_trit_from_word(self.data[idx], (col % 32) as u32)
    }

    /// Matrix-vector product `self · x` → `Vec<i32>` of length `self.rows()`.
    ///
    /// Each output element is the dot product of one matrix row with `x`:
    /// 4 AND + 2 OR + 2 POPCNT per 32-trit word, no multiplications.
    ///
    /// # Panics
    /// Panics if `x.len() != self.cols()`.
    pub fn matvec(&self, x: &TernaryVec) -> Vec<i32> {
        assert_eq!(x.len(), self.cols, "matvec: vector length must match matrix columns");
        let xw = x.words();
        (0..self.rows)
            .map(|r| {
                let row = &self.data[r * self.row_stride .. (r + 1) * self.row_stride];
                row.iter().zip(xw.iter()).map(|(wa, wb)| wa.bct_dot_word(*wb)).sum()
            })
            .collect()
    }
}
