use super::Model;
use crate::prelude::*;
use nannou::prelude::*;

pub fn event(app: &App, model: &mut Model, win_event: WindowEvent) {
    model.input_data.scroll_delta = Vec2::ZERO;

    if let MouseWheel(MouseScrollDelta::PixelDelta(pos), _) = win_event {
        model.input_data.scroll_delta = vec2(pos.x as f32, pos.y as f32);
    }
}
