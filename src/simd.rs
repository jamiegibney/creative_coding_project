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
