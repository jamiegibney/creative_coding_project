use std::time::Instant;

use super::*;

#[derive(Default)]
pub struct AudioProcessors {
    // FILTERS
    pub filter_lp: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_hp: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_peak: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_peak_post: Box<[BiquadFilter; NUM_CHANNELS]>,

    pub filter_comb: Box<[IirCombFilter; NUM_CHANNELS]>,

    pub pre_fx_dc_filter: Box<[DCFilter; NUM_CHANNELS]>,
    pub post_fx_dc_filter: Box<[DCFilter; NUM_CHANNELS]>,

    pub spectral_filter: SpectralFilter,

    // FX
    pub waveshaper: Box<[Waveshaper; NUM_CHANNELS]>,
    // TODO: compression/limiting, delay, diopser, reverb

    // OVERSAMPLING
    pub oversamplers: Vec<Oversampler>,
}

#[derive(Default)]
pub struct AudioGeneration {
    pub amp_envelope: AdsrEnvelope,
    pub generator: Generator,
}

pub struct AudioData {
    pub voice_gain: Smoother<f64>,
    pub master_gain: Smoother<f64>,

    pub sample_rate: Arc<AtomicF64>,
    pub upsampled_rate: Arc<AtomicF64>,

    pub latency_samples: u32,

    pub oversampling_factor: Arc<AtomicUsize>,

    pub is_processing: bool,
    pub idle_timer_samples: u64,

    pub average_load: Vec<f64>,
    pub average_pos: usize,

    pub callback_time_elapsed: Arc<Mutex<Instant>>,
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            voice_gain: Smoother::default(),
            master_gain: Smoother::default(),

            sample_rate: Arc::default(),
            upsampled_rate: Arc::default(),

            latency_samples: Default::default(),

            oversampling_factor: Arc::default(),

            is_processing: Default::default(),
            idle_timer_samples: Default::default(),

            average_load: Vec::default(),
            average_pos: Default::default(),

            callback_time_elapsed: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

pub struct AudioBuffers {
    pub master_gain_buffer: Vec<f64>,

    pub oversampling_buffer: OversamplingBuffer,

    pub spectral_mask: Option<triple_buffer::Output<SpectralMask>>,
}

impl Default for AudioBuffers {
    fn default() -> Self {
        Self {
            master_gain_buffer: Vec::default(),
            oversampling_buffer: OversamplingBuffer::default(),
            spectral_mask: None,
        }
    }
}

#[derive(Default)]
pub struct AudioSpectrograms {
    pub pre_fx_spectrogram: Arc<Mutex<Option<SpectrumInput>>>,
    pub pre_fx_spectrogram_buffer: Arc<Mutex<Vec<f64>>>,

    pub post_fx_spectrogram: Arc<Mutex<Option<SpectrumInput>>>,
    pub post_fx_spectrogram_buffer: Arc<Mutex<Vec<f64>>>,
}

/// The fields of this struct are used to communicate directly
/// with the audio thread.
#[derive(Default)]
pub struct AudioMessageReceivers {
    pub note_event: Option<Receiver<NoteEvent>>,

    pub filter_freq: Option<Receiver<f64>>,
    pub drive_amount: Option<Receiver<f64>>,
}

pub struct AudioMessageSenders {
    pub note_event: Sender<NoteEvent>,
    pub filter_freq: Sender<f64>,
    pub drive_amount: Sender<f64>,
}
