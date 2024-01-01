use super::*;
use crate::app::audio::AudioMessageSenders;
use crate::dsp::{ResonatorBankParams, SpectralMask};
use crate::generative::{ContoursGPU, SmoothLifeGPU};
use crate::{app::*, fonts::*};
use atomic::Atomic;
use nannou::prelude::*;
use nannou::text::{Font, Justify, Layout};
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use triple_buffer::Input;

pub struct UIComponents {
    // ### SPECTRAL FILTER ###
    mask_label: Label,
    mask_algorithm: Menu<GenerativeAlgo>,
    /// float
    mask_scan_line_speed: TextSlider,
    /// toggle
    mask_is_post_fx: Button,
    mask_mix: TextSlider,
    pub mask_resolution: Menu<SpectralFilterSize>,
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

    voronoi_cell_count: TextSlider,
    voronoi_cell_speed: TextSlider,
    voronoi_border_weight: TextSlider,

    // ### SPECTROGRAMS ###
    spectrogram_label: Label,
    /// usize
    spectrogram_resolution: Menu<SpectrogramSize>,
    /// float
    spectrogram_timing: TextSlider,
    spectrogram_view: Menu<SpectrogramView>,

    // ### RESONATOR BANK ###
    reso_bank_label: Label,
    pub reso_bank_scale: Menu<Scale>,
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
    reso_bank_quantize: Button,
    /// trigger
    reso_bank_randomize: Button,
    /// trigger
    reso_bank_push: Button,

    // f64
    reso_bank_mix: TextSlider,
    pub exciter_osc: Menu<ExciterOscillator>,

    // f64
    reso_bank_cell_jitter: TextSlider,
    // u32
    reso_bank_cell_count: TextSlider,
    // u32
    reso_bank_resonator_count: TextSlider,
    // f64
    reso_bank_field_friction: TextSlider,

    // ### POST EFFECTS ###
    effects_label: Label,

    // ### Low filter
    low_filter_label: Label,
    /// toggle
    pub low_filter_type: Button,
    /// f64 (smoother callback)
    pub low_filter_cutoff: TextSlider,
    /// f64 (smoother callback)
    pub low_filter_q: TextSlider,
    // f64 (smoother callback)
    pub low_filter_gain: TextSlider,

    // ### High filter
    high_filter_label: Label,
    /// toggle
    pub high_filter_type: Button,
    /// f64 (smoother callback)
    pub high_filter_cutoff: TextSlider,
    /// f64 (smoother callback)
    pub high_filter_q: TextSlider,
    // f64 (smoother callback)
    pub high_filter_gain: TextSlider,

    // ### Stereo delay
    delay_label: Label,
    /// f64 (smoother callback)
    delay_time_ms: TextSlider,
    /// f64 (smoother callback)
    delay_feedback: TextSlider,
    /// f64 (smoother callback)
    delay_mix: TextSlider,
    /// toggle
    delay_is_ping_pong: Button,

    // ### Distortion
    dist_label: Label,
    /// f64 (smoother callback)
    dist_amount: TextSlider,
    dist_type: Menu<DistortionType>,

    // ### Compression
    comp_label: Label,
    /// f64 (slider callback)
    comp_thresh: TextSlider,
    /// f64 (slider callback)
    comp_ratio: TextSlider,
    /// f64 (slider callback)
    comp_attack: TextSlider,
    /// f64 (slider callback)
    comp_release: TextSlider,

    // ### Pre-FX gain
    pre_fx_gain: TextSlider,
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
                    .with_callback(move |selected| {
                        mask_algo.sr(selected);
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
                let mask_resolution = Arc::clone(&params.mask_resolution);
                Menu::new(ui_layout.mask_general.resolution)
                    .with_callback(move |selected| {
                        mask_resolution.sr(selected);
                    })
                    .with_label_layout(small_label_layout())
                    .with_item_text_layout(small_value_layout())
                    .with_selected_item_text_layout(Layout {
                        font: Some(
                            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                                .expect("failed to load font bytes"),
                        ),
                        ..small_value_layout()
                    })
            },
            mask_scan_line_speed: {
                let scan_line_speed = Arc::clone(&params.mask_scan_line_speed);
                TextSlider::new(
                    scan_line_speed.lr(),
                    ui_layout.mask_general.scan_line_speed,
                )
                .with_output_range(-10.0..=10.0)
                .with_value_chars(4)
                .with_value_layout(Layout {
                    font: Some(
                        Font::from_bytes(BOLD_FONT_MONO_BYTES)
                            .expect("failed to load font bytes"),
                    ),
                    ..small_value_layout()
                })
                .with_sensitivity(0.003)
                .with_suffix(" x")
                .with_default_value(1.0)
                .with_callback(move |raw_val, _| {
                    scan_line_speed.sr(scale(raw_val, -1.0, 1.0));
                })
            },
            mask_is_post_fx: {
                let mask_is_post_fx = Arc::clone(&params.mask_is_post_fx);
                Button::new(ui_layout.mask_general.is_post_fx)
                    .with_callback(move |state| mask_is_post_fx.sr(state))
                    .with_enabled_layout(Layout {
                        font: Some(
                            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                                .expect("failed to load font bytes"),
                        ),
                        ..small_value_layout()
                    })
                    .with_disabled_layout(Layout {
                        font: Some(
                            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                                .expect("failed to load font bytes"),
                        ),
                        ..small_value_layout()
                    })
                    .with_enabled_text("Post FX")
                    .with_disabled_text("Pre FX")
            },
            mask_mix: {
                let mask_mix = Arc::clone(&params.mask_mix);
                TextSlider::new(1.0, ui_layout.mask_general.mix)
                    .with_value_layout(Layout {
                        font: Some(
                            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                                .expect("failed to load font bytes"),
                        ),
                        ..small_value_layout()
                    })
                    .with_default_value(1.0)
                    .with_callback(move |_, val| {
                        mask_mix.sr(val);
                    })
                    .with_formatting_callback(|_, val| {
                        format!("{:.0} %", val * 100.0)
                    })
            },
            mask_reset: Button::new(ui_layout.mask_general.reset)
                .with_label_layout(main_value_layout())
                .with_label("Regenerate")
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
                    .with_output_range(-1.00..=1.0)
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
                        smoothlife_resolution.sr(selected);
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
                    .with_output_range(1.0..=10.0)
                    .with_value_chars(4)
                    .with_suffix(" x")
                    .with_default_value(smoothlife_speed.lr())
                    .with_callback(move |_, value| smoothlife_speed.sr(value))
            },
            smoothlife_preset: {
                let smoothlife_preset = Arc::clone(&params.smoothlife_preset);
                Menu::new(ui_layout.smooth_life.preset)
                    .with_callback(move |selected| {
                        smoothlife_preset.sr(selected);
                    })
                    .with_label("Preset")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
            },

            voronoi_cell_count: {
                let cell_count = Arc::clone(&params.voronoi_cell_count);
                TextSlider::new(10.0, ui_layout.voronoi.cell_count)
                    .with_label("Count")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(2.0..=32.0)
                    .with_integer_rounding()
                    .with_default_value(cell_count.lr() as f64)
                    .with_value_chars(2)
                    .with_callback(move |_, val| {
                        cell_count.sr(val as u32);
                    })
            },
            voronoi_cell_speed: {
                let cell_speed = Arc::clone(&params.voronoi_cell_speed);
                TextSlider::new(0.0, ui_layout.voronoi.cell_speed)
                    .with_label("Speed")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(-1.0..=1.0)
                    .with_default_value(cell_speed.lr())
                    .with_suffix(" x")
                    .with_value_chars(4)
                    .with_callback(move |_, val| {
                        cell_speed.sr(val);
                    })
            },
            voronoi_border_weight: {
                let weight = Arc::clone(&params.voronoi_border_weight);
                TextSlider::new(0.0, ui_layout.voronoi.border_weight)
                    .with_label("Weight")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_value_chars(4)
                    .with_output_range(0.01..=1.0)
                    .with_default_value(weight.lr())
                    .with_callback(move |_, value| {
                        weight.sr(value);
                    })
            },

            spectrogram_label: Label::new(ui_layout.spectrogram.label)
                .with_text("PARAMETRIC EQ")
                .with_text_layout(big_label_layout()),
            spectrogram_resolution: {
                let spectrogram_resolution =
                    Arc::clone(&params.spectrogram_resolution);
                Menu::new(ui_layout.spectrogram.resolution)
                    .with_callback(move |selected| {
                        spectrogram_resolution.sr(selected);
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
                        spectrogram_view.sr(selected);
                    })
                    .with_label("View")
                    .with_label_layout(main_label_layout())
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
            },

            reso_bank_label: Label::new(ui_layout.reso_bank.label)
                // .with_text("RESONATOR BANK")
                .with_text("RESONATOR FIELD")
                .with_text_layout(big_label_layout()),
            reso_bank_scale: {
                let reso_bank_scale = Arc::clone(&params.reso_bank_scale);
                Menu::new(ui_layout.reso_bank.scale)
                    .with_callback(move |selected| {
                        reso_bank_scale.sr(selected);
                    })
                    .with_item_text_layout(small_value_layout())
                    .with_selected_item_text_layout(Layout {
                        font: Some(
                            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                                .expect("failed to load font bytes"),
                        ),
                        ..small_value_layout()
                    })
            },
            reso_bank_root_note: {
                let reso_bank_root_note =
                    Arc::clone(&params.reso_bank_root_note);
                TextSlider::new(0.0, ui_layout.reso_bank.root_note)
                    .with_output_range(60.0..=71.0)
                    // .with_sensitivity(0.001)
                    .with_value_layout(Layout {
                        font: Some(
                            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                                .expect("failed to load font bytes"),
                        ),
                        ..small_value_layout()
                    })
                    .with_sensitivity(0.007)
                    .with_default_value(60.0) // C4
                    .with_integer_rounding()
                    .with_callback(move |_, value| {
                        reso_bank_root_note.sr(value as u8);
                    })
                    .with_formatting_callback(|raw, val| {
                        format!("{}", Note::from_value(val as i32))
                    })
            },
            reso_bank_spread: {
                let reso_bank_spread = Arc::clone(&params.reso_bank_spread);
                TextSlider::new(0.0, ui_layout.reso_bank.spread)
                    .with_label("Freq. spread")
                    .with_label_layout(Layout {
                        justify: Justify::Right,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_value_chars(4)
                    .with_default_value(reso_bank_spread.current_value())
                    .with_callback(move |_, value| {
                        reso_bank_spread.set_target_value(value)
                    })
            },
            reso_bank_shift: {
                let reso_bank_shift = Arc::clone(&params.reso_bank_shift);
                TextSlider::new(0.0, ui_layout.reso_bank.shift)
                    .with_sensitivity(0.001)
                    .with_label("Freq. shift")
                    .with_suffix(" st")
                    .with_positive_value_prefix()
                    .with_label_layout(Layout {
                        justify: Justify::Right,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_value_chars(5)
                    .with_output_range(-36.0..=36.0)
                    .with_default_value(reso_bank_shift.current_value())
                    .with_callback(move |_, value| {
                        reso_bank_shift.set_target_value(value)
                    })
            },
            reso_bank_inharm: {
                let reso_bank_inharm = Arc::clone(&params.reso_bank_inharm);
                TextSlider::new(0.0, ui_layout.reso_bank.inharm)
                    .with_label("Inharmonic")
                    .with_label_layout(Layout {
                        justify: Justify::Right,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_value_chars(4)
                    .with_default_value(reso_bank_inharm.current_value())
                    .with_callback(move |_, value| {
                        reso_bank_inharm.set_target_value(value)
                    })
            },
            reso_bank_pan: {
                let reso_bank_pan = Arc::clone(&params.reso_bank_pan);
                TextSlider::new(0.0, ui_layout.reso_bank.pan)
                    .with_label("Panning")
                    .with_label_layout(Layout {
                        justify: Justify::Right,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_value_chars(4)
                    .with_default_value(reso_bank_pan.current_value())
                    .with_callback(move |_, value| {
                        reso_bank_pan.set_target_value(value)
                    })
            },
            reso_bank_quantize: {
                let reso_bank_quantize = Arc::clone(&params.reso_bank_quantize);
                Button::new(ui_layout.reso_bank.quantize)
                    .with_callback(move |state| reso_bank_quantize.sr(state))
                    .with_state(true)
                    .with_enabled_layout(Layout {
                        font: Some(
                            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                                .expect("failed to load font bytes"),
                        ),
                        ..small_value_layout()
                    })
                    .with_disabled_layout(Layout {
                        font: Some(
                            Font::from_bytes(BOLD_FONT_MONO_BYTES)
                                .expect("failed to load font bytes"),
                        ),
                        ..small_value_layout()
                    })
                    .with_enabled_text("Quantise On")
                    .with_disabled_text("Quantise Off")
            },
            reso_bank_randomize: Button::new(ui_layout.reso_bank.randomise)
                .with_label("Regenerate")
                .with_label_layout(main_value_layout())
                .toggleable(false),
            reso_bank_push: Button::new(ui_layout.reso_bank.push)
                .with_label("Push")
                .with_label_layout(main_value_layout())
                .toggleable(false),

            reso_bank_resonator_count: {
                let reso_count = Arc::clone(&params.reso_bank_resonator_count);
                TextSlider::new(8.0, ui_layout.reso_bank.reso_count)
                    .with_label("Resonators")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_integer_rounding()
                    .with_output_range(1.0..=32.0)
                    .with_default_value(8.0)
                    .with_callback(move |_, value| {
                        reso_count.sr(value as u32);
                    })
            },
            reso_bank_cell_count: {
                let cell_count = Arc::clone(&params.reso_bank_cell_count);
                TextSlider::new(12.0, ui_layout.reso_bank.cell_count)
                    .with_label("Cells")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_integer_rounding()
                    .with_output_range(8.0..=16.0)
                    .with_default_value(12.0)
                    .with_callback(move |_, value| {
                        cell_count.sr(value as u32);
                    })
            },
            reso_bank_cell_jitter: {
                let cell_jitter = Arc::clone(&params.reso_bank_cell_jitter);
                TextSlider::new(0.1, ui_layout.reso_bank.cell_jitter)
                    .with_label("Jitter")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_default_value(0.1)
                    .with_value_chars(4)
                    .with_callback(move |_, value| {
                        cell_jitter.sr(value);
                    })
            },
            reso_bank_field_friction: {
                let friction = Arc::clone(&params.reso_bank_field_friction);
                TextSlider::new(0.5, ui_layout.reso_bank.field_friction)
                    .with_label("Friction")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(0.1..=1.0)
                    .with_default_value(0.5)
                    .with_value_chars(4)
                    .with_callback(move |_, value| {
                        friction.sr(value);
                    })
            },

            reso_bank_mix: {
                let mix = Arc::clone(&params.reso_bank_mix);
                TextSlider::new(0.0, ui_layout.reso_bank.mix)
                    .with_label("Mix")
                    .with_suffix(" %")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(0.0..=100.0)
                    .with_default_value(mix.current_value() * 100.0)
                    .with_value_chars(5)
                    .with_callback(move |raw, _| {
                        mix.set_target_value(raw);
                    })
            },

            exciter_osc: {
                let osc = Arc::clone(&params.exciter_osc);
                Menu::new(ui_layout.reso_bank.exciter)
                    .with_label("Exciter")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
                    .with_callback(move |selected| {
                        osc.sr(selected);
                    })
            },

            effects_label: Label::new(ui_layout.other.effects_label)
                .with_text("EFFECTS")
                .with_text_layout(big_label_layout()),

            low_filter_label: Label::new(ui_layout.low_filter.label)
                .with_text("LOW FILTER")
                .with_text_layout(big_label_layout()),
            low_filter_type: {
                let low_filter_type = Arc::clone(&params.low_filter_is_shelf);
                Button::new(ui_layout.low_filter.f_type)
                    .with_label("Type")
                    .with_label_layout(Layout {
                        justify: Justify::Right,
                        ..main_label_layout()
                    })
                    .with_enabled_text("Shelf")
                    .with_disabled_text("Cut")
                    .with_enabled_layout(main_value_layout())
                    .with_disabled_layout(main_value_layout())
                    .with_state(low_filter_type.lr())
                    .with_callback(move |state| low_filter_type.sr(state))
            },
            low_filter_cutoff: {
                let low_filter_cutoff = Arc::clone(&params.low_filter_cutoff);
                TextSlider::new(0.0, ui_layout.low_filter.cutoff_hz)
                    .with_sensitivity(0.0015)
                    .with_label("Cutoff")
                    .with_label_layout(Layout {
                        justify: Justify::Right,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(3.4868..=135.075_366) // 10Hz to 20kHz
                    .with_default_value(freq_to_note(
                        low_filter_cutoff.current_value(),
                    ))
                    .with_callback(move |_, value| {
                        low_filter_cutoff.set_target_value(note_to_freq(value));
                    })
                    .with_formatting_callback(|raw_value, output_value| {
                        let val = note_to_freq(output_value);

                        if val < 100.0 {
                            format!("{val:.2} Hz")
                        }
                        else if val < 1000.0 {
                            format!("{val:.1} Hz")
                        }
                        else if val < 10000.0 {
                            format!("{:.2} kHz", val / 1000.0)
                        }
                        else {
                            format!("{:.1} kHz", val / 1000.0)
                        }
                    })
            },
            low_filter_q: {
                let low_filter_q = Arc::clone(&params.low_filter_q);
                TextSlider::new(0.0, ui_layout.low_filter.q)
                    .with_sensitivity(0.002)
                    .with_label("Q")
                    .with_label_layout(Layout {
                        justify: Justify::Right,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_value_chars(4)
                    .with_output_range(0.3..=10.0)
                    .with_default_value(low_filter_q.current_value())
                    .with_callback(move |_, value| {
                        low_filter_q.set_target_value(value.recip());
                    })
            },
            low_filter_gain: {
                let low_filter_gain = Arc::clone(&params.low_filter_gain_db);
                TextSlider::new(0.0, ui_layout.low_filter.gain)
                    .with_sensitivity(0.002)
                    .with_label("Gain")
                    .with_positive_value_prefix()
                    .with_label_layout(Layout {
                        justify: Justify::Right,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(-24.0..=24.0)
                    .with_value_chars(5)
                    .with_suffix(" dB")
                    .with_default_value(low_filter_gain.current_value())
                    .with_callback(move |_, value| {
                        low_filter_gain.set_target_value(value);
                    })
            },

            high_filter_label: Label::new(ui_layout.high_filter.label)
                .with_text("HIGH FILTER")
                .with_text_layout(big_label_layout()),
            high_filter_type: {
                let high_filter_type = Arc::clone(&params.high_filter_is_shelf);
                Button::new(ui_layout.high_filter.f_type)
                    .with_label("Type")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_enabled_text("Shelf")
                    .with_disabled_text("Cut")
                    .with_enabled_layout(main_value_layout())
                    .with_disabled_layout(main_value_layout())
                    .with_state(high_filter_type.lr())
                    .with_callback(move |state| high_filter_type.sr(state))
            },
            high_filter_cutoff: {
                let high_filter_cutoff = Arc::clone(&params.high_filter_cutoff);
                TextSlider::new(0.0, ui_layout.high_filter.cutoff_hz)
                    .with_sensitivity(0.0015)
                    .with_label("Cutoff")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(3.4868..=135.075_366) // 200Hz to 20kHz
                    .with_default_value(freq_to_note(
                        high_filter_cutoff.current_value(),
                    ))
                    .with_callback(move |_, value| {
                        high_filter_cutoff
                            .set_target_value(note_to_freq(value));
                    })
                    .with_formatting_callback(|raw_value, output_value| {
                        let val = note_to_freq(output_value);

                        if val < 100.0 {
                            format!("{val:.2} Hz")
                        }
                        else if val < 1000.0 {
                            format!("{val:.1} Hz")
                        }
                        else if val < 10000.0 {
                            format!("{:.2} kHz", val / 1000.0)
                        }
                        else {
                            format!("{:.1} kHz", val / 1000.0)
                        }
                    })
            },
            high_filter_q: {
                let high_filter_q = Arc::clone(&params.high_filter_q);
                TextSlider::new(0.0, ui_layout.high_filter.q)
                    .with_sensitivity(0.002)
                    .with_label("Q")
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_value_chars(4)
                    .with_output_range(0.3..=10.0)
                    .with_default_value(high_filter_q.current_value())
                    .with_callback(move |_, value| {
                        high_filter_q.set_target_value(value.recip());
                    })
            },
            high_filter_gain: {
                let high_filter_gain = Arc::clone(&params.high_filter_gain_db);
                TextSlider::new(0.0, ui_layout.high_filter.gain)
                    .with_sensitivity(0.002)
                    .with_label("Gain")
                    .with_positive_value_prefix()
                    .with_label_layout(Layout {
                        justify: Justify::Left,
                        ..main_label_layout()
                    })
                    .with_value_layout(main_value_layout())
                    .with_output_range(-24.0..=24.0)
                    .with_value_chars(5)
                    .with_suffix(" dB")
                    .with_default_value(high_filter_gain.current_value())
                    .with_callback(move |_, value| {
                        high_filter_gain.set_target_value(value);
                    })
            },

            delay_label: Label::new(ui_layout.delay.label)
                .with_text("DELAY")
                .with_text_layout(big_label_layout()),
            delay_time_ms: {
                let pp_delay_time_ms = Arc::clone(&params.delay_time_ms);
                TextSlider::new(0.0, ui_layout.delay.time_ms)
                    .with_label("Time")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_suffix(" ms")
                    .with_output_range(10.0..=999.9)
                    .with_value_chars(5)
                    .with_default_value(pp_delay_time_ms.lr())
                    .with_callback(move |_, value| {
                        pp_delay_time_ms.sr(value);
                    })
            },
            delay_feedback: {
                let pp_delay_feedback = Arc::clone(&params.delay_feedback);
                TextSlider::new(0.0, ui_layout.delay.feedback)
                    .with_label("Feedback")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_suffix(" %")
                    .with_value_chars(5)
                    .with_output_range(0.0..=100.0)
                    .with_default_value(
                        pp_delay_feedback.current_value() * 100.0,
                    )
                    .with_callback(move |value, _| {
                        pp_delay_feedback.set_target_value(value);
                    })
            },
            delay_mix: {
                let pp_delay_mix = Arc::clone(&params.delay_mix);
                TextSlider::new(0.0, ui_layout.delay.mix)
                    .with_label("Mix")
                    .with_suffix(" %")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_output_range(0.0..=100.0)
                    .with_value_chars(5)
                    .with_default_value(pp_delay_mix.current_value() * 100.0)
                    .with_callback(move |value, _| {
                        pp_delay_mix.set_target_value(value);
                    })
            },
            delay_is_ping_pong: {
                let use_ping_pong = Arc::clone(&params.use_ping_pong);
                Button::new(ui_layout.delay.use_ping_pong)
                    .with_label("Ping-pong")
                    .with_label_layout(main_label_layout())
                    .with_enabled_layout(main_value_layout())
                    .with_disabled_layout(main_value_layout())
                    .with_state(true)
                    .with_callback(move |state| use_ping_pong.sr(state))
            },

            dist_label: Label::new(ui_layout.distortion.label)
                .with_text("DISTORTION")
                .with_text_layout(big_label_layout()),
            dist_amount: {
                let dist_amount = Arc::clone(&params.dist_amount);
                TextSlider::new(0.0, ui_layout.distortion.amount)
                    .with_label("Amount")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_suffix(" %")
                    .with_output_range(0.0..=100.0)
                    .with_default_value(dist_amount.current_value())
                    .with_callback(move |raw, _| {
                        dist_amount.set_target_value(raw);
                    })
            },
            dist_type: {
                let dist_type = Arc::clone(&params.dist_type);
                Menu::new(ui_layout.distortion.dist_type)
                    .with_label_layout(main_label_layout())
                    .with_item_text_layout(main_value_layout())
                    .with_selected_item_text_layout(main_value_layout())
                    .with_callback(move |selected| {
                        dist_type.sr(selected);
                    })
                    .with_label("Type")
            },

            comp_label: Label::new(ui_layout.compression.label)
                .with_text("COMPRESSION")
                .with_text_layout(big_label_layout()),
            comp_thresh: {
                let comp_thresh = Arc::clone(&params.comp_thresh);
                TextSlider::new(0.0, ui_layout.compression.threshold)
                    .with_label("Threshold")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_output_range(MINUS_INFINITY_DB..=0.0)
                    .with_default_value(params.comp_thresh.current_value())
                    .with_callback(move |_, value| {
                        comp_thresh.set_target_value(value);
                    })
                    .with_formatting_callback(|_, val| {
                        if val <= -99.9 {
                            return String::from("-inf dB");
                        }
                        if (0.0..=0.01).contains(&val) {
                            return String::from("0.00 dB");
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
            },
            comp_ratio: {
                let comp_ratio = Arc::clone(&params.comp_ratio);
                TextSlider::new(0.0, ui_layout.compression.ratio)
                    .with_label("Ratio")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_suffix(":1")
                    .with_output_range(1.0..=20.0)
                    .with_default_value(params.comp_ratio.current_value())
                    .with_value_chars(4)
                    .with_callback(move |_, value| {
                        comp_ratio.set_target_value(value);
                    })
            },
            comp_attack: {
                let comp_attack = Arc::clone(&params.comp_attack_ms);
                TextSlider::new(0.0, ui_layout.compression.attack)
                    .with_label("Attack")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_suffix(" ms")
                    .with_output_range(1.0..=500.0)
                    .with_value_chars(5)
                    .with_default_value(params.comp_attack_ms.current_value())
                    .with_callback(move |_, value| {
                        comp_attack.set_target_value(value);
                    })
            },
            comp_release: {
                let comp_release = Arc::clone(&params.comp_release_ms);
                TextSlider::new(0.0, ui_layout.compression.release)
                    .with_label("Release")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_suffix(" ms")
                    .with_output_range(1.0..=950.0)
                    .with_value_chars(5)
                    .with_default_value(params.comp_release_ms.current_value())
                    .with_callback(move |_, value| {
                        comp_release.set_target_value(value);
                    })
            },

            master_gain: {
                let master_gain = Arc::clone(&params.master_gain);
                TextSlider::new(0.0, ui_layout.other.master_gain)
                    .with_sensitivity(0.001)
                    .with_label("Output Gain")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_output_range(-100.0..=18.0) // -inf to +12
                    .with_default_value(0.0)
                    .with_formatting_callback(|_, val| {
                        if val <= -99.9 {
                            return String::from("-inf dB");
                        }
                        if (0.0..=0.01).contains(&val) {
                            return String::from("0.00 dB");
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
                    .with_callback(move |_, val| {
                        master_gain.set_target_value(db_to_level(val));
                    })
            },
            pre_fx_gain: {
                let pre_fx_gain = Arc::clone(&params.pre_fx_gain);
                TextSlider::new(0.0, ui_layout.other.pre_fx_gain)
                    .with_label("Pre-FX Gain")
                    .with_label_layout(main_label_layout())
                    .with_value_layout(main_value_layout())
                    .with_output_range(-36.0..=36.0)
                    .with_default_value(0.0)
                    .with_sensitivity(0.002)
                    .with_callback(move |_, val| {
                        pre_fx_gain.set_target_value(db_to_level(val));
                    })
                    .with_formatting_callback(|_, val| {
                        if (0.0..=0.01).contains(&val) {
                            return String::from("0.00 dB");
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
            },
        }
    }

    pub fn attach_reso_bank_randomize_callback<F: Fn(bool) + 'static>(
        mut self,
        cb: F,
    ) -> Self {
        self.reso_bank_randomize.set_callback(cb);
        self
    }

    pub fn attach_reso_bank_push_callback<F: Fn(bool) + 'static>(
        mut self,
        cb: F,
    ) -> Self {
        self.reso_bank_push.set_callback(cb);
        self
    }

    pub fn attach_mask_reset_callback<F: Fn(bool) + 'static>(
        mut self,
        cb: F,
    ) -> Self {
        self.mask_reset.set_callback(cb);
        self
    }

    #[allow(clippy::missing_panics_doc, clippy::significant_drop_tightening)]
    pub fn setup_mask_callbacks(
        mut self,
        contours: Arc<RwLock<ContoursGPU>>,
        smooth_life: Arc<RwLock<SmoothLifeGPU>>,
        params: &UIParams,
    ) -> Self {
        let ctr = contours.read().unwrap();
        let ctr_speed = ctr.z_increment_arc();
        let ctr_upper = ctr.upper_arc();
        let ctr_count = ctr.num_contours_arc();

        self.contour_speed.set_callback(move |_, val| {
            ctr_speed.sr(val as f32);
        });

        self.contour_count.set_callback(move |_, val| {
            ctr_count.sr(val as u32);
        });

        self.contour_thickness.set_callback(move |_, val| {
            ctr_upper.sr(val as f32);
        });

        let sml = smooth_life.read().unwrap();
        let sml_speed = sml.speed_arc();
        let sml_preset = sml.preset_arc();

        self.smoothlife_speed.set_callback(move |_, val| {
            sml_speed.sr(val as f32);
        });

        self.smoothlife_preset.set_callback(move |selected| {
            sml_preset.sr(selected);
        });

        self
    }

    pub fn setup_audio_channels(
        mut self,
        audio_senders: Arc<AudioMessageSenders>,
    ) -> Self {
        // let as_1 = Arc::clone(&audio_senders);
        // self.mask_is_post_fx.set_callback(move |state| {
        //     as_1.spectral_mask_post_fx.send(()).unwrap();
        // });
        // let rbp = Arc::new(Mutex::new(ResonatorBankParams::default()));

        self
    }
}

impl UIDraw for UIComponents {
    fn update(&mut self, app: &App, input_data: &InputData) {
        self.mask_algorithm.update(app, input_data);
        self.mask_scan_line_speed.update(app, input_data);
        // self.mask_is_post_fx.update(app, input_data);
        self.mask_mix.update(app, input_data);
        self.mask_resolution.update(app, input_data);
        self.mask_reset.update(app, input_data);

        match self.mask_algorithm.output() {
            GenerativeAlgo::Contours => {
                self.contour_count.update(app, input_data);
                self.contour_thickness.update(app, input_data);
                self.contour_speed.update(app, input_data);
            }
            GenerativeAlgo::SmoothLife => {
                // unused components
                // self.smoothlife_resolution.update(app, input_data);
                // self.smoothlife_speed.update(app, input_data);
                self.smoothlife_preset.update(app, input_data);
            }
            GenerativeAlgo::Voronoi => {
                self.voronoi_border_weight.update(app, input_data);
                self.voronoi_cell_speed.update(app, input_data);
                self.voronoi_cell_count.update(app, input_data);
            }
        }

        // unused components
        // self.spectrogram_resolution.update(app, input_data);
        // self.spectrogram_timing.update(app, input_data);
        self.spectrogram_view.update(app, input_data);

        self.reso_bank_scale.update(app, input_data);
        self.reso_bank_root_note.update(app, input_data);
        self.reso_bank_spread.update(app, input_data);
        self.reso_bank_shift.update(app, input_data);
        self.reso_bank_inharm.update(app, input_data);
        self.reso_bank_pan.update(app, input_data);
        self.reso_bank_quantize.update(app, input_data);
        self.reso_bank_randomize.update(app, input_data);
        self.reso_bank_push.update(app, input_data);

        self.reso_bank_resonator_count.update(app, input_data);
        // self.reso_bank_cell_count.update(app, input_data);
        self.reso_bank_cell_jitter.update(app, input_data);
        self.reso_bank_field_friction.update(app, input_data);
        self.reso_bank_mix.update(app, input_data);
        self.exciter_osc.update(app, input_data);

        self.low_filter_type.update(app, input_data);
        self.low_filter_cutoff.update(app, input_data);

        if self.low_filter_type.enabled() {
            self.low_filter_gain.update(app, input_data);
        }
        else {
            self.low_filter_q.update(app, input_data);
        }

        self.high_filter_type.update(app, input_data);
        self.high_filter_cutoff.update(app, input_data);

        if self.high_filter_type.enabled() {
            self.high_filter_gain.update(app, input_data);
        }
        else {
            self.high_filter_q.update(app, input_data);
        }

        self.dist_amount.update(app, input_data);
        self.dist_type.update(app, input_data);

        self.delay_time_ms.update(app, input_data);
        self.delay_feedback.update(app, input_data);
        self.delay_mix.update(app, input_data);
        self.delay_is_ping_pong.update(app, input_data);

        self.comp_thresh.update(app, input_data);
        self.comp_ratio.update(app, input_data);
        self.comp_attack.update(app, input_data);
        self.comp_release.update(app, input_data);

        self.pre_fx_gain.update(app, input_data);
        self.master_gain.update(app, input_data);
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        self.mask_label.draw(app, draw, frame);
        self.spectrogram_label.draw(app, draw, frame);
        self.reso_bank_label.draw(app, draw, frame);
        self.low_filter_label.draw(app, draw, frame);
        self.high_filter_label.draw(app, draw, frame);
        self.dist_label.draw(app, draw, frame);
        self.delay_label.draw(app, draw, frame);
        self.comp_label.draw(app, draw, frame);
        self.effects_label.draw(app, draw, frame);

        if self.low_filter_type.enabled() {
            self.low_filter_gain.draw(app, draw, frame);
        }
        else {
            self.low_filter_q.draw(app, draw, frame);
        }

        self.high_filter_type.draw(app, draw, frame);
        self.high_filter_cutoff.draw(app, draw, frame);

        if self.high_filter_type.enabled() {
            self.high_filter_gain.draw(app, draw, frame);
        }
        else {
            self.high_filter_q.draw(app, draw, frame);
        }

        self.mask_scan_line_speed.draw(app, draw, frame);
        // self.mask_is_post_fx.draw(app, draw, frame);
        self.mask_mix.draw(app, draw, frame);
        self.mask_resolution.draw(app, draw, frame);
        self.mask_reset.draw(app, draw, frame);

        match self.mask_algorithm.output() {
            GenerativeAlgo::Contours => {
                self.contour_count.draw(app, draw, frame);
                self.contour_thickness.draw(app, draw, frame);
                self.contour_speed.draw(app, draw, frame);
            }
            GenerativeAlgo::SmoothLife => {
                // unused components
                // self.smoothlife_resolution.draw(app, draw, frame);
                // self.smoothlife_speed.draw(app, draw, frame);
                self.smoothlife_preset.draw(app, draw, frame);
            }
            GenerativeAlgo::Voronoi => {
                self.voronoi_cell_speed.draw(app, draw, frame);
                self.voronoi_border_weight.draw(app, draw, frame);
                self.voronoi_cell_count.draw(app, draw, frame);
            }
        }
        self.mask_algorithm.draw(app, draw, frame); // menu

        // unused components
        // self.spectrogram_timing.draw(app, draw, frame);
        // self.spectrogram_resolution.draw(app, draw, frame); // menu
        self.spectrogram_view.draw(app, draw, frame); // menu

        self.reso_bank_root_note.draw(app, draw, frame);
        self.reso_bank_spread.draw(app, draw, frame);
        self.reso_bank_shift.draw(app, draw, frame);
        self.reso_bank_inharm.draw(app, draw, frame);
        self.reso_bank_pan.draw(app, draw, frame);
        self.reso_bank_quantize.draw(app, draw, frame);
        self.reso_bank_randomize.draw(app, draw, frame);
        self.reso_bank_push.draw(app, draw, frame);
        self.reso_bank_scale.draw(app, draw, frame); // menu

        self.reso_bank_resonator_count.draw(app, draw, frame);
        // self.reso_bank_cell_count.draw(app, draw, frame);
        self.reso_bank_cell_jitter.draw(app, draw, frame);
        self.reso_bank_field_friction.draw(app, draw, frame);

        self.reso_bank_mix.draw(app, draw, frame);
        self.exciter_osc.draw(app, draw, frame);

        self.low_filter_type.draw(app, draw, frame);
        self.low_filter_cutoff.draw(app, draw, frame);

        self.dist_amount.draw(app, draw, frame);

        self.delay_time_ms.draw(app, draw, frame);
        self.delay_feedback.draw(app, draw, frame);
        self.delay_mix.draw(app, draw, frame);
        self.delay_is_ping_pong.draw(app, draw, frame);

        self.dist_type.draw(app, draw, frame); // menu

        self.comp_thresh.draw(app, draw, frame);
        self.comp_ratio.draw(app, draw, frame);
        self.comp_attack.draw(app, draw, frame);
        self.comp_release.draw(app, draw, frame);

        self.pre_fx_gain.draw(app, draw, frame);
        self.master_gain.draw(app, draw, frame);
    }

    fn rect(&self) -> &nannou::prelude::Rect {
        unimplemented!("UIComponents does not have a bounding rect!")
    }
}
