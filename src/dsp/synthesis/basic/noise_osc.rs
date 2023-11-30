use super::*;
use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct NoiseOsc;

impl NoiseOsc {
    pub fn process() -> f64 {
        random_f64().mul_add(2.0, -1.0)
    }
}
