use super::*;
use crate::dsp::*;
use crate::prelude::*;
use two_pole_resonator::TwoPoleResonator;

type Resonator = AudioUtility<StereoWrapper<TwoPoleResonator>>;

#[derive(Clone, Debug, Default)]
pub struct ResonatorBankParams {
    pub root_note: f64,
    pub scale: Scale,

    /// Whether each resonator's left and right filter should have the same pitch.
    // stereo_link: bool,
    /// Whether each resonator's pitch should be quantised to the musical scale.
    pub quantise_to_scale: bool,
    /// The overall range of (original) resonator pitches.
    pub freq_spread: f64,
    /// The overall shift in (original) resonator pitches.
    pub freq_shift: f64,
    /// The amount each pitch can skew towards its original value.
    pub inharm: f64,
}

#[derive(Clone, Debug, Default)]
pub struct ResonatorBank {
    resonators: Vec<Resonator>,
    original_pitches: Vec<f64>,
    active_pitches: Vec<Smoother<f64>>,
    params: ResonatorBankParams,
}

impl ResonatorBank {
    const INHARM_SCALE: f64 = 0.01;
    const NOTE_MIN: f64 = 20.0; // 25.96 Hz
    const NOTE_MAX: f64 = 128.0; // 13,289.75 Hz

    pub fn new(sample_rate: f64, max_num_resonators: usize) -> Self {
        assert!(max_num_resonators > 0);

        let mut smoother = Smoother::new(1000.0, 72.0, sample_rate);
        smoother.set_smoothing_type(SmoothingType::Cosine);

        let mut s = Self {
            resonators: vec![
                AudioUtility::new(StereoWrapper::from_single(TwoPoleResonator::new(
                    sample_rate
                )));
                max_num_resonators
            ],
            original_pitches: vec![0.0; max_num_resonators],
            active_pitches: vec![smoother; max_num_resonators],
            // stereo_link: true,
            params: ResonatorBankParams {
                freq_shift: 0.0,
                freq_spread: 0.0,
                root_note: 69.0,
                quantise_to_scale: false,
                scale: Scale::default(),
                inharm: 0.0,
            },
        };

        s.resonators.iter_mut().for_each(|res| {
            res.l.set_resonance(0.9999);
            res.r.set_resonance(0.9999);
            res.set_gain_db(-75.0);
        });

        s
    }

    pub fn set_params(&mut self, params: ResonatorBankParams) {
        self.params = params;
        self.set_active_pitches();
    }

    pub fn set_num_resonators(&mut self, num_resonators: usize) {
        assert!(num_resonators <= self.resonators.capacity() && num_resonators != 0);
        unsafe {
            self.resonators.set_len(num_resonators);
            self.active_pitches.set_len(num_resonators);
            self.original_pitches.set_len(num_resonators);
        }
    }

    pub fn randomise_resonator_pitches(&mut self) {
        let min = lerp(Self::NOTE_MIN, 78.0, 1.0 - self.params.freq_spread);
        let max = lerp(66.0, Self::NOTE_MAX, self.params.freq_spread);

        self.original_pitches.iter_mut().for_each(|p| {
            *p = scale(random_f64(), min, max);
        });

        self.apply_freq_shift();
        self.set_active_pitches();
    }

    pub fn randomise_pan(&mut self, max_panning: f64) {
        self.resonators.iter_mut().for_each(|res| {
            res.set_pan(2.0f64.mul_add(random_f64(), -1.0) * max_panning.clamp(0.0, 1.0));
        });
    }

    pub fn quantise_to_scale(&mut self, quantise_to_scale: bool) {
        self.params.quantise_to_scale = quantise_to_scale;
        self.set_active_pitches();
    }

    /// When `spread` is `0.0`, all notes are constrained to 1 octave of range.
    /// When `spread` is `1.0`, all notes are constrained to 9 octaves of range.
    pub fn set_freq_spread(&mut self, spread: f64) {
        self.params.freq_spread = spread.clamp(0.0, 1.0);
    }

    pub fn set_freq_shift(&mut self, shift: f64) {
        self.params.freq_shift = shift;
        self.apply_freq_shift();
        self.set_active_pitches();
    }

    /// Sets the root note of the bank's scale.
    ///
    /// Only active if `quantise_to_scale` is true.
    pub fn set_root_note(&mut self, root_note_midi: f64) {
        self.params.root_note = root_note_midi;
        self.set_active_pitches();
    }

    /// Sets the internal musical scale of the bank.
    ///
    /// Only active if `quantise_to_scale` is true.
    pub fn set_scale(&mut self, scale: Scale) {
        self.params.scale = scale;
        self.set_active_pitches();
    }

    pub fn set_inharm(&mut self, inharm: f64) {
        self.params.inharm = inharm.clamp(0.0, 1.0) * Self::INHARM_SCALE;
        self.set_active_pitches();
    }

    pub fn inner(&self) -> &[Resonator] {
        &self.resonators
    }

    pub fn inner_mut(&mut self) -> &mut [Resonator] {
        &mut self.resonators
    }

    fn update_filters(&mut self) {
        // avoid recalculating filter coefs if the pitches haven't changed
        if !self.active_pitches[0].is_active() {
            return;
        }

        self.resonators
            .iter_mut()
            .zip(self.active_pitches.iter_mut())
            .for_each(|(res, p)| {
                let note = p.next();

                res.l.set_cutoff(note_to_freq(note));
                res.r.set_cutoff(note_to_freq(note));
            });
    }

    fn set_active_pitches(&mut self) {
        self.active_pitches
            .iter_mut()
            .zip(self.original_pitches.iter())
            .for_each(|(p, &original)| {
                if self.params.quantise_to_scale {
                    let quantised = self
                        .params
                        .scale
                        .quantise_to_scale(original, self.params.root_note);

                    p.set_target_value(lerp(quantised, original, self.params.inharm));
                } else {
                    p.set_target_value(original);
                }
            });
    }

    fn apply_freq_shift(&mut self) {
        self.original_pitches.iter_mut().for_each(|p| {
            *p += self.params.freq_shift;
        });
    }
}

impl Effect for ResonatorBank {
    fn process_mono(&mut self, mut input: f64, ch_idx: usize) -> f64 {
        self.update_filters();
        for res in &mut self.resonators {
            input = res.process_mono(input, ch_idx);
        }

        input
    }

    fn process_stereo(&mut self, mut left: f64, mut right: f64) -> (f64, f64) {
        self.update_filters();
        let (mut out_l, mut out_r) = (0.0, 0.0);
        for res in &mut self.resonators {
            let (l, r) = res.process_stereo(left, right);
            out_l += l;
            out_r += r;
        }

        (out_l, out_r)
    }

    fn get_sample_rate(&self) -> f64 {
        self.resonators[0].get_sample_rate()
    }
}
