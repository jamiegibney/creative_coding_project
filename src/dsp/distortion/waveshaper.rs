use std::ops::RangeInclusive;

use super::*;

/// A waveshaper which dynamically accepts any transfer function and asymmetric
/// drive levels.
pub struct Waveshaper {
    drive: f64,
    drive_lower: f64,

    drive_range: RangeInclusive<f64>,

    asymmetric: bool,

    xfer_function: Box<dyn Fn(f64, f64) -> f64 + Send>,
}

impl Waveshaper {
    /// Returns a new, initialised waveshaper.
    ///
    /// By default, this uses `xfer::s_curve` as its transfer function, its
    /// drive parameter range is `0.0` to `1.0`, and it operates symmetrically.
    ///
    /// See the `set_xfer_function()` method to provide a custom transfer 
    /// function, and `set_drive_range()` to change its drive parameter range.
    #[must_use]
    pub fn new() -> Self {
        Self {
            drive: 0.0,
            drive_lower: 0.0,

            drive_range: 0.0..=1.0,

            asymmetric: false,

            xfer_function: Box::new(xfer::s_curve),
        }
    }

    /// Processes a single sample through the waveshaper. The output is not
    /// clipped.
    #[must_use]
    pub fn process(&self, sample: f64) -> f64 {
        let xfer = &self.xfer_function;

        if self.asymmetric && sample.is_sign_negative() {
            return xfer(sample, self.drive_lower);
        }

        xfer(sample, self.drive)
    }

    /// Moves `function` into the waveshaper, which will then use it as its
    /// transfer function. The passed function must have two arguments of type
    /// `f64`, and return `f64`, to be accepted. The first argument refers to
    /// the function's input, the second its "modification" amount (such as
    /// curve tension).
    ///
    /// If the transfer function you want to use only has one argument, use the
    /// `set_xfer_function_single_argument()` method.
    ///
    /// # Notes
    ///
    /// The transfer function should follow these rules:
    ///
    /// - Its input should operate in the range `-1.0` to `1.0`,
    /// - Its second argument should accept the range `0.0` to `1.0`,
    /// - It *should prefer* to output values between `-1.0` and `1.0`.
    ///
    /// The waveshaper's drive parameter operates in the range of `0.0` to `1.0`
    /// by default. If you need to map the waveshaper's drive to the range your
    /// transfer function accepts, see the `set_drive_range()` method. An example
    /// of this may be if the transfer function has an "inverse" part in a different
    /// part of its range (e.g. if `0.0` to `1.0` is its "normal" range, and
    /// `0.0` to `1.0` is its "inverse" range).
    pub fn set_xfer_function<F>(&mut self, function: F)
    where
        F: Fn(f64, f64) -> f64 + Send + 'static,
    {
        self.xfer_function = Box::new(function);
    }

    /// If the transfer function you want to pass only has a single argument
    /// (such as the sine function, for example), use this function to pass it
    /// to the waveshaper.
    pub fn set_xfer_function_single_argument<F>(&mut self, function: F)
    where
        F: Fn(f64) -> f64 + Send + 'static,
    {
        let xfer = move |x: f64, _: f64| -> f64 { function(x) };
        self.xfer_function = Box::new(xfer);
    }

    /// Sets the drive of the waveshaper. If asymmetric distortion is enabled,
    /// this is only used for positive parts of the signal.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `drive` is outside the range of `0.0` to `1.0`.
    pub fn set_drive(&mut self, drive: f64) {
        debug_assert!(self.drive_range.contains(&drive));
        self.drive =
            drive.clamp(*self.drive_range.start(), *self.drive_range.end());
    }

    /// Sets the drive of the waveshaper for negative parts of the signal; only
    /// used if asymmetric distortion is enabled.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `drive` is outside the range of `0.0` to `1.0`.
    pub fn set_drive_lower(&mut self, drive: f64) {
        debug_assert!(self.drive_range.contains(&drive));
        self.drive_lower =
            drive.clamp(*self.drive_range.start(), *self.drive_range.end());
    }

    /// Sets the range for the waveshaper's `drive` parameters to use. The parameters
    /// are clamped to this range, and if a value which exceeds this range is passed
    /// to either the `set_drive()` or `set_drive_lower()` methods, they will panic
    /// in debug mode.
    ///
    /// The intended purpose of this function is to mutate the range which you will
    /// normally pass to the waveshaper, for whatever reason you need.
    ///
    /// # Example
    ///
    /// ```
    /// let mut ws = Waveshaper::new();
    ///
    /// // by default, this will panic (in debug mode)
    /// // ws.set_drive(-1.0);
    ///
    /// ws.set_drive_range(-1.0..=1.0);
    ///
    /// ws.set_drive(-1.0);
    /// ws.set_drive_lower(-0.1234);
    /// ```
    pub fn set_drive_range(&mut self, range: RangeInclusive<f64>) {
        self.drive_range = range;
    }

    /// Sets whether the waveshaper separately applies drive to the positive and
    /// negative parts of the waveform.
    pub fn set_asymmetric(&mut self, asymmetric: bool) {
        self.asymmetric = asymmetric;
    }
}

impl Default for Waveshaper {
    fn default() -> Self {
        Self::new()
    }
}
