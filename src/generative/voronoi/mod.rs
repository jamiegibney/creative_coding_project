//! Voronoi (Worley) noise algorithms.
use nannou::noise::{NoiseFn, RangeFunction, Worley};
use nannou::prelude::*;

pub struct Voronoi {
    win_rect: Rect,
    worley: Worley,
}

impl Voronoi {
    pub fn new(win_rect: Rect) -> Self {
        Self {
            win_rect,
            worley: Worley::new()
                .set_range_function(RangeFunction::EuclideanSquared),
        }
    }

    pub fn update(&mut self) {
        // self.worley.get
    }
}
