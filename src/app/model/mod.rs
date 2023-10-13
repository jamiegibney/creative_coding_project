use nannou_audio::Stream;

use super::audio::*;
use super::view::view;
use super::*;
use crate::musical::*;
use std::sync::{Arc, Mutex};

/// The app's model, i.e. its general state.
pub struct Model {
    window: window::Id,

    pub audio_stream: nannou_audio::Stream<AudioModel>,
    pub audio_senders: AudioSenders,

    pub octave: Octave,
    pub note_handler: NoteHandlerRef,
}

impl Model {
    pub fn new(app: &App) -> Self {
        let window = app
            .new_window()
            .size(1400, 800)
            .key_pressed(key::key_pressed)
            .key_released(key::key_released)
            .mouse_moved(mouse::mouse_moved)
            .view(view)
            .build()
            .expect("failed to build app window!");

        let (audio_stream, audio_senders, note_handler) = build_audio_system();

        Self {
            window,

            audio_stream,
            audio_senders,

            octave: Octave::default(), // C3 - B3

            note_handler: Arc::clone(&note_handler),
        }
    }
}

/// Builds the audio stream, audio message channel senders, and input note
/// handler.
fn build_audio_system() -> (Stream<AudioModel>, AudioSenders, NoteHandlerRef) {
    // setup audio structs
    let note_handler = Arc::new(Mutex::new(NoteHandler::new()));
    let audio_context = AudioContext::build(Arc::clone(&note_handler));
    let mut audio_model = AudioModel::new(audio_context);

    // obtain audio message channels
    let audio_senders = audio_model.initialize();

    // setup audio stream
    let audio_host = nannou_audio::Host::new();
    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio::process)
        .channels(2)
        .sample_rate(unsafe { SAMPLE_RATE } as u32)
        .frames_per_buffer(BUFFER_SIZE)
        .build()
        .unwrap();

    stream.play().unwrap();

    (stream, audio_senders, note_handler)
}
