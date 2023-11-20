//! Generic enum for audio generators.
use super::*;

#[derive(Debug, Clone, Copy)]
pub enum Generator {
    Saw(PhasorOsc),
    Sine(SineOsc),
    Noise,
}

impl Generator {
    pub fn process(&mut self) -> (f64, f64) {
        match self {
            Self::Saw(gen) => gen.process(),
            Self::Sine(gen) => gen.process(),
            Self::Noise => NoiseOsc::process_stereo(),
        }
    }
}
