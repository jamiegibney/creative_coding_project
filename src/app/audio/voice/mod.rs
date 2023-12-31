//! Polyphonic voices.

pub mod note;
pub mod voice;

pub use note::{NoteEvent, NoteHandler};
pub use voice::{Voice, VoiceEvent, VoiceHandler};
