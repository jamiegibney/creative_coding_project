use crate::app::audio::audio_constructor::DEFAULT_SPECTRAL_BLOCK_SIZE;
use crate::app::musical::*;
use crate::dsp::BUTTERWORTH_Q;
use crate::prelude::*;
use std::fmt::{Display, Formatter, Result};

use atomic_float::AtomicF64;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize},
    Arc, RwLock,
};

pub mod eq;
pub use eq::*;

/// All parameters controlled by the user interface.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct UIParams {
    // ### SPECTRAL FILTER ###
    /// The algorithm to use for the spectral mask.
    pub mask_algorithm: Arc<RwLock<GenerativeAlgo>>,
    /// The speed of the spectral mask scan line.
    pub mask_scan_line_speed: Arc<AtomicF64>,
    /// Whether the spectral filter is pre- or post-FX.
    pub mask_is_post_fx: Arc<AtomicBool>,
    /// Whether to use the GPU to compute the generative algorithms.
    pub mask_uses_gpu: Arc<AtomicBool>,

    // CONTOURS ALGORITHMS
    /// The number of contours to draw.
    pub contour_count: Arc<AtomicU32>,
    /// The thickness of each contour line.
    pub contour_thickness: Arc<AtomicF64>,
    /// The speed of the contour animation.
    pub contour_speed: Arc<AtomicF64>,

    // SMOOTHLIFE ALGORITHM
    /// The resolution of the smooth life simulation.
    pub smoothlife_resolution: Arc<RwLock<SmoothLifeSize>>,
    /// The speed of the smoothlife simulation.
    pub smoothlife_speed: Arc<AtomicF64>,
    /// The state preset of the smoothlife simulation.
    pub smoothlife_preset: Arc<RwLock<SmoothLifePreset>>,

    // ### SPECTROGRAMS ###
    /// The resolution of both spectrograms.
    pub spectrogram_resolution: Arc<RwLock<SpectrogramSize>>,
    /// The timing of the spectrograms.
    pub spectrogram_timing: Arc<AtomicF64>,
    /// Which spectrograms are drawn.
    pub spectrogram_view: Arc<RwLock<SpectrogramView>>,

    // ### RESONATOR BANK ###
    /// The musical scale of the resonator bank.
    pub reso_bank_scale: Arc<RwLock<Scale>>,
    /// The root note of the resonator bank.
    pub reso_bank_root_note: Arc<AtomicU8>,
    /// The frequency spread (range) of each resonator.
    pub reso_bank_spread: Arc<AtomicF64>,
    /// The frequency shift of each resonator.
    pub reso_bank_shift: Arc<AtomicF64>,
    /// How much each resonator's pitch skews towards its original pitch.
    pub reso_bank_inharm: Arc<AtomicF64>,
    /// How much panning may be applied to each resonator.
    pub reso_bank_pan: Arc<AtomicF64>,
    /// Whether the resonators should quantise their pitch to a scale.
    pub reso_bank_quantise: Arc<AtomicBool>,

    // ### POST EFFECTS ###

    // TODO: atomic smoothers required!
    // LOW-PASS
    /// The cutoff of the low-pass filter in Hz.
    pub low_pass_cutoff_hz: Arc<SmootherAtomic<f64>>,
    /// The Q value of the low-pass filter.
    pub low_pass_q: Arc<SmootherAtomic<f64>>,

    // HIGH-PASS
    /// The cutoff of the high-pass filter in Hz.
    pub high_pass_cutoff_hz: Arc<SmootherAtomic<f64>>,
    /// The Q value of the high-pass filter.
    pub high_pass_q: Arc<SmootherAtomic<f64>>,

    // PING-PONG DELAY
    /// The time between ping-pong delay taps in milliseconds.
    pub pp_delay_time_ms: Arc<SmootherAtomic<f64>>,
    /// The ping-pong delay feedback.
    pub pp_delay_feedback: Arc<SmootherAtomic<f64>>,
    /// The dry/wet mix for the ping-pong delay.
    pub pp_delay_mix: Arc<SmootherAtomic<f64>>,
    /// Whether the ping-pong delay's timing should be tempo-synced.
    pub pp_delay_tempo_sync: Arc<AtomicBool>,

    // DISTORTION
    pub dist_amount: Arc<SmootherAtomic<f64>>,
    pub dist_type: Arc<RwLock<DistortionType>>,
    // // EQ
    // /// The parameters for the three-band EQ.
    // pub eq_params: EQParams,
}

impl Default for UIParams {
    fn default() -> Self {
        Self {
            mask_algorithm: Arc::new(RwLock::new(GenerativeAlgo::default())),
            mask_scan_line_speed: Arc::new(AtomicF64::new(0.1)),
            mask_is_post_fx: Arc::new(AtomicBool::new(false)),
            mask_uses_gpu: Arc::new(AtomicBool::new(true)),

            contour_count: Arc::new(AtomicU32::new(8)),
            contour_thickness: Arc::new(AtomicF64::new(0.6)),
            contour_speed: Arc::new(AtomicF64::new(0.2)),

            smoothlife_resolution: Arc::new(RwLock::new(
                SmoothLifeSize::default(),
            )),
            smoothlife_speed: Arc::new(AtomicF64::new(2.0)),
            smoothlife_preset: Arc::new(RwLock::new(
                SmoothLifePreset::default(),
            )),

            spectrogram_resolution: Arc::new(RwLock::new(
                SpectrogramSize::default(),
            )),
            spectrogram_timing: Arc::new(AtomicF64::new(1.0)),
            spectrogram_view: Arc::new(RwLock::new(SpectrogramView::default())),

            reso_bank_scale: Arc::new(RwLock::new(Scale::default())),
            reso_bank_root_note: Arc::new(AtomicU8::new(69)), // A4 (440 Hz)
            reso_bank_spread: Arc::new(AtomicF64::new(0.5)),
            reso_bank_shift: Arc::new(AtomicF64::new(0.0)),
            reso_bank_inharm: Arc::new(AtomicF64::new(0.3)),
            reso_bank_pan: Arc::new(AtomicF64::new(1.0)),
            reso_bank_quantise: Arc::new(AtomicBool::new(true)),

            low_pass_cutoff_hz: smoother(4000.0),
            low_pass_q: smoother(BUTTERWORTH_Q),

            high_pass_cutoff_hz: smoother(500.0),
            high_pass_q: smoother(BUTTERWORTH_Q),

            pp_delay_time_ms: smoother(0.35),
            pp_delay_feedback: smoother(0.75),
            pp_delay_mix: smoother(0.5),
            pp_delay_tempo_sync: Arc::new(AtomicBool::new(false)),

            dist_amount: smoother(0.0),
            dist_type: Arc::new(RwLock::new(DistortionType::default())),
            // eq_params: EQParams::default(),
        }
    }
}

fn smoother(val: f64) -> Arc<SmootherAtomic<f64>> {
    Arc::new(
        SmootherAtomic::new(0.03, val, unsafe { SAMPLE_RATE })
            .with_smoothing_type(SmoothingType::Linear),
    )
}

#[derive(Clone, Copy, Debug, Default)]
pub enum GenerativeAlgo {
    #[default]
    /// A perlin noise contour-line generator.
    Contours,
    /// A [SmoothLife](https://arxiv.org/abs/1111.1567) simulation.
    SmoothLife,
}

impl Display for GenerativeAlgo {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Contours => write!(f, "Contours"),
            Self::SmoothLife => write!(f, "Smooth Life"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SmoothLifePreset {
    #[default]
    /// A simulation which quickly stabilises and smoothly scrolls like flowing fluid.
    Fluid,
    /// A simulation which stabilises into a turbulent, flowing state.
    Swirl,
}

impl Display for SmoothLifePreset {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Fluid => write!(f, "Fluid"),
            Self::Swirl => write!(f, "Swirl"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SpectrogramView {
    #[default]
    /// Draw both the pre- and post-FX spectrograms.
    PrePost,
    /// Only draw the pre-FX spectrogram.
    PreOnly,
    /// Only draw the post-FX spectrogram.
    PostOnly,
}

impl Display for SpectrogramView {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::PrePost => write!(f, "Pre/Post"),
            Self::PreOnly => write!(f, "Pre"),
            Self::PostOnly => write!(f, "Post"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum DistortionType {
    #[default]
    /// No distortion.
    None,
    /// A smooth soft clipping function.
    ///
    /// ([`smooth_soft_clip`](crate::dsp::distortion::waveshaper::smooth_soft_clip))
    Soft,
    /// More aggressive clipping function — not technically hard digital clipping! TODO
    Hard,
    /// A wrapping clipping algorithm. TODO
    Wrap,
    /// Downsampling distortion. TODO
    Downsample,
}

impl Display for DistortionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Soft => write!(f, "Soft"),
            Self::Hard => write!(f, "Hard"),
            Self::Wrap => write!(f, "Wrap"),
            Self::Downsample => write!(f, "Downsample"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SmoothLifeSize {
    S16,
    #[default]
    S32,
    S64,
    S128,
    S256,
    S512,
}

impl SmoothLifeSize {
    pub fn value(&self) -> usize {
        match self {
            Self::S16 => 16,
            Self::S32 => 32,
            Self::S64 => 64,
            Self::S128 => 128,
            Self::S256 => 256,
            Self::S512 => 512,
        }
    }
}

impl Display for SmoothLifeSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SpectrogramSize {
    S1024,
    #[default]
    S2048,
    S4096,
    S8192,
}

impl SpectrogramSize {
    pub fn value(&self) -> usize {
        match self {
            Self::S1024 => 1024,
            Self::S2048 => 2048,
            Self::S4096 => 4096,
            Self::S8192 => 8192,
        }
    }
}

impl Display for SpectrogramSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value())
    }
}
