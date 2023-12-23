use super::*;
use nannou::{prelude::*, text::Layout};

pub struct Label {
    rect: Rect,
    text: String,
    text_layout: Layout,
    text_color: Rgb,
}

impl Label {
    const DEFAULT_TEXT: &'static str = "Label";

    /// Creates a new, default `Label`.
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            text: String::from(Self::DEFAULT_TEXT),
            text_layout: default_text_layout(),
            text_color: BIG_LABEL,
        }
    }

    /// Provides the label text for the `Label`.
    pub fn with_text(self, text: &str) -> Self {
        Self {
            text: if text.is_empty() {
                String::from(Self::DEFAULT_TEXT)
            } else {
                String::from(text)
            },
            ..self
        }
    }

    /// Provides a color for the label text.
    pub fn with_text_color(self, text_color: Rgb) -> Self {
        Self { text_color, ..self }
    }

    /// Provides a text layout for the label text.
    pub fn with_text_layout(self, text_layout: Layout) -> Self {
        Self {
            text_layout,
            ..self
        }
    }

    /// Sets the label text.
    pub fn set_text(&mut self, text: &str) {
        if !text.is_empty() {
            self.text = String::from(text);
        }
    }

    /// Sets the label text color.
    pub fn set_text_color(&mut self, text_color: Rgb) {
        self.text_color = text_color;
    }

    /// Sets the label text layout.
    pub fn set_text_layout(&mut self, text_layout: Layout) {
        self.text_layout = text_layout;
    }
}

impl UIDraw for Label {
    fn update(&mut self, _: &InputData) {
        eprintln!("redundant call to Label \"update()\" (as UIDraw) method: Label does not have an update loop!");
    }

    fn draw(&self, _: &App, draw: &Draw, _: &Frame) {
        draw.text(&self.text)
            .wh(self.rect.wh())
            .xy(self.rect.xy())
            .color(self.text_color)
            .layout(&self.text_layout);
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}
