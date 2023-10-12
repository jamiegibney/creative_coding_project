use super::audio::*;
use super::view::view;
use super::*;
use crate::musical::*;
use crate::prelude::*;
use std::sync::mpsc;

/// The app's model, i.e. its general state.
pub struct Model {
    pub audio_stream: nannou_audio::Stream<AudioModel>,
    pub audio_senders: AudioSenders,
    _window: window::Id,
    pub note: Option<Note>,
    pub octave: Octave,
}

pub fn model(app: &App) -> Model {
    let _window = app
        .new_window()
        .size(1400, 800)
        .key_released(key::key_released)
        .key_pressed(key::key_pressed)
        .mouse_moved(mouse::mouse_moved)
        .view(view)
        .build()
        .unwrap();

    let audio_host = nannou_audio::Host::new();

    let mut audio_model = AudioModel::new();
    let audio_senders = audio_model.initialize();
    // audio_senders.drive_amount.send(0.5).unwrap();

    let sample_rate = unsafe { SAMPLE_RATE };

    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio::process)
        .channels(2)
        .sample_rate(sample_rate as u32)
        .frames_per_buffer(BUFFER_SIZE)
        .build()
        .unwrap();

    stream.play().unwrap();

    Model {
        audio_stream: stream,
        _window,
        audio_senders,
        note: None,
        octave: Octave::default(), // C3 - B3
    }
}
