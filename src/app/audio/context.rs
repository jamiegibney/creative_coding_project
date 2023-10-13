use super::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct AudioContext {
    pub note_handler: NoteHandlerRef,
}

impl AudioContext {
    /// Builds a new `AudioContext`.
    pub fn build(note_handler_ref: NoteHandlerRef) -> Self {
        Self { note_handler: note_handler_ref }
    }

    /// Returns a thread-safe reference to the `NoteHandler`.
    pub fn note_handler_ref(&self) -> NoteHandlerRef {
        Arc::clone(&self.note_handler)
    }
}
