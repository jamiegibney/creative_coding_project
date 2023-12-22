use super::*;
use nannou::prelude::*;

// god help me

pub struct MaskUILayout {
    pub mask_algorithm: Rect,
    pub mask_scan_line_speed: Rect,
    pub mask_resolution: Rect,
    pub mask_is_post_fx: Rect,
    pub mask_uses_gpu: Rect,
    pub mask_reset: Rect,
}

impl Default for MaskUILayout {
    fn default() -> Self {
        Self {
            mask_algorithm: todo!(),
            mask_scan_line_speed: todo!(),
            mask_resolution: todo!(),
            mask_is_post_fx: todo!(),
            mask_uses_gpu: todo!(),
            mask_reset: todo!(),
        }
    }
}

pub struct ContourUILayout {
    pub contour_count: Rect,
    pub contour_thickness: Rect,
    pub contour_speed: Rect,
}

impl Default for ContourUILayout {
    fn default() -> Self {
        Self {
            contour_count: todo!(),
            contour_thickness: todo!(),
            contour_speed: todo!(),
        }
    }
}

pub struct SmoothLifeUILayout {
    pub smoothlife_resolution: Rect,
    pub smoothlife_speed: Rect,
    pub smoothlife_preset: Rect,
}

impl Default for SmoothLifeUILayout {
    fn default() -> Self {
        Self {
            smoothlife_resolution: todo!(),
            smoothlife_speed: todo!(),
            smoothlife_preset: todo!(),
        }
    }
}

pub struct SpectrogramUILayout {
    pub spectrogram_resolution: Rect,
    pub spectrogram_timing: Rect,
    pub spectrogram_view: Rect,
}

impl Default for SpectrogramUILayout {
    fn default() -> Self {
        Self {
            spectrogram_resolution: todo!(),
            spectrogram_timing: todo!(),
            spectrogram_view: todo!(),
        }
    }
}

pub struct ResoBankUILayout {
    pub reso_bank_scale: Rect,
    pub reso_bank_root_note: Rect,
    pub reso_bank_spread: Rect,
    pub reso_bank_shift: Rect,
    pub reso_bank_inharm: Rect,
    pub reso_bank_pan: Rect,
    pub reso_bank_quantise: Rect,
    pub reso_bank_randomise: Rect,
}

impl Default for ResoBankUILayout {
    fn default() -> Self {
        Self {
            reso_bank_scale: todo!(),
            reso_bank_root_note: todo!(),
            reso_bank_spread: todo!(),
            reso_bank_shift: todo!(),
            reso_bank_inharm: todo!(),
            reso_bank_pan: todo!(),
            reso_bank_quantise: todo!(),
            reso_bank_randomise: todo!(),
        }
    }
}

pub struct LowpassUILayout {
    pub low_pass_cutoff_hz: Rect,
    pub low_pass_q: Rect,
}

impl Default for LowpassUILayout {
    fn default() -> Self {
        Self { low_pass_cutoff_hz: todo!(), low_pass_q: todo!() }
    }
}

pub struct HighpassUILayout {
    pub high_pass_cutoff_hz: Rect,
    pub high_pass_q: Rect,
}

impl Default for HighpassUILayout {
    fn default() -> Self {
        Self { high_pass_cutoff_hz: todo!(), high_pass_q: todo!() }
    }
}

pub struct DelayUILayout {
    pub pp_delay_time_ms: Rect,
    pub pp_delay_feedback: Rect,
    pub pp_delay_mix: Rect,
    pub pp_delay_tempo_sync: Rect,
}

impl Default for DelayUILayout {
    fn default() -> Self {
        Self {
            pp_delay_time_ms: todo!(),
            pp_delay_feedback: todo!(),
            pp_delay_mix: todo!(),
            pp_delay_tempo_sync: todo!(),
        }
    }
}

pub struct DistortionUILayout {
    pub dist_amount: Rect,
    pub dist_type: Rect,
}

impl Default for DistortionUILayout {
    fn default() -> Self {
        Self { dist_amount: todo!(), dist_type: todo!() }
    }
}

#[derive(Default)]
pub struct UILayout {
    pub mask_general: MaskUILayout,
    pub contour: ContourUILayout,
    pub smooth_life: SmoothLifeUILayout,
    pub spectrogram: SpectrogramUILayout,
    pub reso_bank: ResoBankUILayout,
    pub low_pass: LowpassUILayout,
    pub high_pass: HighpassUILayout,
    pub delay: DelayUILayout,
    pub distortion: DistortionUILayout,
}
