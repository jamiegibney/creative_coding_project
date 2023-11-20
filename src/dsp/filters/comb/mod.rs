//! FIR (finite impulse response) and IIR (infinite impulse response) comb filter forms.

#![allow(clippy::must_use_candidate)]
mod filter;
mod fir;
mod iir;
pub use fir::FirCombFilter;
pub use iir::IirCombFilter;
pub use crate::dsp::*;




#[cfg(test)]
mod tests {
    // use super::*;
}
