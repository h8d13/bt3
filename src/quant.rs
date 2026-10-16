//! Block codecs for ternary weight formats used by LLM runtimes.
//!
//! Bit-exact with the published block layouts. Byte-level only: file parsing
//! (GGUF headers, tensor names, alignment) stays with the caller. Block
//! scales are returned as raw IEEE fp16 bits, so no float code is needed here.
//!
//! Every format here holds [`BLOCK_TRITS`] trits per block and decodes to 4
//! [`BctTer64`] words. Inside a block, trits come in runs of consecutive
//! elements; each run is one [`IlBctTer32`] word (`t + 1` per 2 bits: `00` = −1,
//! `01` = 0, `10` = +1) and never crosses a 64-trit word.
//!
//! # TQ2_0
//!
//! 66 bytes: `qs[64]`, then `d` (fp16, little-endian). Each trit is its
//! 2-bit code. Element `e` lives in byte `32 * (e / 128) + e % 32` at bit
//! `2 * ((e / 32) % 4)`: 8 runs of 32, run `i` being bit plane `2 * (i % 4)`
//! of bytes `32 * (i / 4) ..+ 32`.
//!
//! # TQ1_0
//!
//! 54 bytes: `qs[48]`, `qh[4]`, then `d`. Each byte holds a base-3 value `v`
//! (digits = codes, most significant first) as the fraction `⌈v · 256 / 243⌉`,
//! so `(byte · 243) >> 8` gives `v` back for every byte value. Digit `n` of
//! byte `m` in a segment is trit `first + n * len + m`:
//!
//! | bytes | `len` | digits | `first` |
//! |-------|-------|--------|---------|
//! | `qs[0..32]`  | 32 | 5 | 0   |
//! | `qs[32..48]` | 16 | 5 | 160 |
//! | `qh[0..4]`   | 4  | 4 | 240 |
//!
//! `qh` bytes store `3 · v` (4 digits, the lowest is always 0).

use crate::store::{BctTer32, BctTer64, IlBctTer32, CHUNK5_DEC, UTER5_LUT};

/// Trits per block, for every format in this module.
pub const BLOCK_TRITS: usize = 256;

/// Bytes per TQ2_0 block: 64 bytes of codes + 2 bytes of fp16 scale.
pub const TQ2_0_BLOCK_BYTES: usize = 66;

/// Bytes per TQ1_0 block: 48 + 4 bytes of packed codes + 2 bytes of fp16 scale.
pub const TQ1_0_BLOCK_BYTES: usize = 54;

/// Low bit of each 2-bit pair: a code with both bits set (`11`) is invalid.
const PAIR_L: u64 = 0x5555_5555_5555_5555;

/// TQ1_0 segments: `(byte offset, len, digits, first trit)`.
const TQ1_0_SEGMENTS: [(usize, usize, usize, usize); 3] =
    [(0, 32, 5, 0), (32, 16, 5, 160), (48, 4, 4, 240)];

// ── runs ─────────────────────────────────────────────────────────────────────

/// OR the first `len` trits of IL word `raw` into `pos`/`neg` at trit `start`.
#[inline(always)]
fn put_run(pos: &mut [u64; 4], neg: &mut [u64; 4], start: usize, len: usize, raw: u64) {
    // pairs past `len` are 00 (−1) here, so the mask is what drops them
    let half = IlBctTer32::from_raw(raw).to_bct();
    let mask = (1u64 << len) - 1;
    let (w, at) = (start / 64, start % 64);
    pos[w] |= (half.pos_mask() as u64 & mask) << at;
    neg[w] |= (half.neg_mask() as u64 & mask) << at;
}

/// IL word of the 32 trits from trit `start` (fewer if the word ends first;
/// the rest read as Zero).
#[inline(always)]
fn get_run(words: &[BctTer64; 4], start: usize) -> u64 {
    let (w, at) = (words[start / 64], start % 64);
    let half = BctTer32::new((w.pos_mask() >> at) as u32, (w.neg_mask() >> at) as u32);
    IlBctTer32::from_bct(half).raw()
}

#[inline(always)]
fn to_words(pos: [u64; 4], neg: [u64; 4]) -> [BctTer64; 4] {
    core::array::from_fn(|w| BctTer64::new(pos[w], neg[w]))
}

// ── TQ2_0 ────────────────────────────────────────────────────────────────────

/// Decode one TQ2_0 block into 4 [`BctTer64`] words (trits 0..255) and the
/// raw fp16 bits of its scale.
///
/// Returns `None` if any code is `11`, which a valid encoder never writes.
///
/// # Example
/// ```
/// use balanced_ternary::quant::{tq2_0_decode, tq2_0_encode};
/// use balanced_ternary::BctTer64;
///
/// let words = [BctTer64::new(0b01, 0b10), BctTer64::ZERO, BctTer64::MAX, BctTer64::MIN];
/// let block = tq2_0_encode(&words, 0x3800); // d = 0.5
/// assert_eq!(tq2_0_decode(&block), Some((words, 0x3800)));
/// ```
pub fn tq2_0_decode(block: &[u8; TQ2_0_BLOCK_BYTES]) -> Option<([BctTer64; 4], u16)> {
    let (mut pos, mut neg) = ([0u64; 4], [0u64; 4]);
    for run in 0..8 {
        let shift = 2 * (run % 4);
        let mut raw = 0u64;
        for (m, &b) in block[32 * (run / 4)..][..32].iter().enumerate() {
            raw |= (((b >> shift) & 3) as u64) << (2 * m);
        }
        if raw & (raw >> 1) & PAIR_L != 0 {
            return None;
        }
        put_run(&mut pos, &mut neg, 32 * run, 32, raw);
    }
    Some((to_words(pos, neg), u16::from_le_bytes([block[64], block[65]])))
}

/// Encode 4 [`BctTer64`] words (trits 0..255) and raw fp16 scale bits `d`
/// into one TQ2_0 block. Inverse of [`tq2_0_decode`].
pub fn tq2_0_encode(words: &[BctTer64; 4], d: u16) -> [u8; TQ2_0_BLOCK_BYTES] {
    let mut block = [0u8; TQ2_0_BLOCK_BYTES];
    for run in 0..8 {
        let raw = get_run(words, 32 * run);
        let shift = 2 * (run % 4);
        for (m, b) in block[32 * (run / 4)..][..32].iter_mut().enumerate() {
            *b |= (((raw >> (2 * m)) & 3) as u8) << shift;
        }
    }
    block[64..].copy_from_slice(&d.to_le_bytes());
    block
}

// ── TQ1_0 ────────────────────────────────────────────────────────────────────

/// Decode one TQ1_0 block into 4 [`BctTer64`] words (trits 0..255) and the
/// raw fp16 bits of its scale.
///
/// Every byte value is valid, so this cannot fail.
///
/// # Example
/// ```
/// use balanced_ternary::quant::{tq1_0_decode, tq1_0_encode};
/// use balanced_ternary::BctTer64;
///
/// let words = [BctTer64::new(0b01, 0b10), BctTer64::ZERO, BctTer64::MAX, BctTer64::MIN];
/// let block = tq1_0_encode(&words, 0x3800); // d = 0.5
/// assert_eq!(tq1_0_decode(&block), (words, 0x3800));
/// ```
pub fn tq1_0_decode(block: &[u8; TQ1_0_BLOCK_BYTES]) -> ([BctTer64; 4], u16) {
    let (mut pos, mut neg) = ([0u64; 4], [0u64; 4]);
    for (off, len, digits, first) in TQ1_0_SEGMENTS {
        let mut runs = [0u64; 5];
        for (m, &b) in block[off..][..len].iter().enumerate() {
            // 5 digit codes of v, digit k at bits 2k: digit n (MSD first) is k = 4 − n
            let code = UTER5_LUT[(b as usize * 243) >> 8] as u64;
            for (n, run) in runs[..digits].iter_mut().enumerate() {
                *run |= ((code >> (2 * (4 - n))) & 3) << (2 * m);
            }
        }
        for (n, &raw) in runs[..digits].iter().enumerate() {
            put_run(&mut pos, &mut neg, first + n * len, len, raw);
        }
    }
    (to_words(pos, neg), u16::from_le_bytes([block[52], block[53]]))
}

/// Encode 4 [`BctTer64`] words (trits 0..255) and raw fp16 scale bits `d`
/// into one TQ1_0 block. Inverse of [`tq1_0_decode`]; the 13 byte values
/// that alias another `v` come back in their canonical `⌈v · 256 / 243⌉` form.
pub fn tq1_0_encode(words: &[BctTer64; 4], d: u16) -> [u8; TQ1_0_BLOCK_BYTES] {
    let mut block = [0u8; TQ1_0_BLOCK_BYTES];
    for (off, len, digits, first) in TQ1_0_SEGMENTS {
        let runs: [u64; 5] = core::array::from_fn(|n| {
            if n < digits { get_run(words, first + n * len) } else { 0 }
        });
        for (m, b) in block[off..][..len].iter_mut().enumerate() {
            // missing low digit (qh) stays 00 = digit 0, which is the "· 3"
            let code = runs[..digits].iter().enumerate()
                .fold(0, |c, (n, &raw)| c | ((raw >> (2 * m)) & 3) << (2 * (4 - n)));
            let v = (CHUNK5_DEC[code as usize] as i32 + 121) as u32;
            *b = (v * 256).div_ceil(243) as u8;
        }
    }
    block[52..].copy_from_slice(&d.to_le_bytes());
    block
}

#[cfg(test)]
mod tests {
    use super::*;

    // reference TQ2_0 quantizer output for x[i] = t[i] * 0.5, t from the LCG
    // below; generated externally and checked against a reference dequantizer
    #[rustfmt::skip]
    const GOLDEN: [u8; TQ2_0_BLOCK_BYTES] = [
        0x81, 0x64, 0x2a, 0x24, 0x52, 0x9a, 0x49, 0x10, 0xa5, 0x51, 0x64, 0x80,
        0x6a, 0x85, 0x16, 0xa8, 0x25, 0x06, 0x46, 0x98, 0x84, 0x1a, 0x11, 0x69,
        0x52, 0x05, 0x52, 0x28, 0x02, 0x21, 0x48, 0x66, 0x48, 0x61, 0x99, 0x22,
        0x24, 0x62, 0x54, 0x85, 0x42, 0x64, 0x56, 0x14, 0xa8, 0x60, 0x64, 0x2a,
        0x21, 0x99, 0x09, 0x09, 0x22, 0x45, 0x80, 0x66, 0x19, 0x26, 0x55, 0x16,
        0x59, 0x5a, 0x61, 0x09, 0x00, 0x38,
    ];

    fn lcg(s: &mut u64) -> u64 {
        *s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *s >> 33
    }

    fn trit(words: &[BctTer64; 4], e: usize) -> i8 {
        let (w, b) = (words[e / 64], e % 64);
        ((w.pos_mask() >> b) & 1) as i8 - ((w.neg_mask() >> b) & 1) as i8
    }

    fn rand_words(s: &mut u64) -> [BctTer64; 4] {
        core::array::from_fn(|_| {
            let (mut pos, mut neg) = (0u64, 0u64);
            for b in 0..64 {
                match lcg(s) % 3 {
                    0 => neg |= 1 << b,
                    2 => pos |= 1 << b,
                    _ => {}
                }
            }
            BctTer64::new(pos, neg)
        })
    }

    #[test]
    fn tq2_0_decode_matches_reference() {
        let (words, d) = tq2_0_decode(&GOLDEN).unwrap();
        assert_eq!(d, 0x3800, "fp16 0.5");
        // same LCG and seed as the C generator
        let mut s = 0x2545_f491_4f6c_dd1d;
        for e in 0..BLOCK_TRITS {
            assert_eq!(trit(&words, e), (lcg(&mut s) % 3) as i8 - 1, "trit {e}");
        }
    }

    #[test]
    fn tq2_0_encode_matches_reference() {
        let (words, d) = tq2_0_decode(&GOLDEN).unwrap();
        assert_eq!(tq2_0_encode(&words, d), GOLDEN);
    }

    #[test]
    fn tq2_0_roundtrip() {
        let mut s = 11;
        for _ in 0..64 {
            let words = rand_words(&mut s);
            let d = lcg(&mut s) as u16;
            assert_eq!(tq2_0_decode(&tq2_0_encode(&words, d)), Some((words, d)));
        }
    }

    #[test]
    fn tq2_0_decode_rejects_code_11() {
        // every bit plane of every byte: one bad code anywhere fails the block
        for i in 0..64 {
            for shift in (0..8).step_by(2) {
                let mut block = tq2_0_encode(&[BctTer64::ZERO; 4], 0);
                block[i] |= 3 << shift;
                assert_eq!(tq2_0_decode(&block), None, "byte {i} shift {shift}");
            }
        }
    }

    // reference TQ1_0 quantizer output, same inputs and LCG as GOLDEN
    #[rustfmt::skip]
    const GOLDEN_TQ1: [u8; TQ1_0_BLOCK_BYTES] = [
        0x5c, 0x34, 0xf8, 0x32, 0xb8, 0xf6, 0x92, 0x0b, 0x8e, 0x62, 0x35, 0x07,
        0xfa, 0x79, 0xd1, 0x55, 0x86, 0xc9, 0xcc, 0x4a, 0x25, 0xef, 0x5f, 0xa7,
        0xb9, 0x74, 0xb9, 0x4e, 0xac, 0x6b, 0x3e, 0xdf, 0xb3, 0x51, 0xf0, 0x4c,
        0x6f, 0x31, 0x60, 0x7b, 0x3e, 0x8d, 0x80, 0x7f, 0xfc, 0x51, 0x70, 0xf7,
        0x04, 0xcb, 0x46, 0x1d, 0x00, 0x38,
    ];

    #[test]
    fn tq1_0_decode_matches_reference() {
        let (words, d) = tq1_0_decode(&GOLDEN_TQ1);
        assert_eq!(d, 0x3800, "fp16 0.5");
        let mut s = 0x2545_f491_4f6c_dd1d;
        for e in 0..BLOCK_TRITS {
            assert_eq!(trit(&words, e), (lcg(&mut s) % 3) as i8 - 1, "trit {e}");
        }
    }

    #[test]
    fn tq1_0_encode_matches_reference() {
        let (words, d) = tq1_0_decode(&GOLDEN_TQ1);
        assert_eq!(tq1_0_encode(&words, d), GOLDEN_TQ1);
    }

    #[test]
    fn tq1_0_roundtrip() {
        let mut s = 23;
        for _ in 0..64 {
            let words = rand_words(&mut s);
            let d = lcg(&mut s) as u16;
            assert_eq!(tq1_0_decode(&tq1_0_encode(&words, d)), (words, d));
        }
    }

    #[test]
    fn tq1_0_decode_every_byte() {
        // reference digit extraction: digit n of byte b = ((b · 3ⁿ mod 256) · 3) >> 8;
        // covers the 13 bytes that alias another v as well
        for b in 0..=255u8 {
            let mut block = [0u8; TQ1_0_BLOCK_BYTES];
            for (off, ..) in TQ1_0_SEGMENTS {
                block[off] = b;
            }
            let (words, _) = tq1_0_decode(&block);
            for (_, len, digits, first) in TQ1_0_SEGMENTS {
                for n in 0..digits {
                    let q = b.wrapping_mul(3u8.pow(n as u32));
                    let want = ((q as u16 * 3) >> 8) as i8 - 1;
                    assert_eq!(trit(&words, first + n * len), want, "byte {b} digit {n}");
                }
            }
        }
    }
}
