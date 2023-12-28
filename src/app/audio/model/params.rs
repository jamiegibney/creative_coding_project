use super::*;
use crate::prelude::*;
use atomic::Atomic;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicU8},
    Arc,
};

/// All audio-related parameters linked to the UI.
#[derive(Default)]
pub struct AudioParams {
    ///  The block size of the spectral filter.
    pub mask_resolution: Arc<Atomic<SpectralFilterSize>>,
    /// Whether the spectral filter is post-FX or not.
    pub mask_is_post_fx: Arc<AtomicBool>,

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
    pub reso_bank_quantise: Arc<AtomicBool>,
    /// The number of active resonators in the bank.
    pub reso_bank_resonator_count: Arc<AtomicU32>,

    /// The cutoff of the filter in Hz.
    pub low_filter_cutoff: Arc<SmootherAtomic<f64>>,
    /// The Q value of the cut filter.
    pub low_filter_q: Arc<SmootherAtomic<f64>>,
    /// The gain value of the shelf filter
    pub low_filter_gain_db: Arc<SmootherAtomic<f64>>,
    /// Whether the low filter is a shelf filter or not.
    pub low_filter_is_shelf: Arc<AtomicBool>,

    /// The cutoff of the high-pass filter in Hz.
    pub high_filter_cutoff: Arc<SmootherAtomic<f64>>,
    /// The Q value of the high-pass filter.
    pub high_filter_q: Arc<SmootherAtomic<f64>>,
    /// The gain value of the shelf filter.
    pub high_filter_gain_db: Arc<SmootherAtomic<f64>>,
    /// Whether the high filter is a shelf filter or not.
    pub high_filter_is_shelf: Arc<AtomicBool>,

    /// The time between delay taps in milliseconds.
    pub delay_time_ms: Arc<SmootherAtomic<f64>>,
    /// The delay feedback.
    pub delay_feedback: Arc<SmootherAtomic<f64>>,
    /// The dry/wet mix for the delay.
    pub delay_mix: Arc<SmootherAtomic<f64>>,
    /// Whether to use ping-pong delay or not.
    pub use_ping_pong: Arc<AtomicBool>,

    /// The "amount" of distortion, i.e. drive.
    pub dist_amount: Arc<SmootherAtomic<f64>>,
    /// The distortion algorithm.
    pub dist_type: Arc<Atomic<DistortionType>>,

    /// Compressor threshold.
    pub comp_thresh: Arc<SmootherAtomic<f64>>,
    /// Compressor ratio.
    pub comp_ratio: Arc<SmootherAtomic<f64>>,
    /// Compressor attack time in milliseconds.
    pub comp_attack_ms: Arc<SmootherAtomic<f64>>,
    /// Compressor release time in milliseconds.
    pub comp_release_ms: Arc<SmootherAtomic<f64>>,

    /// The device's master gain level.
    pub master_gain: Arc<SmootherAtomic<f64>>,
}
