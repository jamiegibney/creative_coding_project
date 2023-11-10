use super::*;
use crate::gui;

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(WHITE);

    let pre_spectrum_mesh_color = Rgba::new(0.0, 0.0, 0.0, 0.3);
    let post_spectrum_line_color = Rgba::new(0.2, 0.2, 0.2, 1.0);

    model.pre_spectrum_analyzer.borrow_mut().draw(
        &draw, 
        None, 
        Some(pre_spectrum_mesh_color),
    );

    model.post_spectrum_analyzer.borrow_mut().draw(
        &draw,
        Some(post_spectrum_line_color),
        None,
    );

    draw.to_frame(app, &frame).unwrap();
}
