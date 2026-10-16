//! Block codecs for ternary weight formats used by LLM runtimes.
//!
//! Bit-exact with the published block layouts. Byte-level only: file parsing
//! (GGUF headers, tensor names, alignment) stays with the caller. Block
//! scales are returned as raw IEEE fp16 bits, so no float code is needed here.
//!
//! # TQ2_0
//!
//! 256 trits + one fp16 scale `d` in 66 bytes (`qs[64]`, then `d` little-endian).
//! Each trit `t` is stored as `t + 1` in 2 bits (`00` = −1, `01` = 0, `10` = +1),
//! the same code as [`IlBctTer32`]. Element `e` lives in byte
//! `32 * (e / 128) + e % 32` at bit `2 * ((e / 32) % 4)`.
//!
//! So the block is 8 runs of 32 consecutive trits: run `i` is bit plane
//! `2 * (i % 4)` of bytes `32 * (i / 4) ..+ 32`, which is one [`IlBctTer32`]
//! word and one half of the [`BctTer64`] word `i / 2`.

use crate::store::{BctTer32, BctTer64, IlBctTer32};

/// Trits per TQ2_0 block.
pub const TQ2_0_BLOCK_TRITS: usize = 256;

/// Bytes per TQ2_0 block: 64 bytes of codes + 2 bytes of fp16 scale.
pub const TQ2_0_BLOCK_BYTES: usize = 66;

/// Low bit of each 2-bit pair: a code with both bits set (`11`) is invalid.
const PAIR_L: u64 = 0x5555_5555_5555_5555;

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
    let mut pos = [0u64; 4];
    let mut neg = [0u64; 4];
    for run in 0..8 {
        let bytes = &block[32 * (run / 4)..][..32];
        let shift = 2 * (run % 4);
        let mut raw = 0u64;
        for (m, &b) in bytes.iter().enumerate() {
            raw |= (((b >> shift) & 3) as u64) << (2 * m);
        }
        if raw & (raw >> 1) & PAIR_L != 0 {
            return None;
        }
        let half = IlBctTer32::from_raw(raw).to_bct();
        let at = 32 * (run % 2);
        pos[run / 2] |= (half.pos_mask() as u64) << at;
        neg[run / 2] |= (half.neg_mask() as u64) << at;
    }
    let words = core::array::from_fn(|w| BctTer64::new(pos[w], neg[w]));
    Some((words, u16::from_le_bytes([block[64], block[65]])))
}

/// Encode 4 [`BctTer64`] words (trits 0..255) and raw fp16 scale bits `d`
/// into one TQ2_0 block. Inverse of [`tq2_0_decode`].
pub fn tq2_0_encode(words: &[BctTer64; 4], d: u16) -> [u8; TQ2_0_BLOCK_BYTES] {
    let mut block = [0u8; TQ2_0_BLOCK_BYTES];
    for run in 0..8 {
        let w = words[run / 2];
        let at = 32 * (run % 2);
        let half = BctTer32::new((w.pos_mask() >> at) as u32,
                                 (w.neg_mask() >> at) as u32);
        let raw = IlBctTer32::from_bct(half).raw();
        let shift = 2 * (run % 4);
        for (m, b) in block[32 * (run / 4)..][..32].iter_mut().enumerate() {
            *b |= (((raw >> (2 * m)) & 3) as u8) << shift;
        }
    }
    block[64..].copy_from_slice(&d.to_le_bytes());
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
        for e in 0..TQ2_0_BLOCK_TRITS {
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
}
