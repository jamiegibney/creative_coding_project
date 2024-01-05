//! Spectrogram types and logic.

pub mod analyzer;
pub mod process;

pub use analyzer::SpectrumAnalyzer;
pub use process::{SpectrumInput, SpectrumOutput, SPECTRUM_OVERLAP_FACTOR, SPECTRUM_WINDOW_SIZE};
