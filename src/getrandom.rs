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

// ---------------------------------------------------------------------------
// SplitMix64 PRNG — one-shot getrandom seed, then O(1) per call
// ---------------------------------------------------------------------------

/// Fast pseudo-random number generator seeded from one `getrandom` syscall.
///
/// Uses the **SplitMix64** algorithm (Blackman & Vigna 2018), which passes
/// BigCrush / PractRand.  Period is 2^64.  **Not** cryptographically secure —
/// use the standalone `rand_bter27()` / `rand_uter27()` functions if you need
/// unpredictable output.
///
/// # Example
///
/// ```
/// use balanced_ternary::getrandom::SplitMix64;
///
/// let mut rng = SplitMix64::new();
/// let _ = rng.next_u64();
/// ```
pub struct SplitMix64(u64);

impl SplitMix64 {
    /// Seed from the kernel's entropy pool — **one** `getrandom` syscall.
    /// Subsequent calls to `next_u64` / `rand_*` make no syscalls.
    pub fn new() -> Self {
        let mut buf = [0u8; 8];
        getrandom_bytes(&mut buf);
        let seed = u64::from_le_bytes(buf);
        Self(if seed == 0 { 0x9e3779b97f4a7c15 } else { seed })
    }

    /// Create a deterministic PRNG from a fixed seed — **no** syscall, no entropy.
    pub const fn from_seed(seed: u64) -> Self {
        Self(if seed == 0 { 1 } else { seed })
    }

    /// Draw the next 64 pseudo-random bits (~1 ns, no syscall).
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    /// Draw the next 32 pseudo-random bits.
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    /// Random [`BTer9`] uniform over −9841..+9841.  No syscall after construction.
    ///
    /// Rejection rate < 0.0005 % (2^32 mod 3^9 / 2^32); in practice always one iteration.
    #[cfg(feature = "ternary-store")]
    pub fn rand_bter9(&mut self) -> BTer9 {
        const RANGE: u32 = 19683; // 3^9
        const CUTOFF: u32 = u32::MAX - u32::MAX % RANGE;
        loop {
            let v = self.next_u32();
            if v < CUTOFF {
                return BTer9::from_dec((v % RANGE) as i32 - 9841);
            }
        }
    }

    /// Random [`UTer9`] uniform over 0..19682.  No syscall after construction.
    #[cfg(feature = "ternary-store")]
    pub fn rand_uter9(&mut self) -> UTer9 {
        const RANGE: u32 = 19683;
        const CUTOFF: u32 = u32::MAX - u32::MAX % RANGE;
        loop {
            let v = self.next_u32();
            if v < CUTOFF {
                return UTer9::from_dec(v % RANGE);
            }
        }
    }

    /// Random [`BTer27`] uniform over ±3762798742493.  No syscall after construction.
    ///
    /// Uses full u64 space → rejection rate < 0.00005 %; essentially always one iteration.
    #[cfg(feature = "ternary-store")]
    pub fn rand_bter27(&mut self) -> BTer27 {
        const RANGE: u64 = 7_625_597_484_987; // 3^27
        const CUTOFF: u64 = u64::MAX - u64::MAX % RANGE;
        loop {
            let v = self.next_u64();
            if v < CUTOFF {
                return BTer27::from_dec((v % RANGE) as i64 - 3_812_798_742_493);
            }
        }
    }

    /// Random [`UTer27`] uniform over 0..7625597484986.  No syscall after construction.
    #[cfg(feature = "ternary-store")]
    pub fn rand_uter27(&mut self) -> UTer27 {
        const RANGE: u64 = 7_625_597_484_987;
        const CUTOFF: u64 = u64::MAX - u64::MAX % RANGE;
        loop {
            let v = self.next_u64();
            if v < CUTOFF {
                return UTer27::from_dec(v % RANGE);
            }
        }
    }
}
