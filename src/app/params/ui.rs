use crate::app::audio::audio_constructor::DEFAULT_SPECTRAL_BLOCK_SIZE;
use crate::app::musical::*;
use crate::dsp::BUTTERWORTH_Q;
use crate::prelude::*;
use atomic::Atomic;
use bytemuck::NoUninit;
use std::fmt::{Display, Formatter, Result};

use atomic_float::AtomicF64;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize},
    Arc,
};

pub mod eq;
pub use eq::*;

/// All parameters controlled by the user interface.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct UIParams {
    // ### SPECTRAL FILTER ###
    /// The algorithm to use for the spectral mask.
    pub mask_algorithm: Arc<Atomic<GenerativeAlgo>>,
    /// The speed of the spectral mask scan line.
    pub mask_scan_line_speed: Arc<AtomicF64>,
    /// Whether the spectral filter is pre- or post-FX.
    pub mask_is_post_fx: Arc<AtomicBool>,
    /// Whether to use the GPU to compute the generative algorithms.
    pub mask_resolution: Arc<Atomic<SpectralFilterSize>>,

    // CONTOURS ALGORITHMS
    /// The number of contours to draw.
    pub contour_count: Arc<AtomicU32>,
    /// The thickness of each contour line.
    pub contour_thickness: Arc<AtomicF64>,
    /// The speed of the contour animation.
    pub contour_speed: Arc<AtomicF64>,

    // SMOOTHLIFE ALGORITHM
    /// The resolution of the smooth life simulation.
    pub smoothlife_resolution: Arc<Atomic<SmoothLifeSize>>,
    /// The speed of the smoothlife simulation.
    pub smoothlife_speed: Arc<AtomicF64>,
    /// The state preset of the smoothlife simulation.
    pub smoothlife_preset: Arc<Atomic<SmoothLifePreset>>,

    // ### SPECTROGRAMS ###
    /// The resolution of both spectrograms.
    pub spectrogram_resolution: Arc<Atomic<SpectrogramSize>>,
    /// The timing of the spectrograms.
    pub spectrogram_timing: Arc<AtomicF64>,
    /// Which spectrograms are drawn.
    pub spectrogram_view: Arc<Atomic<SpectrogramView>>,

    // ### RESONATOR BANK ###
    /// The musical scale of the resonator bank.
    pub reso_bank_scale: Arc<Atomic<Scale>>,
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

    /// The number of active resonators in the bank.
    pub reso_bank_resonator_count: Arc<AtomicU32>,
    /// The number of active Voronoi cells.
    pub reso_bank_cell_count: Arc<AtomicU32>,
    /// The amount of jitter applied to Voronoi cells.
    pub reso_bank_cell_jitter: Arc<AtomicF64>,
    /// How uniformly Voronoi cells are distributed — higher values
    /// correspond to a less even distribution.
    pub reso_bank_cell_scatter: Arc<AtomicF64>,

    // ### POST EFFECTS ###

    // LOW-PASS
    /// The cutoff of the filter in Hz.
    pub low_filter_cutoff: Arc<SmootherAtomic<f64>>,
    /// The Q value of the cut filter.
    pub low_filter_q: Arc<SmootherAtomic<f64>>,
    /// The gain value of the shelf filter
    pub low_filter_gain_db: Arc<SmootherAtomic<f64>>,
    /// Whether the low filter is a shelf filter or not.
    pub low_filter_is_shelf: Arc<AtomicBool>,

    // HIGH-PASS
    /// The cutoff of the high-pass filter in Hz.
    pub high_filter_cutoff: Arc<SmootherAtomic<f64>>,
    /// The Q value of the high-pass filter.
    pub high_filter_q: Arc<SmootherAtomic<f64>>,
    /// The gain value of the shelf filter.
    pub high_filter_gain_db: Arc<SmootherAtomic<f64>>,
    /// Whether the high filter is a shelf filter or not.
    pub high_filter_is_shelf: Arc<AtomicBool>,

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
    pub dist_type: Arc<Atomic<DistortionType>>,

    // COMPRESSION
    pub comp_thresh: Arc<SmootherAtomic<f64>>,
    pub comp_ratio: Arc<SmootherAtomic<f64>>,
    pub comp_attack_ms: Arc<SmootherAtomic<f64>>,
    pub comp_release_ms: Arc<SmootherAtomic<f64>>,

    pub master_gain: Arc<SmootherAtomic<f64>>,
    // // EQ
    // /// The parameters for the three-band EQ.
    // pub eq_params: EQParams,
}

impl Default for UIParams {
    fn default() -> Self {
        Self {
            mask_algorithm: Arc::new(Atomic::new(GenerativeAlgo::default())),
            mask_scan_line_speed: Arc::new(AtomicF64::new(0.1)),
            mask_is_post_fx: Arc::new(AtomicBool::new(false)),
            mask_resolution: Arc::new(Atomic::new(
                SpectralFilterSize::default(),
            )),

            contour_count: Arc::new(AtomicU32::new(8)),
            contour_thickness: Arc::new(AtomicF64::new(0.6)),
            contour_speed: Arc::new(AtomicF64::new(0.2)),

            smoothlife_resolution: Arc::new(Atomic::new(
                SmoothLifeSize::default(),
            )),
            smoothlife_speed: Arc::new(AtomicF64::new(2.0)),
            smoothlife_preset: Arc::new(Atomic::new(
                SmoothLifePreset::default(),
            )),

            spectrogram_resolution: Arc::new(Atomic::new(
                SpectrogramSize::default(),
            )),
            spectrogram_timing: Arc::new(AtomicF64::new(1.0)),
            spectrogram_view: Arc::new(Atomic::new(SpectrogramView::default())),

            reso_bank_scale: Arc::new(Atomic::new(Scale::default())),
            reso_bank_root_note: Arc::new(AtomicU8::new(69)), // A4 (440 Hz)
            reso_bank_spread: Arc::new(AtomicF64::new(0.5)),
            reso_bank_shift: Arc::new(AtomicF64::new(0.0)),
            reso_bank_inharm: Arc::new(AtomicF64::new(0.3)),
            reso_bank_pan: Arc::new(AtomicF64::new(1.0)),
            reso_bank_quantise: Arc::new(AtomicBool::new(true)),

            reso_bank_resonator_count: Arc::new(AtomicU32::new(8)),
            reso_bank_cell_count: Arc::new(AtomicU32::new(12)),
            reso_bank_cell_jitter: Arc::new(AtomicF64::new(0.1)),
            reso_bank_cell_scatter: Arc::new(AtomicF64::new(0.5)),

            low_filter_cutoff: smoother(4000.0),
            low_filter_q: smoother(BUTTERWORTH_Q),
            low_filter_gain_db: smoother(0.0),
            low_filter_is_shelf: Arc::new(AtomicBool::new(true)),

            high_filter_cutoff: smoother(500.0),
            high_filter_q: smoother(BUTTERWORTH_Q),
            high_filter_gain_db: smoother(0.0),
            high_filter_is_shelf: Arc::new(AtomicBool::new(true)),

            pp_delay_time_ms: smoother(0.35),
            pp_delay_feedback: smoother(0.75),
            pp_delay_mix: smoother(0.5),
            pp_delay_tempo_sync: Arc::new(AtomicBool::new(false)),

            dist_amount: smoother(0.0),
            dist_type: Arc::new(Atomic::new(DistortionType::default())),

            comp_thresh: smoother(0.0),
            comp_ratio: smoother(1.0),
            comp_attack_ms: smoother(30.0),
            comp_release_ms: smoother(100.0),

            master_gain: smoother(1.0),
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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

unsafe impl NoUninit for GenerativeAlgo {}

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

unsafe impl NoUninit for SmoothLifePreset {}

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

unsafe impl NoUninit for SpectrogramView {}

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

unsafe impl NoUninit for DistortionType {}

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

unsafe impl NoUninit for SmoothLifeSize {}

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

unsafe impl NoUninit for SpectrogramSize {}

#[derive(Clone, Copy, Debug, Default)]
pub enum SpectralFilterSize {
    S64,
    S128,
    S256,
    S512,
    #[default]
    S1024,
    S2048,
    S4096,
    S8192,
    S16384,
}

impl SpectralFilterSize {
    pub fn value(&self) -> usize {
        match self {
            Self::S64 => 64,
            Self::S128 => 128,
            Self::S256 => 256,
            Self::S512 => 512,
            Self::S1024 => 1024,
            Self::S2048 => 2048,
            Self::S4096 => 4096,
            Self::S8192 => 8192,
            Self::S16384 => 16384,
        }
    }
}

impl Display for SpectralFilterSize {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value())
    }
}

unsafe impl NoUninit for SpectralFilterSize {}
