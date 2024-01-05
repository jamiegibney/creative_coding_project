//! Value smoothers.

/// Non-atomic linear segment generation. Internal system for `Smoother`.
mod ramp;
/// Atomic linear segment generation. Internal system for `SmootherAtomic`.
mod ramp_atomic;

/// Smoothable traits and type implementations.
pub mod smoothable_types;
/// Non-atomic value smoothing.
pub mod smoother;
/// Atomic value smoothing.
pub mod smoother_atomic;
pub use smoothable_types::{Smoothable, SmoothableAtomic};
pub use smoother::Smoother;
pub use smoother_atomic::SmootherAtomic;
