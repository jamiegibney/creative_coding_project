use super::*;
use std::f64::consts::TAU;

/// Basic non-antialiased sine wave oscillator.
#[derive(Debug, Clone, Copy)]
pub struct SineOsc {
    phase_l: f64,
    phase_r: f64,
    phase_increment: f64,
}

impl SineOsc {
    pub fn new(freq_hz: f64, sample_rate: f64) -> Self {
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);
        let phase_increment = freq_hz / sample_rate * TAU;

        Self { phase_l: 0.0, phase_r: 0.1, phase_increment }
    }

    fn increment_phase(&mut self) {
        self.phase_l += self.phase_increment;
        self.phase_r += self.phase_increment;

        if self.phase_l >= TAU {
            self.phase_l -= TAU;
        }
        if self.phase_r >= TAU {
            self.phase_r -= TAU;
        }
    }
}

impl GeneratorProcessor for SineOsc {
    fn process(&mut self) -> (f64, f64) {
        let out_l = self.phase_l.sin();
        let out_r = self.phase_r.sin();

        self.increment_phase();

        (out_l, out_r)
    }

    fn set_freq(&mut self, freq_hz: f64, sample_rate: f64) {
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);
        self.phase_increment = freq_hz / sample_rate * TAU;
    }
}
