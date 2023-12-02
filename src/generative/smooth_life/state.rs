use super::*;
use std::ops::{Add, Rem};

pub struct SLState {
    pub radius_inner: f64,
    pub radius_outer: f64,

    pub alpha_n: f64,
    pub alpha_m: f64,

    pub b1: f64,
    pub b2: f64,
    pub d1: f64,
    pub d2: f64,

    pub dt: f64,
}

impl SLState {
    pub fn fluid() -> Self {
        Self {
            radius_inner: 10.0 / 3.0,
            radius_outer: 10.0,
            alpha_n: 0.028,
            alpha_m: 0.147,
            b1: 0.238,
            b2: 0.465,
            d1: 0.267,
            d2: 0.445,
            dt: 2.0,
        }
    }

    /// [`Source`](https://arxiv.org/abs/1111.1567)
    pub fn transition(&self, n: f64, m: f64) -> f64 {
        self.sigmoid_n(
            n,
            self.sigmoid_m(self.b1, self.d1, m),
            self.sigmoid_m(self.b2, self.d2, m),
        )
    }

    fn sigmoid(x: f64, a: f64, alpha: f64) -> f64 {
        1.0 / (1.0 + (-(x - a) * 4.0 / alpha).exp())
    }

    fn sigmoid_n(&self, x: f64, a: f64, b: f64) -> f64 {
        Self::sigmoid(x, a, self.alpha_n)
            * (1.0 - Self::sigmoid(x, b, self.alpha_n))
    }

    #[allow(clippy::suboptimal_flops)]
    fn sigmoid_m(&self, x: f64, y: f64, m: f64) -> f64 {
        let sgm = Self::sigmoid(m, 0.5, self.alpha_m);

        x * (1.0 - sgm) + y * sgm
    }
}

const RA: f64 = 11.0;

impl Default for SLState {
    fn default() -> Self {
        Self {
            radius_inner: RA / 3.0,
            radius_outer: RA,

            alpha_n: 0.028,
            alpha_m: 0.147,

            b1: 0.278,
            b2: 0.365,
            d1: 0.267,
            d2: 0.445,

            // dt: 1_000_000.0,
            dt: 0.005,
        }
    }
}
