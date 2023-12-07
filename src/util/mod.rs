//! Global utility functions — these are publicly re-exported in `prelude.rs`.

use crate::settings::{SAMPLE_RATE, TUNING_FREQ_HZ};
use nannou::prelude::{DVec2, Vec2};
use std::f64::consts::PI;
use std::sync::atomic::Ordering::Relaxed;

pub mod interp;
pub mod smoothing;
pub mod thread_pool;
pub mod window;
pub mod xfer;
pub mod atomic_load;

pub use interp::InterpolationType as InterpType;

pub use interp::{ilerp, lerp};
pub use smoothing::*;
pub use thread_pool::ThreadPool;
pub use xfer::SmoothingType;
pub use atomic_load::AtomicLoad;

/// Calculates the modulo-1 value of a floating-point value.
pub fn mod1(x: f64) -> f64 {
    x - x.floor()
}

/// Calculates the frequency value of the provided MIDI note value.
pub fn note_to_freq(note_value: f64) -> f64 {
    ((note_value - 69.0) / 12.0).exp2() * unsafe { TUNING_FREQ_HZ }
}

/// Calculates the MIDI note value of the provided frequency value.
pub fn freq_to_note(freq: f64) -> f64 {
    12.0f64.mul_add((freq / unsafe { TUNING_FREQ_HZ }).log2(), 69.0)
}

/// Calculates amplitude in decibels from a linear power level.
pub fn level_to_db(level: f64) -> f64 {
    20.0 * level.log10()
}

/// Calculates the linear power level from amplitude as decibels.
pub fn db_to_level(db_value: f64) -> f64 {
    10.0f64.powf(db_value / 20.0)
}

/// Maps a value from the provided input range to the provided output range.
pub fn map(value: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
    scale(normalise(value, in_min, in_max), out_min, out_max)
}

/// Scales a value to a provided range, assuming it is normalised.
///
/// Like `map()`, but with no input range.
pub fn scale(value: f64, min: f64, max: f64) -> f64 {
    value.mul_add(max - min, min)
}

/// Normalises a value from a provided range.
///
/// Like `map()`, but with the output range set to `0.0 - 1.0`.
pub fn normalise(value: f64, min: f64, max: f64) -> f64 {
    (value - min) / (max - min)
}

/// Returns a vector of interleaved elements from the input, i.e.
/// `0.x, 0.y, 1.x, 1.y, ...`
pub fn interleave_dvec2_to_f64(input: &[DVec2]) -> Vec<f64> {
    let mut v = Vec::with_capacity(input.len() * 2);

    for &pos in input {
        v.push(pos.x);
        v.push(pos.y);
    }

    v
}

/// Returns a vector of interleaved elements from the input, i.e.
/// `0.x, 0.y, 1.x, 1.y, ...`
pub fn interleave_vec2_to_f32(input: &[Vec2]) -> Vec<f32> {
    let mut v = Vec::with_capacity(input.len() * 2);

    for &pos in input {
        v.push(pos.x);
        v.push(pos.y);
    }

    v
}

/// Returns whether the absolute value of `value` is less than the provided
/// `tolerance` value. Useful for checking approximate equality.
pub fn within_tolerance(value: f64, target: f64, tolerance: f64) -> bool {
    (value - target).abs() <= tolerance
}

/// Returns the length of one sample in seconds, based on the current sample rate.
pub fn sample_length() -> f64 {
    unsafe { SAMPLE_RATE }.recip()
}

/// The unnormalised sinc function (`sin(x) / x`).
///
/// For a normalised sinc function, multiply `x` by `π`.
#[rustfmt::skip]
pub fn sinc(x: f64) -> f64 {
    if x == 0.0 { 1.0 }
    else { x.sin() / x }
}

/// Returns true if `value` is equal to `target`, with a tolerance of
/// ±`f64::EPSILON`.
pub fn epsilon_eq(value: f64, target: f64) -> bool {
    (target - value).abs() < f64::EPSILON
}

/// Returns a vector containing points of a Lanczos kernel. `a_factor` is the "a"
/// variable in the kernel calculation. Only holds enough points to represent each lobe.
/// Returns `4 * a_factor + 1` elements (when `trim_zeroes == false`).
///
/// `scale` will automatically scale each element in the kernel, and `trim_zeroes` will
/// remove the first and last elements (which are always `0.0`) if true.
///
/// [Source](https://en.wikipedia.org/wiki/Lanczos_resampling)
///
/// # Panics
///
/// Panics if `a_factor == 0`.
#[rustfmt::skip]
pub fn lanczos_kernel(a_factor: u8, scale: f64, trim_zeroes: bool) -> Vec<f64> {
    assert_ne!(a_factor, 0);

    let a = a_factor as f64;
    let num_stages = a_factor * 4 + 1;

    (if trim_zeroes { 1..num_stages - 1 } else { 0..num_stages })
        .map(|i| {
            if i % 2 == 0 { 0.0 }
            else {
                let x = 2.0f64.mul_add(-a, i as f64) / 2.0;

                if x == 0.0 { 1.0 }
                else if -a < x && x < a {
                    sinc(PI * x) * sinc((PI * x) / a) * scale
                }
                else { 0.0 }
            }
        })
        .collect()
}

/// Returns a normalised value representing the logarithmic value of a frequency
/// based on the current sample rate.
///
/// In other words, this function accepts a linear frequency value, scales it
/// logarithmically such that octaves are evenly spaced, and then normalises
/// it between `start_hz` Hz and the Nyquist rate such that the output range
/// is `0.0` to `1.0`.
///
/// # Panics
///
/// Panics if `start_hz == 0`.
///
/// Panics in debug mode if either `freq_hz`, `start_hz`, or `sample_rate` is negative.
///
/// # Source
///
/// [Found by experimenting on Desmos.](https://www.desmos.com/calculator/nqgorlqxyw)
pub fn freq_log_norm(freq_hz: f64, start_hz: f64, sample_rate: f64) -> f64 {
    assert!(!epsilon_eq(start_hz, 0.0));
    debug_assert!(
        freq_hz.is_sign_positive() && start_hz.is_sign_positive() && sample_rate.is_sign_positive()
    );
    let log_start = start_hz.log10();
    let norm = ((sample_rate / 2.0).log10() - log_start).recip();

    norm * (freq_hz.log10() - log_start)
}

/// The inverse of [`freq_log_norm()`](freq_log_norm).
///
/// The expectation of this function is that `freq_hz_log_norm` is a normalised value
/// between `0.0` and `1.0`, and that it will transpose a logarithmically-scaled frequency
/// value between `0.0` and `1.0` back to its original frequeny value between `0.0` and the
/// Nyquist frequency.
///
/// # Panics
///
/// Panics if `start_hz == 0`.
///
/// Panics in debug mode if either `freq_hz`, `start_hz`, or `sample_rate` is negative.
///
/// # Source
///
/// [Found by experimenting on Desmos.](https://www.desmos.com/calculator/nqgorlqxyw)
pub fn freq_lin_from_log(freq_hz_log_norm: f64, start_hz: f64, sample_rate: f64) -> f64 {
    assert!(!epsilon_eq(start_hz, 0.0));
    debug_assert!(
        freq_hz_log_norm.is_sign_positive()
            && start_hz.is_sign_positive()
            && sample_rate.is_sign_positive()
    );

    let log_start = start_hz.log10();
    // find the normalisation factor
    let norm = ((sample_rate / 2.0).log10() - log_start).recip();
    // denormalise and shift the original
    let log = (freq_hz_log_norm / norm) + log_start;

    // "de-log" the original
    10.0_f64.powf(log)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_freq_conversion() {
        let e6 = 88.0;
        let freq = note_to_freq(e6);
        assert!(within_tolerance(freq, 1318.51, 0.001));
        assert!(within_tolerance(freq_to_note(freq), e6, f64::EPSILON));
    }

    #[test]
    fn test_amplitude_conversion() {
        let level = 0.5;
        let db = level_to_db(level);
        assert!(within_tolerance(db, -6.020_599_913_279_624, f64::EPSILON));
        assert!(within_tolerance(db_to_level(db), level, f64::EPSILON));
    }
}
