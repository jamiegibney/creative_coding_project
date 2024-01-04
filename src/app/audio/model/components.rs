use crossbeam_channel::{Receiver as CCReceiver, Sender as CCSender};
use std::time::Instant;

use crate::dsp::filtering::{
    comb::delay::Delay, resonator::resonator_bank::ResoBankData,
};

use super::*;

#[derive(Default)]
pub struct AudioProcessors {
    // FILTERS
    pub filter_low: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_peak: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_high: Box<[BiquadFilter; NUM_CHANNELS]>,

    pub filter_pk_ts: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_hs_ts: Box<[BiquadFilter; NUM_CHANNELS]>,
    pub filter_peak_ts: Box<[BiquadFilter; NUM_CHANNELS]>,

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
    pub compressor: Box<Compressor>,

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

    pub low_filter_is_shelf: bool,
    pub high_filter_is_shelf: bool,

    pub distortion_algorithm: DistortionType,

    pub spectral_filter_size: usize,
    pub spectral_mask_post_fx: bool,
    pub average_load: Vec<f64>,
    pub average_pos: usize,

    pub reso_bank_scale: Scale,
    pub reso_bank_root_note: f64,

    pub delay_time_ms: f64,

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

            low_filter_is_shelf: false,
            high_filter_is_shelf: false,

            spectral_mask_post_fx: false,
            spectral_filter_size: 1024,
            average_load: Vec::default(),
            average_pos: Default::default(),

            reso_bank_scale: Scale::default(),
            reso_bank_root_note: 69.0,

            delay_time_ms: 250.0,

            distortion_algorithm: DistortionType::default(),

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

    pub reso_bank_data: Option<triple_buffer::Output<ResoBankData>>,
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
