//! Two-dimensional grid used by the SmoothLife algorithm.

use rand::{self, Rng};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct Grid {
    data: Vec<Vec<f64>>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![vec![0.0; width]; height],
        }
    }

    pub fn new_square(size: usize) -> Self {
        Self::new(size, size)
    }

    pub fn with_random(mut self) -> Self {
        self.randomize();
        self
    }

    /// # Panics
    ///
    /// Panics if the chunk is outside the bounds of the grid, or if `w < x || h < y`.
    pub fn with_random_chunk(mut self, xy: (usize, usize), wh: (usize, usize)) -> Self {
        self.randomize_chunk(xy, wh);
        self
    }

    pub fn with_value(mut self, value: f64) -> Self {
        self.value(value);
        self
    }

    /// # Panics
    ///
    /// Panics if the chunk is outside the bounds of the grid, or if `w < x || h < y`.
    pub fn with_value_chunk(mut self, xy: (usize, usize), wh: (usize, usize), value: f64) -> Self {
        self.value_chunk(xy, wh, value);
        self
    }

    pub fn randomize(&mut self) {
        self.iter_mut()
            .for_each(|v| v.iter_mut().for_each(|x| *x = random_f64()));
    }

    /// # Panics
    ///
    /// Panics if the chunk is outside the bounds of the grid, or if `w < x || h < y`.
    pub fn randomize_chunk(&mut self, xy: (usize, usize), wh: (usize, usize)) {
        let (x, y) = xy;
        let (w, h) = wh;
        assert!(x <= w && y <= h && w <= self.width() && h <= self.height());

        self.iter_mut().skip(x).take(w).for_each(|v| {
            v.iter_mut().skip(y).take(h).for_each(|x| *x = random_f64());
        });
    }

    pub fn value(&mut self, value: f64) {
        self.iter_mut()
            .for_each(|v| v.iter_mut().for_each(|x| *x = value));
    }

    /// # Panics
    ///
    /// Panics if the chunk is outside the bounds of the grid, or if `w < x || h < y`.
    pub fn value_chunk(&mut self, xy: (usize, usize), wh: (usize, usize), value: f64) {
        let (x, y) = xy;
        let (w, h) = wh;
        assert!(x <= w && y <= h && w <= self.width() && h <= self.height());

        self.iter_mut()
            .for_each(|v| v.iter_mut().for_each(|x| *x = value));
    }

    pub fn width(&self) -> usize {
        self.data.len()
    }

    pub fn height(&self) -> usize {
        self.data.first().map_or(0, |v| v.len())
    }
}

impl Deref for Grid {
    type Target = Vec<Vec<f64>>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Grid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub fn random_f64() -> f64 {
    rand::thread_rng().gen_range(0.0..=1.0)
}
