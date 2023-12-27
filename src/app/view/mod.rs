use crate::gui::colors::*;
use nannou::geom::{path, Path};

use super::*;

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = &app.draw();
    let window = app.main_window();
    draw.background().color(BLACK);

    let V2 { x: _width, y: _height } = WINDOW_SIZE;

    let bank_rect = &model.bank_rect;
    draw.rect()
        .xy(bank_rect.xy())
        .wh(bank_rect.wh())
        .color(GREEN);

    // let pre_spectrum_mesh_color = Rgba::new(0.8, 0.8, 0.8, 1.0);
    let pre_spectrum_mesh_color = Rgba::new(0.2, 0.2, 0.2, 1.0);
    // let post_spectrum_line_color = Rgba::new(1.0, 1.0, 1.0, 1.0);
    let post_spectrum_mesh_color = Rgba::new(0.9, 0.4, 0.0, 0.3);
    let line_weight = 2.0;

    let spectrogram_view = model.ui_params.spectrogram_view.lr();

    if matches!(
        spectrogram_view,
        SpectrogramView::PrePost | SpectrogramView::PreOnly
    ) {
        let mut pre_spectrum = model.pre_spectrum_analyzer.borrow_mut();
        let spectrum_rect = pre_spectrum.rect();
        draw.rect()
            .wh(spectrum_rect.wh())
            .xy(spectrum_rect.xy())
            .color(BLACK);

        pre_spectrum.draw(
            draw,
            Some(pre_spectrum_mesh_color),
            line_weight,
            None,
        );

        drop(pre_spectrum);
    }

    if matches!(
        spectrogram_view,
        SpectrogramView::PrePost | SpectrogramView::PostOnly
    ) {
        model.post_spectrum_analyzer.borrow_mut().draw(
            draw,
            None,
            line_weight,
            Some(post_spectrum_mesh_color),
        );
    }

    outline_rect(model.pre_spectrum_analyzer.borrow().rect(), draw, 2.0);

    match model.ui_params.mask_algorithm.lr() {
        GenerativeAlgo::Contours => model
            .contours
            .as_ref()
            .unwrap()
            .read()
            .unwrap()
            .draw(app, draw, &frame),
        GenerativeAlgo::SmoothLife => model
            .smooth_life
            .as_ref()
            .unwrap()
            .read()
            .unwrap()
            .draw(app, draw, &frame),
    }

    model.draw_mask_scan_line(draw);

    model.ui_components.draw(app, draw, &frame);

    // if the frame fails to draw, we'll just ignore it
    let _ = draw.to_frame(app, &frame);
    let _ = model.egui.draw_to_frame(&frame);
}

fn outline_rect(rect: &Rect, draw: &Draw, width: f32) {
    draw.rect()
        .xy(rect.xy())
        .wh(rect.wh())
        .stroke(BG_NON_SELECTED)
        .stroke_weight(width)
        .no_fill();

    // let bl = rect.bottom_left();
    // let br = rect.bottom_right();
    // let tl = rect.top_left();
    // let tr = rect.top_right();
    //
    // draw.line().points(bl, br).weight(width).color(GREY);
    // draw.line().points(br, tr).weight(width).color(GREY);
    // draw.line().points(tr, tl).weight(width).color(GREY);
    // draw.line().points(tl, bl).weight(width).color(GREY);
}
