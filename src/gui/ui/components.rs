use super::*;
use crate::{app::*, fonts::*};
use nannou::prelude::*;
use nannou::text::{Font, Justify, Layout};
use std::rc::Rc;
use std::sync::Arc;

pub struct UIComponents {
    // ### SPECTRAL FILTER ###
    mask_label: Label,
    mask_algorithm: Menu<GenerativeAlgo>,
    /// float
    mask_scan_line_speed: TextSlider,
    /// toggle
    mask_is_post_fx: Button,
    mask_resolution: Menu<SpectrogramSize>,
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
    spectrogram_label: Label,
    /// usize
    spectrogram_resolution: Menu<SpectrogramSize>,
    /// float
    spectrogram_timing: TextSlider,
    spectrogram_view: Menu<SpectrogramView>,

    // ### RESONATOR BANK ###
    reso_bank_label: Label,
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
    effects_label: Label,

    // ### Low pass
    low_pass_label: Label,
    /// f64 (smoother callback)
    low_pass_cutoff_hz: TextSlider,
    /// f64 (smoother callback)
    low_pass_q: TextSlider,

    // ### High pass
    high_pass_label: Label,
    /// f64 (smoother callback)
    high_pass_cutoff_hz: TextSlider,
    /// f64 (smoother callback)
    high_pass_q: TextSlider,

    // ### Ping-pong delay
    delay_label: Label,
    /// f64 (smoother callback)
    pp_delay_time_ms: TextSlider,
    /// f64 (smoother callback)
    pp_delay_feedback: TextSlider,
    /// f64 (smoother callback)
    pp_delay_mix: TextSlider,
    /// toggle
    pp_delay_tempo_sync: Button,

    // ### Distortion
    dist_label: Label,
    /// f64 (smoother callback)
    dist_amount: TextSlider,
    dist_type: Menu<DistortionType>,

    // ### Compression
    comp_label: Label,
    // f64 (slider callback)
    comp_thresh: TextSlider,
    // f64 (slider callback)
    comp_ratio: TextSlider,
    // f64 (slider callback)
    comp_attack: TextSlider,
    // f64 (slider callback)
    comp_release: TextSlider,

    // ### Master gain
    master_gain: TextSlider,
}

fn small_value_layout() -> Layout {
    Layout {
        font: Some(
            Font::from_bytes(MEDIUM_FONT_MONO_BYTES)
                .expect("failed to load font bytes"),
        ),
        font_size: 12,
        ..default_text_layout()
    }
}

fn small_label_layout() -> Layout {
    Layout {
        font: Some(
            Font::from_bytes(MEDIUM_FONT_MONO_BYTES)
                .expect("failed to load font bytes"),
        ),
        font_size: 12,
        ..default_text_layout()
    }
}

fn main_value_layout() -> Layout {
    Layout {
        font: Some(
            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                .expect("failed to load font bytes"),
        ),
        font_size: 18,
        ..default_text_layout()
    }
}

fn main_label_layout() -> Layout {
    Layout {
        font: Some(
            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                .expect("failed to load font bytes"),
        ),
        font_size: 14,
        ..default_text_layout()
    }
}

fn big_label_layout() -> Layout {
    Layout {
        font: Some(
            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                .expect("failed to load font bytes"),
        ),
        font_size: 19,
        ..default_text_layout()
    }
}

impl UIComponents {
    // this should probably be broken down into smaller modules...
    #[allow(clippy::too_many_lines, clippy::missing_panics_doc)]
    pub fn new(params: &UIParams) -> Self {
        let ui_layout = UILayout::default();

        Self {
            mask_label: Label::new(ui_layout.mask_general.label)
                .with_text("SPECTRAL FILTER")
                .with_text_layout(big_label_layout()),
            mask_algorithm: {
                let mask_algo = Arc::clone(&params.mask_algorithm);
                Menu::new(ui_layout.mask_general.algorithm)
                    .with_callback(move |variant| {
                        let mut guard = mask_algo.write().unwrap();
                        *guard = variant;
                    })
                    .with_label("Algorithm")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
            },
            mask_resolution: {
                Menu::new(ui_layout.mask_general.resolution)
                    // .with_callback(callback)
                    .with_label_layout(small_label_layout())
                    .with_item_text_layout(small_value_layout())
                    .with_selected_item_text_layout(small_value_layout())
            },
            mask_scan_line_speed: {
                let scan_line_speed = Arc::clone(&params.mask_scan_line_speed);
                TextSlider::new(
                    scan_line_speed.lr(),
                    ui_layout.mask_general.scan_line_speed,
                )
                .with_output_range(0.1..=10.0)
                .with_value_chars(4)
                .with_value_layout(small_value_layout())
                .with_sensitivity(0.005)
                .with_suffix(" x")
                .with_default_value(1.0)
                .with_callback(move |raw_val, _| {
                    scan_line_speed.sr(scale(raw_val, 0.01, 1.0));
                })
            },
            mask_is_post_fx: {
                let mask_is_post_fx = Arc::clone(&params.mask_is_post_fx);
                Button::new(ui_layout.mask_general.is_post_fx)
                    .with_callback(move |state| mask_is_post_fx.sr(state))
                    .with_enabled_layout(small_value_layout())
                    .with_disabled_layout(small_value_layout())
                    .with_enabled_text("Post FX")
                    .with_disabled_text("Pre FX")
            },
            // mask_uses_gpu: {
            //     let mask_uses_gpu = Arc::clone(&params.mask_uses_gpu);
            //     Button::new(ui_layout.mask_general.uses_gpu)
            //         .with_callback(move |state| mask_uses_gpu.sr(state))
            // },
            mask_reset: Button::new(ui_layout.mask_general.reset)
                .with_label_layout(main_value_layout())
                .with_label("Reset!")
                .toggleable(false),

            contour_count: {
                let contour_count = Arc::clone(&params.contour_count);
                TextSlider::new(0.0, ui_layout.contour.count)
                    .with_label("Count")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(1.0..=40.0)
                    .with_integer_rounding()
                    .with_default_value(contour_count.lr() as f64)
                    .with_callback(move |_, value| {
                        contour_count.sr(value as u32);
                    })
            },
            contour_thickness: {
                let contour_thickness = Arc::clone(&params.contour_thickness);
                TextSlider::new(0.0, ui_layout.contour.thickness)
                    .with_label("Thickness")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(0.1..=0.9)
                    .with_value_chars(4)
                    .with_default_value(contour_thickness.lr())
                    .with_callback(move |_, value| contour_thickness.sr(value))
            },
            contour_speed: {
                let contour_speed = Arc::clone(&params.contour_speed);
                TextSlider::new(0.0, ui_layout.contour.speed)
                    .with_label("Speed")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(0.01..=1.0)
                    .with_value_chars(4)
                    .with_suffix(" x")
                    .with_default_value(contour_speed.lr())
                    .with_callback(move |_, value| contour_speed.sr(value))
            },

            smoothlife_resolution: {
                let smoothlife_resolution =
                    Arc::clone(&params.smoothlife_resolution);
                Menu::new(ui_layout.smooth_life.resolution)
                    .with_callback(move |selected| {
                        let mut guard = smoothlife_resolution.write().unwrap();
                        *guard = selected;
                    })
                    .with_label("Resolution")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
            },
            smoothlife_speed: {
                let smoothlife_speed = Arc::clone(&params.smoothlife_speed);
                TextSlider::new(0.0, ui_layout.smooth_life.speed)
                    .with_label("Speed")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(0.1..=0.9)
                    .with_value_chars(4)
                    .with_suffix(" x")
                    .with_default_value(smoothlife_speed.lr())
                    .with_callback(move |_, value| smoothlife_speed.sr(value))
            },
            smoothlife_preset: {
                let smoothlife_preset = Arc::clone(&params.smoothlife_preset);
                Menu::new(ui_layout.smooth_life.preset)
                    .with_callback(move |selected| {
                        let mut guard = smoothlife_preset.write().unwrap();
                        *guard = selected;
                    })
                    .with_label("Preset")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
            },

            spectrogram_label: Label::new(ui_layout.spectrogram.label)
                .with_text("SPECTROGRAMS")
                .with_text_layout(big_label_layout()),
            spectrogram_resolution: {
                let spectrogram_resolution =
                    Arc::clone(&params.spectrogram_resolution);
                Menu::new(ui_layout.spectrogram.resolution)
                    .with_callback(move |selected| {
                        let mut guard = spectrogram_resolution.write().unwrap();
                        *guard = selected;
                    })
                    .with_label("Resolution")
                    .with_label_layout(main_label_layout())
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
            },
            spectrogram_timing: {
                let spectrogram_timing = Arc::clone(&params.spectrogram_timing);
                TextSlider::new(0.0, ui_layout.spectrogram.timing)
                    .with_label("Speed")
                    .with_suffix(" x")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_value_chars(3)
                    .with_output_range(0.2..=5.0)
                    .with_default_value(1.0)
                    .with_callback(move |_, value| spectrogram_timing.sr(value))
            },
            spectrogram_view: {
                let spectrogram_view = Arc::clone(&params.spectrogram_view);
                Menu::new(ui_layout.spectrogram.view)
                    .with_callback(move |selected| {
                        let mut guard = spectrogram_view.write().unwrap();
                        *guard = selected;
                    })
                    .with_label("View")
                    .with_label_layout(main_label_layout())
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
            },

            reso_bank_label: Label::new(ui_layout.reso_bank.label)
                .with_text("RESONATOR BANK")
                .with_text_layout(big_label_layout()),
            reso_bank_scale: {
                let reso_bank_scale = Arc::clone(&params.reso_bank_scale);
                Menu::new(ui_layout.reso_bank.scale).with_callback(
                    move |selected| {
                        let mut guard = reso_bank_scale.write().unwrap();
                        *guard = selected;
                    },
                )
            },
            reso_bank_root_note: {
                let reso_bank_root_note =
                    Arc::clone(&params.reso_bank_root_note);
                TextSlider::new(0.0, ui_layout.reso_bank.root_note)
                    .with_output_range(1.0..=127.0)
                    .with_default_value(reso_bank_root_note.lr() as f64)
                    .with_integer_rounding()
                    .with_callback(move |_, value| {
                        reso_bank_root_note.sr(value as u8);
                    })
            },
            reso_bank_spread: {
                let reso_bank_spread = Arc::clone(&params.reso_bank_spread);
                TextSlider::new(0.0, ui_layout.reso_bank.spread)
                    .with_label("Freq. spread")
                    .with_value_chars(4)
                    .with_default_value(reso_bank_spread.lr())
                    .with_callback(move |_, value| reso_bank_spread.sr(value))
            },
            reso_bank_shift: {
                let reso_bank_shift = Arc::clone(&params.reso_bank_shift);
                TextSlider::new(0.0, ui_layout.reso_bank.shift)
                    .with_label("Freq. shift")
                    .with_suffix(" st")
                    .with_value_chars(4)
                    .with_output_range(-24.0..=24.0)
                    .with_default_value(reso_bank_shift.lr())
                    .with_callback(move |_, value| reso_bank_shift.sr(value))
            },
            reso_bank_inharm: {
                let reso_bank_inharm = Arc::clone(&params.reso_bank_inharm);
                TextSlider::new(0.0, ui_layout.reso_bank.inharm)
                    .with_label("Inharmonic")
                    .with_value_chars(4)
                    .with_default_value(reso_bank_inharm.lr())
                    .with_callback(move |_, value| reso_bank_inharm.sr(value))
            },
            reso_bank_pan: {
                let reso_bank_pan = Arc::clone(&params.reso_bank_pan);
                TextSlider::new(0.0, ui_layout.reso_bank.pan)
                    .with_label("Panning")
                    .with_value_chars(4)
                    .with_default_value(reso_bank_pan.lr())
                    .with_callback(move |_, value| reso_bank_pan.sr(value))
            },
            reso_bank_quantise: {
                let reso_bank_quantise = Arc::clone(&params.reso_bank_quantise);
                Button::new(ui_layout.reso_bank.quantise)
                    .with_callback(move |state| reso_bank_quantise.sr(state))
                    .with_enabled_text("Quantise On")
                    .with_disabled_text("Quantise Off")
            },
            reso_bank_randomise: Button::new(ui_layout.reso_bank.randomise)
                .with_label("Reset!"),

            effects_label: Label::new(ui_layout.other.effects_label)
                .with_text("EFFECTS")
                .with_text_layout(big_label_layout()),

            low_pass_label: Label::new(ui_layout.low_filter.label)
                .with_text("LOW FILTER")
                // .with_text_color(Rgb::new(0.1, 0.3, 0.6))
                .with_text_layout(big_label_layout()),
            low_pass_cutoff_hz: {
                let low_pass_cutoff_hz = Arc::clone(&params.low_pass_cutoff_hz);
                TextSlider::new(0.0, ui_layout.low_filter.cutoff_hz)
                    .with_label("Cutoff")
                    .with_suffix(" Hz")
                    .with_output_range(10.0..=20000.0)
                    .with_default_value(low_pass_cutoff_hz.current_value())
                    .with_callback(move |_, value| {
                        low_pass_cutoff_hz.set_target_value(value);
                    })
                // .with_formatting_callback(|raw_value, output_value| todo!())
            },
            low_pass_q: {
                let low_pass_q = Arc::clone(&params.low_pass_q);
                TextSlider::new(0.0, ui_layout.low_filter.q)
                    .with_label("Q")
                    .with_value_chars(5)
                    .with_output_range(0.025..=10.0)
                    .with_default_value(low_pass_q.current_value())
                    .with_callback(move |_, value| {
                        low_pass_q.set_target_value(value);
                    })
            },

            high_pass_label: Label::new(ui_layout.high_filter.label)
                .with_text("HIGH FILTER")
                // .with_text_color(Rgb::new(0.6, 0.3, 0.1))
                .with_text_layout(big_label_layout()),
            high_pass_cutoff_hz: {
                let hp_cutoff = Arc::clone(&params.high_pass_cutoff_hz);
                TextSlider::new(0.0, ui_layout.high_filter.cutoff_hz)
                    .with_label("Cutoff")
                    .with_suffix(" Hz")
                    .with_output_range(10.0..=20000.0)
                    .with_default_value(hp_cutoff.current_value())
                    .with_callback(move |_, value| {
                        hp_cutoff.set_target_value(value);
                    })
                // .with_formatting_callback(|raw_value, output_value| todo!())
            },
            high_pass_q: {
                let high_pass_q = Arc::clone(&params.high_pass_q);
                TextSlider::new(0.0, ui_layout.high_filter.q)
                    .with_label("Q")
                    .with_value_chars(5)
                    .with_output_range(0.025..=10.0)
                    .with_default_value(high_pass_q.current_value())
                    .with_callback(move |_, value| {
                        high_pass_q.set_target_value(value);
                    })
            },

            delay_label: Label::new(ui_layout.delay.label)
                .with_text("DELAY")
                .with_text_layout(big_label_layout()),
            pp_delay_time_ms: {
                let pp_delay_time_ms = Arc::clone(&params.pp_delay_time_ms);
                TextSlider::new(0.0, ui_layout.delay.time_ms)
                    .with_label("Time")
                    .with_suffix(" ms")
                    .with_output_range(0.1..=1.0)
                    .with_default_value(pp_delay_time_ms.current_value())
                    .with_callback(move |_, value| {
                        pp_delay_time_ms.set_target_value(value);
                    })
            },
            pp_delay_feedback: {
                let pp_delay_feedback = Arc::clone(&params.pp_delay_feedback);
                TextSlider::new(0.0, ui_layout.delay.feedback)
                    .with_label("Feedback")
                    .with_suffix(" %")
                    .with_default_value(pp_delay_feedback.current_value())
                    .with_callback(move |_, value| {
                        pp_delay_feedback.set_target_value(value);
                    })
            },
            pp_delay_mix: {
                let pp_delay_mix = Arc::clone(&params.pp_delay_mix);
                TextSlider::new(0.0, ui_layout.delay.mix)
                    .with_label("Mix")
                    .with_default_value(pp_delay_mix.current_value())
                    .with_callback(move |_, value| {
                        pp_delay_mix.set_target_value(value);
                    })
                    .with_formatting_callback(|value, _| {
                        format!("{:.0} %", value * 100.0)
                    })
            },
            pp_delay_tempo_sync: {
                let pp_delay_tempo_sync =
                    Arc::clone(&params.pp_delay_tempo_sync);
                Button::new(ui_layout.delay.tempo_sync)
                    .with_label("Sync")
                    .with_callback(move |state| pp_delay_tempo_sync.sr(state))
            },

            dist_label: Label::new(ui_layout.distortion.label)
                .with_text("DISTORTION")
                .with_text_layout(big_label_layout()),
            dist_amount: {
                let dist_amount = Arc::clone(&params.dist_amount);
                TextSlider::new(0.0, ui_layout.distortion.amount)
                    .with_label("Amount")
                    .with_suffix(" %")
                    .with_default_value(dist_amount.current_value())
                    .with_callback(move |_, value| {
                        dist_amount.set_target_value(value);
                    })
            },
            dist_type: {
                let dist_type = Arc::clone(&params.dist_type);
                Menu::new(ui_layout.distortion.dist_type)
                    .with_callback(move |selected| {
                        let mut guard = dist_type.write().unwrap();
                        *guard = selected;
                    })
                    .with_label("Type")
            },

            comp_label: Label::new(ui_layout.compression.label)
                .with_text("COMPRESSION")
                .with_text_layout(big_label_layout()),
            comp_thresh: TextSlider::new(0.0, ui_layout.compression.threshold),
            comp_ratio: TextSlider::new(0.0, ui_layout.compression.ratio),
            comp_attack: TextSlider::new(0.0, ui_layout.compression.attack),
            comp_release: TextSlider::new(0.0, ui_layout.compression.release),

            master_gain: TextSlider::new(0.0, ui_layout.other.master_gain)
                .with_label("Master Gain")
                .with_label_layout(main_label_layout())
                .with_value_layout(main_value_layout())
                .with_output_range(-100.0..=12.0)
                .with_default_value(0.0)
                .with_formatting_callback(|raw_value, val| {
                    if val <= -99.9 {
                        return String::from("-inf dB");
                    }
                    if (0.0..=0.01).contains(&val) {
                        return String::from("0.0 dB");
                    }

                    let val_str = if val.is_sign_negative() {
                        format!("{val:.10}")
                    }
                    else {
                        format!("+{val:.10}")
                    };

                    let mut decimal_idx = val_str.find('.').unwrap();

                    // 5
                    let truncate_to = if decimal_idx == 4 {
                        6
                    }
                    else if decimal_idx > 5 {
                        decimal_idx
                    }
                    else {
                        5
                    };

                    let mut out = val_str[..truncate_to].to_string();
                    out.push_str(" dB");
                    out
                })
                .with_callback(move |_, output_value| {}),
        }
    }

    pub fn attach_reso_bank_randomise_callback<F: Fn(bool) + 'static>(
        mut self,
        cb: F,
    ) -> Self {
        self.reso_bank_randomise.set_callback(cb);
        self
    }

    pub fn attach_mask_reset_callback<F: Fn(bool) + 'static>(
        mut self,
        cb: F,
    ) -> Self {
        self.mask_reset.set_callback(cb);
        self
    }
}

impl UIDraw for UIComponents {
    fn update(&mut self, input_data: &InputData) {
        self.mask_algorithm.update(input_data);
        self.mask_scan_line_speed.update(input_data);
        self.mask_is_post_fx.update(input_data);
        self.mask_resolution.update(input_data);
        self.mask_reset.update(input_data);

        match self.mask_algorithm.output() {
            GenerativeAlgo::Contours => {
                self.contour_count.update(input_data);
                self.contour_thickness.update(input_data);
                self.contour_speed.update(input_data);
            }
            GenerativeAlgo::SmoothLife => {
                self.smoothlife_resolution.update(input_data);
                self.smoothlife_speed.update(input_data);
                self.smoothlife_preset.update(input_data);
            }
        }

        self.spectrogram_resolution.update(input_data);
        self.spectrogram_timing.update(input_data);
        self.spectrogram_view.update(input_data);

        self.reso_bank_scale.update(input_data);
        self.reso_bank_root_note.update(input_data);
        self.reso_bank_spread.update(input_data);
        self.reso_bank_shift.update(input_data);
        self.reso_bank_inharm.update(input_data);
        self.reso_bank_pan.update(input_data);
        self.reso_bank_quantise.update(input_data);
        self.reso_bank_randomise.update(input_data);

        self.low_pass_cutoff_hz.update(input_data);
        self.low_pass_q.update(input_data);

        self.high_pass_cutoff_hz.update(input_data);
        self.high_pass_q.update(input_data);

        self.dist_amount.update(input_data);
        self.dist_type.update(input_data);

        self.pp_delay_time_ms.update(input_data);
        self.pp_delay_feedback.update(input_data);
        self.pp_delay_mix.update(input_data);
        self.pp_delay_tempo_sync.update(input_data);

        self.comp_thresh.update(input_data);
        self.comp_ratio.update(input_data);
        self.comp_attack.update(input_data);
        self.comp_release.update(input_data);

        self.master_gain.update(input_data);
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        self.mask_label.draw(app, draw, frame);
        self.spectrogram_label.draw(app, draw, frame);
        self.reso_bank_label.draw(app, draw, frame);
        self.low_pass_label.draw(app, draw, frame);
        self.high_pass_label.draw(app, draw, frame);
        self.dist_label.draw(app, draw, frame);
        self.delay_label.draw(app, draw, frame);
        self.comp_label.draw(app, draw, frame);
        self.effects_label.draw(app, draw, frame);

        self.mask_scan_line_speed.draw(app, draw, frame);
        self.mask_is_post_fx.draw(app, draw, frame);
        self.mask_resolution.draw(app, draw, frame);
        self.mask_reset.draw(app, draw, frame);

        match self.mask_algorithm.output() {
            GenerativeAlgo::Contours => {
                self.contour_count.draw(app, draw, frame);
                self.contour_thickness.draw(app, draw, frame);
                self.contour_speed.draw(app, draw, frame);
            }
            GenerativeAlgo::SmoothLife => {
                self.smoothlife_resolution.draw(app, draw, frame);
                self.smoothlife_speed.draw(app, draw, frame);
                self.smoothlife_preset.draw(app, draw, frame);
            }
        }
        self.mask_algorithm.draw(app, draw, frame); // menu

        self.spectrogram_timing.draw(app, draw, frame);
        self.spectrogram_resolution.draw(app, draw, frame); // menu
        self.spectrogram_view.draw(app, draw, frame); // menu

        self.reso_bank_root_note.draw(app, draw, frame);
        self.reso_bank_spread.draw(app, draw, frame);
        self.reso_bank_shift.draw(app, draw, frame);
        self.reso_bank_inharm.draw(app, draw, frame);
        self.reso_bank_pan.draw(app, draw, frame);
        self.reso_bank_quantise.draw(app, draw, frame);
        self.reso_bank_randomise.draw(app, draw, frame);
        self.reso_bank_scale.draw(app, draw, frame); // menu

        self.low_pass_cutoff_hz.draw(app, draw, frame);
        self.low_pass_q.draw(app, draw, frame);

        self.high_pass_cutoff_hz.draw(app, draw, frame);
        self.high_pass_q.draw(app, draw, frame);

        self.dist_amount.draw(app, draw, frame);
        self.dist_type.draw(app, draw, frame); // menu

        self.pp_delay_time_ms.draw(app, draw, frame);
        self.pp_delay_feedback.draw(app, draw, frame);
        self.pp_delay_mix.draw(app, draw, frame);
        self.pp_delay_tempo_sync.draw(app, draw, frame);

        self.comp_thresh.draw(app, draw, frame);
        self.comp_ratio.draw(app, draw, frame);
        self.comp_attack.draw(app, draw, frame);
        self.comp_release.draw(app, draw, frame);

        self.master_gain.draw(app, draw, frame);
    }

    fn rect(&self) -> &nannou::prelude::Rect {
        unimplemented!("UIComponents does not have a bounding rect!")
    }
}
