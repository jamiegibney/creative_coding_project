use crate::util::interp;
use crate::util::xfer;

#[derive(Copy, Clone)]
pub enum SmoothingType {
    /// Linear mapping from `a -> b`
    Linear,
    /// Cosine function mapping from `a -> b`
    Cosine,
    /// Quarter-sine function mapping from `a -> b`, biased towards b
    SineTop,
    /// Quarter-sine function mapping from `a -> b`, biased towards a
    SineBottom,
    /// Standard curve mapping from `a -> b` with tension argument
    CurveNormal(f64),
    /// Curved mapping from `a -> b` with tension argument and a linear start
    CurveLinearStart(f64),
    /// Rounder curve mapping from `a -> b` with tension argument
    CurveRounder(f64),
}

/// A struct for values to be smoothed over time.
///
/// Supports linear and cosine interpolation from point to point.
///
/// Call `instance.next()` per sample to increment the value, or use
/// `instance.skip(...)` to increment by multiple samples.
pub struct SmoothedValue {
    target_value: f64,
    current_value: f64,
    start_value: f64,
    progress: f64,
    step_size: f64,
    sample_rate: f64,
    smoothing_type: SmoothingType,
}

impl SmoothedValue {
    /// Returns a new, initialised `SmoothedValue` instance.
    ///
    /// The instance must be mutable. Use `instance.prepare(...)` before using the instance.
    pub fn new() -> Self {
        Self {
            target_value: 0.0,
            current_value: 0.0,
            start_value: 0.0,
            progress: 1.0,
            step_size: 0.0,
            sample_rate: 0.0,
            smoothing_type: SmoothingType::Linear,
        }
    }

    /// Prepares an instance for use, also calculating the step size to use.
    ///
    /// Linear smoothing is used by default. Use `instance.set_smoothing_type(...)`
    /// for other smoothing functions.
    ///
    /// # Examples
    /// ```
    /// instance.prepare(44100.0, 0.0, 0.5, pa::interp::InterpolationType::Cosine);
    /// instance.prepare(96000.0, 100.0, 0.02, pa::interp::InterpolationType::Linear);
    /// ```
    pub fn prepare(
        &mut self,
        sample_rate: f64,
        init_value: f64,
        smoothing_time: f64,
    ) {
        self.sample_rate = if sample_rate < 0.0 { 0.0 } else { sample_rate };
        self.reset_to(init_value);
        self.set_smoothing_time(smoothing_time);
    }

    /// Sets the smoothing time of the instance in seconds.
    ///
    /// This will recalculate the step size, but the progress
    /// is maintained.
    pub fn set_smoothing_time(&mut self, smoothing_time_seconds: f64) {
        let smoothing_time_seconds = if smoothing_time_seconds < 0.0 {
            0.0
        }
        else {
            smoothing_time_seconds
        };

        self.set_step_size(smoothing_time_seconds);
    }

    /// Sets the type of smoothing function used.
    pub fn set_smoothing_type(&mut self, smoothing_type: SmoothingType) {
        self.smoothing_type = smoothing_type;
    }

    /// Sets a new target value.
    ///
    /// This resets the instance's progress.
    pub fn set_target_value(&mut self, value: f64) {
        self.target_value = value;
        self.start_value = self.current_value;
        self.progress = 0.0;
    }

    /// Sets a new range for the instance to use.
    ///
    /// `progress` is clamped between `0.0` and `1.0`.
    ///
    /// If you want to retain the instance's progress, pass `instance.progress`:
    /// ```
    /// instance.set_range(0.0, 1.0, instance.progress);
    /// ```
    /// If you want to jump to a specific value, consider using `pa::interp::i_lerp(...)`:
    /// ```
    /// instance.set_range(a, b, pa::interp::i_lerp(a, b, value));
    /// ```
    pub fn set_range(
        &mut self,
        start_value: f64,
        target_value: f64,
        progress: f64,
    ) {
        self.start_value = start_value;
        self.target_value = target_value;
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// Returns the next value from the instance, incrementing
    /// the step.
    pub fn next(&mut self) -> f64 {
        self.increment();
        self.calculate_value()
    }

    /// Skips a specified number of steps, returning the latest value.
    ///
    /// If the target value is reached during a skip, it is returned.
    pub fn skip(&mut self, num_steps: usize) -> f64 {
        for _ in 0..num_steps {
            if self.progress >= 1.0 {
                return self.target_value;
            }
            self.increment();
        }

        self.calculate_value()
    }

    /// Stops smoothing in-place, returning the current value.
    pub fn stop(&mut self) -> f64 {
        self.target_value = self.current_value;
        self.progress = 1.0;

        self.current_value
    }

    /// Resets the instance to a certain value and stops smoothing.
    pub fn reset_to(&mut self, reset_value: f64) {
        self.current_value = reset_value;
        self.start_value = reset_value;
        self.stop();
    }

    /// # Internal method
    /// Calculates the step size based on the sample rate and smoothing time.
    fn set_step_size(&mut self, smoothing_time: f64) {
        let num_steps = (smoothing_time * self.sample_rate).ceil();

        self.step_size = 1.0 / num_steps;
    }

    /// # Internal method
    /// Increments the progress by the step size.
    fn increment(&mut self) {
        if self.progress >= 1.0 {
            return;
        }
        self.progress = (self.progress + self.step_size).clamp(0.0, 1.0);
    }

    /// # Internal method
    /// Calculates the new value based on the progress and interpolation type.
    fn calculate_value(&mut self) -> f64 {
        let (a, b, t) = (self.start_value, self.target_value, self.progress);

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

impl Default for SmoothedValue {
    fn default() -> Self {
        Self::new()
    }
}
