use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Phasor {
    phase_l: f64,
    phase_r: f64,
    phase_increment: f64,
}

impl Phasor {
    pub fn new(freq_hz: f64) -> Self {
        let sample_rate = unsafe { SAMPLE_RATE };
        debug_assert!(0.0 < freq_hz && freq_hz <= sample_rate / 2.0);

        let phase_increment = freq_hz / sample_rate;

        Self { phase_l: 0.0, phase_r: 0.1, phase_increment }
    }
}

impl GeneratorProcessor for Phasor {
    fn process(&mut self) -> (f64, f64) {
        let out_l = self.phase_l.mul_add(2.0, -1.0);
        let out_r = self.phase_r.mul_add(2.0, -1.0);
        self.phase_l += self.phase_increment;
        self.phase_r += self.phase_increment;

        if self.phase_l >= 1.0 {
            self.phase_l -= 1.0;
        }

        if self.phase_r >= 1.0 {
            self.phase_r -= 1.0;
        }

        (out_l, out_r)
    }
}
