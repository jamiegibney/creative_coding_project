//! Module for signal generation.
use super::*;

pub mod generator;
pub mod basic;

pub use basic::*;

pub use generator::Generator;
pub use phasor::PhasorOsc;
pub use sine::SineOsc;
pub use noise_osc::NoiseOsc;

pub trait GeneratorProcessor {
    fn process(&mut self) -> (f64, f64);
}
