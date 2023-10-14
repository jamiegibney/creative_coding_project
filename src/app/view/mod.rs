use super::*;

pub fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);
    let _ = draw.to_frame(app, &frame);
}
