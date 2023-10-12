use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteEvent {
    NoteOn {
        note: u8,
        // timing: u32,
        // id: Option<i32>,
    },
    NoteOff {
        note: u8,
        // timing: u32,
        // id: Option<i32>,
    }
}

impl NoteEvent {
    pub fn note_value(&self) -> u8 {
        match self {
            Self::NoteOn { note, .. } => *note,
            Self::NoteOff { note, .. } => *note,
        }
    }

    pub fn freq_value(&self) -> f64 {
        note_to_freq(self.note_value() as f64)
    }

}
