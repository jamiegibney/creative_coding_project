//! "Utility" processor for basic amplitude control.
use super::Effect;

pub struct AudioUtility {
    gain_db: f64,
    pan: f64,
    width: f64,
    swap_stereo: bool,
}

impl AudioUtility {
    pub fn new() -> Self {
        todo!()
    }
}
