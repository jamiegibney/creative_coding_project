// NOTE: the atomic versions were abandoned.
/// Non-atomic linear segment generation. Internal system for `Smoother`.
mod ramp;

/// Smoothable traits and type implementations.
pub mod smoothable_types;
/// Non-atomic value smoothing.
pub mod smoother;
/// Atomic value smoothing.
pub use smoothable_types::{Smoothable, SmoothableAtomic};
pub use smoother::Smoother;
