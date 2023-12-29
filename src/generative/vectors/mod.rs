use super::*;
use crate::{
    dsp::{filtering::resonator::resonator_bank::ResoBankData, ResonatorBank},
    prelude::*,
};
use nannou::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Vectors {
    points: Vec<Vec2>,
    point_radius: f32,
    point_color: Rgba,
    num_active_points: usize,

    rect: Rect,
}

impl Vectors {
    pub fn new(num_points: usize, rect: Rect) -> Self {
        let l = rect.left();
        let r = rect.right();
        let b = rect.bottom();
        let t = rect.top();

        let mut s = Self {
            points: (0..num_points).map(|_| Vec2::ZERO).collect(),
            point_radius: 1.0,
            point_color: Rgba::new(1.0, 1.0, 1.0, 1.0),
            num_active_points: num_points,
            rect,
        };

        s.randomize_points();

        s
    }

    pub fn with_point_color(mut self, color: Rgba) -> Self {
        self.set_point_color(color);
        self
    }

    pub fn with_point_radius(mut self, radius: f32) -> Self {
        self.set_point_radius(radius);
        self.randomize_points();
        self
    }

    pub fn with_num_active_points(mut self, num_points: usize) -> Self {
        self.set_num_active_points(num_points);
        self
    }

    pub fn set_point_color(&mut self, color: Rgba) {
        self.point_color = color;
    }

    pub fn set_point_radius(&mut self, radius: f32) {
        if radius.is_sign_positive() {
            self.point_radius = radius;
        }
    }

    pub fn set_num_active_points(&mut self, num_points: usize) {
        self.num_active_points = num_points.min(self.points.len());
    }

    pub fn set_reso_bank_data_mutex(
        &self,
        reso_bank_data: &Arc<Mutex<ResoBankData>>,
    ) {
        if let Ok(mut guard) = reso_bank_data.lock() {
            self.set_reso_bank_data(&mut guard);
        }
    }

    pub fn set_reso_bank_data(&self, reso_bank_data: &mut ResoBankData) {
        let len = self
            .num_active_points
            .min(reso_bank_data.pitches.len())
            .min(reso_bank_data.panning.len());
        let (l, r, b, t) = self.get_rect_dims();

        for i in 0..len {
            let x = self.points[i].x;
            let y = self.points[i].y;

            reso_bank_data.pitches[i] = map_f32(
                y,
                b,
                t,
                ResonatorBank::NOTE_MIN as f32,
                ResonatorBank::NOTE_MAX as f32,
            ) as f64;
            reso_bank_data.panning[i] = map_f32(x, l, r, -1.0, 1.0) as f64;
        }
    }

    pub fn randomize_points(&mut self) {
        let len = self.num_active_points;

        for i in 0..len {
            self.points[i] = self.clamped_vec(random_vector(&self.rect));
        }
    }

    fn clamped_vec(&self, point: Vec2) -> Vec2 {
        let padded = self.rect.pad(self.point_radius + 1.0);
        point.clamp(padded.bottom_left(), padded.top_right())
    }

    /// (left, right, bottom, top)
    fn get_rect_dims(&self) -> (f32, f32, f32, f32) {
        let l = self.rect.left();
        let r = self.rect.right();
        let b = self.rect.bottom();
        let t = self.rect.top();

        (l, r, b, t)
    }
}

impl UIDraw for Vectors {
    fn update(&mut self, app: &App, input_data: &InputData) {
        // todo!()
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        for &point in self.points.iter().take(self.num_active_points) {
            draw.ellipse()
                .color(self.point_color)
                .xy(point)
                .radius(self.point_radius);
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

fn random_vector(rect: &Rect) -> Vec2 {
    let x = random_range(rect.left(), rect.right());
    let y = random_range(rect.bottom(), rect.top());
    Vec2::new(x, y)
}
