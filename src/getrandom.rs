//! `no_std` entropy via the Linux `getrandom(2)` syscall.
//!
//! Provides random [`Digit`], [`Ternary`], and [`Tryte`] values without
//! depending on `std` or the `rand` crate, using the kernel directly.
//!
//! # Platform support
//!
//! Linux on `x86_64` or `aarch64`.  Other targets will not compile this
//! module (the `getrandom` feature gate prevents accidental inclusion).
//!
//! # Example
//!
//! ```
//! use balanced_ternary::getrandom::{rand_digit, rand_digits};
//!
//! let d = rand_digit();
//! assert!(matches!(d, balanced_ternary::Digit::Neg | balanced_ternary::Digit::Zero | balanced_ternary::Digit::Pos));
//!
//! let digits = rand_digits(8);
//! assert_eq!(digits.len(), 8);
//! ```

use crate::Digit;

#[cfg(feature = "ternary-string")]
use crate::Ternary;

#[cfg(feature = "tryte")]
use crate::Tryte;

#[cfg(feature = "ternary-store")]
use crate::{BTer9, BTer27, UTer9, UTer27};

// ---------------------------------------------------------------------------
// Raw syscall — Linux only
// ---------------------------------------------------------------------------

/// Fills `buf` with random bytes from the kernel's entropy pool.
///
/// Calls `getrandom(buf, len, 0)` directly, retrying on `EINTR` (errno −4).
/// Panics if the syscall returns any other error (should not happen for
/// heap-backed buffers with `flags = 0`).
pub fn getrandom_bytes(buf: &mut [u8]) {
    if buf.is_empty() {
        return;
    }
    let mut filled = 0usize;
    while filled < buf.len() {
        let ret = unsafe { sys_getrandom(buf.as_mut_ptr().add(filled), buf.len() - filled, 0) };
        if ret < 0 {
            assert_eq!(ret, -4, "getrandom syscall failed with errno {}", -ret);
            // EINTR: retry
        } else {
            filled += ret as usize;
        }
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn sys_getrandom(buf: *mut u8, len: usize, flags: u32) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        in("rax") 318usize,         // SYS_getrandom
        in("rdi") buf,
        in("rsi") len,
        in("rdx") flags as usize,
        // syscall clobbers rcx and r11
        out("rcx") _,
        out("r11") _,
        lateout("rax") ret,
        options(nostack),
    );
    ret
}

#[cfg(target_arch = "aarch64")]
unsafe fn sys_getrandom(buf: *mut u8, len: usize, flags: u32) -> isize {
    let ret: isize;
    core::arch::asm!(
        "svc #0",
        in("x8")  278usize,         // SYS_getrandom
        in("x0")  buf,
        in("x1")  len,
        in("x2")  flags as usize,
        lateout("x0") ret,
        options(nostack),
    );
    ret
}

// ---------------------------------------------------------------------------
// Random Digit
// ---------------------------------------------------------------------------

/// Returns a uniformly distributed random [`Digit`] (`−1`, `0`, or `+1`).
///
/// Uses rejection sampling on one random byte to avoid modulo bias:
/// values 0..252 split evenly into three groups of 84; values 252..255
/// are discarded (probability 3/256 ≈ 1.2 %).
pub fn rand_digit() -> Digit {
    loop {
        let mut buf = [0u8; 1];
        getrandom_bytes(&mut buf);
        let b = buf[0];
        if b < 252 {
            return match b % 3 {
                0 => Digit::Neg,
                1 => Digit::Zero,
                _ => Digit::Pos,
            };
        }
        // reject and retry (probability < 1.2%)
    }
}

/// Returns `n` uniformly distributed random [`Digit`]s.
pub fn rand_digits(n: usize) -> alloc::vec::Vec<Digit> {
    // Fill a raw byte buffer then map each byte, retrying on bias tail.
    let mut digits = alloc::vec::Vec::with_capacity(n);
    let mut buf = alloc::vec![0u8; n + (n / 64).max(4)]; // over-allocate slightly
    loop {
        getrandom_bytes(&mut buf);
        for &b in &buf {
            if b < 252 {
                digits.push(match b % 3 {
                    0 => Digit::Neg,
                    1 => Digit::Zero,
                    _ => Digit::Pos,
                });
                if digits.len() == n {
                    return digits;
                }
            }
        }
        // Need more bytes; refill and continue
    }
}

// ---------------------------------------------------------------------------
// Random Ternary (requires "ternary-string")
// ---------------------------------------------------------------------------

/// Returns a random [`Ternary`] of exactly `len` digits.
///
/// Each trit is uniformly distributed over `{−1, 0, +1}`.
#[cfg(feature = "ternary-string")]
pub fn rand_ternary(len: usize) -> Ternary {
    Ternary::new(rand_digits(len))
}

// ---------------------------------------------------------------------------
// Random Tryte (requires "tryte")
// ---------------------------------------------------------------------------

/// Returns a random [`Tryte<N>`].
///
/// Each trit is uniformly distributed over `{−1, 0, +1}`.
#[cfg(feature = "tryte")]
pub fn rand_tryte<const N: usize>() -> Tryte<N> {
    let mut digits = [Digit::Zero; N];
    for d in digits.iter_mut() {
        *d = rand_digit();
    }
    Tryte::new(digits)
}

// ---------------------------------------------------------------------------
// Random fixed-point types (requires "ternary-store")
// ---------------------------------------------------------------------------

/// Returns a random [`BTer9`] (uniform over −9841..+9841).
#[cfg(feature = "ternary-store")]
pub fn rand_bter9() -> BTer9 {
    // Sample a uniform i32 in the BTer9 value range [−9841, +9841] = [−(3^9−1)/2, +(3^9−1)/2].
    // 3^9 = 19683 values; use 2 random bytes (65536 values), reject above 65535 − 65535%19683.
    const RANGE: u32 = 19683; // 3^9
    const CUTOFF: u32 = u16::MAX as u32 + 1 - (u16::MAX as u32 + 1) % RANGE; // 65535 − 65535%19683
    loop {
        let mut buf = [0u8; 2];
        getrandom_bytes(&mut buf);
        let v = u16::from_le_bytes(buf) as u32;
        if v < CUTOFF {
            let n = (v % RANGE) as i32 - 9841;
            return BTer9::from_dec(n);
        }
    }
}

/// Returns a random [`UTer9`] (uniform over 0..19682).
#[cfg(feature = "ternary-store")]
pub fn rand_uter9() -> UTer9 {
    const RANGE: u32 = 19683;
    const CUTOFF: u32 = u16::MAX as u32 + 1 - (u16::MAX as u32 + 1) % RANGE;
    loop {
        let mut buf = [0u8; 2];
        getrandom_bytes(&mut buf);
        let v = u16::from_le_bytes(buf) as u32;
        if v < CUTOFF {
            return UTer9::from_dec(v % RANGE);
        }
    }
}

/// Returns a random [`BTer27`] (uniform over −3762798742493..+3762798742493).
#[cfg(feature = "ternary-store")]
pub fn rand_bter27() -> BTer27 {
    // 3^27 = 7625597484987 values; use 6 random bytes (2^48 values).
    // Reject above 2^48 − 2^48 % 3^27 to eliminate bias.
    const RANGE: u64 = 7_625_597_484_987; // 3^27
    const SPACE: u64 = 1u64 << 48;        // 2^48
    const CUTOFF: u64 = SPACE - SPACE % RANGE;
    loop {
        let mut buf = [0u8; 8];
        getrandom_bytes(&mut buf);
        // Use lower 6 bytes (48 bits)
        let v = u64::from_le_bytes(buf) & 0x0000_FFFF_FFFF_FFFF;
        if v < CUTOFF {
            let n = (v % RANGE) as i64 - 3_812_798_742_493;
            return BTer27::from_dec(n);
        }
    }
}

/// Returns a random [`UTer27`] (uniform over 0..7625597484986).
#[cfg(feature = "ternary-store")]
pub fn rand_uter27() -> UTer27 {
    const RANGE: u64 = 7_625_597_484_987; // 3^27
    const SPACE: u64 = 1u64 << 48;
    const CUTOFF: u64 = SPACE - SPACE % RANGE;
    loop {
        let mut buf = [0u8; 8];
        getrandom_bytes(&mut buf);
        let v = u64::from_le_bytes(buf) & 0x0000_FFFF_FFFF_FFFF;
        if v < CUTOFF {
            return UTer27::from_dec(v % RANGE);
        }
    }
}
