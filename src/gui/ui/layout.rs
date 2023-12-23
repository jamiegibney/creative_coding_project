use super::*;
use nannou::prelude::*;

// god help me

fn def_rect() -> Rect {
    Rect::from_xy_wh(pt2(-4000.0, 1000.0), pt2(300.0, 300.0))
}

const MAIN_HEIGHT: f32 = 28.0;
const SMALL_HEIGHT: f32 = 18.0;

fn main_width_chars(max_chars: u32) -> f32 {
    let padding = 18.0;
    max_chars as f32 * 12.0 + padding
}

fn small_width_chars(max_chars: u32) -> f32 {
    let padding = 12.0;
    max_chars as f32 * 8.5 + padding
}

pub struct MaskUILayout {
    pub label: Rect,
    pub algorithm: Rect,
    pub scan_line_speed: Rect,
    pub resolution: Rect,
    pub is_post_fx: Rect,
    pub reset: Rect,
}

impl Default for MaskUILayout {
    fn default() -> Self {
        let reset_rect = Rect::from_xy_wh(
            pt2(0.0, 28.0),
            pt2(main_width_chars(6), MAIN_HEIGHT),
        );
        let upper_size = 256.0;

        let reso_rect = Rect::from_xy_wh(
            pt2(0.0, 332.0 - SMALL_HEIGHT * 2.0),
            pt2(small_width_chars(4), SMALL_HEIGHT * 4.0),
        );

        let sp_w = small_width_chars(6);
        let speed_rect = Rect::from_xy_wh(
            pt2(128.0 - sp_w / 2.0, 323.0),
            pt2(sp_w, SMALL_HEIGHT),
        );

        let pfx_w = small_width_chars(7);
        let post_fx_rect = Rect::from_xy_wh(
            pt2(-128.0 + pfx_w / 2.0, 323.0),
            pt2(pfx_w, SMALL_HEIGHT),
        );

        let label_rect =
            Rect::from_xy_wh(pt2(0.0, 366.0), pt2(180.0, MAIN_HEIGHT));

        let al_w = main_width_chars(11);
        let algo_rect = Rect::from_xy_wh(
            pt2(128.0 + al_w / 2.0 + 10.0, 310.0 - MAIN_HEIGHT - 26.0),
            pt2(al_w, MAIN_HEIGHT * 2.0),
        );

        Self {
            label: label_rect,
            algorithm: algo_rect,
            scan_line_speed: speed_rect,
            resolution: reso_rect,
            is_post_fx: post_fx_rect,
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
        // let algo_rect = Rect::from_xy_wh(
        //     pt2(128.0 + al_w / 2.0 + 10.0, 310.0 - MAIN_HEIGHT - 26.0),
        //     pt2(main_width_chars(11), MAIN_HEIGHT * 2.0),
        // );

        let sp_w = main_width_chars(6);
        let speed_rect = Rect::from_xy_wh(
            pt2(128.0 + sp_w / 2.0 + 10.0, 190.0 + MAIN_HEIGHT / 2.0),
            pt2(sp_w, MAIN_HEIGHT),
        );

        let th_w = main_width_chars(4);
        let thick_rect = Rect::from_xy_wh(
            pt2(128.0 + th_w / 2.0 + 10.0, 120.0 + MAIN_HEIGHT / 2.0),
            pt2(th_w, MAIN_HEIGHT),
        );

        let ct_w = main_width_chars(2);
        let count_rect = Rect::from_xy_wh(
            pt2(128.0 + ct_w / 2.0 + 10.0, 50.0 + MAIN_HEIGHT / 2.0),
            pt2(ct_w, MAIN_HEIGHT),
        );

        Self { count: count_rect, thickness: thick_rect, speed: speed_rect }
    }
}

pub struct SmoothLifeUILayout {
    pub resolution: Rect,
    pub speed: Rect,
    pub preset: Rect,
}

impl Default for SmoothLifeUILayout {
    fn default() -> Self {
        let pr_w = main_width_chars(5);
        let preset_rect = Rect::from_xy_wh(
            pt2(128.0 + pr_w / 2.0 + 10.0, 120.0),
            pt2(pr_w, MAIN_HEIGHT * 2.0),
        );

        let sp_w = main_width_chars(6);
        let speed_rect = Rect::from_xy_wh(
            pt2(128.0 + sp_w / 2.0 + 10.0, 190.0 + MAIN_HEIGHT / 2.0),
            pt2(sp_w, MAIN_HEIGHT),
        );

        let rs_w = main_width_chars(3);
        let reso_rect = Rect::from_xy_wh(
            pt2(128.0 + rs_w / 2.0 + 10.0, 50.0 - MAIN_HEIGHT * 2.0),
            pt2(rs_w, MAIN_HEIGHT * 6.0),
        );

        Self { resolution: reso_rect, speed: speed_rect, preset: preset_rect }
    }
}

pub struct SpectrogramUILayout {
    pub label: Rect,
    pub resolution: Rect,
    pub timing: Rect,
    pub view: Rect,
}

impl Default for SpectrogramUILayout {
    fn default() -> Self {
        Self {
            label: def_rect(),
            resolution: def_rect(),
            timing: def_rect(),
            view: def_rect(),
        }
    }
}

pub struct ResoBankUILayout {
    pub label: Rect,
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
            label: def_rect(),
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
    pub label: Rect,
    pub cutoff_hz: Rect,
    pub q: Rect,
}

impl Default for LowpassUILayout {
    fn default() -> Self {
        Self { label: def_rect(), cutoff_hz: def_rect(), q: def_rect() }
    }
}

pub struct HighpassUILayout {
    pub label: Rect,
    pub cutoff_hz: Rect,
    pub q: Rect,
}

impl Default for HighpassUILayout {
    fn default() -> Self {
        Self { label: def_rect(), cutoff_hz: def_rect(), q: def_rect() }
    }
}

pub struct DelayUILayout {
    pub label: Rect,
    pub time_ms: Rect,
    pub feedback: Rect,
    pub mix: Rect,
    pub tempo_sync: Rect,
}

impl Default for DelayUILayout {
    fn default() -> Self {
        Self {
            label: def_rect(),
            time_ms: def_rect(),
            feedback: def_rect(),
            mix: def_rect(),
            tempo_sync: def_rect(),
        }
    }
}

pub struct DistortionUILayout {
    pub label: Rect,
    pub amount: Rect,
    pub dist_type: Rect,
}

impl Default for DistortionUILayout {
    fn default() -> Self {
        Self { label: def_rect(), amount: def_rect(), dist_type: def_rect() }
    }
}

pub struct CompressionUILayout {
    pub label: Rect,
    pub threshold: Rect,
    pub ratio: Rect,
    pub attack: Rect,
    pub release: Rect,
}

impl Default for CompressionUILayout {
    fn default() -> Self {
        Self {
            label: def_rect(),
            threshold: def_rect(),
            ratio: def_rect(),
            attack: def_rect(),
            release: def_rect(),
        }
    }
}

pub struct OtherUILayout {
    pub effects_label: Rect,
    pub master_gain: Rect,
}

impl Default for OtherUILayout {
    fn default() -> Self {
        Self { effects_label: def_rect(), master_gain: def_rect() }
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
    pub compression: CompressionUILayout,
    pub other: OtherUILayout,
}
