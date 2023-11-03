#[derive(Debug, Clone, Copy)]
pub struct TriOsc {
    phase_l: f64,
    phase_r: f64,
    phase_increment: f64,
}

impl TriOsc {
    pub fn new(freq_hz: f64) -> Self {
        todo!()
    }

    pub fn process(&mut self) -> f64 {
        todo!()
    }
}
