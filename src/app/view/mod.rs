use crate::{gui::colors::*, prelude::xfer::s_curve};
use nannou::geom::{path, Path};

use super::*;

pub fn view(app: &App, model: &Model, frame: Frame) {
    if !model.input_data.is_win_focussed {
        return;
    }

    let draw = &app.draw();
    let window = app.main_window();
    let is_first_frame = frame.nth() == 0;
    if is_first_frame {
        draw.background().color(BLACK);
    }

    let line_weight = 2.0;
    let bank_rect = &model.bank_rect;

    model.redraw_under_menus(draw, is_first_frame);
    model.redraw_filter_sliders(draw);

    let spectrum_rect = model.spectrum_rect;
    draw.rect()
        .xy(spectrum_rect.xy())
        .wh(spectrum_rect.wh())
        .color(BLACK);

    model.draw_log_lines(draw);

    let spectrogram_view = model.ui_params.spectrogram_view.lr();

    if matches!(
        spectrogram_view,
        SpectrogramView::PrePost | SpectrogramView::PreOnly
    ) {
        model
            .pre_spectrum_analyzer
            .draw(app, draw, &frame);
    }

    if matches!(
        spectrogram_view,
        SpectrogramView::PrePost | SpectrogramView::PostOnly
    ) {
        model
            .post_spectrum_analyzer
            .draw(app, draw, &frame);
    }

    let mask_rect = model.mask_rect;
    draw.rect()
        .xy(mask_rect.xy())
        .wh(mask_rect.wh() * 1.05)
        .color(BLACK);

    let mask_mix = model.ui_params.mask_mix.lr();

    if !epsilon_eq(mask_mix, 0.0) {
        match model.ui_params.mask_algorithm.lr() {
            GenerativeAlgo::Contours => {
                model.contours.read().unwrap().draw(app, draw, &frame);
            }
            GenerativeAlgo::SmoothLife => {
                model.smooth_life.read().unwrap().draw(app, draw, &frame);
            }
            GenerativeAlgo::Voronoi => {
                model.voronoi_mask.read().unwrap().draw(app, draw, &frame);
            }
        }
    }
    if mask_mix < 1.0 {
        draw.rect()
            .xy(model.mask_rect.xy())
            .wh(model.mask_rect.wh())
            .color(Rgba::new(1.0, 1.0, 1.0, s_curve(1.0 - mask_mix, -0.9)));
    }

    outline_rect(&model.mask_rect, draw, 2.0);
    model.draw_mask_scan_line(draw);

    model.eq_display.draw(app, draw, &frame);
    // model.draw_filter_line(draw);
    // model.draw_filter_nodes(draw);
    outline_rect(&model.spectrum_rect, draw, 2.0);

    // TODO
    // I tried to get this to draw lazily, but it seems as though
    // the voronoi cells update one or two frames after the vectors.
    // need to look into that further, but for now this just
    // gets drawn every frame. the voronoi field does get updated
    // lazily which is where the compute shader is dispatched, so
    // at least there is some performance saving there.
    draw.rect()
        .xy(bank_rect.xy())
        .wh(bank_rect.wh())
        .color(BLACK);
    model.voronoi_reso_bank.draw(app, draw, &frame);
    model.vectors_reso_bank.draw(app, draw, &frame);
    outline_rect(bank_rect, draw, line_weight);

    model.ui_components.draw(app, draw, &frame);

    // if the frame fails to draw we'll just ignore it
    let _ = draw.to_frame(app, &frame);
}

fn outline_rect(rect: &Rect, draw: &Draw, width: f32) {
    draw.rect()
        .xy(rect.xy())
        .wh(rect.wh())
        .stroke(BG_NON_SELECTED)
        .stroke_weight(width)
        .no_fill();
}
