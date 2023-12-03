use crate::dsp::SpectralMask;
use nannou::prelude::*;

pub trait DrawMask {
    fn update(&mut self, delta_time: f64);
    fn draw(&self, app: &App, draw: &Draw, frame: &Frame);

    fn column_to_mask(&self, mask: &mut SpectralMask, x: f64) {}
    fn row_to_mask(&self, mask: &mut SpectralMask, y: f64) {}
}
