use nannou_audio::Buffer;
use std::ops::{Add, Mul, Sub};
use wide::{f32x4, f32x8, f64x2, f64x4};

/// A trait to mark SIMD-compatible types. Covers `f32`, `f64`, and their SIMD vector types
/// available in the `wide` crate: `f32x4`, `f32x8`, and `f64x2`, `f64x4`.
pub trait SimdType:
    Mul<Output = Self> + Sub<Output = Self> + Add<Output = Self> + Copy + Sized
{
    fn from_f64(value: f64) -> Self;
    fn from_f32(value: f32) -> Self;
}

impl SimdType for f32 {
    fn from_f64(value: f64) -> Self {
        value as Self
    }

    fn from_f32(value: f32) -> Self {
        value
    }
}

impl SimdType for f32x4 {
    fn from_f64(value: f64) -> Self {
        Self::splat(value as f32)
    }

    fn from_f32(value: f32) -> Self {
        Self::splat(value)
    }
}

impl SimdType for f32x8 {
    fn from_f64(value: f64) -> Self {
        Self::splat(value as f32)
    }

    fn from_f32(value: f32) -> Self {
        Self::splat(value)
    }
}

impl SimdType for f64 {
    fn from_f64(value: f64) -> Self {
        value
    }

    fn from_f32(value: f32) -> Self {
        value as Self
    }
}

impl SimdType for f64x2 {
    fn from_f64(value: f64) -> Self {
        Self::splat(value)
    }

    fn from_f32(value: f32) -> Self {
        Self::splat(value as f64)
    }
}

impl SimdType for f64x4 {
    fn from_f64(value: f64) -> Self {
        Self::splat(value)
    }

    fn from_f32(value: f32) -> Self {
        Self::splat(value as f64)
    }
}

/// A trait to implement SIMD for processing stereo channels simultaneously.
pub trait SimdBuffer {
    /// Returns a `f64x2` SIMD array from the buffer. Thus, this only works for a buffer
    /// of stereo channels.
    ///
    /// `pos` **must** be less than `buffer.len() - 2`, and must be even.
    fn to_simd_array(&self, pos: usize) -> f64x2;

    /// Unpacks and writes `simd_array` into the buffer at `pos`. Only works for stereo
    /// channels.
    ///
    /// `pos` **must** be less than `buffer.len() - 2`, and must be even.
    fn from_simd_array(&mut self, simd_array: f64x2, pos: usize);
}

impl SimdBuffer for Buffer<f64> {
    fn to_simd_array(&self, pos: usize) -> f64x2 {
        debug_assert!(pos < self.len() - 2 && pos % 2 == 0);

        f64x2::new([self[pos], self[pos + 1]])
    }

    fn from_simd_array(&mut self, simd_array: f64x2, pos: usize) {
        debug_assert!(pos < self.len() - 2 && pos % 2 == 0);

        let arr = simd_array.to_array();

        self[pos] = arr[0];
        self[pos + 1] = arr[1];
    }
}

impl SimdBuffer for Vec<f64> {
    fn to_simd_array(&self, pos: usize) -> f64x2 {
        debug_assert!(pos < self.len() - 2 && pos % 2 == 0);

        f64x2::new([self[pos], self[pos + 1]])
    }

    fn from_simd_array(&mut self, simd_array: f64x2, pos: usize) {
        debug_assert!(pos < self.len() - 2 && pos % 2 == 0);

        let arr = simd_array.to_array();

        self[pos] = arr[0];
        self[pos + 1] = arr[1];
    }
}

impl SimdBuffer for &mut [f64] {
    fn to_simd_array(&self, pos: usize) -> f64x2 {
        debug_assert!(pos < self.len() - 2 && pos % 2 == 0);

        f64x2::new([self[pos], self[pos + 1]])
    }

    fn from_simd_array(&mut self, simd_array: f64x2, pos: usize) {
        debug_assert!(pos < self.len() - 2 && pos % 2 == 0);

        let arr = simd_array.to_array();

        self[pos] = arr[0];
        self[pos + 1] = arr[1];
    }
}
