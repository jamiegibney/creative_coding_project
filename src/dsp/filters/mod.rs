pub mod allpass;
pub mod biquad;
pub mod comb;

/// A trait which allows for filters to be dynamically dispatched.
pub trait Filter {
    fn process(&mut self, sample: f64) -> f64;
}

// TODO SIMD optimisations, vroom
// Add more common methods to this trait.
