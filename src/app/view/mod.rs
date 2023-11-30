use nannou::geom::{path, Path};

use super::*;

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = &app.draw();
    let window = app.main_window();
    if frame.nth() < 2 {}
    draw.background().color(WHITE);

    let V2 { x: _width, y: _height } = WINDOW_SIZE;

    // if PRINT_DSP_LOAD {
    //
    // }

    let pre_spectrum_mesh_color = Rgba::new(0.0, 0.0, 0.0, 0.3);
    // let post_spectrum_line_color = Rgba::new(0.2, 0.2, 0.2, 1.0);
    let post_spectrum_mesh_color = Rgba::new(0.0, 1.0, 0.0, 0.784);

    let mut pre_spectrum = model.pre_spectrum_analyzer.borrow_mut();
    let spectrum_rect = pre_spectrum.rect();
    draw.rect()
        .wh(spectrum_rect.wh())
        .xy(spectrum_rect.xy())
        .color(WHITE);

    pre_spectrum.draw(draw, Some(pre_spectrum_mesh_color), None);

    drop(pre_spectrum);

    model.post_spectrum_analyzer.borrow_mut().draw(
        draw,
        None,
        Some(post_spectrum_mesh_color),
    );

    outline_rect(model.pre_spectrum_analyzer.borrow().rect(), draw, 2.0);

    // model.flow_field.draw(window.device(), draw, &frame);
    model.contours.draw(app, draw, &frame);
    model.draw_mask_scan_line(draw);

    outline_rect(model.contours.rect(), draw, 2.0);

    // if the frame fails to draw, we'll just ignore it
    let _ = draw.to_frame(app, &frame);
}

fn outline_rect(rect: &Rect, draw: &Draw, width: f32) {
    let bl = rect.bottom_left();
    let br = rect.bottom_right();
    let tl = rect.top_left();
    let tr = rect.top_right();

    draw.line().points(bl, br).weight(width).color(GREY);
    draw.line().points(br, tr).weight(width).color(GREY);
    draw.line().points(tr, tl).weight(width).color(GREY);
    draw.line().points(tl, bl).weight(width).color(GREY);
}
