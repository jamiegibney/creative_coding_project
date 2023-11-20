#[derive(Debug, Clone, Copy)]
pub struct SquareOsc {
    phase_l: f64,
    phase_r: f64,
    phase_increment: f64,
}

impl SquareOsc {
    pub fn new(_freq_hz: f64) -> Self {
        todo!()
    }

    pub fn process(&mut self) -> f64 {
        todo!()
    }
}
