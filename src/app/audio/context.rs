use super::*;
use crate::app::audio::VoiceEvent;
use std::sync::{mpsc::Receiver, Arc};

// TODO this is a bit of a weird intermediate struct used for building
// and holding data, which could could be extracted elsewhere...
#[derive(Debug)]
pub struct AudioContext {
    pub note_handler: NoteHandlerRef,
    pub sample_rate: f64,
    pub spectral_mask_output: Option<triple_buffer::Output<SpectralMask>>,
    pub voice_event_sender: Sender<VoiceEvent>,
    pub voice_event_receiver: Option<Receiver<VoiceEvent>>,
}

impl AudioContext {
    /// Returns a thread-safe reference to the `NoteHandler`.
    pub fn note_handler_ref(&self) -> NoteHandlerRef {
        Arc::clone(&self.note_handler)
    }
}
