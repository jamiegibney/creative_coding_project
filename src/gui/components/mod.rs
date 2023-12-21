pub mod button;
pub mod menu;
pub mod text_slider;

use super::colors::*;
use super::*;

use nannou::prelude::*;
use nannou::text::{Align, Justify, Layout};

#[derive(Clone, Copy, Debug, Default)]
pub enum UIComponentState {
    #[default]
    Idle,
    Hovered,
    Disabled,
    Clicked,
}

pub fn default_layout() -> Layout {
    Layout {
        line_spacing: 0.0,
        line_wrap: None,
        justify: Justify::Center,
        font_size: 20,
        font: None,
        y_align: Align::Middle,
    }
}

pub fn str_to_option(s: &str) -> Option<String> {
    (!s.is_empty()).then_some(s.to_string())
}
