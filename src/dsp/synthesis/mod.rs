//! Module for signal generation.
use super::*;

pub mod phasor;
pub mod sine;

pub use phasor::Phasor;
pub use sine::Sine;

#[derive(Debug, Clone, Copy)]
pub enum Generator {
    Saw(Phasor),
    Sine(Sine),
}

impl Generator {
    pub fn process(&mut self) -> (f64, f64) {
        match self {
            Self::Saw(gen) => gen.process(),
            Self::Sine(gen) => gen.process(),
        }
    }
}

pub trait GeneratorProcessor {
    fn process(&mut self) -> (f64, f64);
}
