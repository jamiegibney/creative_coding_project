use super::Phasor;
use super::*;

/// Basic non-antialiased triangle wave oscillator.
///
/// [Formula source](https://www.desmos.com/calculator/dzdtwqrnto)
#[derive(Debug, Clone, Copy)]
pub struct TriOsc {
    phasor: Phasor,
}

impl TriOsc {
    pub fn new(freq_hz: f64, sample_rate: f64) -> Self {
        Self {
            phasor: Phasor::new(freq_hz, sample_rate),
        }
    }
}

impl GeneratorProcessor for TriOsc {
    fn process(&mut self) -> (f64, f64) {
        self.phasor.increment_phase();
        let p = self.phasor.period_length_samples();
        let x = self.phasor.next();

        let xp4_p = (x + (p / 4.0)) / p;

        let floor = (xp4_p + 0.5).floor();
        let inner: f64 = 2.0 * (xp4_p - floor);

        let out = 2.0f64.mul_add(inner.abs(), -1.0);

        (out, out)
    }

    fn set_freq(&mut self, freq_hz: f64, sample_rate: f64) {
        self.phasor.set_freq(freq_hz, sample_rate);
    }
}
