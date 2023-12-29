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
            pt2(main_width_chars(10), MAIN_HEIGHT),
        );
        let upper_size = 256.0;

        let reso_rect = Rect::from_xy_wh(
            pt2(0.0, 332.0 - SMALL_HEIGHT * 4.5),
            pt2(small_width_chars(5), SMALL_HEIGHT * 9.0),
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
        let pr_w = main_width_chars(7);
        let preset_rect = Rect::from_xy_wh(
            pt2(128.0 + pr_w / 2.0 + 10.0, 190.0 - MAIN_HEIGHT / 2.0),
            pt2(pr_w, MAIN_HEIGHT * 3.0),
        );

        // unused
        let sp_w = main_width_chars(6);
        let speed_rect = Rect::from_xy_wh(
            pt2(128.0 + sp_w / 2.0 + 10.0, 190.0 + MAIN_HEIGHT / 2.0),
            pt2(sp_w, MAIN_HEIGHT),
        );

        // unused
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

// let spectrum_rect =
//     Rect::from_corners(pt2(-540.0, -310.0), pt2(128.0, -40.0));
//
// let upper_size = 256.0;
//
// let bank_rect = Rect::from_corners(
//     pt2(-540.0, 50.0),
//     pt2(-540.0 + upper_size, 50.0 + upper_size),
// );
//
// let contour_size_fl = upper_size / 2.0;
// let mask_rect = Rect::from_corners(
//     pt2(-contour_size_fl, 50.0),
//     pt2(contour_size_fl, 50.0 + upper_size),
// );

impl Default for SpectrogramUILayout {
    fn default() -> Self {
        let label_rect =
            Rect::from_xy_wh(pt2(-206.0, -12.0), pt2(120.0, MAIN_HEIGHT));

        let vw_w = main_width_chars(8);
        let view_rect = Rect::from_xy_wh(
            pt2(-206.0, -364.0 - MAIN_HEIGHT),
            pt2(vw_w, MAIN_HEIGHT * 3.0),
        );

        // unused
        let rs_w = main_width_chars(4);
        let reso_rect = Rect::from_xy_wh(
            pt2(-426.0, -364.0 - MAIN_HEIGHT * 1.5),
            pt2(rs_w, MAIN_HEIGHT * 4.0),
        );

        // unused
        let tm_w = main_width_chars(5);
        let time_rect =
            Rect::from_xy_wh(pt2(-16.0, -364.0), pt2(tm_w, MAIN_HEIGHT));

        Self {
            label: label_rect,
            resolution: reso_rect,
            timing: time_rect,
            view: view_rect,
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

    pub reso_count: Rect,
    pub cell_jitter: Rect,
    pub cell_count: Rect,
    pub cell_scatter: Rect,

    pub mix: Rect,
    pub exciter: Rect,
}

impl Default for ResoBankUILayout {
    fn default() -> Self {
        let label_rect =
            Rect::from_xy_wh(pt2(-412.0, 366.0), pt2(120.0, MAIN_HEIGHT));

        // let bank_rect = Rect::from_corners(
        //     pt2(-540.0, 50.0),
        //     pt2(-540.0 + upper_size, 50.0 + upper_size),
        // );

        let reset_rect = Rect::from_xy_wh(
            pt2(-412.0, 28.0),
            pt2(main_width_chars(15), MAIN_HEIGHT),
        );

        let sh_w = main_width_chars(8);
        let shift_rect = Rect::from_xy_wh(
            pt2(-540.0 - sh_w / 2.0 - 10.0, 258.0 + MAIN_HEIGHT / 2.0),
            pt2(sh_w, MAIN_HEIGHT),
        );

        let sp_w = main_width_chars(4);
        let spread_rect = Rect::from_xy_wh(
            pt2(-540.0 - sp_w / 2.0 - 10.0, 190.0 + MAIN_HEIGHT / 2.0),
            pt2(sp_w, MAIN_HEIGHT),
        );

        let ih_w = main_width_chars(4);
        let inharm_rect = Rect::from_xy_wh(
            pt2(-540.0 - ih_w / 2.0 - 10.0, 120.0 + MAIN_HEIGHT / 2.0),
            pt2(ih_w, MAIN_HEIGHT),
        );

        let pn_w = main_width_chars(4);
        let pan_rect = Rect::from_xy_wh(
            pt2(-540.0 - pn_w / 2.0 - 10.0, 50.0 + MAIN_HEIGHT / 2.0),
            pt2(pn_w, MAIN_HEIGHT),
        );

        let qn_w = small_width_chars(12);
        let quant_rect = Rect::from_xy_wh(
            pt2(-398.0 + qn_w / 2.0, 323.0),
            pt2(qn_w, SMALL_HEIGHT),
        );

        let sc_w = small_width_chars(9);
        let scale_rect = Rect::from_xy_wh(
            pt2(-503.0 + sc_w / 2.0, 332.0 - SMALL_HEIGHT * 2.5),
            pt2(sc_w, SMALL_HEIGHT * 5.0),
        );

        let rn_w = small_width_chars(2);
        let root_rect = Rect::from_xy_wh(
            pt2(-540.0 + rn_w / 2.0, 323.0),
            pt2(rn_w, SMALL_HEIGHT),
        );

        let rc_w = main_width_chars(2);
        let count_rect = Rect::from_xy_wh(
            pt2(-284.0 + rc_w / 2.0 + 10.0, 260.0 + MAIN_HEIGHT / 2.0),
            pt2(rc_w, MAIN_HEIGHT),
        );

        let sc_w = main_width_chars(4);
        let scatter_rect = Rect::from_xy_wh(
            pt2(-284.0 + sc_w / 2.0 + 10.0, 190.0 + MAIN_HEIGHT / 2.0),
            pt2(sc_w, MAIN_HEIGHT),
        );

        let mx_w = main_width_chars(7);
        let mix_rect = Rect::from_xy_wh(
            pt2(-284.0 + mx_w / 2.0 + 10.0, 120.0 + MAIN_HEIGHT / 2.0),
            pt2(mx_w, MAIN_HEIGHT),
        );

        let ex_w = main_width_chars(6);
        let exciter_rect = Rect::from_xy_wh(
            pt2(
                -284.0 + mx_w / 2.0 + 10.0,
                50.0 + MAIN_HEIGHT / 2.0 - MAIN_HEIGHT * 2.0,
            ),
            pt2(mx_w, MAIN_HEIGHT * 5.0),
        );

        Self {
            label: label_rect,

            root_note: root_rect,
            scale: scale_rect,
            quantise: quant_rect,

            shift: shift_rect,
            spread: spread_rect,
            inharm: inharm_rect,
            pan: pan_rect,

            randomise: reset_rect,

            reso_count: count_rect,
            cell_count: def_rect(),
            cell_jitter: def_rect(),
            cell_scatter: scatter_rect,

            mix: mix_rect,
            exciter: exciter_rect,
        }
    }
}

pub struct LowFilterUILayout {
    pub label: Rect,
    pub f_type: Rect,
    pub cutoff_hz: Rect,
    pub q: Rect,
    pub gain: Rect,
}

impl Default for LowFilterUILayout {
    fn default() -> Self {
        let label_rect =
            Rect::from_xy_wh(pt2(-615.0, -45.0), pt2(120.0, MAIN_HEIGHT));

        // let spectrum_rect =
        //     Rect::from_corners(pt2(-540.0, -310.0), pt2(128.0, -40.0));

        let ty_w = main_width_chars(5);
        let type_rect = Rect::from_xy_wh(
            pt2(-540.0 - ty_w / 2.0 - 10.0, -120.0 + MAIN_HEIGHT / 2.0),
            pt2(ty_w, MAIN_HEIGHT),
        );

        let co_w = main_width_chars(9);
        let cutoff_rect = Rect::from_xy_wh(
            pt2(-540.0 - co_w / 2.0 - 10.0, -215.0 + MAIN_HEIGHT / 2.0),
            pt2(co_w, MAIN_HEIGHT),
        );

        let q_w = main_width_chars(4);
        let q_rect = Rect::from_xy_wh(
            pt2(-540.0 - q_w / 2.0 - 10.0, -310.0 + MAIN_HEIGHT / 2.0),
            pt2(q_w, MAIN_HEIGHT),
        );

        let gn_w = main_width_chars(8);
        let gain_rect = Rect::from_xy_wh(
            pt2(-540.0 - gn_w / 2.0 - 10.0, -310.0 + MAIN_HEIGHT / 2.0),
            pt2(gn_w, MAIN_HEIGHT),
        );

        Self {
            label: label_rect,
            f_type: type_rect,
            cutoff_hz: cutoff_rect,
            q: q_rect,
            gain: gain_rect,
        }
    }
}

pub struct HighFilterUILayout {
    pub label: Rect,
    pub f_type: Rect,
    pub cutoff_hz: Rect,
    pub q: Rect,
    pub gain: Rect,
}

impl Default for HighFilterUILayout {
    fn default() -> Self {
        let label_rect =
            Rect::from_xy_wh(pt2(210.0, -45.0), pt2(120.0, MAIN_HEIGHT));

        let ty_w = main_width_chars(5);
        let type_rect = Rect::from_xy_wh(
            pt2(128.0 + ty_w / 2.0 + 10.0, -120.0 + MAIN_HEIGHT / 2.0),
            pt2(ty_w, MAIN_HEIGHT),
        );

        let co_w = main_width_chars(9);
        let cutoff_rect = Rect::from_xy_wh(
            pt2(128.0 + co_w / 2.0 + 10.0, -215.0 + MAIN_HEIGHT / 2.0),
            pt2(co_w, MAIN_HEIGHT),
        );

        let q_w = main_width_chars(4);
        let q_rect = Rect::from_xy_wh(
            pt2(128.0 + q_w / 2.0 + 10.0, -310.0 + MAIN_HEIGHT / 2.0),
            pt2(q_w, MAIN_HEIGHT),
        );

        let gn_w = main_width_chars(8);
        let gain_rect = Rect::from_xy_wh(
            pt2(128.0 + gn_w / 2.0 + 10.0, -310.0 + MAIN_HEIGHT / 2.0),
            pt2(gn_w, MAIN_HEIGHT),
        );

        Self {
            label: label_rect,
            f_type: type_rect,
            cutoff_hz: cutoff_rect,
            q: q_rect,
            gain: gain_rect,
        }
    }
}

pub struct DelayUILayout {
    pub label: Rect,
    pub time_ms: Rect,
    pub feedback: Rect,
    pub mix: Rect,
    pub use_ping_pong: Rect,
}

impl Default for DelayUILayout {
    fn default() -> Self {
        let label_rect =
            Rect::from_xy_wh(pt2(500.0, 70.0), pt2(120.0, MAIN_HEIGHT));

        let tm_w = main_width_chars(8);
        let time_rect =
            Rect::from_xy_wh(pt2(440.0, 8.0), pt2(tm_w, MAIN_HEIGHT));

        let pp_w = main_width_chars(3);
        let ping_rect =
            Rect::from_xy_wh(pt2(440.0, -70.0), pt2(pp_w, MAIN_HEIGHT));

        let fb_w = main_width_chars(7);
        let feed_rect =
            Rect::from_xy_wh(pt2(570.0, 8.0), pt2(fb_w, MAIN_HEIGHT));

        let mx_w = main_width_chars(7);
        let mix_rect =
            Rect::from_xy_wh(pt2(570.0, -70.0), pt2(mx_w, MAIN_HEIGHT));

        Self {
            label: label_rect,
            time_ms: time_rect,
            feedback: feed_rect,
            mix: mix_rect,
            use_ping_pong: ping_rect,
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
        let label_rect =
            Rect::from_xy_wh(pt2(500.0, 230.0), pt2(120.0, MAIN_HEIGHT));

        let am_w = main_width_chars(7);
        let amount_rect =
            Rect::from_xy_wh(pt2(442.0, 165.0), pt2(am_w, MAIN_HEIGHT));

        let ty_w = main_width_chars(5);
        let type_rect = Rect::from_xy_wh(
            pt2(572.0, 165.0 - MAIN_HEIGHT * 2.0),
            pt2(ty_w, MAIN_HEIGHT * 5.0),
        );

        Self { label: label_rect, amount: amount_rect, dist_type: type_rect }
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
        let label_rect =
            Rect::from_xy_wh(pt2(500.0, -180.0), pt2(120.0, MAIN_HEIGHT));

        let th_w = main_width_chars(8);
        let thrs_rect =
            Rect::from_xy_wh(pt2(435.0, -250.0), pt2(th_w, MAIN_HEIGHT));

        let rt_w = main_width_chars(6);
        let ratio_rect =
            Rect::from_xy_wh(pt2(565.0, -250.0), pt2(rt_w, MAIN_HEIGHT));

        let at_w = main_width_chars(8);
        let att_rect =
            Rect::from_xy_wh(pt2(435.0, -320.0), pt2(at_w, MAIN_HEIGHT));

        let rl_w = main_width_chars(8);
        let rel_rect =
            Rect::from_xy_wh(pt2(565.0, -320.0), pt2(rl_w, MAIN_HEIGHT));

        Self {
            label: label_rect,
            threshold: thrs_rect,
            ratio: ratio_rect,
            attack: att_rect,
            release: rel_rect,
        }
    }
}

pub struct OtherUILayout {
    pub effects_label: Rect,
    pub master_gain: Rect,
}

impl Default for OtherUILayout {
    fn default() -> Self {
        let mg_w = main_width_chars(8);
        let m_gain_rect =
            Rect::from_xy_wh(pt2(630.0, 350.0), pt2(mg_w, MAIN_HEIGHT));

        let fx_rect =
            Rect::from_xy_wh(pt2(500.0, 300.0), pt2(120.0, MAIN_HEIGHT));
        Self { effects_label: fx_rect, master_gain: m_gain_rect }
    }
}

#[derive(Default)]
pub struct UILayout {
    pub mask_general: MaskUILayout,
    pub contour: ContourUILayout,
    pub smooth_life: SmoothLifeUILayout,
    pub spectrogram: SpectrogramUILayout,
    pub reso_bank: ResoBankUILayout,
    pub low_filter: LowFilterUILayout,
    pub high_filter: HighFilterUILayout,
    pub delay: DelayUILayout,
    pub distortion: DistortionUILayout,
    pub compression: CompressionUILayout,
    pub other: OtherUILayout,
}
