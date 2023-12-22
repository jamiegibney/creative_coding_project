use super::*;
use nannou::prelude::*;

// god help me

pub struct MaskUILayout {
    pub algorithm: Rect,
    pub scal_line_speed: Rect,
    pub resolution: Rect,
    pub is_post_fx: Rect,
    pub uses_gpu: Rect,
    pub reset: Rect,
}

impl Default for MaskUILayout {
    fn default() -> Self {
        Self {
            algorithm: todo!(),
            scal_line_speed: todo!(),
            resolution: todo!(),
            is_post_fx: todo!(),
            uses_gpu: todo!(),
            reset: todo!(),
        }
    }
}

pub struct ContourUILayout {
    pub count: Rect,
    pub thickness: Rect,
    pub speed: Rect,
}

impl Default for ContourUILayout {
    fn default() -> Self {
        Self {
            count: todo!(),
            thickness: todo!(),
            speed: todo!(),
        }
    }
}

pub struct SmoothLifeUILayout {
    pub resolution: Rect,
    pub speed: Rect,
    pub preset: Rect,
}

impl Default for SmoothLifeUILayout {
    fn default() -> Self {
        Self {
            resolution: todo!(),
            speed: todo!(),
            preset: todo!(),
        }
    }
}

pub struct SpectrogramUILayout {
    pub resolution: Rect,
    pub timing: Rect,
    pub view: Rect,
}

impl Default for SpectrogramUILayout {
    fn default() -> Self {
        Self {
            resolution: todo!(),
            timing: todo!(),
            view: todo!(),
        }
    }
}

pub struct ResoBankUILayout {
    pub scale: Rect,
    pub root_note: Rect,
    pub spread: Rect,
    pub shift: Rect,
    pub inharm: Rect,
    pub pan: Rect,
    pub quantise: Rect,
    pub randomise: Rect,
}

impl Default for ResoBankUILayout {
    fn default() -> Self {
        Self {
            scale: todo!(),
            root_note: todo!(),
            spread: todo!(),
            shift: todo!(),
            inharm: todo!(),
            pan: todo!(),
            quantise: todo!(),
            randomise: todo!(),
        }
    }
}

pub struct LowpassUILayout {
    pub cutoff_hz: Rect,
    pub q: Rect,
}

impl Default for LowpassUILayout {
    fn default() -> Self {
        Self { cutoff_hz: todo!(), q: todo!() }
    }
}

pub struct HighpassUILayout {
    pub cutoff_hz: Rect,
    pub q: Rect,
}

impl Default for HighpassUILayout {
    fn default() -> Self {
        Self { cutoff_hz: todo!(), q: todo!() }
    }
}

pub struct DelayUILayout {
    pub time_ms: Rect,
    pub feedback: Rect,
    pub mix: Rect,
    pub tempo_sync: Rect,
}

impl Default for DelayUILayout {
    fn default() -> Self {
        Self {
            time_ms: todo!(),
            feedback: todo!(),
            mix: todo!(),
            tempo_sync: todo!(),
        }
    }
}

pub struct DistortionUILayout {
    pub amount: Rect,
    pub dist_type: Rect,
}

impl Default for DistortionUILayout {
    fn default() -> Self {
        Self { amount: todo!(), dist_type: todo!() }
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
