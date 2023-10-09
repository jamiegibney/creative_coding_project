use super::*;
use crate::dsp::ramp;
use crate::dsp::*;
use crate::dsp::{adsr::AdsrEnvelope, ramp::Ramp};
use std::sync::mpsc::{channel, Receiver, Sender};

/// A struct containing the channel senders returned by
/// `AudioModel::initialize()`.
///
/// The fields of this struct are used to communicate directly
/// with the audio thread.
pub struct AudioSenders {
    pub envelope_trigger: Sender<bool>,
    pub filter_freq: Sender<f64>,
}

/// The audio state for the whole program.
pub struct AudioModel {
    rng: SmallRng,
    pub filter_lp: [BiquadFilter; 2],
    pub filter_hp: [BiquadFilter; 2],
    pub filter_peak: [BiquadFilter; 2],
    pub filter_comb: [IirCombFilter; 2],

    pub filter_freq: Ramp,
    filter_freq_receiver: Option<Receiver<f64>>,
    volume: f64,

    pub envelope: AdsrEnvelope,
    envelope_trigger: bool,
    envelope_trigger_receiver: Option<Receiver<bool>>,
}

impl AudioModel {
    pub fn new() -> Self {
        let sample_rate = unsafe { SAMPLE_RATE };
        let comb = IirCombFilter::with_interpolation(true);

        Self {
            rng: SmallRng::seed_from_u64(0),
            filter_lp: [BiquadFilter::new(sample_rate); 2],
            filter_hp: [BiquadFilter::new(sample_rate); 2],
            filter_peak: [BiquadFilter::new(sample_rate); 2],
            filter_comb: [comb.clone(), comb],

            filter_freq: Ramp::new(440.0, 0.0),
            filter_freq_receiver: None,

            volume: db_to_level(-18.0),
            // volume: db_to_level(-42.0),
            envelope: AdsrEnvelope::new(),
            envelope_trigger: false,
            envelope_trigger_receiver: None,
        }
    }

    pub fn set_filters(&mut self) {
        let params = FilterParams {
            freq: 440.0,
            gain: 0.0,
            q: 30.0,
            filter_type: FilterType::Lowpass,
        };

        for lp in &mut self.filter_lp {
            lp.set_params(&params);
        }

        for hp in &mut self.filter_hp {
            hp.set_params(&params);
            hp.set_type(FilterType::Highpass);
        }

        for peak in &mut self.filter_peak {
            peak.set_params(&params);
            peak.set_type(FilterType::Peak);
            peak.set_gain(10.0);
        }

        for comb in &mut self.filter_comb {
            comb.set_positive_polarity(true);
            comb.set_interpolation(InterpType::CatmullCubic);
            comb.set_gain_db(-0.2);
        }
    }

    pub fn set_filter_freq(&mut self, freq: f64) {
        for lp in &mut self.filter_lp {
            lp.set_freq(freq);
        }

        for hp in &mut self.filter_hp {
            hp.set_freq(freq);
        }

        for peak in &mut self.filter_peak {
            peak.set_freq(freq);
        }

        for comb in &mut self.filter_comb {
            comb.set_freq(freq);
        }
    }

    pub fn process_filters(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        let l = self.filter_peak[0].process(
            self.filter_hp[0].process(self.filter_lp[0].process(sample_l)),
        );
        let r = self.filter_peak[1].process(
            self.filter_hp[1].process(self.filter_lp[1].process(sample_r)),
        );

        (l, r)
    }

    pub fn process_comb_filters(
        &mut self,
        (sample_l, sample_r): (f64, f64),
    ) -> (f64, f64) {
        let l = self.filter_comb[0].process(sample_l);
        let r = self.filter_comb[1].process(sample_r);

        (l, r)
    }

    pub fn initialize(&mut self) -> AudioSenders {
        self.set_filters();

        self.envelope.set_parameters(1.0, 50.0, 0.0, 50.0);

        let (envelope_trigger_sender, receiver) = channel();
        self.envelope_trigger_receiver = Some(receiver);

        let (filter_freq_sender, receiver) = channel();
        self.filter_freq_receiver = Some(receiver);

        AudioSenders {
            envelope_trigger: envelope_trigger_sender,
            filter_freq: filter_freq_sender,
        }
    }

    pub fn try_receive(&mut self) {
        // envelope trigger
        if let Some(trigger) = &self.envelope_trigger_receiver {
            if let Ok(msg) = trigger.try_recv() {
                self.envelope_trigger = msg;
            }
        }

        // filter frequency
        if let Some(freq) = &self.filter_freq_receiver {
            if let Ok(msg) = freq.try_recv() {
                self.filter_freq.reset(msg, 0.01);
            }
        }
    }
}

impl Default for AudioModel {
    fn default() -> Self {
        Self::new()
    }
}

/// The main audio processing callback.
pub fn audio(audio: &mut AudioModel, output: &mut Buffer) {
    for f in output.frames_mut() {
        let noise = || nannou::rand::random_f64().mul_add(2.0, -1.0);

        let env_level = audio.envelope.next(audio.envelope_trigger);
        let volume = audio.volume * (env_level);

        let noise = (noise() * volume, noise() * volume);
        // let (out_l, out_r) = audio.process_filters(noise);
        let (out_l, out_r) = audio.process_comb_filters(noise);

        audio.try_receive();
        let freq = audio.filter_freq.next();

        if audio.filter_freq.is_active() {
            audio.set_filter_freq(freq);
        }

        f[0] = out_l as f32;
        f[1] = out_r as f32;
    }
}
