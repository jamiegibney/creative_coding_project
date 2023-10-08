#![allow(unused)]
use crate::settings::SAMPLE_RATE;
use EnvelopeState as ES;

const DEFAULT_ATTACK_TIME_MS: f64 = 10.0;
const DEFAULT_DECAY_TIME_MS: f64 = 100.0;
const DEFAULT_SUSTAIN_LEVEL: f64 = 0.5;
const DEFAULT_RELEASE_TIME_MS: f64 = 50.0;

/// this is only used to allow for an easier conversion to a variable
/// attack level in the future
const ATTACK_LEVEL: f64 = 1.0;

// helper function
// fn sample_rate() -> f64 {
//     unsafe { SAMPLE_RATE }
// }

#[derive(Debug, Default, Clone, Copy)]
enum EnvelopeState {
    #[default]
    Idle,

    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy)]
struct Adsr {
    attack_time_ms: f64,
    decay_time_ms: f64,
    sustain_level: f64,
    release_time_ms: f64,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            attack_time_ms: DEFAULT_ATTACK_TIME_MS,
            decay_time_ms: DEFAULT_DECAY_TIME_MS,
            sustain_level: DEFAULT_SUSTAIN_LEVEL,
            release_time_ms: DEFAULT_RELEASE_TIME_MS,
        }
    }
}

#[derive(Default)]
pub struct Envelope {
    adsr: Adsr,
    state: EnvelopeState,
    previous_level: f64,
    target_level: f64,
    current_level: f64,
    step_size: f64,
    // sample_rate: f64,
}

impl Envelope {
    /// Creates a new `Envelope` with the default ADSR settings:
    ///
    /// Attack time:   10 ms
    /// Decay time:    100 ms
    /// Sustain level: 1.0 x
    /// Release time:  50 ms
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next(&mut self, signal: bool) -> f64 {
        self.update_level();
        self.current_level
    }

    /// Sets the attack time of the envelope in milliseconds.
    ///
    /// # Panics
    ///
    /// Will panic in debug mode if the provided attack time is negative.
    pub fn set_attack_time(&mut self, attack_time_ms: f64) {
        self.adsr.attack_time_ms = attack_time_ms;
        self.adsr_assert();
    }

    /// Sets the decay time of the envelope in milliseconds.
    ///
    /// # Panics
    ///
    /// Will panic in debug mode if the provided decay time is negative.
    pub fn set_decay_time(&mut self, decay_time_ms: f64) {
        self.adsr.decay_time_ms = decay_time_ms;
        self.adsr_assert();
    }

    /// Sets the sustain level of the envelope within the range `0.0` to `1.0`.
    ///
    /// # Panics
    ///
    /// Will panic in debug mode if the provided sustain level is outside of the above range.
    pub fn set_sustain_level(&mut self, sustain_level: f64) {
        self.adsr.sustain_level = sustain_level;
        self.adsr_assert();
    }

    /// Sets the release time of the envelope in milliseconds.
    ///
    /// # Panics
    ///
    /// Will panic in debug mode if the provided release time is negative.
    pub fn set_release_time(&mut self, release_time_ms: f64) {
        self.adsr.release_time_ms = release_time_ms;
        self.adsr_assert();
    }

    /// Debug assertions relating to ADSR parameters.
    fn adsr_assert(&self) {
        let Adsr {
            attack_time_ms: a,
            decay_time_ms: d,
            sustain_level: s,
            release_time_ms: r,
        } = self.adsr;

        debug_assert!(
            a.is_sign_positive()
                && d.is_sign_positive()
                && r.is_sign_positive()
                && (0.0..=1.0).contains(&s)
        );
    }

    fn set_step_size(&mut self) {
        let sample_length = unsafe { SAMPLE_RATE }.recip();
        self.step_size = match self.state {
            ES::Idle | ES::Sustain => 0.0,
            ES::Attack => sample_length * self.adsr.attack_time_ms / 1000.0,
            ES::Decay => sample_length * self.adsr.decay_time_ms / 1000.0,
            ES::Release => sample_length * self.adsr.release_time_ms / 1000.0,
        }
    }

    fn begin_attack(&mut self) {
        self.set_step_size();
        self.previous_level = 0.0;
        self.current_level = 0.0;
        self.target_level = ATTACK_LEVEL;
    }

    fn begin_decay(&mut self) {
        self.set_step_size();
        self.previous_level = ATTACK_LEVEL;
        self.target_level = self.adsr.sustain_level;
        self.state = ES::Decay;
    }

    fn begin_sustain(&mut self) {
        self.set_step_size();
        let sus = self.adsr.sustain_level;
        self.previous_level = sus;
        self.current_level = sus;
        self.current_level = sus;
        self.state = ES::Sustain;

    }

    fn begin_release(&mut self) {
        self.set_step_size();
        self.previous_level = self.current_level;
        self.target_level = 0.0;
        self.state = ES::Release;
    }

    fn update_level(&mut self) {
        self.current_level += self.step_size;
        if self.current_level < 0.0 || 1.0 < self.current_level {
            dbg!(
                self.current_level,
                "Envelope exceeded normal range; clamping to range"
            );
            self.current_level.clamp(0.0, 1.0);
        }
    }
}
