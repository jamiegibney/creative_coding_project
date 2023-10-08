pub mod smoothed_float;
pub mod xfer;
pub mod interp;
use crate::settings::TUNING_FREQ_HZ;

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

/// Returns whether the absolute value of `value` is less than the provided
/// `tolerance` value. Useful for checking approximate equality.
pub fn within_tolerance(value: f64, tolerance: f64) -> bool {
    value.abs() <= tolerance
}
