//! An FIR (Finite Impulse Response) comb filter implementation.
use crate::util::interp::InterpolationType as InterpType;

// y[n] = a_0x[n] - b_dy[n-d]

use super::*;

#[derive(Debug, Clone)]
pub struct CombFilter {
    // coefficients
    pub a0: f64,
    pub bd: f64,

    pub buffer: RingBuffer,

    pub freq: f64,
    pub gain_db: f64,
    pub positive_polarity: bool,
    pub interpolation: bool,
}

impl CombFilter {
    #[must_use]
    pub fn new(interpolation: bool) -> Self {
        Self {
            a0: 1.0,
            bd: 0.0,

            // allocates 1 second
            buffer: RingBuffer::new(unsafe { SAMPLE_RATE } as usize),

            freq: unsafe { TUNING_FREQ_HZ },
            gain_db: MINUS_INFINITY_DB,
            positive_polarity: true,
            interpolation,
        }
    }

    pub fn reset_sample_rate(&mut self) {
        self.buffer.prepare_with_time(1.0, 0.03);
    }

    pub fn set_freq(&mut self, freq: f64) {
        self.freq = freq;
        self.assertions();
        self.set_delay_time();
    }

    pub fn set_gain_db(&mut self, gain: f64) {
        self.gain_db = gain;
        self.assertions();
    }

    pub fn set_positive_polarity(&mut self, polarity_should_be_positive: bool) {
        self.positive_polarity = polarity_should_be_positive;
    }

    pub fn set_interpolation(&mut self, interpolation_type: InterpType) {
        self.buffer.set_interpolation_type(interpolation_type);
    }

    fn set_delay_time(&mut self) {
        self.buffer.set_delay_time(1.0 / self.freq);
    }

    fn assertions(&self) {
        debug_assert!(
            1.0 <= self.freq
                && self.freq <= unsafe { SAMPLE_RATE } / 2.0
                && self.gain_db <= 0.0
        );
    }
}
