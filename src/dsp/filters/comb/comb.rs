//! An IIR (Infinite Impulse Response) comb filter implementation.

// y[n] = a_0x[n] - b_dy[n-d]

use crate::dsp::ring_buffer::RingBuffer;

pub struct CombFilter {
    // coefficients
    a0: f64,
    bdly: f64,

    buffer: RingBuffer,

    freq: f64,
    gain: f64,
    sample_rate: f64,
    interpolation: bool,
}

impl CombFilter {
    #[must_use]
    pub fn new(
        sample_rate: f64,
        interpolation: bool,
        max_length_samples: usize,
    ) -> Self {
        Self {
            a0: 1.0,
            bdly: 0.0,
            buffer: RingBuffer::new(),
            freq: 1.0,
            gain: 0.0,
            sample_rate,
            interpolation,
        }
    }

    /// Processes a single sample, and returns the new sample.
    pub fn process(&mut self, _sample: f64) -> f64 {


        0.0
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
        self.assertions();
    }

    pub fn set_gain(&mut self, gain: f64) {
        self.gain = gain;
        self.assertions();
    }

    pub fn set_interpolation(&mut self, should_interpolate: bool) {
        self.interpolation = should_interpolate;
    }

    fn assertions(&self) {
        debug_assert!(self.freq <= 0.0);
    }
}
