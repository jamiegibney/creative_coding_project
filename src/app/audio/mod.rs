use super::*;
use crate::{dsp::filters::biquad::*, util};

/// The audio state for the whole program.
pub struct AudioModel {
    rng: SmallRng,
    // TODO: use this sample rate rather than a static mut?
    sample_rate: f64,
    pub filter: BiquadFilter,
    pub filter_2: BiquadFilter,
}

impl AudioModel {
    pub fn new() -> Self {
        let sample_rate = unsafe { SAMPLE_RATE };
        Self {
            rng: SmallRng::seed_from_u64(0),
            sample_rate,
            filter: BiquadFilter::new(sample_rate),
            filter_2: BiquadFilter::new(sample_rate),
        }
    }

    pub fn initialise(&mut self) {
        let params = FilterParams {
            freq: 500.0,
            gain: 10.0,
            q: 10.0,
            filter_type: FilterType::Lowpass,
        };
        self.filter.set_params(&params);
        self.filter_2.set_params(&params);
        self.filter_2.set_type(FilterType::Highpass);
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

        let volume = util::db_to_level(-18.0) as f32;

        f[0] = sample * volume;
        f[1] = sample * volume;
    }
}
