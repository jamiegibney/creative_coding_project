//! Module for an ADSR envelope generator.
#![allow(unused)]
use crate::prelude::*;
use AdsrStage as AS;

const DEFAULT_ATTACK_TIME_MS: f64 = 10.0;
const DEFAULT_DECAY_TIME_MS: f64 = 100.0;
const DEFAULT_SUSTAIN_LEVEL: f64 = 0.5;
const DEFAULT_RELEASE_TIME_MS: f64 = 50.0;

const DEFAULT_CURVE_AMOUNT: f64 = 0.7;

/// An enum representing the possible stages of an ADSR envelope.
#[derive(Debug, Clone, Copy, Default)]
pub enum AdsrStage {
    #[default]
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// An envelope generator with attack, decay, sustain, and release (ADSR) stages.
#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    attack_time_ms: f64,
    attack_level: f64,
    attack_curve: f64,

    decay_time_ms: f64,
    decay_curve: f64,

    sustain_level: f64,

    release_time_ms: f64,
    release_curve: f64,

    ramp: Smoother<f64>,
    stage: AdsrStage,
    trigger: bool,
}

impl AdsrEnvelope {
    /// Creates a new ADSR envelope with the following default settings:
    ///
    /// - Attack:  `10.0 ms`
    /// - Decay:   `100.0 ms`
    /// - Sustain: `50.0 %`
    /// - Release: `50.0 ms`
    ///
    /// The envelope starts in an idle state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            attack_time_ms: DEFAULT_ATTACK_TIME_MS,
            attack_level: 1.0,
            attack_curve: DEFAULT_CURVE_AMOUNT,

            decay_time_ms: DEFAULT_DECAY_TIME_MS,
            decay_curve: DEFAULT_CURVE_AMOUNT,

            sustain_level: DEFAULT_SUSTAIN_LEVEL,

            release_time_ms: DEFAULT_RELEASE_TIME_MS,
            release_curve: DEFAULT_CURVE_AMOUNT,

            ramp: Smoother::new(0.0, 1.0),
            stage: AdsrStage::Idle,
            trigger: false,
        }
    }

    /// Progresses the state of the envelope by one sample, returning its new value.
    ///
    /// This method automatically updates the stage of the envelope based on the input
    /// trigger, and is intended to be called at the sample rate.
    pub fn next(&mut self) -> f64 {
        self.update_stage(self.trigger);

        // has the ramp finished?
        if !self.ramp.is_active() {
            self.progress_stage();
        }

        self.ramp.next()
    }

    /// Sets the envelope's trigger.
    pub fn trigger(&mut self, trigger: bool) {
        self.trigger = trigger;
    }

    /// Sets the main parameters of the envelope at once.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if a timing parameter is negative or if the sustain
    /// level is outside the range of `0.0` to `1.0`.
    pub fn set_parameters(
        &mut self,
        attack_time_ms: f64,
        decay_time_ms: f64,
        sustain_level: f64,
        release_time_ms: f64,
    ) {
        self.attack_time_ms = attack_time_ms;
        self.decay_time_ms = decay_time_ms;
        self.sustain_level = sustain_level;
        self.release_time_ms = release_time_ms;
        self.debug_parameter_assertions();
    }

    /// Sets the attack time of the envelope in milliseconds.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided attack time is negative.
    pub fn set_attack_time_ms(&mut self, attack_time_ms: f64) {
        self.attack_time_ms = attack_time_ms;
        self.debug_parameter_assertions();
    }

    /// Sets the attack level of the envelope.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided level is outside the range of
    /// `0.0` to `1.0`.
    pub fn set_attack_level(&mut self, attack_level: f64) {
        self.attack_level = attack_level;
        self.debug_parameter_assertions();
    }

    /// Sets the attack curve of the envelope. Positive values "skew upwards".
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided value is outside the range of
    /// `-1.0` to `1.0`.
    pub fn set_attack_curve(&mut self, curve_amount: f64) {
        self.attack_curve = curve_amount;
        self.debug_parameter_assertions();
    }

    /// Sets the decay time of the envelope in milliseconds.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided decay time is negative.
    pub fn set_decay_time_ms(&mut self, decay_time_ms: f64) {
        self.decay_time_ms = decay_time_ms;
        self.debug_parameter_assertions();
    }

    /// Sets the decay curve of the envelope. Positive values "skew upwards".
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided value is outside the range of
    /// `-1.0` to `1.0`.
    pub fn set_decay_curve(&mut self, curve_amount: f64) {
        self.decay_curve = curve_amount;
        self.debug_parameter_assertions();
    }

    /// Sets the sustain level of the envelope between `0.0` and `1.0`.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided sustain level is outside of the range of
    /// `0.0` to `1.0`.
    pub fn set_sustain_level(&mut self, sustain_level: f64) {
        self.sustain_level = sustain_level;
        self.debug_parameter_assertions();
    }

    /// Sets the release time of the envelope in milliseconds.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided release time is negative.
    pub fn set_release_time_ms(&mut self, release_time_ms: f64) {
        self.release_time_ms = release_time_ms;
        self.debug_parameter_assertions();
    }

    /// Sets the decay curve of the envelope. Positive values "skew upwards".
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided value is outside the range of
    /// `-1.0` to `1.0`.
    pub fn set_release_curve(&mut self, curve_amount: f64) {
        self.release_curve = curve_amount;
        self.debug_parameter_assertions();
    }

    /// Returns the current `AdsrStage` of the envelope generator.
    #[must_use]
    pub fn get_stage(&self) -> AdsrStage {
        self.stage
    }

    /// Returns whether the envelope is in an idle stage or not.
    pub fn is_idle(&self) -> bool {
        matches!(self.stage, AS::Idle)
    }

    /* PRIVATE METHODS */

    /// Updates the stage of the envelope based on the state of the provided trigger.
    fn update_stage(&mut self, trigger: bool) {
        match self.stage {
            AS::Idle | AS::Release => {
                if trigger {
                    self.set_attack_stage();
                }
            }
            AS::Attack | AS::Decay | AS::Sustain => {
                if !trigger {
                    self.set_release_stage();
                }
            }
        }
    }

    /// Progresses the stage of the envelope when its current ramp is finished.
    fn progress_stage(&mut self) {
        match self.stage {
            AS::Idle | AS::Sustain => (),
            AS::Attack => self.set_decay_stage(),
            AS::Decay => self.set_sustain_stage(),
            AS::Release => self.set_idle_stage(),
        }
    }

    /// Internally sets the envelope to its idle state.
    fn set_idle_stage(&mut self) {
        /// target 0.0, no ramping
        self.ramp.set_target_value(0.0);
        self.ramp.set_smoothing_period(0.0);
        self.stage = AS::Idle;
    }

    /// Internally sets the envelope to its attack state.
    fn set_attack_stage(&mut self) {
        // target attack level, attack time ramping
        self.ramp
            .set_smoothing_type(SmoothingType::CurveNormal(self.attack_curve));
        self.ramp.set_target_value(self.attack_level);
        self.ramp.set_smoothing_period(self.attack_time_ms);
        self.stage = AS::Attack;
    }

    /// Internally sets the envelope to its decay state.
    fn set_decay_stage(&mut self) {
        // target sustain level, decay time ramping
        self.ramp
            .set_smoothing_type(SmoothingType::CurveNormal(self.decay_curve));
        self.ramp.set_target_value(self.sustain_level);
        self.ramp.set_smoothing_period(self.decay_time_ms);
        self.stage = AS::Decay;
    }

    /// Internally sets the envelope to its sustain state.
    fn set_sustain_stage(&mut self) {
        // target sustain level, no ramping
        self.ramp.set_target_value(self.sustain_level);
        self.ramp.finish();
        self.stage = AS::Sustain;
    }

    /// Internally sets the envelope to its release state.
    fn set_release_stage(&mut self) {
        // target 0.0, release time ramping
        self.ramp
            .set_smoothing_type(SmoothingType::CurveNormal(self.release_curve));
        self.ramp.set_target_value(0.0);
        self.ramp.set_smoothing_period(self.release_time_ms);
        self.stage = AS::Release;
    }

    /// Debug assertions to ensure the provided parameters are within the appropriate ranges.
    fn debug_parameter_assertions(&self) {
        let Self {
            attack_time_ms: att,
            attack_level: att_lvl,
            attack_curve: att_crv,
            decay_time_ms: dec,
            decay_curve: dec_crv,
            sustain_level: sus,
            release_time_ms: rel,
            release_curve: rel_crv,
            ..
        } = self;

        debug_assert!(
            att.is_sign_positive()
                && dec.is_sign_positive()
                && rel.is_sign_positive()
                && (0.0..=1.0).contains(sus)
                && (0.0..=1.0).contains(att_lvl)
                && (-1.0..=1.0).contains(att_crv)
                && (-1.0..=1.0).contains(dec_crv)
                && (-1.0..=1.0).contains(rel_crv)
        );
    }
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn sus_out_of_range() {
        let mut env = AdsrEnvelope::new();
        env.set_sustain_level(1.2);
    }

    #[test]
    #[should_panic]
    fn att_negative() {
        let mut env = AdsrEnvelope::new();
        env.set_attack_time_ms(-8472.24);
    }

    #[test]
    // yes thank you clippy, very cool
    #[allow(clippy::cognitive_complexity)]
    fn correct_stages() {
        let mut env = AdsrEnvelope::new();
        let samples_as_ms = 10.0 / unsafe { SAMPLE_RATE } * 1000.0;
        env.set_parameters(samples_as_ms, samples_as_ms, 0.5, samples_as_ms);

        // starts idle?
        assert!(matches!(env.get_stage(), AdsrStage::Idle));

        for _ in 0..5 {
            env.next();
        }

        // attack stage at first?
        assert!(matches!(env.get_stage(), AdsrStage::Attack));

        for _ in 0..10 {
            env.next();
        }

        // decay after attack?
        assert!(matches!(env.get_stage(), AdsrStage::Decay));

        for _ in 0..10 {
            env.next();
        }

        // sustain after decay?
        assert!(matches!(env.get_stage(), AdsrStage::Sustain));

        for _ in 0..10000 {
            env.next();
        }

        // holds sustain whilst still triggered?
        assert!(matches!(env.get_stage(), AdsrStage::Sustain));

        env.next();

        // enters release after sustain?
        assert!(matches!(env.get_stage(), AdsrStage::Release));

        for _ in 0..5 {
            env.next();
        }

        // enters attack if triggered during release?
        assert!(matches!(env.get_stage(), AdsrStage::Attack));

        for _ in 0..5 {
            env.next();
        }

        // enters release if not triggered during attack?
        assert!(matches!(env.get_stage(), AdsrStage::Release));

        for _ in 0..6 {
            env.next();
        }

        // returns to idle after release?
        assert!(matches!(env.get_stage(), AdsrStage::Idle));
    }
}
