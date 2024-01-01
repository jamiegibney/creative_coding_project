use super::*;
use crate::{
    dsp::{ResoBankData, ResonatorBank},
    prelude::*,
};
use nannou::prelude::*;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

const MAX_VELOCITY: f32 = 3.0;
const MIN_START_VELOCITY: f32 = 1.5;

#[derive(Debug, Clone)]
pub struct Point {
    pub pos: Vec2,            // position of the point
    pub vel: Vec2,                // velocity of the point - speed and direction
    deceleration_factor: f32, // how much the point decelerates each frame
}

impl Point {
    pub fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub fn randomize_velocity(&mut self) {
        self.vel.x = random_range(-MAX_VELOCITY, MAX_VELOCITY) * 0.7;
        self.vel.y = random_range(-MAX_VELOCITY, MAX_VELOCITY);
    }

    pub fn randomize_deceleration(&mut self) {
        self.deceleration_factor = random_range(0.90, 0.98);
    }

    pub fn contains(&self, pos: Vec2, radius: f32) -> bool {
        let size = radius * 2.0;
        Rect::from_xy_wh(self.pos, pt2(size, size)).contains(pos)
    }
}

#[derive(Debug, Clone)]
pub struct Vectors {
    pub points: Vec<Point>,
    point_radius: f32,
    point_color: Rgba,
    pub num_active_points: usize,
    deceleration_scale: f32,

    pub can_mouse_interact: bool,
    points_overriden: bool,

    clicked_idx: Option<usize>,

    rect: Rect,
}

impl Vectors {
    pub fn new(num_points: usize, rect: Rect) -> Self {
        let mid = rect.xy();

        let mut s = Self {
            points: (0..num_points)
                .map(|_| Point {
                    pos: mid,
                    vel: Vec2::ZERO,
                    deceleration_factor: 1.0,
                })
                .collect(),
            point_radius: 1.0,
            point_color: Rgba::new(1.0, 1.0, 1.0, 1.0),
            num_active_points: num_points,
            deceleration_scale: 1.0,
            can_mouse_interact: true,
            points_overriden: false,
            clicked_idx: None,
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

    pub fn override_points(&mut self) -> &mut [Point] {
        self.points_overriden = true;

        &mut self.points[..self.num_active_points]
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
            let x = self.points[i].pos.x;
            let y = self.points[i].pos.y;

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
            let pos = self.clamped_vec(random_vector(&self.rect));

            self.points[i].set_pos(pos);
            self.points[i].randomize_deceleration();
            self.points[i].randomize_velocity();
        }
    }

    pub fn push_points(&mut self) {
        let len = self.num_active_points;

        for i in 0..len {
            let new_vel = Vec2::new(
                random_range(-MAX_VELOCITY, MAX_VELOCITY) * 0.7,
                random_range(-MAX_VELOCITY, MAX_VELOCITY),
            );

            self.points[i].vel += new_vel;
        }
    }

    pub fn set_friction(&mut self, friction: f64) {
        self.deceleration_scale = (1.0 - friction * 0.09) as f32;
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
        let len = self.num_active_points;

        if self.points_overriden {
            self.points_overriden = false;
            return;
        }

        if !input_data.is_left_clicked {
            self.clicked_idx = None;

            if self.rect.contains(input_data.mouse_pos) {
                self.can_mouse_interact = true;
            }
        }
        else if input_data.is_left_clicked
            && !self.rect.contains(input_data.mouse_pos)
            && self.clicked_idx.is_none()
        {
            self.can_mouse_interact = false;
        }

        for i in 0..len {
            if self.clicked_idx.is_none()
                && input_data.is_left_clicked
                && self.points[i]
                    .contains(input_data.mouse_pos, self.point_radius)
            {
                self.clicked_idx = Some(i);
            }

            if let Some(idx) = self.clicked_idx {
                if idx == i && self.can_mouse_interact {
                    self.points[i].pos = input_data.mouse_pos;
                }
            }

            let decel =
                self.points[i].deceleration_factor * self.deceleration_scale;

            self.points[i].pos =
                self.clamped_vec(self.points[i].pos + self.points[i].vel);

            self.points[i].vel *= decel;

            if self.points[i].vel.x < 0.04 && self.points[i].vel.y < 0.04 {
                self.points[i].vel = Vec2::ZERO;
            }
        }
    }

    fn draw(&self, app: &App, draw: &Draw, frame: &Frame) {
        for point in self.points.iter().take(self.num_active_points) {
            draw.ellipse()
                .color(self.point_color)
                .xy(point.pos)
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
