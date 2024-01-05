//! Window event callback.

use super::Model;
use crate::prelude::*;
use nannou::prelude::*;

/// The window's event callback.
pub fn event(app: &App, model: &mut Model, win_event: WindowEvent) {
    model.input_data.scroll_delta = Vec2::ZERO;

    match win_event {
        MouseWheel(MouseScrollDelta::PixelDelta(pos), _) => {
            model.input_data.scroll_delta = vec2(pos.x as f32, pos.y as f32);
        }
        Focused => model.input_data.is_win_focussed = true,
        Unfocused => model.input_data.is_win_focussed = false,
        _ => {}
    }
}
