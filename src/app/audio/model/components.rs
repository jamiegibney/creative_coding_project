use crossbeam_channel::{Receiver as CCReceiver, Sender as CCSender};
use std::time::Instant;

use crate::dsp::filtering::comb::delay::Delay;

use super::*;

#[derive(Default)]
pub struct AudioProcessors {
    // FILTERS
    pub filter_lp: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_hp: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_peak: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_hs: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_peak_post: Box<[BiquadFilter; NUM_CHANNELS]>,

    pub filter_comb: Box<[IirCombFilter; NUM_CHANNELS]>,
    pub delay: Box<[DryWet<Delay>]>,
    pub stereo_delay: Box<DryWet<StereoDelay>>,

    pub pre_fx_dc_filter: Box<[DCFilter; NUM_CHANNELS]>,
    pub post_fx_dc_filter: Box<[DCFilter; NUM_CHANNELS]>,

    pub spectral_filter: SpectralFilter,
    pub resonator_bank: DryWet<ResonatorBank>,
    pub resonator: Box<[TwoPoleResonator; NUM_CHANNELS]>,

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
    // pub master_gain: Arc<SmootherAtomic<f64>>,
    pub sample_rate: Arc<AtomicF64>,
    pub upsampled_rate: Arc<AtomicF64>,

    pub latency_samples: u32,

    pub oversampling_factor: Arc<AtomicUsize>,

    pub is_processing: bool,
    pub idle_timer_samples: u64,

    // pub spectral_mask_post_fx: bool,
    pub average_load: Vec<f64>,
    pub average_pos: usize,

    pub sample_timer: u32,

    pub callback_time_elapsed: Arc<Mutex<Instant>>,
}

impl Default for AudioData {
    fn default() -> Self {
        Self {
            voice_gain: Smoother::default(),
            // master_gain: Arc::new(SmootherAtomic::default()),
            sample_rate: Arc::default(),
            upsampled_rate: Arc::default(),

            latency_samples: Default::default(),

            oversampling_factor: Arc::default(),

            is_processing: Default::default(),
            idle_timer_samples: Default::default(),

            // spectral_mask_post_fx: false,
            average_load: Vec::default(),
            average_pos: Default::default(),

            sample_timer: 0,

            callback_time_elapsed: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

#[derive(Default)]
pub struct AudioBuffers {
    pub master_gain_buffer: Vec<f64>,

    pub oversampling_buffer: OversamplingBuffer,

    pub spectral_mask: Option<triple_buffer::Output<SpectralMask>>,
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
    // TODO change these to crossbeam channels
    pub note_event: Option<CCReceiver<NoteEvent>>,

    pub filter_freq: Option<CCReceiver<f64>>,
    pub drive_amount: Option<CCReceiver<f64>>,
    pub resonator_bank_params: Option<CCReceiver<ResonatorBankParams>>,
    pub resonator_bank_reset_pitch: Option<CCReceiver<()>>,
    pub resonator_bank_reset_pan: Option<CCReceiver<()>>,
    pub spectral_mask_post_fx: Option<CCReceiver<()>>,
}

pub struct AudioMessageSenders {
    // TODO change these to crossbeam channels
    pub note_event: CCSender<NoteEvent>,
    pub filter_freq: CCSender<f64>,
    pub drive_amount: CCSender<f64>,
    pub resonator_bank_params: CCSender<ResonatorBankParams>,
    pub resonator_bank_reset_pitch: CCSender<()>,
    pub resonator_bank_reset_pan: CCSender<()>,
    pub spectral_mask_post_fx: CCSender<()>,
}
