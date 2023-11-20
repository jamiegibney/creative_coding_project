//! Spectral frequency masking.

use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct SpectralMask {
    points: Vec<f64>,
}

impl Deref for SpectralMask {
    type Target = Vec<f64>;

    fn deref(&self) -> &Self::Target {
        &self.points
    }
}

impl DerefMut for SpectralMask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.points
    }
}

impl SpectralMask {
    /// # Panics
    ///
    /// Panics if `order > 14`.
    ///
    /// # Orders
    ///
    /// `0 == 1`
    /// `1 == 2`
    /// `2 == 4`
    /// `3 == 8`
    /// `4 == 16`
    /// `5 == 32`
    /// `6 == 64`
    /// `7 == 128`
    /// `8 == 256`
    /// `9 == 512`
    /// `10 == 1,024`
    /// `11 == 2,048`
    /// `12 == 4,096`
    /// `13 == 8,192`
    /// `14 == 16,384`
    pub fn new(order: u32) -> Self {
        assert!(order <= 14); // 16,384 max elements

        Self { points: vec![1.0; 2usize.pow(order)] }
    }

    /// # Panics
    ///
    /// Panics if `vec` is empty.
    pub fn from_vec(vec: Vec<f64>) -> Self {
        assert!(!vec.is_empty());

        Self { points: vec }
    }

    /// # Panics
    ///
    /// Panics if `vec` is empty.
    pub fn from_vec_cloned(vec: &[f64]) -> Self {
        assert!(!vec.is_empty());

        Self { points: Vec::from(vec) }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { points: Vec::with_capacity(capacity) }
    }

    /// # Panics
    ///
    /// Panics if idx is greater than the size of the mask.
    pub fn bin_freq(&self, idx: usize, sample_rate: f64) -> f64 {
        assert!(idx <= self.points.len());
        let size = self.points.len() as f64;
        let k = idx as f64;
        let nyquist = sample_rate / 2.0;

        k * (nyquist / size)
    }
}

// #[derive(Clone)]
// pub struct SpectralMask {
//     frames: Vec<SpectralFrame>,
// }
//
// impl SpectralMask {
//     /// Creates a new spectral mask with `num_frames` frames, where each frame has
//     /// `2 ^ frame_size_order` elements.
//     ///
//     /// # Panics
//     ///
//     /// Panics if `frame_size_order > 14` (`16,384` elements).
//     ///
//     /// # Orders
//     ///
//     /// `0 == 1`
//     /// `1 == 2`
//     /// `2 == 4`
//     /// `3 == 8`
//     /// `4 == 16`
//     /// `5 == 32`
//     /// `6 == 64`
//     /// `7 == 128`
//     /// `8 == 256`
//     /// `9 == 512`
//     /// `10 == 1,024`
//     /// `11 == 2,048`
//     /// `12 == 4,096`
//     /// `13 == 8,192`
//     /// `14 == 16,384`
//     pub fn new(frame_size_order: u32, num_frames: usize) -> Self {
//         Self { frames: vec![SpectralFrame::new(frame_size_order); num_frames] }
//     }
//
//     /// # Panics
//     ///
//     /// Panics if `frame_size_order > 14`.
//     pub fn resize(&mut self, frame_size_order: u32, num_frames: usize) {
//         self.frames
//             .resize(num_frames, SpectralFrame::new(frame_size_order));
//     }
//
//     pub fn num_frames(&self) -> usize {
//         self.frames.len()
//     }
//
//     pub fn frame_size(&self) -> usize {
//         // returns a number if no frames exist instead of Option
//         self.frames.first().map_or(0, |frame| frame.len())
//     }
// }
//
// impl Deref for SpectralMask {
//     type Target = Vec<SpectralFrame>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.frames
//     }
// }
//
// impl DerefMut for SpectralMask {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.frames
//     }
// }
