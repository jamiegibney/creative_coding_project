use super::*;
use crate::app::*;
use std::rc::Rc;
use std::sync::Arc;

pub struct UIComponents {
    // ### SPECTRAL FILTER ###
    mask_algorithm: Menu<GenerativeAlgo>,
    /// float
    mask_scan_line_speed: TextSlider,
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
    smoothlife_resolution: Menu<SmoothLifeSize>,
    /// float
    smoothlife_speed: TextSlider,
    smoothlife_preset: Menu<SmoothLifePreset>,

    // ### SPECTROGRAMS ###
    /// usize
    spectrogram_resolution: Menu<SpectrogramSize>,
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
    reso_bank_randomise: Button,

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
                Menu::new(layout.mask_general.algorithm).with_callback(
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
                    layout.mask_general.scal_line_speed,
                )
                .with_output_range(0.01..=1.0)
                .with_value_chars(4)
                .with_suffix(" x")
                .with_default_value(0.1)
                .with_callback(move |_, value| scan_line_speed.sr(value))
            },
            mask_is_post_fx: {
                let mask_is_post_fx = Arc::clone(&params.mask_is_post_fx);
                Button::new(layout.mask_general.is_post_fx)
                    .with_callback(move |state| mask_is_post_fx.sr(state))
            },
            mask_uses_gpu: {
                let mask_uses_gpu = Arc::clone(&params.mask_uses_gpu);
                Button::new(layout.mask_general.uses_gpu)
                    .with_callback(move |state| mask_uses_gpu.sr(state))
            },
            mask_reset: Button::new(layout.mask_general.reset),

            contour_count: {
                let contour_count = Arc::clone(&params.contour_count);
                TextSlider::new(0.0, layout.contour.count)
                    .with_integer_rounding()
                    .with_default_value(contour_count.lr() as f64)
                    .with_callback(move |_, value| {
                        contour_count.sr(value as u32);
                    })
            },
            contour_thickness: {
                let contour_thickness = Arc::clone(&params.contour_thickness);
                TextSlider::new(0.0, layout.contour.thickness)
                    .with_default_value(contour_thickness.lr())
                    .with_callback(move |_, value| contour_thickness.sr(value))
            },
            contour_speed: {
                let contour_speed = Arc::clone(&params.contour_speed);
                TextSlider::new(0.0, layout.contour.speed)
                    .with_default_value(contour_speed.lr())
                    .with_callback(move |_, value| contour_speed.sr(value))
            },

            smoothlife_resolution: {
                let smoothlife_resolution =
                    Arc::clone(&params.smoothlife_resolution);
                Menu::new(layout.smooth_life.resolution).with_callback(
                    move |selected| {
                        let mut guard = smoothlife_resolution.write().unwrap();
                        *guard = selected;
                    },
                )
            },
            smoothlife_speed: {
                let smoothlife_speed = Arc::clone(&params.smoothlife_speed);
                TextSlider::new(0.0, layout.smooth_life.speed)
                    .with_callback(move |_, value| smoothlife_speed.sr(value))
            },
            smoothlife_preset: {
                let smoothlife_preset = Arc::clone(&params.smoothlife_preset);
                Menu::new(layout.smooth_life.preset).with_callback(
                    move |selected| {
                        let mut guard = smoothlife_preset.write().unwrap();
                        *guard = selected;
                    },
                )
            },

            spectrogram_resolution: {
                let spectrogram_resolution =
                    Arc::clone(&params.spectrogram_resolution);
                Menu::new(layout.spectrogram.resolution).with_callback(
                    move |selected| {
                        let mut guard = spectrogram_resolution.write().unwrap();
                        *guard = selected;
                    },
                )
            },
            spectrogram_timing: {
                let spectrogram_timing = Arc::clone(&params.spectrogram_timing);
                TextSlider::new(0.0, layout.spectrogram.timing)
                    .with_callback(move |_, value| spectrogram_timing.sr(value))
            },
            spectrogram_view: {
                let spectrogram_view = Arc::clone(&params.spectrogram_view);
                Menu::new(layout.spectrogram.view).with_callback(
                    move |selected| {
                        let mut guard = spectrogram_view.write().unwrap();
                        *guard = selected;
                    },
                )
            },

            reso_bank_scale: {
                let reso_bank_scale = Arc::clone(&params.reso_bank_scale);
                Menu::new(layout.reso_bank.scale).with_callback(
                    move |selected| {
                        let mut guard = reso_bank_scale.write().unwrap();
                        *guard = selected;
                    },
                )
            },
            reso_bank_root_note: {
                let reso_bank_root_note =
                    Arc::clone(&params.reso_bank_root_note);
                TextSlider::new(0.0, layout.reso_bank.root_note)
                    .with_default_value(reso_bank_root_note.lr() as f64)
                    .with_callback(move |_, value| {
                        reso_bank_root_note.sr(value as u8);
                    })
            },
            reso_bank_spread: {
                let reso_bank_spread = Arc::clone(&params.reso_bank_spread);
                TextSlider::new(0.0, layout.reso_bank.spread)
                    .with_default_value(reso_bank_spread.lr())
                    .with_callback(move |_, value| reso_bank_spread.sr(value))
            },
            reso_bank_shift: {
                let reso_bank_shift = Arc::clone(&params.reso_bank_shift);
                TextSlider::new(0.0, layout.reso_bank.shift)
                    .with_default_value(reso_bank_shift.lr())
                    .with_callback(move |_, value| reso_bank_shift.sr(value))
            },
            reso_bank_inharm: {
                let reso_bank_inharm = Arc::clone(&params.reso_bank_inharm);
                TextSlider::new(0.0, layout.reso_bank.inharm)
                    .with_default_value(reso_bank_inharm.lr())
                    .with_callback(move |_, value| reso_bank_inharm.sr(value))
            },
            reso_bank_pan: {
                let reso_bank_pan = Arc::clone(&params.reso_bank_pan);
                TextSlider::new(0.0, layout.reso_bank.pan)
                    .with_default_value(reso_bank_pan.lr())
                    .with_callback(move |_, value| reso_bank_pan.sr(value))
            },
            reso_bank_quantise: {
                let reso_bank_quantise = Arc::clone(&params.reso_bank_quantise);
                Button::new(layout.reso_bank.quantise)
                    .with_callback(move |state| reso_bank_quantise.sr(state))
            },
            reso_bank_randomise: Button::new(layout.reso_bank.randomise),

            low_pass_cutoff_hz: {
                let low_pass_cutoff_hz = Arc::clone(&params.low_pass_cutoff_hz);
                TextSlider::new(0.0, layout.low_pass.cutoff_hz)
                    .with_default_value(low_pass_cutoff_hz.current_value())
                    .with_callback(move |_, value| {
                        low_pass_cutoff_hz.set_target_value(value);
                    })
            },
            low_pass_q: {
                let low_pass_q = Arc::clone(&params.low_pass_q);
                TextSlider::new(0.0, layout.low_pass.q)
                    .with_default_value(low_pass_q.current_value())
                    .with_callback(move |_, value| {
                        low_pass_q.set_target_value(value);
                    })
            },

            high_pass_cutoff_hz: {
                let hp_cutoff = Arc::clone(&params.high_pass_cutoff_hz);
                TextSlider::new(0.0, layout.high_pass.cutoff_hz)
                    .with_default_value(hp_cutoff.current_value())
                    .with_callback(move |_, value| {
                        hp_cutoff.set_target_value(value);
                    })
            },
            high_pass_q: {
                let high_pass_q = Arc::clone(&params.high_pass_q);
                TextSlider::new(0.0, layout.high_pass.q)
                    .with_default_value(high_pass_q.current_value())
                    .with_callback(move |_, value| {
                        high_pass_q.set_target_value(value);
                    })
            },

            pp_delay_time_ms: {
                let pp_delay_time_ms = Arc::clone(&params.pp_delay_time_ms);
                TextSlider::new(0.0, layout.delay.time_ms)
                    .with_default_value(pp_delay_time_ms.current_value())
                    .with_callback(move |_, value| {
                        pp_delay_time_ms.set_target_value(value);
                    })
            },
            pp_delay_feedback: {
                let pp_delay_feedback = Arc::clone(&params.pp_delay_feedback);
                TextSlider::new(0.0, layout.delay.feedback)
                    .with_default_value(pp_delay_feedback.current_value())
                    .with_callback(move |_, value| {
                        pp_delay_feedback.set_target_value(value);
                    })
            },
            pp_delay_mix: {
                let pp_delay_mix = Arc::clone(&params.pp_delay_mix);
                TextSlider::new(0.0, layout.delay.mix)
                    .with_default_value(pp_delay_mix.current_value())
                    .with_callback(move |_, value| {
                        pp_delay_mix.set_target_value(value);
                    })
            },
            pp_delay_tempo_sync: {
                let pp_delay_tempo_sync =
                    Arc::clone(&params.pp_delay_tempo_sync);
                Button::new(layout.delay.tempo_sync)
                    .with_callback(move |state| pp_delay_tempo_sync.sr(state))
            },

            dist_amount: {
                let dist_amount = Arc::clone(&params.dist_amount);
                TextSlider::new(0.0, layout.distortion.amount)
                    .with_default_value(dist_amount.current_value())
                    .with_callback(move |_, value| {
                        dist_amount.set_target_value(value);
                    })
            },
            dist_type: {
                let dist_type = Arc::clone(&params.dist_type);
                Menu::new(layout.distortion.dist_type).with_callback(
                    move |selected| {
                        let mut guard = dist_type.write().unwrap();
                        *guard = selected;
                    },
                )
            },

            params,
        }
    }

    pub fn attach_reso_bank_randomise_callback<F: Fn(bool) + 'static>(
        &mut self,
        cb: F,
    ) {
        self.reso_bank_randomise.set_callback(cb);
    }

    pub fn attach_mask_reset_callback<F: Fn(bool) + 'static>(&mut self, cb: F) {
        self.mask_reset.set_callback(cb);
    }
}
