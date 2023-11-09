//! Global utility functions — these are publicly re-exported in `prelude.rs`.

use crate::settings::{SAMPLE_RATE, TUNING_FREQ_HZ};
use nannou::prelude::{DVec2, Vec2};

pub mod interp;
pub mod smoothing;
pub mod thread_pool;
pub mod window;
pub mod xfer;

pub use interp::InterpolationType as InterpType;
pub use interp::{ilerp, lerp};
pub use smoothing::*;
pub use thread_pool::ThreadPool;
pub use xfer::SmoothingType;

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
pub fn map(
    value: f64,
    in_min: f64,
    in_max: f64,
    out_min: f64,
    out_max: f64,
) -> f64 {
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
pub fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    }
    else {
        x.sin() / x
    }
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
