//! Various sequence generators

// TODO:
// I would like to be as generic as possible, allowing for different
// sequence lengths and a variety of patterns. The existing rhythms in rhythm16.rs
// could certainly be used, but it would also be useful to be able to program any
// custom sequence through the API.
//
// In a nutshell, this struct should be called at certain intervals (maybe per-sample?)
// which increments an internal clock, and then the clock should wrap when the sequence
// length is reached. It should output a bool whenever a note should be triggered at a
// given interval.
pub struct Sequence {
    sample_rate: f64,
    clock: u32,
    sequence_length: u32,
    rhythm_interval: f64,
}

impl Sequence {
    pub fn new(sample_rate: f64, length: u32) -> Self {
        Self {
            sample_rate,
            clock: 0,
            sequence_length: length, // TODO: what is this measurement?
            rhythm_interval: todo!(),
        }
    }
}
