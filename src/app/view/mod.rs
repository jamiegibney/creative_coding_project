use super::*;
use crate::gui;

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    model.pre_spectrum_analyzer.borrow_mut().draw(&draw);

    let _ = draw.to_frame(app, &frame);
}
