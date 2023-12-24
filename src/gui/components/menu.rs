#![allow(clippy::suboptimal_flops)]
use super::*;

/// A menu UI component, which essentially exposes an `enum` as a menu.
///
/// Requires that the `enum` implements the [`MenuEnum`] trait.
#[allow(clippy::struct_excessive_bools)]
pub struct Menu<E: MenuEnum> {
    rect: Rect,

    label: Option<String>,
    label_layout: Layout,

    current_item_name: String,

    item_names: Vec<String>,
    item_text_layout: Layout,
    item_rects: Vec<Rect>,

    selected_item_text_layout: Option<Layout>,

    hovered_idx: Option<usize>,

    is_open: bool,
    was_just_closed: bool,
    can_open: bool,
    can_update_state: bool,
    state: UIComponentState,

    callback: Option<Box<dyn Fn(E)>>,

    variant: E,
}

impl<E: MenuEnum> Menu<E> {
    /// Creates a new, default `Menu`.
    pub fn new(rect: Rect) -> Self {
        let num_variants = E::num_variants();

        let mut s = Self {
            rect,

            label: None,
            label_layout: default_text_layout(),

            current_item_name: String::with_capacity(16),

            item_names: Self::variant_names(),
            item_text_layout: default_text_layout(),
            item_rects: Self::divide_rect(&rect, num_variants),

            selected_item_text_layout: None,

            hovered_idx: None,

            is_open: false,
            was_just_closed: false,
            can_open: false,
            can_update_state: false,
            state: UIComponentState::Idle,

            callback: None,

            variant: E::default(),
        };

        s.update_current_name();
        s
    }

    /* * * CONSTRUCTORS * * */

    /// Provides the `Menu` with a label.
    pub fn with_label(self, label: &str) -> Self {
        Self { label: str_to_option(label), ..self }
    }

    /// Sets the label's text layout.
    pub fn with_label_layout(self, layout: Layout) -> Self {
        Self { label_layout: layout, ..self }
    }

    /// Sets the font size for the menu label.
    pub fn with_label_font_size(mut self, size: u32) -> Self {
        self.label_layout.font_size = size;
        self
    }

    /// Sets each item's text layout.
    pub fn with_item_text_layout(self, layout: Layout) -> Self {
        Self { item_text_layout: layout, ..self }
    }

    /// Sets the font size for the item labels.
    pub fn with_item_font_size(mut self, size: u32) -> Self {
        self.item_text_layout.font_size = size;
        if let Some(layout) = &mut self.selected_item_text_layout {
            layout.font_size = size;
        }
        self
    }

    /// Sets the font size for the whole `Menu`.
    pub fn with_font_size(self, size: u32) -> Self {
        self.with_item_font_size(size).with_label_font_size(size)
    }

    /// Provides a text layout for the selected item when the menu is open.
    pub fn with_selected_item_text_layout(self, layout: Layout) -> Self {
        Self { selected_item_text_layout: Some(layout), ..self }
    }

    /// Provides a callback to the menu, which is called whenever a new item is selected.
    ///
    /// The callback's argument is the current variant of the menu.
    pub fn with_callback<F>(self, callback: F) -> Self
    where
        F: Fn(E) + 'static,
    {
        Self { callback: Some(Box::new(callback)), ..self }
    }

    /// Creates the menu in its open state.
    pub fn starts_open(self) -> Self {
        Self { is_open: true, ..self }
    }

    /// Sets the initial item variant for the menu.
    pub fn initial_variant(mut self, variant: E) -> Self {
        self.variant = variant;
        self.update_current_name();

        self
    }

    /* * * METHODS * * */

    /// Sets a callback, which is called whenever a new item is selected.
    ///
    /// The callback's argument is the current variant of the menu.
    pub fn set_callback<F: Fn(E) + 'static>(&mut self, callback: F) {
        self.callback = Some(Box::new(callback));
    }

    /// Returns the current item of the menu.
    pub fn output(&self) -> E {
        self.variant
    }

    /// Returns a reference to the menu's label text layout.
    pub fn label_layout(&self) -> &Layout {
        &self.label_layout
    }

    /// Returns a reference to the menu's item text layout.
    pub fn item_text_layout(&self) -> &Layout {
        &self.item_text_layout
    }

    /// Returns a reference to the menu's selected item text layout.
    pub fn selected_text_layout(&self) -> Option<&Layout> {
        self.selected_item_text_layout.as_ref()
    }

    /// Returns a mutable reference to the menu's label text layout.
    pub fn label_layout_mut(&mut self) -> &mut Layout {
        &mut self.label_layout
    }

    /// Returns a mutable reference to the menu's item text layout.
    pub fn item_text_layout_mut(&mut self) -> &mut Layout {
        &mut self.item_text_layout
    }

    /// Returns a mutable reference to the menu's selected item text layout.
    pub fn selected_text_layout_mut(&mut self) -> Option<&mut Layout> {
        self.selected_item_text_layout.as_mut()
    }

    fn reset_to_default(&mut self) {
        self.variant = E::default();

        self.update_current_name();

        if let Some(cb) = &self.callback {
            cb(self.variant);
        }
    }

    fn divide_rect(rect: &Rect, num_variants: usize) -> Vec<Rect> {
        let h = rect.h();
        let cell_height = h / num_variants as f32;

        (0..num_variants)
            .rev()
            .map(|i| {
                let off = h * 0.5 - cell_height * 0.5 - rect.y();
                let y = (i as f32) * cell_height - off;

                Rect::from_xy_wh(pt2(rect.x(), y), pt2(rect.w(), cell_height))
            })
            .collect()
    }

    fn variant_names() -> Vec<String> {
        let num_variants = E::num_variants();

        (0..num_variants)
            .map(|i| {
                format!(
                    "{}",
                    E::from_idx(i)
                        .unwrap_or_else(|| {
                            panic!(
                                "exceeded the number of enum variants: please update the num_variants() method for enum with default \"{:#?}\"",
                                E::default())
                        })
                )
            })
            .collect()
    }

    fn update_current_name(&mut self) {
        self.current_item_name = format!("{}", self.variant);
    }

    fn contains_mouse(&self, mouse_pos: Vec2) -> bool {
        self.rect.contains(mouse_pos)
    }
}

impl<E: MenuEnum> UIDraw for Menu<E> {
    fn should_update(&self, input: &InputData) -> bool {
        (self.contains_mouse(input.mouse_pos) && input.is_left_clicked)
            || self.is_open
    }

    fn update(&mut self, input: &InputData) {
        // guard against the mouse already being clicked when entering the
        // menu's bounding rect
        if !self.item_rects[0].contains(input.mouse_pos) {
            self.can_open = !input.is_left_clicked;
        }
        else if !self.can_open && !input.is_left_clicked {
            self.can_open = true;
        }

        // should the menu be updated?
        if !self.should_update(input) {
            if !input.is_left_clicked && self.was_just_closed {
                self.was_just_closed = false;
            }

            self.state = UIComponentState::Idle;

            return;
        }

        // should the menu close if it is open?
        if !self.contains_mouse(input.mouse_pos)
            && self.hovered_idx.is_none()
            && input.is_left_clicked
        {
            self.is_open = false;
        }

        // should the menu open if it is closed?
        if !self.is_open
            && self.item_rects[0].contains(input.mouse_pos)
            && input.is_left_clicked
            && !self.was_just_closed
            && self.can_open
        {
            if input.is_alt_down || input.is_os_mod_down {
                self.reset_to_default();
                return;
            }

            self.can_update_state = false;
            self.is_open = true;
        }

        // should the selected item change?
        if self.is_open {
            let btm = self.rect().bottom();
            let win_btm = (-WINDOW_SIZE.y / 2.0) as f32;
            let shift = if btm < win_btm { win_btm - btm } else { 0.0 };

            for (i, rect) in self.item_rects.iter().enumerate() {
                let rect = rect.shift_y(shift);
                let contains = rect.contains(input.mouse_pos);

                if input.is_left_clicked {
                    if contains && self.can_update_state {
                        self.variant = E::from_idx(i).unwrap();

                        self.update_current_name();

                        if let Some(cb) = &self.callback {
                            cb(self.variant);
                        }

                        self.can_update_state = false;
                        self.is_open = false;
                        self.was_just_closed = true;

                        break;
                    }

                    self.hovered_idx = None;
                }
                else if contains {
                    self.hovered_idx = Some(i);
                }
            }
            if !input.is_left_clicked {
                self.can_update_state = true;
                // self.hovered_idx = None;
            }
        }
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        let top_rect = &self.item_rects[0];
        let label_rect =
            top_rect.shift(pt2(0.0, top_rect.h() + top_rect.h() * 0.1));

        // label
        if let Some(label) = &self.label {
            draw.text(label)
                .xy(label_rect.xy())
                .wh(label_rect.wh())
                .layout(&self.label_layout)
                .color(LABEL);
        }

        if self.is_open {
            let selected_idx = self.variant.idx();
            let btm = self.rect().bottom();
            let win_btm = (-WINDOW_SIZE.y / 2.0) as f32;
            let shift = if btm < win_btm { win_btm - btm } else { 0.0 };

            for (i, rect) in self.item_rects.iter().enumerate() {
                let is_selected = i == selected_idx;
                let rect = rect.shift_y(shift);

                let (text_color, rect_color) = self.hovered_idx.map_or(
                    if is_selected {
                        (SELECTED, BG_SELECTED)
                    }
                    else {
                        (NON_SELECTED, BG_NON_SELECTED)
                    },
                    |idx| {
                        if is_selected {
                            (SELECTED, BG_SELECTED)
                        }
                        else if i == idx {
                            (HOVERED, BG_HOVERED)
                        }
                        else {
                            (NON_SELECTED, BG_NON_SELECTED)
                        }
                    },
                );

                /// background rect
                draw.rect().xy(rect.xy()).wh(rect.wh()).color(rect_color);

                let text_rect =
                    rect.pad_left(rect.w() * 0.02).pad_bottom(rect.h() * 0.15);

                /// item label
                draw.text(&self.item_names[i])
                    .xy(text_rect.xy())
                    .wh(text_rect.wh())
                    .layout(if is_selected {
                        self.selected_item_text_layout
                            .as_ref()
                            .unwrap_or(&self.item_text_layout)
                    }
                    else {
                        &self.item_text_layout
                    })
                    .color(text_color);
            }
        }
        else {
            let rect = &self.item_rects[0];

            let text_rect =
                rect.pad_left(rect.w() * 0.02).pad_bottom(rect.h() * 0.15);

            draw.rect()
                .xy(rect.xy())
                .wh(rect.wh())
                .color(BG_NON_SELECTED);

            // selected item when collapsed
            draw.text(&self.current_item_name)
                .xy(text_rect.xy())
                .wh(text_rect.wh())
                .layout(
                    self.selected_item_text_layout
                        .as_ref()
                        .unwrap_or(&self.item_text_layout),
                )
                .color(VALUE);
        }
    }

    fn draw_bounding_rect(&self, draw: &Draw) {
        draw.rect()
            .wh(self.rect.wh())
            .xy(self.rect.xy())
            .stroke(RED)
            .stroke_weight(1.0)
            .no_fill();

        for rect in &self.item_rects {
            draw.rect()
                .wh(rect.wh())
                .xy(rect.xy())
                .stroke(GREEN)
                .stroke_weight(1.0)
                .no_fill();
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}
