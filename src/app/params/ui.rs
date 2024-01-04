//! UI parameters and their default values.

use super::*;
use crate::app::audio::audio_constructor::DEFAULT_SPECTRAL_BLOCK_SIZE;
use crate::app::musical::*;
use crate::dsp::BUTTERWORTH_Q;
use crate::prelude::*;
use atomic::Atomic;

use atomic_float::AtomicF64;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize},
    Arc,
};

/// All parameters controlled by the user interface.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct UIParams {
    // ### SPECTRAL FILTER ###
    /// The algorithm to use for the spectral filter.
    pub mask_algorithm: Arc<Atomic<GenerativeAlgo>>,
    /// The speed of the spectral filter scan line.
    pub mask_scan_line_speed: Arc<AtomicF64>,
    /// Whether the spectral filter is pre- or post-FX.
    pub mask_is_post_fx: Arc<AtomicBool>,

    pub mask_mix: Arc<AtomicF64>,
    /// The block size of the spectral filter.
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

    /// The number of Voronoi cells.
    pub voronoi_cell_count: Arc<AtomicU32>,
    /// The speed of the Voronoi flow field.
    pub voronoi_cell_speed: Arc<AtomicF64>,
    /// The weight of the Voronoi borders and isolines.
    pub voronoi_border_weight: Arc<AtomicF64>,

    // ### RESONATOR BANK ###
    /// The musical scale of the resonator bank.
    pub reso_bank_scale: Arc<Atomic<Scale>>,
    /// The root note of the resonator bank.
    pub reso_bank_root_note: Arc<AtomicU8>,
    /// The frequency spread (range) of each resonator.
    pub reso_bank_spread: Arc<SmootherAtomic<f64>>,
    /// The frequency shift of each resonator.
    pub reso_bank_shift: Arc<SmootherAtomic<f64>>,
    /// How much each resonator's pitch skews towards its original pitch.
    pub reso_bank_inharm: Arc<SmootherAtomic<f64>>,
    /// How much panning may be applied to each resonator.
    pub reso_bank_pan: Arc<SmootherAtomic<f64>>,
    /// Whether the resonators should quantise their pitch to a scale.
    pub reso_bank_quantize: Arc<AtomicBool>,

    /// The number of active resonators in the bank.
    pub reso_bank_resonator_count: Arc<AtomicU32>,
    pub reso_bank_cell_count: Arc<AtomicU32>,
    // pub reso_bank_cell_count_sender: ,
    pub reso_bank_cell_jitter: Arc<AtomicF64>,
    /// The friction applied to each point in the vector field.
    pub reso_bank_field_friction: Arc<AtomicF64>,

    /// The dry/wet mix of the resonator bank.
    pub reso_bank_mix: Arc<SmootherAtomic<f64>>,
    /// The exciter oscillator.
    pub exciter_osc: Arc<Atomic<ExciterOscillator>>,

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

    // STEREO DELAY
    /// The time between delay taps in milliseconds.
    pub delay_time_ms: Arc<AtomicF64>,
    /// The delay feedback.
    pub delay_feedback: Arc<SmootherAtomic<f64>>,
    /// The dry/wet mix for the delay.
    pub delay_mix: Arc<SmootherAtomic<f64>>,
    /// Whether to use ping-pong delay or not.
    pub use_ping_pong: Arc<AtomicBool>,

    // DISTORTION
    /// The "amount" of distortion.
    pub dist_amount: Arc<SmootherAtomic<f64>>,
    /// The distortion algorithm.
    pub dist_type: Arc<Atomic<DistortionType>>,

    // COMPRESSION
    /// Compression threshold in decibels.
    pub comp_thresh: Arc<SmootherAtomic<f64>>,
    /// Compression ratio.
    pub comp_ratio: Arc<SmootherAtomic<f64>>,
    /// Compression attack time in milliseconds.
    pub comp_attack_ms: Arc<SmootherAtomic<f64>>,
    /// Compression release time in milliseconds.
    pub comp_release_ms: Arc<SmootherAtomic<f64>>,

    pub pre_fx_gain: Arc<SmootherAtomic<f64>>,
    /// Master gain level in decibels.
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
            mask_mix: Arc::new(AtomicF64::new(1.0)),
            mask_resolution: Arc::new(Atomic::new(
                SpectralFilterSize::default(),
            )),

            contour_count: Arc::new(AtomicU32::new(8)),
            contour_thickness: Arc::new(AtomicF64::new(0.6)),
            contour_speed: Arc::new(AtomicF64::new(0.2)),

            smoothlife_resolution: Arc::new(Atomic::new(
                SmoothLifeSize::default(),
            )),
            smoothlife_speed: Arc::new(AtomicF64::new(0.2)),
            smoothlife_preset: Arc::new(Atomic::new(
                SmoothLifePreset::default(),
            )),

            voronoi_cell_count: Arc::new(AtomicU32::new(10)),
            voronoi_cell_speed: Arc::new(AtomicF64::new(0.3)),
            voronoi_border_weight: Arc::new(AtomicF64::new(0.65)),

            spectrogram_resolution: Arc::new(Atomic::new(
                SpectrogramSize::default(),
            )),
            spectrogram_timing: Arc::new(AtomicF64::new(1.0)),
            spectrogram_view: Arc::new(Atomic::new(SpectrogramView::default())),

            reso_bank_scale: Arc::new(Atomic::new(Scale::default())),
            reso_bank_root_note: Arc::new(AtomicU8::new(60)), // C4
            reso_bank_spread: smoother(0.5),
            reso_bank_shift: smoother(0.0),
            reso_bank_inharm: smoother(0.3),
            reso_bank_pan: smoother(1.0),
            reso_bank_quantize: Arc::new(AtomicBool::new(true)),

            reso_bank_resonator_count: Arc::new(AtomicU32::new(8)),
            reso_bank_cell_count: Arc::new(AtomicU32::new(12)),
            reso_bank_cell_jitter: Arc::new(AtomicF64::new(0.1)),
            reso_bank_field_friction: Arc::new(AtomicF64::new(0.5)),

            reso_bank_mix: smoother(1.0),
            exciter_osc: Arc::new(Atomic::new(ExciterOscillator::default())),

            low_filter_cutoff: smoother(500.0),
            low_filter_q: smoother(BUTTERWORTH_Q),
            low_filter_gain_db: smoother(0.0),
            low_filter_is_shelf: Arc::new(AtomicBool::new(false)),

            high_filter_cutoff: smoother(2000.0),
            high_filter_q: smoother(BUTTERWORTH_Q),
            high_filter_gain_db: smoother(0.0),
            high_filter_is_shelf: Arc::new(AtomicBool::new(true)),

            delay_time_ms: Arc::new(AtomicF64::new(250.0)),
            delay_feedback: smoother(0.75),
            delay_mix: smoother(0.0),
            use_ping_pong: Arc::new(AtomicBool::new(true)),

            dist_amount: smoother(0.0),
            dist_type: Arc::new(Atomic::new(DistortionType::default())),

            comp_thresh: smoother(-12.0),
            comp_ratio: smoother(10.0),
            comp_attack_ms: smoother(80.0),
            comp_release_ms: smoother(200.0),

            pre_fx_gain: smoother(0.0),
            master_gain: smoother(1.0),
            // eq_params: EQParams::default(),
        }
    }
}

fn smoother(val: f64) -> Arc<SmootherAtomic<f64>> {
    Arc::new(
        SmootherAtomic::new(70.0, val, unsafe { OVERSAMPLED_SAMPLE_RATE })
            .with_smoothing_type(SmoothingType::Linear),
    )
}
