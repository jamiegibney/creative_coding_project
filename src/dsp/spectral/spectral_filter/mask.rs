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
        todo!(
            r#"this needs to allocate the maximum size, and then requires a "working" block size."#
        );
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

    /// Returns the frequency of bin with index `idx`.
    ///
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
