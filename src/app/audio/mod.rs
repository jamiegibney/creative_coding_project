use super::*;
use crate::dsp::adsr::AdsrEnvelope;
use crate::{dsp::filters::biquad::*, prelude::*};
use std::sync::{mpsc, Arc, Mutex};

type Sender<T> = Arc<Mutex<mpsc::Sender<T>>>;
type Receiver<T> = Arc<Mutex<mpsc::Receiver<T>>>;

/// The audio state for the whole program.
pub struct AudioModel {
    rng: SmallRng,
    // TODO: use this sample rate rather than a static mut?
    sample_rate: f64,
    pub filter: BiquadFilter,
    pub filter_2: BiquadFilter,

    volume: f64,

    envelope: AdsrEnvelope,
    envelope_trigger: bool,
    envelope_trigger_receiver: Option<mpsc::Receiver<bool>>,
}

impl AudioModel {
    pub fn new() -> Self {
        let sample_rate = unsafe { SAMPLE_RATE };
        Self {
            rng: SmallRng::seed_from_u64(0),
            sample_rate,
            filter: BiquadFilter::new(sample_rate),
            filter_2: BiquadFilter::new(sample_rate),

            volume: db_to_level(-18.0),

            envelope: AdsrEnvelope::new(),
            envelope_trigger: false,
            envelope_trigger_receiver: None,
        }
    }

    pub fn initialise(&mut self) -> mpsc::Sender<bool> {
        let params = FilterParams {
            freq: 500.0,
            gain: 10.0,
            q: 10.0,
            filter_type: FilterType::Lowpass,
        };
        self.filter.set_params(&params);
        self.filter_2.set_params(&params);
        self.filter_2.set_type(FilterType::Highpass);
        self.envelope.set_parameters(10.0, 500.0, 0.2, 1000.0);

        let (sender, receiver) = mpsc::channel();
        // let sender = Arc::new(Mutex::new(sender));

        self.envelope_trigger_receiver = Some(receiver);

        // Arc::clone(&sender)
        sender
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
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
        let sample = audio.filter_2.process(audio.filter.process(noise)) as f32;

        if let Some(trigger) = &audio.envelope_trigger_receiver {
            if let Ok(msg) = trigger.try_recv() {
                audio.envelope_trigger = msg;
            }
        }

        let env_level = audio.envelope.next(audio.envelope_trigger);
        let volume = audio.volume * env_level;

        f[0] = sample * volume as f32;
        f[1] = sample * volume as f32;
    }
}
