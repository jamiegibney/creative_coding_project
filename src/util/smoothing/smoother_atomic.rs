use super::*;
use super::ramp_atomic::RampAtomic;

pub struct SmootherAtomic<T: SmoothableAtomic> {
    ramp: RampAtomic<T>,
    start_value: T,
    target_value: T,
}
