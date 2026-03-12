//! TERSCII — ternary character encoding.
//!
//! A 9×9 character table (81 values, 4 trits each) designed for ternary computing.
//! See <https://homepage.divms.uiowa.edu/~jones/ternary/terscii.shtml>
//!
//! Each character maps to a value 0–80 represented as a [`Ternary`] number.

use crate::Ternary;
#[cfg(feature = "tryte")]
use crate::Tryte;
#[cfg(feature = "tryte")]
use alloc::{string::String, vec::Vec};

/// Inverse lookup table: ASCII code → TERSCII value (0–80), or -1 if not mapped.
///
/// Built from [`TABLE`]: covers all 128 ASCII code points.
/// Non-ASCII characters are rejected before this table is consulted.
#[rustfmt::skip]
const ENCODE_LUT: [i8; 128] = [
//   0     1     2     3     4     5     6     7     8     9    10    11    12    13    14    15
     0,    9,   18,   27,   36,   45,   54,   63,   72,   -1,   -1,   -1,   -1,   -1,   -1,   -1, // 0x00–0x0F (controls ES–SD)
    -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1,   -1, // 0x10–0x1F
     1,   64,   -1,   -1,   -1,   -1,   -1,   19,   -1,   -1,   -1,   -1,   28,   10,   55,   -1, // 0x20–0x2F  ' ','!',..,"'",..,,,-,.
     2,   11,   20,   29,   38,   47,   56,   65,   74,    3,   46,   37,   -1,   -1,   -1,   73, // 0x30–0x3F  0–9,:,;,..?
    -1,   12,   21,   30,   39,   48,   57,   66,   75,    4,   13,   22,   31,   40,   49,   58, // 0x40–0x4F  @,A–O
    67,   76,    5,   14,   23,   32,   41,   50,   59,   68,   77,   -1,   -1,   -1,   -1,    6, // 0x50–0x5F  P–Z,[,\,],^,_
    -1,   15,   24,   33,   42,   51,   60,   69,   78,    7,   16,   25,   34,   43,   52,   61, // 0x60–0x6F  `,a–o
    70,   79,    8,   17,   26,   35,   44,   53,   62,   71,   80,   -1,   -1,   -1,   -1,   -1, // 0x70–0x7F  p–z,{,|,},~,DEL
];

/// TERSCII table: index = character value (0–80).
///
/// Arranged as a 9×9 grid; value = row × 9 + column.
///
/// ```text
///      col: 0   1   2   3   4   5   6   7   8
/// row 0:   ES  SP   0   9   I   R   _   i   r
/// row 1:   EL   -   1   A   J   S   a   j   s
/// row 2:   ET   '   2   B   K   T   b   k   t
/// row 3:   LR   ,   3   C   L   U   c   l   u
/// row 4:   OP   ;   4   D   M   V   d   m   v
/// row 5:   RL   :   5   E   N   W   e   n   w
/// row 6:   SU   .   6   F   O   X   f   o   x
/// row 7:   HT   !   7   G   P   Y   g   p   y
/// row 8:   SD   ?   8   H   Q   Z   h   q   z
/// ```
pub const TABLE: &str =
    "\x00 09IR_ir\x01-1AJSajs\x02'2BKTbkt\x03,3CLUclu\
     \x04;4DMVdmv\x05:5ENWenw\x06.6FOXfox\x07!7GPYgpy\x08?8HQZhqz";

/// Encode a character to its TERSCII [`Ternary`] value (0–80).
///
/// Returns `None` if the character is not in the TERSCII table.
/// Uses an O(1) 128-entry LUT for all ASCII characters.
#[inline]
pub fn encode(c: char) -> Option<Ternary> {
    let code = c as u32;
    if code < 128 {
        let v = ENCODE_LUT[code as usize];
        if v >= 0 { Some(Ternary::from_dec(v as i64)) } else { None }
    } else {
        None
    }
}

/// Decode a [`Ternary`] value (0–80) back to its TERSCII character.
///
/// Returns `None` if the value is outside the 0–80 range.
/// Uses O(1) byte indexing since all TERSCII chars are single-byte ASCII.
#[inline]
pub fn decode(t: &Ternary) -> Option<char> {
    let v = t.to_dec();
    if v >= 0 && (v as usize) < 81 {
        Some(TABLE.as_bytes()[v as usize] as char)
    } else {
        None
    }
}

/// Precomputed `Tryte<5>` for each TERSCII value 0–80.
///
/// Eliminates `from_i64` (5 divisions per call) in the hot path of `encode_tryte`.
#[cfg(feature = "tryte")]
const TRYTE5_LUT: [Tryte<5>; 81] = {
    let mut lut = [Tryte::<5>::ZERO; 81];
    let mut i = 0usize;
    while i < 81 {
        lut[i] = Tryte::<5>::from_i64(i as i64);
        i += 1;
    }
    lut
};

/// Encode a character to its TERSCII value as a [`Tryte<5>`] (stack-allocated, no heap).
///
/// TERSCII uses 4-trit quartets (3⁴ = 81 values), but balanced ternary `Tryte<4>` only
/// reaches ±40, so `Tryte<5>` (range ±121) is required to cover the full 0–80 range.
#[cfg(feature = "tryte")]
#[inline]
pub fn encode_tryte(c: char) -> Option<Tryte<5>> {
    let code = c as u32;
    if code < 128 {
        let v = ENCODE_LUT[code as usize];
        if v >= 0 { Some(TRYTE5_LUT[v as usize]) } else { None }
    } else {
        None
    }
}

/// Decode a [`Tryte<5>`] TERSCII value back to a character.
///
/// Returns `None` if the value is outside 0–80.
/// Uses parallel constant multiplications (no serial Horner dependency chain)
/// to compute the decimal value, then a single unsigned bounds check (`v < 81`
/// after casting, which subsumes the `>= 0` check) before the `TABLE` lookup.
#[cfg(feature = "tryte")]
#[inline]
pub fn decode_tryte(t: Tryte<5>) -> Option<char> {
    let raw = t.to_digit_slice();
    let v = raw[0] as i8 as i64 * 81
          + raw[1] as i8 as i64 * 27
          + raw[2] as i8 as i64 * 9
          + raw[3] as i8 as i64 * 3
          + raw[4] as i8 as i64;
    if v >= 0 && (v as usize) < 81 {
        Some(TABLE.as_bytes()[v as usize] as char)
    } else {
        None
    }
}

/// A TERSCII-encoded string: a sequence of [`Tryte<5>`] values, one per character.
///
/// Implements [`Display`] as space-separated balanced-ternary tryte representations.
/// ```
#[cfg(feature = "tryte")]
pub struct TersciiString(Vec<Tryte<5>>);

#[cfg(feature = "tryte")]
impl core::ops::Deref for TersciiString {
    type Target = [Tryte<5>];
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[cfg(feature = "tryte")]
impl core::fmt::Display for TersciiString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut first = true;
        for t in &self.0 {
            if !first { f.write_str(" ")?; }
            core::fmt::Display::fmt(t, f)?;
            first = false;
        }
        Ok(())
    }
}

/// Encode a string into a [`TersciiString`].
///
/// Returns `None` if any character is not in the TERSCII table.
#[cfg(feature = "tryte")]
#[inline]
pub fn encode_str(s: &str) -> Option<TersciiString> {
    s.chars().map(encode_tryte).collect::<Option<Vec<_>>>().map(TersciiString)
}

/// Decode a [`TersciiString`] (or `&[Tryte<5>]`) back to a [`String`].
///
/// Returns `None` if any value is outside 0–80.
#[cfg(feature = "tryte")]
#[inline]
pub fn decode_str(trytes: &[Tryte<5>]) -> Option<String> {
    trytes.iter().map(|&t| decode_tryte(t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_known_values() {
        assert_eq!(encode(' ').unwrap().to_dec(), 1);
        assert_eq!(encode('H').unwrap().to_dec(), 75);
        assert_eq!(encode('e').unwrap().to_dec(), 51);
        assert_eq!(encode('l').unwrap().to_dec(), 34);
        assert_eq!(encode('o').unwrap().to_dec(), 61);
        assert_eq!(encode(',').unwrap().to_dec(), 28);
        assert_eq!(encode('W').unwrap().to_dec(), 50);
        assert_eq!(encode('r').unwrap().to_dec(), 8);
        assert_eq!(encode('d').unwrap().to_dec(), 42);
        assert_eq!(encode('!').unwrap().to_dec(), 64);
    }

    #[test]
    fn test_roundtrip_hello_world() {
        let msg: String = "Hello, World!"
            .chars()
            .map(|c| decode(&encode(c).unwrap()).unwrap())
            .collect();
        assert_eq!(msg, "Hello, World!");
    }

    #[test]
    fn test_roundtrip_all_printable() {
        let printable = " -',.;:.!?01234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";
        for c in printable.chars() {
            let t = encode(c).unwrap_or_else(|| panic!("encode({c:?}) failed"));
            let c2 = decode(&t).unwrap_or_else(|| panic!("decode failed for {c:?}"));
            assert_eq!(c, c2);
        }
    }

    #[test]
    fn test_decode_out_of_range() {
        assert!(decode(&Ternary::from_dec(-1)).is_none());
        assert!(decode(&Ternary::from_dec(81)).is_none());
        assert!(decode(&Ternary::from_dec(100)).is_none());
    }

    #[test]
    fn test_encode_unknown() {
        assert!(encode('€').is_none());
        assert!(encode('α').is_none());
    }

    #[test]
    fn test_table_length() {
        assert_eq!(TABLE.chars().count(), 81);
    }

    #[cfg(feature = "tryte")]
    #[test]
    fn test_encode_decode_str() {
        let encoded = encode_str("Hello, World!").unwrap();
        assert_eq!(encoded.len(), 13);
        assert_eq!(decode_str(&encoded).unwrap(), "Hello, World!");
    }

    #[cfg(feature = "tryte")]
    #[test]
    fn test_encode_str_unknown() {
        assert!(encode_str("Hello €").is_none());
    }
}
