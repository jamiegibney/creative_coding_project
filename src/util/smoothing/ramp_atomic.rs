use super::smoothable_types::SmoothableAtomic;
use crate::prelude::*;
use atomic_float::AtomicF64;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

/// A value smoother (AKA ramp), which computes an interpolated value for each sample.
///
/// This version performs atomic operations, in case you need to share its values
/// among threads.
#[derive(Debug)]
pub struct RampAtomic<T: SmoothableAtomic> {
    /// The type of smoothing to use.
    smoothing_type: SmoothingType,

    /// The number of smoothing steps remaining until the target is reached.
    steps_remaining: AtomicI32,

    /// The step increment for each step, which should be called each sample.
    step_size: AtomicF64,

    /// The smoothed value for the current sample.
    current_value: AtomicF64,

    /// The target value.
    target: T::Atomic,

    /// The duration of smoothing in milliseconds.
    duration_ms: f64,
}

impl<T: SmoothableAtomic> RampAtomic<T> {
    /// Returns a new `Smoother` with the provided smoothing type.
    pub fn new(smoothing_type: SmoothingType) -> Self {
        Self { smoothing_type, ..Default::default() }
    }

    /// Returns the next smoothed value. Intended to be called once per sample — if you need
    /// to skip a certain number of steps, see the [`skip()`][Self::skip()] method.
    pub fn next(&self) -> T {
        self.skip(1)
    }

    /// Returns the next smoothed value. Intended to be called once per sample — if you need
    /// to skip a certain number of steps, see the [`skip()`][Self::skip()] method.
    //
    // TODO: is this any faster than the above in terms of skip(1)?
    fn next_alt(&self) -> T {
        let target = T::atomic_load(&self.target);

        if self.steps_remaining.load(Relaxed) > 0 {
            let current_value = self.current_value.load(Relaxed);
            let target_float = target.to_f64();
            let increment = self.step_size.load(Relaxed);

            let steps = self.steps_remaining.fetch_sub(1, Relaxed);
            let new = if steps == 1 {
                self.steps_remaining.store(0, Relaxed);
                target_float
            }
            else {
                current_value + increment
            };

            T::from_f64(new)
        }
        else {
            target
        }
    }

    /// Returns the smoothed value after `num_steps` iterations. Equivalent to calling the
    /// [`next()`][Self::next()] method `num_steps` times, with slight optimisation.
    pub fn skip(&self, num_steps: u32) -> T {
        debug_assert_ne!(num_steps, 0);

        let target = T::atomic_load(&self.target);

        if self.steps_remaining.load(Relaxed) > 0 {
            let current_value = self.current_value.load(Relaxed);
            let target_float = target.to_f64();
            let increment = self.step_size.load(Relaxed);

            let steps =
                self.steps_remaining.fetch_sub(num_steps as i32, Relaxed);
            let new = if steps <= num_steps as i32 {
                self.steps_remaining.store(0, Relaxed);
                target_float
            }
            else {
                current_value + (increment * num_steps as f64)
            };

            self.current_value.store(new, Relaxed);

            T::from_f64(new)
        }
        else {
            target
        }
    }

    /// Returns the current value held by the smoother, which is the last value returned
    /// by the [`next()`][Self::next()] method.
    pub fn current_value(&self) -> T {
        T::from_f64(self.current_value.load(Relaxed))
    }

    pub fn next_block(&self, block: &mut [T], block_len: usize) {
        self.next_block_exact(&mut block[..block_len]);
    }

    pub fn next_block_exact(&self, block: &mut [T]) {
        let target = T::atomic_load(&self.target);

        let steps_remaining = self.steps_remaining.load(Relaxed) as usize;
        let num_smoothed_values = block.len().min(steps_remaining);

        if num_smoothed_values > 0 {
            let mut current_value = self.current_value.load(Relaxed);
            let target_float = target.to_f64();
            let increment = self.step_size.load(Relaxed);

            if num_smoothed_values == steps_remaining {
                block[..num_smoothed_values - 1].fill_with(|| {
                    current_value += increment;
                    T::from_f64(current_value)
                });

                current_value = target_float.to_f64();
                block[num_smoothed_values - 1] = target;
            }
            else {
                block[..num_smoothed_values].fill_with(|| {
                    current_value += increment;
                    T::from_f64(current_value)
                });
            }

            block[num_smoothed_values..].fill(target);

            self.current_value.store(current_value, Relaxed);
            self.steps_remaining
                .fetch_sub(num_smoothed_values as i32, Relaxed);
        }
        else {
            block.fill(target);
        }
    }

    /// Resets the smoother at the provided value.
    pub fn reset(&self, value: T) {
        T::atomic_store(&self.target, value);
        self.current_value.store(value.to_f64(), Relaxed);
        self.steps_remaining.store(0, Relaxed);
    }

    /// Sets the target value to smooth towards.
    pub fn set_target(&self, target: T) {
        T::atomic_store(&self.target, target);

        let steps_remaining = self.duration_samples();
        self.steps_remaining.store(steps_remaining as i32, Relaxed);

        let current = self.current_value.load(Relaxed);
        let target_float = target.to_f64();
        self.step_size.store(
            if steps_remaining > 0 { self.compute_step_size() } else { 0.0 },
            Relaxed,
        );
    }

    /// The number of steps remaining until the target value is reached.
    pub fn steps_remaining(&self) -> u32 {
        self.steps_remaining.load(Relaxed) as u32
    }

    /// Returns whether the smoother is active or not, i.e. whether the `next()` method
    /// will yield new values when called.
    pub fn is_active(&self) -> bool {
        self.steps_remaining() > 0
    }

    /// Computes the total number of steps required to reach the target value.
    fn duration_samples(&self) -> u32 {
        (unsafe { SAMPLE_RATE } * self.duration_ms / 1000.0).round() as u32
    }

    /// Computes the size of each step.
    fn compute_step_size(&self) -> f64 {
        (T::atomic_load(&self.target).to_f64()
            - self.current_value.load(Relaxed))
            / (self.steps_remaining.load(Relaxed) as f64)
    }
}

impl<T: SmoothableAtomic> Default for RampAtomic<T> {
    fn default() -> Self {
        Self {
            smoothing_type: SmoothingType::default(),
            steps_remaining: AtomicI32::new(0),
            step_size: AtomicF64::default(),
            current_value: AtomicF64::new(0.0),
            target: Default::default(),
            duration_ms: 0.0,
        }
    }
}

impl<T: SmoothableAtomic> Clone for RampAtomic<T> {
    fn clone(&self) -> Self {
        Self {
            smoothing_type: self.smoothing_type.clone(),
            steps_remaining: AtomicI32::new(self.steps_remaining.load(Relaxed)),
            step_size: AtomicF64::new(self.step_size.load(Relaxed)),
            current_value: AtomicF64::new(self.current_value.load(Relaxed)),
            target: T::atomic_new(T::atomic_load(&self.target)),
            duration_ms: self.duration_ms.clone(),
        }
    }
}
