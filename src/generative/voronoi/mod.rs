//! Voronoi noise types and implementation.

use crate::generative::VectorField;
use crate::prelude::*;
use nannou::image::{ImageBuffer, Rgba};
use nannou::prelude::*;
use nannou::wgpu;

const VORONOI_SHADER_X_THREADS: u32 = 16;
const VORONOI_SHADER_Y_THREADS: u32 = 16;
const MAX_NUM_POINTS: usize = 32;

mod compute;
pub mod general;

pub use general::VoronoiGPU;

/// 2D point representation compatible with the shader.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    pub fn from_vec2(v2: Vec2) -> Self {
        Self { x: v2.x, y: v2.y }
    }

    pub fn copy_from_vec2(&mut self, v2: Vec2) {
        self.x = v2.x;
        self.y = v2.y;
    }

    pub fn sqr_dist(self, other: Self) -> f32 {
        let dy = self.y - other.y;
        let dx = self.x - other.x;
        dx.mul_add(dx, dy * dy)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Points {
    points: [Point; MAX_NUM_POINTS],
}

impl Points {
    pub fn new() -> Self {
        Self { points: [Point::from_vec2(Vec2::ZERO); MAX_NUM_POINTS] }
    }

    pub fn copy_from_vectors(&mut self, vectors: &VectorField) {
        debug_assert!(vectors.points.capacity() >= MAX_NUM_POINTS);
        let tr = vectors.rect().top_right();

        for i in 0..MAX_NUM_POINTS {
            let point = Vec2::new(
                256.0 - (tr.x - vectors.points[i].pos.x),
                tr.y - vectors.points[i].pos.y,
            );

            self.points[i].copy_from_vec2(point);
            // self.points[i].copy_from_vec2(tr - vectors.points[i].pos);
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe { wgpu::bytes::from(self) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct VoronoiStateGeneral {
    active_cells: u32,
    weight: f32,
    width: u32,
    height: u32,
}

impl VoronoiStateGeneral {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { wgpu::bytes::from(self) }
    }
}
