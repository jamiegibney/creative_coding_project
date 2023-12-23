use super::*;
use nannou::prelude::*;
use nannou::text::Layout;

/// A button UI component, which can act as a trigger or toggle.
///
/// It is recommended that you attach a callback to the button if using it
/// as a trigger.
pub struct Button {
    state: UIComponentState,
    label: Option<String>,

    enabled_text: String,
    enabled_layout: Layout,

    disabled_text: String,
    disabled_layout: Layout,

    label_layout: Layout,

    is_toggle: bool,
    enabled: bool,
    can_be_clicked: bool,

    rect: Rect,

    callback: Option<Box<dyn Fn(bool)>>,
}

impl Button {
    /// Creates a new, default `Button`.
    pub fn new(rect: Rect) -> Self {
        Self {
            state: UIComponentState::Idle,

            label: None,

            enabled_text: String::from("On"),
            enabled_layout: default_text_layout(),

            disabled_text: String::from("Off"),
            disabled_layout: default_text_layout(),

            label_layout: default_text_layout(),

            is_toggle: true,
            enabled: false,
            can_be_clicked: false,

            rect,

            callback: None,
        }
    }

    /* * * CONSTRUCTORS * * */

    /// Provides a label for the button. This will appear above the state
    /// text for toggleable buttons, and will act as the main text for
    /// non-toggleable buttons. The default for non-toggleable buttons is
    /// `"Button"`.
    pub fn with_label(self, label: &str) -> Self {
        Self { label: str_to_option(label), ..self }
    }

    /// Provides a text layout for the button's label.
    pub fn with_label_layout(self, layout: Layout) -> Self {
        Self { label_layout: layout, ..self }
    }

    /// Provides a text layout for the button's enabled state.
    pub fn with_enabled_layout(self, layout: Layout) -> Self {
        Self { enabled_layout: layout, ..self }
    }

    /// Provides a text layout for the button's disabled state.
    pub fn with_disabled_layout(self, layout: Layout) -> Self {
        Self { disabled_layout: layout, ..self }
    }

    /// Sets the enabled text for a toggleable button. The default is `"On"`.
    pub fn with_enabled_text(self, text: &str) -> Self {
        Self { enabled_text: text.to_string(), ..self }
    }

    /// Sets the disabled text for a toggleable button. The default is `"Off"`.
    pub fn with_disabled_text(self, text: &str) -> Self {
        Self { disabled_text: text.to_string(), ..self }
    }

    /// Sets the font size for the whole `Button`.
    pub fn with_font_size(mut self, size: u32) -> Self {
        self.disabled_layout.font_size = size;
        self.enabled_layout.font_size = size;
        self.label_layout.font_size = size;

        self
    }

    /// Sets the initial state of the button. Only applies to toggleable buttons.
    pub fn with_state(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    /// Provides a which is called when the button is pressed, regardless of button
    /// type (toggleable or non-toggleable).
    ///
    /// The argument is the state of the button. This is always `true` for
    /// non-toggleable buttons.
    pub fn with_callback<F>(self, cb: F) -> Self
    where
        F: Fn(bool) + 'static,
    {
        Self { callback: Some(Box::new(cb)), ..self }
    }

    /// Sets whether the button should be toggleable or not. Both types have
    /// different behaviours and appearances — see below.
    ///
    /// # Toggleable buttons
    ///
    /// - Has two states, which are toggled by a button press.
    /// - Configurable string and text layout for each state.
    /// - Optional callback to be called for each button press.
    /// - Optional label, which is drawn above the state text.
    ///
    /// # Non-toggleable buttons
    ///
    /// - Acts as a one-shot trigger when pressed.
    /// - Configurable label string and text layout.
    /// - Optional callback to be called when the button is pressed (recommended).
    pub fn toggleable(self, should_be_toggleable: bool) -> Self {
        Self { is_toggle: should_be_toggleable, ..self }
    }

    /* * * METHODS * * */

    /// Returns a reference to the button's label text layout.
    pub fn label_layout(&self) -> &Layout {
        &self.label_layout
    }

    /// Returns a reference to the button's enabled text layout.
    pub fn enabled_layout(&self) -> &Layout {
        &self.enabled_layout
    }

    /// Returns a reference to the button's disabled text layout.
    pub fn disabled_layout(&self) -> &Layout {
        &self.disabled_layout
    }

    /// Returns a mutable reference to the button's label text layout.
    pub fn label_layout_mut(&mut self) -> &mut Layout {
        &mut self.label_layout
    }

    /// Returns a mutable reference to the button's enabled text layout.
    pub fn enabled_layout_mut(&mut self) -> &mut Layout {
        &mut self.enabled_layout
    }

    /// Returns a mutable reference to the button's disabled text layout.
    pub fn disabled_layout_mut(&mut self) -> &mut Layout {
        &mut self.disabled_layout
    }

    /// Provides a which is called when the button is pressed, regardless of button
    /// type (toggleable or non-toggleable).
    ///
    /// The argument is the state of the button. This is always `true` for
    /// non-toggleable buttons.
    pub fn set_callback<F: Fn(bool) + 'static>(&mut self, cb: F) {
        self.callback = Some(Box::new(cb));
    }
}

impl UIDraw for Button {
    fn should_update(&self, input_data: &InputData) -> bool {
        self.rect.contains(input_data.mouse_pos)
    }

    fn update(&mut self, input_data: &InputData) {
        // should the button be updated?
        if !self.should_update(input_data) {
            self.can_be_clicked = false;
            return;
        }

        let left_clicked = input_data.is_left_clicked;

        // logic to check whether the mouse button is already down
        // when hovering over the button
        if !self.can_be_clicked {
            if left_clicked {
                return;
            }

            self.can_be_clicked = true;
        }

        if self.is_toggle {
            // toggle the button and call the callback, if it exists.
            if left_clicked && !matches!(self.state, UIComponentState::Clicked)
            {
                self.state = UIComponentState::Clicked;
                self.enabled = !self.enabled;

                if let Some(cb) = &self.callback {
                    cb(self.enabled);
                }
            }
            else if !left_clicked
                && matches!(self.state, UIComponentState::Clicked)
            {
                self.state = UIComponentState::Hovered;
            }
        }
        else if input_data.is_left_clicked {
            // if non-toggleable, call the callback when clicked, if it exists.
            if !matches!(self.state, UIComponentState::Clicked) {
                self.state = UIComponentState::Clicked;

                if let Some(cb) = &self.callback {
                    cb(true);
                }
            }
        }
        else {
            self.state = UIComponentState::Hovered;
        }
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        let (x, y, w, h) = self.rect.x_y_w_h();
        let padding = w * 0.05;
        let rect = self.rect;

        if !self.is_toggle {
            let label = self.label.as_deref().unwrap_or("Button");
            let is_clicked = matches!(self.state, UIComponentState::Clicked);

            // background rect
            draw.rect()
                .xy(rect.xy())
                .wh(rect.wh())
                .color(if is_clicked { BG_HOVERED } else { BG_NON_SELECTED });

            let label_rect = rect.pad_bottom(3.5);

            // label
            draw.text(label)
                .xy(label_rect.xy())
                .wh(label_rect.wh())
                .color(VALUE)
                .layout(&self.label_layout);
        }
        else if let Some(label) = self.label.as_ref() {
            let label_rect = rect.pad_bottom(h * 0.5);

            // label
            draw.text(label)
                .xy(label_rect.xy())
                .wh(label_rect.wh())
                .color(LABEL)
                .layout(&self.label_layout);
        }

        if self.is_toggle {
            let value_rect = if self.label.is_some() {
                rect.pad_top(h * 0.5)
            }
            else {
                rect
            };

            // background rect
            draw.rect()
                .xy(value_rect.xy())
                .wh(value_rect.wh())
                .color(BG_NON_SELECTED);

            let value_rect = value_rect.pad_bottom(2.5);

            if self.enabled {
                // enabled text
                draw.text(&self.enabled_text)
                    .xy(value_rect.xy())
                    .wh(value_rect.wh())
                    .color(VALUE)
                    .layout(&self.enabled_layout);
            }
            else {
                // disabled text
                draw.text(&self.disabled_text)
                    .xy(value_rect.xy())
                    .wh(value_rect.wh())
                    .color(VALUE)
                    .layout(&self.disabled_layout);
            }
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}
