use super::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct AudioContext {
    pub note_handler: NoteHandlerRef,
    pub sample_rate: f64,
}

impl AudioContext {
    /// Builds a new `AudioContext`.
    pub fn build(note_handler_ref: NoteHandlerRef, sample_rate: f64) -> Self {
        Self { note_handler: note_handler_ref, sample_rate }
    }

    /// Returns a thread-safe reference to the `NoteHandler`.
    pub fn note_handler_ref(&self) -> NoteHandlerRef {
        Arc::clone(&self.note_handler)
    }
}
