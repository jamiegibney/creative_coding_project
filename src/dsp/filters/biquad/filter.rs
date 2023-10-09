#![allow(clippy::module_name_repetitions)]
use super::*;
use crate::{prelude::SAMPLE_RATE, util};
use std::f64::consts::{FRAC_1_SQRT_2, PI, TAU};
use util::db_to_level;
use FilterType as FT;

// TODO see if the wikipedia biquad implementation works any better?
// https://en.wikipedia.org/wiki/Digital_biquad_filter#Transposed_direct_form_2

#[derive(Debug, Clone, Copy)]
struct Coefs {
    a0: f64,
    a1: f64,
    a2: f64,
    b1: f64,
    b2: f64,
}

impl Coefs {
    fn identity() -> Self {
        // This state can be used to leave the input signal totally unaffected.
        Self { a0: 1.0, a1: 0.0, a2: 0.0, b1: 0.0, b2: 0.0 }
    }
}

impl Default for Coefs {
    fn default() -> Self {
        Self::identity()
    }
}

/// A struct which covers the parameters used by biquad filters.
#[derive(Debug, Clone, Copy)]
pub struct BiquadParams {
    pub freq: f64,
    pub gain: f64,
    pub q: f64,
    pub filter_type: FilterType,
}

impl Default for BiquadParams {
    fn default() -> Self {
        Self {
            freq: 1.0,
            q: FRAC_1_SQRT_2,
            filter_type: FT::default(),
            gain: 0.0,
        }
    }
}

/// A biquadratic filter implementation, which offers all of the filter types
/// available in `FilterType`.
///
/// Its parameters are stored internally as a `BiquadParams`, which can be passed
/// to the `set_params()` method to mutate the filter's state. There are also
/// separate methods for each of the filter's parameters for finer control.
/// Neither of these options differ in performance.
///
/// # Performance
///
/// The filter will lazily update its coefficients, i.e. only when a parameter
/// is updated (even if the value is the same) will it recompute its coefficients.
/// Consider avoiding unnecessary calls to the `set_q()`, `set_type()`,
/// `set_gain()`, `set_freq()`, `reset_sample_rate()`, or `set_params()`
/// methods where possible to leverage the lazy behaviour.
///
/// Note that certain filter types do not use all parameters which can be
/// passed to the filter. These values are ignored during processing, but
/// updating them will still signal the filter to recompute.
#[derive(Debug, Clone, Default)]
pub struct BiquadFilter {
    coefs: Coefs,
    delayed: (f64, f64),
    needs_recompute: bool,

    params: BiquadParams,
    sample_rate: f64,
}

impl Filter for BiquadFilter {
    /// Processes a single sample of the filter and returns the new sample.
    ///
    /// Note that this filter will lazily update its coefficients; if there is
    /// no parameter change between calls to this method, only the sample output
    /// is computed — not the filter coefficients. In other words, this method
    /// will compute much faster if there is no parameter change between calls.
    fn process(&mut self, sample: f64) -> f64 {
        let Coefs { a0, a1, a2, b1, b2 } = self.coefs;
        let (z1, z2) = self.delayed;

        if self.needs_recompute {
            match self.params.filter_type {
                FT::Peak => self.set_peak_coefs(),
                FT::Lowpass => self.set_lowpass_coefs(),
                FT::Highpass => self.set_highpass_coefs(),
                FT::Lowshelf | FT::Highshelf => {
                    dbg!(
                        self.params.filter_type,
                        "shelf filters not yet implemented for biquads",
                    );
                    self.suspend();
                }
                FT::Bandpass => self.set_bandpass_coefs(),
                FT::Notch => self.set_notch_coefs(),
                FT::Allpass => self.set_allpass_coefs(),
            };

            self.needs_recompute = false;
        }

        let output = a0.mul_add(sample, z1);

        self.delayed = (
            a1.mul_add(sample, output * -b1) + z2,
            a2.mul_add(sample, output * -b2),
        );

        output
    }
}

impl BiquadFilter {
    /// Creates a new, initialised `Filter`, set to the default `Peak` filter type.
    #[must_use]
    pub fn new(sample_rate: f64) -> Self {
        Self { sample_rate, ..Self::default() }
    }

    /// "Suspends" the filter, leaving any processed signal totally unaltered.
    ///
    /// See `force_recompute()` if you need to "resume" the filter's processing
    /// after a call to this method.
    ///
    /// Alternatively, adjusting any of the filter parameters via the `set_q()`,
    /// `set_type()`, `set_gain()`, `set_freq()`, `reset_sample_rate()`, or
    /// `set_params()` methods will also "resume" the filter's processing.
    ///
    /// Note that this function does not alter its filter parameters.
    ///
    /// Note that this function acts instantaneously, and does not attempt to
    /// prevent clicking or signal discontinuities.
    pub fn suspend(&mut self) {
        self.coefs = Coefs::identity();
        self.needs_recompute = false;
    }

    /// Resets the sample rate of the filter.
    ///
    /// # Safety
    ///
    /// This can be called whilst the filter is actively processing and its
    /// coefficients will update upon the next call to the `process()` method,
    /// but the audio output is not guaranteed to be safe.
    pub fn reset_sample_rate(&mut self, new_sample_rate: f64) {
        self.sample_rate = new_sample_rate;
        self.needs_recompute = true;
    }

    /// Forces the filter to recompute its coefficients on the next call of the
    /// `process()` method.
    ///
    /// This can be used to "resume" the filter's processing after a call to
    /// `suspend()`.
    pub fn force_recompute(&mut self) {
        self.needs_recompute = true;
    }

    /// Sets the parameters of the filter all at once.
    ///
    /// # Panics
    ///
    /// This function will panic in debug mode if the parameter's filter
    /// and/or q value is negative.
    /// It will also panic in debug mode if the frequency is over half of
    /// the sample rate.
    pub fn set_params(&mut self, params: &BiquadParams) {
        self.params = *params;
        self.needs_recompute = true;
        self.debug_assertions();
    }

    /// Sets the frequency of the filter.
    ///
    /// # Panics
    ///
    /// This function will panic in debug mode if `freq` is negative.
    /// It will also panic in debug mode if the frequency is over half of
    /// the sample rate.
    pub fn set_freq(&mut self, freq: f64) {
        self.params.freq = freq;
        self.needs_recompute = true;
        self.debug_assertions();
    }

    /// Sets the gain of the filter.
    pub fn set_gain(&mut self, gain: f64) {
        self.params.gain = gain;
        self.needs_recompute = true;
        self.debug_assertions();
    }

    /// Sets the Q of the filter.
    ///
    /// # Panics
    /// This function will panic in debug mode if `Q` is negative.
    pub fn set_q(&mut self, q: f64) {
        self.params.q = q;
        self.needs_recompute = true;
        self.debug_assertions();
    }

    /// Sets the filter type of the filter.
    ///
    /// # Note
    ///
    /// Note that the shelving filters are not yet implemented.
    pub fn set_type(&mut self, filter_type: FilterType) {
        self.params.filter_type = filter_type;
        self.needs_recompute = true;
        self.debug_assertions();
    }

    /// Returns the half-power points (-3 dB gain) of the bandpass/notch filter.
    ///
    /// # Panics
    /// This function will panic if the current filter type is not `Bandpass` or `Notch`.
    #[must_use]
    pub fn bp_notch_half_power_points(&self) -> (f64, f64) {
        debug_assert!(matches!(
            self.params.filter_type,
            FT::Notch | FT::Bandpass
        ));
        let BiquadParams { freq, q, .. } = self.params;
        let f_min =
            (freq / (2.0 * q)) * (4.0f64.mul_add(q.powi(2), 1.0).sqrt() - 1.0);
        let f_max = f_min + (freq / q);

        (f_min, f_max)
    }

    /// Returns the bandwidth of the bandpass/notch filter.
    ///
    /// # Panics
    /// This function will panic if the current filter type is not `Bandpass` or `Notch`.
    #[must_use]
    pub fn bp_notch_bandwidth(&self) -> f64 {
        debug_assert!(matches!(
            self.params.filter_type,
            FT::Notch | FT::Bandpass
        ));

        self.params.freq / self.params.q
    }

    /* PRIVATE METHODS */

    /// Sets the filter coefficients for a peak filter.
    fn set_peak_coefs(&mut self) {
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;
        let BiquadParams { freq, gain, q, filter_type } = self.params;
        let sr = &self.sample_rate;

        let phi = (TAU * freq) / sr;
        let g0 = (util::db_to_level(gain) - 1.0) / 2.0;
        let g1 = 2.0 / (1.0 + g0);
        let phi_2q_tan = (phi / (2.0 * q)).tan();

        *b2 = g1.mul_add(-phi_2q_tan, 1.0) / g1.mul_add(phi_2q_tan, 1.0);
        *b1 = -(1.0 + *b2) * phi.cos();
        *a0 = g0.mul_add(1.0 - *b2, 1.0);
        *a1 = *b1;
        *a2 = 1.0 + (*b2 - *a0);
    }

    /// Sets the filter coefficients for a lowpass filter.
    fn set_lowpass_coefs(&mut self) {
        self.lowpass_highpass_b_coefs();
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;

        *a0 = 0.25 * (1.0 + *b1 + *b2);
        *a1 = 2.0 * *a0;
        *a2 = *a0;
    }

    /// Sets the filter coefficients for a highpass filter.
    fn set_highpass_coefs(&mut self) {
        self.lowpass_highpass_b_coefs();
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;

        *a0 = 0.25 * (1.0 - *b1 + *b2);
        *a1 = -2.0 * *a0;
        *a2 = *a0;
    }

    /// Sets common coefficients for lowpass and highpass designs.
    fn lowpass_highpass_b_coefs(&mut self) {
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;
        let BiquadParams { freq, q, filter_type, .. } = self.params;
        let sr = &self.sample_rate;

        let phi = (TAU * freq) / sr;
        let sin_phi = phi.sin();
        let q_2 = 2.0 * q;

        *b2 = (q_2 - sin_phi) / (q_2 + sin_phi);
        *b1 = -(1.0 + *b2) * phi.cos();
    }

    /// Sets the filter coefficients for a lowshelf filter.
    fn set_lowshelf_coefs(&mut self) {
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;
        let BiquadParams { freq, gain, .. } = self.params;
        let sr = self.sample_rate;

        let gain = db_to_level(gain);
        let k2 = (PI * freq / sr).tan().powi(2);
        let sqrt_2_g = (2.0 * gain).sqrt();
        let sqrt_2_gk = sqrt_2_g * k2;

        // TODO this is a total mess, can it be refactored for clarity?
        if gain.is_sign_positive() {
            // boost
            let norm = 1.0 / (1.0 + sqrt_2_g + k2);
            *a0 = gain.mul_add(k2, 1.0 + sqrt_2_gk) * norm;
            *a1 = 2.0 * gain.mul_add(k2, -1.0) * norm;
            *a2 = gain.mul_add(k2, 1.0 - sqrt_2_gk) * norm;
            *b1 = 2.0 * (k2 - 1.0) * norm;
            *b2 = (1.0 - sqrt_2_g + k2) * norm;
        }
        else {
            // cut
            let norm = 1.0 / gain.mul_add(k2, 1.0 + sqrt_2_gk);
            *a0 = (1.0 + sqrt_2_g + k2) * norm;
            *a1 = 2.0 * (k2 - 1.0) * norm;
            *a2 = (1.0 - sqrt_2_g + k2) * norm;
            *b1 = 2.0 * gain.mul_add(k2, -1.0) * norm;
            *b2 = gain.mul_add(k2, 1.0 - sqrt_2_gk) * norm;
        }
    }

    /// Sets the filter coefficients for a highshelf filter.
    fn set_highshelf_coefs(&mut self) {
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;
        let BiquadParams { freq, q, gain, .. } = self.params;
        let sr = &self.sample_rate;

        let gain = db_to_level(gain);
        let k2 = (PI * freq / sr).tan().powi(2);
        let sqrt_2_g = (2.0 * gain).sqrt();
        let sqrt_2_gk = sqrt_2_g * k2;

        // TODO this is also a total mess, can it be refactored for clarity?
        if gain.is_sign_positive() {
            // boost
            let norm = 1.0 / (1.0 + sqrt_2_g + k2);
            *a0 = (gain + sqrt_2_gk + k2) * norm;
            *a1 = 2.0 * (k2 - gain) * norm;
            *a2 = (gain - sqrt_2_gk + k2) * norm;
            *b1 = 2.0 * (k2 - 1.0) * norm;
            *b2 = (1.0 - sqrt_2_g + k2) * norm;
        }
        else {
            // cut
            let norm = 1.0 / (gain + sqrt_2_gk + k2);
            *a0 = (1.0 + sqrt_2_g + k2) * norm;
            *a1 = 2.0 * (k2 - 1.0) * norm;
            *a2 = (1.0 - sqrt_2_g + k2) * norm;
            *b1 = 2.0 * (k2 - gain) * norm;
            *b2 = (gain - sqrt_2_gk + k2) * norm;
        }
    }

    /// Sets the filter coefficients for a bandpass filter.
    fn set_bandpass_coefs(&mut self) {
        self.bandpass_notch_b_coefs();
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;

        *a0 = (1.0 - *b2) / 2.0;
        *a1 = 0.0;
        *a2 = -(*a0);
    }

    /// Sets the filter coefficients for a notch filter.
    fn set_notch_coefs(&mut self) {
        self.bandpass_notch_b_coefs();
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;

        *a0 = (1.0 + *b2) / 2.0;
        *a1 = *b1;
        *a2 = *a0;
    }

    /// Sets common coefficients for notch and bandpass designs.
    fn bandpass_notch_b_coefs(&mut self) {
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;
        let BiquadParams { freq, q, .. } = self.params;
        let sr = &self.sample_rate;

        let phi = (TAU * freq) / sr;

        *b2 = (PI / 4.0) - (phi / (2.0 * q)).tan();
        *b1 = -(1.0 + *b2) * phi.cos();
    }

    /// Sets the filter coefficients for a allpass filter.
    fn set_allpass_coefs(&mut self) {
        let Coefs { a0, a1, a2, b1, b2 } = &mut self.coefs;
        let BiquadParams { freq, q, .. } = self.params;
        let sr = &self.sample_rate;

        let phi = (TAU * freq) / sr;

        *b2 = (phi / 2.0).mul_add(-q, PI / 4.0);
        *b1 = -(1.0 + *b2) * phi.cos();
        *a0 = *b2;
        *a1 = *b1;
        *a2 = 1.0;
    }

    /// Debug assertions used whenever a parameter is changed.
    fn debug_assertions(&self) {
        let BiquadParams { freq, q, filter_type, .. } = self.params;
        let sr = self.sample_rate;

        // general assertions
        debug_assert!(
            freq.is_sign_positive() && q.is_sign_positive() && freq <= sr / 2.0
        );

        // type-specific assertions
        match self.params.filter_type {
            FT::Lowpass | FT::Highpass => debug_assert!(q >= 0.5f64.sqrt()),
            FT::Allpass | FT::Peak | FT::Bandpass | FT::Notch => {
                // TODO do some tests with this, as it seems like it may not be necessary
                debug_assert!(freq < (q * sr) / 4.0);
            }
            _ => (),
        }
    }
}
