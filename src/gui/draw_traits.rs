use crate::dsp::SpectralMask;
use nannou::prelude::*;
use std::sync::Arc;

/// Commonly-accessed mouse data:
///
/// - Mouse position
/// - Mouse scroll delta
/// - LMB down state
#[derive(Clone, Copy, Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct InputData {
    /// The position of the mouse.
    pub mouse_pos: Vec2,
    /// The amount scrolled since the last frame.
    pub scroll_delta: Vec2,
    /// Whether the left mouse button is clicked.
    pub is_left_clicked: bool,
    /// Whether the right mouse button is clicked.
    pub is_right_clicked: bool,

    /// The time in seconds since the last frame.
    ///
    /// *(More specifically, the time in seconds since the last call to the app's
    /// [`update()`](crate::app::update::update) callback.)*
    pub delta_time: f64,

    /// Whether the OS modifier key is down (command on MacOS).
    pub is_os_mod_down: bool,
    /// Whether the shift modifier is down.
    pub is_shift_down: bool,
    /// Whether the alt modifier is down.
    pub is_alt_down: bool,
    /// Whether the control modifier is down.
    pub is_ctrl_down: bool,
}

/// Trait for UI components which can be drawn.
// NOTE: a required "new" method would be sensible, but in this situation
// not all UI components have the same constructor requirements as some of them
// need access to WGPU stuff upon construction.
pub trait UIDraw {
    /// The component's update method, to be used in the app's
    /// [`update()`](crate::app::update::update) callback.
    ///
    /// This method should update any internal state based on the time and
    /// input events.
    fn update(&mut self, app: &App, input_data: &InputData);

    /// The component's draw loop, to be called in the app's
    /// [`view()`](crate::app::view::view) callback.
    ///
    /// This method should only draw the component, not update any state.
    fn draw(&self, app: &App, draw: &Draw, frame: &Frame);

    /// An optional method for querying whether the component should be updated or not.
    ///
    /// To defer control to the component, it is recommended that this method is used
    /// as a return guard in the component's [`update()`](UIDraw::update) method.
    fn should_update(&self, mouse_data: &InputData) -> bool {
        true
    }

    /// A method for drawing the component's bounding rect. Useful for debugging.
    ///
    /// By default, this will draw an outline of the rect returned by the
    /// [`rect()`](UIDraw::rect) method.
    fn draw_bounding_rect(&self, draw: &Draw) {
        let rect = self.rect();
        draw.rect()
            .wh(rect.wh())
            .xy(rect.xy())
            .stroke(GREEN)
            .stroke_weight(1.0)
            .no_fill();
    }

    /// Returns a reference to the component's bounding rect.
    fn rect(&self) -> &Rect;
}

/// Trait for UI components which act as spectral masks.
pub trait DrawMask: UIDraw {
    fn column_to_mask(&self, mask: &mut SpectralMask, x: f64) {}
    fn row_to_mask(&self, mask: &mut SpectralMask, y: f64) {}
}
