use super::*;
use std::f64::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct Sine {
    phase_l: f64,
    phase_r: f64,
    phase_increment: f64,
}

impl Sine {
    pub fn new(freq_hz: f64) -> Self {
        let sample_rate = unsafe { SAMPLE_RATE };
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);

        let phase_increment = freq_hz / sample_rate * TAU;

        Self { phase_l: 0.0, phase_r: 0.1, phase_increment }
    }
}

impl GeneratorProcessor for Sine {
    fn process(&mut self) -> (f64, f64) {
        let out_l = self.phase_l.sin();
        let out_r = self.phase_r.sin();
        self.phase_l += self.phase_increment;
        self.phase_r += self.phase_increment;

        if self.phase_l >= TAU {
            self.phase_l -= TAU;
        }

        if self.phase_r >= TAU {
            self.phase_r -= TAU;
        }

        (out_l, out_r)
    }
}
