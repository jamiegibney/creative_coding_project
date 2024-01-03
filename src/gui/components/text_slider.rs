use std::ops::RangeInclusive;

use super::*;
use nannou::prelude::*;
use nannou::text::Layout;
use std::fmt::Debug;

/// A simple slider with a text readout.
///
/// Holding shift whilst dragging the slider enables 10x fine value control; alt- or
/// cmd-clicking the slider will reset it to its default value, if one is provided.
///
/// Scrolling over the value will also change it.
///
/// **TODO**: implement logarithmic scaling.
/// **TODO**: implement coloring.
/// **TODO**: implement the ability to manually type and parse a value. (yikes)
#[allow(clippy::struct_excessive_bools)]
pub struct TextSlider {
    raw_value: f64,
    output_value: f64,
    default_value: Option<f64>,
    integer_rounding: bool,
    last_int: f64,

    output_range: RangeInclusive<f64>,
    log_scaling: bool,

    label: Option<String>,
    label_layout: Layout,
    value_layout: Layout,
    value_num_chars: usize,
    positive_prefix: bool,

    rect: Rect,

    is_active: bool,
    can_update: bool,
    pub needs_redraw: bool,
    state: UIComponentState,

    prev_mouse_pos: Option<Vec2>,

    value_prefix: Option<String>,
    value_suffix: Option<String>,

    drag_sensitivity: f64,

    callback: Option<Box<dyn Fn(f64, f64)>>,
    formatting_callback: Option<Box<dyn Fn(f64, f64) -> String>>,

    value_text: String,
}

impl TextSlider {
    /// Created a new `TextSlider` with default options.
    pub fn new(value: f64, rect: Rect) -> Self {
        let black = Rgba::new(0.0, 0.0, 0.0, 1.0);

        Self {
            rect,

            raw_value: value,
            output_value: value,
            default_value: None,
            output_range: 0.0..=1.0,

            integer_rounding: false,
            last_int: 0.0,
            log_scaling: false,

            label: None,
            label_layout: default_text_layout(),

            value_layout: default_text_layout(),
            value_num_chars: 5,
            positive_prefix: false,

            value_prefix: None,
            value_suffix: None,
            value_text: String::with_capacity(16),

            is_active: false,
            can_update: false,
            needs_redraw: true,

            prev_mouse_pos: None,
            state: UIComponentState::Idle,
            drag_sensitivity: 0.004,

            callback: None,
            formatting_callback: None,
        }
    }

    /* * * CONSTRUCTORS * * */

    /// Provides the `TextSlider` with a label.
    pub fn with_label(self, label: &str) -> Self {
        Self { label: str_to_option(label), ..self }
    }

    /// Sets the font size for both the label and value readout.
    pub fn with_font_size(mut self, size: u32) -> Self {
        self.set_font_size(size);
        self
    }

    /// Sets the text layout of the label.
    pub fn with_label_layout(self, layout: Layout) -> Self {
        Self { label_layout: layout, ..self }
    }

    /// Sets the text layout of the value readout.
    pub fn with_value_layout(self, layout: Layout) -> Self {
        Self { value_layout: layout, ..self }
    }

    /// Sets the number of chars used to show the value read out. The default value
    /// is `5`. This includes any decimal points, but ignores any prefix or suffix.
    /// Does not apply if the slider uses integer rounding.
    ///
    /// Note that this value is not strictly adhered to, and the text formatting
    /// will always allow the whole value to be shown, and always includes 1 decimal
    /// value if the value would otherwise be truncated at the decimal point.
    ///
    /// See [`with_max_value_num_chars()`](Self::with_max_value_num_chars) to have
    /// use a constant number of chars.
    pub fn with_value_chars(mut self, num_chars: usize) -> Self {
        self.set_value_num_chars(num_chars);
        self.update_output_value();

        self
    }

    /// Sets the number of chars used to show the value readout based on the maximum
    /// value of the slider's range, such that the number of chars in the readout will
    /// never change. Does not apply if the slider uses integer rounding.
    pub fn with_max_value_chars(self) -> Self {
        Self {
            value_num_chars: self.output_range.end().to_string().len() + 2,
            ..self
        }
    }

    /// Sets the output range of the `TextSlider`. The default is `0.0..=1.0`.
    pub fn with_output_range(mut self, range: RangeInclusive<f64>) -> Self {
        let mut s = Self { output_range: range, ..self };
        s.update_output_value();

        s
    }

    /// Provides a default value to the `TextSlider`.
    ///
    /// Must be called after [`with_output_range()`](Self::with_output_range),
    /// if provided.
    pub fn with_default_value(mut self, value: f64) -> Self {
        let mut s =
            Self {
                default_value: Some(value.clamp(
                    *self.output_range.start(),
                    *self.output_range.end(),
                )),
                ..self
            };
        s.reset_to_default();

        s
    }

    /// Enables logarithmic output value scaling for the `TextSlider`.
    pub fn with_log_scaling(self) -> Self {
        unimplemented!();
        Self { log_scaling: true, ..self }
    }

    /// Provides a text prefix for the value readout.
    pub fn with_prefix(self, prefix: &str) -> Self {
        Self { value_prefix: str_to_option(prefix), ..self }
    }

    /// Provides a text suffix for the value readout.
    pub fn with_suffix(self, suffix: &str) -> Self {
        Self { value_suffix: str_to_option(suffix), ..self }
    }

    /// Prefixes positive values with "+".
    pub fn with_positive_value_prefix(self) -> Self {
        Self { positive_prefix: true, ..self }
    }

    /// Provides a callback which is called whenever the `TextSlider`'s value is updated.
    ///
    /// The first argument is the raw slider value, and the second is the output value.
    pub fn with_callback<F>(self, cb: F) -> Self
    where
        F: Fn(f64, f64) + 'static,
    {
        Self { callback: Some(Box::new(cb)), ..self }
    }

    /// Provides a callback to format the value text. Overrides any text-formatting
    /// options passed to the `TextSlider` when provided.
    ///
    /// The first argument is the raw slider value, and the second is the output value.
    pub fn with_formatting_callback<F: Fn(f64, f64) -> String + 'static>(
        mut self,
        cb: F,
    ) -> Self {
        // self = Self { formatting_callback: Some(Box::new(cb)), ..self };
        self.set_formatting_callback(cb);
        self.update_output_value();
        self
    }

    /// Sets the drag sensitivity of the `TextSlider`. The default value is `0.004`.
    pub fn with_sensitivity(self, sensitivity: f64) -> Self {
        Self { drag_sensitivity: sensitivity, ..self }
    }

    /// Uses integer rounding for the output value.
    pub fn with_integer_rounding(mut self) -> Self {
        self.integer_rounding = true;
        self.update_output_value();

        self
    }

    /* * * METHODS * * */

    /// Provides the `TextSlider` with a label.
    pub fn set_label(&mut self, label: &str) {
        self.label = str_to_option(label);
    }

    /// Sets the font size both the label and value readout. If you want to control
    /// these font sizes separately, see the [`label_layout_mut()`](Self::label_layout_mut)
    /// and [`value_layout_mut()`](Self::value_layout_mut) methods.
    pub fn set_font_size(&mut self, size: u32) {
        self.label_layout.font_size = size;
        self.value_layout.font_size = size;
    }

    /// Sets the text layout of the label.
    pub fn set_label_layout(&mut self, layout: Layout) {
        self.label_layout = layout;
    }

    /// Sets the text layout of the value readout.
    pub fn set_value_layout(&mut self, layout: Layout) {
        self.value_layout = layout;
    }

    /// Sets the output range of the `TextSlider`. The default is `0.0..=1.0`.
    pub fn set_output_range(&mut self, range: RangeInclusive<f64>) {
        self.output_range = range;
    }

    /// Sets the default value of the `TextSlider`.
    pub fn set_default_value(&mut self, value: f64) {
        self.default_value = Some(value);
    }

    /// Sets whether to use logarithmic scaling for the output value.
    pub fn log_scaling(&mut self, use_log_scaling: bool) {
        unimplemented!();
        self.log_scaling = use_log_scaling;
    }

    /// Sets the prefix for the value readout.
    pub fn set_prefix(&mut self, prefix: &str) {
        self.value_prefix = str_to_option(prefix);
    }

    /// Sets the suffix for the value readout.
    pub fn set_suffix(&mut self, suffix: &str) {
        self.value_suffix = str_to_option(suffix);
    }

    /// Sets whether to prefix positive values with "+".
    pub fn set_positive_prefix(&mut self, prefix: bool) {
        self.positive_prefix = prefix;
    }

    /// Sets the number of chars used to represent the value read out. The default value is `5`.
    ///
    /// This includes any decimal points, but ignores any prefix or suffix.
    pub fn set_value_num_chars(&mut self, num_chars: usize) {
        self.value_num_chars = num_chars;
    }

    /// Sets the number of chars used to show the value readout based on the maximum
    /// value of the slider's range, such that the number of chars in the readout will
    /// never change. Does not apply if the slider uses integer rounding.
    pub fn set_max_value_chars(&mut self) {
        self.value_num_chars = self.output_range.end().to_string().len() + 2;
    }

    /// Sets the drag sensitivity of the `TextSlider`. The default value is `0.001`.
    pub fn set_sensitivity(&mut self, sensitivity: f64) {
        self.drag_sensitivity = sensitivity;
    }

    /// Provides a callback which is called whenever the `TextSlider`'s value is updated.
    ///
    /// The first argument is the raw slider value, and the second is the output value.
    pub fn set_callback<F: Fn(f64, f64) + 'static>(&mut self, cb: F) {
        self.callback = Some(Box::new(cb));
    }

    /// Detaches any attached callback from the `TextSlider`.
    ///
    /// See [`set_callback()`](Self::set_callback) for more information.
    pub fn detach_callback(&mut self) {
        self.callback = None;
    }

    /// Provides a callback to format the value text. Overrides any text-formatting
    /// options passed to the `TextSlider` when provided.
    pub fn set_formatting_callback<F: Fn(f64, f64) -> String + 'static>(
        &mut self,
        cb: F,
    ) {
        self.formatting_callback = Some(Box::new(cb));
    }

    /// Detaches any attached formatting callback from the `TextSlider`.
    ///
    /// See [`set_formatting_callback()`](Self::set_formatting_callback) for more
    /// information.
    pub fn detach_formatting_callback(&mut self) {
        self.formatting_callback = None;
    }

    /// Uses integer rounding for the output value.
    pub fn set_integer_rounding(&mut self, use_integer_rounding: bool) {
        self.integer_rounding = use_integer_rounding;
    }

    //

    /// Returns a reference to the current label text layout.
    pub fn label_layout(&self) -> &Layout {
        &self.label_layout
    }

    /// Returns a mutable reference to the current label text layout.
    pub fn label_layout_mut(&mut self) -> &mut Layout {
        &mut self.label_layout
    }

    /// Returns a reference to the current value text layout.
    pub fn value_layout(&self) -> &Layout {
        &self.value_layout
    }

    /// Returns a mutable reference to the current value text layout.
    pub fn value_layout_mut(&mut self) -> &mut Layout {
        &mut self.value_layout
    }

    /// Returns whether the `TextSlider` is current active or not — this may act as a way
    /// of checking whether the `TextSlider` should be updated or not.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Returns the output value of the `TextSlider`.
    pub fn value(&self) -> f64 {
        self.output_value
    }

    /// Returns the raw, normalised value of the `TextSlider`.
    pub fn value_raw(&self) -> f64 {
        self.raw_value
    }

    /// Sets the output value of the `TextSlider`.
    pub fn set_value(&mut self, value: f64) {
        let (min, max) = self.min_max();
        self.raw_value = normalize(value.clamp(min, max), min, max);
        self.update_output_value();
    }

    /// Sets the raw value of the `TextSlider`.
    pub fn set_value_raw(&mut self, value: f64) {
        self.raw_value = value.clamp(0.0, 1.0);
        self.update_output_value();
    }

    /// Returns whether `pos` is contained within the bounding rect of the `TextSlider`.
    pub fn within_bounds(&self, pos: Vec2) -> bool {
        self.rect.contains(pos)
    }

    fn reset_to_default(&mut self) {
        if let Some(default) = self.default_value {
            let (min, max) = self.min_max();

            self.raw_value = map_range(default, min, max, 0.0, 1.0);
            self.update_output_value();
        }
    }

    fn min_max(&self) -> (f64, f64) {
        (*self.output_range.start(), *self.output_range.end())
    }

    fn update_output_value(&mut self) {
        self.needs_redraw = true;
        let (min, max) = self.min_max();
        self.output_value = map_range(self.raw_value, 0.0, 1.0, min, max);
        if self.integer_rounding {
            self.output_value = self.output_value.floor();
        }

        if let Some(cb) = self.callback.as_mut() {
            if !(self.integer_rounding
                && epsilon_eq(self.output_value, self.last_int))
            {
                cb(self.raw_value, self.output_value);
            }
        }

        self.value_text = format!(
            "{}{}{}",
            self.value_prefix.as_ref().map_or("", |pre| pre),
            self.format_output_value(),
            self.value_suffix.as_ref().map_or("", |suf| suf),
        );

        if self.integer_rounding {
            self.last_int = self.output_value;
        }
    }

    fn format_output_value(&self) -> String {
        let val = self.output_value;

        if let Some(cb) = &self.formatting_callback {
            return cb(self.raw_value, self.output_value);
        }

        if self.integer_rounding {
            return if self.positive_prefix && val.is_sign_positive() {
                format!("+{val:.0}")
            }
            else {
                format!("{val:.0}")
            };
        }

        let val_str = if self.positive_prefix && val.is_sign_positive() {
            format!("+{val:.10}")
        }
        else {
            format!("{val:.10}")
        };

        let mut decimal_idx = val_str.find('.').unwrap();

        let truncate_to = if decimal_idx == self.value_num_chars - 1 {
            self.value_num_chars + 1
        }
        else if decimal_idx > self.value_num_chars {
            decimal_idx
        }
        else {
            self.value_num_chars
        };

        val_str[..truncate_to].to_string()
    }
}

impl UIDraw for TextSlider {
    fn should_update(&self, input_data: &InputData) -> bool {
        self.within_bounds(input_data.mouse_pos) || self.is_active
    }

    fn update(&mut self, _: &App, input: &InputData) {
        self.needs_redraw = false;
        // guard against the mouse already being clicked when entering the
        // slider's bounding rect
        if !self.within_bounds(input.mouse_pos) && !self.is_active {
            self.can_update = !input.is_left_clicked;
        }
        else if (!self.can_update && !input.is_left_clicked) || self.is_active
        {
            self.can_update = true;
        }

        // should the slider update?
        if !self.should_update(input) || !self.can_update {
            return;
        }

        let y_scr = -input.scroll_delta.y;

        // should the value update based on mouse scrolling?
        if y_scr.abs() >= 1.0 {
            let sensitivity = self.drag_sensitivity
                * if input.is_shift_down { 0.1 } else { 1.0 }
                * 0.4;

            self.raw_value = (y_scr as f64)
                .mul_add(sensitivity, self.raw_value)
                .clamp(0.0, 1.0);

            self.update_output_value();
        }

        // should the slider return to an idle state?
        if !input.is_left_clicked {
            self.prev_mouse_pos = None;
            self.is_active = false;

            return;
        }

        // should the value reset?
        if !self.is_active && input.is_alt_down || input.is_os_mod_down {
            self.reset_to_default();
        }

        self.is_active = true;

        // is the slider being dragged?
        if let Some(prev) = self.prev_mouse_pos {
            let delta = (input.mouse_pos.y - prev.y) as f64;

            // is the drag delta large enough to update?
            if delta.abs() <= f64::EPSILON {
                self.prev_mouse_pos = Some(input.mouse_pos);

                return;
            }

            let sensitivity = self.drag_sensitivity
                * if input.is_shift_down { 0.1 } else { 1.0 };

            self.raw_value =
                delta.mul_add(sensitivity, self.raw_value).clamp(0.0, 1.0);

            self.update_output_value();
        }

        self.prev_mouse_pos = Some(input.mouse_pos);
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        if !self.needs_redraw && frame.nth() > 0 {
            return;
        }

        let (x, y, w, h) = self.rect.x_y_w_h();
        let bl = pt2(x, y);
        let (w2, h2) = (w * 0.5, h * 0.5);
        let mid = vec2(x + w2, y + h2);
        let label_rect = self.rect.shift(pt2(0.0, h + h * 0.1));

        let value_rect = self.label.as_ref().map_or_else(
            || self.rect,
            |label| {
                draw.text(label)
                    .xy(label_rect.xy())
                    .wh(label_rect.wh())
                    .color(LABEL)
                    .layout(&self.label_layout);

                self.rect
            },
        );

        draw.rect()
            .xy(self.rect.xy())
            .wh(self.rect.wh())
            .color(BG_NON_SELECTED);

        let value_rect = value_rect.pad_bottom(2.5);

        draw.text(&self.value_text)
            .xy(value_rect.xy())
            .wh(value_rect.wh())
            .color(VALUE)
            .layout(&self.value_layout);
    }

    fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    fn force_redraw(&mut self, app: &App, draw: &Draw, frame: &Frame) {
        self.needs_redraw = true;
        self.draw(app, draw, frame);
        self.needs_redraw = false;
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}
