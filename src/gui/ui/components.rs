use super::*;
use crate::app::*;
use std::rc::Rc;
use std::sync::Arc;

pub struct UIComponents {
    // ### SPECTRAL FILTER ###
    mask_algorithm: Menu<GenerativeAlgo>,
    /// float
    mask_scan_line_speed: TextSlider,
    /// usize
    mask_resolution: TextSlider,
    /// toggle
    mask_is_post_fx: Button,
    /// toggle
    mask_uses_gpu: Button,
    /// trigger
    mask_reset: Button,

    // ### Contour algorithm
    /// u32
    contour_count: TextSlider,
    /// float
    contour_thickness: TextSlider,
    /// float
    contour_speed: TextSlider,

    // ### Smooth life algorithm
    /// usize
    smoothlife_resolution: TextSlider,
    /// float
    smoothlife_speed: TextSlider,
    smoothlife_preset: Menu<SmoothLifePreset>,

    // ### SPECTROGRAMS ###
    /// usize
    spectrogram_resolution: TextSlider,
    /// float
    spectrogram_timing: TextSlider,
    spectrogram_view: Menu<SpectrogramView>,

    // ### RESONATOR BANK ###
    reso_bank_scale: Menu<Scale>,
    /// u8
    reso_bank_root_note: TextSlider,
    /// f64
    reso_bank_spread: TextSlider,
    /// f64
    reso_bank_shift: TextSlider,
    /// f64
    reso_bank_inharm: TextSlider,
    /// f64
    reso_bank_pan: TextSlider,
    /// toggle
    reso_bank_quantise: Button,
    /// trigger
    reso_bank_randomis: Button,

    // ### POST EFFECTS ###

    // ### Low pass
    /// f64 (smoother callback)
    low_pass_cutoff_hz: TextSlider,
    /// f64 (smoother callback)
    low_pass_q: TextSlider,

    // ### High pass
    /// f64 (smoother callback)
    high_pass_cutoff_hz: TextSlider,
    /// f64 (smoother callback)
    high_pass_q: TextSlider,

    // ### Ping-pong delay
    /// f64 (smoother callback)
    pp_delay_time_ms: TextSlider,
    /// f64 (smoother callback)
    pp_delay_feedback: TextSlider,
    /// f64 (smoother callback)
    pp_delay_mix: TextSlider,
    /// toggle
    pp_delay_tempo_sync: Button,

    // ### Distortion
    /// f64 (smoother callback)
    dist_amount: TextSlider,
    dist_type: Menu<DistortionType>,

    params: Rc<UIParams>,
    // params: Arc<Mutex<UIParams>>,
}

impl UIComponents {
    // this should probably be broken down into smaller modules...
    #[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
    pub fn new(params: Rc<UIParams>) -> Self {
        let layout = UILayout::default();

        Self {
            mask_algorithm: {
                let mask_algo = Arc::clone(&params.mask_algorithm);
                Menu::new(layout.mask_general.mask_algorithm).with_callback(
                    move |variant| {
                        let mut guard = mask_algo.write().unwrap();
                        *guard = variant;
                    },
                )
            },
            mask_scan_line_speed: {
                let scan_line_speed = Arc::clone(&params.mask_scan_line_speed);
                TextSlider::new(
                    scan_line_speed.lr(),
                    layout.mask_general.mask_scan_line_speed,
                )
                .with_output_range(0.01..=1.0)
                .with_value_chars(4)
                .with_suffix(" x")
                .with_default_value(0.1)
                .with_callback(move |_, value| scan_line_speed.sr(value))
            },
            mask_resolution: todo!(),
            mask_is_post_fx: todo!(),
            mask_uses_gpu: todo!(),
            mask_reset: todo!(),

            contour_count: todo!(),
            contour_thickness: todo!(),
            contour_speed: todo!(),

            smoothlife_resolution: todo!(),
            smoothlife_speed: todo!(),
            smoothlife_preset: todo!(),

            spectrogram_resolution: todo!(),
            spectrogram_timing: todo!(),
            spectrogram_view: todo!(),

            reso_bank_scale: todo!(),
            reso_bank_root_note: todo!(),
            reso_bank_spread: todo!(),
            reso_bank_shift: todo!(),
            reso_bank_inharm: todo!(),
            reso_bank_pan: todo!(),
            reso_bank_quantise: todo!(),
            reso_bank_randomis: todo!(),

            low_pass_cutoff_hz: todo!(),
            low_pass_q: todo!(),

            high_pass_cutoff_hz: todo!(),
            high_pass_q: todo!(),

            pp_delay_time_ms: todo!(),
            pp_delay_feedback: todo!(),
            pp_delay_mix: todo!(),
            pp_delay_tempo_sync: todo!(),

            dist_amount: todo!(),
            dist_type: todo!(),

            params,
        }
    }
}
