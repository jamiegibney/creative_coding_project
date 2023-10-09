use dyn_clone::DynClone;
use std::fmt::Debug;
pub const BUTTERWORTH_Q: f64 = std::f64::consts::FRAC_1_SQRT_2;

pub mod allpass;
pub mod biquad;
pub mod comb;
pub mod first_order;

/// A trait which allows for filters to be dynamically dispatched.
pub trait Filter: Send + DynClone {
    /// Generic processing method for a filter.
    fn process(&mut self, sample: f64) -> f64;
}

dyn_clone::clone_trait_object!(Filter);

/// An enum which covers the available filter types.
///
/// Currently, peak, lowpass, highpass, bandpass, notch, and allpass biquad
/// filters are implemented.
#[derive(Debug, Clone, Copy, Default)]
pub enum FilterType {
    #[default]
    Peak,
    Lowpass,
    Highpass,
    Lowshelf,
    Highshelf,
    Bandpass,
    Notch,
    Allpass,
}

// TODO SIMD optimisations, vroom
// Add more common methods to this trait.
