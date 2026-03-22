//! Ternary matrix types built on [`BctTer64`] word storage.
//!
//! Each word holds 64 trits as two `u64` bitmasks (`pos`, `neg`).
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

use crate::store::BctTer64;

// ── helpers ──────────────────────────────────────────────────────────────────

#[inline(always)]
fn words_for(n: usize) -> usize { (n + 63) / 64 }

#[inline(always)]
fn word_set(w: BctTer64, b: u32, v: i8) -> BctTer64 {
    let mask = 1u64 << b;
    let pos = w.pos_mask() & !mask;
    let neg = w.neg_mask() & !mask;
    match v {
        1  => BctTer64::new(pos | mask, neg),
        -1 => BctTer64::new(pos, neg | mask),
        _  => BctTer64::new(pos, neg),
    }
}

#[inline(always)]
fn word_get(w: BctTer64, b: u32) -> i8 {
    let mask = 1u64 << b;
    if w.pos_mask() & mask != 0 { 1 }
    else if w.neg_mask() & mask != 0 { -1 }
    else { 0 }
}

// ── TernaryVec ───────────────────────────────────────────────────────────────

/// A dense ternary vector stored as packed [`BctTer64`] words.
///
/// Words are stored in little-endian trit order: word 0 holds trits 0..63,
/// word 1 holds trits 64..127, etc.
#[derive(Clone)]
pub struct TernaryVec {
    words: Vec<BctTer64>,
    len: usize,
}

impl TernaryVec {
    /// Create a zero vector of `len` trits.
    pub fn zeros(len: usize) -> Self {
        Self {
            words: (0..words_for(len)).map(|_| BctTer64::ZERO).collect(),
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
        let (w, b) = (i / 64, (i % 64) as u32);
        self.words[w] = word_set(self.words[w], b, v);
    }

    /// Get trit at position `i`: −1, 0, or +1.
    ///
    /// # Panics
    /// Panics if `i >= self.len()`.
    pub fn get(&self, i: usize) -> i8 {
        assert!(i < self.len, "trit index out of bounds");
        let (w, b) = (i / 64, (i % 64) as u32);
        word_get(self.words[w], b)
    }

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

/// A row-major ternary matrix stored as packed 64-trit BCT words.
///
/// Row `r` occupies words `data[r*row_stride .. (r+1)*row_stride]`.
#[derive(Clone)]
pub struct TernaryMatrix {
    data: Vec<BctTer64>,
    rows: usize,
    cols: usize,
    row_stride: usize,
}

impl TernaryMatrix {
    /// Create a zero matrix of shape `rows × cols`.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        let row_stride = words_for(cols);
        Self {
            data: (0..rows * row_stride).map(|_| BctTer64::ZERO).collect(),
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
        let idx = row * self.row_stride + col / 64;
        self.data[idx] = word_set(self.data[idx], (col % 64) as u32, v);
    }

    /// Get element at `(row, col)`: −1, 0, or +1.
    ///
    /// # Panics
    /// Panics if `row >= self.rows()` or `col >= self.cols()`.
    pub fn get(&self, row: usize, col: usize) -> i8 {
        assert!(row < self.rows && col < self.cols, "matrix index out of bounds");
        let idx = row * self.row_stride + col / 64;
        word_get(self.data[idx], (col % 64) as u32)
    }

    /// Matrix-vector product, writing results into a caller-provided slice.
    ///
    /// Avoids the heap allocation of [`matvec`](Self::matvec). Use this in hot paths.
    ///
    /// # Panics
    /// Panics if `x.len() != self.cols()` or `out.len() != self.rows()`.
    #[inline]
    pub fn matvec_into(&self, x: &TernaryVec, out: &mut [i32]) {
        assert_eq!(x.len(), self.cols, "matvec: vector length must match matrix columns");
        assert_eq!(out.len(), self.rows, "matvec: output length must match matrix rows");
        let xw = &x.words;
        let stride = self.row_stride;
        for r in 0..self.rows {
            let base = r * stride;
            let mut acc = 0i32;
            for w in 0..stride {
                acc += self.data[base + w].bct_dot_word(xw[w]);
            }
            out[r] = acc;
        }
    }

    /// Matrix-vector product `self · x` → `Vec<i32>` of length `self.rows()`.
    ///
    /// Allocates the output on every call. For repeated use in hot paths, prefer
    /// [`matvec_into`](Self::matvec_into) with a pre-allocated buffer.
    ///
    /// # Panics
    /// Panics if `x.len() != self.cols()`.
    pub fn matvec(&self, x: &TernaryVec) -> Vec<i32> {
        let mut out = alloc::vec![0i32; self.rows];
        self.matvec_into(x, &mut out);
        out
    }
}
