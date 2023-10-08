#![allow(clippy::must_use_candidate)]
mod comb;

pub use comb::*;

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

#[cfg(test)]
mod tests {
    // use super::*;
}
