//! Exact integer transforms on the Boolean lattice.
//!
//! Index `mask` denotes a subset. Inputs must have a nonzero power-of-two
//! length; length one represents the empty ground set. Arithmetic panics on
//! overflow rather than silently wrapping, including in release builds.

pub mod convolution;
pub mod metrics;
pub mod naive;
pub mod svg;
pub mod trace;
pub mod transform;

pub(crate) fn dimension(values: &[i64]) -> usize {
    assert!(
        values.len().is_power_of_two(),
        "length must be a nonzero power of two"
    );
    values.len().trailing_zeros() as usize
}

/// Reproducible SplitMix64 samples in -10..=10 (not a cryptographic RNG).
pub fn random_values(n: u32, seed: u64) -> Vec<i64> {
    let len = 1usize.checked_shl(n).expect("dimension exceeds mask width");
    let mut state = seed;
    (0..len)
        .map(|_| {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            ((z ^ (z >> 31)) % 21) as i64 - 10
        })
        .collect()
}
