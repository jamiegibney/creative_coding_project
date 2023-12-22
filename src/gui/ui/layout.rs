use super::*;
use nannou::prelude::*;

// god help me

fn def_rect() -> Rect {
    Rect::from_xy_wh(pt2(-4000.0, 1000.0), pt2(300.0, 300.0))
}

pub struct MaskUILayout {
    pub algorithm: Rect,
    pub scan_line_speed: Rect,
    pub resolution: Rect,
    pub is_post_fx: Rect,
    pub uses_gpu: Rect,
    pub reset: Rect,
}

impl Default for MaskUILayout {
    fn default() -> Self {
        let w = 70.0;
        let contour_size_fl = 128.0;
        let reset_rect = Rect::from_corners(
            pt2(-contour_size_fl / 2.0 - 230.0, -contour_size_fl - 50.0),
            pt2(-contour_size_fl / 2.0 - 230.0 + w, -contour_size_fl - 10.0),
        );

        Self {
            algorithm: Rect::from_xy_wh(pt2(-358.0, -130.0), pt2(30.0, 35.0)),
            scan_line_speed: reset_rect.shift(pt2(60.0, 0.0)),
            resolution: def_rect(),
            is_post_fx: def_rect(),
            uses_gpu: def_rect(),
            reset: reset_rect,
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
        Self { count: def_rect(), thickness: def_rect(), speed: def_rect() }
    }
}

pub struct SmoothLifeUILayout {
    pub resolution: Rect,
    pub speed: Rect,
    pub preset: Rect,
}

impl Default for SmoothLifeUILayout {
    fn default() -> Self {
        Self { resolution: def_rect(), speed: def_rect(), preset: def_rect() }
    }
}

pub struct SpectrogramUILayout {
    pub resolution: Rect,
    pub timing: Rect,
    pub view: Rect,
}

impl Default for SpectrogramUILayout {
    fn default() -> Self {
        Self { resolution: def_rect(), timing: def_rect(), view: def_rect() }
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
            scale: def_rect(),
            root_note: def_rect(),
            spread: def_rect(),
            shift: def_rect(),
            inharm: def_rect(),
            pan: def_rect(),
            quantise: def_rect(),
            randomise: def_rect(),
        }
    }
}

pub struct LowpassUILayout {
    pub cutoff_hz: Rect,
    pub q: Rect,
}

impl Default for LowpassUILayout {
    fn default() -> Self {
        Self { cutoff_hz: def_rect(), q: def_rect() }
    }
}

pub struct HighpassUILayout {
    pub cutoff_hz: Rect,
    pub q: Rect,
}

impl Default for HighpassUILayout {
    fn default() -> Self {
        Self { cutoff_hz: def_rect(), q: def_rect() }
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
            time_ms: def_rect(),
            feedback: def_rect(),
            mix: def_rect(),
            tempo_sync: def_rect(),
        }
    }
}

pub struct DistortionUILayout {
    pub amount: Rect,
    pub dist_type: Rect,
}

impl Default for DistortionUILayout {
    fn default() -> Self {
        Self { amount: def_rect(), dist_type: def_rect() }
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
