//! One-pole lowpass filter.

use crate::dsp::Effect;
use crate::prelude::*;

/// Source: https://www.musicdsp.org/en/latest/Effects/169-compressor.html
#[derive(Clone)]
pub struct OnePoleLowpass {
    a0: f64,
    b1: f64,

    old: f64,
}

impl OnePoleLowpass {
    /// Returns a new `OnePoleLowpass` filter with identity coefficients (i.e., the input
    /// is unaltered).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the cutoff frequency of the filter in Hz.
    ///
    /// # See also
    ///
    /// [`set_cutoff_time()`](Self::set_cutoff_time)
    /// [`set_cutoff_time_samples()`](Self::set_cutoff_time_samples)
    pub fn set_cutoff_freq(&mut self, freq_hz: f64) {
        let sr = unsafe { SAMPLE_RATE };
        assert!(freq_hz.is_sign_positive() && freq_hz <= sr / 2.0);

        let c = 2.0 - (TAU * freq_hz / sr).cos();

        self.b1 = (c * c - 1.0).sqrt() - c;
        self.a0 = 1.0 + self.b1;
    }

    /// Sets the cutoff frequency based on a time value in samples. Useful for averaged
    /// level measurement similar to RMS.
    ///
    /// `time_samples` is the time window in samples (`window * sample_rate`), and
    /// `speed` controls the rate of change. `9.0` is a common value for `speed`.
    ///
    /// Based on *Audio Processes by David Creasey*.
    ///
    /// # See also
    ///
    /// [`set_cutoff_time()`](Self::set_cutoff_time)
    /// [`set_cutoff_freq()`](Self::set_cutoff_freq)
    pub fn set_cutoff_time_samples(&mut self, time_samples: f64, speed: f64) {
        let g = speed.powf(-(time_samples.recip()));

        self.a0 = 1.0 - g;
        self.b1 = g;
    }

    /// Sets the cutoff frequency based on a time value in milliseconds. Useful for
    /// averaged level measurement similar to RMS.
    ///
    /// `time_ms` is the time window in milliseconds, and `speed` controls the rate
    /// of change. `9.0` is a common value for `speed`.
    ///
    /// # See also
    ///
    /// [`set_cutoff_time_samples()`](Self::set_cutoff_time_samples)
    /// [`set_cutoff_freq()`](Self::set_cutoff_freq)
    pub fn set_cutoff_time(&mut self, time_ms: f64, speed: f64) {
        let samples = unsafe { SAMPLE_RATE } * time_ms * 0.001;
        self.set_cutoff_time_samples(samples, speed);
    }
}

impl Effect for OnePoleLowpass {
    fn process_mono(&mut self, input: f64) -> f64 {
        self.old = self.a0 * input - self.b1 * self.old;
        self.old
    }
}

impl Default for OnePoleLowpass {
    fn default() -> Self {
        Self { a0: 1.0, b1: 0.0, old: 0.0 }
    }
}
