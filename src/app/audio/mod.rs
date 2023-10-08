use super::*;
use crate::dsp::filters::biquad::*;
use crate::dsp::ramp;
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
    // TODO: use this sample rate rather than a static mut?
    sample_rate: f64,
    pub filter_lp: BiquadFilter,
    pub filter_hp: BiquadFilter,
    pub filter_peak: BiquadFilter,

    filter_freq: Ramp,
    filter_freq_receiver: Option<Receiver<f64>>,
    volume: f64,

    envelope: AdsrEnvelope,
    envelope_trigger: bool,
    envelope_trigger_receiver: Option<Receiver<bool>>,
}

impl AudioModel {
    pub fn new() -> Self {
        let sample_rate = unsafe { SAMPLE_RATE };
        Self {
            rng: SmallRng::seed_from_u64(0),
            sample_rate,
            filter_lp: BiquadFilter::new(sample_rate),
            filter_hp: BiquadFilter::new(sample_rate),
            filter_peak: BiquadFilter::new(sample_rate),

            filter_freq: Ramp::new(440.0, 0.0),
            filter_freq_receiver: None,

            volume: db_to_level(-42.0),

            envelope: AdsrEnvelope::new(),
            envelope_trigger: false,
            envelope_trigger_receiver: None,
        }
    }

    pub fn initialize(&mut self) -> AudioSenders {
        let params = FilterParams {
            freq: 440.0,
            gain: 0.0,
            q: 30.0,
            filter_type: FilterType::Lowpass,
        };

        self.filter_lp.set_params(&params);

        self.filter_hp.set_params(&params);
        self.filter_hp.set_type(FilterType::Highpass);

        self.filter_peak.set_params(&params);
        self.filter_peak.set_type(FilterType::Peak);
        self.filter_peak.set_gain(10.0);

        self.envelope.set_parameters(10.0, 500.0, 1.0, 1000.0);

        let (envelope_trigger_sender, receiver) = channel();
        self.envelope_trigger_receiver = Some(receiver);

        let (filter_freq_sender, receiver) = channel();
        self.filter_freq_receiver = Some(receiver);

        AudioSenders {
            envelope_trigger: envelope_trigger_sender,
            filter_freq: filter_freq_sender,
        }
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
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
                self.filter_freq.reset(msg, 0.1);
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
        let noise = audio.rng.gen::<f64>().mul_add(2.0, -1.0);
        let sample = audio
            .filter_peak
            .process(audio.filter_hp.process(audio.filter_lp.process(noise)))
            as f32;

        audio.try_receive();
        let freq = audio.filter_freq.next();
        audio.filter_lp.set_freq(freq);
        audio.filter_hp.set_freq(freq);
        audio.filter_peak.set_freq(freq);

        let env_level = audio.envelope.next(audio.envelope_trigger);
        let volume = audio.volume * env_level;

        f[0] = sample * volume as f32;
        f[1] = sample * volume as f32;
    }
}
