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
fn words_for(n: usize) -> usize { n.div_ceil(64) }

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
        for (r, o) in out.iter_mut().enumerate() {
            let row = &self.data[r * stride..(r + 1) * stride];
            *o = row.iter().zip(xw).map(|(w, &xv)| w.bct_dot_word(xv)).sum();
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

    /// Matrix-vector product with `i8` activations, writing into `out`.
    ///
    /// Ternary weights against 8-bit activations. Each weight only selects
    /// `+x`, `-x` or nothing, so the row sum is `Σ x[pos] − Σ x[neg]` with
    /// no multiplications. Scalar reference; the per-trit loop is kept
    /// branch-free so it can auto-vectorize.
    ///
    /// # Panics
    /// Panics if `x.len() != self.cols()` or `out.len() != self.rows()`.
    ///
    /// # Example
    /// ```
    /// use balanced_ternary::matrix::TernaryMatrix;
    ///
    /// let mut m = TernaryMatrix::zeros(2, 3);
    /// m.set(0, 0, 1); m.set(0, 1, -1);
    /// m.set(1, 1, 1); m.set(1, 2,  1);
    ///
    /// let mut out = [0i32; 2];
    /// m.matvec_i8_into(&[10, 20, -30], &mut out);
    /// assert_eq!(out, [10 - 20, 20 - 30]);
    /// ```
    #[inline]
    pub fn matvec_i8_into(&self, x: &[i8], out: &mut [i32]) {
        assert_eq!(x.len(), self.cols, "matvec_i8: vector length must match matrix columns");
        assert_eq!(out.len(), self.rows, "matvec_i8: output length must match matrix rows");
        let stride = self.row_stride;
        for (r, o) in out.iter_mut().enumerate() {
            let row = &self.data[r * stride..(r + 1) * stride];
            let mut acc = 0i32;
            // last chunk may be short: trits past `cols` are never set, and
            // there is no x to read for them
            for (w, xs) in row.iter().zip(x.chunks(64)) {
                let (p, n) = (w.pos_mask(), w.neg_mask());
                for (b, &xi) in xs.iter().enumerate() {
                    let t = ((p >> b) & 1) as i32 - ((n >> b) & 1) as i32;
                    acc += t * xi as i32;
                }
            }
            *o = acc;
        }
    }

    /// Matrix-vector product with `i8` activations → `Vec<i32>`.
    ///
    /// Allocates the output on every call; prefer
    /// [`matvec_i8_into`](Self::matvec_i8_into) in hot paths.
    ///
    /// # Panics
    /// Panics if `x.len() != self.cols()`.
    pub fn matvec_i8(&self, x: &[i8]) -> Vec<i32> {
        let mut out = alloc::vec![0i32; self.rows];
        self.matvec_i8_into(x, &mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // small LCG: deterministic, no RNG dependency
    fn lcg(s: &mut u64) -> u64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *s >> 33
    }

    fn rand_mat(rows: usize, cols: usize, s: &mut u64) -> TernaryMatrix {
        let mut m = TernaryMatrix::zeros(rows, cols);
        for r in 0..rows {
            for c in 0..cols {
                m.set(r, c, (lcg(s) % 3) as i8 - 1);
            }
        }
        m
    }

    // element-wise reference through get(): independent of the word layout
    fn naive_i8(m: &TernaryMatrix, x: &[i8]) -> Vec<i32> {
        (0..m.rows())
            .map(|r| (0..m.cols()).map(|c| m.get(r, c) as i32 * x[c] as i32).sum())
            .collect()
    }

    #[test]
    fn matvec_i8_matches_naive() {
        let mut s = 0x2545_f491_4f6c_dd1d;
        // word edges: under, exactly, just past, and several words; 0 cols
        for &(rows, cols) in &[(1, 1), (3, 63), (4, 64), (5, 65), (7, 200), (2, 768), (3, 0)] {
            let m = rand_mat(rows, cols, &mut s);
            let x: Vec<i8> = (0..cols).map(|_| lcg(&mut s) as i8).collect();
            assert_eq!(m.matvec_i8(&x), naive_i8(&m, &x), "{rows}x{cols}");
        }
    }

    #[test]
    fn matvec_i8_extremes() {
        // all +1 weights against -128 and all -1 against 127: largest |sum|
        let cols = 768;
        let mut pos = TernaryMatrix::zeros(1, cols);
        let mut neg = TernaryMatrix::zeros(1, cols);
        for c in 0..cols {
            pos.set(0, c, 1);
            neg.set(0, c, -1);
        }
        assert_eq!(pos.matvec_i8(&alloc::vec![-128; cols]), [-128 * cols as i32]);
        assert_eq!(neg.matvec_i8(&alloc::vec![127; cols]), [-127 * cols as i32]);
    }

    #[test]
    fn matvec_i8_into_matches_alloc() {
        let mut s = 7;
        let m = rand_mat(9, 130, &mut s);
        let x: Vec<i8> = (0..130).map(|_| lcg(&mut s) as i8).collect();
        let mut out = alloc::vec![0i32; 9];
        m.matvec_i8_into(&x, &mut out);
        assert_eq!(out, m.matvec_i8(&x));
    }

    #[test]
    #[should_panic(expected = "matvec_i8: vector length")]
    fn matvec_i8_rejects_bad_len() {
        TernaryMatrix::zeros(2, 10).matvec_i8(&[0; 9]);
    }
}
