#![allow(clippy::should_implement_trait)]
use crate::prelude::*;

/// A ramp (i.e. linear segment) generator. Useful for smoothing values over time,
/// or as the internal system of an envelope.
#[derive(Debug, Clone)]
pub struct Ramp {
    current_value: f64,
    last_value: f64,
    target_value: f64,
    // the size of each step per sample
    step_size: f64,
    // the internal interpolation amount
    interp: f64,
    smoothing_type: SmoothingType,
}

impl Ramp {
    #[must_use]
    pub fn new(target_value: f64, ramp_duration_secs: f64) -> Self {
        let mut s = Self {
            current_value: 0.0,
            last_value: 0.0,
            target_value,
            step_size: 0.0,
            interp: 0.0,
            smoothing_type: SmoothingType::default(),
        };

        s.reset(target_value, ramp_duration_secs);

        s
    }

    /// Resets the target value and duration of the ramp generator, retaining
    /// its last value (i.e. it will interpolate from its current value to the
    /// new target).
    ///
    /// If you change the sample rate, this is the function to call to update
    /// the ramp's timing.
    pub fn reset(&mut self, target_value: f64, ramp_duration_secs: f64) {
        self.last_value = self.current_value;
        self.target_value = target_value;

        if ramp_duration_secs <= 0.0 {
            self.interp = 1.0;
            self.step_size = 0.0;
        }
        else {
            self.interp = 0.0;
            self.step_size =
                1.0 / (ramp_duration_secs * unsafe { SAMPLE_RATE });
        }
    }

    /// The same as reset, but sets the value of the ramp before resetting it,
    /// meaning that it will progress from that value.
    pub fn reset_with_value(
        &mut self,
        start_value: f64,
        target_value: f64,
        ramp_duration_secs: f64,
    ) {
        self.current_value = start_value;
        self.reset(target_value, ramp_duration_secs);
    }

    pub fn set_smoothing_type(&mut self, smoothing_type: SmoothingType) {
        self.smoothing_type = smoothing_type;
    }

    /// Clears the ramp, resetting it completely. Call the `reset()` method to
    /// re-initialize the ramp after calling this.
    pub fn clear(&mut self) {
        self.current_value = 0.0;
        self.last_value = 0.0;
        self.target_value = 0.0;
        self.step_size = 0.0;
        self.interp = 0.0;
    }

    /// Progresses the ramp by one step, returning its new value.
    ///
    /// This is intended to be called at the sample rate.
    pub fn next(&mut self) -> f64 {
        self.interp += self.step_size;

        // is the ramp finished?
        if self.interp > 1.0 {
            self.interp = 1.0;
            self.step_size = 0.0;
            self.current_value = self.target_value;
            self.last_value = self.current_value;
        }
        else {
            self.calculate_current_value();
        }

        self.current_value
    }

    /// Skips a certain number of steps, returning the value at the end of the skip.
    ///
    /// Equivalent to calling `next()` `steps` times, unless "steps" would cause
    /// the ramp to reach its target, in which case the ramp is automatically reset
    /// and returns its target value.
    pub fn skip(&mut self, steps: usize) -> f64 {
        if self.step_size.mul_add(steps as f64, self.interp) > 1.0 {
            self.interp = 1.0;
            self.step_size = 0.0;
            self.calculate_current_value();
        }
        else {
            for _ in 0..steps {
                self.next();
            }
        }

        self.current_value
    }

    /// Returns the current value of the ramp.
    #[must_use]
    pub fn current_value(&self) -> f64 {
        self.current_value
    }

    /// Returns whether the ramp is active, i.e. still interpolating.
    #[must_use]
    pub fn is_active(&self) -> bool {
        !within_tolerance(self.interp, 1.0, f64::EPSILON)
    }

    fn calculate_current_value(&mut self) -> f64 {
        let (a, b, t) = (self.last_value, self.target_value, self.interp);

        self.current_value = match self.smoothing_type {
            SmoothingType::Linear => interp::lerp(a, b, t),
            SmoothingType::Cosine => interp::cosine(a, b, t),
            SmoothingType::SineTop => interp::lerp(a, b, xfer::sine_upper(t)),
            SmoothingType::SineBottom => {
                interp::lerp(a, b, xfer::sine_lower(t))
            }
            SmoothingType::CurveNormal(tension) => {
                interp::lerp(a, b, xfer::s_curve(t, tension))
            }
            SmoothingType::CurveLinearStart(tension) => {
                interp::lerp(a, b, xfer::s_curve_linear_centre(t, tension))
            }
            SmoothingType::CurveRounder(tension) => {
                interp::lerp(a, b, xfer::s_curve_round(t, tension))
            }
        };

        self.current_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finished() {
        let target = 1462.0;
        let mut ramp = Ramp::new(target, 1.0);

        let steps = unsafe { SAMPLE_RATE } as usize;

        for _ in 0..steps {
            ramp.next();
        }

        assert!(within_tolerance(ramp.current_value(), target, f64::EPSILON));
        assert!(!ramp.is_active());
    }

    #[test]
    #[should_panic]
    fn not_finished() {
        let target = 324.4326;
        let mut ramp = Ramp::new(target, 1.0);

        let steps = unsafe { SAMPLE_RATE } as usize;

        for _ in 0..steps - 1 {
            ramp.next();
        }

        assert!(within_tolerance(ramp.current_value(), target, f64::EPSILON));
    }

    #[test]
    fn skip_test() {
        let target = 10.0;
        let mut ramp = Ramp::new(target, 1.0);

        let steps = unsafe { SAMPLE_RATE } as usize;

        for _ in 0..steps / 2 {
            ramp.next();
        }
    }
}
