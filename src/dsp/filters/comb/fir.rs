use super::filter::CombFilter;
use crate::prelude::*;

/// A FIR (Finite Impulse Response) comb filter.
#[derive(Debug, Clone)]
pub struct FirCombFilter {
    filter: CombFilter,
}

impl FirCombFilter {
    /// Creates a new, initialized filter with an internal buffer holding
    /// one second of samples.
    pub fn with_interpolation(interpolation: bool) -> Self {
        Self { filter: CombFilter::new(interpolation) }
    }

    /// Processes a single sample of the comb filter, returning the new sample.
    pub fn process(&mut self, sample: f64) -> f64 {
        self.filter.buffer.push(sample);

        sample * self.filter.a0 + self.filter.buffer.read() * self.filter.bd
    }

    /// Use this if you change the sample rate to reallocate the internal buffer.
    pub fn reset_sample_rate(&mut self) {
        self.filter.reset_sample_rate();
    }

    /// Sets the frequency of the comb filter.
    ///
    /// # Panics
    ///
    /// Panics if `freq` is less than 1 or greater than half of the sample rate.
    pub fn set_freq(&mut self, freq: f64) {
        self.filter.set_freq(freq);
    }

    /// Sets the gain of the comb filter.
    ///
    /// # Panics
    ///
    /// Panics if the gain is greater than 0 dB.
    pub fn set_gain_db(&mut self, gain_db: f64) {
        self.filter.set_gain_db(gain_db);

        let level = db_to_level(gain_db);
        let polarity = if self.filter.positive_polarity { 1.0 } else { -1.0 };
        self.filter.bd = (1.0 - level) / 2.0 * polarity;
        // self.filter.a0 = 1.0 - self.filter.bd.abs();
    }

    /// Sets the polarity of the comb filter.
    pub fn set_positive_polarity(&mut self, polarity_should_be_positive: bool) {
        self.filter
            .set_positive_polarity(polarity_should_be_positive);
    }

    /// Sets whether the comb filter should interpolate between samples.
    pub fn set_interpolation(&mut self, interpolation_type: InterpType) {
        self.filter.set_interpolation(interpolation_type);
    }
}

