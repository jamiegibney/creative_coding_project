#![allow(unused)]
use super::ramp::Ramp;
use crate::prelude::*;
use AdsrStage as AS;

const DEFAULT_ATTACK_TIME_MS: f64 = 10.0;
const DEFAULT_DECAY_TIME_MS: f64 = 100.0;
const DEFAULT_SUSTAIN_LEVEL: f64 = 0.5;
const DEFAULT_RELEASE_TIME_MS: f64 = 50.0;

/// this is here to make it easier to add a variable attack level in the future
const ATTACK_LEVEL: f64 = 1.0;

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

/// A linear envelope generator with attack, decay, sustain, and release (ADSR) stages.
///
/// TODO: add the ability to provide a transfer function to each stage?
#[derive(Debug)]
pub struct AdsrEnvelope {
    attack_time_ms: f64,
    decay_time_ms: f64,
    sustain_level: f64,
    release_time_ms: f64,

    ramp: Ramp,
    stage: AdsrStage,
}

impl AdsrEnvelope {
    /// Creates a new ADSR envelope with the following default settings:
    ///
    /// Attack:  10.0 ms
    /// Decay:   100.0 ms
    /// Sustain: 50.0 %
    /// Release: 50.0 ms
    ///
    /// The envelopes starts in an idle state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            attack_time_ms: DEFAULT_ATTACK_TIME_MS,
            decay_time_ms: DEFAULT_DECAY_TIME_MS,
            sustain_level: DEFAULT_SUSTAIN_LEVEL,
            release_time_ms: DEFAULT_RELEASE_TIME_MS,
            ramp: Ramp::new(0.0, 0.0),
            stage: AdsrStage::Idle,
        }
    }

    /// Progresses the state of the envelope by one sample, returning its new value.
    ///
    /// This method automatically updates the stage of the envelope based on the input
    /// trigger, and is intended to be called at the sample rate.
    pub fn next(&mut self, trigger: bool) -> f64 {
        self.update_stage(trigger);

        // has the ramp finished?
        if !self.ramp.is_active() {
            // self.progress_stage();
        }

        self.ramp.next()
    }

    /// Sets all of the parameters of the envelope at once.
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

    /// Sets the decay time of the envelope in milliseconds.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the provided decay time is negative.
    pub fn set_decay_time_ms(&mut self, decay_time_ms: f64) {
        self.decay_time_ms = decay_time_ms;
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

    /// Returns the current `AdsrStage` of the envelope generator.
    #[must_use]
    pub fn get_stage(&self) -> AdsrStage {
        self.stage
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
        self.ramp.reset(0.0, 0.0);
        self.stage = AS::Idle;
    }

    /// Internally sets the envelope to its attack state.
    fn set_attack_stage(&mut self) {
        // target attack level, attack time ramping
        self.ramp.reset(ATTACK_LEVEL, self.attack_time_ms / 1000.0);
        self.stage = AS::Attack;
    }

    /// Internally sets the envelope to its decay state.
    fn set_decay_stage(&mut self) {
        // target sustain level, decay time ramping
        self.ramp
            .reset(self.sustain_level, self.decay_time_ms / 1000.0);
        self.stage = AS::Decay;
    }

    /// Internally sets the envelope to its sustain state.
    fn set_sustain_stage(&mut self) {
        // target sustain level, no ramping
        self.ramp.reset(self.sustain_level, 0.0);
        self.stage = AS::Sustain;
    }

    /// Internally sets the envelope to its release state.
    fn set_release_stage(&mut self) {
        // target 0.0, release time ramping
        self.ramp.reset(0.0, self.release_time_ms / 1000.0);
        self.stage = AS::Release;
    }

    /// Debug assertions to ensure the provided parameters are within the appropriate ranges.
    #[cfg(debug_assertions)]
    fn debug_parameter_assertions(&self) {
        let Self {
            attack_time_ms: att,
            decay_time_ms: dec,
            sustain_level: sus,
            release_time_ms: rel,
            ..
        } = self;

        debug_assert!(
            att.is_sign_positive()
                && dec.is_sign_positive()
                && rel.is_sign_positive()
                && (0.0..=1.0).contains(sus)
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
    fn correct_stages() {
        let mut env = AdsrEnvelope::new();
        let samples_as_ms = 10.0 / unsafe { SAMPLE_RATE } * 1000.0;
        env.set_parameters(samples_as_ms, samples_as_ms, 0.5, samples_as_ms);

        assert!(matches!(env.get_stage(), AdsrStage::Idle));

        for _ in 0..5 {
            env.next(true);
        }

        assert!(matches!(env.get_stage(), AdsrStage::Attack));

        for _ in 0..10 {
            env.next(true);
        }

        assert!(matches!(env.get_stage(), AdsrStage::Decay));

        for _ in 0..10 {
            env.next(true);
        }

        assert!(matches!(env.get_stage(), AdsrStage::Sustain));

        for _ in 0..10000 {
            env.next(true);
        }

        assert!(matches!(env.get_stage(), AdsrStage::Sustain));

        env.next(false);

        assert!(matches!(env.get_stage(), AdsrStage::Release));

        for _ in 0..5 {
            env.next(true);
        }

        assert!(matches!(env.get_stage(), AdsrStage::Attack));

        for _ in 0..5 {
            env.next(false);
        }

        assert!(matches!(env.get_stage(), AdsrStage::Release));

        for _ in 0..5 {
            env.next(false);
        }

        assert!(matches!(env.get_stage(), AdsrStage::Idle));
    }
}
