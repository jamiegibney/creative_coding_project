use crate::app::audio::audio_constructor::DEFAULT_SPECTRAL_BLOCK_SIZE;
use crate::app::musical::*;
use crate::dsp::BUTTERWORTH_Q;
/// All parameters controlled by the user interface.
use crate::prelude::*;

pub mod eq;
pub use eq::*;

#[derive(Clone, Debug)]
pub struct UIParams {
    // ### SPECTRAL FILTER ###
    /// The algorithm to use for the spectral mask.
    pub mask_algorithm: GenerativeAlgo,
    /// The speed of the spectral mask scan line.
    pub mask_scan_line_speed: f64,
    /// The resolution of the spectral filter.
    pub mask_resolution: usize,
    /// Whether the spectral filter is pre- or post-FX.
    pub mask_is_post_fx: bool,

    // CONTOURS ALGORITHMS
    /// The number of contours to draw.
    pub contour_count: u32,
    /// The thickness of each contour line.
    pub contour_thickness: f64,
    /// The speed of the contour animation.
    pub contour_speed: f64,

    // SMOOTHLIFE ALGORITHM
    /// The resolution of the smooth life simulation.
    pub smoothlife_resolution: usize,
    /// The speed of the smoothlife simulation.
    pub smoothlife_speed: f64,
    /// The state preset of the smoothlife simulation.
    pub smoothlife_preset: SmoothLifePreset,

    // ### SPECTROGRAMS ###
    /// The resolution of both spectrograms.
    pub spectrogram_resolution: usize,
    /// The timing of the spectrograms.
    pub spectrogram_timing: f64,
    /// Which spectrograms are drawn.
    pub spectrogram_view: SpectrogramView,

    // ### RESONATOR BANK ###
    /// The musical scale of the resonator bank.
    pub reso_bank_scale: Scale,
    /// The root note of the resonator bank.
    pub reso_bank_root_note: u8,
    /// The frequency spread (range) of each resonator.
    pub reso_bank_spread: f64,
    /// The frequency shift of each resonator.
    pub reso_bank_shift: f64,
    /// How much each resonator's pitch skews towards its original pitch.
    pub reso_bank_inharm: f64,
    /// How much panning may be applied to each resonator.
    pub reso_bank_pan: f64,
    /// Whether the resonators should quantise their pitch to a scale.
    pub reso_bank_quantise: bool,

    // ### POST EFFECTS ###

    // LOW-PASS
    /// The cutoff of the low-pass filter in Hz.
    pub low_pass_cutoff_hz: Smoother<f64>,
    /// The Q value of the low-pass filter.
    pub low_pass_q: Smoother<f64>,

    // HIGH-PASS
    /// The cutoff of the high-pass filter in Hz.
    pub high_pass_cutoff_hz: Smoother<f64>,
    /// The Q value of the high-pass filter.
    pub high_pass_q: Smoother<f64>,

    // PING-PONG DELAY
    /// The time between ping-pong delay taps in milliseconds.
    pub pp_delay_time_ms: Smoother<f64>,
    /// The ping-pong delay feedback.
    pub pp_delay_feedback: Smoother<f64>,
    /// The dry/wet mix for the ping-pong delay.
    pub pp_delay_mix: Smoother<f64>,
    /// Whether the ping-pong delay's timing should be tempo-synced.
    pub pp_delay_tempo_sync: bool,

    // DISTORTION
    pub dist_amount: Smoother<f64>,
    pub dist_type: DistortionType,

    // EQ
    /// The parameters for the three-band EQ.
    pub eq_params: EQParams,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum GenerativeAlgo {
    #[default]
    /// A perlin noise contour-line generator.
    Contours,
    /// A [SmoothLife](https://arxiv.org/abs/1111.1567) simulation.
    SmoothLife,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SmoothLifePreset {
    #[default]
    /// A simulation which quickly stabilises and smoothly scrolls like flowing fluid.
    Fluid,
    /// A simulation which stabilises into a turbulent, flowing state.
    Swirl,
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

#[derive(Clone, Copy, Debug, Default)]
pub enum DistortionType {
    #[default]
    /// No distortion.
    None,
    /// A smooth soft clipping function.
    ///
    /// ([`smooth_soft_clip`](crate::dsp::distortion::waveshaper::smooth_soft_clip))
    Soft { drive_factor: f64 },
    /// More aggressive clipping function — not technically hard digital clipping! TODO
    Hard { drive_factor: f64 },
    /// A wrapping clipping algorithm. TODO
    Wrap { drive_factor: f64 },
    /// Downsampling distortion. TODO
    Downsample,
}

impl Default for UIParams {
    fn default() -> Self {
        Self {
            mask_algorithm: GenerativeAlgo::default(),
            mask_scan_line_speed: 0.1,
            mask_resolution: 1 << 9, // 512
            mask_is_post_fx: false,

            contour_count: 8,
            contour_thickness: 0.6,
            contour_speed: 0.2,

            smoothlife_resolution: 1 << 6, // 64
            smoothlife_speed: 2.0,
            smoothlife_preset: SmoothLifePreset::default(),

            spectrogram_resolution: DEFAULT_SPECTRAL_BLOCK_SIZE,
            spectrogram_timing: 1.0,
            spectrogram_view: SpectrogramView::default(),

            reso_bank_scale: Scale::default(),
            reso_bank_root_note: 69, // A4 (440 Hz)
            reso_bank_spread: 0.5,
            reso_bank_shift: 0.0,
            reso_bank_inharm: 0.3,
            reso_bank_pan: 1.0,
            reso_bank_quantise: true,

            low_pass_cutoff_hz: smoother(4000.0),
            low_pass_q: smoother(BUTTERWORTH_Q),

            high_pass_cutoff_hz: smoother(500.0),
            high_pass_q: smoother(BUTTERWORTH_Q),

            pp_delay_time_ms: smoother(0.35),
            pp_delay_feedback: smoother(0.75),
            pp_delay_mix: smoother(0.5),
            pp_delay_tempo_sync: false,

            dist_amount: smoother(0.0),
            dist_type: DistortionType::default(),

            eq_params: EQParams::default(),
        }
    }
}

fn smoother(val: f64) -> Smoother<f64> {
    Smoother::new(0.03, val, unsafe { SAMPLE_RATE })
        .with_smoothing_type(SmoothingType::Linear)
}
