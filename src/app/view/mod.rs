use super::*;



pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = &app.draw();
    let window = app.main_window();
    if frame.nth() < 2 {
        draw.background().color(WHITE);
    }

    let V2 { x: _width, y: _height } = WINDOW_SIZE;

    // if PRINT_DSP_LOAD {
    //
    // }

    let _pre_spectrum_mesh_color = Rgba::new(0.0, 0.0, 0.0, 0.3);
    let _post_spectrum_line_color = Rgba::new(0.2, 0.2, 0.2, 1.0);

    // model.pre_spectrum_analyzer.borrow_mut().draw(
    //     draw,
    //     // None,
    //     Some(pre_spectrum_mesh_color),
    //     None,
    // );
    //
    // model.post_spectrum_analyzer.borrow_mut().draw(
    //     draw, None, // Some(post_spectrum_line_color),
    //     None,
    // );

    // model.flow_field.draw(window.device(), draw, &frame);
    model.contours.draw(window.device(), draw, &frame);
    draw.line()
        .points(vec2(0.0, -128.0), vec2(0.0, 128.0))
        .weight(4.0)
        .color(rgba::<u8>(0, 200, 0, 100));

    // if the frame fails to draw, we'll just ignore it
    let _ = draw.to_frame(app, &frame);
}
