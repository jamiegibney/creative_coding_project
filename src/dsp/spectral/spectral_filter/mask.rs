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
    /// Creates a new `SpectralMask` with `max_size` capacity.
    ///
    /// Note that in order to have usable elements, you need to call the
    /// [`with_size`](Self::with_size) constructor after this call.
    ///
    /// # Panics
    ///
    /// Panics if `max_size` is not a power-of-two value, or if it is greater
    /// than 2^14 (16,384).
    pub fn new(max_size: usize) -> Self {
        assert!(max_size.is_power_of_two() && max_size <= 1 << 14);

        Self { points: Vec::with_capacity(max_size) }
    }

    /// Sets the "working size" of the mask.
    ///
    /// # Panics
    ///
    /// Panics if `size` is not a power-of-two value, or is greater than 
    /// `self.max_size()`.
    pub fn with_size(mut self, size: usize) -> Self {
        assert!(size.is_power_of_two() && size <= self.max_size());

        self.points.resize(size, 0.0);
        self
    }

    /// Sets the "working size" of the mask. This will not allocate.
    ///
    /// # Panics
    ///
    /// Panics if `size` is not a power-of-two value, or if it is greater than
    /// the maximum size set when the mask was created.
    pub fn set_mask_size(&mut self, size: usize) {
        assert!(size.is_power_of_two() && size <= self.points.capacity());

        self.points.resize(size, 0.0);
    }

    /// Returns the maximum size of the mask.
    pub fn max_size(&self) -> usize {
        self.points.capacity()
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
