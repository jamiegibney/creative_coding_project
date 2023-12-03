use super::*;

/// Basic non-antialiased square wave oscillator.
#[derive(Debug, Clone, Copy)]
pub struct SquareOsc {
    phase_l: f64,
    phase_r: f64,
    phase_increment: f64,
}

impl SquareOsc {
    pub fn new(freq_hz: f64, sample_rate: f64) -> Self {
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);

        Self {
            phase_l: 0.0,
            phase_r: 0.0,
            phase_increment: freq_hz / sample_rate,
        }
    }

    fn increment_phase(&mut self) {
        self.phase_l += self.phase_increment;
        self.phase_r += self.phase_increment;

        if self.phase_l >= 1.0 {
            self.phase_l -= 1.0;
        }
        if self.phase_r >= 1.0 {
            self.phase_r -= 1.0;
        }
    }
}

impl GeneratorProcessor for SquareOsc {
    fn process(&mut self) -> (f64, f64) {
        self.increment_phase();

        let out_l = if self.phase_l.is_sign_positive() {
            1.0
        } else {
            -1.0
        };
        let out_r = if self.phase_r.is_sign_positive() {
            1.0
        } else {
            -1.0
        };

        (out_l, out_r)
    }

    fn set_freq(&mut self, freq_hz: f64, sample_rate: f64) {}
}
