//! TERSCII — ternary character encoding.
//!
//! A 9×9 character table (81 values, 4 trits each) designed for ternary computing.
//! See <https://homepage.divms.uiowa.edu/~jones/ternary/terscii.shtml>
//!
//! Each character maps to a value 0–80, which is a 4-trit **unbalanced** ternary number.
//! The natural representation is [`TersciiCode`]: 4 trits in BCT (binary-coded ternary),
//! 2 bits per trit, stored in a `u8`. This gives symmetric O(1) encode/decode.
//!
//! For balanced-ternary arithmetic contexts, the value can also be converted to a
//! [`Ternary`] or [`Tryte<5>`] representation, but those are secondary encodings.

use crate::Ternary;

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

/// A TERSCII character in its natural 4-trit BCT (binary-coded ternary) representation.
///
/// Each of the 4 trits occupies a 2-bit pair: `00` = 0, `01` = 1, `10` = 2.
/// Layout: `t3 t2 t1 t0` (MSB first) packs into bits `[7:6][5:4][3:2][1:0]`.
/// Valid values: 0x00–0xAA (decimal 0–80, i.e. `0000₃`–`2222₃`).
///
/// This is the representation described in the TERSCII spec.
/// Use [`encode_code`] / [`decode_code`] for O(1) symmetric conversion.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct TersciiCode(u8);

impl TersciiCode {
    /// Returns the raw BCT4 byte.
    #[inline] pub const fn raw(self) -> u8 { self.0 }

    /// Constructs from a raw BCT4 byte without validity check.
    ///
    /// # Safety
    /// Caller must ensure no 2-bit group equals `11` (invalid BCT trit),
    /// and that the decimal value of the code is in 0–80.
    #[inline] pub const unsafe fn from_raw_unchecked(raw: u8) -> Self { Self(raw) }

    /// Extracts the four unbalanced ternary digits `[t3, t2, t1, t0]` (each 0–2).
    #[inline]
    pub const fn digits(self) -> [u8; 4] {
        [
            (self.0 >> 6) & 3,
            (self.0 >> 4) & 3,
            (self.0 >> 2) & 3,
             self.0       & 3,
        ]
    }
}

impl core::fmt::Display for TersciiCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let d = self.digits();
        write!(f, "{}{}{}{}", d[0], d[1], d[2], d[3])
    }
}

/// Converts a TERSCII decimal value 0–80 to its 4-trit BCT u8.
const fn dec_to_bct4(v: u8) -> u8 {
    let t3 = v / 27;
    let t2 = (v % 27) / 9;
    let t1 = (v % 9) / 3;
    let t0 = v % 3;
    (t3 << 6) | (t2 << 4) | (t1 << 2) | t0
}

/// ASCII code → TERSCII BCT4 raw byte (0xFF = not in TERSCII table).
///
/// Combines ENCODE_LUT + BCT4 conversion in a single precomputed 128-byte table.
const ENCODE_CODE_LUT: [u8; 128] = {
    let mut lut = [0xFFu8; 128];
    let mut i = 0usize;
    while i < 128 {
        let v = ENCODE_LUT[i];
        if v >= 0 {
            lut[i] = dec_to_bct4(v as u8);
        }
        i += 1;
    }
    lut
};

/// TERSCII BCT4 raw byte → ASCII byte (0xFF = invalid BCT or out of range).
///
/// Entries for all 81 valid TERSCII BCT4 values are set from [`TABLE`].
/// All other entries (invalid BCT patterns where any 2-bit group = `11`, or values
/// beyond 0xAA) remain 0xFF.
const DECODE_CODE_LUT: [u8; 256] = {
    let mut lut = [0xFFu8; 256];
    let table = TABLE.as_bytes();
    let mut v = 0usize;
    while v < 81 {
        lut[dec_to_bct4(v as u8) as usize] = table[v];
        v += 1;
    }
    lut
};

/// Encode a character to its TERSCII value as a [`TersciiCode`] (4-trit BCT, O(1)).
///
/// Returns `None` if the character is not in the TERSCII table.
///
/// # Performance
/// Single `ENCODE_CODE_LUT` lookup — symmetric with [`decode_code`].
#[inline]
pub fn encode_code(c: char) -> Option<TersciiCode> {
    let code = c as u32;
    if code < 128 {
        let raw = ENCODE_CODE_LUT[code as usize];
        if raw != 0xFF { Some(TersciiCode(raw)) } else { None }
    } else {
        None
    }
}

/// Decode a [`TersciiCode`] back to its character (O(1) single LUT lookup).
///
/// Returns `None` if the raw value contains an invalid BCT trit (any 2-bit pair = `11`).
///
/// # Performance
/// Single `DECODE_CODE_LUT` lookup — symmetric with [`encode_code`].
#[inline]
pub fn decode_code(code: TersciiCode) -> Option<char> {
    let byte = DECODE_CODE_LUT[code.0 as usize];
    if byte != 0xFF { Some(byte as char) } else { None }
}

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

/// Encode a string into a `Vec` of [`TersciiCode`]s.
///
/// Returns `None` if any character is not in the TERSCII table.
///
/// Uses `bytes()` (TERSCII is ASCII-only) and pre-allocates the Vec once.
#[inline]
pub fn encode_str(s: &str) -> Option<alloc::vec::Vec<TersciiCode>> {
    let mut v = alloc::vec::Vec::with_capacity(s.len());
    for b in s.bytes() {
        if b >= 128 { return None; }
        let raw = ENCODE_CODE_LUT[b as usize];
        if raw == 0xFF { return None; }
        v.push(TersciiCode(raw));
    }
    Some(v)
}

/// Decode a slice of [`TersciiCode`]s back to a `String`.
///
/// Returns `None` if any code contains an invalid BCT trit (2-bit group = `11`).
///
/// Pre-allocates the exact capacity and writes bytes directly, bypassing
/// `char::encode_utf8` — valid because all TERSCII chars are single-byte ASCII.
#[inline]
pub fn decode_codes(codes: &[TersciiCode]) -> Option<alloc::string::String> {
    let mut s = alloc::string::String::with_capacity(codes.len());
    for &code in codes {
        let byte = DECODE_CODE_LUT[code.0 as usize];
        if byte == 0xFF { return None; }
        // SAFETY: all TERSCII chars are ASCII (TABLE bytes 0x00–0x7A, all < 0x80),
        // so each byte is valid single-byte UTF-8.
        unsafe { s.as_mut_vec().push(byte); }
    }
    Some(s)
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

    #[test]
    fn test_encode_code_known_values() {
        // BCT4 encoding: v = t3*27 + t2*9 + t1*3 + t0, BCT = (t3<<6)|(t2<<4)|(t1<<2)|t0
        // 'H' = 75: 75 = 2*27+2*9+1*3+0 = [2,2,1,0] → BCT = 0b10_10_01_00 = 0xA4
        assert_eq!(encode_code('H').map(|c| c.raw()), Some(0xA4));
        // ' ' = 1:  1  = 0*27+0*9+0*3+1 = [0,0,0,1] → BCT = 0b00_00_00_01 = 0x01
        assert_eq!(encode_code(' ').map(|c| c.raw()), Some(0x01));
        // '0' = 2:  2  = 0*27+0*9+0*3+2 = [0,0,0,2] → BCT = 0b00_00_00_10 = 0x02
        assert_eq!(encode_code('0').map(|c| c.raw()), Some(0x02));
        assert!(encode_code('€').is_none());
    }

    #[test]
    fn test_decode_code_roundtrip_hello_world() {
        let msg: alloc::string::String = "Hello, World!"
            .chars()
            .map(|c| decode_code(encode_code(c).unwrap()).unwrap())
            .collect();
        assert_eq!(msg, "Hello, World!");
    }

    #[test]
    fn test_decode_code_invalid_bct() {
        // 0xFF = 0b11_11_11_11: all 2-bit groups are 11 → invalid BCT
        assert!(decode_code(TersciiCode(0xFF)).is_none());
        // 0x03 = 0b00_00_00_11: last group is 11 → invalid
        assert!(decode_code(TersciiCode(0x03)).is_none());
        // 0xAB = 0b10_10_10_11: last group is 11 → invalid
        assert!(decode_code(TersciiCode(0xAB)).is_none());
    }

    #[test]
    fn test_code_roundtrip_all_printable() {
        let printable = " -',.;:.!?01234567890ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";
        for c in printable.chars() {
            let code = encode_code(c).unwrap_or_else(|| panic!("encode_code({c:?}) failed"));
            let c2 = decode_code(code).unwrap_or_else(|| panic!("decode_code failed for {c:?}"));
            assert_eq!(c, c2);
        }
    }
}
